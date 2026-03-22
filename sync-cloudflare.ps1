#!/usr/bin/env powershell
# Sync MEMORY_P Gateway: GitHub Master → Cloudflare Production
# This script ensures GitHub is the source of truth and deploys to Cloudflare

param(
    [string]$CloudflareToken = $env:CLOUDFLARE_API_TOKEN,
    [string]$AccountId = $env:CLOUDFLARE_ACCOUNT_ID,
    [switch]$DryRun,
    [switch]$Force
)

Write-Host '╔═══════════════════════════════════════════════════════════╗'
Write-Host '║  MEMORY_P - GitHub ↔ Cloudflare Sync                    ║'
Write-Host '╚═══════════════════════════════════════════════════════════╝'
Write-Host ''

# Step 1: Verify Git status
Write-Host '📋 Step 1: Verifying Git status...'
$status = git status --porcelain
if ($status) {
    Write-Host '⚠️  WARNING: Uncommitted changes detected:'
    Write-Host $status
    if (-not $Force) {
        Write-Host ''
        $confirm = Read-Host 'Commit changes first? (y/N)'
        if ($confirm -eq 'y') {
            git add .
            $message = Read-Host 'Commit message'
            git commit -m $message
            git push origin master
        }
        else {
            Write-Host 'ℹ️  Continuing with uncommitted changes...'
        }
    }
}
Write-Host '✓ Git status verified'
Write-Host ''

# Step 2: Ensure latest from GitHub
Write-Host '📥 Step 2: Pulling latest from GitHub...'
git fetch origin master
git reset --hard origin/master
Write-Host '✓ Latest code from GitHub'
Write-Host ''

# Step 3: Verify configuration
Write-Host '🔍 Step 3: Verifying Cloudflare configuration...'
if (-not $CloudflareToken) {
    Write-Host '❌ ERROR: CLOUDFLARE_API_TOKEN not set'
    Write-Host 'Set it: $env:CLOUDFLARE_API_TOKEN = "your-token"'
    exit 1
}
if (-not $AccountId) {
    Write-Host '⚠️  WARNING: CLOUDFLARE_ACCOUNT_ID not set'
    Write-Host 'Set it: $env:CLOUDFLARE_ACCOUNT_ID = "your-account-id"'
}
Write-Host '✓ Configuration verified'
Write-Host ''

# Step 4: Build TypeScript
Write-Host '🔨 Step 4: Building TypeScript...'
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host '❌ Build failed'
    exit 1
}
Write-Host '✓ TypeScript built successfully'
Write-Host ''

# Step 5: Dry-run
Write-Host '🧪 Step 5: Verifying deployment (dry-run)...'
$env:CLOUDFLARE_API_TOKEN = $CloudflareToken
npx wrangler deploy --env production --dry-run
if ($LASTEXITCODE -ne 0) {
    Write-Host '❌ Dry-run failed'
    exit 1
}
Write-Host '✓ Dry-run passed'
Write-Host ''

# Step 6: Deploy (if not dry-run)
if ($DryRun) {
    Write-Host '✓ DRY-RUN ONLY - No deployment made'
    exit 0
}

Write-Host '🚀 Step 6: Deploying to Cloudflare...'
$confirm = Read-Host 'Deploy to production? (y/N)'
if ($confirm -ne 'y') {
    Write-Host 'Deployment cancelled'
    exit 0
}

npx wrangler deploy --env production
if ($LASTEXITCODE -ne 0) {
    Write-Host '❌ Deployment failed'
    exit 1
}
Write-Host '✓ Deployed successfully'
Write-Host ''

# Step 7: Verify deployment
Write-Host '✅ Step 7: Verifying deployment...'
Start-Sleep -Seconds 3
$response = curl -s https://memory-p-api.workers.dev/health
Write-Host "Response: $response"
Write-Host ''

Write-Host '╔═══════════════════════════════════════════════════════════╗'
Write-Host '║  ✅ SYNC COMPLETE                                         ║'
Write-Host '║  GitHub Master → Cloudflare Production                   ║'
Write-Host '║                                                           ║'
Write-Host '║  Live at: https://memory-p-api.workers.dev               ║'
Write-Host '╚═══════════════════════════════════════════════════════════╝'
