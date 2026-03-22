#!/usr/bin/env powershell
# Complete setup flow: GitHub CLI + Cloudflare secrets
# This is the all-in-one solution for configuring MEMORY_P deployments

param(
    [switch]$AutoDeploy
)

Write-Host '╔══════════════════════════════════════════════════════════════╗'
Write-Host '║  MEMORY_P - Complete GitHub + Cloudflare Setup             ║'
Write-Host '║  (GitHub CLI Automated)                                    ║'
Write-Host '╚══════════════════════════════════════════════════════════════╝'
Write-Host ''

# ============================================================================
# PART 1: PREREQUISITES
# ============================================================================

Write-Host '📋 PART 1: Checking prerequisites...'
Write-Host ''

# Check GitHub CLI
Write-Host '  Checking GitHub CLI...'
$ghExists = (Get-Command gh -ErrorAction SilentlyContinue) -ne $null
if (-not $ghExists) {
    Write-Host '  ❌ GitHub CLI not found'
    Write-Host '  Install from: https://cli.github.com'
    Write-Host ''
    Write-Host '  After installing, run:'
    Write-Host '    gh auth login'
    exit 1
}
Write-Host '  ✓ GitHub CLI installed'

# Check GitHub auth
Write-Host '  Checking GitHub authentication...'
$currentAuth = gh auth status --show-token 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host '  ❌ Not authenticated with GitHub'
    Write-Host '  Run: gh auth login'
    exit 1
}
Write-Host '  ✓ Authenticated'

# Check Wrangler
Write-Host '  Checking Wrangler CLI...'
$wranglerExists = (Get-Command wrangler -ErrorAction SilentlyContinue) -ne $null
if (-not $wranglerExists) {
    Write-Host '  ⚠️  Wrangler CLI not found (needed for JWT secrets)'
    Write-Host '  Install: npm install -g wrangler'
}
else {
    Write-Host '  ✓ Wrangler CLI installed'
}

Write-Host ''
Write-Host '✅ Prerequisites OK'
Write-Host ''

# ============================================================================
# PART 2: GET CLOUDFLARE API TOKEN
# ============================================================================

Write-Host '🔐 PART 2: Cloudflare API Token'
Write-Host ''
Write-Host '  1. Go to: https://dash.cloudflare.com/profile/api-tokens'
Write-Host '  2. Click "Create Token"'
Write-Host '  3. Select "Edit Cloudflare Workers"'
Write-Host '  4. Copy the token (will look like: abc123xyz...)'
Write-Host ''

$apiToken = Read-Host -AsSecureString '  Paste Cloudflare API Token'
$apiTokenPlain = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto(
    [System.Runtime.InteropServices.Marshal]::SecureStringToCoTaskMemUnicodePtr($apiToken)
)

if ([string]::IsNullOrWhiteSpace($apiTokenPlain)) {
    Write-Host '  ❌ Token cannot be empty'
    exit 1
}

Write-Host '  ✓ Token received (hidden)' 
Write-Host ''

# ============================================================================
# PART 3: GET CLOUDFLARE ACCOUNT ID
# ============================================================================

Write-Host '📍 PART 3: Cloudflare Account ID'
Write-Host ''
Write-Host '  1. Go to: https://dash.cloudflare.com/'
Write-Host '  2. Look at right sidebar under "Account"'
Write-Host '  3. Copy "Account ID"'
Write-Host ''

$accountId = Read-Host '  Enter Cloudflare Account ID'

if ([string]::IsNullOrWhiteSpace($accountId)) {
    Write-Host '  ❌ Account ID cannot be empty'
    exit 1
}

Write-Host '  ✓ Account ID received'
Write-Host ''

# ============================================================================
# PART 4: ADD SECRETS TO GITHUB
# ============================================================================

Write-Host '📤 PART 4: Adding secrets to GitHub...'
Write-Host ''

Write-Host '  Adding CLOUDFLARE_API_TOKEN...'
$apiTokenPlain | gh secret set CLOUDFLARE_API_TOKEN -R Rigohl/MEMORY_P
if ($LASTEXITCODE -ne 0) {
    Write-Host '  ❌ Failed to add API token'
    exit 1
}
Write-Host '  ✓ API token added'

Write-Host '  Adding CLOUDFLARE_ACCOUNT_ID...'
echo $accountId | gh secret set CLOUDFLARE_ACCOUNT_ID -R Rigohl/MEMORY_P
if ($LASTEXITCODE -ne 0) {
    Write-Host '  ❌ Failed to add Account ID'
    exit 1
}
Write-Host '  ✓ Account ID added'
Write-Host ''

# ============================================================================
# PART 5: VERIFY SECRETS IN GITHUB
# ============================================================================

Write-Host '✅ PART 5: Verifying secrets in GitHub...'
Write-Host ''

$secrets = gh secret list -R Rigohl/MEMORY_P
Write-Host $secrets
Write-Host ''

if ($secrets -match 'CLOUDFLARE_API_TOKEN' -and $secrets -match 'CLOUDFLARE_ACCOUNT_ID') {
    Write-Host '✓ Both secrets are in GitHub'
}
else {
    Write-Host '⚠️  Warning: Could not verify secrets'
}
Write-Host ''

# ============================================================================
# PART 6: TEST DEPLOYMENT
# ============================================================================

Write-Host '🧪 PART 6: Ready for automated deployments'
Write-Host ''

if ($AutoDeploy) {
    Write-Host '  Starting test deployment...'
    $env:CLOUDFLARE_API_TOKEN = $apiTokenPlain
    $env:CLOUDFLARE_ACCOUNT_ID = $accountId
    
    cd $PSScriptRoot
    npx wrangler deploy --env production --dry-run
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host '✓ Dry-run successful'
    }
    else {
        Write-Host '⚠️  Dry-run failed (check wrangler.toml)'
    }
}

Write-Host ''

# ============================================================================
# SUMMARY
# ============================================================================

Write-Host '╔══════════════════════════════════════════════════════════════╗'
Write-Host '║  ✅ SETUP COMPLETE                                          ║'
Write-Host '║                                                              ║'
Write-Host '║  ✅ GitHub secrets configured                               ║'
Write-Host '║  ✅ Cloudflare API ready                                    ║'
Write-Host '║  ✅ Auto-deployment enabled                                 ║'
Write-Host '║                                                              ║'
Write-Host '║  Next Steps:                                                ║'
Write-Host '║  1. Push code to GitHub (master branch)                     ║'
Write-Host '║  2. GitHub Actions will auto-deploy                         ║'
Write-Host '║  3. Check: https://github.com/Rigohl/MEMORY_P/actions      ║'
Write-Host '║                                                              ║'
Write-Host '║  Live at: https://memory-p-api.workers.dev                 ║'
Write-Host '╚══════════════════════════════════════════════════════════════╝'
Write-Host ''

Write-Host 'To view GitHub secrets:'
Write-Host '  https://github.com/Rigohl/MEMORY_P/settings/secrets/actions'
Write-Host ''

Write-Host 'To view deployments:'
Write-Host '  https://github.com/Rigohl/MEMORY_P/actions'
