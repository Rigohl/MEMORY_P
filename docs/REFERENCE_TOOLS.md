# Referencia de Herramientas MCP

Este documento detalla los parámetros y capacidades de las herramientas expuestas por el servidor **MEMORY_P**.

## Herramientas de Análisis

### `scan_project`
Lista archivos en un directorio filtrando por extensión.
- **Parámetros**:
  - `path` (string, requerido): Ruta absoluta al directorio.
  - `extension` (string, opcional, default: "rs"): Extensión de archivo.

### `analyze_project`
Realiza un escaneo profundo buscando métricas de complejidad y vulnerabilidades.
- **Parámetros**:
  - `path` (string, requerido): Ruta absoluta.
  - `extension` (string, opcional): Filtrar archivos.
  - `max_tasks` (integer, opcional): Número de hilos (Rayon).

## Herramientas de Edición

### `edit_project`
Normalización masiva de código (Tabs -> 4 Espacios).
- **Parámetros**:
  - `path` (string, requerido): Directorio raíz.
  - `pattern` (string, opcional): Texto a buscar.
  - `replacement` (string, opcional): Texto a reemplazar.

### `repair_project`
Aplica correcciones estructurales (imports, espacios, líneas vacías).
- **Parámetros**:
  - `smart` (boolean, default: true): Activa la lógica avanzada de deduplicación.

## Códigos de Error
| Código | Mensaje | Causa |
|--------|---------|-------|
| -32600 | Invalid JSON-RPC version | La versión no es "2.0". |
| -32601 | Method not found | La herramienta no existe. |
| -32602 | Invalid params | Falta el parámetro `path` o es inválido. |
