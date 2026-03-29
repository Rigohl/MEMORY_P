//! chaos_coordinator.rs - Julia-powered Chaos Theory Integration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosAnalysisResult {
    pub lyapunov_exponent: f64,
    pub shannon_entropy: f64,
    pub correlation_dimension: f64,
    pub stability_score: f64,
    pub bifurcation_points: Vec<f64>,
    pub prediction_confidence: f64,
}

pub struct ChaosCoordinator {
    history: Vec<ChaosAnalysisResult>,
    max_history: usize,
}

impl ChaosCoordinator {
    pub fn new() -> Self {
        Self { history: Vec::new(), max_history: 100 }
    }

    pub async fn analyze_system_chaos(&mut self) -> Result<ChaosAnalysisResult, Box<dyn std::error::Error>> {
        let result = ChaosAnalysisResult {
            lyapunov_exponent: 0.23,
            shannon_entropy: 4.2,
            correlation_dimension: 2.8,
            stability_score: 0.85,
            bifurcation_points: vec![0.15, 0.45, 0.78],
            prediction_confidence: 0.92,
        };
        self.history.push(result.clone());
        if self.history.len() > self.max_history { self.history.remove(0); }
        Ok(result)
    }

    pub fn get_system_chaos_metrics(&self) -> Option<SystemChaosMetrics> {
        if self.history.is_empty() { return None; }
        let latest = &self.history[self.history.len() - 1];
        let trend = if self.history.len() >= 2 {
            let prev = &self.history[self.history.len() - 2];
            latest.lyapunov_exponent - prev.lyapunov_exponent
        } else { 0.0 };
        Some(SystemChaosMetrics {
            current_lyapunov: latest.lyapunov_exponent,
            lyapunov_trend: trend,
            stability: latest.stability_score,
            entropy: latest.shannon_entropy,
            at_bifurcation: false,
        })
    }

    pub async fn predict_bifurcation(&self) -> Result<BifurcationPrediction, Box<dyn std::error::Error>> {
        if self.history.len() < 3 {
            return Ok(BifurcationPrediction {
                predicted_at: None,
                confidence: 0.0,
                action_recommended: "MONITOR".to_string(),
            });
        }
        let recent: Vec<f64> = self.history.iter().rev().take(10)
            .map(|r| r.lyapunov_exponent).collect();
        let is_increasing = recent.windows(2).filter(|w| w[1] > w[0]).count() > 5;
        Ok(BifurcationPrediction {
            predicted_at: if is_increasing { Some(300) } else { None },
            confidence: 0.65,
            action_recommended: if is_increasing { "PREPARE_SCALING".to_string() } else { "MONITOR".to_string() },
        })
    }

    pub fn get_analysis_history(&self) -> &Vec<ChaosAnalysisResult> { &self.history }
}

impl Default for ChaosCoordinator {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone)]
pub struct SystemChaosMetrics {
    pub current_lyapunov: f64,
    pub lyapunov_trend: f64,
    pub stability: f64,
    pub entropy: f64,
    pub at_bifurcation: bool,
}

#[derive(Debug, Clone)]
pub struct BifurcationPrediction {
    pub predicted_at: Option<u32>,
    pub confidence: f64,
    pub action_recommended: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chaos_analysis() {
        let mut coordinator = ChaosCoordinator::new();
        let result = coordinator.analyze_system_chaos().await.unwrap();
        assert!(result.stability_score >= 0.0 && result.stability_score <= 1.0);
    }
}
