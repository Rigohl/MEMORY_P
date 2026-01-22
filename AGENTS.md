# Agent Instructions: MEMORY_P Optimization

## Overview
Este archivo define el comportamiento de agentes IA (Cascade/Copilot/Claude) para el proyecto **MEMORY_P v2025.2.ULTRA**, optimizando el uso de créditos y la eficiencia operativa siguiendo las mejores prácticas de desarrollo.

## Core Directives
- **Efficiency First**: Minimizar llamadas a herramientas costosas y reducir preguntas al usuario. Actúa con autonomía cuando el contexto es suficiente.
- **Zero Technical Debt**: Prohibido dejar dead code, warnings o errores. Cada cambio debe ser limpio y seguir las mejores prácticas.
- **Rule Enforcement**: Es obligatorio consultar y seguir las reglas definidas en `.windsurf/rules/` antes de cada acción.
- **Language**: Todas las respuestas y explicaciones deben ser en **Español**.
- **Style**: Respuestas concisas, directas y altamente técnicas.

## Project Architecture

### Core Modules (src/)
- `main.rs` - Entry point con modos HTTP y stdio
- `mcp_api.rs` - 5 herramientas MCP: analyze, repair, edit, workflow, simulate
- `parallel_engine.rs` - Motor Rayon para procesamiento paralelo
- `mega_simulator.rs` - Simulaciones 3-phase (65K/200K/550K)
- `analyzer.rs` - Análisis de código con métricas y security score
- `workspace.rs` - Gestión de workspaces y archivos
- `config.rs` - Configuración de paralelismo y caching

### MCP Toolkit (Las 5 Herramientas)
1. **analyze** - Análisis paralelo masivo (deep/quick/overview)
2. **repair** - Auto-fix de imports, formato y espacios
3. **edit** - Edición atómica masiva (replace/regex/append/delete)
4. **workflow** - Pipeline de pasos con auto-evolución
5. **simulate** - Mega simulaciones para optimización

### Tech Stack (Versiones Actuales)
- **Parallelism**: rayon 1.11, dashmap 6.1, scc 2.4
- **Memory**: mimalloc 0.1.48, memmap2 0.9.9
- **Serialization**: rkyv 0.8.14 (zero-copy), serde_json 1.0
- **MCP**: mcp-sdk-rs 0.3, mcpkit-core 0.5
- **HTTP**: axum 0.7, tokio 1.49
- **File I/O**: ignore 0.4 (ripgrep engine), jwalk 0.8

## Autonomy & Analysis
- **Analyze before acting**: Realiza escaneo profundo de dependencias y lógica antes de editar.
- **Fix on the fly**: Corrige errores detectados proactivamente.
- **Ask Less**: Solo pide aclaración si hay ambigüedad crítica en los requisitos.
- **No dead code**: Este proyecto mantiene ZERO dead code policy.

## Context Awareness
- El proyecto utiliza estructura de reglas en `.windsurf/rules/` y flujos en `.windsurf/workflows/`
- Priorizar uso de herramientas MCP para operaciones de código
- No hay dependencias externas como Julia o Bend - todo es Rust puro
- El servidor MCP soporta HTTP (puerto 4040) y stdio

## Development Guidelines

### Compilation Standards
- **Zero warnings**: `cargo check` debe retornar 0 warnings
- **Zero errors**: `cargo build --release` debe compilar exitosamente
- **Tests**: Mantener cobertura de tests si existen

### Code Quality
- Usar `#[allow(dead_code)]` solo para structs de configuración no usados aún
- Prefijar variables no usadas con `_` (ej: `_path`)
- Preferir zero-copy cuando sea posible (mmap, rkyv)
- Usar Rayon para operaciones paralelas en colecciones grandes

### Performance
- **Max Juice Philosophy**: Extraer máximo rendimiento de herramientas disponibles
- **Zero-Copy Mindset**: Evitar trabajo redundante
- **Fail Fast**: Diagnosticar inmediatamente usando logs

## Automation & Safety
- **Turbo Mode**: Solo para comandos seguros definidos en workflows
- **Safety**: Nunca ejecutar comandos destructivos sin confirmación
- **Validation**: Siempre validar cambios con `cargo check` antes de commit

## MCP Server Operations

### Starting Server
```bash
# HTTP mode (default)
cargo run --release

# stdio mode
cargo run --release -- --stdio
# Or:
MCP_STDIO=1 cargo run --release
```

### Testing MCP Endpoint
```bash
curl http://localhost:4040/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"initialize","id":1}'
```

## Known Constraints
- SCC 3.x tiene breaking changes en API - mantenerse en 2.4
- rkyv 0.8 eliminó feature "validation" - usar sin features
- El proyecto NO usa walkdir (deprecated) - usar crate `ignore` de ripgrep

---
*Generado para MEMORY_P v2025.2.ULTRA - MCP Toolkit Paralelo*
