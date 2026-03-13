pub mod handlers;
pub mod http_server;
pub mod protocol;
pub mod tools;
pub mod orchestrator;
pub mod models;
pub mod shared_memory_tools;
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
