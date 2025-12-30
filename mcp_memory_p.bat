@echo off
:loop
curl -X POST http://localhost:4040/mcp -H "Content-Type: application/json" -d @-
if %errorlevel% neq 0 (
    timeout /t 1 >nul
)
goto loop
