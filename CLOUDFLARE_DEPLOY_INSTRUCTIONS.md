# 🚀 Cómo Completar el Deploy en Cloudflare

## Paso 1: Obtener API Token

1. **Ir a Cloudflare Dashboard:**
   - https://dash.cloudflare.com/

2. **Crear API Token:**
   - Ve a: Account Settings → API Tokens
   - Click en "Create Token"
   - Selecciona "Get started" en "Edit Cloudflare Workers"
   - Permisos: Account.Workers Scripts (Edit)
   - Cuentas: Selecciona tu account
   - Autorizar dominio (deja vacío para todos)
   - Click "Create Token"

3. **Copiar Token**

## Paso 2: Deploy Local

```powershell
# En PowerShell, setear variable de entorno y hacer deploy:
$env:CLOUDFLARE_API_TOKEN = "tu_token_aqui"
npx wrangler deploy
```

**Alternativamente** (en Git Bash o terminal Unix):
```bash
export CLOUDFLARE_API_TOKEN="tu_token_aqui"
wrangler deploy
```

## Paso 3: Verificar Deploy

Cuando deployment sea exitoso verás algo como:
```
✓ Uploaded memory-p-api (4.98 KiB gzip)
Deployed to: https://memory-p-api.workers.dev
```

## Paso 4: Probar Endpoints

```bash
# Health check
curl https://memory-p-api.workers.dev/health

# Test con Qdrant
curl -X POST https://memory-p-api.workers.dev/mcp/qdrant/search \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"search","params":{"vector":[0.1,0.2]}}'

# Test con Tantivy
curl -X POST https://memory-p-api.workers.dev/mcp/tantivy/search \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"search","params":{"query":"test"}}'
```

---

## Configuración Completada ✅

- ✅ wrangler.toml configurado
- ✅ cloudflare-worker.ts compilable
- ✅ KV Namespace configurado
- ✅ 19 motores enrutados (puertos 3010-3028)
- ✅ Dry-run exitoso

**Solo falta:** Tu API token de Cloudflare

---

## ⚠️ Seguridad

⚠️ **NO guardes tu token en archivos o Git**
- Solo usado para este deploy
- Revocable desde Cloudflare Dashboard
- Token nunca se envía a repositorio

