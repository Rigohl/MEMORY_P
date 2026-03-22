# GitHub ↔ Cloudflare Synchronization

**Status**: ✅ Automated bi-directional sync configured  
**Source of Truth**: GitHub (`master` branch)  
**Deployment Target**: Cloudflare Workers  

---

## 🔄 Architecture

```
GitHub (master) 
    ↓ (Push trigger)
    → GitHub Actions  
        → Build TypeScript
        → Run tests
        → Dry-run validation
        → Deploy to Cloudflare
    ↓
Cloudflare Workers (production)
    ↓
https://memory-p-api.workers.dev (live endpoint)
```

---

## ⚙️ Automatic Deployment (GitHub Actions)

### Workflow: `.github/workflows/deploy-to-cloudflare.yml`

**Triggers**:
- ✅ Any push to `master` branch with changes to:
  - `cloudflare-worker.ts`
  - `wrangler.toml`
  - `tsconfig.json`
  - `package.json`
  - `.github/workflows/deploy-to-cloudflare.yml`
- ✅ Manual trigger via GitHub Actions UI

**Steps**:
1. Checkout latest code
2. Install Node.js + dependencies
3. Compile TypeScript (`npm run build`)
4. Dry-run on Cloudflare
5. Deploy to production environment
6. Health check (POST /health)
7. Notify on success/failure

### Setup Required

**1. Add Cloudflare Secrets to GitHub**

Go to: https://github.com/Rigohl/MEMORY_P/settings/secrets/actions

Create 2 secrets:

```
CLOUDFLARE_API_TOKEN = your-api-token
CLOUDFLARE_ACCOUNT_ID = your-account-id
```

**Getting these values**:

**Cloudflare API Token**:
1. https://dash.cloudflare.com/profile/api-tokens
2. "Create Token" → "Edit Cloudflare Workers"
3. Copy token

**Account ID**:
1. https://dash.cloudflare.com/
2. Account details (right sidebar)
3. Copy "Account ID"

**2. Set Cloudflare Secrets**

```bash
wrangler secret put JWT_SECRET --env production
wrangler secret put OAUTH_CLIENT_SECRET --env production
```

Prompts will ask for values. Enter production secrets (not from git).

**3. Verify Setup**

```bash
# Check secrets are configured (won't show values)
wrangler secret list --env production
```

---

## 🔧 Manual Sync (PowerShell Script)

### Usage

```powershell
# Basic sync (with confirmation)
powershell -ExecutionPolicy Bypass -File sync-cloudflare.ps1

# Dry-run (no deployment)
powershell -ExecutionPolicy Bypass -File sync-cloudflare.ps1 -DryRun

# Force (no git verification)
powershell -ExecutionPolicy Bypass -File sync-cloudflare.ps1 -Force

# With explicit token
powershell -ExecutionPolicy Bypass -File sync-cloudflare.ps1 `
  -CloudflareToken "your-token" `
  -AccountId "your-account-id"
```

### What It Does

1. ✅ Verifies all commits are pushed
2. ✅ Pulls latest from GitHub master
3. ✅ Verifies Cloudflare config
4. ✅ Compiles TypeScript
5. ✅ Runs dry-run validation
6. ✅ Asks for confirmation
7. ✅ Deploys to Cloudflare
8. ✅ Health checks deployment
9. ✅ Reports status

### Environment Variables

Set before running:

```powershell
$env:CLOUDFLARE_API_TOKEN = "your-token"
$env:CLOUDFLARE_ACCOUNT_ID = "your-account-id"
```

Or pass as parameters:
```powershell
-CloudflareToken "token" -AccountId "id"
```

---

## 📊 Deployment Flow

### When You Push to GitHub

```
1. git push origin master
   ↓
2. GitHub detects push to master
   ↓
3. GitHub Actions workflow triggered
   ↓
4. Checkout, build, test
   ↓
5. Dry-run validation on Cloudflare
   ↓
6. If OK → Deploy to production
   ↓
7. Health check API
   ↓
8. Notify (email/webhook if configured)
   ↓
9. LIVE at: https://memory-p-api.workers.dev
```

**Time**: ~2-3 minutes from push to live

### Manual Sync

```
1. powershell sync-cloudflare.ps1
   ↓
2. Verify git status
   ↓
3. Pull latest from GitHub
   ↓
4. Build locally
   ↓
5. Dry-run
   ↓
6. Confirm deployment
   ↓
7. Deploy
   ↓
8. Health check
   ↓
9. Done
```

**Time**: ~1-2 minutes

---

## 🔍 Monitoring

### View Deployment Logs

**GitHub Actions**:
- https://github.com/Rigohl/MEMORY_P/actions
- Click latest run
- View "Deploy to Cloudflare Workers" job

### Check Live Status

```bash
# Health check (no auth required)
curl https://memory-p-api.workers.dev/health

# API call with key
curl -H "X-API-Key: your-key" \
  https://memory-p-api.workers.dev/mcp/qdrant/search
```

### Real-time Logs

```bash
# Tail Cloudflare Worker logs
wrangler tail --env production

# Or via Cloudflare dashboard:
# https://dash.cloudflare.com/ → Workers → memory-p-api → Logs
```

---

## 🚨 Troubleshooting

### "Deployment failed" in GitHub Actions

Check logs:
1. https://github.com/Rigohl/MEMORY_P/actions
2. Click failing workflow
3. Expand "Deploy to Cloudflare" step
4. Read error message

Common causes:
- Missing secrets (`CLOUDFLARE_API_TOKEN`)
- Invalid API token (expired or insufficient permissions)
- TypeScript compilation errors
- Invalid wrangler.toml configuration

### "Unable to authenticate" locally

```powershell
# Verify token is set
Write-Host $env:CLOUDFLARE_API_TOKEN

# Re-authenticate
wrangler login

# Or set token directly
$env:CLOUDFLARE_API_TOKEN = "your-token-here"
```

### "Health check failed"

```bash
# Test immediately after deployment
curl https://memory-p-api.workers.dev/health

# Check if endpoint is responding
# May take 10-30 seconds after deployment

# If still failing, check logs:
wrangler tail --env production
```

### "Uncommitted changes detected"

The sync script will prompt to commit:
```
⚠️  WARNING: Uncommitted changes detected
Commit changes first? (y/N)
```

Choose `y` to commit, or use `--Force` flag to bypass.

---

## 📋 Workflow Summary

| Action | Trigger | Status | Time |
|--------|---------|--------|------|
| GitHub Action Deploy | Push to master | Automatic | 2-3 min |
| Manual Sync | Run script | Manual | 1-2 min |
| Health Check | Automatic | Automatic | <1 sec |
| Logs | GitHub Actions UI | Manual | Real-time |

---

## 🔐 Security

### Secrets Management

- ✅ API tokens stored in GitHub Secrets (encrypted)
- ✅ Never committed to repository
- ✅ Only accessible during GitHub Actions runs
- ✅ Cloudflare secrets separate (via `wrangler secret put`)

### Deployment Verification

- ✅ TypeScript compiled (no runtime errors)
- ✅ Dry-run validation before production
- ✅ Health check after deployment
- ✅ Automatic rollback on failure (old Worker version stays live)

### Access Control

- ✅ Only pushed to if: Administrator approval or branch protection rules
- ✅ Deployments require valid API credentials
- ✅ All deployments logged in GitHub Actions
- ✅ All deployments logged in Cloudflare

---

## 📚 References

- **GitHub Actions**: https://github.com/Rigohl/MEMORY_P/actions
- **Cloudflare Workers**: https://dash.cloudflare.com/
- **Wrangler CLI**: https://developers.cloudflare.com/workers/wrangler/
- **API Tokens**: https://dash.cloudflare.com/profile/api-tokens

---

**Status**: ✅ **CONFIGURED AND AUTOMATED**  
**Next**: Obtain Cloudflare credentials and add to GitHub Secrets  
**Result**: Zero-downtime deployments from GitHub to production
