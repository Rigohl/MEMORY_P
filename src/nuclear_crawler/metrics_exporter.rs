//! metrics_exporter.rs - Exportador de métricas para Prometheus
//! Monitoreo en tiempo real del Nuclear Crawler

use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;

/// Tipo de métrica
#[derive(Debug, Clone, Copy)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

/// Métrica individual
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub metric_type: MetricType,
    pub labels: Vec<(String, String)>,
    pub timestamp: Instant,
}

/// Exportador de métricas
pub struct MetricsExporter {
    metrics: Arc<DashMap<String, Metric>>,
}

impl MetricsExporter {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(DashMap::new()),
        }
    }

    /// Incrementa un contador
    pub fn increment_counter(&self, name: &str, labels: Vec<(&str, &str)>) {
        let key = self.make_key(name, &labels);

        self.metrics
            .entry(key.clone())
            .and_modify(|m| m.value += 1.0)
            .or_insert_with(|| Metric {
                name: name.to_string(),
                value: 1.0,
                metric_type: MetricType::Counter,
                labels: labels
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                timestamp: Instant::now(),
            });
    }

    /// Establece un gauge
    pub fn set_gauge(&self, name: &str, value: f64, labels: Vec<(&str, &str)>) {
        let key = self.make_key(name, &labels);

        self.metrics.insert(
            key,
            Metric {
                name: name.to_string(),
                value,
                metric_type: MetricType::Gauge,
                labels: labels
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                timestamp: Instant::now(),
            },
        );
    }

    /// Registra un valor en histograma
    pub fn observe_histogram(&self, name: &str, value: f64, labels: Vec<(&str, &str)>) {
        let key = self.make_key(name, &labels);

        // Simplificado: solo guarda el último valor
        // En implementación real: mantener buckets
        self.metrics.insert(
            key,
            Metric {
                name: name.to_string(),
                value,
                metric_type: MetricType::Histogram,
                labels: labels
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                timestamp: Instant::now(),
            },
        );
    }

    /// Genera clave única para métrica
    fn make_key(&self, name: &str, labels: &[(&str, &str)]) -> String {
        let mut key = name.to_string();
        for (k, v) in labels {
            key.push_str(&format!(",{}={}", k, v));
        }
        key
    }

    /// Exporta métricas en formato Prometheus
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();

        // Agrupar por nombre de métrica
        let mut metrics_by_name: std::collections::HashMap<String, Vec<Metric>> =
            std::collections::HashMap::new();

        for entry in self.metrics.iter() {
            let metric = entry.value();
            metrics_by_name
                .entry(metric.name.clone())
                .or_insert_with(Vec::new)
                .push(metric.clone());
        }

        // Exportar cada grupo
        for (name, metrics) in metrics_by_name {
            if let Some(first) = metrics.first() {
                // HELP y TYPE
                output.push_str(&format!("# HELP {} Nuclear Crawler metric\n", name));
                output
                    .push_str(&format!("# TYPE {} {:?}\n", name, first.metric_type).to_lowercase());

                // Valores
                for metric in metrics {
                    let labels_str = if metric.labels.is_empty() {
                        String::new()
                    } else {
                        let labels: Vec<String> = metric
                            .labels
                            .iter()
                            .map(|(k, v)| format!("{}=\"{}\"", k, v))
                            .collect();
                        format!("{{{}}}", labels.join(","))
                    };

                    output.push_str(&format!("{}{} {}\n", name, labels_str, metric.value));
                }

                output.push('\n');
            }
        }

        output
    }

    /// Registra métricas del Nuclear Crawler
    pub fn record_crawler_metrics(
        &self,
        state: &str,
        tor_connected: bool,
        storage_size_mb: f64,
        predictions_count: u64,
    ) {
        // Estado del crawler
        self.set_gauge(
            "nuclear_crawler_state",
            if state == "Running" { 1.0 } else { 0.0 },
            vec![("state", state)],
        );

        // Conexión Tor
        self.set_gauge(
            "nuclear_crawler_tor_connected",
            if tor_connected { 1.0 } else { 0.0 },
            vec![],
        );

        // Almacenamiento
        self.set_gauge("nuclear_crawler_storage_size_mb", storage_size_mb, vec![]);

        // Predicciones
        self.set_gauge(
            "nuclear_crawler_predictions_total",
            predictions_count as f64,
            vec![],
        );
    }

    /// Obtiene el número total de métricas
    pub fn metrics_count(&self) -> usize {
        self.metrics.len()
    }
}

impl Default for MetricsExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_increment() {
        let exporter = MetricsExporter::new();

        exporter.increment_counter("test_counter", vec![("label", "value")]);
        exporter.increment_counter("test_counter", vec![("label", "value")]);

        assert_eq!(exporter.metrics_count(), 1);
    }

    #[test]
    fn test_gauge_set() {
        let exporter = MetricsExporter::new();

        exporter.set_gauge("test_gauge", 42.0, vec![]);
        exporter.set_gauge("test_gauge", 100.0, vec![]);

        assert_eq!(exporter.metrics_count(), 1);
    }

    #[test]
    fn test_prometheus_export() {
        let exporter = MetricsExporter::new();

        exporter.increment_counter("requests_total", vec![("method", "GET")]);
        exporter.set_gauge("cpu_usage", 75.5, vec![]);

        let output = exporter.export_prometheus();

        assert!(output.contains("# HELP"));
        assert!(output.contains("# TYPE"));
        assert!(output.contains("requests_total"));
        assert!(output.contains("cpu_usage"));
    }
}
