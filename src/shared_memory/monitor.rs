//! shared_memory/monitor.rs - Monitor de memoria en tiempo real

use super::types::MemoryStats;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Monitor de memoria en tiempo real
/// Recopila métricas y estadísticas del sistema de memoria compartida
pub struct MemoryMonitor {
    /// Estadísticas actuales
    stats: Arc<RwLock<MemoryStats>>,

    /// Cache hits
    cache_hits: Arc<AtomicU64>,

    /// Cache misses
    cache_misses: Arc<AtomicU64>,

    /// Total de actualizaciones
    total_updates: Arc<AtomicU64>,

    /// Indica si el monitor está activo
    active: Arc<AtomicBool>,
}

impl MemoryMonitor {
    /// Crea un nuevo monitor
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(MemoryStats::new())),
            cache_hits: Arc::new(AtomicU64::new(0)),
            cache_misses: Arc::new(AtomicU64::new(0)),
            total_updates: Arc::new(AtomicU64::new(0)),
            active: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Inicia el monitor
    pub async fn start(&self) {
        if self.active.load(Ordering::Acquire) {
            return;
        }

        info!("🔧 Iniciando monitor de memoria");
        self.active.store(true, Ordering::Release);

        // Iniciar tarea de actualización periódica
        let stats = Arc::clone(&self.stats);
        let cache_hits = Arc::clone(&self.cache_hits);
        let cache_misses = Arc::clone(&self.cache_misses);
        let total_updates = Arc::clone(&self.total_updates);
        let active = Arc::clone(&self.active);

        tokio::spawn(async move {
            while active.load(Ordering::Acquire) {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

                let mut stats_guard = stats.write().await;
                stats_guard.cache_hits = cache_hits.load(Ordering::Acquire);
                stats_guard.cache_misses = cache_misses.load(Ordering::Acquire);
                stats_guard.total_updates = total_updates.load(Ordering::Acquire);
                stats_guard.calculate_cache_hit_rate();

                debug!(
                    "Estadísticas: {} hits, {} misses, tasa {:.2}%",
                    stats_guard.cache_hits,
                    stats_guard.cache_misses,
                    stats_guard.cache_hit_rate * 100.0
                );
            }
        });

        info!("✅ Monitor de memoria iniciado");
    }

    /// Registra un cache hit
    pub async fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Release);
    }

    /// Registra un cache miss
    pub async fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Release);
    }

    /// Registra una actualización
    pub async fn record_update(&self) {
        self.total_updates.fetch_add(1, Ordering::Release);
    }

    /// Obtiene estadísticas actuales
    pub async fn get_stats(&self) -> MemoryStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Actualiza el número de contextos activos
    pub async fn set_active_contexts(&self, count: usize) {
        let mut stats = self.stats.write().await;
        stats.active_contexts = count;
    }

    /// Actualiza el número de contextos persistidos
    pub async fn set_persisted_contexts(&self, count: usize) {
        let mut stats = self.stats.write().await;
        stats.persisted_contexts = count;
    }

    /// Actualiza uso de memoria
    pub async fn set_memory_usage(&self, bytes: u64) {
        let mut stats = self.stats.write().await;
        stats.memory_usage_bytes = bytes;
    }

    /// Detiene el monitor
    pub async fn stop(&self) {
        info!("🔧 Deteniendo monitor de memoria");
        self.active.store(false, Ordering::Release);

        // Esperar a que termine la tarea de actualización
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        info!("✅ Monitor de memoria detenido");
    }
}

impl Default for MemoryMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitor_creation() {
        let monitor = MemoryMonitor::new();
        assert!(!monitor.active.load(Ordering::Acquire));
    }

    #[tokio::test]
    async fn test_monitor_start_stop() {
        let monitor = MemoryMonitor::new();
        monitor.start().await;
        assert!(monitor.active.load(Ordering::Acquire));

        monitor.stop().await;
        assert!(!monitor.active.load(Ordering::Acquire));
    }

    #[tokio::test]
    async fn test_record_cache_hits() {
        let monitor = MemoryMonitor::new();
        monitor.start().await;

        monitor.record_cache_hit().await;
        monitor.record_cache_hit().await;
        monitor.record_cache_miss().await;

        let stats = monitor.get_stats().await;
        assert_eq!(stats.cache_hits, 2);
        assert_eq!(stats.cache_misses, 1);

        monitor.stop().await;
    }

    #[tokio::test]
    async fn test_update_stats() {
        let monitor = MemoryMonitor::new();
        monitor.start().await;

        monitor.set_active_contexts(10).await;
        monitor.set_memory_usage(1024 * 1024).await;

        let stats = monitor.get_stats().await;
        assert_eq!(stats.active_contexts, 10);
        assert_eq!(stats.memory_usage_bytes, 1024 * 1024);

        monitor.stop().await;
    }
}
