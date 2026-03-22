#!/usr/bin/env pwsh
# INSTALL_RUST_AUTO.ps1 - Instala Rust automaticamente en Windows

Write-Host '🔧 MEMORY_P v2.0 - Rust Toolchain Installer' -ForegroundColor Cyan
Write-Host '============================================' -ForegroundColor Cyan
Write-Host ''

# Step 1: Verificar si Rust ya está instalado
Write-Host '[1/4] Verificando si Rust ya está instalado...' -ForegroundColor Yellow
try {
    $rustVersion = & rustc --version 2>&1
    Write-Host "✅ Rust ya instalado: $rustVersion" -ForegroundColor Green
    exit 0
}
catch {
    Write-Host 'ℹ️  Rust no detectado, procediendo con instalación...' -ForegroundColor Yellow
}

# Step 2: Descargar rustup
Write-Host '[2/4] Descargando rustup-init.exe...' -ForegroundColor Yellow
$rustupUrl = 'https://win.rustup.rs'
$rustupPath = "$env:TEMP\rustup-init.exe"

try {
    Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath -TimeoutSec 30
    Write-Host "✅ Descarga exitosa: $rustupPath" -ForegroundColor Green
}
catch {
    Write-Host "❌ Error descargando rustup: $_" -ForegroundColor Red
    exit 1
}

# Step 3: Ejecutar instalador
Write-Host '[3/4] Ejecutando instalador de Rust (default settings)...' -ForegroundColor Yellow
Write-Host '     (Esto puede tomar 5-10 minutos)' -ForegroundColor Cyan

try {
    # Instalar con configuración default (stable toolchain, MSVC ABI)
    & $rustupPath -y --default-toolchain stable --default-host x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) {
        throw "Instalador retornó código: $LASTEXITCODE"
    }
    Write-Host '✅ Instalación completada' -ForegroundColor Green
}
catch {
    Write-Host "❌ Error durante instalación: $_" -ForegroundColor Red
    exit 1
}

# Step 4: Recargar PATH y verificar
Write-Host '[4/4] Verificando instalación...' -ForegroundColor Yellow

# Recargar PATH
$env:Path = [System.Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path', 'User')

# Esperar a que rustup esté disponible
Start-Sleep -Seconds 2

try {
    $rustVersion = & rustc --version
    $cargoVersion = & cargo --version
    Write-Host "✅ Rust: $rustVersion" -ForegroundColor Green
    Write-Host "✅ Cargo: $cargoVersion" -ForegroundColor Green
    Write-Host '' -ForegroundColor Green
    Write-Host '🎉 ¡Rust instalado exitosamente!' -ForegroundColor Green
    Write-Host 'Próximo paso: cargo build --release --all-targets' -ForegroundColor Cyan
}
catch {
    Write-Host '⚠️  Rust instalado pero PowerShell necesita reiniciarse para ver cambios' -ForegroundColor Yellow
    Write-Host 'Por favor: Cierra y reabre PowerShell, luego verifica con: rustc --version' -ForegroundColor Cyan
}

Write-Host ''
