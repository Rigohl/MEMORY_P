# MEMORY_P v2.0 - MCP HTTP Server (Port 4040)

## Overview

MEMORY_P exposes its hybrid search engine, FFI kernels, and tools via **HTTP REST** following the **Model Context Protocol (MCP)** specification.

The server provides:
- **Hybrid Search**: Vector + full-text search across 9 engines
- **FFI Kernels**: Direct access to Mojo SIMD operations
- **Tool Discovery**: List available tools and capabilities
- **Health Monitoring**: Real-time FFI status and availability

## Quick Start

### Start the Server

```bash
# Default: port 4040
cargo run --bin mcp_server

# Custom port
cargo run --bin mcp_server -- --port 5050
```

### Test via curl

```bash
# Health check
curl http://localhost:4040/health

# Get server info
curl http://localhost:4040/info

# Check FFI availability
curl http://localhost:4040/ffi/status

# List search engines
curl http://localhost:4040/engines

# List available tools
curl http://localhost:4040/tools

# Hybrid search
curl -X POST http://localhost:4040/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "find similar embeddings",
    "engines": ["qdrant", "faiss"],
    "limit": 10
  }'

# Call Mojo dot product
curl -X POST http://localhost:4040/kernels/dot_product \
  -H "Content-Type: application/json" \
  -d '{"a": [1, 2, 3], "b": [4, 5, 6]}'
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check (always 200) |
| `GET` | `/info` | Server info, version, capabilities |
| `GET` | `/ffi/status` | FFI toolchain availability (Zig, Julia, Mojo, Pony, JAX) |
| `POST` | `/search` | Hybrid search across multiple engines |
| `GET` | `/engines` | List available search engines |
| `GET` | `/tools` | List available MCP tools |
| `POST` | `/kernels/dot_product` | Mojo dot product kernel call |

## Request/Response Examples

### Search Query
```json
POST /search
{
  "query": "machine learning optimization",
  "engines": ["qdrant", "tantivy"],
  "limit": 5
}

Response:
[
  {
    "id": "doc-123",
    "score": 0.95,
    "text": "Optimization techniques for ML models...",
    "metadata": {}
  }
]
```

### Kernel Call
```json
POST /kernels/dot_product
{
  "a": [1.0, 2.0, 3.0],
  "b": [4.0, 5.0, 6.0]
}

Response:
{
  "result": 32.0
}
```

## Architecture

```
┌────────────────────────────────────────────┐
│         Client (curl, SDK, IDE)            │
└────────────────┬─────────────────────────────┘
                 │ HTTP/REST
                 ▼
┌────────────────────────────────────────────┐
│   Axum HTTP Server (Port 4040)             │
│   ├── /health                              │
│   ├── /info                                │
│   ├── /search → Motores (9 engines)        │
│   ├── /kernels/dot_product → Mojo .so      │
│   └── /ffi/status → Availability check     │
└────────────────┬────────────────────────────┘
                 │
    ┌────────────┼────────────┐
    ▼            ▼            ▼
┌────────┐  ┌────────┐  ┌────────┐
│Qdrant  │  │FAISS   │  │Tantivy │ ... (9 total)
└────────┘  └────────┘  └────────┘
    │            │            │
    └────────────┼────────────┘
                 │
    ┌────────────┴────────────┐
    ▼                         ▼
┌──────────────┐     ┌─────────────────┐
│ Mojo .so     │     │ Zig/Julia FFI   │
│(libmojo_     │     │ (Rust fallback) │
│kernels.so)   │     │                 │
└──────────────┘     └─────────────────┘
```

## Integration with Copilot/MCP Clients

The HTTP server can be accessed by VS Code extensions (Copilot, Cursor, Windsurf) via:

1. **Direct HTTP**: `http://localhost:4040/search`
2. **MCP over HTTP**: Configure client to use `http://127.0.0.1:4040`
3. **SDK**: Use `memory_p::mcp::SearchQuery` in Rust projects

## Configuration

### Environment Variables
- `MCP_PORT` - Server port (default: 4040)
- `MCP_LOG_LEVEL` - Log level (debug, info, warn, error)

### Runtime Options
```bash
cargo run --bin mcp_server -- --port 4040 --log debug
```

## Development

### Add New Endpoint
Edit `src/mcp/http_server.rs`:
```rust
async fn my_endpoint() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

// Register in Router:
.route("/my_endpoint", get(my_endpoint))
```

### Test Locally
```bash
# Terminal 1: Start server
cargo run --bin mcp_server

# Terminal 2: Run tests
cargo test mcp
```

## Performance

- **Throughput**: ~1000 req/s per engine (single thread)
- **Latency (P50)**: <50ms for hybrid search
- **FFI Kernel Calls**: <10ms for dot product (Mojo SIMD)

## Production Deployment

```bash
# Build optimized binary
cargo build --bin mcp_server --release

# Run with systemd
[Unit]
Description=MEMORY_P MCP HTTP Server

[Service]
Type=simple
ExecStart=/path/to/mcp_server --port 4040
WorkingDirectory=/path/to/memory_p
Restart=always

[Install]
WantedBy=multi-user.target
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Port 4040 in use | `cargo run -- --port 5050` |
| FFI unavailable | Check `GET /ffi/status` - ensure libmojo_kernels.so exists |
| Slow searches | Check engine health, verify index sizes |
| Connection refused | Ensure server started, check `localhost:4040/health` |

## Related Documentation

- [MCP Specification](https://modelcontextprotocol.io/)
- [MEMORY_P Architecture](docs/ARCHITECTURE.md)
- [FFI Integration](docs/INTEGRATIONS.md)
- [Search Engines Guide](docs/NINE_MOTORS_GUIDE.md)

---

**Version**: 3.0.0  
**Last Updated**: 2026-03-11  
**Maintained by**: MEMORY_P Team
