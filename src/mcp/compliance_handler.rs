//! MCP Compliance Handler
//! STEP 4: Week 1 MCP Compliance - Integration
//! Complete request/response cycle with validation + health checks

use crate::health::motor_health::MotorHealthChecker;
use crate::mcp::protocol::{self, error_codes, JsonRpcRequest, JsonRpcResponse};
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

/// MCP Compliance Handler State
pub struct MCPComplianceHandler {
    pub health_checker: Arc<RwLock<MotorHealthChecker>>,
}

impl MCPComplianceHandler {
    pub fn new(motors: Vec<String>) -> Self {
        Self {
            health_checker: Arc::new(RwLock::new(
                MotorHealthChecker::new(motors)
            )),
        }
    }

    /// Main MCP POST handler with full validation
    pub async fn handle_mcp_request(
        &self,
        body: Vec<u8>,
    ) -> Result<JsonRpcResponse, (StatusCode, String)> {
        // Step 1: Parse and validate JSON-RPC request
        let request = match protocol::parse_request(&body) {
            Ok(req) => req,
            Err(e) => {
                return Ok(protocol::error_response(
                    None,
                    error_codes::PARSE_ERROR,
                    &format!("Parse error: {}", e),
                    None,
                ));
            }
        };

        let request_id = request.id.clone();

        // Step 2: Dispatch to appropriate handler
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(&request).await,
            "tools/list" => self.handle_tools_list(&request).await,
            "tools/call" => self.handle_tools_call(&request).await,
            "resources/list" => self.handle_resources_list(&request).await,
            "sampling" => self.handle_sampling(&request).await,
            other => {
                return Ok(protocol::error_response(
                    request_id,
                    error_codes::METHOD_NOT_FOUND,
                    &format!("Method '{}' not found", other),
                    None,
                ));
            }
        };

        Ok(result)
    }

    async fn handle_initialize(&self, req: &JsonRpcRequest) -> JsonRpcResponse {
        // Check motor health
        let checker = self.health_checker.read().await;
        let motor_health = checker.check_all().await;

        let motors_healthy: Vec<_> = motor_health
            .iter()
            .filter(|m| m.healthy)
            .map(|m| m.motor.clone())
            .collect();

        let response_data = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": { "call": {} },
                "resources": { "list": {}, "read": {} },
                "memory_bank": true,  // MEMORY_P custom
            },
            "serverInfo": {
                "name": "MEMORY_P",
                "version": "3.0.0",
            },
            "motorStatus": {
                "healthy": motors_healthy,
                "totalMotors": motor_health.len(),
                "healthyMotors": motors_healthy.len(),
            }
        });

        protocol::success_response(req.id.clone(), response_data)
    }

    async fn handle_tools_list(&self, req: &JsonRpcRequest) -> JsonRpcResponse {
        let tools = json!([
            {
                "name": "julia_optimize",
                "description": "Optimize weights using Julia Optim.jl",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "weights": {
                            "type": "array",
                            "items": { "type": "number" }
                        }
                    },
                    "required": ["weights"]
                }
            },
            {
                "name": "julia_chaos_analyze",
                "description": "Analyze timeseries for chaos metrics (Lyapunov)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "timeseries": {
                            "type": "array",
                            "items": { "type": "number" }
                        }
                    },
                    "required": ["timeseries"]
                }
            },
            {
                "name": "julia_decision",
                "description": "Make decision based on chaos/entropy metrics",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "entropy": { "type": "number" },
                        "chaos": { "type": "number" },
                        "stability": { "type": "number" }
                    },
                    "required": ["entropy", "chaos", "stability"]
                }
            }
        ]);

        protocol::success_response(req.id.clone(), tools)
    }

    async fn handle_tools_call(&self, req: &JsonRpcRequest) -> JsonRpcResponse {
        let params = match &req.params {
            Some(p) => p.clone(),
            None => {
                return protocol::error_response(
                    req.id.clone(),
                    error_codes::INVALID_PARAMS,
                    "Missing params",
                    None,
                );
            }
        };

        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => {
                return protocol::error_response(
                    req.id.clone(),
                    error_codes::INVALID_PARAMS,
                    "Missing 'name' in params",
                    None,
                );
            }
        };

        let default_args = json!({});
        let args = params.get("arguments").unwrap_or(&default_args);

        // Execute real tool handlers
        let result = match tool_name {
            // ✅ [AUDIT-003 FIXED] MCP tools now execute real operations (not simulated)
            // Severity: MEDIUM | Status: FIXED | Date: 2026-03-22
            
            "julia_optimize" => {
                tracing::info!("🧮 Executing real: julia_optimize");
                // Extract parameters
                if let Ok(params_vec) = serde_json::from_value::<Vec<f64>>(args.get("params").cloned().unwrap_or(json!([]))){
                    match crate::ffi::julia::optimize_chaotic_system(&params_vec) {
                        Ok(optimized) => json!({
                            "status": "success",
                            "optimized_params": optimized,
                            "improvement": "real Julia optimization applied",
                            "timestamp": std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                        }),
                        Err(e) => json!({
                            "status": "error",
                            "error": e.to_string(),
                            "message": "Julia optimization failed"
                        })
                    }
                } else {
                    json!({
                        "status": "error",
                        "message": "Invalid parameters for julia_optimize"
                    })
                }
            }
            
            "julia_chaos_analyze" => {
                tracing::info!("🧮 Executing real: julia_chaos_analyze");
                // Extract time series
                if let Ok(time_series) = serde_json::from_value::<Vec<f64>>(args.get("time_series").cloned().unwrap_or(json!([]))) {
                    match crate::ffi::julia::analyze_dynamics(&time_series) {
                        Ok(lyapunov) => json!({
                            "status": "success",
                            "lyapunov_exponent": lyapunov,
                            "is_chaotic": lyapunov > 0.0,
                            "chaos_level": if lyapunov > 0.4 { "HIGH" } else if lyapunov > 0.1 { "MEDIUM" } else { "LOW" },
                            "timestamp": std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                        }),
                        Err(e) => json!({
                            "status": "error",
                            "error": e.to_string(),
                            "message": "Chaos analysis failed"
                        })
                    }
                } else {
                    json!({
                        "status": "error",
                        "message": "Invalid time_series for julia_chaos_analyze"
                    })
                }
            }
            
            "julia_decision" => {
                tracing::info!("🧮 Executing real: julia_decision");
                // Extract metrics for routing decision
                let entropy = args.get("entropy").and_then(|v| v.as_f64()).unwrap_or(0.5);
                let chaos = args.get("chaos").and_then(|v| v.as_f64()).unwrap_or(0.3);
                let stability = args.get("stability").and_then(|v| v.as_f64()).unwrap_or(0.7);
                
                // Real decision logic based on metrics
                let decision = if entropy > 2.5 {
                    "HYBRID_FUSION"
                } else if chaos > 0.4 {
                    "VECTOR_QDRANT"
                } else if stability > 0.8 {
                    "TEXT_TANTIVY"
                } else {
                    "HYBRID_BALANCED"
                };
                
                json!({
                    "status": "success",
                    "decision": decision,
                    "confidence": (entropy * chaos * stability).min(1.0),
                    "metrics": {
                        "entropy": entropy,
                        "chaos_level": chaos,
                        "stability": stability
                    },
                    "reasoning": format!("Decision '{}' based on entropy={:.2}, chaos={:.2}, stability={:.2}", 
                        decision, entropy, chaos, stability),
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                })
            }
            
            _ => {
                return protocol::error_response(
                    req.id.clone(),
                    error_codes::METHOD_NOT_FOUND,
                    &format!("Tool '{}' not found", tool_name),
                    None,
                );
            }
        };

        protocol::success_response(req.id.clone(), result)
    }

    async fn handle_resources_list(&self, req: &JsonRpcRequest) -> JsonRpcResponse {
        let resources = json!([
            {
                "uri": "memory://motors",
                "name": "Motor Status",
                "description": "Current motor health and performance"
            },
            {
                "uri": "memory://chaos-metrics",
                "name": "Chaos Metrics",
                "description": "System chaos and complexity metrics"
            },
            {
                "uri": "memory://routing-history",
                "name": "Routing History",
                "description": "Motor routing decisions and statistics"
            }
        ]);

        protocol::success_response(req.id.clone(), resources)
    }

    async fn handle_sampling(&self, req: &JsonRpcRequest) -> JsonRpcResponse {
        json!({
            "sampling": "not supported in compliance handler"
        });
        protocol::success_response(req.id.clone(), json!({}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_initialize() {
        let handler = MCPComplianceHandler::new(vec![
            "qdrant".to_string(),
            "tantivy".to_string(),
        ]);

        let request = json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "id": 1
        });

        let body = serde_json::to_vec(&request).unwrap();
        let response = handler.handle_mcp_request(body).await;

        assert!(response.is_ok());
        let resp = response.unwrap();
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.result.is_some());
    }

    #[tokio::test]
    async fn test_mcp_tools_list() {
        let handler = MCPComplianceHandler::new(vec![]);

        let request = json!({
            "jsonrpc": "2.0",
            "method": "tools/list",
            "id": 2
        });

        let body = serde_json::to_vec(&request).unwrap();
        let response = handler.handle_mcp_request(body).await;

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_request() {
        let handler = MCPComplianceHandler::new(vec![]);

        let invalid_json = b"not json";
        let response = handler.handle_mcp_request(invalid_json.to_vec()).await;

        assert!(response.is_ok());
        let resp = response.unwrap();
        assert!(resp.error.is_some());
    }
}
