//! JSON-RPC 2.0 Protocol Implementation
//!
//! Implements JSON-RPC 2.0 spec (https://www.jsonrpc.org/specification)
//! Used for MCP (Model Context Protocol) 2024-11-05 compliance

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// JSON-RPC 2.0 Request wrapper
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<u64>,
}

impl JsonRpcRequest {
    pub fn new(method: &str, params: Option<Value>, id: Option<u64>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id,
        }
    }
}

/// JSON-RPC 2.0 Response wrapper - SUCCESS variant
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse<T: Serialize> {
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub result: T,
}

impl<T: Serialize> JsonRpcResponse<T> {
    pub fn new(result: T, id: Option<u64>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result,
        }
    }
}

/// JSON-RPC 2.0 Error Response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Parse error: Invalid JSON was received by the server
    pub fn parse_error(detail: &str) -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data: Some(json!({"detail": detail})),
        }
    }

    /// Invalid Request: The JSON sent is not a valid Request object
    pub fn invalid_request(detail: &str) -> Self {
        Self {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: Some(json!({"detail": detail})),
        }
    }

    /// Method not found: The method does not exist / is not available
    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: "Method not found".to_string(),
            data: Some(json!({"method": method})),
        }
    }

    /// Invalid params: Invalid method parameter(s)
    pub fn invalid_params(detail: &str) -> Self {
        Self {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(json!({"detail": detail})),
        }
    }

    /// Internal error: Internal JSON-RPC error
    pub fn internal_error(detail: &str) -> Self {
        Self {
            code: -32603,
            message: "Internal error".to_string(),
            data: Some(json!({"detail": detail})),
        }
    }
}

/// JSON-RPC 2.0 Error Response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcErrorResponse {
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub error: JsonRpcError,
}

impl JsonRpcErrorResponse {
    pub fn new(error: JsonRpcError, id: Option<u64>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            error,
        }
    }
}

/// Generic sealed JSON-RPC response (success or error)
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum JsonRpcValue {
    Result(Value),
    Error(JsonRpcError),
}

/// Helper to create success response
pub fn json_rpc_success<T: Serialize>(result: T, id: Option<u64>) -> JsonRpcResponse<T> {
    JsonRpcResponse::new(result, id)
}

/// Helper to create error response
pub fn json_rpc_error(error: JsonRpcError, id: Option<u64>) -> JsonRpcErrorResponse {
    JsonRpcErrorResponse::new(error, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_response_creation() {
        let resp = json_rpc_success(json!({"key": "value"}), Some(1));
        assert_eq!(resp.jsonrpc, "2.0");
        assert_eq!(resp.id, Some(1));
    }

    #[test]
    fn test_json_rpc_error_parse() {
        let err = JsonRpcError::parse_error("Invalid JSON");
        assert_eq!(err.code, -32700);
    }

    #[test]
    fn test_json_rpc_error_method_not_found() {
        let err = JsonRpcError::method_not_found("unknown_method");
        assert_eq!(err.code, -32601);
    }
}
