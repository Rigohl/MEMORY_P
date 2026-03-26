//! Qdrant VM ←→ Redis Fallback Layer
//!
//! PRIMARY: Qdrant-VM-Rust (low latency, specialized vector search)
//! FALLBACK: Redis (if Qdrant-VM offline)
//! Automatic failover + recovery

use std::sync::Arc;
use tokio::sync::RwLock;
use std::future::Future;

/// Fallback Strategy
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum FallbackStrategy {
    /// Use Qdrant-VM (primary)
    Primary,
    /// Use Redis (fallback after 3 failed attempts)
    Fallback,
    /// Try Qdrant, fall back to Redis if timeout
    Adaptive,
}

/// Qdrant ↔ Redis Coordination
pub struct QdrantFallbackLayer {
    pub current_strategy: Arc<RwLock<FallbackStrategy>>,
    pub qdrant_failures: Arc<RwLock<u32>>,
    pub redis_fallback_count: Arc<RwLock<u64>>,
    pub max_failures_before_fallback: u32,
}

impl QdrantFallbackLayer {
    pub fn new() -> Self {
        Self {
            current_strategy: Arc::new(RwLock::new(FallbackStrategy::Primary)),
            qdrant_failures: Arc::new(RwLock::new(0)),
            redis_fallback_count: Arc::new(RwLock::new(0)),
            max_failures_before_fallback: 3, // Fail 3 times, then fallback to Redis
        }
    }

    /// Execute with fallback strategy
    pub async fn execute_with_fallback<T, F1, F2>(
        &self,
        qdrant_op: F1,
        redis_op: F2,
    ) -> Result<T, String>
    where
        T: Send + 'static,
        F1: Future<Output = Result<T, String>> + Send,
        F2: Future<Output = Result<T, String>> + Send,
    {
        let strategy = *self.current_strategy.read().await;

        match strategy {
            FallbackStrategy::Primary => {
                match qdrant_op.await {
                    Ok(result) => {
                        // Reset failure count on success
                        *self.qdrant_failures.write().await = 0;
                        Ok(result)
                    }
                    Err(e) => {
                        let mut failures = self.qdrant_failures.write().await;
                        *failures += 1;

                        if *failures >= self.max_failures_before_fallback {
                            tracing::warn!(
                                "Qdrant-VM failed {} times, switching to Redis",
                                failures
                            );
                            *self.current_strategy.write().await = FallbackStrategy::Fallback;
                            *self.redis_fallback_count.write().await += 1;
                            redis_op.await
                        } else {
                            Err(format!("Qdrant error (attempt {}): {}", failures, e))
                        }
                    }
                }
            }
            FallbackStrategy::Fallback => {
                tracing::debug!("Using Redis fallback (Qdrant offline)");
                *self.redis_fallback_count.write().await += 1;
                redis_op.await
            }
            FallbackStrategy::Adaptive => {
                // Try Qdrant first with timeout
                let qdrant_result = tokio::time::timeout(
                    std::time::Duration::from_millis(100), // 100ms timeout
                    qdrant_op,
                )
                .await;

                match qdrant_result {
                    Ok(Ok(result)) => {
                        *self.qdrant_failures.write().await = 0;
                        Ok(result)
                    }
                    Ok(Err(_)) | Err(_) => {
                        // Timeout or error → fallback to Redis
                        let mut failures = self.qdrant_failures.write().await;
                        *failures += 1;

                        if *failures >= self.max_failures_before_fallback {
                            *self.current_strategy.write().await = FallbackStrategy::Fallback;
                        }

                        redis_op.await
                    }
                }
            }
        }
    }

    /// Try to recover Qdrant-VM
    pub async fn attempt_qdrant_recovery(&self) {
        tracing::info!("🔧 Attempting Qdrant-VM recovery...");
        *self.qdrant_failures.write().await = 0;
        *self.current_strategy.write().await = FallbackStrategy::Primary;
        tracing::info!("✅ Switched back to Qdrant-VM primary");
    }

    /// Get current strategy
    pub async fn get_strategy(&self) -> FallbackStrategy {
        *self.current_strategy.read().await
    }

    /// Get fallback statistics
    pub async fn get_stats(&self) -> (u32, u64) {
        let failures = *self.qdrant_failures.read().await;
        let fallbacks = *self.redis_fallback_count.read().await;
        (failures, fallbacks)
    }

    /// Force fallback to Redis (admin operation)
    pub async fn force_fallback_to_redis(&self) {
        tracing::warn!("⚠️  Forcing fallback to Redis");
        *self.current_strategy.write().await = FallbackStrategy::Fallback;
    }

    /// Force back to Qdrant (after manual recovery)
    pub async fn force_back_to_qdrant(&self) {
        tracing::info!("🔄 Force enabling Qdrant-VM");
        *self.current_strategy.write().await = FallbackStrategy::Primary;
        *self.qdrant_failures.write().await = 0;
    }
}

impl Default for QdrantFallbackLayer {
    fn default() -> Self {
        Self::new()
    }
}

// Thread safe
unsafe impl Send for QdrantFallbackLayer {}
unsafe impl Sync for QdrantFallbackLayer {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fallback_on_error() {
        let layer = QdrantFallbackLayer::new();

        let qdrant_op = async { Err::<String, _>("Connection error".to_string()) };
        let redis_op = async { Ok::<String, _>("Redis cached".to_string()) };

        // First attempt should fail (but record it)
        let _ = layer
            .execute_with_fallback(qdrant_op, redis_op)
            .await;

        let (failures, _) = layer.get_stats().await;
        assert_eq!(failures, 1);
    }

    #[tokio::test]
    async fn test_strategy_switching() {
        let layer = QdrantFallbackLayer::new();

        assert_eq!(layer.get_strategy().await, FallbackStrategy::Primary);

        layer.force_fallback_to_redis().await;
        assert_eq!(layer.get_strategy().await, FallbackStrategy::Fallback);

        layer.force_back_to_qdrant().await;
        assert_eq!(layer.get_strategy().await, FallbackStrategy::Primary);
    }
}
