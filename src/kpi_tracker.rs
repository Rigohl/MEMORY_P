//! kpi_tracker.rs - Sistema de Medición de KPIs Always-On
//! Six Sigma & Automation Metrics

use crate::error::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Categorías de KPIs Six Sigma
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KpiCategory {
    /// DMAIC: Define, Measure, Analyze, Improve, Control
    Quality,
    /// Throughput y velocidad
    Performance,
    /// Disponibilidad y uptime
    Availability,
    /// Eficiencia de procesos
    Efficiency,
    /// Defectos y errores
    Defects,
    /// Costos y recursos
    Cost,
}

/// Métrica Six Sigma individual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SixSigmaMetric {
    pub name: String,
    pub category: KpiCategory,
    pub value: f64,
    pub target: f64,
    pub upper_spec_limit: f64, // USL
    pub lower_spec_limit: f64, // LSL
    pub timestamp: DateTime<Utc>, // Changed from Instant
    pub unit: String,
}

impl SixSigmaMetric {
    /// Calcula Cp (Process Capability)
    pub fn calculate_cp(&self, std_dev: f64) -> f64 {
        (self.upper_spec_limit - self.lower_spec_limit) / (6.0 * std_dev)
    }

    /// Calcula Cpk (Process Capability Index)
    pub fn calculate_cpk(&self, mean: f64, std_dev: f64) -> f64 {
        let cpu = (self.upper_spec_limit - mean) / (3.0 * std_dev);
        let cpl = (mean - self.lower_spec_limit) / (3.0 * std_dev);
        cpu.min(cpl)
    }

    /// Calcula nivel Sigma
    pub fn calculate_sigma_level(&self, defects: f64, opportunities: f64) -> f64 {
        let dpmo = (defects / opportunities) * 1_000_000.0;
        // Conversión aproximada DPMO a Sigma
        if dpmo <= 3.4 {
            6.0
        } else if dpmo <= 233.0 {
            5.0
        } else if dpmo <= 6_210.0 {
            4.0
        } else if dpmo <= 66_807.0 {
            3.0
        } else if dpmo <= 308_538.0 {
            2.0
        } else {
            1.0
        }
    }

    /// Verifica si está dentro de especificación
    pub fn is_within_spec(&self) -> bool {
        self.value >= self.lower_spec_limit && self.value <= self.upper_spec_limit
    }
}

/// Agregación de métricas para análisis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricAggregation {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
    pub cp: f64,
    pub cpk: f64,
    pub sigma_level: f64,
    pub defect_rate: f64,
}

/// KPI Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiDashboard {
    pub timestamp: DateTime<Utc>, // Changed from Instant
    pub overall_sigma_level: f64,
    pub categories: Vec<CategoryMetrics>,
    pub alerts: Vec<KpiAlert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryMetrics {
    pub category: KpiCategory,
    pub metrics_count: usize,
    pub avg_cpk: f64,
    pub defect_rate: f64,
    pub sigma_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiAlert {
    pub severity: AlertSeverity,
    pub category: KpiCategory,
    pub message: String,
    pub timestamp: DateTime<Utc>, // Changed from Instant
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical, // Sigma < 3
    Warning,  // Sigma < 4
    Info,     // Sigma < 5
}

/// Tracker de KPIs Always-On
pub struct KpiTracker {
    /// Métricas actuales
    metrics: Arc<DashMap<String, Vec<SixSigmaMetric>>>,
    
    /// Configuración
    config: KpiConfig,
    
    /// Estado de ejecución
    running: Arc<RwLock<bool>>,
    
    /// Alertas activas
    alerts: Arc<DashMap<String, KpiAlert>>,
}

#[derive(Debug, Clone)]
pub struct KpiConfig {
    /// Intervalo de medición (segundos)
    pub measurement_interval: Duration,
    
    /// Retención de datos históricos (segundos)
    pub retention_period: Duration,
    
    /// Umbral de alerta Cpk
    pub cpk_warning_threshold: f64,
    
    /// Umbral crítico Cpk
    pub cpk_critical_threshold: f64,
    
    /// Target Sigma level
    pub target_sigma_level: f64,
}

impl Default for KpiConfig {
    fn default() -> Self {
        Self {
            measurement_interval: Duration::from_secs(10), // Medir cada 10s
            retention_period: Duration::from_secs(86400),  // 24 horas
            cpk_warning_threshold: 1.33,                   // Cpk < 1.33 = Warning
            cpk_critical_threshold: 1.0,                   // Cpk < 1.0 = Critical
            target_sigma_level: 4.0,                       // Target: 4 Sigma
        }
    }
}

impl KpiTracker {
    pub fn new(config: KpiConfig) -> Self {
        Self {
            metrics: Arc::new(DashMap::new()),
            config,
            running: Arc::new(RwLock::new(false)),
            alerts: Arc::new(DashMap::new()),
        }
    }

    /// Inicia el tracking de KPIs (auto-ejecutado)
    pub async fn start(&self) -> Result<()> {
        info!("📊 Iniciando KPI Tracker - Six Sigma Always-On");
        
        let mut running = self.running.write().await;
        if *running {
            warn!("KPI Tracker ya está ejecutándose");
            return Ok(());
        }
        *running = true;
        drop(running);

        // Iniciar mediciones automáticas
        self.start_auto_measurement().await;
        
        // Iniciar análisis Six Sigma
        self.start_six_sigma_analysis().await;
        
        // Iniciar limpieza de datos antiguos
        self.start_data_cleanup().await;
        
        info!("✅ KPI Tracker iniciado");
        info!("   • Mediciones: cada {:?}", self.config.measurement_interval);
        info!("   • Target Sigma: {}", self.config.target_sigma_level);
        info!("   • Retención: {:?}", self.config.retention_period);
        
        Ok(())
    }

    /// Registra una métrica
    pub fn record_metric(&self, metric: SixSigmaMetric) {
        let key = format!("{}:{:?}", metric.name, metric.category);
        
        self.metrics
            .entry(key.clone())
            .and_modify(|metrics| {
                metrics.push(metric.clone());
                // Limitar tamaño del buffer
                if metrics.len() > 10_000 {
                    metrics.drain(0..1000);
                }
            })
            .or_insert_with(|| vec![metric.clone()]);
        
        // Verificar alertas
        self.check_alerts(&metric);
    }

    /// Verifica si una métrica genera alertas
    fn check_alerts(&self, metric: &SixSigmaMetric) {
        if !metric.is_within_spec() {
            let alert = KpiAlert {
                severity: AlertSeverity::Critical,
                category: metric.category,
                message: format!(
                    "{} fuera de especificación: {} (LSL: {}, USL: {})",
                    metric.name, metric.value, metric.lower_spec_limit, metric.upper_spec_limit
                ),
                timestamp: Utc::now(),
            };
            
            self.alerts.insert(metric.name.clone(), alert);
        }
    }

    /// Inicia mediciones automáticas
    async fn start_auto_measurement(&self) {
        let _metrics = self.metrics.clone();
        let running = self.running.clone();
        let interval = self.config.measurement_interval;

        tokio::spawn(async move {
            info!("📈 Auto-measurement iniciado");
            
            loop {
                if !*running.read().await {
                    break;
                }

                // Simular mediciones (en implementación real, recoger métricas del sistema)
                // Ejemplo: medir latencia, throughput, error rate, etc.
                
                tokio::time::sleep(interval).await;
            }
            
            info!("📈 Auto-measurement detenido");
        });
    }

    /// Inicia análisis Six Sigma continuo
    async fn start_six_sigma_analysis(&self) {
        let metrics = self.metrics.clone();
        let alerts = self.alerts.clone();
        let config = self.config.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            info!("🎯 Six Sigma Analysis iniciado");
            
            loop {
                if !*running.read().await {
                    break;
                }

                // Analizar cada categoría de métricas
                for entry in metrics.iter() {
                    let (name, metric_list) = entry.pair();
                    
                    if metric_list.len() < 30 {
                        continue; // Necesitamos suficientes datos
                    }
                    
                    // Calcular estadísticas
                    let values: Vec<f64> = metric_list.iter().map(|m| m.value).collect();
                    let mean = values.iter().sum::<f64>() / values.len() as f64;
                    
                    let variance = values.iter()
                        .map(|v| (v - mean).powi(2))
                        .sum::<f64>() / values.len() as f64;
                    let std_dev = variance.sqrt();
                    
                    // Obtener última métrica para specs
                    if let Some(last_metric) = metric_list.last() {
                        let cpk = last_metric.calculate_cpk(mean, std_dev);
                        
                        // Generar alertas según Cpk
                        if cpk < config.cpk_critical_threshold {
                            let alert = KpiAlert {
                                severity: AlertSeverity::Critical,
                                category: last_metric.category,
                                message: format!(
                                    "{}: Cpk crítico ({:.2}) - Proceso fuera de control",
                                    name, cpk
                                ),
                                timestamp: Utc::now(),
                            };
                            alerts.insert(name.clone(), alert);
                        } else if cpk < config.cpk_warning_threshold {
                            let alert = KpiAlert {
                                severity: AlertSeverity::Warning,
                                category: last_metric.category,
                                message: format!(
                                    "{}: Cpk bajo ({:.2}) - Mejora necesaria",
                                    name, cpk
                                ),
                                timestamp: Utc::now(),
                            };
                            alerts.insert(name.clone(), alert);
                        }
                    }
                }

                tokio::time::sleep(Duration::from_secs(60)).await; // Analizar cada minuto
            }
            
            info!("🎯 Six Sigma Analysis detenido");
        });
    }

    /// Inicia limpieza de datos antiguos
    async fn start_data_cleanup(&self) {
        let metrics = self.metrics.clone();
        let retention_period = self.config.retention_period;
        let running = self.running.clone();

        tokio::spawn(async move {
            info!("🧹 Data cleanup iniciado");
            
            loop {
                if !*running.read().await {
                    break;
                }

                // Limpiar métricas antiguas
                for mut entry in metrics.iter_mut() {
                    let metric_list = entry.value_mut();
                    let cutoff = Utc::now() - chrono::Duration::from_std(retention_period)
                        .unwrap_or_else(|_| chrono::Duration::days(7));  // Fallback to 7 days
                    
                    metric_list.retain(|m| m.timestamp > cutoff);
                }

                tokio::time::sleep(Duration::from_secs(3600)).await; // Limpiar cada hora
            }
            
            info!("🧹 Data cleanup detenido");
        });
    }

    /// Obtiene el dashboard de KPIs
    pub fn get_dashboard(&self) -> KpiDashboard {
        let mut category_metrics: Vec<CategoryMetrics> = Vec::new();
        let mut overall_cpk_sum = 0.0;
        let mut overall_count = 0;

        for category in [
            KpiCategory::Quality,
            KpiCategory::Performance,
            KpiCategory::Availability,
            KpiCategory::Efficiency,
            KpiCategory::Defects,
            KpiCategory::Cost,
        ] {
            let mut cpk_sum = 0.0;
            let mut count = 0;
            let mut defects = 0;
            let mut opportunities = 0;

            for entry in self.metrics.iter() {
                let metric_list = entry.value();
                
                if metric_list.is_empty() {
                    continue;
                }
                
                if metric_list[0].category != category {
                    continue;
                }
                
                // Calcular Cpk promedio
                if metric_list.len() >= 30 {
                    let values: Vec<f64> = metric_list.iter().map(|m| m.value).collect();
                    let mean = values.iter().sum::<f64>() / values.len() as f64;
                    let variance = values.iter()
                        .map(|v| (v - mean).powi(2))
                        .sum::<f64>() / values.len() as f64;
                    let std_dev = variance.sqrt();
                    
                    if let Some(last_metric) = metric_list.last() {
                        let cpk = last_metric.calculate_cpk(mean, std_dev);
                        cpk_sum += cpk;
                        count += 1;
                        
                        // Contar defectos
                        for m in metric_list.iter() {
                            opportunities += 1;
                            if !m.is_within_spec() {
                                defects += 1;
                            }
                        }
                    }
                }
            }

            if count > 0 {
                let avg_cpk = cpk_sum / count as f64;
                let defect_rate = if opportunities > 0 {
                    (defects as f64 / opportunities as f64) * 100.0
                } else {
                    0.0
                };
                
                // Calcular Sigma level aproximado
                let dpmo = defect_rate * 10_000.0;
                let sigma_level = if dpmo <= 3.4 { 6.0 }
                    else if dpmo <= 233.0 { 5.0 }
                    else if dpmo <= 6_210.0 { 4.0 }
                    else if dpmo <= 66_807.0 { 3.0 }
                    else { 2.0 };

                category_metrics.push(CategoryMetrics {
                    category,
                    metrics_count: count,
                    avg_cpk,
                    defect_rate,
                    sigma_level,
                });

                overall_cpk_sum += avg_cpk;
                overall_count += 1;
            }
        }

        // Calcular Sigma level general
        let overall_sigma_level = if overall_count > 0 {
            let avg_cpk = overall_cpk_sum / overall_count as f64;
            // Conversión aproximada Cpk a Sigma
            if avg_cpk >= 2.0 { 6.0 }
            else if avg_cpk >= 1.67 { 5.0 }
            else if avg_cpk >= 1.33 { 4.0 }
            else if avg_cpk >= 1.0 { 3.0 }
            else { 2.0 }
        } else {
            0.0
        };

        // Recoger alertas activas
        let alerts: Vec<KpiAlert> = self.alerts
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        KpiDashboard {
            timestamp: Utc::now(),
            overall_sigma_level,
            categories: category_metrics,
            alerts,
        }
    }

    /// Detiene el tracker
    pub async fn stop(&self) {
        info!("🛑 Deteniendo KPI Tracker...");
        let mut running = self.running.write().await;
        *running = false;
        info!("✅ KPI Tracker detenido");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_six_sigma_metric() {
        let metric = SixSigmaMetric {
            name: "latency".to_string(),
            category: KpiCategory::Performance,
            value: 50.0,
            target: 50.0,
            upper_spec_limit: 100.0,
            lower_spec_limit: 0.0,
            timestamp: Utc::now(),
            unit: "ms".to_string(),
        };

        let cpk = metric.calculate_cpk(50.0, 10.0);
        assert!(cpk > 0.0);
        assert!(metric.is_within_spec());
    }

    #[tokio::test]
    async fn test_kpi_tracker() {
        let config = KpiConfig::default();
        let tracker = KpiTracker::new(config);
        
        assert!(tracker.start().await.is_ok());
        
        let metric = SixSigmaMetric {
            name: "test_metric".to_string(),
            category: KpiCategory::Quality,
            value: 95.0,
            target: 100.0,
            upper_spec_limit: 110.0,
            lower_spec_limit: 90.0,
            timestamp: Utc::now(),
            unit: "%".to_string(),
        };
        
        tracker.record_metric(metric);
        
        let dashboard = tracker.get_dashboard();
        assert_eq!(dashboard.categories.len() <= 6, true);
        
        tracker.stop().await;
    }
}
