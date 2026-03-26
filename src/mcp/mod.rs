pub mod handlers;
pub mod http_server;
pub mod protocol;  // ← STEP 3: Strict JSON-RPC validation
pub mod tools;
pub mod orchestrator;
pub mod models;
pub mod shared_memory_tools;
pub mod julia_handler;
pub mod compliance_handler;  // ← STEP 4: Full compliance handler
pub mod motor_wrappers;  // ← MCP Motor Wrappers - native tools for each engine
pub mod autonomous;  // ← NEW v3.2: Autonomous self-managing MCP server
pub mod autonomous_tools;  // ← Autonomous tool implementations
pub mod monitoring;  // ← Background health monitoring
pub mod self_healing;  // ← Self-repair and auto-recovery
pub mod memory_engine;  // ← Predictive memory engine
pub mod memory_handlers;  // ← Memory HTTP handlers
pub mod memory_models;  // ← Memory data models
#[cfg(test)]
pub mod tests;

pub use http_server::{
    execute_tool, start_http_server, AppState, FFIStatus, MCPInfo, McpSession, SearchQuery,
    SearchResult,
};

/// Initialize MCP HTTP server on port 4040
pub async fn init_http_mcp(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing MCP HTTP Server v3.0 on port {}", port);
    http_server::start_http_server(port).await
}
