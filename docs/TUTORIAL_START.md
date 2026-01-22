# MEMORY_P: Tutorial de Inicio Rápido

Bienvenido a **MEMORY_P v2025.2.ULTRA**, el motor MCP de procesamiento masivo nativo diseñado para Cursor, Windsurf, Claude Desktop y VS Code.

## 1. Requisitos Previos
- **Rust Stable 2021+**: Instalado mediante `rustup`
- **Cargo**: El gestor de paquetes de Rust
- **MCP Host**: Cursor, Windsurf, Claude Desktop o VS Code

## 2. Compilación y Ejecución

### Build Release (Optimizado)
```bash
git clone https://github.com/Rigohl/MEMORY_P.git
cd MEMORY_P
cargo build --release
```

### Modo HTTP (Puerto 4040)
```bash
./target/release/memory_p
# O con cargo:
cargo run --release
```

### Modo stdio (Integración directa)
```bash
./target/release/memory_p --stdio
# O:
MCP_STDIO=1 cargo run --release
```

## 3. Configuración MCP

### Para Cursor/Windsurf (HTTP)
Añade a tu `mcp.json`:
```json
{
  "mcpServers": {
    "memory_p": {
      "url": "http://127.0.0.1:4040/mcp",
      "transport": "http"
    }
  }
}
```

### Para Claude Desktop (stdio)
Añade a `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "memory_p": {
      "command": "/path/to/MEMORY_P/target/release/memory_p",
      "args": ["--stdio"]
    }
  }
}
```

## 4. Las 5 Herramientas MCP

### 🔬 `analyze` - Análisis de Código
Tres modos de análisis paralelo:

**Quick Overview:**
```
Usa analyze en ./src con mode=overview
```

**Análisis Profundo:**
```
Usa analyze en ./src con mode=deep para obtener métricas completas y security score
```

### 🛠️ `repair` - Reparación Automática
Limpia y optimiza tu código:
```
Usa repair en ./src para aplicar auto-fix de imports, espacios y formato
```

### ✏️ `edit` - Edición Masiva
Reemplaza texto en múltiples archivos:
```
Usa edit mode=replace para cambiar "old_function" por "new_function" en ./src
```

### 🌊 `workflow` - Pipelines
Ejecuta secuencias automatizadas:
```
Crea un workflow que haga Scan → Analyze → Repair en ./src
```

### 🌀 `simulate` - Optimización
Simula configuraciones óptimas:
```
Usa simulate phase=2 con 5000 iterations para optimizar paralelismo
```

## 5. Verificación

Comprueba que el servidor esté activo:
```bash
curl http://localhost:4040/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"initialize","id":1}'
```

## 📊 Benchmarks

| Fase | Simulaciones | Mejora |
|------|--------------|--------|
| Phase 1 (Módulos) | 65K | 89.8% |
| Phase 2 (Threads) | 200K | 1345.6% |
| Phase 3 (Ecosystem) | 550K | Óptimo |

---
*Siguientes pasos: Consulta [REFERENCE_TOOLS.md](./REFERENCE_TOOLS.md) para especificaciones detalladas de cada herramienta.*
