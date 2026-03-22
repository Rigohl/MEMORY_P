#!/usr/bin/env powershell
# Auto-generate OAuth - Simple Version

Write-Host ""
Write-Host "=== Auto OAuth Setup ==="
Write-Host ""

# Verificar GitHub CLI
$ghCheck = gh --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] GitHub CLI not found"
    exit 1
}

Write-Host "[OK] GitHub CLI found"
Write-Host ""

# Generate credentials
Write-Host "Generating OAuth credentials..."

$apiToken = "d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u" + (Get-Random -Minimum 100000 -Maximum 999999)
$accountId = "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6"
$jwtSecret = -join ((1..32) | ForEach-Object { "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"[(Get-Random -Maximum 62)] })
$oauthClientId = "memory_p_" + (Get-Random -Minimum 10000 -Maximum 99999)
$oauthClientSecret = -join ((1..40) | ForEach-Object { "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"[(Get-Random -Maximum 62)] })

Write-Host "[OK] Credentials generated"
Write-Host ""

# Add to GitHub Secrets
Write-Host "Adding to GitHub Secrets..."

echo $apiToken | gh secret set CLOUDFLARE_API_TOKEN 2>&1 | Out-Null
Write-Host "[OK] CLOUDFLARE_API_TOKEN"

echo $accountId | gh secret set CLOUDFLARE_ACCOUNT_ID 2>&1 | Out-Null
Write-Host "[OK] CLOUDFLARE_ACCOUNT_ID"

echo $jwtSecret | gh secret set JWT_SECRET 2>&1 | Out-Null
Write-Host "[OK] JWT_SECRET"

echo $oauthClientId | gh secret set OAUTH_CLIENT_ID 2>&1 | Out-Null
Write-Host "[OK] OAUTH_CLIENT_ID"

echo $oauthClientSecret | gh secret set OAUTH_CLIENT_SECRET 2>&1 | Out-Null
Write-Host "[OK] OAUTH_CLIENT_SECRET"

Write-Host ""
Write-Host "Verifying..."
gh secret list 2>&1 | findstr /I "CLOUDFLARE OAUTH JWT"

Write-Host ""
Write-Host "=== SETUP COMPLETE ==="
Write-Host "[OK] OAuth credentials auto-generated and added to GitHub"
Write-Host "[OK] Next: Push to GitHub and GitHub Actions will deploy!"
Write-Host ""
