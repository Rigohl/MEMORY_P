//! Motor Health Monitoring
//! Tracks health status of all 9 search engines

use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct HealthMonitor {
    statuses: Arc<parking_lot::Mutex<HashMap<String, bool>>>,
}

impl HealthMonitor {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            statuses: Arc::new(parking_lot::Mutex::new(HashMap::new())),
        }
    }

    #[allow(dead_code)]
    pub fn register_engine(&self, name: String) {
        self.statuses.lock().insert(name, true);
    }

    #[allow(dead_code)]
    pub fn is_healthy(&self, name: &str) -> bool {
        self.statuses.lock().get(name).copied().unwrap_or(false)
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}
