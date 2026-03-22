# 🚀 Cómo Completar el Deploy en Cloudflare

## 🔐 IMPORTANTE: Autenticación Configurada

The gateway now requires **API Key authentication** for all endpoints.

**SOLO /health es pública** (sin autenticación)

---

## Paso 1: Obtener API Token de Cloudflare

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

## Paso 2: Cambiar API Key de la Aplicación

**ANTES de hacer deploy en producción**, cambiar la API key en `wrangler.toml`:

```toml
[env.production]
vars = { MEMORY_P_API_KEY = "tu-clave-segura-aqui" }
```

Usar una clave:
- 32+ caracteres
- Aleatoria y única
- NO la clave por defecto

---

## Paso 3: Deploy Local

```powershell
# En PowerShell, setear variable de entorno y hacer deploy:
$env:CLOUDFLARE_API_TOKEN = "tu_token_aqui"
npx wrangler deploy --env production
```

**Alternativamente** (en Git Bash o terminal Unix):
```bash
export CLOUDFLARE_API_TOKEN="tu_token_aqui"
wrangler deploy --env production
```

## Paso 4: Verificar Deploy

Cuando deployment sea exitoso verás algo como:
```
✓ Uploaded memory-p-api (4.98 KiB gzip)
Deployed to: https://memory-p-api.workers.dev
```

## Paso 5: Probar Endpoints con Autenticación

⚠️ **TODOS los endpoints (excepto /health) ahora requieren API Key**

### 1. Health check (SIN autenticación)
```bash
curl https://memory-p-api.workers.dev/health
```

### 2. Con header X-API-Key (RECOMENDADO)
```bash
# Reemplazar "dev-key-12345" con tu API key
curl -X POST https://memory-p-api.workers.dev/mcp/qdrant/search \
  -H 'X-API-Key: dev-key-12345' \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"search","params":{"vector":[0.1,0.2]}}'
```

### 3. Test con Tantivy
```bash
curl -X POST https://memory-p-api.workers.dev/mcp/tantivy/search \
  -H 'X-API-Key: dev-key-12345' \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"search","params":{"query":"test"}}'
```

---

## 🔐 API Key Management

**Default Keys:**
- Development: `dev-key-12345`
- Production: `change-this-in-production` ⚠️ CAMBIAR

**Cambiar en producción:**
1. Editar `wrangler.toml` con tu clave segura (32+ caracteres)
2. Redeploy: `npx wrangler deploy --env production`

Ver: [API_AUTH.md](API_AUTH.md) para más detalles

---

## Configuración Completada ✅

- ✅ wrangler.toml configurado
- ✅ cloudflare-worker.ts compilable + autenticación
- ✅ KV Namespace configurado
- ✅ 19 motores enrutados (puertos 3010-3028)
- ✅ API Key authentication
- ✅ Dry-run exitoso

**Solo falta:** Tu API token de Cloudflare (y cambiar la API Key en wrangler.toml para producción)

---

## ⚠️ Seguridad

⚠️ **NO guardes tu token en archivos o Git**
- Solo usado para este deploy
- Revocable desde Cloudflare Dashboard
- Token nunca se envía a repositorio

