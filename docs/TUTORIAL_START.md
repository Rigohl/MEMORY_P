# MEMORY_P: Tutorial de Inicio Rápido

Bienvenido a **MEMORY_P**, el motor de procesamiento masivo paralelo diseñado para integrarse con Cursor, Windsurf, Claude Desktop y VS Code vía protocolo MCP 2024-11-05.

En este tutorial, aprenderás a:
- ✅ Configurar tu entorno
- ✅ Conectar MEMORY_P con tu IDE
- ✅ Realizar tu primer análisis de código
- ✅ Usar las herramientas MCP disponibles

## 📋 Requisitos Previos

### Software Necesario
- **Rust Stable 1.70+**: Instalado mediante `rustup`
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Cargo**: El gestor de paquetes de Rust (incluido con rustup)
- **MCP Client**: Cursor, Windsurf o Claude Desktop configurados

### Verificar Instalación
```bash
rustc --version  # Debe mostrar >= 1.70
cargo --version
```

## 🚀 Instalación y Ejecución

### 1. Clonar el Repositorio
```bash
git clone https://github.com/Rigohl/MEMORY_P.git
cd MEMORY_P
```

### 2. Compilar en Modo Release
```bash
cargo build --release
```

Este paso puede tomar varios minutos la primera vez, ya que compila todas las dependencias.

### 3. Ejecutar el Servidor

#### Modo HTTP (para Cursor/Windsurf)
```bash
./target/release/memory_p
# Servidor escuchando en http://127.0.0.1:4040
```

#### Modo stdio (para Claude Desktop)
```bash
cargo run --release -- --stdio
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

Una vez conectado, puedes usar la herramienta `analyze` para obtener métricas de tu código.

### Desde el Chat del IDE

En Cursor o Windsurf, escribe:
```
Analiza los archivos .rs en ./src usando la herramienta analyze
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
        "pattern": "**/*.rs"
      }
    }
  }'
```

### Resultado Esperado

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
    "complexity_avg": 4.2
  }
}
```

## 🛠️ Análisis Profundo

Para análisis más detallados, usa la herramienta con opciones avanzadas:

## 🛠️ Análisis Profundo

Para análisis más detallados, usa la herramienta con opciones avanzadas:

```
Analiza el proyecto en ./src buscando:
- Funciones sin tests
- Uso de unsafe
- Complejidad alta (>10)
- Oportunidades de paralelización
```

El agent usará la herramienta `analyze` con la skill `memory-p-analyzer` para proporcionar insights detallados.

## 🔧 Reparación de Código

Una vez identificados problemas, usa `repair` para limpiar el código:

```
Repara el código en ./src eliminando espacios y deduplicando imports
```

## 🌊 Workflows Complejos

Para operaciones multi-paso, usa `workflow`:

```
Ejecuta el workflow de análisis completo:
1. Analizar código
2. Reparar formato
3. Generar documentación
4. Ejecutar tests
```

## 📚 Próximos Pasos

Ahora que has completado el tutorial básico:

1. ✅ Explora las [5 herramientas MCP disponibles](REFERENCE_TOOLS.md)
2. ✅ Lee la [Guía de Reparación Inteligente](HOWTO_REPAIR.md)
3. ✅ Aprende sobre [Custom Agents](../AGENTS.md)
4. ✅ Descubre las [Skills disponibles](../SKILLS.md)
5. ✅ Revisa el [README principal](../README.md) para arquitectura

## ❓ Solución de Problemas

### El servidor no inicia
```bash
# Verificar que el puerto 4040 esté libre
lsof -i :4040  # Linux/Mac
netstat -ano | findstr :4040  # Windows

# Cambiar puerto si es necesario
MEMORY_P_PORT=8080 cargo run --release
```

### El IDE no detecta las herramientas
1. Verifica que el servidor esté corriendo: `curl http://localhost:4040/`
2. Reinicia el IDE
3. Revisa la consola MCP del IDE para errores

### Error de compilación
```bash
# Actualizar Rust
rustup update stable

# Limpiar y recompilar
cargo clean
cargo build --release
```

## 🤝 Soporte

- **Issues**: [GitHub Issues](https://github.com/Rigohl/MEMORY_P/issues)
- **Documentación**: [Ver docs/](.)
- **Ejemplos**: [Ver PAYLOAD_BANK/](../PAYLOAD_BANK/)

---

*Felicidades! Ahora estás listo para usar MEMORY_P en tu flujo de trabajo diario.* 🎉
