//! kpi_tracker.rs - Sistema de KPIs Six Sigma


pub struct KpiConfig {
    pub enabled: bool,
}

impl Default for KpiConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

pub struct KpiTracker {
    config: KpiConfig,
}

impl KpiTracker {
    pub fn new(config: KpiConfig) -> Self {
        Self { config }
    }

    pub async fn start(&self) -> crate::error::Result<()> {
        if self.config.enabled {
            // Log something or start a worker
        }
        Ok(())
    }

    pub async fn stop(&self) {}

    pub fn get_dashboard(&self) -> Dashboard {
        Dashboard {
            overall_sigma_level: 4.5,
            categories: vec![],
            alerts: vec![],
        }
    }

    pub fn record_metric(&self, _metric: SixSigmaMetric) {}
}

pub struct Dashboard {
    pub overall_sigma_level: f64,
    pub categories: Vec<KpiCategoryInfo>,
    pub alerts: Vec<KpiAlert>,
}

pub struct KpiCategoryInfo {
    pub category: KpiCategory,
    pub metrics_count: usize,
    pub avg_cpk: f64,
    pub defect_rate: f64,
    pub sigma_level: f64,
}

pub struct KpiAlert {
    pub severity: KpiSeverity,
    pub category: KpiCategory,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub enum KpiCategory { Quality, Performance, Availability, Efficiency, Defects, Cost }
#[derive(Debug, Clone, Copy)]
pub enum KpiSeverity { Low, Medium, High, Critical }

#[derive(Clone)]
pub struct SixSigmaMetric {
    pub name: String,
    pub category: KpiCategory,
    pub value: f64,
    pub target: f64,
    pub upper_spec_limit: f64,
    pub lower_spec_limit: f64,
    pub timestamp: std::time::Instant,
    pub unit: String,
}

impl SixSigmaMetric {
    pub fn is_within_spec(&self) -> bool {
        self.value >= self.lower_spec_limit && self.value <= self.upper_spec_limit
    }
}
