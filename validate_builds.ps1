#!/usr/bin/env pwsh
# A - BUILD VALIDATION - Test que todos los 18 binarios compilen sin errores
# Usage: .\validate_builds.ps1
# Output: BUILD_VALIDATION_REPORT.md

param(
    [Switch]$Fast = $false,
    [Switch]$Quiet = $false
)

$ErrorActionPreference = 'Continue'
$WarningPreference = 'SilentlyContinue'

Write-Host '╔════════════════════════════════════════════════════════════════╗' -ForegroundColor Cyan
Write-Host '║  MEMORY_P v2.0 - BUILD VALIDATION (Option A)                  ║' -ForegroundColor Cyan
Write-Host '║  Validar 18 binarios + motores + FFI                          ║' -ForegroundColor Cyan
Write-Host '╚════════════════════════════════════════════════════════════════╝' -ForegroundColor Cyan
Write-Host ''

$StartTime = Get-Date
$Results = @()

# Lista de 18 binarios a validar
$Binaries = @(
    'memory_p',
    'vector_engine',
    'text_engine',
    'specialized_engine',
    'qdrant_search_engine',
    'faiss_search_engine',
    'scann_search_engine',
    'tantivy_engine',
    'lnx_cluster_engine',
    'meilisearch_search_engine',
    'julia_optimization_engine',
    'jax_ml_engine',
    'mojo_search_engine',
    'pony_actor_engine',
    'chaos_analyzer',
    'memorybank_orchestrator',
    'motor_orchestrator',
    'mcp_server',
    'jar'
)

Write-Host '🔨 PHASE 1: Validar que Cargo.toml declara todos los binarios'
Write-Host ''

$CargoContent = Get-Content (Join-Path (Get-Location) 'Cargo.toml') -Raw
foreach ($binary in $Binaries) {
    if ($CargoContent -match "name = `"$binary`"") {
        Write-Host "  ✅ $binary" -ForegroundColor Green
        $Results += [PSCustomObject]@{
            Binary   = $binary
            Declared = '✅'
            Compiled = '⏳'
            Status   = 'Pending'
        }
    }
    else {
        Write-Host "  ❌ $binary - NOT FOUND in Cargo.toml" -ForegroundColor Red
        $Results += [PSCustomObject]@{
            Binary   = $binary
            Declared = '❌'
            Compiled = '❌'
            Status   = 'Missing'
        }
    }
}

Write-Host ''
Write-Host '🔨 PHASE 2: Build all targets'
Write-Host ''

if ($Fast) {
    Write-Host '⚡ Fast mode: Checking compilation cache'
    $CompileCmd = 'cargo build --release --all-targets --offline 2>&1'
}
else {
    Write-Host '🔧 Full build: Compiling with all optimizations'
    Write-Host '   (This may take 3-5 minutes on first run)'
    Write-Host ''
    $CompileCmd = 'cargo build --release --all-targets 2>&1'
}

try {
    $CompileOutput = Invoke-Expression $CompileCmd
    $CompileSuccess = $LASTEXITCODE -eq 0
    
    if ($CompileSuccess) {
        Write-Host '✅ ALL BINARIES COMPILED SUCCESSFULLY' -ForegroundColor Green
        Write-Host ''
        
        # Verificar que existen los binarios compilados
        $ReleaseDir = 'target/release'
        if (Test-Path $ReleaseDir) {
            $Binaries | ForEach-Object {
                $ExePath = Join-Path $ReleaseDir "$_.exe"
                if (Test-Path $ExePath) {
                    $Size = (Get-Item $ExePath).Length / 1MB
                    Write-Host "  📦 $_ - ${Size:F2} MB" -ForegroundColor Green
                    ($Results | Where-Object { $_.Binary -eq $_ })[0].Compiled = '✅'
                    ($Results | Where-Object { $_.Binary -eq $_ })[0].Status = 'Compiled'
                }
                else {
                    Write-Host "  ⚠️  $_ - Binary not found in target/release" -ForegroundColor Yellow
                }
            }
        }
    }
    else {
        Write-Host '❌ BUILD FAILED' -ForegroundColor Red
        Write-Host ''
        Write-Host 'Error output:' -ForegroundColor Red
        Write-Host $CompileOutput
    }
}
catch {
    Write-Host "❌ Compilation error: $_" -ForegroundColor Red
}

Write-Host ''
Write-Host '🔨 PHASE 3: Verify no compilation warnings turned into errors'
Write-Host ''

if ($CompileOutput -match 'error\[E') {
    Write-Host '❌ Found compilation errors' -ForegroundColor Red
    $CompileOutput | Select-String 'error\[E'
}
else {
    Write-Host '✅ No critical errors found' -ForegroundColor Green
}

Write-Host ''
Write-Host '📊 SUMMARY'
Write-Host ''
$Results | Format-Table -AutoSize | Out-String | Write-Host

$EndTime = Get-Date
$Duration = ($EndTime - $StartTime).TotalSeconds

Write-Host ''
Write-Host "⏱️  Build validation completed in ${Duration:F1} seconds" -ForegroundColor Cyan

# Generar reporte Markdown
$Report = @"
# BUILD VALIDATION REPORT - $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')

## Summary
- **Total Binaries**: $(($Results | Measure-Object).Count)
- **Declared**: $(($Results | Where-Object { $_.Declared -eq '✅' } | Measure-Object).Count)
- **Compiled**: $(($Results | Where-Object { $_.Compiled -eq '✅' } | Measure-Object).Count)
- **Duration**: ${Duration:F1}s
- **Status**: $(if ($CompileSuccess) { '✅ PASS' } else { '❌ FAIL' })

## Binaries

| Binary | Declared | Compiled | Status |
|--------|----------|----------|--------|
$($Results | ForEach-Object { "| $($_.Binary) | $($_.Declared) | $($_.Compiled) | $($_.Status) |" } | Out-String)

## Commands for Individual Testing

\`\`\`bash
# Test individual binario
cargo build --release --bin memory_p
cargo build --release --bin mcp_server
cargo build --release --bin qdrant_search_engine

# Run specific binary
./target/release/mcp_server --port 4040
./target/release/qdrant_search_engine
\`\`\`

"@

$Report | Out-File -FilePath 'BUILD_VALIDATION_REPORT.md' -Encoding UTF8
Write-Host ''
Write-Host '📄 Report saved to: BUILD_VALIDATION_REPORT.md' -ForegroundColor Cyan
