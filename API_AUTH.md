# 🔐 MEMORY_P API Authentication

## Overview

The MEMORY_P Cloudflare gateway now requires authentication on all endpoints except `/health`.

**Health check is public** (no auth needed):
```bash
curl https://memory-p-api.workers.dev/health
```

**All other endpoints require API key**:
```bash
curl -X POST https://memory-p-api.workers.dev/mcp/qdrant/search \
  -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"search","params":{}}'
```

---

## Usage

### Method 1: X-API-Key Header (Recommended)
```bash
curl -X POST https://memory-p-api.workers.dev/mcp/qdrant/search \
  -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"search","params":{"vector":[0.1,0.2]}}'
```

### Method 2: Bearer Token (Authorization Header)
```bash
curl -X POST https://memory-p-api.workers.dev/mcp/qdrant/search \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"search","params":{"vector":[0.1,0.2]}}'
```

---

## Configuration

### Dev Mode (Local Testing)
```bash
# Run with dev API key (default)
npx wrangler dev --env development

# Access with
export API_KEY="dev-key-12345"
curl -H "X-API-Key: $API_KEY" http://localhost:8787/mcp/qdrant/search
```

### Production Deployment
```bash
# Deploy with production key
npx wrangler deploy --env production

# Set secure API key BEFORE deploying:
# In wrangler.toml or via Cloudflare UI:
# [env.production]
# vars = { MEMORY_P_API_KEY = "your-secure-key-here" }
```

---

## Errors

### 401 Unauthorized (Missing Key)
```json
{
  "jsonrpc": "2.0",
  "id": null,
  "error": {
    "code": -32000,
    "message": "Unauthorized",
    "data": "Missing or invalid API key. Use -H 'X-API-Key: your-key' or -H 'Authorization: Bearer your-key'"
  }
}
```

### 401 Unauthorized (Invalid Key)
Same response with invalid API key provided.

---

## Security Best Practices

1. **Change default API key** before deploying to production
2. **Use strong, random keys** (32+ characters)
3. **Rotate keys regularly** 
4. **Store keys securely** (environment variables, secrets manager)
5. **Use HTTPS only** (Cloudflare automatically enforces this)
6. **Rate limiting** (can be added in future updates)

---

## Example: Python Client

```python
import requests
import json

API_KEY = "your-api-key"
API_URL = "https://memory-p-api.workers.dev"

headers = {
    "X-API-Key": API_KEY,
    "Content-Type": "application/json"
}

query = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "search",
    "params": {"vector": [0.1, 0.2, 0.3]}
}

response = requests.post(
    f"{API_URL}/mcp/qdrant/search",
    headers=headers,
    json=query
)

print(response.json())
```

---

## Example: TypeScript Client

```typescript
const API_KEY = "your-api-key";
const API_URL = "https://memory-p-api.workers.dev";

const headers = {
  "X-API-Key": API_KEY,
  "Content-Type": "application/json",
};

const query = {
  jsonrpc: "2.0",
  id: 1,
  method: "search",
  params: { vector: [0.1, 0.2, 0.3] },
};

const response = await fetch(`${API_URL}/mcp/qdrant/search`, {
  method: "POST",
  headers,
  body: JSON.stringify(query),
});

const result = await response.json();
console.log(result);
```

---

## Endpoints Always Available (No Auth)

- `GET /health` - Health check

---

## All Protected Endpoints

| Endpoint | Method | Auth Required | Description |
|----------|--------|---------------|-------------|
| `/mcp/qdrant/search` | POST | ✅ | Vector search (Qdrant) |
| `/mcp/faiss/search` | POST | ✅ | GPU vector search (FAISS) |
| `/mcp/scann/search` | POST | ✅ | Trillion-scale search (SCANN) |
| `/mcp/tantivy/search` | POST | ✅ | Full-text search (Tantivy) |
| `/mcp/lnx/search` | POST | ✅ | Distributed search (LNX) |
| `/mcp/meilisearch/search` | POST | ✅ | Typo-tolerant search (MeiliSearch) |
| `/mcp/julia/optimize` | POST | ✅ | Math optimization (Julia) |
| `/mcp/chaos/analyze` | POST | ✅ | Chaos analysis (ChaosAnalyzer) |
| ... (16 total motors) | POST | ✅ | All search endpoints |

---

## Questions?

See: [CLOUDFLARE_DEPLOYMENT.md](CLOUDFLARE_DEPLOYMENT.md)
