# Referencia de Herramientas MCP v2.0

Este documento detalla los parámetros y capacidades de las herramientas expuestas por el servidor **MEMORY_P v2.0** vía protocolo MCP 2024-11-05.

## 🔧 Herramientas Disponibles

MEMORY_P v2.0 expone 9 herramientas principales a través del protocolo MCP:

### Herramientas Core (Rust)
1. **analyze** - Análisis profundo de código con métricas avanzadas
2. **edit** - Edición masiva con búsqueda y reemplazo
3. **repair** - Reparación inteligente y predictiva
4. **workflow** - Orquestación de pipelines complejos
5. **simulate** - Simulaciones de optimización multi-fase

### Herramientas v2.0 (Multi-Lenguaje)
6. **search** - Búsqueda híbrida (Vector + Full-Text + Heurísticas)
7. **optimize** - Optimización matemática con Julia
8. **embed** - Generación de embeddings con JAX
9. **chaos_analyze** - Análisis de teoría del caos

## 📊 Herramientas de Análisis

### `analyze`

Realiza un escaneo profundo buscando métricas de complejidad, vulnerabilidades y patrones. En v2.0, incluye análisis de caos con Julia.

- **Parámetros**:
  - `path` (string, requerido): Ruta absoluta al directorio a analizar
  - `pattern` (string, opcional, default: "**/*.rs"): Patrón glob de archivos
  - `max_parallel` (integer, opcional): Número de hilos paralelos (Rayon)
  - `deep` (boolean, opcional, default: false): Activa análisis profundo
  - `analyze_chaos` (boolean, opcional, default: false): Análisis con Julia (requiere Julia)

- **Retorna**:
  - Número de archivos analizados
  - Líneas de código totales
  - Funciones detectadas
  - Uso de `unsafe` y `unwrap()`
  - Métricas de complejidad
  - **Nuevo v2.0**: Exponente de Lyapunov (si Julia habilitado)
  - **Nuevo v2.0**: Clasificación de estabilidad

- **Ejemplo**:
  ```json
  {
    "name": "analyze",
    "arguments": {
      "path": "/home/user/proyecto",
      "pattern": "**/*.rs",
      "deep": true,
      "analyze_chaos": true
    }
  }
  ```

- **Respuesta v2.0**:
  ```json
  {
    "files_analyzed": 42,
    "total_lines": 12583,
    "functions": 328,
    "complexity_avg": 4.2,
    "chaos_metrics": {
      "lyapunov_exponent": 0.23,
      "classification": "semi-chaotic",
      "stability_score": 0.78
    }
  }
  ```

## ✏️ Herramientas de Edición

### `edit`

Normalización masiva de código con búsqueda y reemplazo paralelo.

- **Parámetros**:
  - `path` (string, requerido): Directorio raíz
  - `pattern` (string, requerido): Texto/regex a buscar
  - `replacement` (string, requerido): Texto de reemplazo
  - `file_pattern` (string, opcional): Filtro de archivos (e.g., "*.rs")
  - `parallel` (boolean, opcional, default: true): Procesamiento paralelo con Rayon

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

Aplica correcciones estructurales automáticas con predicción matemática (v2.0).

- **Parámetros**:
  - `path` (string, requerido): Directorio a reparar
  - `smart` (boolean, default: true): Activa lógica avanzada de deduplicación
  - `extension` (string, opcional, default: "rs"): Extensión de archivos
  - **Nuevo v2.0**: `predictive` (boolean, default: false): Análisis predictivo con Julia
  - **Nuevo v2.0**: `analyze_chaos` (boolean, default: false): Incluir métricas de caos
  - **Nuevo v2.0**: `dry_run` (boolean, default: false): Simular sin aplicar cambios

- **Ejemplo v2.0**:
  ```json
  {
    "name": "repair",
    "arguments": {
      "path": "/home/user/proyecto",
      "smart": true,
      "predictive": true,
      "analyze_chaos": true
    }
  }
  ```

- **Respuesta v2.0**:
  ```json
  {
    "files_processed": 42,
    "changes_applied": 187,
    "prediction": {
      "impact_confidence": 0.97,
      "breaking_changes_risk": 0.02,
      "recommended": true
    },
    "chaos_metrics": {
      "lyapunov_before": 0.45,
      "lyapunov_after": 0.12,
      "improvement": "73%"
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

---

## 🔍 Herramientas v2.0 (Multi-Lenguaje)

### `search`

Búsqueda híbrida combinando Vector Search, Full-Text y Heurísticas.

- **Parámetros**:
  - `query` (string, requerido): Consulta de búsqueda (lenguaje natural o keywords)
  - `mode` (string, opcional, default: "hybrid"): Modo de búsqueda
    - `"vector"`: Solo búsqueda vectorial (Qdrant)
    - `"text"`: Solo full-text (Tantivy)
    - `"heuristic"`: Solo heurísticas (MemoryBank)
    - `"hybrid"`: Fusión de todos (RRF)
  - `limit` (integer, opcional, default: 10): Número máximo de resultados
  - `weights` (object, opcional): Pesos para fusión híbrida
    - `vector` (float, default: 0.4)
    - `text` (float, default: 0.35)
    - `heuristic` (float, default: 0.25)

- **Requiere**: Qdrant + Tantivy (en v2.0)

- **Ejemplo**:
  ```json
  {
    "name": "search",
    "arguments": {
      "query": "parallel optimization algorithms",
      "mode": "hybrid",
      "limit": 5,
      "weights": {
        "vector": 0.41,
        "text": 0.29,
        "heuristic": 0.30
      }
    }
  }
  ```

- **Respuesta**:
  ```json
  {
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
  ```

### `optimize`

Optimización matemática de parámetros usando Julia.

- **Parámetros**:
  - `target` (string, requerido): Objetivo a optimizar
    - `"search_weights"`: Pesos de búsqueda híbrida
    - `"parallel_config"`: Configuración de paralelismo
    - `"custom"`: Función objetivo personalizada
  - `method` (string, opcional, default: "LBFGS"): Algoritmo de optimización
    - `"LBFGS"`: Quasi-Newton
    - `"NelderMead"`: Simplex
    - `"SimulatedAnnealing"`: Recocido simulado
    - `"chaos_theory"`: Basado en análisis de caos
  - `initial_params` (array, opcional): Parámetros iniciales
  - `max_iterations` (integer, opcional, default: 1000)

- **Requiere**: Julia + Optim.jl

- **Ejemplo**:
  ```json
  {
    "name": "optimize",
    "arguments": {
      "target": "search_weights",
      "method": "chaos_theory",
      "max_iterations": 5000
    }
  }
  ```

- **Respuesta**:
  ```json
  {
    "optimal_params": [0.41, 0.29, 0.30],
    "objective_value": 0.912,
    "iterations": 2847,
    "convergence": true,
    "chaos_metrics": {
      "lyapunov": 0.12,
      "stability": "high"
    }
  }
  ```

### `embed`

Generación de embeddings semánticos con JAX.

- **Parámetros**:
  - `texts` (array[string], requerido): Textos a embedder
  - `model` (string, opcional, default: "all-MiniLM-L6-v2"): Modelo a usar
  - `batch_size` (integer, opcional, default: 32): Tamaño de batch
  - `normalize` (boolean, opcional, default: true): Normalizar vectores

- **Requiere**: Python + JAX + sentence-transformers

- **Ejemplo**:
  ```json
  {
    "name": "embed",
    "arguments": {
      "texts": [
        "parallel programming with Rust",
        "mathematical optimization"
      ],
      "model": "all-MiniLM-L6-v2"
    }
  }
  ```

- **Respuesta**:
  ```json
  {
    "embeddings": [
      [0.123, -0.456, 0.789, ...],  // 384-dim
      [0.234, -0.567, 0.890, ...]
    ],
    "dimensions": 384,
    "model": "all-MiniLM-L6-v2",
    "time_ms": 46.2
  }
  ```

### `chaos_analyze`

Análisis de complejidad usando teoría del caos (Julia).

- **Parámetros**:
  - `metrics` (array[number], requerido): Serie temporal de métricas
  - `dimension` (integer, opcional, default: 3): Dimensión de embedding
  - `delay` (integer, opcional, default: 1): Delay de reconstrucción
  - `iterations` (integer, opcional, default: 1000): Iteraciones para Lyapunov

- **Requiere**: Julia + ChaosTools.jl

- **Ejemplo**:
  ```json
  {
    "name": "chaos_analyze",
    "arguments": {
      "metrics": [4.2, 5.1, 6.8, 5.9, 7.2, 8.1, 6.5],
      "dimension": 3
    }
  }
  ```

- **Respuesta**:
  ```json
  {
    "lyapunov_exponent": 0.34,
    "classification": "chaotic",
    "stability_score": 0.45,
    "recommendations": [
      "Refactorizar para reducir complejidad",
      "Simplificar flujo de control",
      "Considerar modularización"
    ]
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

- [Tutorial de Inicio](TUTORIAL_START.md) - Primeros pasos con MEMORY_P v2.0
- [Guía de Reparación Predictiva](HOWTO_REPAIR.md) - Uso detallado de `repair` con Julia
- [README principal](../README.md) - Overview del proyecto v2.0
- [BLUEPRINT](../BLUEPRINT.md) - Arquitectura técnica completa
- [FFI Documentation](../FFI/README.md) - Motor MemoryBank y multi-lenguaje
- [INSTALL](../INSTALL.md) - Instalación del stack completo

---

**Última actualización**: Enero 2026  
**Versión**: 2.0  
**MCP Protocol**: 2024-11-05  
**JSON-RPC**: 2.0
