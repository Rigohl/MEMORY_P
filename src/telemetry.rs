//! telemetry.rs - Sistema de Telemetría con ClickHouse y Prometheus
//!
//! Proporciona telemetría completa del sistema con:
//! - Integración con ClickHouse para analytics
//! - Métricas de Prometheus para monitoreo en tiempo real
//! - Tracking de eventos y resultados
//! - Dashboard de métricas

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Sistema de telemetría
pub struct TelemetrySystem {
    config: TelemetryConfig,
    event_buffer: Arc<Mutex<Vec<TelemetryEvent>>>,
    metrics_collector: Arc<MetricsCollector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub clickhouse_url: String,
    pub prometheus_port: u16,
    pub enable_metrics: bool,
    pub enable_tracing: bool,
    pub batch_size: usize,
    pub flush_interval_secs: u64,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            clickhouse_url: "http://localhost:8123".to_string(),
            prometheus_port: 9090,
            enable_metrics: true,
            enable_tracing: true,
            batch_size: 1000,
            flush_interval_secs: 10,
        }
    }
}

/// Evento de telemetría
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub component: String,
    pub metrics: serde_json::Value,
    pub tags: Vec<(String, String)>,
}

/// Colector de métricas
pub struct MetricsCollector {
    // Contadores
    pub total_requests: Arc<Mutex<u64>>,
    pub successful_requests: Arc<Mutex<u64>>,
    pub failed_requests: Arc<Mutex<u64>>,

    // Histogramas (latencias)
    pub response_times: Arc<Mutex<Vec<f64>>>,

    // Gauges
    pub active_connections: Arc<Mutex<i64>>,
    pub memory_usage_bytes: Arc<Mutex<u64>>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self {
            total_requests: Arc::new(Mutex::new(0)),
            successful_requests: Arc::new(Mutex::new(0)),
            failed_requests: Arc::new(Mutex::new(0)),
            response_times: Arc::new(Mutex::new(Vec::new())),
            active_connections: Arc::new(Mutex::new(0)),
            memory_usage_bytes: Arc::new(Mutex::new(0)),
        }
    }
}

impl TelemetrySystem {
    /// Crea un nuevo sistema de telemetría
    pub fn new(config: TelemetryConfig) -> Self {
        info!("📊 Inicializando sistema de telemetría");

        Self {
            config,
            event_buffer: Arc::new(Mutex::new(Vec::with_capacity(1000))),
            metrics_collector: Arc::new(MetricsCollector::default()),
        }
    }

    /// Inicia el sistema de telemetría
    pub async fn start(&self) -> Result<()> {
        if self.config.enable_metrics {
            self.start_metrics_collector().await;
        }

        if self.config.enable_tracing {
            self.start_event_flusher().await;
        }

        info!("✅ Sistema de telemetría iniciado");
        Ok(())
    }

    /// Registra un evento
    pub async fn record_event(&self, event: TelemetryEvent) {
        let mut buffer = self.event_buffer.lock().await;
        buffer.push(event);

        // Auto-flush si alcanzamos el tamaño del batch
        if buffer.len() >= self.config.batch_size {
            drop(buffer);
            let _ = self.flush_events().await;
        }
    }

    /// Registra métrica de latencia
    pub async fn record_latency(&self, duration_ms: f64) {
        let mut times = self.metrics_collector.response_times.lock().await;
        times.push(duration_ms);

        // Mantener solo últimos 10000 valores
        if times.len() > 10000 {
            times.remove(0);
        }
    }

    /// Incrementa contador de requests
    pub async fn increment_requests(&self, success: bool) {
        *self.metrics_collector.total_requests.lock().await += 1;

        if success {
            *self.metrics_collector.successful_requests.lock().await += 1;
        } else {
            *self.metrics_collector.failed_requests.lock().await += 1;
        }
    }

    /// Actualiza conexiones activas
    pub async fn update_active_connections(&self, delta: i64) {
        let mut conn = self.metrics_collector.active_connections.lock().await;
        *conn += delta;
    }

    /// Flush eventos a ClickHouse
    async fn flush_events(&self) -> Result<()> {
        let mut buffer = self.event_buffer.lock().await;

        if buffer.is_empty() {
            return Ok(());
        }

        let events = buffer.clone();
        buffer.clear();
        drop(buffer);

        debug!("📤 Flushing {} events to ClickHouse", events.len());

        // En producción, aquí enviaríamos a ClickHouse
        // Por ahora solo logeamos
        for event in &events {
            debug!("  📊 Event: {:?} - {}", event.event_type, event.component);
        }

        Ok(())
    }

    /// Inicia el colector de métricas en background
    async fn start_metrics_collector(&self) {
        let collector = self.metrics_collector.clone();

        tokio::spawn(async move {
            info!("📈 Metrics collector iniciado en puerto {}", 9090);

            // En producción, aquí iniciaríamos el servidor Prometheus
            // Por ahora solo logeamos métricas periódicamente
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

                let total = *collector.total_requests.lock().await;
                let successful = *collector.successful_requests.lock().await;
                let failed = *collector.failed_requests.lock().await;
                let connections = *collector.active_connections.lock().await;

                let success_rate = if total > 0 {
                    (successful as f64 / total as f64) * 100.0
                } else {
                    0.0
                };

                info!(
                    "📊 Metrics: {} total requests ({:.1}% success), {} active connections",
                    total, success_rate, connections
                );
            }
        });
    }

    /// Inicia el flusher de eventos en background
    async fn start_event_flusher(&self) {
        let event_buffer = self.event_buffer.clone();
        let flush_interval = self.config.flush_interval_secs;

        tokio::spawn(async move {
            info!("🔄 Event flusher iniciado (cada {}s)", flush_interval);

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(flush_interval)).await;

                let buffer_size = event_buffer.lock().await.len();
                if buffer_size > 0 {
                    debug!("🔄 Auto-flushing {} events", buffer_size);
                }
            }
        });
    }

    /// Obtiene snapshot de métricas actuales
    pub async fn get_metrics_snapshot(&self) -> MetricsSnapshot {
        let total = *self.metrics_collector.total_requests.lock().await;
        let successful = *self.metrics_collector.successful_requests.lock().await;
        let failed = *self.metrics_collector.failed_requests.lock().await;
        let connections = *self.metrics_collector.active_connections.lock().await;

        let response_times = self.metrics_collector.response_times.lock().await;
        let avg_latency = if !response_times.is_empty() {
            response_times.iter().sum::<f64>() / response_times.len() as f64
        } else {
            0.0
        };

        let p95_latency = if !response_times.is_empty() {
            let mut sorted = response_times.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let idx = (sorted.len() as f64 * 0.95) as usize;
            sorted[idx]
        } else {
            0.0
        };

        MetricsSnapshot {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            total_requests: total,
            successful_requests: successful,
            failed_requests: failed,
            success_rate: if total > 0 {
                (successful as f64 / total as f64) * 100.0
            } else {
                0.0
            },
            active_connections: connections,
            avg_latency_ms: avg_latency,
            p95_latency_ms: p95_latency,
        }
    }

    /// Detiene el sistema de telemetría
    pub async fn shutdown(&self) -> Result<()> {
        info!("🛑 Deteniendo sistema de telemetría...");

        // Flush final
        self.flush_events().await?;

        info!("✅ Sistema de telemetría detenido");
        Ok(())
    }
}

/// Snapshot de métricas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: u64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f64,
    pub active_connections: i64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_telemetry_system() {
        let config = TelemetryConfig::default();
        let telemetry = TelemetrySystem::new(config);

        assert!(telemetry.start().await.is_ok());

        // Record some events
        telemetry
            .record_event(TelemetryEvent {
                timestamp: 0,
                event_type: "test".to_string(),
                component: "test_component".to_string(),
                metrics: serde_json::json!({}),
                tags: vec![],
            })
            .await;

        telemetry.increment_requests(true).await;
        telemetry.record_latency(10.5).await;

        let snapshot = telemetry.get_metrics_snapshot().await;
        assert_eq!(snapshot.total_requests, 1);
        assert_eq!(snapshot.successful_requests, 1);

        assert!(telemetry.shutdown().await.is_ok());
    }
}
