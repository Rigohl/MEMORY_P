# GitHub Actions Setup for MEMORY_P Gateway

## Configuration Required

To enable automatic deployments, add these secrets to GitHub:

### 1. Cloudflare API Token

Go to: https://dash.cloudflare.com/profile/api-tokens
- Click "Create Token"
- Select "Edit Cloudflare Workers"
- Copy token

### 2. Add as GitHub Secret

1. Go to repo: https://github.com/Rigohl/MEMORY_P
2. Settings → Secrets and variables → Actions
3. New repository secret:
   - Name: `CLOUDFLARE_API_TOKEN`
   - Value: (paste token)

4. New repository secret:
   - Name: `CLOUDFLARE_ACCOUNT_ID`
   - Value: (get from Cloudflare dashboard → Account details)

### 3. Update wrangler.toml Secrets

```bash
wrangler secret put JWT_SECRET --env production
wrangler secret put OAUTH_CLIENT_SECRET --env production
```

## Auto-Deploy Trigger

Deployments trigger automatically when:
- ✅ Push to `master` branch
- ✅ Changes to: `cloudflare-worker.ts`, `wrangler.toml`, `tsconfig.json`, `package.json`

## Manual Deploy

If needed, trigger manually:
```
GitHub → Actions → Deploy MEMORY_P Gateway to Cloudflare → Run workflow
```

## Monitor Deployments

View logs:
- https://github.com/Rigohl/MEMORY_P/actions

Check live status:
```bash
curl https://memory-p-api.workers.dev/health
```
