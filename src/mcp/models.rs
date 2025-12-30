// [nuclear_god_mode] PROCESSED AT MAX SPEED - STABILITY FIX
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Serialize, Debug, Clone)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    pub result: Option<Value>,
    pub error: Option<Value>,
}

#[derive(Serialize, Debug, Clone)]
pub struct McpDescriptor {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Value>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProjectRequest {
    pub path: String,
    pub extension: Option<String>,
    pub max_tasks: Option<usize>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ProjectResponse {
    pub status: String,
    pub results: Vec<Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UltraRequest {
    pub target_dir: String,
    pub file_extension: Option<String>,
    pub max_tasks: Option<usize>,
    pub dry_run: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UltraEditRequest {
    pub changes: Vec<FileChange>,
    pub dry_run: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub operations: Vec<EditOp>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum EditOp {
    Replace {
        target: String,
        replacement: String,
    },
    RegexReplace {
        pattern: String,
        replacement: String,
    },
    Append {
        content: String,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UltraWorkflowRequest {
    pub steps: Vec<WorkflowStep>,
    pub max_tasks: Option<usize>,
    pub dry_run: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "action", content = "params")]
pub enum WorkflowStep {
    Scan {
        path: String,
        extension: Option<String>,
    },
    Filter {
        pattern: String,
        invert: Option<bool>,
    },
    Analyze,
    Edit {
        operations: Vec<EditOp>,
    },
    Repair,
    /// Auto-evolve: Analyze → Detect issues → Generate fixes → Apply → Repeat
    Evolve {
        /// Maximum iterations (default: 5)
        max_iterations: Option<u32>,
        /// Only report what would change
        dry_run: Option<bool>,
    },
}

#[derive(Serialize, Debug, Clone)]
pub struct UltraResponse {
    pub status: String,
    pub engine_output: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateProjectRequest {
    pub path: String,
    pub name: String,
    pub template: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct CreateProjectResponse {
    pub status: String,
    pub created_files: Vec<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UltraSimulationRequest {
    pub name: String,
    pub logic: String,
    pub parameters: serde_json::Value,
    pub use_gpu: Option<bool>,
}
