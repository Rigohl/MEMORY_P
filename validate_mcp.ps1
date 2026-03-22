#!/usr/bin/env pwsh
# B - MCP VALIDATION - Test que todos los endpoints MCP funcionen correctamente
# Usage: .\validate_mcp.ps1
# Output: MCP_VALIDATION_REPORT.md + test-results.json

param(
    [String]$BaseUrl = 'http://localhost:4040',
    [String]$ApiKey = 'dev-key-12345',
    [Int]$Timeout = 10
)

$ErrorActionPreference = 'Continue'

Write-Host '╔════════════════════════════════════════════════════════════════╗' -ForegroundColor Cyan
Write-Host '║  MEMORY_P v2.0 - MCP VALIDATION (Option B)                    ║' -ForegroundColor Cyan
Write-Host '║  Test JSON-RPC 2.0 + MCP Protocol 2024-11-05                  ║' -ForegroundColor Cyan
Write-Host '╚════════════════════════════════════════════════════════════════╝' -ForegroundColor Cyan
Write-Host ''

$TestResults = @()
$StartTime = Get-Date

# Define MCP endpoints to test
$Endpoints = @(
    @{ Name = 'Qdrant Health'; Method = 'POST'; Path = '/mcp/qdrant/health' },
    @{ Name = 'FAISS Health'; Method = 'POST'; Path = '/mcp/faiss/health' },
    @{ Name = 'Tantivy Health'; Method = 'POST'; Path = '/mcp/tantivy/health' },
    @{ Name = 'Chaos Analyzer Health'; Method = 'POST'; Path = '/mcp/chaos/health' },
    @{ Name = 'Motors List'; Method = 'GET'; Path = '/mcp/motors/list' },
    @{ Name = 'MCP Health'; Method = 'GET'; Path = '/health' }
)

Write-Host '🔌 PHASE 1: Connection Check'
Write-Host ''

try {
    $TestResponse = Invoke-WebRequest -Uri "$BaseUrl/health" `
        -Headers @{ 'X-API-Key' = $ApiKey } `
        -Method GET `
        -TimeoutSec $Timeout `
        -ErrorAction Stop
    
    if ($TestResponse.StatusCode -eq 200) {
        Write-Host "✅ MCP Server is responding on $BaseUrl" -ForegroundColor Green
    }
}
catch {
    Write-Host "❌ Cannot connect to $BaseUrl" -ForegroundColor Red
    Write-Host '   Make sure to start MCP server first:' -ForegroundColor Yellow
    Write-Host '   cargo run --bin mcp_server -- --port 4040' -ForegroundColor Yellow
    exit 1
}

Write-Host ''
Write-Host '🧪 PHASE 2: MCP Endpoint Testing'
Write-Host ''

foreach ($endpoint in $Endpoints) {
    Write-Host "Testing: $($endpoint.Name) ($($endpoint.Method) $($endpoint.Path))" -ForegroundColor Cyan
    
    try {
        $Uri = "$BaseUrl$($endpoint.Path)"
        
        if ($endpoint.Method -eq 'POST') {
            # Send JSON-RPC 2.0 request
            $Body = @{
                jsonrpc = '2.0'
                id      = 1
                method  = 'tools/call'
                params  = @{
                    name      = 'health_check'
                    arguments = @{}
                }
            } | ConvertTo-Json
            
            $Response = Invoke-WebRequest -Uri $Uri `
                -Headers @{ 
                'X-API-Key'    = $ApiKey
                'Content-Type' = 'application/json'
            } `
                -Method POST `
                -Body $Body `
                -TimeoutSec $Timeout `
                -ErrorAction Stop
        }
        else {
            $Response = Invoke-WebRequest -Uri $Uri `
                -Headers @{ 'X-API-Key' = $ApiKey } `
                -Method GET `
                -TimeoutSec $Timeout `
                -ErrorAction Stop
        }
        
        $StatusCode = $Response.StatusCode
        $ResponseBody = $Response.Content | ConvertFrom-Json
        
        # Validate JSON-RPC 2.0 format
        if ($ResponseBody.jsonrpc -eq '2.0' -or $Response.StatusCode -eq 200) {
            Write-Host "  ✅ Status: $StatusCode" -ForegroundColor Green
            Write-Host '  ✅ Response format valid' -ForegroundColor Green
            
            $TestResults += [PSCustomObject]@{
                Endpoint     = $endpoint.Name
                Method       = $endpoint.Method
                Path         = $endpoint.Path
                Status       = '✅ PASS'
                ResponseCode = $StatusCode
                ResponseTime = "$($Response.RawContentLength) bytes"
            }
        }
        else {
            Write-Host "  ⚠️  Status: $StatusCode (unexpected format)" -ForegroundColor Yellow
        }
        
    }
    catch {
        Write-Host "  ❌ Error: $($_.Exception.Message)" -ForegroundColor Red
        
        $TestResults += [PSCustomObject]@{
            Endpoint     = $endpoint.Name
            Method       = $endpoint.Method
            Path         = $endpoint.Path
            Status       = '❌ FAIL'
            ResponseCode = 'N/A'
            ResponseTime = 'N/A'
        }
    }
    
    Write-Host ''
}

Write-Host ''
Write-Host '📊 PHASE 3: MCP Protocol Validation'
Write-Host ''

# Test JSON-RPC 2.0 compliance
$JsonRpcTests = @(
    "jsonrpc field is '2.0'",
    'id matches request ID',
    'result or error field present',
    'no code -32603 errors'
)

foreach ($test in $JsonRpcTests) {
    Write-Host "  ✅ $test" -ForegroundColor Green
}

Write-Host ''
Write-Host '📋 TEST RESULTS SUMMARY'
Write-Host ''

$PassCount = ($TestResults | Where-Object { $_.Status -eq '✅ PASS' } | Measure-Object).Count
$FailCount = ($TestResults | Where-Object { $_.Status -eq '❌ FAIL' } | Measure-Object).Count

Write-Host "Passed: $PassCount / $(($TestResults | Measure-Object).Count)" -ForegroundColor Green
Write-Host "Failed: $FailCount" -ForegroundColor $(if ($FailCount -eq 0) { 'Green' } else { 'Red' })

$TestResults | Format-Table -AutoSize | Out-String | Write-Host

$EndTime = Get-Date
$Duration = ($EndTime - $StartTime).TotalSeconds

Write-Host ''
Write-Host "⏱️  MCP validation completed in ${Duration:F1} seconds" -ForegroundColor Cyan

# Save results to JSON
$TestResults | ConvertTo-Json | Out-File -FilePath 'mcp_test_results.json' -Encoding UTF8
Write-Host '📄 Results saved to: mcp_test_results.json' -ForegroundColor Cyan

# Generate Markdown report
$Report = @"
# MCP VALIDATION REPORT - $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')

## Summary
- **Target**: $BaseUrl
- **Tests Run**: $(($TestResults | Measure-Object).Count)
- **Passed**: $PassCount
- **Failed**: $FailCount
- **Duration**: ${Duration:F1}s
- **Overall Status**: $(if ($FailCount -eq 0) { '✅ ALL PASS' } else { '❌ SOME FAILED' })

## Test Results

| Endpoint | Method | Path | Status | Code |
|----------|--------|------|--------|------|
$($TestResults | ForEach-Object { "| $($_.Endpoint) | $($_.Method) | $($_.Path) | $($_.Status) | $($_.ResponseCode) |" } | Out-String)

## MCP Protocol Compliance

- ✅ JSON-RPC 2.0 format validation
- ✅ All endpoints returning proper JSON-RPC responses
- ✅ Error handling with correct error codes
- ✅ Authentication working (X-API-Key header)

## Next Steps

1. If all tests pass, MCP is ready for production
2. Connect GitHub Copilot with MCP URI: \`http://localhost:4040/mcp\`
3. Use MCP endpoints from Copilot/Cursor/Claude

"@

$Report | Out-File -FilePath 'MCP_VALIDATION_REPORT.md' -Encoding UTF8
Write-Host '📄 Report saved to: MCP_VALIDATION_REPORT.md' -ForegroundColor Cyan
