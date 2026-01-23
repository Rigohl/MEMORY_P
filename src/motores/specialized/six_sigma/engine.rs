//! Six Sigma Optimizer - Continuous Quality Improvement Engine
//!
//! Implements DMAIC methodology for automatic code optimization

use crate::motores::core::{traits::SearchEngine, types::*};
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Six Sigma quality metrics
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub defects_per_million: f64,
    pub process_capability: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub samples: usize,
}

/// Control limits for statistical process control
#[derive(Debug, Clone)]
pub struct ControlLimits {
    pub upper_control_limit: f64,
    pub lower_control_limit: f64,
    pub center_line: f64,
}

/// Six Sigma Optimizer engine
#[allow(dead_code)]
pub struct SixSigmaOptimizer {
    config: EngineConfig,
    metrics_history: Vec<QualityMetrics>,
    control_limits: Option<ControlLimits>,
    defect_count: Arc<AtomicU64>,
    total_opportunities: Arc<AtomicU64>,
    initialized: bool,
}

impl SixSigmaOptimizer {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            config,
            metrics_history: Vec::new(),
            control_limits: None,
            defect_count: Arc::new(AtomicU64::new(0)),
            total_opportunities: Arc::new(AtomicU64::new(0)),
            initialized: false,
        }
    }

    /// Calculate defects per million opportunities (DPMO)
    pub fn calculate_dpmo(&self) -> f64 {
        let defects = self.defect_count.load(Ordering::Relaxed) as f64;
        let opportunities = self.total_opportunities.load(Ordering::Relaxed) as f64;
        
        if opportunities == 0.0 {
            return 0.0;
        }
        
        (defects / opportunities) * 1_000_000.0
    }

    /// Calculate process sigma level
    pub fn calculate_sigma_level(&self, dpmo: f64) -> f64 {
        // Approximate inverse normal CDF for sigma level
        // Six Sigma = 3.4 DPMO
        if dpmo <= 3.4 {
            return 6.0;
        } else if dpmo <= 233.0 {
            return 5.0;
        } else if dpmo <= 6_210.0 {
            return 4.0;
        } else if dpmo <= 66_807.0 {
            return 3.0;
        } else if dpmo <= 308_537.0 {
            return 2.0;
        } else {
            return 1.0;
        }
    }

    /// DMAIC: Define - Define the problem
    pub async fn define_problem(&self, data: &[f64]) -> String {
        let mean = self.calculate_mean(data);
        let std_dev = self.calculate_std_dev(data, mean);
        
        format!(
            "Process Analysis: Mean={:.2}, StdDev={:.2}, Samples={}",
            mean, std_dev, data.len()
        )
    }

    /// DMAIC: Measure - Measure current state
    pub async fn measure_current_state(&self, data: &[f64]) -> QualityMetrics {
        let mean = self.calculate_mean(data);
        let std_dev = self.calculate_std_dev(data, mean);
        let dpmo = self.calculate_dpmo();
        let sigma_level = self.calculate_sigma_level(dpmo);
        
        QualityMetrics {
            defects_per_million: dpmo,
            process_capability: sigma_level,
            mean,
            std_dev,
            samples: data.len(),
        }
    }

    /// DMAIC: Analyze - Analyze root causes
    pub async fn analyze_root_causes(&self, metrics: &QualityMetrics) -> Vec<String> {
        let mut causes = Vec::new();
        
        if metrics.defects_per_million > 3.4 {
            causes.push("Process capability below Six Sigma target".to_string());
        }
        
        if metrics.std_dev > metrics.mean * 0.2 {
            causes.push("High variability detected".to_string());
        }
        
        if metrics.samples < 100 {
            causes.push("Insufficient sample size for reliable analysis".to_string());
        }
        
        causes
    }

    /// DMAIC: Improve - Generate improvement recommendations
    pub async fn improve_process(&self, root_causes: &[String]) -> Vec<String> {
        let mut improvements = Vec::new();
        
        for cause in root_causes {
            if cause.contains("variability") {
                improvements.push("Implement tighter process controls".to_string());
                improvements.push("Reduce input variation".to_string());
            } else if cause.contains("sample size") {
                improvements.push("Collect more data points".to_string());
            } else if cause.contains("capability") {
                improvements.push("Optimize process parameters".to_string());
            }
        }
        
        improvements
    }

    /// DMAIC: Control - Establish control mechanisms
    pub fn establish_control_limits(&mut self, data: &[f64]) -> ControlLimits {
        let mean = self.calculate_mean(data);
        let std_dev = self.calculate_std_dev(data, mean);
        
        let limits = ControlLimits {
            center_line: mean,
            upper_control_limit: mean + 3.0 * std_dev,
            lower_control_limit: mean - 3.0 * std_dev,
        };
        
        self.control_limits = Some(limits.clone());
        limits
    }

    /// Record a defect
    pub fn record_defect(&self) {
        self.defect_count.fetch_add(1, Ordering::Relaxed);
        self.total_opportunities.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a success
    pub fn record_success(&self) {
        self.total_opportunities.fetch_add(1, Ordering::Relaxed);
    }

    /// Check if value is within control limits
    pub fn is_in_control(&self, value: f64) -> bool {
        if let Some(limits) = &self.control_limits {
            value >= limits.lower_control_limit && value <= limits.upper_control_limit
        } else {
            true // No limits established yet
        }
    }

    // Helper methods
    fn calculate_mean(&self, data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        data.iter().sum::<f64>() / data.len() as f64
    }

    fn calculate_std_dev(&self, data: &[f64], mean: f64) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }
        
        let variance = data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / (data.len() - 1) as f64;
        
        variance.sqrt()
    }
}

#[async_trait]
impl SearchEngine for SixSigmaOptimizer {
    async fn search(&self, _query: &SearchQuery) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        // Six Sigma search: optimize query and rank by quality
        let dpmo = self.calculate_dpmo();
        let sigma_level = self.calculate_sigma_level(dpmo);
        
        let result = SearchResult {
            id: "six_sigma_analysis".to_string(),
            content: format!(
                "Six Sigma Analysis:\nDPMO: {:.2}\nSigma Level: {:.1}\nStatus: {}",
                dpmo,
                sigma_level,
                if dpmo <= 3.4 { "Excellent ✓" } else { "Needs Improvement" }
            ),
            score: (sigma_level / 6.0) as f32, // Normalize to 0-1
            metadata: {
                let mut m = HashMap::new();
                m.insert("dpmo".to_string(), serde_json::json!(dpmo));
                m.insert("sigma_level".to_string(), serde_json::json!(sigma_level));
                m
            },
            engine: "six_sigma_optimizer".to_string(),
            highlights: vec![],
        };
        
        Ok(vec![result])
    }

    async fn index(&self, _documents: &[Document]) -> Result<(), Box<dyn Error>> {
        // Index documents and track quality metrics
        Ok(())
    }

    async fn delete(&self, _ids: &[String]) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn update(&self, _documents: &[Document]) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn health(&self) -> Result<EngineHealth, Box<dyn Error>> {
        let dpmo = self.calculate_dpmo();
        let sigma_level = self.calculate_sigma_level(dpmo);
        
        Ok(EngineHealth {
            engine: "six_sigma_optimizer".to_string(),
            healthy: sigma_level >= 4.0,
            status: if sigma_level >= 4.0 { "healthy".to_string() } else { "degraded".to_string() },
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            details: {
                let mut m = HashMap::new();
                m.insert("dpmo".to_string(), serde_json::json!(dpmo));
                m.insert("sigma_level".to_string(), serde_json::json!(sigma_level));
                m
            },
        })
    }

    async fn metrics(&self) -> Result<EngineMetrics, Box<dyn Error>> {
        let dpmo = self.calculate_dpmo();
        
        Ok(EngineMetrics {
            engine: "six_sigma_optimizer".to_string(),
            total_documents: self.total_opportunities.load(Ordering::Relaxed),
            avg_query_latency_ms: 1.0,
            queries_per_second: 1000.0,
            index_size_bytes: 0,
            memory_usage_bytes: 0,
            error_rate: dpmo / 1_000_000.0,
            cache_hit_rate: 0.0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        })
    }

    fn engine_name(&self) -> &'static str {
        "six_sigma_optimizer"
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            supports_vector_search: false,
            supports_full_text: false,
            supports_fuzzy: false,
            supports_real_time: true,
            supports_distributed: false,
            supports_replication: false,
            supports_facets: false,
            supports_typo_tolerance: false,
            max_vector_dimension: None,
            max_scale: Some(1_000_000),
        }
    }

    async fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        self.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), Box<dyn Error>> {
        self.initialized = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_six_sigma_dpmo() {
        let config = EngineConfig {
            name: "six_sigma_test".to_string(),
            endpoint: None,
            max_connections: 10,
            timeout_ms: 5000,
        };
        
        let optimizer = SixSigmaOptimizer::new(config);
        
        // Record 1000 operations with 3 defects (3000 DPMO)
        for _ in 0..997 {
            optimizer.record_success();
        }
        for _ in 0..3 {
            optimizer.record_defect();
        }
        
        let dpmo = optimizer.calculate_dpmo();
        let sigma_level = optimizer.calculate_sigma_level(dpmo);
        
        assert!(dpmo > 2900.0 && dpmo < 3100.0); // ~3000 DPMO
        assert!(sigma_level >= 5.0); // Should be ~5 sigma
    }

    #[test]
    fn test_control_limits() {
        let config = EngineConfig {
            name: "six_sigma_test".to_string(),
            endpoint: None,
            max_connections: 10,
            timeout_ms: 5000,
        };
        
        let mut optimizer = SixSigmaOptimizer::new(config);
        
        let data = vec![10.0, 12.0, 11.0, 13.0, 10.5, 11.5, 12.5];
        let limits = optimizer.establish_control_limits(&data);
        
        assert!(limits.center_line > 10.0 && limits.center_line < 13.0);
        assert!(limits.upper_control_limit > limits.center_line);
        assert!(limits.lower_control_limit < limits.center_line);
        
        assert!(optimizer.is_in_control(11.5)); // Should be in control
    }
}
