//! health_monitor.rs - 24/7 System Health Monitoring
//! 
//! PRESERVES: Existing monitoring.rs logic from autonomous.rs
//! EXTENDS: With motor-specific + FFI-specific health checks

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitor {
    motor_health: HashMap<String, MotorHealthStatus>,
    ffi_health: HashMap<String, FFIHealthStatus>,
    last_update: DateTime<Utc>,
    overall_health: f64, // 0-100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorHealthStatus {
    pub name: String,
    pub online: bool,
    pub latency_avg_ms: f64,
    pub uptime_secs: u64,
    pub error_count: u32,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FFIHealthStatus {
    pub name: String,
    pub healthy: bool,
    pub fallback_active: bool,
    pub native_available: bool,
    pub last_call_ms: f64,
    pub error_count: u32,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            motor_health: HashMap::new(),
            ffi_health: HashMap::new(),
            last_update: Utc::now(),
            overall_health: 100.0,
        }
    }
    
    pub fn update_motor(&mut self, name: &str, status: MotorStatus) {
        let health = MotorHealthStatus {
            name: name.to_string(),
            online: status.is_healthy,
            latency_avg_ms: status.latency_ms,
            uptime_secs: 0,
            error_count: 0,
            last_check: Utc::now(),
        };
        self.motor_health.insert(name.to_string(), health);
        self.last_update = Utc::now();
    }
    
    pub fn update_ffi(&mut self, name: &str, status: FFIStatus) {
        let health = FFIHealthStatus {
            name: name.to_string(),
            healthy: status.is_healthy,
            fallback_active: status.fallback_active,
            native_available: status.native_available,
            last_call_ms: 0.0,
            error_count: 0,
        };
        self.ffi_health.insert(name.to_string(), health);
    }
    
    pub fn calculate_health_percentage(&mut self) -> f64 {
        let motor_count = self.motor_health.len() as f64;
        let ffi_count = self.ffi_health.len() as f64;
        
        if motor_count == 0.0 && ffi_count == 0.0 {
            return 100.0;
        }
        
        let motor_health: f64 = self.motor_health.values()
            .map(|m| if m.online { 100.0 } else { 0.0 })
            .sum::<f64>() / motor_count.max(1.0);
        
        let ffi_health: f64 = self.ffi_health.values()
            .map(|f| if f.healthy { 100.0 } else { 50.0 })
            .sum::<f64>() / ffi_count.max(1.0);
        
        let overall = (motor_health * 0.7 + ffi_health * 0.3);
        self.overall_health = overall;
        overall
    }
    
    pub fn get_unhealthy_motors(&self) -> Vec<String> {
        self.motor_health.iter()
            .filter(|(_, h)| !h.online)
            .map(|(k, _)| k.clone())
            .collect()
    }
    
    pub fn get_unhealthy_ffis(&self) -> Vec<String> {
        self.ffi_health.iter()
            .filter(|(_, h)| !h.healthy)
            .map(|(k, _)| k.clone())
            .collect()
    }
}

pub struct MotorStatus {
    pub is_healthy: bool,
    pub latency_ms: f64,
    pub qps: f64,
    pub error_rate: f64,
}

pub struct FFIStatus {
    pub is_healthy: bool,
    pub native_available: bool,
    pub fallback_active: bool,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_health_monitor() {
        let mut monitor = HealthMonitor::new();
        let motor_status = MotorStatus {
            is_healthy: true,
            latency_ms: 15.0,
            qps: 1000.0,
            error_rate: 0.01,
        };
        monitor.update_motor("qdrant", motor_status);
        assert_eq!(monitor.motor_health.len(), 1);
    }
}
