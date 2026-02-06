//! auto_rebuild.rs - Sistema FORCED_REBUILDS
//! Auto-ajuste de módulos sin intervención manual

use crate::error::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Módulo que puede ser reconstruido
#[derive(Debug, Clone)]
pub struct RebuildableModule {
    pub name: String,
    pub active: bool,
    pub priority: u8,
    pub last_rebuild: std::time::Instant,
}

/// Sistema de auto-rebuild
pub struct AutoRebuild {
    modules: Arc<RwLock<Vec<RebuildableModule>>>,
    interval: u64,
    running: Arc<RwLock<bool>>,
}

impl AutoRebuild {
    pub fn new(interval: u64) -> Self {
        Self {
            modules: Arc::new(RwLock::new(vec![
                RebuildableModule {
                    name: "deepweb_tor".to_string(),
                    active: false,
                    priority: 3,
                    last_rebuild: std::time::Instant::now(),
                },
                RebuildableModule {
                    name: "intelligent_storage".to_string(),
                    active: true,
                    priority: 5,
                    last_rebuild: std::time::Instant::now(),
                },
                RebuildableModule {
                    name: "predictive_nodes".to_string(),
                    active: true,
                    priority: 4,
                    last_rebuild: std::time::Instant::now(),
                },
                RebuildableModule {
                    name: "deep_storage_tunnels".to_string(),
                    active: true,
                    priority: 4,
                    last_rebuild: std::time::Instant::now(),
                },
            ])),
            interval,
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("🔄 Iniciando sistema FORCED_REBUILDS");

        let mut running = self.running.write().await;
        if *running {
            warn!("AutoRebuild ya está ejecutándose");
            return Ok(());
        }
        *running = true;
        drop(running);

        // Iniciar task de auto-rebuild
        let modules = self.modules.clone();
        let interval = self.interval;
        let running = self.running.clone();

        tokio::spawn(async move {
            info!("🔧 FORCED_REBUILDS iniciado (cada {} segundos)", interval);

            loop {
                if !*running.read().await {
                    break;
                }

                // Verificar módulos que necesitan rebuild
                let mut modules_guard = modules.write().await;
                for module in modules_guard.iter_mut() {
                    let elapsed = module.last_rebuild.elapsed().as_secs();

                    // Rebuild si han pasado más del intervalo
                    if elapsed > interval {
                        info!(
                            "🔨 FORCED_REBUILD: módulo '{}' (prioridad: {})",
                            module.name, module.priority
                        );

                        // Simular rebuild
                        module.last_rebuild = std::time::Instant::now();

                        // Auto-ajustar estado basado en métricas
                        // En implementación real: analizar métricas y ajustar
                    }
                }
                drop(modules_guard);

                tokio::time::sleep(Duration::from_secs(60)).await;
            }

            info!("🔧 FORCED_REBUILDS detenido");
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }

    /// Fuerza rebuild de un módulo específico
    pub async fn force_rebuild(&self, module_name: &str) -> Result<()> {
        let mut modules = self.modules.write().await;

        if let Some(module) = modules.iter_mut().find(|m| m.name == module_name) {
            info!("🔨 FORCED_REBUILD manual: módulo '{}'", module_name);
            module.last_rebuild = std::time::Instant::now();
            Ok(())
        } else {
            Err(crate::error::MemoryPError::Other(format!(
                "Módulo '{}' no encontrado",
                module_name
            )))
        }
    }

    /// Activa/desactiva un módulo
    pub async fn toggle_module(&self, module_name: &str, active: bool) -> Result<()> {
        let mut modules = self.modules.write().await;

        if let Some(module) = modules.iter_mut().find(|m| m.name == module_name) {
            module.active = active;
            info!(
                "⚙️  Módulo '{}' -> {}",
                module_name,
                if active { "ACTIVO" } else { "INACTIVO" }
            );
            Ok(())
        } else {
            Err(crate::error::MemoryPError::Other(format!(
                "Módulo '{}' no encontrado",
                module_name
            )))
        }
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let modules = self.modules.blocking_read();
        let active_count = modules.iter().filter(|m| m.active).count();

        serde_json::json!({
            "total_modules": modules.len(),
            "active_modules": active_count,
            "interval_seconds": self.interval,
        })
    }
}
