#!/usr/bin/env pwsh
# build_julia_binaries.ps1 - Compile julia_optimization_engine and chaos_analyzer

Set-Location d:\REPOSITORIOS\memory_p_fresh

Write-Host '╔════════════════════════════════════════════════════════════╗' -ForegroundColor Cyan
Write-Host '║ MEMORY_P v2.0 - Julia Binaries Build                      ║' -ForegroundColor Cyan
Write-Host '╚════════════════════════════════════════════════════════════╝' -ForegroundColor Cyan

Write-Host ''
Write-Host '[1/3] Compiling library...' -ForegroundColor Yellow
cargo build --lib --release --quiet 

Write-Host '[2/3] Compiling julia_optimization_engine...' -ForegroundColor Yellow
cargo build --release --bin julia_optimization_engine --quiet

Write-Host '[3/3] Compiling chaos_analyzer...' -ForegroundColor Yellow
cargo build --release --bin chaos_analyzer --quiet

Write-Host ''
Write-Host '╔════════════════════════════════════════════════════════════╗' -ForegroundColor Cyan
Write-Host '║ BUILD VERIFICATION                                        ║' -ForegroundColor Cyan
Write-Host '╚════════════════════════════════════════════════════════════╝' -ForegroundColor Cyan

$lib_exists = Test-Path '.\target\release\memory_p.rlib'
$julia_exe = Test-Path '.\target\release\julia_optimization_engine.exe'
$chaos_exe = Test-Path '.\target\release\chaos_analyzer.exe'

Write-Host ''
Write-Host 'Artifacts:'
$lib_check = if ($lib_exists) { 'OK' } else { 'MISSING' }
$julia_check = if ($julia_exe) { 'OK' } else { 'MISSING' }
$chaos_check = if ($chaos_exe) { 'OK' } else { 'MISSING' }

Write-Host "  memory_p.rlib: $lib_check"
Write-Host "  julia_optimization_engine.exe: $julia_check"
Write-Host "  chaos_analyzer.exe: $chaos_check"

if ($julia_exe -and $chaos_exe) {
    Write-Host ''
    Write-Host 'Both specialized binaries compiled successfully!'
    
    $julia_size = (Get-Item '.\target\release\julia_optimization_engine.exe').Length
    $chaos_size = (Get-Item '.\target\release\chaos_analyzer.exe').Length
    
    Write-Host ''
    Write-Host 'Binary sizes:'
    Write-Host "  julia_optimization_engine.exe: $([Math]::Round($julia_size/1MB, 2)) MB"
    Write-Host "  chaos_analyzer.exe: $([Math]::Round($chaos_size/1MB, 2)) MB"
}
else {
    Write-Host ''
    Write-Host 'Build failed - binaries missing'
    exit 1
}
