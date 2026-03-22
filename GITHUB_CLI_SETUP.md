# Setup MEMORY_P with GitHub CLI

**Last Updated**: 21 de marzo de 2026  
**Status**: ✅ Automated with GitHub CLI  

---

## 🎯 Quick Start (5 minutes)

### Option 1: Full Automated Setup (Recommended)

```powershell
# Run the complete setup
powershell -ExecutionPolicy Bypass -File setup-complete.ps1
```

This will:
1. ✅ Check GitHub CLI is installed & authenticated
2. ✅ Prompt for Cloudflare API Token
3. ✅ Prompt for Cloudflare Account ID
4. ✅ Add both secrets to GitHub automatically
5. ✅ Verify secrets are in GitHub
6. ✅ Test with dry-run (optional)

### Option 2: Manual with GitHub CLI

```powershell
# Add secrets one at a time
powershell -ExecutionPolicy Bypass -File add-github-secrets.ps1
```

---

## 📋 Prerequisites

### 1. Install GitHub CLI

**Windows**:
```powershell
# Using Chocolatey
choco install gh

# Or using Windows Package Manager
winget install GitHub.cli

# Or download from
https://cli.github.com
```

**Verify**:
```powershell
gh --version
```

### 2. Authenticate with GitHub

```powershell
gh auth login
```

Interactive prompts:
- **What's your preferred protocol?** → `HTTPS`
- **Authenticate Git with your GitHub credentials?** → `Y`
- **How would you like to authenticate?** → `Login with a web browser`

This opens your browser to approve CLI access.

### 3. Get Cloudflare Credentials

**Cloudflare API Token**:
1. https://dash.cloudflare.com/profile/api-tokens
2. Click "Create Token"
3. Select "Edit Cloudflare Workers"
4. Copy token

**Cloudflare Account ID**:
1. https://dash.cloudflare.com/
2. Right sidebar → "Account" section
3. Copy "Account ID"

---

## 🚀 Run Full Setup

```powershell
cd D:\REPOSITORIOS\memory_p_fresh
powershell -ExecutionPolicy Bypass -File setup-complete.ps1
```

**What happens**:

```
1️⃣  Checks GitHub CLI
2️⃣  Checks GitHub authentication
3️⃣  Prompts for API Token (securely)
4️⃣  Prompts for Account ID
5️⃣  Adds to GitHub via 'gh secret set'
6️⃣  Verifies both in GitHub
7️⃣  ✅ Done
```

---

## ✅ Verification

### Check Secrets in GitHub CLI

```powershell
gh secret list
```

Output should show:
```
CLOUDFLARE_ACCOUNT_ID    Updated 2026-03-21
CLOUDFLARE_API_TOKEN     Updated 2026-03-21
```

### Check Secrets in GitHub Web

https://github.com/Rigohl/MEMORY_P/settings/secrets/actions

Should display both secrets (values hidden).

---

## 🔄 Deploy After Setup

### Automatic Deploy (Recommended)

```powershell
# Just push to GitHub
git add .
git commit -m "Your changes"
git push origin master
```

**Then**:
1. GitHub Actions detects push
2. Triggers `.github/workflows/deploy-to-cloudflare.yml`
3. Uses secrets to deploy
4. Live in ~2-3 minutes

### Manual Deploy (If Needed)

```powershell
# Set environment variables
$env:CLOUDFLARE_API_TOKEN = "your-token"
$env:CLOUDFLARE_ACCOUNT_ID = "your-account-id"

# Run sync script
powershell -ExecutionPolicy Bypass -File sync-cloudflare.ps1
```

---

## 🧪 Test Deployment

### After Secrets Are Added

```powershell
# Test dry-run
npx wrangler deploy --env production --dry-run

# Check health endpoint
curl https://memory-p-api.workers.dev/health
```

---

## 📊 GitHub CLI Commands Reference

### View Secrets
```powershell
gh secret list                          # List all secrets
gh secret view CLOUDFLARE_API_TOKEN     # Show specific secret (value hidden)
```

### Add Secrets
```powershell
# Interactive
echo "secret-value" | gh secret set SECRET_NAME

# Or with pipeline
"secret-value" | gh secret set SECRET_NAME
```

### Delete Secrets
```powershell
gh secret delete CLOUDFLARE_API_TOKEN
gh secret delete CLOUDFLARE_ACCOUNT_ID
```

### Verify Current Repo
```powershell
gh repo view                      # Current repo info
gh secret list -R Rigohl/MEMORY_P # Secrets in Rigohl/MEMORY_P
```

---

## 🔐 Security Notes

### ✅ What GitHub CLI Does

- Encrypts secrets using repo's public key
- Stores encrypted in GitHub
- Only accessible to Actions workflows
- NOT visible in your local repository
- NOT stored in plaintext anywhere

### ✅ Best Practices

- ✅ Never commit secrets to git
- ✅ Use `gh secret` for all credentials
- ✅ Rotate tokens periodically
- ✅ Use `-AsSecureString` for interactive input
- ✅ Verify secrets are deployed correctly

---

## 🆘 Troubleshooting

### "gh command not found"

```powershell
# Install GitHub CLI
winget install GitHub.cli

# Or check PATH
$env:PATH -split ';' | Where-Object { $_ -match 'GitHub' }
```

### "Not authenticated"

```powershell
# Re-authenticate
gh auth logout
gh auth login
```

### "Secret already exists"

```powershell
# Delete and re-add
gh secret delete CLOUDFLARE_API_TOKEN
echo "new-value" | gh secret set CLOUDFLARE_API_TOKEN
```

### "Permission denied"

- ✅ Must have push access to repo
- ✅ Must be authenticated with `gh auth login`
- ✅ Check: `gh auth status`

### Deployment fails after adding secrets

- ✅ Verify secrets have correct values
- ✅ Check `wrangler.toml` configuration
- ✅ View logs: `gh run list` → click run → view logs

---

## 📈 Deployment Flow (Complete)

```
Developer PC
    ↓
git push origin master
    ↓
GitHub (master updated)
    ↓
GitHub Actions workflow triggered
    ↓
Actions retrieves secrets:
  - CLOUDFLARE_API_TOKEN (from secret store)
  - CLOUDFLARE_ACCOUNT_ID (from secret store)
    ↓
Build & test
    ↓
Dry-run validation
    ↓
Deploy to Cloudflare
    ↓
Cloudflare Workers (production)
    ↓
Live: https://memory-p-api.workers.dev ✅
```

**Time**: ~2-3 minutes total

---

## 📚 References

- **GitHub CLI Docs**: https://cli.github.com/manual
- **GitHub Secrets**: https://docs.github.com/en/actions/security-guides/encrypted-secrets
- **Cloudflare API Tokens**: https://dash.cloudflare.com/profile/api-tokens
- **GitHub Actions**: https://github.com/Rigohl/MEMORY_P/actions

---

## ✅ Checklist

- [ ] GitHub CLI installed (`gh --version`)
- [ ] GitHub authenticated (`gh auth login`)
- [ ] Cloudflare API Token copied
- [ ] Cloudflare Account ID copied
- [ ] Run `setup-complete.ps1`
- [ ] Verify secrets: `gh secret list`
- [ ] Push to GitHub
- [ ] Check deployment: https://github.com/Rigohl/MEMORY_P/actions
- [ ] Test endpoint: `curl https://memory-p-api.workers.dev/health`

---

**Status**: ✅ **READY TO DEPLOY**  
**Next**: Run `setup-complete.ps1`
