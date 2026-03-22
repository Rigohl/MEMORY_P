# Cloudflare Deployment Guide - MEMORY_P v2.0

## Overview

MEMORY_P 19 microservice binaries deployed as **JSON-RPC 2.0 gateway** on Cloudflare Workers.

## Architecture

```
Cloudflare Workers (API Gateway)
├─ /mcp/{motor}/{endpoint} → routes to local microservice
├─ /health → aggregated motor status
└─ Scheduled health checks → KV storage monitoring
```

## Quick Start

### 1. Prerequisites
```bash
# Install Wrangler globally
npm install -g wrangler

# Verify
wrangler --version
```

### 2. Authenticate with Cloudflare
```bash
wrangler login
# Opens browser for OAuth authentication
```

### 3. Deploy
```bash
# Option A: Simple deploy
wrangler deploy

# Option B: With worker name
wrangler publish --worker-name memory-p-api

# Option C: Custom domain
wrangler deploy --route api.memory-p.workers.dev
```

### 4. Test
```bash
# Check health
curl https://memory-p-api.workers.dev/health

# Call motor
curl -X POST https://memory-p-api.workers.dev/mcp/qdrant/search \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "search",
    "params": {"query": [0.1, 0.2, 0.3]}
  }'
```

## Endpoints

### Health Check
```
GET /health
Response: { jsonrpc: "2.0", id: 1, result: { status, motors_available, timestamp } }
```

### Search Motors (JSON-RPC 2.0)
```
POST /mcp/{motor}/{endpoint}
motors: qdrant, faiss, scann, tantivy, lnx, meilisearch, memorybank, julia, chaos, etc.
```

### Example Requests

**Qdrant Vector Search**
```bash
curl -X POST https://api.memory-p.workers.dev/mcp/qdrant/search \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "search",
    "params": {"vector": [0.1, 0.2], "limit": 10}
  }'
```

**Tantivy Full-Text Search**
```bash
curl -X POST https://api.memory-p.workers.dev/mcp/tantivy/search \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "search",
    "params": {"query": "machine learning"}
  }'
```

**Julia Optimization**
```bash
curl -X POST https://api.memory-p.workers.dev/mcp/julia/optimize \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "optimize_weights",
    "params": {"weights": [0.33, 0.33, 0.34]}
  }'
```

## 19 Motors Available

| Motor | Port | Type | Status |
|-------|------|------|--------|
| qdrant | 3010 | Vector semantic | ✅ |
| faiss | 3011 | GPU billions | ✅ |
| scann | 3012 | Learned indexing | ✅ |
| tantivy | 3013 | Full-text BM25 | ✅ |
| lnx | 3014 | Distributed | ✅ |
| meilisearch | 3015 | Fuzzy | ✅ |
| memorybank | 3016 | Hybrid orchestrator | ✅ |
| mojo | 3017 | SIMD | ✅ |
| pony | 3018 | Actor system | ✅ |
| jax | 3019 | ML inference | ✅ |
| julia | 3020 | Math optimization | ✅ |
| chaos | 3021 | Chaos analysis | ✅ |
| memory_p | 3022 | Core | ✅ |
| mcp_server | 3023 | MCP dispatcher | ✅ |
| motor_orchestrator | 3024 | Router | ✅ |
| jar | 3025 | SQL/CLI | ✅ |
| vector_engine | 3026 | Tier 1 | ✅ |
| text_engine | 3027 | Tier 2 | ✅ |
| specialized_engine | 3028 | Tier 3 | ✅ |

## Configuration

### wrangler.toml
```toml
name = "memory-p-mcp-gateway"
main = "cloudflare-worker.ts"

[[kv_namespaces]]
binding = "BINARIES"
id = "your-kv-namespace-id"
```

### Environment Variables
```bash
# Add to wrangler.toml under [env.production]
UPSTREAM_URL = "http://localhost:3010"  # Local orchestrator
MOTOR_PORTS = "3010,3011,3012..."
```

## Advanced: Local Development

```bash
# Start in dev mode
wrangler dev

# Test locally
curl http://localhost:8787/health

# Watch file changes
wrangler dev --watch ./cloudflare-worker.ts
```

## Monitoring

### Health Status
```bash
# Check all motors
curl https://api.memory-p.workers.dev/health

# Check specific motor
curl https://api.memory-p.workers.dev/mcp/qdrant/health
```

### Logs
```bash
# Tail logs
wrangler tail

# Filter by motor
wrangler tail | grep "memory_p"
```

## Performance Tips

1. **Enable Caching**
   ```typescript
   response.headers.set('Cache-Control', 'max-age=300');
   ```

2. **Rate Limiting** (per plan)
   - Free: 100,000 requests/day
   - Pro: 10M+ requests/month

3. **Monitor KV Usage**
   - Health checks every 30s
   - ~7 KB per motor per day

## Troubleshooting

### 502 Bad Gateway
- Ensure local microservices are running on correct ports
- Check firewall: `localhost:3010-3028` accessible

### Timeout
- Increase timeout in wrangler.toml `[limits]`
- Default: 30s for Cloudflare Workers

### Authentication Failed
```bash
# Re-authenticate
wrangler logout
wrangler login
```

## Security

### CORS Configuration
```typescript
"Access-Control-Allow-Origin": "*"  // Change to specific domain in production
```

### API Keys
- Store sensitive keys in Cloudflare Secrets
- Use `env.API_TOKEN` in worker code

## Scaling

### Multi-Region Deployment
```bash
# Deploy to multiple regions (requires Enterprise)
wrangler deploy --route api-*.memory-p.workers.dev
```

### Load Balancing
- Cloudflare automatically distributes load
- 250+ data centers globally

## Rollback

```bash
# List deployments
wrangler deployments list

# Rollback to previous version
wrangler deployments rollback
```

---

**Deployed on**: Cloudflare Workers  
**Gateway URL**: https://memory-p-api.workers.dev  
**Status**: Production Ready ✅
