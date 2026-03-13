use crate::error::Result;
use crate::shared_memory::SharedMemorySystem;
use crate::telemetry::TelemetrySystem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ManagerConfig {
    pub auto_start_enabled: bool,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            auto_start_enabled: true,
        }
    }
}

pub struct AutoManager {
    config: ManagerConfig,
    active: AtomicBool,
}

impl AutoManager {
    pub fn new(config: ManagerConfig) -> Self {
        Self {
            config,
            active: AtomicBool::new(false),
        }
    }

    pub async fn auto_start(
        &self,
        shared_memory: Arc<SharedMemorySystem>,
        telemetry: Option<Arc<TelemetrySystem>>,
    ) -> Result<()> {
        if !self.config.auto_start_enabled {
            return Ok(());
        }

        shared_memory.initialize().await?;
        if let Some(telemetry) = telemetry {
            telemetry
                .record_event(crate::telemetry::TelemetryEvent {
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    event_type: "auto_manager_start".to_string(),
                    component: "auto_manager".to_string(),
                    metrics: serde_json::json!({ "active": true }),
                    tags: vec![("mode".to_string(), "always_on".to_string())],
                })
                .await;
        }
        self.active.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
}
