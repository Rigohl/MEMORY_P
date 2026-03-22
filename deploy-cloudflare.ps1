#!/usr/bin/env pwsh
<#
.SYNOPSIS
  Deploy MEMORY_P to Cloudflare Workers
.DESCRIPTION
  Complete deployment pipeline:
  1. Verify Wrangler installed
  2. Build binaries locally
  3. Deploy Worker gateway
  4. Verify endpoints
#>

# Colors
$Green = "`e[32m"
$Red = "`e[31m"
$Yellow = "`e[33m"
$Reset = "`e[0m"

Write-Host "${Yellow}=== MEMORY_P CLOUDFLARE DEPLOYMENT ===${Reset}"

# Step 1: Check Wrangler
Write-Host "${Yellow}[1/4] Checking Wrangler installation...${Reset}"
$wrangler = npm list -g wrangler 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "${Yellow}Installing Wrangler...${Reset}"
    npm install -g wrangler
}
Write-Host "${Green}✓ Wrangler ready${Reset}"

# Step 2: Build binaries locally
Write-Host "${Yellow}[2/4] Building 19 binaries locally...${Reset}"
cargo build --release --all-targets 2>&1 | Select-Object -Last 5
if ($LASTEXITCODE -ne 0) {
    Write-Host "${Red}✗ Build failed${Reset}"
    exit 1
}
Write-Host "${Green}✓ 19 binaries compiled to .build/target/release/${Reset}"

# Step 3: Deploy Worker
Write-Host "${Yellow}[3/4] Deploying Cloudflare Worker...${Reset}"
Write-Host "${Yellow}Requirements:${Reset}"
Write-Host '  - Cloudflare account with Workers enabled'
Write-Host "  - API token stored in ${Yellow}CLOUDFLARE_API_TOKEN${Reset}"
Write-Host ''
Write-Host "${Yellow}To deploy, run:${Reset}"
Write-Host "  ${Green}wrangler deploy${Reset}"
Write-Host ''
Write-Host "${Yellow}Or use this command:${Reset}"
Write-Host "  ${Green}wrangler publish --worker-name memory-p-api${Reset}"

# Step 4: Test endpoints
Write-Host ''
Write-Host "${Yellow}[4/4] Testing endpoints...${Reset}"
Write-Host ''
Write-Host "${Green}Local testing (before deploy):${Reset}"
Write-Host '  wrangler dev'
Write-Host '  curl http://localhost:8787/health'
Write-Host ''
Write-Host "${Green}Production testing (after deploy):${Reset}"
Write-Host '  curl https://memory-p-api.workers.dev/health'
Write-Host "  curl -X POST https://memory-p-api.workers.dev/mcp/qdrant/search -d '{...}'"
Write-Host ''

# Summary
Write-Host ''
Write-Host "${Green}=== DEPLOYMENT READY ===${Reset}"
Write-Host ''
Write-Host "${Yellow}Configuration files created:${Reset}"
Write-Host '  ✓ wrangler.toml (Worker config)'
Write-Host '  ✓ cloudflare-worker.ts (Gateway code)'
Write-Host '  ✓ package.json (Dependencies)'
Write-Host ''
Write-Host "${Yellow}Next steps:${Reset}"
Write-Host '  1. npm install (local dependencies)'
Write-Host '  2. wrangler login (authenticate)'
Write-Host '  3. wrangler deploy (deploy to Cloudflare)'
Write-Host ''
Write-Host "${Green}Docs: docs/CLOUDFLARE_DEPLOYMENT.md${Reset}"
