# MCP en MEMORY_P - Guía de Uso en Cloudflare

**Status**: ✅ **LIVE EN PRODUCTION** (Cloudflare Workers)  
**Deployment**: 2026-03-22  
**MCP Version**: 2024-11-05 compatible  
**Protocol**: JSON-RPC 2.0

---

## 🎯 ¿Qué es MCP?

**MCP (Model Context Protocol)** es un protocolo estandarizado que permite:
- Conectar GitHub Copilot, Claude, y otros clientes
- Exponerse como un **servidor MCP**
- Proveer **tools** y **resources** a través de JSON-RPC 2.0

---

## 📍 Endpoints MCP en MEMORY_P

### URL Base (en Cloudflare):
```
https://memory-p-api.workers.dev/mcp/{motor}/{endpoint}
```

### Ejemplo: Acceder a Qdrant motores
```
POST https://memory-p-api.workers.dev/mcp/qdrant/search
POST https://memory-p-api.workers.dev/mcp/qdrant/health
POST https://memory-p-api.workers.dev/mcp/qdrant/index
```

---

## 🔐 Autenticación

Todos los endpoints MCP requieren autenticación:

### Opción 1: API Key (más rápido)
```bash
curl -X POST https://memory-p-api.workers.dev/mcp/qdrant/search \
  -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "search",
    "params": {
      "query": "machine learning",
      "limit": 10
    }
  }'
```

### Opción 2: OAuth 2.0 (PKCE)
```bash
# 1. Get authorization code
curl -X POST https://memory-p-api.workers.dev/oauth/authorize \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "memory_p_oauth_xxxxx",
    "response_type": "code",
    "redirect_uri": "https://your-app.com/callback",
    "scope": "mcp:full",
    "code_challenge": "E9Mrozoa2owUednLe...",
    "code_challenge_method": "S256"
  }'

# 2. Exchange code for JWT token
curl -X POST https://memory-p-api.workers.dev/oauth/token \
  -H "Content-Type: application/json" \
  -d '{
    "grant_type": "authorization_code",
    "code": "auth_code_from_step1",
    "client_id": "memory_p_oauth_xxxxx",
    "client_secret": "your_oauth_secret",
    "code_verifier": "original_code_verifier"
  }'

# 3. Use JWT token in MCP calls
curl -X POST https://memory-p-api.workers.dev/mcp/qdrant/search \
  -H "Authorization: Bearer {jwt_token_from_step2}" \
  -H "Content-Type: application/json" \
  -d '{...mcp_request...}'
```

---

## 🧠 19 Motores Disponibles en MCP

| Motor | Tipo | Endpoint | Caso de Uso |
|-------|------|----------|-----------|
| **qdrant** | Vector | /mcp/qdrant/ | Búsqueda semántica |
| **faiss** | GPU Vector | /mcp/faiss/ | Búsqueda a escala GPU |
| **scann** | Learned Index | /mcp/scann/ | Índices aprendidos |
| **tantivy** | Full-text | /mcp/tantivy/ | BM25 / Búsqueda exacta |
| **lnx** | Distributed | /mcp/lnx/ | Búsqueda distribuida |
| **meilisearch** | Fuzzy | /mcp/meilisearch/ | Búsqueda tolerante a errores |
| **memorybank** | Multilenguaje | /mcp/memorybank/ | Coordinación multi-idioma |
| **mojo** | SIMD | /mcp/mojo/ | Kernels acelerados |
| **pony** | Actor | /mcp/pony/ | Sistema de actores |
| **jax** | ML | /mcp/jax/ | Inferencia ML |
| **julia** | Math | /mcp/julia/ | Optimización matemática |
| **chaos** | Analysis | /mcp/chaos/ | Análisis de teoría del caos |
| **memory_p** | Core | /mcp/memory_p/ | Orquestación central |
| **mcp_server** | MCP | /mcp/mcp_server/ | Servidor MCP puro |
| **motor_orchestrator** | Orchestration | /mcp/motor_orchestrator/ | Orquestación de motores |
| **jar** | CLI | /mcp/jar/ | Herramienta JAR |
| **vector** | Vector Group | /mcp/vector/ | Grupo de búsqueda vectorial |
| **text** | Text Group | /mcp/text/ | Grupo de búsqueda textual |
| **specialized** | Specialized | /mcp/specialized/ | Motores especializados |

---

## 📝 Formato de Request MCP (JSON-RPC 2.0)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "search",
  "params": {
    "query": "tu consulta aquí",
    "limit": 10,
    "filters": { ... }
  }
}
```

### Estructura Común:
- `jsonrpc`: "2.0" (requerido)
- `id`: número único (requerido para tracking)
- `method`: nombre del método (search, health, index, etc.)
- `params`: parámetros específicos del motor

---

## ✅ Respuesta MCP

**Success (200):**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "hits": [
      {
        "id": "doc_1",
        "score": 0.95,
        "content": "Machine learning models...",
        "metadata": { ... }
      }
    ],
    "total": 1000,
    "elapsed_ms": 45
  }
}
```

**Error (400/401/500):**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32601,
    "message": "Method not found",
    "data": {
      "details": "Available methods: search, health, index"
    }
  }
}
```

---

## 🧪 Ejemplos de Uso

### 1️⃣ Health Check (sin autenticación requerida para /health)
```bash
curl https://memory-p-api.workers.dev/health
```

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "status": "healthy",
    "motors_available": 19,
    "timestamp": "2026-03-22T03:22:00Z"
  }
}
```

### 2️⃣ Búsqueda Semántica en Qdrant
```bash
curl -X POST https://memory-p-api.workers.dev/mcp/qdrant/search \
  -H "X-API-Key: dev-key-12345" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "search",
    "params": {
      "query": "semantic search with vectors",
      "limit": 5,
      "threshold": 0.7
    }
  }'
```

### 3️⃣ Búsqueda Full-Text en Tantivy
```bash
curl -X POST https://memory-p-api.workers.dev/mcp/tantivy/search \
  -H "X-API-Key: dev-key-12345" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "search",
    "params": {
      "query": "cloud infrastructure",
      "fuzziness": 1
    }
  }'
```

### 4️⃣ Indexar Documento
```bash
curl -X POST https://memory-p-api.workers.dev/mcp/qdrant/index \
  -H "X-API-Key: dev-key-12345" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "index",
    "params": {
      "documents": [
        {
          "id": "doc_1",
          "content": "About machine learning",
          "embedding": [0.1, 0.2, 0.3, ...]
        }
      ]
    }
  }'
```

---

## 💡 Integración con GitHub Copilot

### Conectar como MCP Server:

1. **Crear config en VS Code:**
   ```json
   // .vscode/settings.json
   {
     "codeium.copilot.serverUrl": "https://memory-p-api.workers.dev",
     "codeium.copilot.mcp.servers": [
       {
         "url": "https://memory-p-api.workers.dev/mcp",
         "auth": "bearer {JWT_TOKEN}"
       }
     ]
   }
   ```

2. **Usar en VS Code:**
   - Abre Copilot Chat (@symbols)
   - Copilot descubrirá los 19 motores automáticamente
   - Invoca: `@qdrant search for machine learning`

3. **Integración con Claude/ChatGPT:**
   - USA `mcp://memory-p-api.workers.dev/mcp` 
   - Copilot indexa los 19 motores como tools
   - Llama automáticamente al motor más apropiado

---

## 🔗 Conexión Directa via MCP URI

```
mcp://memory-p-api.workers.dev/mcp?
  auth=bearer+{jwt_token}&
  motor=qdrant&
  endpoint=search
```

---

## 📊 Monitoreo y Salud

### Ver todos los motores disponibles:
```bash
curl https://memory-p-api.workers.dev/health \
  -H "X-API-Key: dev-key-12345"
```

### Chequear salud de un motor específico:
```bash
curl -X POST https://memory-p-api.workers.dev/mcp/{motor}/health \
  -H "X-API-Key: dev-key-12345" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1}'
```

---

## 🚀 Deploy Status

✅ **Cloudflare Workers**: Live at https://memory-p-api.workers.dev  
✅ **MCP Protocol**: 2024-11-05 compatible  
✅ **JSON-RPC**: Version 2.0  
✅ **Authentication**: API Key + OAuth 2.0 PKCE  
✅ **19 Motors**: Todos disponibles  
✅ **CORS**: Habilitado para clientes web  

---

## 🔐 Production Checklist

- [ ] Reemplaza `dev-key-12345` con real API key
- [ ] Activa rate limiting en Cloudflare  
- [ ] Configura CORS según dominio
- [ ] Usar HTTPS (ya está en Cloudflare)
- [ ] Monitorea logs de errores
- [ ] Configura alertas de tiempo de respuesta (SLA < 100ms)

---

## 📞 Support

**Endpoints problemas:**
- Motor no encontrado? → Ver `/health` para lista completa
- Autenticación falló? → Verifica X-API-Key o token JWT
- Timeout? → Motor puede estar offline, intenta health check

**URL de monitoreo live:**
```
https://memory-p-api.workers.dev/health
```

---

**Listo para usar MCP con MEMORY_P en Cloudflare!** 🎉
