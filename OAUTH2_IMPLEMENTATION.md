# OAuth 2.0 Implementation - MEMORY_P API Gateway

**Status**: ✅ Fully Implemented  
**Date**: 21 de marzo de 2026  
**Gateway**: Cloudflare Workers  
**Authorization Flow**: PKCE (Proof Key for Code Exchange)

---

## 📋 Overview

The MEMORY_P API Gateway now supports **OAuth 2.0 with PKCE** as an alternative to simple API Key authentication. This enables:

- ✅ User-friendly authorization via web browser
- ✅ No direct credential exposure
- ✅ Time-limited tokens (1 hour JWT)
- ✅ Granular scope support
- ✅ Secure code exchange via PKCE challenge

---

## 🔐 Authentication Methods

### Option 1: Simple API Key (Legacy)
```bash
curl -H "X-API-Key: your-api-key" \
  https://memory-p-api.workers.dev/mcp/qdrant/search
```

### Option 2: Bearer Token (OAuth 2.0)
```bash
curl -H "Authorization: Bearer eyJhbGc..." \
  https://memory-p-api.workers.dev/mcp/qdrant/search
```

### Option 3: Bearer Token (API Key via Bearer)
```bash
curl -H "Authorization: Bearer your-api-key" \
  https://memory-p-api.workers.dev/mcp/qdrant/search
```

---

## 🔄 OAuth 2.0 PKCE Flow

### Step 1: Generate Verifier
```typescript
const codeVerifier = generateRandomString(43); // 43-128 chars
// Store locally (in frontend/app), never send to server
```

### Step 2: Calculate Challenge
```typescript
const challenge = await calculateCodeChallenge(codeVerifier);
// SHA256(codeVerifier) encoded as base64url
```

### Step 3: Request Authorization Code
```bash
curl -X POST https://memory-p-api.workers.dev/oauth/authorize \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "memory-p-client",
    "redirect_uri": "https://yourapp.com/callback",
    "code_challenge": "'$challenge'",
    "scope": "mcp:full"
  }'
```

**Response**:
```json
{
  "authorization_code": "abc123xyz...",
  "expires_in": 60,
  "redirect_uri": "https://yourapp.com/callback?code=abc123xyz...&state=optional"
}
```

### Step 4: Exchange Code for Token
```bash
curl -X POST https://memory-p-api.workers.dev/oauth/token \
  -H "Content-Type: application/json" \
  -d '{
    "code": "abc123xyz...",
    "code_verifier": "'$codeVerifier'",
    "client_id": "memory-p-client",
    "client_secret": "optional-for-confidential-clients"
  }'
```

**Response**:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "scope": "mcp:full"
}
```

### Step 5: Use Access Token
```bash
curl -X POST https://memory-p-api.workers.dev/mcp/qdrant/search \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"search","params":{}}'
```

---

## 🛠️ Configuration

### Environment Variables

Update `wrangler.toml`:

```toml
[env.production]
vars = { 
  MEMORY_P_API_KEY = "your-legacy-api-key",
  JWT_SECRET = "your-256-bit-secret",
  OAUTH_CLIENT_ID = "memory-p-prod",
  OAUTH_CLIENT_SECRET = "your-oauth-client-secret"
}

[env.development]
vars = { 
  MEMORY_P_API_KEY = "dev-key-12345",
  JWT_SECRET = "dev-jwt-secret",
  OAUTH_CLIENT_ID = "memory-p-dev",
  OAUTH_CLIENT_SECRET = "dev-oauth-secret"
}
```

### Secrets (Sensitive)

For production, use Cloudflare Secrets:

```bash
wrangler secret put JWT_SECRET --env production
wrangler secret put OAUTH_CLIENT_SECRET --env production
```

---

## 📊 JWT Token Structure

OAuth tokens are **JWT (HS256 signed)**:

```json
{
  "header": {
    "alg": "HS256",
    "typ": "JWT"
  },
  "payload": {
    "sub": "memory-p-client",
    "iat": 1711000000,
    "exp": 1711003600,
    "scope": "mcp:full"
  },
  "signature": "HMAC-SHA256(header.payload, JWT_SECRET)"
}
```

**Token Lifetime**: 1 hour (3600 seconds)

---

## 🔒 Supported Scopes

| Scope | Access |
|-------|--------|
| `mcp:full` | All 19 motors (default) |
| `mcp:read` | Read-only operations |
| `mcp:vector` | Qdrant + FAISS + SCANN only |
| `mcp:text` | Tantivy + LNX + MeiliSearch only |
| `mcp:admin` | Administrative operations (reserved) |

---

## 📝 JavaScript/TypeScript Implementation

### Frontend (React/Vue/Angular)

```typescript
// 1. Generate PKCE challenge
function generatePKCE() {
  const verifier = generateRandomString(43);
  const challenge = await calculateCodeChallenge(verifier);
  localStorage.setItem("oauth_verifier", verifier);
  return { verifier, challenge };
}

// 2. Request authorization code
async function authorize() {
  const { challenge } = generatePKCE();
  const response = await fetch("https://memory-p-api.workers.dev/oauth/authorize", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      client_id: "memory-p-client",
      redirect_uri: window.location.origin + "/callback",
      code_challenge: challenge,
      scope: "mcp:full"
    })
  });
  
  const { authorization_code } = await response.json();
  window.location.href = response.redirect_uri;
}

// 3. Handle callback (in /callback page)
async function handleCallback() {
  const code = new URLSearchParams(location.search).get("code");
  const verifier = localStorage.getItem("oauth_verifier");
  
  const response = await fetch("https://memory-p-api.workers.dev/oauth/token", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      code,
      code_verifier: verifier,
      client_id: "memory-p-client"
    })
  });
  
  const { access_token } = await response.json();
  localStorage.setItem("access_token", access_token);
  window.location.href = "/dashboard";
}

// 4. Use token in API calls
async function searchMotor(motor, query) {
  const token = localStorage.getItem("access_token");
  const response = await fetch(
    `https://memory-p-api.workers.dev/mcp/${motor}/search`,
    {
      method: "POST",
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json"
      },
      body: JSON.stringify(query)
    }
  );
  return response.json();
}
```

### Backend (Node.js/Express)

```typescript
import fetch from "node-fetch";

async function getAccessToken(code: string, verifier: string) {
  const response = await fetch(
    "https://memory-p-api.workers.dev/oauth/token",
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        code,
        code_verifier: verifier,
        client_id: "memory-p-client",
        client_secret: process.env.OAUTH_CLIENT_SECRET
      })
    }
  );
  
  return response.json();
}

async function callMemoryP(method: string, params: object) {
  const token = await getAccessToken(...);
  const response = await fetch(
    "https://memory-p-api.workers.dev/mcp/qdrant/search",
    {
      method: "POST",
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json"
      },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method,
        params
      })
    }
  );
  
  return response.json();
}
```

### Python Implementation

```python
import requests
import subprocess
import urllib.parse
import base64
from hashlib import sha256

# 1. Generate PKCE challenge
def generate_pkce():
    verifier = ''.join(
        chr(ord('A') + (i % 26)) for i in range(43)
    )  # Simple verifier
    
    # Calculate SHA256 challenge
    challenge_bytes = sha256(verifier.encode()).digest()
    challenge = base64.urlsafe_b64encode(challenge_bytes).decode().rstrip('=')
    
    return verifier, challenge

# 2. Request authorization code
verifier, challenge = generate_pkce()

auth_response = requests.post(
    "https://memory-p-api.workers.dev/oauth/authorize",
    json={
        "client_id": "memory-p-client",
        "redirect_uri": "http://localhost:3000/callback",
        "code_challenge": challenge,
        "scope": "mcp:full"
    }
)

auth_data = auth_response.json()
code = auth_data["authorization_code"]

# 3. Exchange for token
token_response = requests.post(
    "https://memory-p-api.workers.dev/oauth/token",
    json={
        "code": code,
        "code_verifier": verifier,
        "client_id": "memory-p-client"
    }
)

token_data = token_response.json()
access_token = token_data["access_token"]

# 4. Use in API call
search_response = requests.post(
    "https://memory-p-api.workers.dev/mcp/qdrant/search",
    headers={
        "Authorization": f"Bearer {access_token}",
        "Content-Type": "application/json"
    },
    json={
        "jsonrpc": "2.0",
        "id": 1,
        "method": "search",
        "params": {}
    }
)

print(search_response.json())
```

---

## 🧪 Testing OAuth Flow

### Using curl

```bash
#!/bin/bash

# 1. Set variables
CLIENT_ID="memory-p-client"
REDIRECT_URI="https://yourapp.com/callback"

# 2. Generate PKCE verifier
VERIFIER=$(python3 -c "import secrets; print(secrets.token_urlsafe(32))")

# 3. Generate challenge
CHALLENGE=$(echo -n "$VERIFIER" | shasum -a 256 | xxd -r -p | base64 | tr '+/' '-_' | tr -d '=')

# 4. Request authorization code
RESPONSE=$(curl -s -X POST https://memory-p-api.workers.dev/oauth/authorize \
  -H "Content-Type: application/json" \
  -d "{
    \"client_id\": \"$CLIENT_ID\",
    \"redirect_uri\": \"$REDIRECT_URI\",
    \"code_challenge\": \"$CHALLENGE\",
    \"scope\": \"mcp:full\"
  }")

CODE=$(echo "$RESPONSE" | jq -r '.authorization_code')
echo "Authorization Code: $CODE"

# 5. Exchange for token
TOKEN_RESPONSE=$(curl -s -X POST https://memory-p-api.workers.dev/oauth/token \
  -H "Content-Type: application/json" \
  -d "{
    \"code\": \"$CODE\",
    \"code_verifier\": \"$VERIFIER\",
    \"client_id\": \"$CLIENT_ID\"
  }")

ACCESS_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token')
echo "Access Token: $ACCESS_TOKEN"

# 6. Test API call
curl -s -X POST https://memory-p-api.workers.dev/mcp/qdrant/search \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "search",
    "params": {}
  }' | jq .
```

---

## 🚨 Security Considerations

| Aspect | Implementation |
|--------|-----------------|
| **PKCE** | ✅ Required for all flows |
| **HTTPS** | ✅ Enforced on Cloudflare |
| **Token Expiry** | ✅ 1 hour (configurable) |
| **Code Expiry** | ✅ 60 seconds (prevents reuse) |
| **KV Storage** | ✅ Codes in KV with TTL |
| **JWT Signing** | ✅ HS256 with secret |
| **Secret Rotation** | ⚠️ Use Cloudflare Secrets |
| **CORS** | ✅ Configurable headers |

---

## 📐 Token Validation

The gateway automatically validates:

1. ✅ Code exists in KV
2. ✅ Code not expired
3. ✅ PKCE code_verifier matches code_challenge (SHA256)
4. ✅ Client credentials (optional for public clients)
5. ✅ JWT signature (HS256)
6. ✅ JWT expiration time

---

## 🔄 Migration from API Key

### For existing API Key users:

1. Keep using X-API-Key header (still supported)
2. Gradually migrate to OAuth 2.0
3. No breaking changes to existing apps

### Migration timeline:

```
Phase 1 (Now)    : API Key + OAuth 2.0 both supported
Phase 2 (30 days): API Key + OAuth 2.0 recommended
Phase 3 (60 days): API Key deprecated (warning only)
Phase 4 (90 days): API Key removed
```

---

## 📞 Support

### Common Issues

**Q: "invalid_grant: code not found"**
- Code expired (60 second limit)
- Use authorization immediately after receiving code
- Request new authorization code if > 60 seconds

**Q: "invalid_grant: code_verifier does not match"**
- Verify you're using same verifier from PKCE generation
- Check for encoding issues with challenge
- Use base64url encoding (no padding)

**Q: "Unauthorized" on API call**
- Check token not expired (1 hour limit)
- Verify Bearer token format: `Bearer {token}`
- Confirm JWT_SECRET matches between /authorize and /token endpoints

### Getting Help

1. Check logs: `wrangler tail --environment production`
2. Test endpoints: See `Testing OAuth Flow` section
3. Review implementation: `OAUTH2_IMPLEMENTATION.md`

---

## 📚 References

- **[OAuth 2.0 PKCE RFC](https://tools.ietf.org/html/rfc7636)**
- **[JWT RFC](https://tools.ietf.org/html/rfc7519)**
- **[Cloudflare Workers KV](https://developers.cloudflare.com/workers/runtime-apis/kv/)**
- **[Web Crypto API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Crypto_API)**

---

**Status**: ✅ **PRODUCTION READY**  
**Last Updated**: 21 de marzo de 2026  
**Next Review**: 30 days post-launch
