function mem {
    param (
        [Parameter(Mandatory = $true, Position = 0)]
        [string]$Command,
        [Parameter(Position = 1)]
        [string]$Target = ".",
        [Parameter(Position = 2)]
        [string]$ExtraArg
    )

    $BaseUrl = "http://127.0.0.1:4040/mcp"

    switch ($Command) {
        "status" {
            Invoke-RestMethod -Uri $BaseUrl -Method Post -Body '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"ultra_overview","arguments":{"path":"."}}}' -ContentType "application/json" | Select-Object -ExpandProperty result | Select-Object -ExpandProperty content | ForEach-Object { Write-Host $_.text -ForegroundColor Cyan }
        }
        "analyze" {
            $path = Resolve-Path $Target
            Write-Host "🔬 Analyzin' $path with MAX JUICE..." -ForegroundColor Yellow
            $payload = @{
                jsonrpc = "2.0"
                id      = 2
                method  = "callTool"
                params  = @{
                    name      = "ultra_analyze"
                    arguments = @{
                        path        = "$path"
                        extension   = "rs"
                        max_threads = 16
                    }
                }
            }
            $json = $payload | ConvertTo-Json -Depth 5
            Invoke-RestMethod -Uri $BaseUrl -Method Post -Body $json -ContentType "application/json" | Select-Object -ExpandProperty result | Select-Object -ExpandProperty content | ForEach-Object { Write-Host $_.text -ForegroundColor Green }
        }
        "repair" {
            $path = Resolve-Path $Target
            Write-Host "🛠️ Repairing $path..." -ForegroundColor Red
            $payload = @{
                jsonrpc = "2.0"
                id      = 3
                method  = "callTool"
                params  = @{
                    name      = "ultra_repair"
                    arguments = @{
                        path      = "$path"
                        extension = "rs"
                    }
                }
            }
            $json = $payload | ConvertTo-Json -Depth 5
            Invoke-RestMethod -Uri $BaseUrl -Method Post -Body $json -ContentType "application/json" | Select-Object -ExpandProperty result | Select-Object -ExpandProperty content | ForEach-Object { Write-Host $_.text -ForegroundColor Green }
        }
        "search" {
            $path = Resolve-Path $Target
            $pattern = $ExtraArg
            if (-not $pattern) { Write-Error "Pattern required for search: mp search <path> <pattern>"; return }
            Write-Host "🔍 Searching '$pattern' in $path..." -ForegroundColor Cyan
            $payload = @{
                jsonrpc = "2.0"
                id      = 4
                method  = "callTool"
                params  = @{
                    name      = "ultra_search"
                    arguments = @{
                        path      = "$path"
                        pattern   = $pattern
                        extension = "rs"
                    }
                }
            }
            $json = $payload | ConvertTo-Json -Depth 5
            Invoke-RestMethod -Uri $BaseUrl -Method Post -Body $json -ContentType "application/json" | Select-Object -ExpandProperty result | Select-Object -ExpandProperty content | ForEach-Object { Write-Host $_.text -ForegroundColor Yellow }
        }
        "run" {
            $bankPath = Join-Path $PSScriptRoot "PAYLOAD_BANK"
            $file = Join-Path $bankPath $Target
            if (-not (Test-Path $file)) { $file = "$file.json" }
            
            if (Test-Path $file) {
                Write-Host "🚀 Launching from Bank: $(Split-Path $file -Leaf)" -ForegroundColor Green
                Invoke-RestMethod -Uri $BaseUrl -Method Post -InFile $file -ContentType "application/json" | Select-Object -ExpandProperty result | Select-Object -ExpandProperty content | ForEach-Object { Write-Host $_.text -ForegroundColor Cyan }
            }
            else {
                Write-Error "Payload '$Target' not found in $bankPath"
                Write-Host "Available payloads:" -ForegroundColor Gray
                Get-ChildItem $bankPath -Filter "*.json" | ForEach-Object { Write-Host " - $($_.BaseName)" }
            }
        }
        "edit" {
            $bankPath = Join-Path $PSScriptRoot "PAYLOAD_BANK"
            $file = Join-Path $bankPath $Target
            if (-not (Test-Path $file)) { $file = "$file.json" }
            
            if (Test-Path $file) {
                Write-Host "✏️ Launching MASSIVE EDIT from: $(Split-Path $file -Leaf)" -ForegroundColor Red
                Invoke-RestMethod -Uri $BaseUrl -Method Post -InFile $file -ContentType "application/json" | Select-Object -ExpandProperty result | Select-Object -ExpandProperty content | ForEach-Object { Write-Host $_.text -ForegroundColor Cyan }
            }
            else {
                Write-Error "Edit Payload '$Target' not found in $bankPath"
            }
        }
        "workflow" {
            $bankPath = Join-Path $PSScriptRoot "PAYLOAD_BANK"
            $file = Join-Path $bankPath $Target
            if (-not (Test-Path $file)) { $file = "$file.json" }
            
            if (Test-Path $file) {
                Write-Host "🌊 Launching SEQUENTIAL WORKFLOW from: $(Split-Path $file -Leaf)" -ForegroundColor Cyan
                Invoke-RestMethod -Uri $BaseUrl -Method Post -InFile $file -ContentType "application/json" | Select-Object -ExpandProperty result | Select-Object -ExpandProperty content | ForEach-Object { Write-Host $_.text -ForegroundColor Cyan }
            }
            else {
                Write-Error "Workflow Payload '$Target' not found in $bankPath"
            }
        }
        "simulation" {
            $bankPath = Join-Path $PSScriptRoot "PAYLOAD_BANK"
            $file = Join-Path $bankPath $Target
            if (-not (Test-Path $file)) { $file = "$file.json" }
            
            # Usamos el puerto 8079 (temp_mcp) EXCLUSIVAMENTE para simulaciones
            $SimUrl = "http://127.0.0.1:8079/mcp"

            if (Test-Path $file) {
                Write-Host "🌀 Launching BEND SIMULATION (via TEMP_MCP:8079) from: $(Split-Path $file -Leaf)" -ForegroundColor Cyan
                # Nota: temp_mcp usa nombre 'create_simulation' en el tool definition, 
                # El usuario tendría que asegurarse que el payload use ese nombre. 
                # O modificamos el payload on-the-fly si es necesario, pero mejor respetar el payload.
                Invoke-RestMethod -Uri $SimUrl -Method Post -InFile $file -ContentType "application/json" | Select-Object -ExpandProperty result | Select-Object -ExpandProperty content | ForEach-Object { Write-Host $_.text -ForegroundColor Cyan }
            }
            else {
                Write-Error "Simulation Payload '$Target' not found in $bankPath"
            }
        }
        "list" {
            $bankPath = Join-Path $PSScriptRoot "PAYLOAD_BANK"
            Write-Host "📚 PAYLOAD BANK CONTENTS:" -ForegroundColor Magenta
            Get-ChildItem $bankPath -Filter "*.json" | ForEach-Object { Write-Host " 📄 $($_.BaseName)" -ForegroundColor Gray }
        }
        "help" {
            Write-Host "🔥 MEMORY_P CLI (mem) - The Advanced Agentic Tool" -ForegroundColor Magenta
            Write-Host "  mem status           -> System Health"
            Write-Host "  mem analyze <path>   -> Ultra Analysis"
            Write-Host "  mem repair <path>    -> Auto Repair"
            Write-Host "  mem search <p> <pat> -> Regex Search"
            Write-Host "  mem run <name>       -> Execute Stored Payload"
            Write-Host "  mem edit <name>      -> Execute Massive Edit"
            Write-Host "  mem workflow <name>  -> Execute Sequential Pipeline"
            Write-Host "  mem simulation <name> -> Execute Bend GPU Simulation"
            Write-Host "  mem list             -> List Payload Bank"
        }
        default {
            Write-Error "Unknown command: $Command. Try 'mem help'"
        }
    }
}

Write-Host "✅ MEM CLI loaded! running 'mem help'..." -ForegroundColor Green
mem help
