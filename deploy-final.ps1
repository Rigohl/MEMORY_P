#!/usr/bin/env powershell
# MEMORY_P Automated Cloudflare Deployment
# Usage: powershell -ExecutionPolicy Bypass -File deploy-final.ps1

param(
    [string]$CloudflareToken = '',
    [string]$ApiKey = 'change-this-in-production',
    [switch]$Production
)

Write-Host '╔═══════════════════════════════════════════════════════════╗'
Write-Host '║  MEMORY_P - FINAL CLOUDFLARE DEPLOYMENT AUTOMATION       ║'
Write-Host '╚═══════════════════════════════════════════════════════════╝'
Write-Host ''

# Step 1: Validate
if (-not $CloudflareToken) {
    Write-Host '❌ ERROR: Cloudflare API token required'
    Write-Host ''
    Write-Host 'Usage:'
    Write-Host "  powershell -ExecutionPolicy Bypass -File deploy-final.ps1 -CloudflareToken 'your-token' -Production"
    Write-Host ''
    Write-Host 'Get token: https://dash.cloudflare.com/ → API Tokens'
    exit 1
}

if ($ApiKey -eq 'change-this-in-production' -and $Production) {
    Write-Host '⚠️  WARNING: Using default API key in production!'
    Write-Host "Set custom key: -ApiKey 'your-secure-key'"
    Write-Host ''
    $confirm = Read-Host 'Continue? (y/N)'
    if ($confirm -ne 'y') {
        Write-Host 'Deployment cancelled'
        exit 0
    }
}

# Step 2: Update wrangler.toml with API key
Write-Host '📝 Updating wrangler.toml with API key...'
$wranglerPath = 'wrangler.toml'
if (Test-Path $wranglerPath) {
    $content = Get-Content $wranglerPath -Raw
    $content = $content -replace 'MEMORY_P_API_KEY = ".*?"', "MEMORY_P_API_KEY = `"$ApiKey`""
    Set-Content $wranglerPath $content
    Write-Host '✓ Updated API key in wrangler.toml'
}
else {
    Write-Host '❌ wrangler.toml not found'
    exit 1
}

# Step 3: Install/update dependencies
Write-Host ''
Write-Host '📦 Installing dependencies...'
npm install --save-dev wrangler@latest 2>&1 | Select-Object -Last 3
Write-Host '✓ Dependencies ready'

# Step 4: Dry-run
Write-Host ''
Write-Host '🔍 Running dry-run...'
$env:CLOUDFLARE_API_TOKEN = $CloudflareToken
$dryRunOutput = npx wrangler deploy --dry-run 2>&1
if ($dryRunOutput -match 'error') {
    Write-Host '❌ Dry-run failed:'
    Write-Host $dryRunOutput
    exit 1
}
Write-Host '✓ Dry-run successful'

# Step 5: Compile TypeScript
Write-Host ''
Write-Host '🔨 Compiling TypeScript...'
npm run build 2>&1 | Select-Object -Last 2
if ($LASTEXITCODE -ne 0) {
    Write-Host '❌ TypeScript compilation failed'
    exit 1
}
Write-Host '✓ TypeScript compiled'

# Step 6: Deploy
Write-Host ''
Write-Host '🚀 Deploying to Cloudflare...'
$environment = if ($Production) { 'production' } else { 'development' }
$deployOutput = npx wrangler deploy --env $environment 2>&1
Write-Host $deployOutput

# Step 7: Verify
if ($deployOutput -match 'Deployed to:') {
    Write-Host ''
    Write-Host '╔═══════════════════════════════════════════════════════════╗'
    Write-Host '║  ✅ DEPLOYMENT SUCCESSFUL!                              ║'
    Write-Host '╚═══════════════════════════════════════════════════════════╝'
    Write-Host ''
    $url = $deployOutput | Select-String 'Deployed to:' | ForEach-Object { $_ -match 'https.*'; $matches[0] } | Select-Object -First 1
    if ($url) {
        Write-Host "🌐 Gateway URL: $url"
    }
    else {
        Write-Host '🌐 Gateway URL: https://memory-p-api.workers.dev'
    }
    Write-Host ''
    Write-Host "🔐 API Key (store securely): $ApiKey"
    Write-Host ''
    Write-Host '📝 Test endpoint:'
    Write-Host '  curl -X POST https://memory-p-api.workers.dev/mcp/qdrant/search \'
    Write-Host "    -H 'X-API-Key: $ApiKey' \"
    Write-Host "    -H 'Content-Type: application/json' \"
    Write-Host "    -d '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"search\",\"params\":{}}'"
    Write-Host ''
}
else {
    Write-Host ''
    Write-Host '❌ DEPLOYMENT FAILED'
    Write-Host 'Check logs: ~/.wrangler/logs/'
    exit 1
}
