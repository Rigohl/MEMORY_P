// src/mcp/tests.rs - Tests for MCP tool execution

#[cfg(test)]
mod mcp_tools_tests {
    use crate::mcp::http_server::{execute_tool, AppState, FFIStatus};
    use serde_json::json;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_search_hybrid_tool() {
        // Create app state with mock data
        let state = Arc::new(AppState {
            info: crate::mcp::http_server::MCPInfo {
                name: "test".to_string(),
                version: "test".to_string(),
                capabilities: vec![],
                engines: vec![],
                tools: vec![],
            },
            ffi_status: crate::mcp::http_server::FFIStatus {
                zig: true,
                julia: false,
                mojo: false,
                pony: false,
                jax: false,
            },
            sessions: Default::default(),
            notification_tx: {
                let (tx, _) = tokio::sync::broadcast::channel(256);
                tx
            },
            workspace_root: ".".to_string(),
        });

        // Test search_hybrid with text and vector
        let args = json!({
            "text": "rust",
            "vector": [0.1, 0.2, 0.3],
            "limit": 5
        });

        let result = execute_tool(&state, "search_hybrid", args).await;
        assert!(result.is_ok(), "search_hybrid should succeed: {:?}", result);

        let value = result.unwrap();
        assert!(
            value.get("results").is_some(),
            "Response should have 'results' field"
        );
        assert!(
            value.get("total").is_some(),
            "Response should have 'total' field"
        );
    }

    #[tokio::test]
    async fn test_search_text_tool() {
        let state = Arc::new(AppState {
            info: crate::mcp::http_server::MCPInfo {
                name: "test".to_string(),
                version: "test".to_string(),
                capabilities: vec![],
                engines: vec![],
                tools: vec![],
            },
            ffi_status: crate::mcp::http_server::FFIStatus {
                zig: true,
                julia: false,
                mojo: false,
                pony: false,
                jax: false,
            },
            sessions: Default::default(),
            notification_tx: {
                let (tx, _) = tokio::sync::broadcast::channel(256);
                tx
            },
            workspace_root: ".".to_string(),
        });

        let args = json!({
            "text": "search",
            "limit": 3
        });

        let result = execute_tool(&state, "search_text", args).await;
        assert!(result.is_ok(), "search_text should succeed: {:?}", result);

        let value = result.unwrap();
        assert!(
            value.get("results").is_some(),
            "Response should have 'results' field"
        );
    }

    #[tokio::test]
    async fn test_search_vector_tool() {
        let state = Arc::new(AppState {
            info: crate::mcp::http_server::MCPInfo {
                name: "test".to_string(),
                version: "test".to_string(),
                capabilities: vec![],
                engines: vec![],
                tools: vec![],
            },
            ffi_status: crate::mcp::http_server::FFIStatus {
                zig: true,
                julia: false,
                mojo: false,
                pony: false,
                jax: false,
            },
            sessions: Default::default(),
            notification_tx: {
                let (tx, _) = tokio::sync::broadcast::channel(256);
                tx
            },
            workspace_root: ".".to_string(),
        });

        let args = json!({
            "vector": [0.5, 0.5, 0.5],
            "limit": 2
        });

        let result = execute_tool(&state, "search_vector", args).await;
        assert!(result.is_ok(), "search_vector should succeed: {:?}", result);

        let value = result.unwrap();
        assert!(
            value.get("results").is_some(),
            "Response should have 'results' field"
        );
    }
}
