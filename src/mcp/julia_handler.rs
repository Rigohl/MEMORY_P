/// MCP Handler for Julia Mathematical Optimization
/// Connects MCP protocol to brain/julia/julia_math.jl via FFI
/// 
/// Tool: julia_optimize
/// Endpoint: POST /mcp/tools/call
/// Request: { "method": "tools/call", "params": { "name": "julia_optimize", "arguments": { "weights": [0.33, 0.33, 0.34] } } }
/// Response: { "result": { "optimized": [0.41, 0.29, 0.30], "improvement": 12.5 } }

use crate::ffi::julia;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct JuliaOptimizeRequest {
    pub weights: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JuliaOptimizeResponse {
    pub optimized: Vec<f64>,
    pub improvement_percent: f64,
}

/// MCP Tool Handler: julia_optimize
/// Calls julia::optimize_chaotic_system() which invokes julia_optimize_weights_ffi()
pub async fn julia_optimize_handler(
    Json(request): Json<JuliaOptimizeRequest>,
) -> Json<serde_json::Value> {
    tracing::info!(
        "[MCP Julia] Received optimize request with {} weights",
        request.weights.len()
    );

    // Call real Julia FFI
    match julia::optimize_chaotic_system(&request.weights) {
        Ok(optimized) => {
            // Calculate improvement metric
            let original_variance: f64 = request
                .weights
                .iter()
                .map(|w: &f64| (w - 0.33).powi(2))
                .sum::<f64>()
                / request.weights.len() as f64;

            let optimized_variance: f64 = optimized
                .iter()
                .map(|w| (w - 0.33).powi(2))
                .sum::<f64>()
                / optimized.len() as f64;

            let improvement = if original_variance > 0.0 {
                ((original_variance - optimized_variance) / original_variance * 100.0).max(0.0)
            } else {
                0.0
            };

            tracing::info!(
                "[MCP Julia] ✅ Optimization succeeded: improvement={:.1}%",
                improvement
            );

            Json(json!({
                "status": "success",
                "optimized": optimized,
                "improvement_percent": improvement,
                "original": request.weights,
            }))
        }
        Err(e) => {
            tracing::error!("[MCP Julia] ❌ Optimization failed: {}", e);
            Json(json!({
                "status": "error",
                "error": format!("{}", e),
                "optimized": request.weights.clone(),
            }))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JuliaChaosRequest {
    pub timeseries: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JuliaChaosResponse {
    pub lyapunov_exponent: f64,
    pub is_chaotic: bool,
}

/// MCP Tool Handler: julia_chaos_analyze
/// Calls julia::analyze_dynamics() which invokes julia_chaos_analysis_ffi()
pub async fn julia_chaos_handler(
    Json(request): Json<JuliaChaosRequest>,
) -> Json<serde_json::Value> {
    tracing::info!(
        "[MCP Julia] Received chaos analysis request with {} points",
        request.timeseries.len()
    );

    match julia::analyze_dynamics(&request.timeseries) {
        Ok(lyapunov) => {
            let is_chaotic = lyapunov > 0.0;
            tracing::info!(
                "[MCP Julia] ✅ Chaos analysis succeeded: lyapunov={:.4}, chaotic={}",
                lyapunov,
                is_chaotic
            );

            Json(json!({
                "status": "success",
                "lyapunov_exponent": lyapunov,
                "is_chaotic": is_chaotic,
            }))
        }
        Err(e) => {
            tracing::error!("[MCP Julia] ❌ Chaos analysis failed: {}", e);
            Json(json!({
                "status": "error",
                "error": format!("{}", e),
            }))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JuliaDecisionRequest {
    pub entropy: f64,
    pub chaos: f64,
    pub stability: f64,
}

/// MCP Tool Handler: julia_decision
/// Calls julia::get_search_decision() which invokes julia_get_decision_ffi()
pub async fn julia_decision_handler(
    Json(request): Json<JuliaDecisionRequest>,
) -> Json<serde_json::Value> {
    tracing::info!(
        "[MCP Julia] Received decision request: entropy={:.3}, chaos={:.3}, stability={:.3}",
        request.entropy,
        request.chaos,
        request.stability
    );

    match julia::get_search_decision(request.entropy, request.chaos, request.stability) {
        Ok(decision) => {
            tracing::info!("[MCP Julia] ✅ Decision made: {}", decision);

            Json(json!({
                "status": "success",
                "decision": decision,
                "metrics": {
                    "entropy": request.entropy,
                    "chaos": request.chaos,
                    "stability": request.stability,
                }
            }))
        }
        Err(e) => {
            tracing::error!("[MCP Julia] ❌ Decision failed: {}", e);

            Json(json!({
                "status": "error",
                "error": format!("{}", e),
                "decision": "HYBRID_BALANCED"  // Safe fallback
            }))
        }
    }
}
