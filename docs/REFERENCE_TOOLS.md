# Referencia de Herramientas MCP

Este documento detalla las 5 herramientas MCP expuestas por **MEMORY_P v2025.2.ULTRA**.

## 🔬 Tool 1: `analyze`

Análisis masivo paralelo con 4 modos de operación.

**Parámetros:**
- `path` (string, requerido): Ruta al proyecto
- `mode` (string, enum): Modo de análisis
  - `"deep"`: Análisis completo con métricas, seguridad y complejidad
  - `"quick"`: Análisis rápido solo con métricas básicas
  - `"overview"`: Vista arquitectónica del proyecto
  - `"optimize"`: **🆕 Auto-optimización de threads usando Amdahl's Law**
- `extension` (string, default: "rs"): Filtrar por extensión
- `use_gitignore` (boolean, default: true): Respetar .gitignore
- `include_hidden` (boolean, default: false): Incluir archivos ocultos

**Modo Optimize (Avanzado):**

El modo `optimize` utiliza un modelo matemático basado en **Amdahl's Law** para calcular automáticamente la configuración óptima de paralelismo para tu carga de trabajo:

- Analiza el tamaño del workload (número de archivos)
- Calcula overhead por thread (context switching, sincronización)
- Balancea throughput vs overhead
- Retorna: threads recomendados, batch size óptimo, speedup esperado, efficiency score

**Ejemplo (overview):**
```json
{
  "name": "analyze",
  "arguments": {
    "path": "./src",
    "mode": "overview"
  }
}
```

**Ejemplo (optimize - Auto-Optimización):**
```json
{
  "name": "analyze",
  "arguments": {
    "path": "./src",
    "mode": "optimize",
    "extension": "rs"
  }
}
```

Salida ejemplo:
```
⚡ Auto-Optimization Results:
📂 Workload: 1523 files
🧵 Recommended Threads: 16
📦 Recommended Batch Size: 100
🚀 Expected Speedup: 12.8x
📊 Efficiency Score: 0.9234

💡 Apply with: max_threads=16, chunk_size=100
```

## 🛠️ Tool 2: `repair`

Reparación paralela automática de código.

**Parámetros:**
- `path` (string, requerido): Ruta al proyecto
- `extension` (string, default: "rs"): Extensión de archivos
- `dry_run` (boolean, default: false): Simular sin escribir

**Aplica:**
- Deduplicación de imports
- Normalización de espacios finales
- EOL consistency
- Reducción de líneas vacías consecutivas

**Ejemplo:**
```json
{
  "name": "repair",
  "arguments": {
    "path": "./src",
    "dry_run": false
  }
}
```

## ✏️ Tool 3: `edit`

Edición masiva atómica con 4 modos.

**Parámetros:**
- `mode` (string, enum, requerido): Tipo de operación
  - `"replace"`: Reemplazo literal de texto
  - `"regex"`: Búsqueda/reemplazo con regex
  - `"append"`: Añadir contenido
  - `"delete"`: Eliminar archivos
- `changes` (array): Para modes replace/regex/append
  - `path` (string): Archivo a modificar
  - `operations` (array): Lista de operaciones
- `paths` (array): Para mode delete
- `dry_run` (boolean, default: true): Simular cambios

**Ejemplo (replace):**
```json
{
  "name": "edit",
  "arguments": {
    "mode": "replace",
    "changes": [{
      "path": "./src/main.rs",
      "operations": [{
        "search": "old_text",
        "replace": "new_text"
      }]
    }],
    "dry_run": false
  }
}
```

## 🌊 Tool 4: `workflow`

Pipeline de pasos secuenciales con auto-evolución.

**Parámetros:**
- `steps` (array, requerido): Lista de pasos
  - `action` (string, enum): "Scan", "Filter", "Analyze", "Edit", "Repair", "Evolve"
  - `params` (object): Parámetros del paso
- `dry_run` (boolean, default: true): Modo simulación
- `max_threads` (integer): Threads para procesamiento paralelo

**Ejemplo:**
```json
{
  "name": "workflow",
  "arguments": {
    "steps": [
      {"action": "Scan", "params": {"path": "./src"}},
      {"action": "Analyze", "params": {}},
      {"action": "Repair", "params": {}}
    ],
    "dry_run": false
  }
}
```

## 🌀 Tool 5: `simulate`

Mega simulaciones en 3 fases para optimización.

**Parámetros:**
- `phase` (integer, enum, requerido): 1, 2 o 3
  - `1`: Optimización por módulo (15K iterations)
  - `2`: Tuning de paralelismo (150K iterations)
  - `3`: Análisis de ecosistema (500K iterations)
- `iterations` (integer, default: 1000): Simulaciones por config
- `modules` (array): Solo para phase 1 - lista de módulos
- `use_gpu` (boolean, default: false): Usar aceleración GPU

**Ejemplo:**
```json
{
  "name": "simulate",
  "arguments": {
    "phase": 2,
    "iterations": 5000,
    "use_gpu": false
  }
}
```

## 📊 Códigos de Error MCP

| Código | Mensaje | Causa |
|--------|---------|-------|
| -32600 | Invalid JSON-RPC version | La versión no es "2.0" |
| -32601 | Method not found | El método MCP no existe |
| -32602 | Invalid params | Parámetros faltantes o inválidos |
| -32603 | Internal error | Error interno del servidor |
