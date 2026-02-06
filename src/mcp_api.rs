use crate::analyzer::CodeAnalyzer;
use crate::auto_manager::AutoManager; // Auto-gestión MCP 2026
use crate::error::MemoryPError;
use crate::kpi_tracker::KpiTracker; // KPI Tracking Six Sigma
use crate::mcp::handlers::*;
use crate::mcp::models::*;
use crate::parallel_engine::{self, ParallelConfig};

use axum::{
    extract::Extension,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;

pub fn routes() -> Router {
    Router::new()
        .route(
            "/",
            get(mcp_descriptor_handler).post(mcp_descriptor_handler),
        )
        .route("/mcp", post(mcp_json_rpc_handler))
        .route("/mcp/sse", get(mcp_sse_handler))
        .route("/mcp/status", get(auto_status_handler)) // Auto-Manager status
        .route("/mcp/health", get(auto_health_handler)) // Health check
        .route("/mcp/kpis", get(kpi_dashboard_handler)) // KPI Dashboard (Six Sigma)
        .route("/mcp/kpis/record", post(kpi_record_handler)) // Record metric
        .route("/create_project", post(create_project_handler))
        .route("/analyze_project", post(analyze_project_handler))
        .route("/edit_project", post(edit_project_handler))
        .route("/repair_project", post(repair_project_handler))
        .route("/ultra", post(ultra_engine_handler))
}

/// Handler para status del auto-manager (MCP 2026)
pub async fn auto_status_handler(
    Extension(auto_manager): Extension<Arc<AutoManager>>,
) -> Json<Value> {
    Json(auto_manager.get_detailed_status())
}

/// Handler para health check rápido (MCP 2026)
pub async fn auto_health_handler(
    Extension(auto_manager): Extension<Arc<AutoManager>>,
) -> Json<Value> {
    let health = auto_manager.get_overall_health();
    Json(json!({
        "status": format!("{:?}", health),
        "healthy": matches!(health, crate::auto_manager::HealthStatus::Healthy),
        "protocol_version": "2026.1.0",
        "auto_managed": true,
        "always_on": true
    }))
}

/// Handler para KPI Dashboard (Six Sigma)
pub async fn kpi_dashboard_handler(
    Extension(kpi_tracker): Extension<Arc<KpiTracker>>,
) -> Json<Value> {
    let dashboard = kpi_tracker.get_dashboard();
    
    Json(json!({
        "timestamp": dashboard.timestamp.timestamp(),
        "age_seconds": (Utc::now() - dashboard.timestamp).num_seconds(),
        "overall_sigma_level": dashboard.overall_sigma_level,
        "target_sigma": 4.0,
        "categories": dashboard.categories.iter().map(|cat| {
            json!({
                "category": format!("{:?}", cat.category),
                "metrics_count": cat.metrics_count,
                "avg_cpk": cat.avg_cpk,
                "defect_rate": cat.defect_rate,
                "sigma_level": cat.sigma_level,
                "status": if cat.sigma_level >= 4.0 { "excellent" }
                         else if cat.sigma_level >= 3.0 { "good" }
                         else { "needs_improvement" }
            })
        }).collect::<Vec<_>>(),
        "alerts": dashboard.alerts.iter().map(|alert| {
            json!({
                "severity": format!("{:?}", alert.severity),
                "category": format!("{:?}", alert.category),
                "message": alert.message,
                "age_seconds": (Utc::now() - alert.timestamp).num_seconds()
            })
        }).collect::<Vec<_>>(),
        "methodology": "Six Sigma DMAIC",
        "automation": "always-on"
    }))
}

/// Handler para registrar métrica
pub async fn kpi_record_handler(
    Extension(kpi_tracker): Extension<Arc<KpiTracker>>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    use crate::kpi_tracker::{KpiCategory, SixSigmaMetric};
    use std::time::Instant;
    
    // Parse request
    let name = payload["name"].as_str().unwrap_or("unknown").to_string();
    let value = payload["value"].as_f64().unwrap_or(0.0);
    let target = payload["target"].as_f64().unwrap_or(value);
    let usl = payload["upper_spec_limit"].as_f64().unwrap_or(target * 1.2);
    let lsl = payload["lower_spec_limit"].as_f64().unwrap_or(target * 0.8);
    let unit = payload["unit"].as_str().unwrap_or("").to_string();
    
    let category = match payload["category"].as_str().unwrap_or("performance") {
        "quality" => KpiCategory::Quality,
        "performance" => KpiCategory::Performance,
        "availability" => KpiCategory::Availability,
        "efficiency" => KpiCategory::Efficiency,
        "defects" => KpiCategory::Defects,
        "cost" => KpiCategory::Cost,
        _ => KpiCategory::Performance,
    };
    
    let metric = SixSigmaMetric {
        name: name.clone(),
        category,
        value,
        target,
        upper_spec_limit: usl,
        lower_spec_limit: lsl,
        timestamp: Utc::now(),
        unit,
    };
    
    kpi_tracker.record_metric(metric.clone());
    
    Json(json!({
        "status": "recorded",
        "metric": name,
        "value": value,
        "within_spec": metric.is_within_spec(),
        "category": format!("{:?}", category)
    }))
}

pub async fn mcp_json_rpc_handler(Json(req): Json<JsonRpcRequest>) -> Json<JsonRpcResponse> {
    let id = req.id.clone().unwrap_or(Value::Null);

    if req.jsonrpc != "2.0" {
        let err = MemoryPError::InvalidParams("Invalid JSON-RPC version".to_string());
        return Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(json!({ "code": -32600, "message": format!("{}", err) })),
        });
    }

    let method = req.method.as_str();

    let result = match method {
        "initialize" => Some(json!({
            "protocolVersion": "2026.1.0",
            "capabilities": {
                "tools": { "listChanged": true },
                "resources": { "listChanged": true, "subscribe": true },
                "prompts": { "listChanged": true },
                "experimental": {
                    "ffiEnabled": true,
                    "autoManaged": true,
                    "alwaysOn": true,
                    "multiLanguage": ["julia", "jax", "mojo", "pony", "zig"],
                    "ffi": {
                        "julia": {
                            "status": "active",
                            "version": "1.10.0",
                            "features": ["optimization", "chaos_analysis", "differential_equations"]
                        },
                        "jax": {
                            "status": "active",
                            "version": "0.4.23",
                            "features": ["embeddings", "gpu_inference", "parallelism"]
                        },
                        "mojo": {
                            "status": "active",
                            "version": "0.6.0",
                            "features": ["simd_kernels", "dot_products", "vectorization"]
                        },
                        "pony": {
                            "status": "active",
                            "version": "0.54.0",
                            "features": ["actor_system", "distributed_search", "zero_copy"]
                        },
                        "zig": {
                            "status": "active",
                            "version": "0.11.0",
                            "features": ["ffi_bridge", "memory_safety", "c_interop"]
                        }
                    },
                    "autoManagement": {
                        "healthChecks": true,
                        "autoRecovery": true,
                        "resourceOptimization": true,
                        "predictiveMaintenance": true
                    }
                }
            },
            "serverInfo": {
                "name": "MEMORY_P MCP Server",
                "version": "2.0.0-ALWAYS-ON"
            }
        })),
        "tools/list" | "listTools" => {
            let tools = vec![
                // === TOOL 1: analyze (combines ultra_analyze + ultra_overview) ===
                Tool {
                    name: "analyze".to_string(),
                    description: "🔬 Análisis masivo paralelo con métricas, seguridad y overview arquitectónico.".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "Ruta al proyecto" },
                            "mode": { "type": "string", "enum": ["deep", "quick", "overview"], "description": "deep=completo, quick=rápido, overview=arquitectura" },
                            "extension": { "type": "string", "default": "rs" },
                            "use_gitignore": { "type": "boolean", "default": true },
                            "include_hidden": { "type": "boolean", "default": false }
                        },
                        "required": ["path"]
                    }),
                    annotations: None,
                },
                // === TOOL 2: repair ===
                Tool {
                    name: "repair".to_string(),
                    description: "🛠️ Reparación paralela: imports duplicados, formato, EOL, espacios.".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "path": { "type": "string" },
                            "extension": { "type": "string", "default": "rs" },
                            "dry_run": { "type": "boolean", "default": false }
                        },
                        "required": ["path"]
                    }),
                    annotations: None,
                },
                // === TOOL 3: edit (combines ultra_edit + ultra_delete) ===
                Tool {
                    name: "edit".to_string(),
                    description: "✏️ Edición masiva atómica: replace, regex, append, delete.".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "mode": { "type": "string", "enum": ["replace", "regex", "append", "delete"], "description": "Tipo de operación" },
                            "changes": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "path": { "type": "string" },
                                        "operations": { "type": "array" }
                                    }
                                }
                            },
                            "paths": { "type": "array", "items": { "type": "string" }, "description": "Para mode=delete" },
                            "dry_run": { "type": "boolean", "default": true }
                        },
                        "required": ["mode"]
                    }),
                    annotations: None,
                },
                // === TOOL 4: workflow (with Evolve + Repair steps) ===
                Tool {
                    name: "workflow".to_string(),
                    description: "🌊 Pipeline: Scan → Filter → Analyze → Edit → Repair → Evolve (auto-fix loop).".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "steps": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "action": { "type": "string", "enum": ["Scan", "Filter", "Analyze", "Edit", "Repair", "Evolve"] },
                                        "params": { "type": "object" }
                                    }
                                }
                            },
                            "dry_run": { "type": "boolean", "default": true },
                            "max_threads": { "type": "integer" }
                        },
                        "required": ["steps"]
                    }),
                    annotations: None,
                },
                // === TOOL 5: simulate (3 phases: 15K/150K/500K) ===
                Tool {
                    name: "simulate".to_string(),
                    description: "🌀 Mega simulaciones: Phase1=15K/módulo, Phase2=150K paralelismo, Phase3=500K ecosystem.".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "phase": { "type": "integer", "enum": [1, 2, 3], "description": "1=módulos, 2=paralelismo, 3=ecosystem" },
                            "iterations": { "type": "integer", "default": 1000, "description": "Simulaciones por config" },
                            "modules": { "type": "array", "items": { "type": "string" }, "description": "Para phase 1" },
                            "use_gpu": { "type": "boolean", "default": false },
                            "name": { "type": "string", "description": "Nombre de simulación custom" },
                            "logic": { "type": "string", "description": "Código Bend custom" }
                        },
                        "required": ["phase"]
                    }),
                    annotations: None,
                },
            ];
            Some(json!({ "tools": tools }))
        }
        "tools/call" | "callTool" => {
            let params = req.params.as_ref().unwrap();
            let tool_name = params.get("name").unwrap().as_str().unwrap();
            let arguments = params.get("arguments").unwrap();

            match tool_name {
                // === HANDLER 1: analyze (deep/quick/overview) ===
                "analyze" => {
                    let path = arguments
                        .get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or(".");
                    let mode = arguments
                        .get("mode")
                        .and_then(|v| v.as_str())
                        .unwrap_or("deep");
                    let ext = arguments
                        .get("extension")
                        .and_then(|v| v.as_str())
                        .unwrap_or("rs");
                    let use_gitignore = arguments
                        .get("use_gitignore")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);
                    let include_hidden = arguments
                        .get("include_hidden")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    match mode {
                        "overview" => {
                            let cargo_path = std::path::Path::new(path).join("Cargo.toml");
                            let total_files =
                                CodeAnalyzer::scan_files(path, ext, use_gitignore, include_hidden)
                                    .map(|f| f.len())
                                    .unwrap_or(0);
                            let has_cargo = cargo_path.exists();
                            Some(json!({ "content": [{ "type": "text", "text": format!(
                                "🏛️ Overview: {} | Files: {} | Cargo.toml: {}",
                                path, total_files, if has_cargo { "✅" } else { "❌" }
                            )}]}))
                        }
                        _ => {
                            let config = ParallelConfig::default();
                            match CodeAnalyzer::scan_files(path, ext, use_gitignore, include_hidden)
                            {
                                Ok(files) => match parallel_engine::ultra_analyze(&files, config) {
                                    Ok((_res, stats)) => Some(json!({
                                        "content": [{ "type": "text", "text": format!(
                                            "🔬 Analyze [{}] en {}ms. Archivos: {} (exitosos: {})",
                                            mode, stats.total_duration_ms, stats.total_files, stats.successful
                                        )}]
                                    })),
                                    Err(e) => Some(
                                        json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }] }),
                                    ),
                                },
                                Err(e) => Some(
                                    json!({ "content": [{ "type": "text", "text": format!("Scan Error: {}", e) }] }),
                                ),
                            }
                        }
                    }
                }
                // === HANDLER 2: repair ===
                "repair" => {
                    let path = arguments
                        .get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or(".");
                    let ext = arguments
                        .get("extension")
                        .and_then(|v| v.as_str())
                        .unwrap_or("rs");
                    let config = ParallelConfig::default();

                    match CodeAnalyzer::scan_files(path, ext, true, false) {
                        Ok(files) => match parallel_engine::ultra_repair(&files, config) {
                            Ok((_res, stats)) => Some(json!({
                                "content": [{ "type": "text", "text": format!(
                                    "🛠️ Repair en {}ms. Archivos: {} (reparados: {})",
                                    stats.total_duration_ms, stats.total_files, stats.successful
                                )}]
                            })),
                            Err(e) => Some(
                                json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }] }),
                            ),
                        },
                        Err(e) => Some(
                            json!({ "content": [{ "type": "text", "text": format!("Scan Error: {}", e) }] }),
                        ),
                    }
                }
                // === HANDLER 3: edit (replace/regex/append/delete) ===
                "edit" => {
                    let mode = arguments
                        .get("mode")
                        .and_then(|v| v.as_str())
                        .unwrap_or("replace");
                    let dry_run = arguments
                        .get("dry_run")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);

                    if mode == "delete" {
                        // Delete mode
                        let paths_raw = arguments.get("paths").and_then(|v| v.as_array());
                        let mut paths: Vec<PathBuf> = Vec::new();
                        if let Some(arr) = paths_raw {
                            for p in arr {
                                if let Some(s) = p.as_str() {
                                    paths.push(PathBuf::from(s));
                                }
                            }
                        }
                        let config = ParallelConfig::default();
                        match parallel_engine::ultra_delete(&paths, config, dry_run) {
                            Ok((_res, stats)) => Some(json!({
                                "content": [{ "type": "text", "text": format!(
                                    "🗑️ Delete {} en {}ms. Archivos: {} (eliminados: {})",
                                    if dry_run { "[DRY]" } else { "[REAL]" },
                                    stats.total_duration_ms, stats.total_files, stats.successful
                                )}]
                            })),
                            Err(e) => Some(
                                json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }] }),
                            ),
                        }
                    } else {
                        // Edit mode (replace/regex/append)
                        match serde_json::from_value::<UltraEditRequest>(arguments.clone()) {
                            Ok(req) => {
                                let app_cfg = crate::config::AppConfig::load();
                                let config = app_cfg.to_parallel_config();
                                match parallel_engine::ultra_edit(&req.changes, config, dry_run) {
                                    Ok((_res, stats)) => Some(json!({
                                        "content": [{ "type": "text", "text": format!(
                                            "✏️ Edit [{}] {} en {}ms. Archivos: {}",
                                            mode, if dry_run { "[DRY]" } else { "[APPLIED]" },
                                            stats.total_duration_ms, stats.total_files
                                        )}]
                                    })),
                                    Err(e) => Some(
                                        json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }] }),
                                    ),
                                }
                            }
                            Err(e) => Some(
                                json!({ "content": [{ "type": "text", "text": format!("Invalid params: {}", e) }] }),
                            ),
                        }
                    }
                }
                // === HANDLER 4: workflow (with Evolve) ===
                "workflow" => {
                    match serde_json::from_value::<UltraWorkflowRequest>(arguments.clone()) {
                        Ok(req) => {
                            let app_cfg = crate::config::AppConfig::load();
                            let mut config = app_cfg.to_parallel_config();
                            if let Some(max_tasks) = req.max_tasks {
                                config.max_threads = max_tasks as usize;
                            }
                            match parallel_engine::ultra_workflow(&req, config) {
                                Ok((_res, stats)) => Some(json!({
                                    "content": [{ "type": "text", "text": format!(
                                        "🌊 Workflow en {}ms. Pasos: {} (exitosos: {})",
                                        stats.total_duration_ms, req.steps.len(), stats.successful
                                    )}]
                                })),
                                Err(e) => Some(
                                    json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }] }),
                                ),
                            }
                        }
                        Err(e) => Some(
                            json!({ "content": [{ "type": "text", "text": format!("Invalid params: {}", e) }] }),
                        ),
                    }
                }
                // === HANDLER 5: simulate (3 phases) ===
                "simulate" => {
                    let phase = arguments.get("phase").and_then(|v| v.as_i64()).unwrap_or(1);
                    let iterations = arguments
                        .get("iterations")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(1000) as usize;
                    let use_gpu = arguments
                        .get("use_gpu")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    // Phase-based mega simulation with actual execution
                    let config = crate::mega_simulator::SimConfig {
                            phase: phase as u8,
                            iterations,
                            modules: arguments
                                .get("modules")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(String::from))
                                        .collect()
                                })
                                .unwrap_or_default(),
                            use_gpu,
                            context7_enabled: true,
                        };

                        match crate::mega_simulator::run_mega_simulation(config) {
                            Ok(result) => {
                                // Save results to file
                                let result_path = format!("phase{}_results.json", phase);
                                let _ = crate::mega_simulator::save_results(
                                    &result,
                                    std::path::Path::new(&result_path),
                                );

                                let improvements_summary: Vec<String> = result
                                    .improvements
                                    .iter()
                                    .map(|i| {
                                        format!(
                                            "{}: {:.1}% improvement",
                                            i.target, i.improvement_pct
                                        )
                                    })
                                    .collect();

                                Some(json!({ "content": [{ "type": "text", "text": format!(
                                    "🌀 Phase {} Complete!\n⏱️ {}ms | 📊 {}/{} sims\n\n📈 Improvements:\n{}",
                                    result.phase,
                                    result.duration_ms,
                                    result.completed,
                                    result.total_sims,
                                    improvements_summary.join("\n")
                                )}]}))
                            }
                            Err(e) => Some(
                                json!({ "content": [{ "type": "text", "text": format!("Sim Error: {}", e) }] }),
                            ),
                        }
                }
                _ => Some(json!({ "content": [{ "type": "text", "text": "Tool no encontrada" }] })),
            }
        }
        _ => None,
    };

    Json(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result,
        error: None,
    })
}
