#!/usr/bin/env pwsh
<#
.SYNOPSIS
    MEMORY_P Integration Bridge
    Conecta TRAe Bot v2.0 con MEMORY_P para procesamiento automático de mensajes

.DESCRIPTION
    Este script configura la integración entre:
    - TRAe Bot (procesamiento IA)
    - MEMORY_P (servidor MCP)
    - Sistema de interceptación

.PARAMETER Action
    setup    - Configurar integración
    status   - Ver estado de conexión
    test     - Probar integración
    logs     - Ver logs del puente
#>

param(
    [ValidateSet("setup", "status", "test", "logs")]
    [string]$Action = "status"
)

$TRAeDir = "C:\Users\DELL\Desktop\PROYECTOS\MEMORY_P\BOT\TRAe"
$MemoryPDir = "C:\Users\DELL\Desktop\PROYECTOS\MEMORY_P"
$BridgeLog = Join-Path $MemoryPDir "trae_integration.log"

function Write-BridgeLog {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Add-Content -Path $BridgeLog -Value "[$timestamp] [$Level] $Message"
}

function Setup-Integration {
    Write-Host "🔗 Configurando integración TRAe ↔ MEMORY_P..." -ForegroundColor Cyan

    # Verificar archivos
    $checks = @(
        @{ Path = Join-Path $TRAeDir "bot_v2.ps1"; Name = "Bot v2.0" }
        @{ Path = Join-Path $TRAeDir "interceptor.ps1"; Name = "Interceptor" }
        @{ Path = Join-Path $MemoryPDir "src/main.rs"; Name = "MEMORY_P Server" }
    )

    $allGood = $true
    foreach ($check in $checks) {
        if (Test-Path $check.Path) {
            Write-Host "  ✅ $($check.Name)" -ForegroundColor Green
        } else {
            Write-Host "  ❌ $($check.Name)" -ForegroundColor Red
            $allGood = $false
        }
    }

    if ($allGood) {
        Write-BridgeLog "Integration setup completed successfully"
        Write-Host @"

╔════════════════════════════════════════════════════════════════╗
║              ✅ INTEGRACIÓN CONFIGURADA                       ║
╠════════════════════════════════════════════════════════════════╣
║                                                                ║
║  TRAe Bot:  $TRAeDir
║  MEMORY_P:  $MemoryPDir
║                                                                ║
║  Próximos pasos:                                              ║
║  1. Iniciar MEMORY_P: cargo run (en $MemoryPDir)            ║
║  2. Iniciar TRAe:     & '.\interceptor.ps1' start           ║
║  3. Usar bot:         & '.\bot_v2.ps1' -interactive          ║
║                                                                ║
╚════════════════════════════════════════════════════════════════╝
"@
    }
}

function Show-Status {
    Write-Host @"
╔════════════════════════════════════════════════════════════════╗
║         TRAe ↔ MEMORY_P Integration Status                    ║
╠════════════════════════════════════════════════════════════════╣
║                                                                ║
"@

    # TRAe Status
    $botExists = Test-Path (Join-Path $TRAeDir "bot_v2.ps1")
    $botStatus = if ($botExists) { "✅ Ready" } else { "❌ Missing" }
    Write-Host "║  TRAe Bot v2.0:        $botStatus"

    $interceptorExists = Test-Path (Join-Path $TRAeDir "interceptor.ps1")
    $interceptorStatus = if ($interceptorExists) { "✅ Ready" } else { "❌ Missing" }
    Write-Host "║  Interceptor:          $interceptorStatus"

    # MEMORY_P Status
    $memoryPExists = Test-Path (Join-Path $MemoryPDir "src/main.rs")
    $memoryPStatus = if ($memoryPExists) { "✅ Ready" } else { "❌ Missing" }
    Write-Host "║  MEMORY_P Server:      $memoryPStatus"

    # Cache
    $cacheSize = 0
    $cachePath = Join-Path $TRAeDir "cache.json"
    if (Test-Path $cachePath) {
        $cacheSize = (Get-Item $cachePath).Length / 1KB
    }
    Write-Host "║  Cache Size:           $($cacheSize)KB"

    # Logs
    $logSize = 0
    $logPath = Join-Path $TRAeDir "bot-activity.log"
    if (Test-Path $logPath) {
        $logSize = (Get-Item $logPath).Length / 1KB
    }
    Write-Host "║  Activity Logs:        $($logSize)KB"

    Write-Host @"
║                                                                ║
╚════════════════════════════════════════════════════════════════╝
"@
}

function Test-Integration {
    Write-Host "🧪 Probando integración..." -ForegroundColor Cyan

    # Test 1: Bot script existe
    if (Test-Path (Join-Path $TRAeDir "bot_v2.ps1")) {
        Write-Host "  ✅ bot_v2.ps1 encontrado"
    } else {
        Write-Host "  ❌ bot_v2.ps1 no encontrado"
        return
    }

    # Test 2: API Key configurada
    if ($env:OPENAI_API_KEY) {
        Write-Host "  ✅ OPENAI_API_KEY configurada"
    } else {
        Write-Host "  ⚠️  OPENAI_API_KEY no configurada (requerida para usar bot)"
    }

    # Test 3: Interceptor existe
    if (Test-Path (Join-Path $TRAeDir "interceptor.ps1")) {
        Write-Host "  ✅ interceptor.ps1 encontrado"
    } else {
        Write-Host "  ⚠️  interceptor.ps1 no encontrado"
    }

    # Test 4: Cache writable
    $cacheTest = Join-Path $TRAeDir "cache.json"
    try {
        @{} | ConvertTo-Json | Out-File $cacheTest -Force
        Write-Host "  ✅ Cache writable"
    } catch {
        Write-Host "  ❌ Cache no writable"
    }

    Write-Host @"

✓ Tests completados
  Para usar: & '$(Join-Path $TRAeDir 'bot_v2.ps1')' -interactive
"@
}

function Show-Logs {
    if (-not (Test-Path $BridgeLog)) {
        Write-Host "📋 No hay logs aún" -ForegroundColor Yellow
        return
    }

    Write-Host "📋 Últimos eventos del puente:" -ForegroundColor Yellow
    Get-Content $BridgeLog -Tail 20
}

# Main
switch ($Action) {
    "setup" { Setup-Integration }
    "status" { Show-Status }
    "test" { Test-Integration }
    "logs" { Show-Logs }
}
