//! Health monitoring for all search engines
//!
//! Provides centralized health checking and monitoring for all 9 engines

use super::traits::SearchEngine;
use super::types::EngineHealth;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Health monitor for all engines
pub struct HealthMonitor {
    /// Registered engines
    engines: Arc<RwLock<HashMap<String, Arc<dyn SearchEngine>>>>,
    /// Last health check results
    last_health: Arc<RwLock<HashMap<String, EngineHealth>>>,
    /// Health check interval
    check_interval: Duration,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(check_interval: Duration) -> Self {
        Self {
            engines: Arc::new(RwLock::new(HashMap::new())),
            last_health: Arc::new(RwLock::new(HashMap::new())),
            check_interval,
        }
    }

    /// Register an engine for monitoring
    pub async fn register_engine(&self, name: String, engine: Arc<dyn SearchEngine>) {
        let mut engines = self.engines.write().await;
        engines.insert(name, engine);
    }

    /// Unregister an engine
    pub async fn unregister_engine(&self, name: &str) {
        let mut engines = self.engines.write().await;
        engines.remove(name);
    }

    /// Check health of all engines
    pub async fn check_all(&self) -> HashMap<String, EngineHealth> {
        let engines = self.engines.read().await;
        let mut results = HashMap::new();

        for (name, engine) in engines.iter() {
            match engine.health().await {
                Ok(health) => {
                    results.insert(name.clone(), health);
                }
                Err(e) => {
                    let error_health = EngineHealth {
                        engine: name.clone(),
                        healthy: false,
                        status: format!("Error: {}", e),
                        last_check: Self::current_timestamp(),
                        details: HashMap::new(),
                    };
                    results.insert(name.clone(), error_health);
                }
            }
        }

        // Update last health cache
        let mut last_health = self.last_health.write().await;
        *last_health = results.clone();

        results
    }

    /// Check health of a specific engine
    pub async fn check_engine(&self, name: &str) -> Option<EngineHealth> {
        let engines = self.engines.read().await;
        if let Some(engine) = engines.get(name) {
            match engine.health().await {
                Ok(health) => {
                    // Update cache
                    let mut last_health = self.last_health.write().await;
                    last_health.insert(name.to_string(), health.clone());
                    Some(health)
                }
                Err(e) => {
                    let error_health = EngineHealth {
                        engine: name.to_string(),
                        healthy: false,
                        status: format!("Error: {}", e),
                        last_check: Self::current_timestamp(),
                        details: HashMap::new(),
                    };
                    // Update cache
                    let mut last_health = self.last_health.write().await;
                    last_health.insert(name.to_string(), error_health.clone());
                    Some(error_health)
                }
            }
        } else {
            None
        }
    }

    /// Get last known health status
    pub async fn get_cached_health(&self, name: &str) -> Option<EngineHealth> {
        let last_health = self.last_health.read().await;
        last_health.get(name).cloned()
    }

    /// Get health of all engines from cache
    pub async fn get_all_cached_health(&self) -> HashMap<String, EngineHealth> {
        let last_health = self.last_health.read().await;
        last_health.clone()
    }

    /// Check if an engine is healthy
    pub async fn is_healthy(&self, name: &str) -> bool {
        self.check_engine(name)
            .await
            .map(|h| h.healthy)
            .unwrap_or(false)
    }

    /// Get list of all registered engine names
    pub async fn get_engine_names(&self) -> Vec<String> {
        let engines = self.engines.read().await;
        engines.keys().cloned().collect()
    }

    /// Get overall system health
    pub async fn get_system_health(&self) -> SystemHealth {
        let results = self.check_all().await;
        let total = results.len();
        let healthy = results.values().filter(|h| h.healthy).count();
        let unhealthy = total - healthy;

        SystemHealth {
            total_engines: total,
            healthy_engines: healthy,
            unhealthy_engines: unhealthy,
            health_percentage: if total > 0 {
                (healthy as f64 / total as f64) * 100.0
            } else {
                0.0
            },
            engine_status: results,
            timestamp: Self::current_timestamp(),
        }
    }

    /// Start background health checking
    pub fn start_background_checks(self: Arc<Self>) {
        let monitor = Arc::clone(&self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(monitor.check_interval);
            loop {
                interval.tick().await;
                let _ = monitor.check_all().await;
            }
        });
    }

    /// Get current timestamp
    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }
}

/// Overall system health status
#[derive(Debug, Clone)]
pub struct SystemHealth {
    pub total_engines: usize,
    pub healthy_engines: usize,
    pub unhealthy_engines: usize,
    pub health_percentage: f64,
    pub engine_status: HashMap<String, EngineHealth>,
    pub timestamp: i64,
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_monitor_creation() {
        let monitor = HealthMonitor::new(Duration::from_secs(60));
        let engines = monitor.get_engine_names().await;
        assert_eq!(engines.len(), 0);
    }

    #[tokio::test]
    async fn test_system_health_empty() {
        let monitor = HealthMonitor::new(Duration::from_secs(60));
        let health = monitor.get_system_health().await;
        assert_eq!(health.total_engines, 0);
        assert_eq!(health.health_percentage, 0.0);
    }
}
