/// MCP Motor Wrappers - Native MCP tools for each search engine
///
/// Each motor (Qdrant, FAISS, SCANN, Tantivy, etc.) is exposed as an MCP tool
/// that can be called via JSON-RPC 2.0 protocol.
///
/// Protocol: HTTP POST to /mcp with JSON-RPC 2.0 request
/// Methods: motor_qdrant, motor_faiss, motor_scann, motor_tantivy, etc.

use crate::motores::core::types::{SearchResult, EngineCapabilities};
use serde::{Deserialize, Serialize};

/// Wrapper para cada motor como MCP tool compatible
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorWrapper {
    /// Nombre del motor (ej: "motor_qdrant")
    pub name: String,
    /// Descripción para MCP tools/list
    pub description: String,
    /// Capacidades del motor
    pub capabilities: EngineCapabilities,
    /// Latencia esperada (SLA en ms)
    pub sla_ms: u32,
}

/// Request payload para buscar en un motor específico
#[derive(Debug, Deserialize, Serialize)]
pub struct MotorSearchRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub filters: Option<String>,
    #[serde(default)]
    pub timeout_ms: Option<u32>,
}

fn default_limit() -> usize {
    10
}

/// Response wrapper para todas las búsquedas de motores
#[derive(Debug, Serialize)]
pub struct MotorSearchResponse {
    pub motor: String,
    pub results: Vec<SearchResult>,
    pub latency_ms: u64,
    pub query_id: String,
    pub timestamp: String,
}

/// Factory para crear MCP tool definitions
pub struct MotorToolFactory;

impl MotorToolFactory {
    /// Retorna la lista de todos los motores como MCP tools
    pub fn get_motor_tools() -> Vec<MotorWrapper> {
        vec![
            // Vector Search Engines
            MotorWrapper {
                name: "motor_qdrant".to_string(),
                description: "Vector semantic search via Qdrant (streaming, real-time)".to_string(),
                capabilities: EngineCapabilities {
                    supports_vector_search: true,
                    supports_full_text: false,
                    supports_distributed: false,
                    supports_replication: false,
                    supports_facets: false,
                    supports_fuzzy: false,
                    supports_real_time: true,
                    supports_typo_tolerance: false,
                    max_vector_dimension: Some(4096),
                    max_scale: Some(1_000_000_000),
                },
                sla_ms: 100,
            },
            MotorWrapper {
                name: "motor_faiss".to_string(),
                description: "GPU-ready vector search for billions-scale (CPU fallback available)".to_string(),
                capabilities: EngineCapabilities {
                    supports_vector_search: true,
                    supports_full_text: false,
                    supports_distributed: false,
                    supports_replication: false,
                    supports_facets: false,
                    supports_fuzzy: false,
                    supports_real_time: true,
                    supports_typo_tolerance: false,
                    max_vector_dimension: Some(2048),
                    max_scale: Some(10_000_000_000),
                },
                sla_ms: 50,
            },
            MotorWrapper {
                name: "motor_scann".to_string(),
                description: "Learned indexing for trillion-scale vectors (experimental)".to_string(),
                capabilities: EngineCapabilities {
                    supports_vector_search: true,
                    supports_full_text: false,
                    supports_distributed: false,
                    supports_replication: false,
                    supports_facets: false,
                    supports_fuzzy: false,
                    supports_real_time: false,
                    supports_typo_tolerance: false,
                    max_vector_dimension: None,
                    max_scale: Some(10_000_000_000),
                },
                sla_ms: 200,
            },
            // Text Search Engines
            MotorWrapper {
                name: "motor_tantivy".to_string(),
                description: "BM25 full-text search, single-node, sub-10ms latency".to_string(),
                capabilities: EngineCapabilities {
                    supports_vector_search: false,
                    supports_full_text: true,
                    supports_distributed: false,
                    supports_replication: false,
                    supports_facets: true,
                    supports_fuzzy: false,
                    supports_real_time: true,
                    supports_typo_tolerance: false,
                    max_vector_dimension: None,
                    max_scale: Some(100_000_000),
                },
                sla_ms: 10,
            },
            MotorWrapper {
                name: "motor_lnx".to_string(),
                description: "Distributed text search via LNX (3-node Raft cluster)".to_string(),
                capabilities: EngineCapabilities {
                    supports_vector_search: false,
                    supports_full_text: true,
                    supports_distributed: true,
                    supports_replication: true,
                    supports_facets: true,
                    supports_fuzzy: true,
                    supports_real_time: true,
                    supports_typo_tolerance: false,
                    max_vector_dimension: None,
                    max_scale: Some(10_000_000_000),
                },
                sla_ms: 150,
            },
            MotorWrapper {
                name: "motor_meilisearch".to_string(),
                description: "Typo-tolerant search, optimized for UX and auto-complete".to_string(),
                capabilities: EngineCapabilities {
                    supports_vector_search: false,
                    supports_full_text: true,
                    supports_distributed: false,
                    supports_replication: false,
                    supports_facets: true,
                    supports_fuzzy: false,
                    supports_real_time: true,
                    supports_typo_tolerance: true,
                    max_vector_dimension: None,
                    max_scale: Some(100_000_000),
                },
                sla_ms: 80,
            },
            // Specialized Engines
            MotorWrapper {
                name: "motor_julia_nlp".to_string(),
                description: "Mathematical NLP analysis via Julia (symbolic, symbolic derivatives)".to_string(),
                capabilities: EngineCapabilities {
                    supports_vector_search: false,
                    supports_full_text: true,
                    supports_distributed: false,
                    supports_replication: false,
                    supports_facets: false,
                    supports_fuzzy: true,
                    supports_real_time: true,
                    supports_typo_tolerance: true,
                    max_vector_dimension: None,
                    max_scale: Some(10_000_000),
                },
                sla_ms: 500,
            },
            MotorWrapper {
                name: "motor_memory_bank".to_string(),
                description: "Multi-language memory coordination via FFI (Zig, Julia, JAX, Mojo, Pony)".to_string(),
                capabilities: EngineCapabilities {
                    supports_vector_search: true,
                    supports_full_text: true,
                    supports_distributed: true,
                    supports_replication: false,
                    supports_facets: true,
                    supports_fuzzy: true,
                    supports_real_time: true,
                    supports_typo_tolerance: false,
                    max_vector_dimension: Some(4096),
                    max_scale: Some(100_000_000),
                },
                sla_ms: 200,
            },
            // Experimental
            MotorWrapper {
                name: "motor_toshi".to_string(),
                description: "Experimental distributed search (for testing, not production)".to_string(),
                capabilities: EngineCapabilities {
                    supports_vector_search: false,
                    supports_full_text: true,
                    supports_distributed: true,
                    supports_replication: true,
                    supports_facets: false,
                    supports_fuzzy: true,
                    supports_real_time: true,
                    supports_typo_tolerance: false,
                    max_vector_dimension: None,
                    max_scale: Some(1_000_000_000),
                },
                sla_ms: 300,
            },
        ]
    }

    /// Convierte lista de motors a formato MCP tools/list
    pub fn motors_to_mcp_tools(motors: Vec<MotorWrapper>) -> serde_json::Value {
        let tools: Vec<serde_json::Value> = motors
            .iter()
            .map(|motor| {
                serde_json::json!({
                    "name": motor.name,
                    "description": motor.description,
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search query string or vector embedding"
                            },
                            "limit": {
                                "type": "integer",
                                "default": 10,
                                "minimum": 1,
                                "maximum": 1000,
                                "description": "Maximum results to return"
                            },
                            "filters": {
                                "type": "string",
                                "description": "Optional filter expressions (motor-specific format)"
                            },
                            "timeout_ms": {
                                "type": "integer",
                                "default": motor.sla_ms,
                                "description": format!("Timeout in milliseconds (SLA: {}ms)", motor.sla_ms)
                            }
                        },
                        "required": ["query"]
                    }
                })
            })
            .collect();

        serde_json::json!(tools)
    }
}

/// Intelligent routing decision based on query type
pub enum RoutingDecision {
    SingleMotor(String),
    HybridMotors(Vec<String>),
    Sequential(Vec<String>),
}

/// RoutingAI: Selecciona motors basado en query y teoría del caos
pub struct RoutingAI {
    motors: Vec<MotorWrapper>,
}

impl RoutingAI {
    pub fn new() -> Self {
        Self {
            motors: MotorToolFactory::get_motor_tools(),
        }
    }

    /// Route query to optimal motor(s)
    pub fn route(&self, _query: &str, query_type: &str) -> RoutingDecision {
        match query_type {
            "semantic" => RoutingDecision::SingleMotor("motor_qdrant".to_string()),
            "text" => RoutingDecision::SingleMotor("motor_tantivy".to_string()),
            "massive_scale" => RoutingDecision::SingleMotor("motor_scann".to_string()),
            "distributed" => RoutingDecision::SingleMotor("motor_lnx".to_string()),
            "typo_tolerant" => RoutingDecision::SingleMotor("motor_meilisearch".to_string()),
            "math" => RoutingDecision::SingleMotor("motor_julia_nlp".to_string()),
            "hybrid" => RoutingDecision::HybridMotors(vec![
                "motor_qdrant".to_string(),
                "motor_tantivy".to_string(),
            ]),
            "experimental" => RoutingDecision::SingleMotor("motor_toshi".to_string()),
            _ => RoutingDecision::SingleMotor("motor_memory_bank".to_string()), // fallback
        }
    }

    /// Get all available motors
    pub fn available_motors(&self) -> &[MotorWrapper] {
        &self.motors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motor_tools_list() {
        let tools = MotorToolFactory::get_motor_tools();
        assert_eq!(tools.len(), 9);
        assert_eq!(tools[0].name, "motor_qdrant");
        assert_eq!(tools[8].name, "motor_toshi");
    }

    #[test]
    fn test_motor_tools_mcp_format() {
        let tools = MotorToolFactory::get_motor_tools();
        let mcp_tools = MotorToolFactory::motors_to_mcp_tools(tools);
        let arr = mcp_tools.as_array().unwrap();
        assert_eq!(arr.len(), 9);
        assert_eq!(arr[0]["name"].as_str().unwrap(), "motor_qdrant");
    }

    #[test]
    fn test_routing() {
        let router = RoutingAI::new();
        match router.route("find similar", "semantic") {
            RoutingDecision::SingleMotor(m) => assert_eq!(m, "motor_qdrant"),
            _ => panic!("Expected SingleMotor"),
        }

        match router.route("search text", "text") {
            RoutingDecision::SingleMotor(m) => assert_eq!(m, "motor_tantivy"),
            _ => panic!("Expected SingleMotor"),
        }

        match router.route("search all", "hybrid") {
            RoutingDecision::HybridMotors(motors) => {
                assert_eq!(motors.len(), 2);
                assert!(motors.contains(&"motor_qdrant".to_string()));
            }
            _ => panic!("Expected HybridMotors"),
        }
    }
}
