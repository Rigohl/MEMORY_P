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
