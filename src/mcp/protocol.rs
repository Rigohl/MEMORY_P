//! MCP Protocol Implementation
//! Strict JSON-RPC 2.0 per spec 2024-11-05 with input validation
//! STEP 3: Week 1 MCP Compliance - Input Validation

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::error::MemoryPError;

/// JSON-RPC 2.0 Request per spec (strict validation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,  // MUST be "2.0"
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,  // Can be null, string, or number
}

/// JSON-RPC 2.0 Response per spec (strict validation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,  // MUST be "2.0"
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 Error per spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

/// Parse and validate incoming MCP request
pub fn parse_request(data: &[u8]) -> crate::error::Result<JsonRpcRequest> {
    // Step 1: Parse JSON
    let json: Value = serde_json::from_slice(data)
        .map_err(|e| MemoryPError::Other(format!("Invalid JSON: {}", e)))?;

    // Step 2: Validate jsonrpc field
    let jsonrpc = json
        .get("jsonrpc")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MemoryPError::Other("Missing or invalid 'jsonrpc' field".to_string()))?;

    if jsonrpc != "2.0" {
        return Err(MemoryPError::Other(format!("Invalid jsonrpc version: {}, must be '2.0'", jsonrpc)));
    }

    // Step 3: Validate method field
    let method = json
        .get("method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MemoryPError::Other("Missing or invalid 'method' field".to_string()))?;

    if method.is_empty() {
        return Err(MemoryPError::Other("method cannot be empty".to_string()));
    }

    // Step 4: Validate allowed methods
    let allowed_methods = vec![
        "initialize",
        "tools/list",
        "tools/call",
        "resources/list",
        "resources/read",
        "sampling",
        "roots/list",
        "roots/add",
    ];

    if !allowed_methods.contains(&method) {
        return Err(MemoryPError::Other(format!("Unknown method: {}. Allowed: {:?}", method, allowed_methods)));
    }

    // Step 5: Extract id (can be null, string, or number - but MUST be present for requests)
    let id = json.get("id").cloned();

    // Step 6: Extract params (optional, but if present must be object or array)
    let params = json.get("params").cloned();
    if let Some(ref p) = params {
        if !p.is_object() && !p.is_array() && !p.is_null() {
            return Err(MemoryPError::Other("params must be an object, array, or null".to_string()));
        }
    }

    Ok(JsonRpcRequest {
        jsonrpc: jsonrpc.to_string(),
        method: method.to_string(),
        params,
        id,
    })
}

/// Create MCP-compliant response
pub fn create_response(
    id: Option<Value>,
    result: Option<Value>,
    error: Option<JsonRpcError>,
) -> crate::error::Result<JsonRpcResponse> {
    // Validate: exactly one of result or error
    match (result.is_some(), error.is_some()) {
        (true, true) => {
            return Err("Response cannot have both result and error".into());
        }
        (false, false) => {
            return Err("Response must have either result or error".into());
        }
        _ => {}
    }

    Ok(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result,
        error,
        id,
    })
}

/// Success response helper
pub fn success_response(id: Option<Value>, data: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(data),
        error: None,
        id,
    }
}

/// Error response helper
pub fn error_response(
    id: Option<Value>,
    code: i32,
    message: &str,
    data: Option<Value>,
) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(JsonRpcError {
            code,
            message: message.to_string(),
            data,
        }),
        id,
    }
}

// Standard JSON-RPC error codes (per spec)
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    pub const SERVER_ERROR_START: i32 = -32099;
    pub const SERVER_ERROR_END: i32 = -32000;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_request() {
        let json = r#"{"jsonrpc": "2.0", "method": "tools/list", "id": 1}"#;
        let req = parse_request(json.as_bytes()).expect("Should parse");
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "tools/list");
        assert_eq!(req.id, Some(json!(1)));
    }

    #[test]
    fn test_invalid_jsonrpc_version() {
        let json = r#"{"jsonrpc": "1.0", "method": "tools/list", "id": 1}"#;
        let result = parse_request(json.as_bytes());
        assert!(result.is_err(), "Should reject jsonrpc != 2.0");
    }

    #[test]
    fn test_missing_method() {
        let json = r#"{"jsonrpc": "2.0", "id": 1}"#;
        let result = parse_request(json.as_bytes());
        assert!(result.is_err(), "Should require method field");
    }

    #[test]
    fn test_invalid_method() {
        let json = r#"{"jsonrpc": "2.0", "method": "invalid/method", "id": 1}"#;
        let result = parse_request(json.as_bytes());
        assert!(result.is_err(), "Should reject unknown methods");
    }

    #[test]
    fn test_success_response() {
        let resp = success_response(Some(json!(1)), json!({"ok": true}));
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_error_response() {
        let resp = error_response(
            Some(json!(1)),
            error_codes::METHOD_NOT_FOUND,
            "Method not found",
            None,
        );
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
    }
}
