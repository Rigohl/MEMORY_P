//! shared_memory/cleanup.rs - Gestor de limpieza automática

use super::types::{SharedContext, AgentId, ContextId};
use super::context::ContextManager;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};
use tracing::{info, debug, warn};

/// Gestor de limpieza automática de memoria
/// Elimina contextos inactivos y libera recursos no utilizados
pub struct CleanupManager {
    /// Indica si el gestor está activo
    active: Arc<AtomicBool>,
    
    /// Intervalo de limpieza en segundos
    cleanup_interval_secs: u64,
    
    /// Edad máxima de contextos inactivos en segundos
    max_inactive_age_secs: u64,
}

impl CleanupManager {
    /// Crea un nuevo gestor de limpieza
    pub fn new() -> Self {
        Self {
            active: Arc::new(AtomicBool::new(false)),
            cleanup_interval_secs: 300, // 5 minutos
            max_inactive_age_secs: 3600, // 1 hora
        }
    }
    
    /// Crea un gestor con configuración personalizada
    pub fn with_config(cleanup_interval_secs: u64, max_inactive_age_secs: u64) -> Self {
        Self {
            active: Arc::new(AtomicBool::new(false)),
            cleanup_interval_secs,
            max_inactive_age_secs,
        }
    }
    
    /// Inicia la limpieza automática
    pub async fn start(
        &self,
        context_manager: Arc<ContextManager>,
        active_contexts: Arc<DashMap<AgentId, SharedContext>>,
    ) {
        if self.active.load(Ordering::Acquire) {
            warn!("⚠️  Gestor de limpieza ya está activo");
            return;
        }
        
        info!("🔧 Iniciando gestor de limpieza automática");
        self.active.store(true, Ordering::Release);
        
        let active = Arc::clone(&self.active);
        let cleanup_interval = self.cleanup_interval_secs;
        let max_inactive_age = self.max_inactive_age_secs;
        
        tokio::spawn(async move {
            while active.load(Ordering::Acquire) {
                sleep(Duration::from_secs(cleanup_interval)).await;
                
                debug!("🧹 Ejecutando limpieza automática");
                
                let current_time = current_timestamp();
                let mut cleaned = 0;
                
                // Buscar contextos inactivos
                let contexts_to_clean: Vec<ContextId> = active_contexts
                    .iter()
                    .filter_map(|entry| {
                        let context = entry.value();
                        let age = current_time - context.metadata.last_accessed;
                        
                        if age > max_inactive_age as i64 {
                            Some(context.context_id.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                
                // Eliminar contextos inactivos
                for context_id in contexts_to_clean {
                    if let Err(e) = context_manager.delete(&context_id).await {
                        warn!("Error eliminando contexto {}: {}", context_id, e);
                    } else {
                        cleaned += 1;
                    }
                }
                
                if cleaned > 0 {
                    info!("🧹 Limpiados {} contextos inactivos", cleaned);
                }
            }
        });
        
        info!("✅ Gestor de limpieza automática iniciado");
    }
    
    /// Limpia contextos inactivos manualmente
    pub async fn cleanup_inactive(&self, max_age_secs: u64) -> Result<usize, crate::error::MemoryPError> {
        debug!("🧹 Limpiando contextos inactivos (edad > {} segundos)", max_age_secs);
        
        // TODO: Implementar limpieza manual
        // Por ahora, retornar 0
        Ok(0)
    }
    
    /// Detiene el gestor de limpieza
    pub async fn stop(&self) {
        info!("🔧 Deteniendo gestor de limpieza");
        self.active.store(false, Ordering::Release);
        
        // Esperar a que termine la tarea de limpieza
        sleep(Duration::from_millis(100)).await;
        
        info!("✅ Gestor de limpieza detenido");
    }
    
    /// Indica si el gestor está activo
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }
}

impl Default for CleanupManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Obtiene timestamp actual en segundos
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_memory::context::ContextManager;
    
    #[tokio::test]
    async fn test_cleanup_manager_creation() {
        let manager = CleanupManager::new();
        assert!(!manager.is_active());
    }
    
    #[tokio::test]
    async fn test_cleanup_manager_start_stop() {
        let manager = CleanupManager::new();
        let context_manager = Arc::new(ContextManager::new().await.unwrap());
        let active_contexts = Arc::new(DashMap::new());
        
        manager.start(context_manager, active_contexts).await;
        assert!(manager.is_active());
        
        manager.stop().await;
        assert!(!manager.is_active());
    }
    
    #[test]
    fn test_cleanup_with_custom_config() {
        let manager = CleanupManager::with_config(60, 300);
        assert_eq!(manager.cleanup_interval_secs, 60);
        assert_eq!(manager.max_inactive_age_secs, 300);
    }
}
