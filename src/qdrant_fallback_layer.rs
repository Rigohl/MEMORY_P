//! Qdrant VM ←→ Redis Fallback Layer

use std::sync::Arc;
use tokio::sync::RwLock;
use std::future::Future;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum FallbackStrategy {
    Primary,
    Fallback,
    Adaptive,
}

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
            max_failures_before_fallback: 3,
        }
    }

    pub async fn execute_with_fallback<T, F1, F2>(
        &self, qdrant_op: F1, redis_op: F2,
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
                        *self.qdrant_failures.write().await = 0;
                        Ok(result)
                    }
                    Err(e) => {
                        let mut failures = self.qdrant_failures.write().await;
                        *failures += 1;
                        if *failures >= self.max_failures_before_fallback {
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
                *self.redis_fallback_count.write().await += 1;
                redis_op.await
            }
            FallbackStrategy::Adaptive => {
                let qdrant_result = tokio::time::timeout(
                    std::time::Duration::from_millis(100), qdrant_op,
                ).await;
                match qdrant_result {
                    Ok(Ok(result)) => {
                        *self.qdrant_failures.write().await = 0;
                        Ok(result)
                    }
                    Ok(Err(_)) | Err(_) => {
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

    pub async fn attempt_qdrant_recovery(&self) {
        *self.qdrant_failures.write().await = 0;
        *self.current_strategy.write().await = FallbackStrategy::Primary;
    }

    pub async fn get_strategy(&self) -> FallbackStrategy { *self.current_strategy.read().await }
    pub async fn get_stats(&self) -> (u32, u64) {
        (*self.qdrant_failures.read().await, *self.redis_fallback_count.read().await)
    }
    pub async fn force_fallback_to_redis(&self) { *self.current_strategy.write().await = FallbackStrategy::Fallback; }
    pub async fn force_back_to_qdrant(&self) {
        *self.current_strategy.write().await = FallbackStrategy::Primary;
        *self.qdrant_failures.write().await = 0;
    }
}

impl Default for QdrantFallbackLayer {
    fn default() -> Self { Self::new() }
}

unsafe impl Send for QdrantFallbackLayer {}
unsafe impl Sync for QdrantFallbackLayer {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_strategy_switching() {
        let layer = QdrantFallbackLayer::new();
        assert_eq!(layer.get_strategy().await, FallbackStrategy::Primary);
        layer.force_fallback_to_redis().await;
        assert_eq!(layer.get_strategy().await, FallbackStrategy::Fallback);
    }
}
