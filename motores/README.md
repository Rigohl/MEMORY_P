# Motores - 8 Search Engines Architecture

This directory contains the implementation of MEMORY_P v2.0's 8 specialized search engines.

## Directory Structure

```
motores/
├── core/                   # Common traits and types
│   ├── search_engine.rs    # SearchEngine trait
│   ├── types.rs           # Shared types
│   └── routing_ai.rs      # AI routing logic
│
├── vector_search/         # Vector similarity engines (3)
│   ├── qdrant/           # Qdrant semantic general
│   ├── faiss/            # FAISS-GPU ultra-fast
│   └── scann/            # SCANN Google trillion-scale
│
├── text_search/          # Text search engines (3)
│   ├── tantivy/          # Tantivy single-node BM25
│   ├── lnx/              # LNX distributed Raft
│   └── meilisearch/      # MeiliSearch typo-tolerant
│
├── specialized/          # Specialized engines (2)
│   ├── julia_nlp/        # Julia NLP mathematical
│   └── memory_bank/      # MemoryBank Ultra FFI
│
├── hybrid/               # Intelligent coordination
│   ├── fusion_engine.rs  # Multi-engine fusion
│   └── routing_ai.rs     # AI-based routing
│
└── factory.rs            # Engine factory pattern
```

## Engine Selection Guide

### Vector Search
- **Qdrant**: <10M vectors, semantic search with metadata
- **FAISS-GPU**: 10M-1B vectors, ultra-low latency (<2ms)
- **SCANN**: >1B vectors, enterprise trillion-scale

### Text Search
- **Tantivy**: <50M docs, single-node BM25
- **LNX**: >50M docs, distributed multi-node
- **MeiliSearch**: Any size, typo-tolerant user-facing

### Specialized
- **Julia NLP**: Mathematical text analysis, fuzzy matching
- **MemoryBank Ultra**: Multi-language FFI coordination

## Integration

Each engine implements the `SearchEngine` trait defined in `core/search_engine.rs`:

```rust
#[async_trait]
pub trait SearchEngine: Send + Sync {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>;
    fn engine_type(&self) -> EngineType;
    fn engine_name(&self) -> &str;
    async fn health_check(&self) -> HealthStatus;
}
```

## Documentation

See:
- [MOTOR_ARCHITECTURE.md](../docs/MOTOR_ARCHITECTURE.md) - Detailed engine specs
- [DISTRIBUTED_ARCHITECTURE.md](../docs/DISTRIBUTED_ARCHITECTURE.md) - Scaling strategies
- [README.md](../README.md) - Architecture overview

---

**Status:** Structure defined, implementations in progress  
**Version:** 2.0.0  
**Last Updated:** Enero 2026
