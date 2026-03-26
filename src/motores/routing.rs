//! Motor Routing AI with Chaos-Driven Intelligence
//! Intelligent routing between 9 search engines based on MATHEMATICAL caos metrics
//! 
//! ARQUITECTURA DE RESPUESTA INMEDIATA:
//! 1. Usuario escribe prompt + [ENTER]
//! 2. route_query() calcula entropía del query
//! 3. Llama a Julia get_decision_ffi() (chaos metrics)
//! 4. Julia (o fallback Rust) decide motor óptimo
//! 5. Respuesta INMEDIATA + DETERMINÍSTICA (no random)
//!
//! Decisiones basadas en:
//! - entropy: incertidumbre del query
//! - lyapunov: caos del sistema
//! - stability: estabilidad del contexto

use crate::ffi;
use crate::prediction_engine::PredictionEngine;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct RoutingAI;

impl RoutingAI {
    /// Create routing AI instance
    /// KEPT SUPPRESSION: Factory method used by motor orchestrator
    /// Instantiates intelligent query router for 9-motor selection
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    /// CRÍTICO: Route query using CHAOS MATHEMATICS
    /// 
    /// Flujo:
    /// 1. Calculate query entropy (información contenida)
    /// 2. Get Julia chaos metrics (Lyapunov exponent + stability)
    /// 3. Call julia_get_decision_ffi(entropy, lyapunov, stability)
    /// 4. Map decision to optimal motor combination
    ///
    /// Returns: Ordered vector of motors (best first)
    #[allow(dead_code)]
    pub async fn route_query(&self, query: &str) -> Vec<String> {
        debug!("🎯 Routing query: {} (length: {})", query, query.len());

        // STEP 1: Calculate query entropy
        let entropy = calculate_shannon_entropy(query);
        debug!("📊 Query entropy: {:.4}", entropy);

        // STEP 2: Get current chaos metrics from prediction engine
        let engine = PredictionEngine::new();
        let chaos_metrics = engine.get_chaos_metrics();
        debug!(
            "🌀 Chaos metrics: λ={:.4}, D_c={:.4}, H={:.4}",
            chaos_metrics.lyapunov_exponent,
            chaos_metrics.correlation_dimension,
            chaos_metrics.entropy_rate
        );

        // STEP 3: Call Julia for IMMEDIATE routing decision
        let motor_decision = ffi::julia::get_decision_ffi(
            entropy,
            chaos_metrics.lyapunov_exponent,
            chaos_metrics.exploration_factor,
        )
        .await;

        debug!("🧬 Julia routing decision: {}", motor_decision);

        // STEP 4: Map decision to motors (9-motor selection)
        let motors = match motor_decision.as_str() {
            "HYBRID_FUSION" => {
                // High uncertainty: Use MULTIPLE motors in parallel (9/9)
                // Fuses results from multiple approaches
                info!("✨ HYBRID_FUSION: Consulting all 9 motors in parallel");
                vec![
                    "qdrant",     // Vector primary
                    "faiss",      // Vector fallback (GPU)
                    "scann",      // Learned indexing
                    "tantivy",    // Text primary
                    "lnx",        // Distributed text
                    "meilisearch", // Typo-tolerant
                    "toshi",      // Experimental
                    "julia_nlp",  // Math analysis
                    "memory_bank", // Persistence
                ]
            }
            "VECTOR_QDRANT" => {
                // Chaotic system: Semantic/vector search prioritized (3/9)
                info!("🌀 VECTOR_QDRANT: Semantic search optimized for chaos");
                vec!["qdrant", "faiss", "scann"]
            }
            "TEXT_TANTIVY" => {
                // Stable system: Exact/text match sufficient (3/9)
                info!("✏️ TEXT_TANTIVY: Text search optimized for stability");
                vec!["tantivy", "lnx", "meilisearch"]
            }
            "HYBRID_BALANCED" | _ => {
                // Default: Balanced approach (2+2+1)
                info!("⚖️ HYBRID_BALANCED: Mixed strategy for edge-of-chaos");
                vec![
                    "qdrant",
                    "tantivy",
                    "scann",
                    "memory_bank",
                    "julia_nlp",
                ]
            }
        };

        info!(
            "🚀 Selected motors: {} (decision: {})",
            motors.join(", "),
            motor_decision
        );

        motors.iter().map(|s| s.to_string()).collect()
    }

    /// Select BEST engine (primary)
    /// (Uses first engine from route_query result)
    #[allow(dead_code)]
    pub async fn select_best_engine(&self, query: &str) -> String {
        let motors = self.route_query(query).await;
        motors.first().cloned().unwrap_or_else(|| "memory_bank".to_string())
    }
}

impl Default for RoutingAI {
    fn default() -> Self {
        Self::new()
    }
}

/// Shannon Entropy Calculator
/// Measures query uncertainty (información content)
/// 
/// High entropy (>2.5): Query is ambiguous/uncertain → HYBRID_FUSION
/// Low entropy (<1.0): Query is specific/clear → Specialized search
#[inline]
fn calculate_shannon_entropy(text: &str) -> f64 {
    if text.is_empty() {
        return 0.0;
    }

    // Frequency analysis
    let mut freq = [0u32; 256];
    for byte in text.bytes() {
        freq[byte as usize] += 1;
    }

    let len = text.len() as f64;
    let mut entropy = 0.0;

    for count in &freq {
        if *count > 0 {
            let p = *count as f64 / len;
            entropy -= p * p.log2();
        }
    }

    entropy
}
