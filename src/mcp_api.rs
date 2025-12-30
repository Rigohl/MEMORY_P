use crate::analyzer::CodeAnalyzer;
use crate::error::MemoryPError;
use crate::mcp::handlers::*;
use crate::mcp::models::*;
use crate::parallel_engine::{self, ParallelConfig};

use axum::{
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::path::PathBuf;

pub fn routes() -> Router {
    Router::new()
        .route(
            "/",
            get(mcp_descriptor_handler).post(mcp_descriptor_handler),
        )
        .route("/mcp", post(mcp_json_rpc_handler))
        .route("/mcp/sse", get(mcp_sse_handler))
        .route("/create_project", post(create_project_handler))
        .route("/analyze_project", post(analyze_project_handler))
        .route("/edit_project", post(edit_project_handler))
        .route("/repair_project", post(repair_project_handler))
        .route("/ultra", post(ultra_engine_handler))
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
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": { "listChanged": true },
                "resources": { "listChanged": true, "subscribe": true },
                "prompts": { "listChanged": true },
                "logging": {}
            },
            "serverInfo": {
                "name": "MEMORY_P_ULTRA",
                "version": "2025.2.ULTRA",
                "description": "Motor de procesamiento masivo paralelo nativo para Cursor, Windsurf y VS Code."
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

                    // Check for custom simulation
                    if let (Some(name), Some(logic)) = (
                        arguments.get("name").and_then(|v| v.as_str()),
                        arguments.get("logic").and_then(|v| v.as_str()),
                    ) {
                        match crate::simulation_engine::run_bend_simulation(
                            name,
                            logic,
                            &json!({}),
                            use_gpu,
                        ) {
                            Ok(output) => Some(
                                json!({ "content": [{ "type": "text", "text": format!("🌀 Custom Sim:\n{}", output) }] }),
                            ),
                            Err(e) => Some(
                                json!({ "content": [{ "type": "text", "text": format!("Sim Error: {}", e) }] }),
                            ),
                        }
                    } else {
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
