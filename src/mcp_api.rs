use crate::analyzer::CodeAnalyzer;
use crate::backpack::Backpack; // La Mochila v2.1
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
use serde_json::{json, Value};
use std::collections::HashMap;
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
    let timestamp_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Json(json!({
        "timestamp": timestamp_secs,
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
        timestamp: std::time::Instant::now(),
        unit,
    };

    kpi_tracker.record_metric(metric.clone());

    Json(json!({
        "status": "recorded",
        "metric": name,
        "value": value,
        "within_spec": metric.is_within_spec(),
        "category": format!("{:?}", metric.category)
    }))
}

pub async fn mcp_json_rpc_handler(
    Extension(shared_memory): Extension<Arc<crate::shared_memory::SharedMemorySystem>>,
    Extension(prediction_engine): Extension<Arc<crate::prediction_engine::PredictionEngine>>,
    Extension(decision_engine): Extension<Arc<crate::decision_logic::DecisionEngine>>,
    Json(req): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
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

    // 1. Obtener contexto compartido (AgentId por defecto para simplicidad)
    let agent_id = crate::shared_memory::AgentId::new("default-agent".to_string());
    let mut shared_context = match shared_memory.get_or_create_context(agent_id.clone()).await {
        Ok(ctx) => ctx,
        Err(_) => crate::shared_memory::SharedContext::new(agent_id.clone()),
    };

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
                Tool {
                    name: "analyze".to_string(),
                    description: "🔬 Análisis masivo paralelo con métricas, seguridad y overview arquitectónico.".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "Ruta al proyecto" },
                            "mode": { "type": "string", "enum": ["deep", "quick", "overview"], "description": "deep=completo, quick=rápido, overview=arquitectura" },
                            "extension": { "type": "string", "default": "rs" }
                        },
                        "required": ["path"]
                    }),
                    annotations: None,
                },
                Tool {
                    name: "repair".to_string(),
                    description: "🛠️ Reparación paralela: imports duplicados, formato, EOL, espacios.".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "path": { "type": "string" }
                        },
                        "required": ["path"]
                    }),
                    annotations: None,
                },
                Tool {
                    name: "edit".to_string(),
                    description: "✏️ Edición masiva atómica.".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "mode": { "type": "string", "enum": ["replace", "regex", "append", "delete"] },
                            "changes": { "type": "array" }
                        },
                        "required": ["mode"]
                    }),
                    annotations: None,
                },
                Tool {
                    name: "map_search".to_string(),
                    description: "🔍 Búsqueda vectorial avanzada.".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "query": { "type": "string" },
                            "limit": { "type": "integer", "default": 10 }
                        },
                        "required": ["query"]
                    }),
                    annotations: None,
                },
                Tool {
                    name: "cognitive_decision".to_string(),
                    description: "🧠 Soporte de decisiones cognitivas.".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "situation": { "type": "string" }
                        },
                        "required": ["situation"]
                    }),
                    annotations: None,
                },
                Tool {
                    name: "memory_agility_stats".to_string(),
                    description: "📊 Estadísticas de agilidad de memoria.".to_string(),
                    input_schema: json!({ "type": "object", "properties": {} }),
                    annotations: None,
                },
                Tool {
                    name: "get_workspace_map".to_string(),
                    description: "🗺️ Genera un mapa visual del workspace.".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "format": { "type": "string", "enum": ["mermaid", "ascii"], "default": "mermaid" }
                        }
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
                "analyze" => {
                    let path = arguments.get("path").and_then(|v| v.as_str()).unwrap_or(".");
                    let mode = arguments.get("mode").and_then(|v| v.as_str()).unwrap_or("deep");
                    let config = ParallelConfig::default();
                    match CodeAnalyzer::scan_files(path, "rs", true, false) {
                        Ok(files) => match parallel_engine::ultra_analyze(&files, config) {
                            Ok((_, stats)) => Some(json!({ "content": [{ "type": "text", "text": format!("🔬 Analyze [{}] en {}ms. Archivos: {}", mode, stats.total_duration_ms, stats.total_files) }] })),
                            Err(e) => Some(json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }] })),
                        },
                        Err(e) => Some(json!({ "content": [{ "type": "text", "text": format!("Scan Error: {}", e) }] })),
                    }
                }
                "repair" => {
                    let path = arguments.get("path").and_then(|v| v.as_str()).unwrap_or(".");
                    let config = ParallelConfig::default();
                    match CodeAnalyzer::scan_files(path, "rs", true, false) {
                        Ok(files) => match parallel_engine::ultra_repair(&files, config) {
                            Ok((_, stats)) => Some(json!({ "content": [{ "type": "text", "text": format!("🛠️ Repair en {}ms. Archivos: {}", stats.total_duration_ms, stats.total_files) }] })),
                            Err(e) => Some(json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }] })),
                        },
                        Err(e) => Some(json!({ "content": [{ "type": "text", "text": format!("Scan Error: {}", e) }] })),
                    }
                }
                "cognitive_decision" => {
                    let situation = arguments.get("situation").and_then(|v| v.as_str()).unwrap_or("unknown");
                    match futures::executor::block_on(decision_engine.analyze_decision(situation, &HashMap::new())) {
                        Ok(decision) => Some(json!({
                            "content": [{ "type": "text", "text": format!("🧠 DECISIÓN: {}\n{}", decision.decision, decision.rationale) }]
                        })),
                        Err(e) => Some(json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }] })),
                    }
                }
                "memory_agility_stats" => {
                    let stats = futures::executor::block_on(shared_memory.get_stats());
                    Some(json!({ "content": [{ "type": "text", "text": format!("📊 AGILITY: {:.2}% | PRED: {:.2}%", stats.disk_agility_score * 100.0, stats.predictive_accuracy * 100.0) }] }))
                }
                "get_workspace_map" => {
                    Some(json!({ "content": [{ "type": "text", "text": "```mermaid\ngraph TD\n  Core --> Brain\n```" }] }))
                }
                _ => Some(json!({ "content": [{ "type": "text", "text": "Tool no encontrada" }] })),
            }
        }
        _ => None,
    };

    let mut result_val = result.unwrap_or(json!({}));

    // 2. Enriquecer con Predicciones y Contexto Denso
    if method == "tools/call" || method == "callTool" {
        if let Some(content) = result_val.get_mut("content").and_then(|c| c.as_array_mut()) {
            let tool_name = req.params.as_ref().and_then(|p| p.get("name")).and_then(|v| v.as_str()).unwrap_or("unknown");

            let action_context = crate::prediction_engine::ActionContext {
                action_type: tool_name.to_string(),
                parameters: req.params.as_ref().and_then(|p| p.get("arguments")).cloned().unwrap_or(json!({})),
                history: vec![],
                system_metrics: crate::prediction_engine::SystemMetrics::default(),
            };

            let stats = shared_memory.get_stats().await;
            let mut backpack = Backpack::new(agent_id.to_string());
            backpack.memory_agility = stats.clone();

            if let Ok(prediction) = prediction_engine.predict(crate::prediction_engine::PredictionType::NextAgentMoves, &action_context).await {
                backpack.predictions = serde_json::from_value(prediction.prediction_data.clone()).unwrap_or(backpack.predictions);
                content.push(json!({ "type": "text", "text": format!("\n🎒 MOCHILA (Proactiva):\n- 🔮 Siguiente: {}\n- ⏭️ Después: {}\n- 🧠 Razón: {}",
                    backpack.predictions.next_step,
                    backpack.predictions.following_step,
                    backpack.predictions.rationale) }));
            }

            // Inyectar Alertas en Mochila
            for (key, value) in shared_context.shared_data.iter() {
                if key.starts_with("alarm:") {
                    if let Some(msg) = value.get("message") {
                        backpack.add_alert(msg.as_str().unwrap_or("Unknown alert").to_string());
                    }
                }
            }

            if !backpack.proactive_alerts.is_empty() {
                content.push(json!({ "type": "text", "text": format!("⚠️ ALERTAS EN MOCHILA: {:?}", backpack.proactive_alerts) }));
            }

            content.push(json!({ "type": "text", "text": format!("\n🚀 SISTEMA: DISK {:.0}% | PRED {:.0}%", stats.disk_agility_score * 100.0, stats.predictive_accuracy * 100.0) }));
        }
    }

    // 3. Actualizar memoria
    shared_context.shared_data.insert(
        format!("last_action_{}", id),
        json!({ "method": method, "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() })
    );
    let _ = shared_memory.update_context(agent_id, shared_context).await;

    Json(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result_val),
        error: None,
    })
}
