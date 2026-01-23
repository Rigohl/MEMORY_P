# MEMORY_P v2.0: Tutorial de Inicio Rápido

Bienvenido a **MEMORY_P v2.0**, el servidor MCP always-on con cerebro matemático multi-lenguaje y motor de búsqueda híbrido. Integrado con Cursor, Windsurf, Claude Desktop y VS Code vía protocolo MCP 2024-11-05.

En este tutorial, aprenderás a:
- ✅ Configurar tu entorno (Rust core + stack opcional)
- ✅ Conectar MEMORY_P con tu IDE
- ✅ Realizar tu primera búsqueda híbrida
- ✅ Usar las herramientas MCP disponibles
- ✅ Explorar capacidades multi-lenguaje (opcional)

## 📋 Requisitos Previos

### Instalación Mínima (Solo Rust Core)
- **Rust Stable 1.75+**: Instalado mediante `rustup`
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Cargo**: El gestor de paquetes de Rust (incluido con rustup)
- **MCP Client**: Cursor, Windsurf o Claude Desktop configurados

### Instalación Completa (Stack Multi-Lenguaje)
Para aprovechar todas las capacidades de v2.0:
- **Julia 1.10+** (análisis matemático y caos)
- **Python 3.11+ con JAX** (ML inference)
- **Mojo** (kernels SIMD, opcional)
- **Zig 0.12+** (FFI bridge, opcional)
- **PostgreSQL 16** + pgvector
- **Qdrant** (vector search)
- **Redis** (caching)

**📖 Ver [INSTALL.md](../INSTALL.md) para guía completa de instalación**

### Verificar Instalación Mínima
```bash
rustc --version  # Debe mostrar >= 1.75
cargo --version
```

## 🚀 Instalación y Ejecución

### Opción 1: Quick Start (Solo Rust)

```bash
# 1. Clonar el repositorio
git clone https://github.com/Rigohl/MEMORY_P.git
cd MEMORY_P

# 2. Compilar en modo release (solo core Rust)
cargo build --release --no-default-features --features "core"

# 3. Ejecutar el servidor
./target/release/memory_p
# Servidor escuchando en http://127.0.0.1:4040
```

Este modo proporciona:
- ✅ Análisis de código con Rayon
- ✅ Reparación inteligente
- ✅ Edición masiva
- ✅ Workflows básicos

### Opción 2: Full Stack (Multi-Lenguaje)

```bash
# 1. Instalar dependencias (ver INSTALL.md para detalles)
# - Julia, Python+JAX, Qdrant, PostgreSQL, Redis

# 2. Compilar con todas las features
cargo build --release --all-features

# 3. Inicializar base de datos
./scripts/init_database.sh

# 4. Ejecutar servicios externos
docker-compose up -d  # Qdrant, Redis, ClickHouse

# 5. Ejecutar servidor
./target/release/memory_p
```

Este modo añade:
- 🧮 **Julia**: Análisis matemático y optimización
- 🤖 **JAX**: ML inference y embeddings
- 🔍 **Qdrant**: Vector search semántico
- 📊 **ClickHouse**: Analytics avanzado
- ⚡ **Hybrid Search**: Fusión de todos los motores

### 3. Modos de Ejecución

#### Modo HTTP (para Cursor/Windsurf)
```bash
./target/release/memory_p
# Servidor escuchando en http://127.0.0.1:4040
# WebSocket en ws://127.0.0.1:4040/mcp/stream
```

#### Modo stdio (para Claude Desktop)
```bash
cargo run --release -- --stdio
```

#### Modo Debug (con logs detallados)
```bash
RUST_LOG=debug ./target/release/memory_p
```

## ⚙️ Configuración del Cliente MCP

### Para Cursor / Windsurf

1. Abre la configuración de MCP en tu IDE
2. Añade la siguiente configuración:

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

3. Guarda y reinicia el IDE

### Para Claude Desktop

1. Edita `~/.config/claude/claude_desktop_config.json` (Linux/Mac) o `%APPDATA%\Claude\claude_desktop_config.json` (Windows)
2. Añade:

```json
{
  "mcpServers": {
    "memory_p": {
      "command": "cargo",
      "args": ["run", "--release", "--", "--stdio"],
      "cwd": "/ruta/completa/a/MEMORY_P"
    }
  }
}
```

3. Reinicia Claude Desktop

## 🔬 Tu Primer Análisis: `analyze`

Una vez conectado, puedes usar la herramienta `analyze` para obtener métricas profundas de tu código.

### Desde el Chat del IDE

En Cursor o Windsurf, escribe:
```
Analiza los archivos .rs en ./src usando la herramienta analyze con deep=true
```

### Respuesta HTTP Directa

```bash
curl -X POST http://localhost:4040/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "analyze",
      "arguments": {
        "path": "./src",
        "pattern": "**/*.rs",
        "deep": true
      }
    }
  }'
```

### Resultado Esperado (Rust Core)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "files_analyzed": 42,
    "total_lines": 12583,
    "functions": 328,
    "unsafe_count": 0,
    "unwrap_count": 5,
    "complexity_avg": 4.2,
    "parallel_opportunities": 12
  }
}
```

### Resultado Avanzado (Con Julia)

Si Julia está habilitado, el análisis incluye métricas de caos:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "basic": { /* ... */ },
    "chaos_metrics": {
      "lyapunov_exponent": 0.23,
      "complexity_classification": "semi-chaotic",
      "stability_score": 0.78,
      "refactoring_priority": "medium"
    }
  }
}
```

## 🔍 Búsqueda Híbrida (v2.0)

Una de las características principales de v2.0 es la búsqueda híbrida.

### Búsqueda Semántica

```bash
curl -X POST http://localhost:4040/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "search",
      "arguments": {
        "query": "parallel optimization algorithms",
        "mode": "hybrid",
        "limit": 5
      }
    }
  }'
```

### Resultado

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "results": [
      {
        "id": "src/parallel_engine.rs:42",
        "score": 0.912,
        "content": "pub fn optimize_parallel(...)",
        "source": "hybrid_fusion",
        "breakdown": {
          "vector": 0.89,
          "text": 0.91,
          "heuristic": 0.94
        }
      }
    ],
    "total_time_ms": 3.2,
    "engines_used": ["qdrant", "tantivy", "memorybank"]
  }
}
```

## 🛠️ Análisis Profundo

Para análisis más detallados, usa la herramienta con opciones avanzadas:

```
Analiza el proyecto en ./src buscando:
- Funciones sin tests
- Uso de unsafe
- Complejidad alta (>10)
- Oportunidades de paralelización
- Patrones caóticos (con Julia)
```

El agent usará la herramienta `analyze` con la skill `memory-p-analyzer` para proporcionar insights detallados incluyendo análisis matemático si Julia está disponible.

## 🔧 Reparación Predictiva de Código

MEMORY_P v2.0 incluye reparación predictiva usando matemáticas.

```bash
# Reparación básica
curl -X POST http://localhost:4040/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "repair",
      "arguments": {
        "path": "./src",
        "smart": true,
        "predictive": true
      }
    }
  }'
```

Con `predictive: true`, el sistema:
1. Analiza patrones de errores con Julia
2. Predice impacto de correcciones
3. Aplica solo cambios seguros (>95% confianza)
4. Genera reporte de mejoras

## 🎯 Optimización Matemática (v2.0)

Si Julia está habilitado, puedes optimizar parámetros:

```bash
curl -X POST http://localhost:4040/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
      "name": "optimize",
      "arguments": {
        "target": "search_weights",
        "method": "chaos_theory"
      }
    }
  }'
```

Julia ejecutará:
- Análisis de estabilidad del sistema
- Optimización global con Optim.jl
- Validación con teoría del caos
- Retorno de parámetros óptimos

## 🌊 Workflows Complejos

Para operaciones multi-paso, usa `workflow`:

```
Ejecuta el workflow de análisis completo v2.0:
1. Analizar código con métricas de caos
2. Buscar patrones similares (híbrido)
3. Reparar con predicción
4. Optimizar parámetros (Julia)
5. Generar documentación
6. Ejecutar tests y benchmarks
```

## 📚 Próximos Pasos

Ahora que has completado el tutorial básico:

1. ✅ Explora las [herramientas MCP disponibles](REFERENCE_TOOLS.md)
2. ✅ Lee la [Guía de Reparación Predictiva](HOWTO_REPAIR.md)
3. ✅ Aprende sobre [Custom Agents](../AGENTS.md) y su integración con Julia/JAX
4. ✅ Descubre las [Skills multi-lenguaje](../SKILLS.md)
5. ✅ Revisa el [BLUEPRINT](../BLUEPRINT.md) para arquitectura detallada
6. ✅ Lee [INSTALL.md](../INSTALL.md) para setup completo del stack

### Características Avanzadas a Explorar

- **Búsqueda Híbrida**: Combina vector + full-text + heurísticas
- **Optimización Julia**: Usa matemáticas para mejorar código
- **SIMD Kernels**: Mojo para 35000x más velocidad
- **Actor System**: Pony para concurrencia sin data races
- **ML Inference**: JAX para embeddings y clasificación
- **FFI Bridge**: Zig para interoperabilidad perfecta

## 🌟 Ejemplos del Mundo Real

### Optimizar Parámetros de Búsqueda

```bash
# En el IDE
@memory-p-optimizer optimiza los pesos de búsqueda híbrida
usando análisis de Julia para maximizar precisión@10

# El agent:
# 1. Ejecuta búsquedas de prueba
# 2. Mide precisión actual
# 3. Llama a Julia Optim.jl
# 4. Encuentra pesos óptimos: [0.41, 0.29, 0.30]
# 5. Valida mejora: 87.6% → 91.2% precisión
```

### Detectar Patrones Caóticos

```bash
# Analizar complejidad del módulo
@memory-p-analyzer analiza parallel_engine.rs con métricas de caos

# Resultado:
# - Lyapunov exponent: 0.34 (caótico)
# - Recomendación: Simplificar flujo de control
# - Prioridad: Alta
```

### Benchmarking con Mojo

```bash
# Identificar hotspot y optimizar con SIMD
@performance-benchmark profile dot_product y genera kernel Mojo

# El agent:
# 1. Identifica función crítica
# 2. Genera kernel Mojo con SIMD
# 3. Integra vía FFI
# 4. Benchmark: 12µs → 0.34µs (35x mejora)
```

## ❓ Solución de Problemas

### El servidor no inicia
```bash
# Verificar que el puerto 4040 esté libre
lsof -i :4040  # Linux/Mac
netstat -ano | findstr :4040  # Windows

# Cambiar puerto si es necesario
MEMORY_P_PORT=8080 cargo run --release

# Verificar logs
RUST_LOG=debug ./target/release/memory_p
```

### El IDE no detecta las herramientas
1. Verifica que el servidor esté corriendo: `curl http://localhost:4040/health`
2. Reinicia el IDE
3. Revisa la consola MCP del IDE para errores
4. Verifica configuración MCP (ver sección anterior)

### Julia no se encuentra
```bash
# Verificar instalación
julia --version

# Añadir al PATH
export PATH="/opt/julia/bin:$PATH"

# O desabilitar Julia
cargo build --release --no-default-features --features "core"
```

### Qdrant no conecta
```bash
# Verificar Docker
docker ps | grep qdrant

# Iniciar Qdrant
docker run -d -p 6333:6333 -p 6334:6334 qdrant/qdrant

# Verificar conectividad
curl http://localhost:6333/
```

### Error de compilación
```bash
# Actualizar Rust
rustup update stable

# Limpiar y recompilar
cargo clean
cargo build --release

# Compilar solo core si hay problemas con features
cargo build --release --no-default-features --features "core"
```

### FFI errors (Julia/Mojo/Zig)
```bash
# Verificar que las bibliotecas estén en el path
export LD_LIBRARY_PATH=/opt/julia/lib:$LD_LIBRARY_PATH

# Recompilar FFI components
cd FFI && ./build_all.sh

# Desabilitar FFI features
cargo build --release --no-default-features --features "core,search"
```

## 🤝 Soporte

- **Issues**: [GitHub Issues](https://github.com/Rigohl/MEMORY_P/issues)
- **Documentación**: [Ver docs/](.)
- **Ejemplos**: [Ver PAYLOAD_BANK/](../PAYLOAD_BANK/)
- **Arquitectura**: [BLUEPRINT.md](../BLUEPRINT.md)
- **Instalación**: [INSTALL.md](../INSTALL.md)
- **FFI/Multi-language**: [FFI/README.md](../FFI/README.md)

---

*¡Felicidades! Ahora estás listo para usar MEMORY_P v2.0 con todas sus capacidades multi-lenguaje.* 🎉

**Próximos desafíos**:
- Implementar búsqueda híbrida personalizada
- Optimizar código con Julia
- Crear kernels SIMD con Mojo
- Integrar tus propios agents con matemáticas predictivas
