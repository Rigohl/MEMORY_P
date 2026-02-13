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
# Vector Search API - Documentación Completa

## 🎯 Overview

MEMORY_P ahora incluye un sistema de búsqueda vectorial avanzado similar a Qdrant, con capacidades de:
- **Índices HNSW** (Hierarchical Navigable Small World) para búsquedas ultra-rápidas
- **Múltiples métricas de distancia** (cosine, euclidean, dot product, manhattan)
- **Filtros avanzados por metadata** con operadores must/must_not
- **Búsqueda por batch** para alto rendimiento
- **Embeddings con JAX** y cache en memoria (Redis opcional)
- **Soporte para múltiples modelos** (MiniLM, BGE, E5)

## 🚀 Quick Start

### 1. Indexar Documentos

```bash
curl -X POST http://localhost:4040/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "index_documents",
      "arguments": {
        "documents": [
          {
            "id": "doc1",
            "text": "Rust is a systems programming language focused on safety and performance",
            "metadata": {
              "category": "programming",
              "language": "rust",
              "difficulty": "intermediate"
            }
          },
          {
            "id": "doc2",
            "text": "Python is popular for machine learning and data science",
            "metadata": {
              "category": "programming",
              "language": "python",
              "difficulty": "beginner"
            }
          },
          {
            "id": "doc3",
            "text": "Docker containers provide isolated environments for applications",
            "metadata": {
              "category": "devops",
              "tool": "docker",
              "difficulty": "intermediate"
            }
          }
        ],
        "model": "MiniLM-L6",
        "batch_size": 32
      }
    }
  }'
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [{
      "type": "text",
      "text": "📚 Indexing Complete\n\n✅ Indexed: 3\n❌ Failed: 0\n⏱️ Time: 45ms"
    }]
  }
}
```

### 2. Búsqueda Vectorial Simple

```bash
curl -X POST http://localhost:4040/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "map_search",
      "arguments": {
        "query": "programming languages for system development",
        "limit": 5,
        "metric": "cosine"
      }
    }
  }'
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [{
      "type": "text",
      "text": "🔍 Vector Search Results\n\n⏱️ Query time: 12ms\n📊 Results: 2\n🤖 Model: MiniLM-L6-v2\n\n• [doc1] Score: 0.8745\n  Metadata: {\"category\":\"programming\",\"language\":\"rust\"}\n\n• [doc2] Score: 0.7234\n  Metadata: {\"category\":\"programming\",\"language\":\"python\"}"
    }]
  }
}
```

### 3. Búsqueda con Filtros Avanzados

```bash
curl -X POST http://localhost:4040/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "map_search",
      "arguments": {
        "query": "beginner friendly technologies",
        "limit": 10,
        "filters": {
          "must": {
            "difficulty": "beginner"
          },
          "must_not": {
            "category": "devops"
          }
        }
      }
    }
  }'
```

### 4. Encontrar Documentos Similares

```bash
curl -X POST http://localhost:4040/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
      "name": "similar_docs",
      "arguments": {
        "document_id": "doc1",
        "limit": 5
      }
    }
  }'
```

### 5. Estadísticas del Motor

```bash
curl -X POST http://localhost:4040/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 5,
    "method": "tools/call",
    "params": {
      "name": "vector_stats",
      "arguments": {}
    }
  }'
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "content": [{
      "type": "text",
      "text": "📊 Vector Engine Statistics\n\n{\n  \"initialized\": true,\n  \"total_documents\": 1250,\n  \"total_queries\": 457,\n  \"dimension\": 384,\n  \"metric\": \"Cosine\",\n  \"cache_stats\": {\n    \"cache_size\": 1250\n  }\n}"
    }]
  }
}
```

## 🔧 Configuración Avanzada

### Modelos de Embeddings Soportados

| Modelo | Dimensión | Uso Recomendado |
|--------|-----------|-----------------|
| **MiniLM-L6** | 384 | Rápido, general purpose (default) |
| **MiniLM-L12** | 384 | Balance precisión/velocidad |
| **BGE-Small** | 384 | Alta calidad, idioma inglés |
| **BGE-Base** | 768 | Mayor precisión, más recursos |
| **BGE-Large** | 1024 | Máxima precisión |
| **E5-Small** | 384 | Multilingüe |
| **E5-Base** | 768 | Multilingüe, alta calidad |

### Métricas de Distancia

| Métrica | Descripción | Mejor Para |
|---------|-------------|------------|
| **Cosine** | Similitud angular (default) | Embeddings normalizados, textos |
| **Euclidean** | Distancia L2 | Vectores densos, coordenadas |
| **DotProduct** | Producto punto | Embeddings optimizados |
| **Manhattan** | Distancia L1 | Datos dispersos |

### Filtros por Metadata

Los filtros soportan tres operadores:

#### 1. `must` - Condiciones que DEBEN cumplirse
```json
{
  "must": {
    "category": "tech",
    "status": "published",
    "language": "en"
  }
}
```

#### 2. `must_not` - Condiciones de EXCLUSIÓN
```json
{
  "must_not": {
    "archived": true,
    "draft": true
  }
}
```

#### 3. `timestamp_range` - Rango de timestamps
```json
{
  "timestamp_range": [1704067200, 1735689599]
}
```

#### Combinación de filtros
```json
{
  "must": {
    "category": "tech",
    "verified": true
  },
  "must_not": {
    "status": "draft"
  },
  "timestamp_range": [1704067200, 1735689599]
}
```

## 📊 Casos de Uso

### 1. Sistema de Recomendaciones

```python
# Indexar artículos
index_request = {
    "documents": [
        {
            "id": f"article_{i}",
            "text": article["content"],
            "metadata": {
                "title": article["title"],
                "author": article["author"],
                "tags": article["tags"],
                "views": article["view_count"]
            }
        }
        for i, article in enumerate(articles)
    ]
}

# Encontrar artículos similares
similar = find_similar("article_42", limit=10)
```

### 2. Búsqueda Semántica en Documentación

```python
# Indexar documentación
docs = []
for file in glob("docs/**/*.md"):
    content = read_file(file)
    docs.append({
        "id": file,
        "text": content,
        "metadata": {
            "type": "markdown",
            "section": get_section(file),
            "last_modified": os.path.getmtime(file)
        }
    })

index_documents(docs)

# Búsqueda semántica
results = search("how to configure authentication?", limit=5)
```

### 3. Detección de Duplicados

```python
# Indexar documentos
index_documents(all_documents)

# Encontrar duplicados
for doc in all_documents:
    similar = find_similar(doc.id, limit=5)
    if similar[0].score > 0.95:  # Muy similar
        print(f"Posible duplicado: {doc.id} ~ {similar[0].id}")
```

### 4. Clasificación por Similitud

```python
# Indexar ejemplos etiquetados
labeled_examples = [
    {"id": "spam_1", "text": "...", "metadata": {"label": "spam"}},
    {"id": "ham_1", "text": "...", "metadata": {"label": "ham"}},
    # ...
]

index_documents(labeled_examples)

# Clasificar nuevo texto
results = search(new_text, limit=5)
predicted_label = most_common([r.metadata["label"] for r in results])
```

## 🎨 Integración con Cursor/Windsurf

### En `.cursorrules` o `.windsurfrules`:

```yaml
tools:
  - name: search_codebase
    description: Búsqueda semántica en código
    uses: map_search
    config:
      model: BGE-Small
      metric: cosine

  - name: find_similar_files
    description: Encuentra archivos similares
    uses: similar_docs
```

### Uso en chat:

```
@workspace Encuentra código similar a este módulo de autenticación
```

El agente automáticamente usará `similar_docs` con el contexto actual.

## ⚡ Performance Tips

### 1. Batch Indexing
```json
{
  "documents": [...],  // Hasta 1000 documentos
  "batch_size": 64     // Óptimo para embeddings paralelos
}
```

### 2. Cache de Embeddings
El sistema cachea embeddings automáticamente. Textos idénticos no se re-calculan.

### 3. Filtros Pre-búsqueda
Aplica filtros restrictivos primero para reducir el espacio de búsqueda:
```json
{
  "filters": {
    "must": {"active": true},  // Reduce dataset
    "timestamp_range": [recent_start, now]
  }
}
```

### 4. Límite Apropiado
- Para previews: `limit: 5-10`
- Para análisis: `limit: 50-100`
- Máximo: `limit: 1000`

## 🔒 Seguridad y Límites

| Parámetro | Límite | Razón |
|-----------|--------|-------|
| `limit` | 1-1000 | Prevenir OOM |
| `batch_size` | 1-256 | Balance memoria/velocidad |
| `documents.length` | 1-10000 | Timeout prevención |
| `query.length` | 1-10000 chars | Embedding limits |

## 🐛 Troubleshooting

### Error: "Motor vectorial no inicializado"
**Solución:** El motor se inicializa automáticamente en el primer uso. Verifica que el servidor esté corriendo.

### Error: "Vector dimension mismatch"
**Solución:** Asegúrate de usar el mismo modelo para indexar y buscar. Limpia el índice si cambiaste de modelo.

### Búsqueda muy lenta
**Soluciones:**
1. Reduce el `limit`
2. Agrega filtros restrictivos
3. Usa modelo más pequeño (MiniLM-L6)
4. Considera aumentar recursos del servidor

### Cache crece demasiado
**Solución:**
```bash
# Limpiar cache manualmente
curl -X POST http://localhost:4040/mcp/admin/clear_cache
```

## 📚 Referencias

- [MCP Specification](https://spec.modelcontextprotocol.io/specification/draft/basic/utilities/)
- [HNSW Paper](https://arxiv.org/abs/1603.09320)
- [Sentence Transformers](https://www.sbert.net/)
- [Vector Search Best Practices](https://www.pinecone.io/learn/vector-search/)

## 🤝 Contribuir

Reporta issues o mejoras en: https://github.com/tu-org/MEMORY_P/issues

---

**Versión:** 2.0.0
**Última actualización:** Enero 2025
**Compatibilidad MCP:** 2024-11-05+
