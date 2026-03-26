//! intelligent_storage.rs - Almacenamiento inteligente con expansión dinámica
//! Persistencia que se expande según actividades prioritarias

use crate::error::Result;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Prioridad de almacenamiento
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StoragePriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Item almacenado
#[derive(Debug, Clone)]
pub struct StorageItem {
    pub key: String,
    pub data: Vec<u8>,
    pub priority: StoragePriority,
    pub access_count: u64,
    pub created_at: std::time::Instant,
}

/// Sistema de almacenamiento inteligente
pub struct IntelligentStorage {
    items: Arc<DashMap<String, StorageItem>>,
    capacity: Arc<RwLock<usize>>,
    running: Arc<RwLock<bool>>,
}

impl Default for IntelligentStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl IntelligentStorage {
    pub fn new() -> Self {
        Self {
            items: Arc::new(DashMap::new()),
            capacity: Arc::new(RwLock::new(1024 * 1024 * 100)), // 100MB inicial
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("💾 Iniciando Intelligent Storage...");

        let mut running = self.running.write().await;
        if *running {
            warn!("Intelligent Storage ya está ejecutándose");
            return Ok(());
        }
        *running = true;
        drop(running);

        // Iniciar task de auto-expansión
        self.start_auto_expansion().await;

        info!("✅ Intelligent Storage iniciado");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Deteniendo Intelligent Storage...");

        let mut running = self.running.write().await;
        *running = false;

        info!("✅ Intelligent Storage detenido");
        Ok(())
    }

    /// Auto-expansión basada en uso
    async fn start_auto_expansion(&self) {
        let items = self.items.clone();
        let capacity = self.capacity.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            loop {
                if !*running.read().await {
                    break;
                }

                // Analizar uso actual
                let total_size: usize = items.iter().map(|e| e.value().data.len()).sum();
                let current_capacity = *capacity.read().await;
                let usage_percent = (total_size as f64 / current_capacity as f64) * 100.0;

                // Expandir si uso > 80%
                if usage_percent > 80.0 {
                    let new_capacity = current_capacity * 2;
                    *capacity.write().await = new_capacity;
                    info!(
                        "📈 Almacenamiento expandido: {} MB -> {} MB",
                        current_capacity / 1024 / 1024,
                        new_capacity / 1024 / 1024
                    );
                }

                // Limpiar items de baja prioridad si uso > 90%
                if usage_percent > 90.0 {
                    let to_remove: Vec<String> = items
                        .iter()
                        .filter(|e| e.value().priority == StoragePriority::Low)
                        .map(|e| e.key().clone())
                        .collect();

                    for key in to_remove {
                        items.remove(&key);
                    }

                    info!("🧹 Limpieza de almacenamiento completada");
                }

                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            }
        });
    }

    /// Almacena un item con prioridad
    pub async fn store(&self, key: String, data: Vec<u8>, priority: StoragePriority) -> Result<()> {
        let item = StorageItem {
            key: key.clone(),
            data,
            priority,
            access_count: 0,
            created_at: std::time::Instant::now(),
        };

        self.items.insert(key.clone(), item);
        Ok(())
    }

    /// Recupera un item
    pub async fn retrieve(&self, key: &str) -> Option<Vec<u8>> {
        if let Some(mut item) = self.items.get_mut(key) {
            item.access_count += 1;
            Some(item.data.clone())
        } else {
            None
        }
    }

    /// Obtiene estadísticas
    pub fn get_stats(&self) -> serde_json::Value {
        let total_items = self.items.len();
        let total_size: usize = self.items.iter().map(|e| e.value().data.len()).sum();
        let capacity = *self.capacity.blocking_read();

        serde_json::json!({
            "total_items": total_items,
            "total_size_mb": total_size / 1024 / 1024,
            "capacity_mb": capacity / 1024 / 1024,
            "usage_percent": (total_size as f64 / capacity as f64) * 100.0,
        })
    }
}
