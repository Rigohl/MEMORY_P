# Referencia de Herramientas MCP

Este documento detalla los parámetros y capacidades de las herramientas expuestas por el servidor **MEMORY_P** vía protocolo MCP 2024-11-05.

## 🔧 Herramientas Disponibles

MEMORY_P expone 5 herramientas principales a través del protocolo MCP:

## 🔧 Herramientas Disponibles

MEMORY_P expone 5 herramientas principales a través del protocolo MCP:

1. **analyze** - Análisis profundo de código con métricas y vulnerabilidades
2. **edit** - Edición masiva con búsqueda y reemplazo
3. **repair** - Reparación inteligente de formato y estructura
4. **workflow** - Orquestación de pipelines complejos
5. **simulate** - Simulaciones de optimización multi-fase

## 📊 Herramientas de Análisis

### `analyze`

Realiza un escaneo profundo buscando métricas de complejidad, vulnerabilidades y patrones.
- **Parámetros**:
  - `path` (string, requerido): Ruta absoluta.
  - `extension` (string, opcional): Filtrar archivos.
  - `max_tasks` (integer, opcional): Número de hilos (Rayon).

### `analyze`

Realiza un escaneo profundo buscando métricas de complejidad, vulnerabilidades y patrones.

- **Parámetros**:
  - `path` (string, requerido): Ruta absoluta al directorio a analizar.
  - `pattern` (string, opcional, default: "**/*.rs"): Patrón glob de archivos.
  - `max_parallel` (integer, opcional): Número de hilos paralelos (Rayon).

- **Retorna**:
  - Número de archivos analizados
  - Líneas de código totales
  - Funciones detectadas
  - Uso de `unsafe` y `unwrap()`
  - Métricas de complejidad

- **Ejemplo**:
  ```json
  {
    "name": "analyze",
    "arguments": {
      "path": "/home/user/proyecto",
      "pattern": "**/*.rs"
    }
  }
  ```

## ✏️ Herramientas de Edición

### `edit`

Normalización masiva de código con búsqueda y reemplazo.
- **Parámetros**:
  - `path` (string, requerido): Directorio raíz.
  - `pattern` (string, opcional): Texto a buscar.
  - `replacement` (string, opcional): Texto a reemplazar.

### `edit`

Normalización masiva de código con búsqueda y reemplazo.

- **Parámetros**:
  - `path` (string, requerido): Directorio raíz.
  - `pattern` (string, requerido): Texto/regex a buscar.
  - `replacement` (string, requerido): Texto de reemplazo.
  - `file_pattern` (string, opcional): Filtro de archivos (e.g., "*.rs").

- **Ejemplo**:
  ```json
  {
    "name": "edit",
    "arguments": {
      "path": "/home/user/proyecto",
      "pattern": "\\t",
      "replacement": "    ",
      "file_pattern": "*.rs"
    }
  }
  ```

### `repair`

Aplica correcciones estructurales automáticas (imports, espacios, líneas vacías).
- **Parámetros**:
  - `smart` (boolean, default: true): Activa la lógica avanzada de deduplicación.

### `repair`

Aplica correcciones estructurales automáticas (imports, espacios, líneas vacías).

- **Parámetros**:
  - `path` (string, requerido): Directorio a reparar.
  - `smart` (boolean, default: true): Activa lógica avanzada de deduplicación.
  - `extension` (string, opcional, default: "rs"): Extensión de archivos.

- **Ejemplo**:
  ```json
  {
    "name": "repair",
    "arguments": {
      "path": "/home/user/proyecto",
      "smart": true
    }
  }
  ```

## 🌊 Herramientas de Workflow

### `workflow`

Orquestación de pipelines complejos con múltiples fases.

- **Parámetros**:
  - `path` (string, requerido): Ruta al archivo de configuración del workflow.
  - `phases` (array, opcional): Lista de fases a ejecutar.

- **Ejemplo**:
  ```json
  {
    "name": "workflow",
    "arguments": {
      "path": "/home/user/proyecto/workflow.json",
      "phases": ["analyze", "repair", "test"]
    }
  }
  ```

## 🌀 Herramientas de Simulación

### `simulate`

Ejecuta simulaciones de optimización multi-fase (hasta 815K iteraciones).

- **Parámetros**:
  - `path` (string, requerido): Directorio del proyecto.
  - `iterations` (integer, opcional, default: 25000): Número de simulaciones.
  - `phases` (integer, opcional, default: 5): Número de fases.

- **Ejemplo**:
  ```json
  {
    "name": "simulate",
    "arguments": {
      "path": "/home/user/proyecto",
      "iterations": 50000,
      "phases": 3
    }
  }
  ```

## ⚠️ Códigos de Error MCP
## ⚠️ Códigos de Error MCP

Errores estándar JSON-RPC 2.0:

| Código | Mensaje | Causa | Solución |
|--------|---------|-------|----------|
| -32600 | Invalid Request | JSON malformado o versión incorrecta | Verificar que `jsonrpc: "2.0"` |
| -32601 | Method not found | La herramienta no existe | Usar `tools/list` para ver herramientas disponibles |
| -32602 | Invalid params | Falta parámetro requerido o tipo incorrecto | Verificar `path` y otros parámetros requeridos |
| -32603 | Internal error | Error del servidor | Ver logs del servidor MEMORY_P |

## 🔗 Integración con Clients

### Cursor / Windsurf

Las herramientas están disponibles automáticamente al conectar con el servidor MCP:

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

### Claude Desktop

Usar modo stdio para máxima compatibilidad:

```json
{
  "mcpServers": {
    "memory_p": {
      "command": "cargo",
      "args": ["run", "--release", "--", "--stdio"]
    }
  }
}
```

## 📚 Ver También

- [Tutorial de Inicio](TUTORIAL_START.md) - Primeros pasos con MEMORY_P
- [Guía de Reparación](HOWTO_REPAIR.md) - Uso detallado de `repair`
- [README principal](../README.md) - Overview del proyecto
