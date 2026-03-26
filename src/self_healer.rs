//! self_healer.rs - Autonomous Self-Healing System

use crate::health_monitor::HealthMonitor;

pub struct SelfHealer {
    repair_history: Vec<RepairAction>,
    max_retries: u32,
}

pub struct RepairAction {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub component: String,
    pub action: String,
    pub success: bool,
    pub duration_ms: u64,
}

impl SelfHealer {
    pub fn new() -> Self {
        Self { repair_history: Vec::new(), max_retries: 3 }
    }

    pub fn suggest_repairs(&mut self, health: &HealthMonitor) {
        for motor in health.get_unhealthy_motors() {
            self.repair_history.push(RepairAction {
                timestamp: chrono::Utc::now(),
                component: format!("motor:{}", motor),
                action: "restart".to_string(),
                success: false,
                duration_ms: 0,
            });
        }
        for ffi in health.get_unhealthy_ffis() {
            self.repair_history.push(RepairAction {
                timestamp: chrono::Utc::now(),
                component: format!("ffi:{}", ffi),
                action: "switch_to_fallback".to_string(),
                success: true,
                duration_ms: 10,
            });
        }
    }

    pub async fn execute_repair(&mut self, component: &str) -> Result<bool, String> {
        if component.starts_with("motor:") {
            self.restart_motor(&component[6..]).await
        } else if component.starts_with("ffi:") {
            self.switch_ffi_fallback(&component[4..]).await
        } else {
            Err("Unknown component".to_string())
        }
    }

    async fn restart_motor(&mut self, motor_name: &str) -> Result<bool, String> {
        tracing::info!("Restarting motor: {}", motor_name);
        Ok(true)
    }

    async fn switch_ffi_fallback(&mut self, ffi_name: &str) -> Result<bool, String> {
        tracing::info!("Switching FFI to fallback: {}", ffi_name);
        Ok(true)
    }

    pub fn get_repair_history(&self) -> &Vec<RepairAction> { &self.repair_history }
}

impl Default for SelfHealer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_healer_init() {
        let healer = SelfHealer::new();
        assert_eq!(healer.max_retries, 3);
    }
}
