//! Qdrant-Rust VM Manager
//!
//! Specialized Rust VM for Qdrant vector search with ultra-high performance.
//! Installed on Oracle VM3 (VM.Standard.A1.Flex: ARM 4vCPU, 24GB RAM - FREE TIER)
//! 
//! Performance Targets (with 24GB):
//! - Vector capacity: 10M+ vectors @ 1536 dimensions
//! - P99 search latency: <10ms (vs <20ms with 4GB)
//! - Throughput: 10K+ QPS
//! - Collections: Unlimited (limited only by disk space)

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::time::{interval, Duration};

/// Qdrant VM Configuration
#[derive(Clone, Debug)]
pub struct QdrantVMConfig {
    pub vm_name: String,
    pub ip_address: String,
    pub port: u16,
    pub qdrant_port: u16,
    pub ssh_keypath: String,
    pub qdrant_data_dir: String,
    /// Vector dimensions supported (default: 1536 for OpenAI)
    pub vector_dims: usize,
    /// Collection names to monitor
    pub collections: Vec<String>,
}

impl Default for QdrantVMConfig {
    fn default() -> Self {
        Self {
            vm_name: "vm-qdrant-rust".to_string(),
            ip_address: "0.0.0.0".to_string(), // Will be set from Oracle config
            port: 22,
            qdrant_port: 6333,
            ssh_keypath: String::new(),
            qdrant_data_dir: "/data/qdrant".to_string(),
            vector_dims: 1536,
            collections: vec!["memory_contexts".to_string(), "motor_embeddings".to_string()],
        }
    }
}

/// Qdrant VM Health Status
#[derive(Clone, Debug)]
pub struct QdrantVMStatus {
    pub online: bool,
    pub latency_ms: f64,
    pub uptime_secs: u64,
    pub collections_count: usize,
    pub points_total: u64,
    pub memory_usage_mb: f64,
    pub disk_usage_mb: f64,
    pub replicas: usize,
    pub last_sync: String,
}

impl Default for QdrantVMStatus {
    fn default() -> Self {
        Self {
            online: false,
            latency_ms: 0.0,
            uptime_secs: 0,
            collections_count: 0,
            points_total: 0,
            memory_usage_mb: 0.0,
            disk_usage_mb: 0.0,
            replicas: 0,
            last_sync: "never".to_string(),
        }
    }
}

/// Qdrant Vector Search via Rust VM
#[derive(Clone)]
pub struct QdrantVMManager {
    config: QdrantVMConfig,
    status: Arc<RwLock<QdrantVMStatus>>,
    latency_history: Arc<RwLock<Vec<f64>>>, // Last 100 requests
    is_primary: Arc<RwLock<bool>>, // Use Qdrant-VM or fallback to Redis
}

impl QdrantVMManager {
    pub fn new(config: QdrantVMConfig) -> Self {
        Self {
            config,
            status: Arc::new(RwLock::new(QdrantVMStatus::default())),
            latency_history: Arc::new(RwLock::new(Vec::with_capacity(100))),
            is_primary: Arc::new(RwLock::new(true)),
        }
    }

    /// Initialize Qdrant VM connection
    pub async fn initialize(&self) -> Result<(), String> {
        tracing::info!("Initializing Qdrant VM: {} ({}:{})", 
            self.config.vm_name, self.config.ip_address, self.config.qdrant_port);

        // In production: SSH to VM and verify qdrant service
        // For now: check connectivity
        if self.verify_connectivity().await.is_ok() {
            let mut status = self.status.write().unwrap();
            status.online = true;
            status.last_sync = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            Err(format!("Cannot connect to Qdrant VM at {}", self.config.ip_address))
        }
    }

    /// Verify TCP connectivity to Qdrant VM
    async fn verify_connectivity(&self) -> Result<(), String> {
        // Check if port is open (simplified)
        match tokio::net::TcpStream::connect(
            format!("{}:{}", self.config.ip_address, self.config.qdrant_port)
        ).await {
            Ok(_) => {
                tracing::debug!("✅ Connected to Qdrant VM");
                Ok(())
            }
            Err(e) => {
                tracing::warn!("❌ Cannot connect to Qdrant VM: {}", e);
                Err(e.to_string())
            }
        }
    }

    /// Vector search via Qdrant VM
    pub async fn vector_search(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, f64)>, String> {
        let start = std::time::Instant::now();

        if !*self.is_primary.read().unwrap() {
            return Err("Qdrant VM is not primary (using Redis fallback)".to_string());
        }

        // In production: Use qdrant-client crate to search collection
        // For now: Return mock results
        let latency = start.elapsed().as_secs_f64() * 1000.0;
        self.record_latency(latency);

        if latency > 100.0 {
            tracing::warn!("⚠️ High latency detected: {:.2}ms", latency);
        }

        Ok(vec![
            ("id-1".to_string(), 0.95),
            ("id-2".to_string(), 0.87),
        ])
    }

    /// Health check with metrics
    pub async fn health_check(&self) -> QdrantVMStatus {
        let start = std::time::Instant::now();

        // Verify connectivity
        let is_online = self.verify_connectivity().await.is_ok();
        let latency = start.elapsed().as_secs_f64() * 1000.0;

        let mut status = self.status.write().unwrap();
        status.online = is_online;
        status.latency_ms = latency;
        status.last_sync = chrono::Utc::now().to_rfc3339();

        if !is_online {
            *self.is_primary.write().unwrap() = false;
            tracing::warn!("Qdrant VM offline → Switching to Redis fallback");
        } else {
            *self.is_primary.write().unwrap() = true;
        }

        status.clone()
    }

    /// Record latency for trend analysis
    fn record_latency(&self, latency: f64) {
        let mut history = self.latency_history.write().unwrap();
        history.push(latency);
        if history.len() > 100 {
            history.remove(0);
        }
    }

    /// Get average latency (last 100 requests)
    pub fn avg_latency_ms(&self) -> f64 {
        let history = self.latency_history.read().unwrap();
        if history.is_empty() {
            return 0.0;
        }
        history.iter().sum::<f64>() / history.len() as f64
    }

    /// Get P99 latency
    pub fn p99_latency_ms(&self) -> f64 {
        let mut history = self.latency_history.read().unwrap().clone();
        if history.is_empty() {
            return 0.0;
        }
        history.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p99_idx = (history.len() as f64 * 0.99) as usize;
        history.get(p99_idx).copied().unwrap_or(0.0)
    }

    /// Start background health monitoring
    pub async fn start_monitoring(&self) {
        let manager = self.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(30));
            loop {
                ticker.tick().await;
                let status = manager.health_check().await;
                tracing::debug!("Qdrant VM Health: online={}, latency={:.2}ms, collections={}",
                    status.online, status.latency_ms, status.collections_count);
            }
        });
    }

    /// Get current status
    pub fn status(&self) -> QdrantVMStatus {
        self.status.read().unwrap().clone()
    }

    /// Is Qdrant VM primary?
    pub fn is_primary(&self) -> bool {
        *self.is_primary.read().unwrap()
    }

    /// Force fallback to Redis
    pub fn fallback_to_redis(&self) {
        *self.is_primary.write().unwrap() = false;
        tracing::warn!("🔄 Forced fallback to Redis cache");
    }

    /// Attempt recovery
    pub async fn attempt_recovery(&self) -> Result<(), String> {
        tracing::info!("🔧 Attempting Qdrant VM recovery...");
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        self.verify_connectivity().await?;
        *self.is_primary.write().unwrap() = true;
        tracing::info!("✅ Qdrant VM recovered");
        Ok(())
    }
}

// Implement Send + Sync for Tokio integration
unsafe impl Send for QdrantVMManager {}
unsafe impl Sync for QdrantVMManager {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = QdrantVMConfig::default();
        assert_eq!(config.qdrant_port, 6333);
        assert_eq!(config.vector_dims, 1536);
        assert!(!config.collections.is_empty());
    }

    #[test]
    fn test_status_default() {
        let status = QdrantVMStatus::default();
        assert!(!status.online);
        assert_eq!(status.latency_ms, 0.0);
    }

    #[tokio::test]
    async fn test_latency_recording() {
        let config = QdrantVMConfig::default();
        let manager = QdrantVMManager::new(config);
        
        manager.record_latency(10.0);
        manager.record_latency(20.0);
        manager.record_latency(30.0);
        
        let avg = manager.avg_latency_ms();
        assert!((avg - 20.0).abs() < 0.1);
    }
}
