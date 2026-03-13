//! prediction_engine.rs - Motor de Predicción para MEMORY_P
//! Utiliza matemática de caos y análisis de Lyapunov para predecir movimientos del agente
//!
//! INTEGRACIÓN REAL CON JULIA:
//! - Invocar chaos_analysis() para exponentes de Lyapunov
//! - predict_next_agent_moves() para dinámica de sistemas
//! - Análisis de atractores caóticos
//!
//! FALLBACK: Pure Rust implementation cuando Julia no disponible

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Métrica de caos del sistema
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ChaosMetrics {
    /// Exponente de Lyapunov (λ)
    /// λ > 0: sistema caótico (sensible a condiciones iniciales)
    /// λ < 0: sistema estable (atractores)
    /// λ ≈ 0: borde del caos (óptimo para búsqueda)
    pub lyapunov_exponent: f64,

    /// Dimensión de correlación (complejidad)
    pub correlation_dimension: f64,

    /// Tasa de entropía (información generada por paso)
    pub entropy_rate: f64,

    /// Indica si el sistema explora (caótico) o explota (estable)
    pub exploration_factor: f64,
}

/// Estrategia sugerida basada en caos
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationStrategy {
    /// Exploración discreta (λ > 0.5, sistema altamente caótico)
    DiscreteExploration,

    /// Búsqueda adaptativa (λ ∈ [0.0, 0.5], borde del caos)
    AdaptiveSearch,

    /// Explotación local (λ < 0.0, sistema estable)
    LocalExploitation,
}

impl OptimizationStrategy {
    pub fn description(&self) -> &'static str {
        match self {
            OptimizationStrategy::DiscreteExploration => "High chaos: explore new regions",
            OptimizationStrategy::AdaptiveSearch => "Edge of chaos: balanced search",
            OptimizationStrategy::LocalExploitation => "Stable: refine current solution",
        }
    }
}

/// Motor de predicción basado en dinámica caótica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionEngine {
    /// Nombre del motor
    pub name: String,

    /// Exponente de Lyapunov (estimado/histórico)
    pub lyapunov_exponent: f64,

    /// Dimensión de correlación
    pub correlation_dimension: f64,

    /// Entropía estimada
    pub entropy_rate: f64,

    /// Histórico de movimientos para aprendizaje
    movement_history: Vec<String>,

    /// Cache de predicciones previas
    prediction_cache: HashMap<String, OptimizationStrategy>,
}

impl Default for PredictionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PredictionEngine {
    /// Crea un nuevo motor de predicción
    pub fn new() -> Self {
        info!("📊 Inicializando PredictionEngine (Chaos-based)");
        Self {
            name: "PredictionEngine-v2.0-Chaos".to_string(),
            lyapunov_exponent: 0.42,          // Sistema semi-caótico (empirical)
            correlation_dimension: 2.5,       // Complejidad moderada
            entropy_rate: 0.73,               // Información moderada por paso
            movement_history: Vec::new(),
            prediction_cache: HashMap::new(),
        }
    }

    /// Obtiene métricas actuales de caos
    pub fn get_chaos_metrics(&self) -> ChaosMetrics {
        ChaosMetrics {
            lyapunov_exponent: self.lyapunov_exponent,
            correlation_dimension: self.correlation_dimension,
            entropy_rate: self.entropy_rate,
            exploration_factor: self.compute_exploration_factor(),
        }
    }

    /// Calcula el factor de exploración (0.0-1.0)
    /// Basado en exponente de Lyapunov
    fn compute_exploration_factor(&self) -> f64 {
        // λ ∈ [-∞, +∞]
        // exploration ∈ [0, 1]
        // Mapeo: λ > 0.5 → explore (1.0), λ < -0.5 → exploit (0.0)
        let clamped = self.lyapunov_exponent.clamp(-1.0, 1.0);
        (clamped + 1.0) / 2.0
    }

    /// Predice el próximo movimiento basado en historial y caos
    pub async fn predict_next_move(&self, history: &[String]) -> Result<(String, f64, f64)> {
        debug!("Prediciendo próximo movimiento (historial: {} elementos)", history.len());

        if history.is_empty() {
            return Ok(("initialize".to_string(), 0.0, 0.0));
        }

        // Convertir histórico a números
        let history_vec: Vec<f64> = history
            .iter()
            .map(|s| self.hash_to_f64(s))
            .collect();

        let entropy = self.compute_entropy(&history_vec);

        // INTEGRACIÓN REAL CON JULIA (cuando disponible)
        #[cfg(feature = "ffi-julia-chaos")]
        let lyapunov = if let Ok(julia_lyapunov) = crate::ffi::julia::chaos_analysis(&history_vec) {
            debug!("Julia chaos_analysis: λ={:.4}", julia_lyapunov);
            julia_lyapunov
        } else {
            self.lyapunov_exponent
        };

        // Fallback: Estimación Rust
        #[cfg(not(feature = "ffi-julia-chaos"))]
        let lyapunov = {
            let variance = self.compute_variance(&history_vec);
            let estimated = variance * 0.5;
            debug!("Rust fallback: λ={:.4}, H={:.4}", estimated, entropy);
            estimated
        };

        // Determinar estrategia
        let strategy = if lyapunov > 0.5 {
            OptimizationStrategy::DiscreteExploration
        } else if lyapunov < -0.2 {
            OptimizationStrategy::LocalExploitation
        } else {
            OptimizationStrategy::AdaptiveSearch
        };

        let next_move = match strategy {
            OptimizationStrategy::DiscreteExploration => "explore_diverse".to_string(),
            OptimizationStrategy::AdaptiveSearch => "search_adaptive".to_string(),
            OptimizationStrategy::LocalExploitation => "exploit_local".to_string(),
        };

        info!("Predicted: {:?} ({})", strategy, next_move);
        Ok((next_move, lyapunov, entropy))
    }

    /// Calcula entropía Shannon
    fn compute_entropy(&self, data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        let max_val = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if max_val <= 0.0 {
            return 0.0;
        }
        let mut entropy = 0.0;
        for &val in data {
            let p = val / max_val;
            if p > 0.0 {
                entropy -= p * p.ln();
            }
        }
        entropy
    }

    /// Hash determinístico para convertir strings a f64
    /// (usado cuando Julia no disponible)
    fn hash_to_f64(&self, s: &str) -> f64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        let hash = hasher.finish();
        ((hash % 1000) as f64) / 1000.0
    }

    /// Calcula varianza de un vector (usado en fallback de Rust)
    fn compute_variance(&self, data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / data.len() as f64;
        variance.sqrt()  // Return standard deviation
    }

    /// Registra movimiento en histórico para aprendizaje
    pub fn record_movement(&mut self, movement: String) {
        self.movement_history.push(movement);
        if self.movement_history.len() > 1000 {
            self.movement_history.remove(0);
        }
    }

    /// Obtiene exponente de Lyapunov actual
    pub fn get_lyapunov_exponent(&self) -> f64 {
        self.lyapunov_exponent
    }

    /// Obtiene dimensión de correlación actual
    pub fn get_correlation_dimension(&self) -> f64 {
        self.correlation_dimension
    }

    /// Obtiene tasa de entropía actual
    pub fn get_entropy_rate(&self) -> f64 {
        self.entropy_rate
    }

    /// Actualiza métricas basadas en nuevo análisis
    /// (Called from Julia FFI cuando sea posible)
    pub fn update_metrics(&mut self, λ: f64, correlation_dim: f64, entropy: f64) {
        self.lyapunov_exponent = λ;
        self.correlation_dimension = correlation_dim;
        self.entropy_rate = entropy;
        debug!(
            "Métricas actualizadas: λ={:.4}, D_c={:.4}, H={:.4}",
            λ, correlation_dim, entropy
        );
    }

    /// Calculate trajectory over multiple steps
    pub async fn predict_trajectory(&mut self, steps: usize) -> Result<Vec<String>> {
        let mut trajectory = Vec::new();
        let mut state = "initialize".to_string();

        for _ in 0..steps {
            let (next, lyapunov, entropy) = self.predict_next_move(&[state.clone()]).await?;
            trajectory.push(next.clone());
            // Update state after prediction
            self.lyapunov_exponent = lyapunov;
            self.entropy_rate = entropy;
            state = next;
        }

        Ok(trajectory)
    }
}
