//! nuclear_crawler/mod.rs - Nuclear Crawler Hybrid System
//! Sistema avanzado de crawling con auto-gestión, validación continua y monitoreo

pub mod auto_rebuild;
pub mod deep_storage_tunnels;
pub mod deepweb_tor;
pub mod intelligent_storage;
pub mod metrics_exporter;
pub mod predictive_nodes;

use crate::error::{MemoryPError, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Estado del crawler
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrawlerState {
    Idle,
    Running,
    Paused,
    Error,
}

/// Configuración del Nuclear Crawler
#[derive(Debug, Clone)]
pub struct CrawlerConfig {
    /// Habilitar modo Tor para deepweb
    pub enable_tor: bool,

    /// Habilitar almacenamiento inteligente
    pub enable_intelligent_storage: bool,

    /// Habilitar nodos predictivos
    pub enable_predictive_nodes: bool,

    /// Intervalo de auto-rebuild (segundos)
    pub auto_rebuild_interval: u64,

    /// Tamaño del buffer paralelo
    pub parallel_buffer_size: usize,

    /// Nivel de seguridad (1-5)
    pub security_level: u8,
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        Self {
            enable_tor: false,
            enable_intelligent_storage: true,
            enable_predictive_nodes: true,
            auto_rebuild_interval: 300, // 5 minutos
            parallel_buffer_size: 1024,
            security_level: 3,
        }
    }
}

/// Nuclear Crawler principal
pub struct NuclearCrawler {
    config: CrawlerConfig,
    state: Arc<RwLock<CrawlerState>>,
    auto_rebuild: auto_rebuild::AutoRebuild,
    deepweb_tor: Option<deepweb_tor::DeepwebTor>,
    intelligent_storage: intelligent_storage::IntelligentStorage,
    predictive_nodes: predictive_nodes::PredictiveNodes,
    metrics_exporter: Arc<metrics_exporter::MetricsExporter>,
}

impl NuclearCrawler {
    /// Crea un nuevo Nuclear Crawler
    pub fn new(config: CrawlerConfig) -> Self {
        let deepweb_tor = if config.enable_tor {
            Some(deepweb_tor::DeepwebTor::new())
        } else {
            None
        };

        Self {
            state: Arc::new(RwLock::new(CrawlerState::Idle)),
            auto_rebuild: auto_rebuild::AutoRebuild::new(config.auto_rebuild_interval),
            deepweb_tor,
            intelligent_storage: intelligent_storage::IntelligentStorage::new(),
            predictive_nodes: predictive_nodes::PredictiveNodes::new(),
            metrics_exporter: Arc::new(metrics_exporter::MetricsExporter::new()),
            config,
        }
    }

    /// Inicia el crawler con auto-gestión
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Iniciando Nuclear Crawler Hybrid System");

        let mut state = self.state.write().await;
        if *state == CrawlerState::Running {
            warn!("Nuclear Crawler ya está ejecutándose");
            return Ok(());
        }
        *state = CrawlerState::Running;
        drop(state);

        // 1. Iniciar auto-rebuild
        self.auto_rebuild.start().await?;

        // 2. Iniciar deepweb tor si está habilitado
        if let Some(tor) = &self.deepweb_tor {
            tor.start().await?;
        }

        // 3. Iniciar intelligent storage
        self.intelligent_storage.start().await?;

        // 4. Iniciar predictive nodes
        if self.config.enable_predictive_nodes {
            self.predictive_nodes.start().await?;
        }

        info!("✅ Nuclear Crawler iniciado - Modo Always-On activo");
        Ok(())
    }

    /// Detiene el crawler
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Deteniendo Nuclear Crawler...");

        let mut state = self.state.write().await;
        *state = CrawlerState::Idle;
        drop(state);

        // Detener componentes
        self.auto_rebuild.stop().await?;

        if let Some(tor) = &self.deepweb_tor {
            tor.stop().await?;
        }

        self.intelligent_storage.stop().await?;
        self.predictive_nodes.stop().await?;

        info!("✅ Nuclear Crawler detenido");
        Ok(())
    }

    /// Obtiene el estado actual
    pub async fn get_state(&self) -> CrawlerState {
        *self.state.read().await
    }

    /// Realiza una búsqueda con auto-corrección predictiva
    pub async fn search(&self, query: &str) -> Result<Vec<String>> {
        if *self.state.read().await != CrawlerState::Running {
            return Err(MemoryPError::Other("Crawler no está ejecutándose".into()));
        }

        // Intentar búsqueda con auto-corrección
        match self.predictive_nodes.predict_and_search(query).await {
            Ok(results) => Ok(results),
            Err(e) => {
                warn!("Búsqueda falló, intentando auto-corrección: {}", e);
                // Auto-corrección automática
                self.predictive_nodes.auto_correct_and_retry(query).await
            }
        }
    }

    /// Obtiene estadísticas del crawler
    pub fn get_stats(&self) -> serde_json::Value {
        // Actualizar métricas antes de exportar
        let state_str = format!("{:?}", *self.state.blocking_read());
        let tor_connected = self
            .deepweb_tor
            .as_ref()
            .map(|t| {
                // Obtener estado de forma síncrona usando blocking_on
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(t.is_connected())
                })
            })
            .unwrap_or(false);

        let storage_stats = self.intelligent_storage.get_stats();
        let storage_size_mb = storage_stats["total_size_mb"].as_f64().unwrap_or(0.0);

        let predictive_stats = self.predictive_nodes.get_stats();
        let predictions_count = predictive_stats["total_predictions"].as_u64().unwrap_or(0);

        self.metrics_exporter.record_crawler_metrics(
            &state_str,
            tor_connected,
            storage_size_mb,
            predictions_count,
        );

        serde_json::json!({
            "state": state_str,
            "config": {
                "tor_enabled": self.config.enable_tor,
                "intelligent_storage": self.config.enable_intelligent_storage,
                "predictive_nodes": self.config.enable_predictive_nodes,
                "security_level": self.config.security_level,
            },
            "auto_rebuild": self.auto_rebuild.get_stats(),
            "intelligent_storage": storage_stats,
            "predictive_nodes": predictive_stats,
            "metrics_count": self.metrics_exporter.metrics_count(),
        })
    }

    /// Exporta métricas en formato Prometheus
    pub fn export_prometheus_metrics(&self) -> String {
        self.metrics_exporter.export_prometheus()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crawler_lifecycle() {
        let config = CrawlerConfig::default();
        let crawler = NuclearCrawler::new(config);

        assert_eq!(crawler.get_state().await, CrawlerState::Idle);

        assert!(crawler.start().await.is_ok());
        assert_eq!(crawler.get_state().await, CrawlerState::Running);

        assert!(crawler.stop().await.is_ok());
        assert_eq!(crawler.get_state().await, CrawlerState::Idle);
    }
}

impl NuclearCrawler {
    /// Realiza una búsqueda real en internet (Simulado por ahora)
    pub async fn search_internet(&self, query: &str) -> Result<Vec<String>> {
        info!("🌐 Buscando en internet: {}", query);

        // Simulación de crawling real
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        Ok(vec![
            format!("Result for {}: Documentation from official source", query),
            format!("Result for {}: StackOverflow discussion", query),
            format!("Result for {}: GitHub repository example", query)
        ])
    }
}
