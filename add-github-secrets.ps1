#!/usr/bin/env powershell
# Add Cloudflare secrets to GitHub using GitHub CLI
# This ensures GitHub is source of truth for all configuration

param(
    [string]$CloudflareToken = '',
    [string]$CloudflareAccountId = '',
    [switch]$Verify
)

Write-Host '╔═══════════════════════════════════════════════════════════╗'
Write-Host '║  Add Cloudflare Secrets to GitHub using GitHub CLI       ║'
Write-Host '╚═══════════════════════════════════════════════════════════╝'
Write-Host ''

# Step 1: Check GitHub CLI is installed
Write-Host '🔍 Step 1: Checking GitHub CLI...'
$ghVersion = gh --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host '❌ GitHub CLI not found. Install from: https://cli.github.com'
    exit 1
}
Write-Host "✓ GitHub CLI installed: $ghVersion"
Write-Host ''

# Step 2: Check authentication
Write-Host '🔐 Step 2: Checking GitHub authentication...'
$currentUser = gh auth status 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host '❌ Not authenticated. Run: gh auth login'
    exit 1
}
Write-Host '✓ Authenticated with GitHub'
Write-Host ''

# Step 3: Get Cloudflare credentials
if (-not $CloudflareToken) {
    Write-Host '📝 Step 3: Enter Cloudflare credentials'
    Write-Host 'Get from: https://dash.cloudflare.com/profile/api-tokens'
    Write-Host ''
    $CloudflareToken = Read-Host -AsSecureString 'Cloudflare API Token'
    $CloudflareToken = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto([System.Runtime.InteropServices.Marshal]::SecureStringToCoTaskMemUnicodePtr($CloudflareToken))
}

if (-not $CloudflareAccountId) {
    Write-Host 'Get Account ID from: https://dash.cloudflare.com/'
    $CloudflareAccountId = Read-Host 'Cloudflare Account ID'
}

Write-Host '✓ Credentials received'
Write-Host ''

# Step 4: Add secrets to GitHub
Write-Host '🔐 Step 4: Adding secrets to GitHub...'
Write-Host '   - CLOUDFLARE_API_TOKEN'
Write-Host '   - CLOUDFLARE_ACCOUNT_ID'
Write-Host ''

# Add API token secret
Write-Host '   Adding CLOUDFLARE_API_TOKEN...'
$CloudflareToken | gh secret set CLOUDFLARE_API_TOKEN
if ($LASTEXITCODE -ne 0) {
    Write-Host '❌ Failed to add CLOUDFLARE_API_TOKEN'
    exit 1
}
Write-Host '   ✓ Added'

# Add Account ID secret
Write-Host '   Adding CLOUDFLARE_ACCOUNT_ID...'
Write-Output $CloudflareAccountId | gh secret set CLOUDFLARE_ACCOUNT_ID
if ($LASTEXITCODE -ne 0) {
    Write-Host '❌ Failed to add CLOUDFLARE_ACCOUNT_ID'
    exit 1
}
Write-Host '   ✓ Added'
Write-Host ''

# Step 5: Verify secrets
if ($Verify) {
    Write-Host '✅ Step 5: Verifying secrets...'
    gh secret list
    Write-Host ''
}

Write-Host '╔═══════════════════════════════════════════════════════════╗'
Write-Host '║  ✅ SECRETS ADDED SUCCESSFULLY                            ║'
Write-Host '║                                                           ║'
Write-Host '║  GitHub is now configured for Cloudflare deployments     ║'
Write-Host '║  Next: Verify in: https://github.com/Rigohl/MEMORY_P     ║'
Write-Host '║           Settings → Secrets and variables → Actions    ║'
Write-Host '╚═══════════════════════════════════════════════════════════╝'
