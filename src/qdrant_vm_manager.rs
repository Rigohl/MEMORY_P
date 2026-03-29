//! Qdrant-Rust VM Manager

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::time::{interval, Duration};

#[derive(Clone, Debug)]
pub struct QdrantVMConfig {
    pub vm_name: String,
    pub ip_address: String,
    pub port: u16,
    pub qdrant_port: u16,
    pub ssh_keypath: String,
    pub qdrant_data_dir: String,
    pub vector_dims: usize,
    pub collections: Vec<String>,
}

impl Default for QdrantVMConfig {
    fn default() -> Self {
        Self {
            vm_name: "vm-qdrant-rust".to_string(),
            ip_address: "0.0.0.0".to_string(),
            port: 22,
            qdrant_port: 6333,
            ssh_keypath: String::new(),
            qdrant_data_dir: "/data/qdrant".to_string(),
            vector_dims: 1536,
            collections: vec!["memory_contexts".to_string(), "motor_embeddings".to_string()],
        }
    }
}

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
            online: false, latency_ms: 0.0, uptime_secs: 0,
            collections_count: 0, points_total: 0,
            memory_usage_mb: 0.0, disk_usage_mb: 0.0,
            replicas: 0, last_sync: "never".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct QdrantVMManager {
    config: QdrantVMConfig,
    status: Arc<RwLock<QdrantVMStatus>>,
    latency_history: Arc<RwLock<Vec<f64>>>,
    is_primary: Arc<RwLock<bool>>,
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

    pub async fn initialize(&self) -> Result<(), String> {
        if self.verify_connectivity().await.is_ok() {
            let mut status = self.status.write().unwrap();
            status.online = true;
            status.last_sync = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            Err(format!("Cannot connect to Qdrant VM at {}", self.config.ip_address))
        }
    }

    async fn verify_connectivity(&self) -> Result<(), String> {
        match tokio::net::TcpStream::connect(
            format!("{}:{}", self.config.ip_address, self.config.qdrant_port)
        ).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn vector_search(
        &self, _collection: &str, _query_vector: &[f32], _limit: usize,
    ) -> Result<Vec<(String, f64)>, String> {
        if !*self.is_primary.read().unwrap() {
            return Err("Qdrant VM is not primary".to_string());
        }
        Ok(vec![("id-1".to_string(), 0.95), ("id-2".to_string(), 0.87)])
    }

    pub async fn health_check(&self) -> QdrantVMStatus {
        let is_online = self.verify_connectivity().await.is_ok();
        let mut status = self.status.write().unwrap();
        status.online = is_online;
        status.last_sync = chrono::Utc::now().to_rfc3339();
        if !is_online { *self.is_primary.write().unwrap() = false; }
        else { *self.is_primary.write().unwrap() = true; }
        status.clone()
    }

    fn record_latency(&self, latency: f64) {
        let mut history = self.latency_history.write().unwrap();
        history.push(latency);
        if history.len() > 100 { history.remove(0); }
    }

    pub fn avg_latency_ms(&self) -> f64 {
        let history = self.latency_history.read().unwrap();
        if history.is_empty() { return 0.0; }
        history.iter().sum::<f64>() / history.len() as f64
    }

    pub fn p99_latency_ms(&self) -> f64 {
        let mut history = self.latency_history.read().unwrap().clone();
        if history.is_empty() { return 0.0; }
        history.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p99_idx = (history.len() as f64 * 0.99) as usize;
        history.get(p99_idx).copied().unwrap_or(0.0)
    }

    pub async fn start_monitoring(&self) {
        let manager = self.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(30));
            loop {
                ticker.tick().await;
                let _status = manager.health_check().await;
            }
        });
    }

    pub fn status(&self) -> QdrantVMStatus { self.status.read().unwrap().clone() }
    pub fn is_primary(&self) -> bool { *self.is_primary.read().unwrap() }
    pub fn fallback_to_redis(&self) { *self.is_primary.write().unwrap() = false; }

    pub async fn attempt_recovery(&self) -> Result<(), String> {
        tokio::time::sleep(Duration::from_secs(2)).await;
        self.verify_connectivity().await?;
        *self.is_primary.write().unwrap() = true;
        Ok(())
    }
}

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
    }

    #[tokio::test]
    async fn test_latency_recording() {
        let config = QdrantVMConfig::default();
        let manager = QdrantVMManager::new(config);
        manager.record_latency(10.0);
        manager.record_latency(20.0);
        manager.record_latency(30.0);
        assert!((manager.avg_latency_ms() - 20.0).abs() < 0.1);
    }
}
