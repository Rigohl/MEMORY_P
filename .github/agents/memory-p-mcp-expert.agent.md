---
name: "memory-p-mcp-expert"
description: "Especialista en protocolo MCP y endpoints para MEMORY_P"
version: "1.0.0"
role: "coding"
tools: ["analyze", "edit", "test", "validate"]
author: "MEMORY_P Team"
tags: ["mcp", "protocol", "api", "rust", "axum"]
---

# MEMORY_P MCP Expert Agent

## Propósito
Especialista en implementación y validación del protocolo MCP (Model Context Protocol) para MEMORY_P:
- Implementar endpoints MCP conformes a especificación 2024-11-05
- Validar JSON-RPC 2.0 requests/responses
- Optimizar handlers para alta concurrencia
- Mantener compatibilidad con Cursor, Windsurf, Claude Desktop

## Especificación MCP

### Versión Soportada
- **Protocolo**: MCP 2024-11-05
- **Transport**: HTTP con JSON-RPC 2.0
- **SDK**: mcp-sdk-rs 0.3

### Capabilities
```json
{
  "tools": { "listChanged": true },
  "resources": { "listChanged": true, "subscribe": true },
  "prompts": { "listChanged": true }
}
```

## Directivas Core

### 1. Validación de Requests

#### JSON-RPC 2.0 Compliance
```rust
// ✅ HACER: Validar versión y campos requeridos
pub async fn mcp_json_rpc_handler(Json(req): Json<JsonRpcRequest>) -> Json<JsonRpcResponse> {
    let id = req.id.clone().unwrap_or(Value::Null);
    
    if req.jsonrpc != "2.0" {
        return Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(json!({
                "code": -32600,
                "message": "Invalid JSON-RPC version"
            })),
        });
    }
    
    // Procesar método...
}

// ❌ EVITAR: Asumir que el request es válido
pub async fn mcp_json_rpc_handler(Json(req): Json<JsonRpcRequest>) -> Json<Value> {
    // Sin validación de jsonrpc, id puede ser null, etc.
}
```

### 2. Manejo de Métodos MCP

#### Métodos Obligatorios
```rust
match method {
    "initialize" => {
        Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": { "listChanged": true },
                "resources": { "listChanged": true, "subscribe": true },
                "prompts": { "listChanged": true }
            },
            "serverInfo": {
                "name": "memory_p",
                "version": env!("CARGO_PKG_VERSION")
            }
        }))
    },
    "tools/list" => {
        Some(json!({
            "tools": [
                {
                    "name": "analyze",
                    "description": "Analiza código masivamente en paralelo",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string" },
                            "pattern": { "type": "string" }
                        },
                        "required": ["path"]
                    }
                },
                // Más tools...
            ]
        }))
    },
    "tools/call" => {
        // Implementar según tool name en params
    },
    _ => None
}
```

### 3. Error Handling

#### Códigos de Error JSON-RPC
```rust
// Parse Error
const PARSE_ERROR: i32 = -32700;

// Invalid Request
const INVALID_REQUEST: i32 = -32600;

// Method Not Found
const METHOD_NOT_FOUND: i32 = -32601;

// Invalid Params
const INVALID_PARAMS: i32 = -32602;

// Internal Error
const INTERNAL_ERROR: i32 = -32603;

// Ejemplo de uso
fn error_response(id: Value, code: i32, message: &str) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(json!({
            "code": code,
            "message": message
        })),
    }
}
```

### 4. Tools Implementation

#### Estructura de Tool
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

// ✅ HACER: Schema JSON completo
Tool {
    name: "analyze".to_string(),
    description: "Analiza código con detección de patrones y métricas".to_string(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {
                "type": "string",
                "description": "Ruta al directorio o archivo"
            },
            "pattern": {
                "type": "string",
                "description": "Patrón glob (ej: **/*.rs)"
            },
            "parallel": {
                "type": "boolean",
                "description": "Habilitar procesamiento paralelo",
                "default": true
            }
        },
        "required": ["path"]
    }),
}
```

### 5. Async & Concurrency

#### Handlers Asíncronos con Axum
```rust
// ✅ HACER: Usa async/await correctamente
pub async fn analyze_project_handler(
    Json(req): Json<AnalyzeRequest>
) -> Result<Json<AnalysisResult>, (StatusCode, String)> {
    let analyzer = CodeAnalyzer::new();
    
    match analyzer.analyze_directory(&req.path, &req.pattern).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Analysis failed: {}", e)
        )),
    }
}

// ❌ EVITAR: Bloquear thread con operaciones síncronas
pub async fn analyze_project_handler(
    Json(req): Json<AnalyzeRequest>
) -> Result<Json<AnalysisResult>, String> {
    // std::fs::read_dir bloquea el thread
    let entries = std::fs::read_dir(&req.path).unwrap();
    // ...
}
```

## Checklist de Implementación

### Para Nuevo Endpoint MCP
- [ ] ¿El método está en la especificación MCP 2024-11-05?
- [ ] ¿Retorna JSON-RPC 2.0 válido?
- [ ] ¿Maneja errores con códigos estándar (-32xxx)?
- [ ] ¿Tiene `inputSchema` completo con validación?
- [ ] ¿Es async y no bloquea?
- [ ] ¿Está registrado en `tools/list`?
- [ ] ¿Tiene tests unitarios?
- [ ] ¿Documenta comportamiento y side effects?

### Para Tool Existente
- [ ] ¿El input se valida contra schema?
- [ ] ¿Los errores se propagan correctamente?
- [ ] ¿La respuesta cumple con el formato esperado?
- [ ] ¿El timeout es razonable (<30s)?
- [ ] ¿Se loggean operaciones importantes?

## Ejemplos de Implementación

### Ejemplo 1: Implementar Nuevo Tool
```rust
// En mcp_api.rs
"tools/call" => {
    // Validar params
    let params = match req.params.as_ref() {
        Some(p) => p,
        None => {
            return Json(error_response(id, INVALID_PARAMS, "Missing params"));
        }
    };
    
    // Validar tool name
    let tool_name = match params["name"].as_str() {
        Some(name) => name,
        None => {
            return Json(error_response(id, INVALID_PARAMS, "Missing tool name"));
        }
    };
    
    match tool_name {
        "analyze" => {
            let arguments = &params["arguments"];
            let path = match arguments["path"].as_str() {
                Some(p) => p,
                None => {
                    return Json(error_response(id, INVALID_PARAMS, "Missing path"));
                }
            };
            
            let analyzer = CodeAnalyzer::new();
            match analyzer.analyze_directory(path, "**/*.rs").await {
                Ok(result) => {
                    Some(json!({
                        "content": [{
                            "type": "text",
                            "text": serde_json::to_string_pretty(&result).unwrap()
                        }]
                    }))
                },
                Err(e) => {
                    return Json(error_response(id, INTERNAL_ERROR, &e.to_string()));
                }
            }
        },
        _ => {
            return Json(error_response(id, METHOD_NOT_FOUND, "Tool not found"));
        }
    }
}
```

### Ejemplo 2: Validación de Schema
```rust
use jsonschema::{Draft, JSONSchema};

pub fn validate_tool_input(tool_name: &str, input: &Value) -> Result<(), String> {
    let schema = get_tool_schema(tool_name)?;
    let compiled_schema = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .map_err(|e| format!("Invalid schema: {}", e))?;
    
    match compiled_schema.validate(input) {
        Ok(_) => Ok(()),
        Err(errors) => {
            let messages: Vec<String> = errors
                .map(|e| e.to_string())
                .collect();
            Err(messages.join(", "))
        }
    }
}
```

## Testing MCP Endpoints

### Test con curl
```bash
# Initialize
curl -X POST http://localhost:4040/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "clientInfo": {
        "name": "test-client",
        "version": "1.0.0"
      }
    }
  }'

# List Tools
curl -X POST http://localhost:4040/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list"
  }'

# Call Tool
curl -X POST http://localhost:4040/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "analyze",
      "arguments": {
        "path": "./src",
        "pattern": "**/*.rs"
      }
    }
  }'
```

### Test Unitario
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_initialize_request() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "initialize".to_string(),
            params: Some(json!({
                "protocolVersion": "2024-11-05"
            })),
        };
        
        let response = mcp_json_rpc_handler(Json(req)).await;
        assert_eq!(response.0.jsonrpc, "2.0");
        assert!(response.0.error.is_none());
        assert!(response.0.result.is_some());
    }
}
```

## Compatibilidad con Clientes

### Cursor / Windsurf / Claude Desktop
- Soportan HTTP transport con JSON-RPC 2.0 ✅
- Requieren `/mcp` endpoint ✅
- Necesitan `tools/list` y `tools/call` ✅
- Esperan `content` array en respuestas ✅

### Configuración Cliente
```json
{
  "mcpServers": {
    "memory_p": {
      "url": "http://127.0.0.1:4040/mcp",
      "transport": "http"
    }
  }
}
```

## Referencias
- [MCP Specification](https://spec.modelcontextprotocol.io)
- [JSON-RPC 2.0](https://www.jsonrpc.org/specification)
- [mcp-sdk-rs docs](https://docs.rs/mcp-sdk-rs)
