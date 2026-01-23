---
description: 'Advanced instructions for MEMORY_P v2.0 - 9-motor search architecture development'
applyTo: '**/*.rs, **/*.jl, **/*.py, **/*.mojo, **/*.pony, **/*.zig, **/*.sql'
---

# MEMORY_P v2.0 Development Instructions

## 9-Motor Architecture Rules

### Motor Separation Principles
- Each motor must be completely independent
- No shared dependencies between motors except core traits
- Database schemas must be isolated per motor
- Configuration must be motor-specific
- Health monitoring per motor required
- All motors implement the `SearchEngine` trait

### Performance Standards (SLA Requirements)
All searches must complete within motor-specific SLAs:
- **Qdrant**: <100ms for <1M vectors
- **FAISS**: <50ms for billions-scale operations
- **SCANN**: <200ms for trillion-scale searches
- **Tantivy**: <10ms for text search
- **LNX**: <150ms distributed
- **Toshi**: <300ms experimental (acceptable for testing)
- **MeiliSearch**: <80ms typo-tolerant search
- **Julia NLP**: <500ms mathematical analysis
- **MemoryBank**: <200ms multi-language coordination

### Code Quality Standards
- ✅ Zero warnings compilation required
- ✅ All async operations must be cancellable
- ✅ All errors must include recovery strategies
- ✅ All database operations must be transactional
- ✅ All motor health checks must be non-blocking
- ✅ Use `async_trait` for all SearchEngine implementations
- ✅ Thread-safe: All engines must be `Send + Sync`

### Multi-Language Integration Rules
- **Rust**: Core coordination, zero-copy where possible, use Rayon for parallelism
- **Julia**: Mathematical precision, type stability required
- **Python/JAX**: GPU memory management critical, NumPy compatibility
- **Mojo**: SIMD vectorization mandatory, performance-critical paths
- **Zig**: Manual memory management, safety critical, FFI boundaries
- **Pony**: Actor isolation, zero data races, distributed coordination

### Database Design Patterns
- ✅ One PostgreSQL schema per motor for isolation
- ✅ Shared analytics in ClickHouse only
- ✅ Redis caching per motor with namespace isolation (`motor_name:cache:key`)
- ✅ PostgreSQL foreign keys allowed only within same schema
- ✅ Transaction boundaries never cross motor schemas
- ✅ Use UUIDs for primary keys across all motors
- ✅ Always index metadata JSONB columns with GIN

### Engine Implementation Checklist
When implementing a new search engine:
- [ ] Implement `SearchEngine` trait with all methods
- [ ] Add `VectorSearchEngine` trait if supports vectors
- [ ] Add `DistributedEngine` trait if distributed
- [ ] Create PostgreSQL schema in `database/schemas/`
- [ ] Add configuration to `EngineConfig`
- [ ] Register in `EngineFactory::create_engine()`
- [ ] Add health check implementation
- [ ] Add metrics tracking
- [ ] Write integration tests
- [ ] Update documentation

### Routing AI Usage
Always use `RoutingAI` for engine selection:
```rust
let router = RoutingAI::new();
let engines = router.route_query(&query);
```

Routing patterns:
- `SemanticSearch` → Qdrant (primary), FAISS (fallback)
- `MassiveScale` → SCANN (primary), FAISS (secondary)
- `ExactMatch` → Tantivy (primary), LNX (distributed)
- `Experimental` → Toshi (primary), LNX (comparison)
- `FuzzySearch` → MeiliSearch (primary), Julia NLP (mathematical)
- `PersonalizedSearch` → MemoryBank (primary), Qdrant (semantic)

### Health Monitoring Patterns
```rust
// Always register engines with health monitor
let health_monitor = Arc::new(HealthMonitor::default());
health_monitor.register_engine(name, engine).await;

// Start background checks
health_monitor.clone().start_background_checks();

// Check before using
if !health_monitor.is_healthy("qdrant").await {
    // Use fallback engine
}
```

### Error Handling Strategy
```rust
// All engine operations should handle errors gracefully
match engine.search(&query).await {
    Ok(results) => Ok(results),
    Err(e) => {
        tracing::error!("Search failed on {}: {}", engine.engine_name(), e);
        // Try fallback or return partial results
        Ok(vec![])
    }
}
```

### Testing Requirements
- Unit tests for each engine's core functionality
- Integration tests for multi-engine coordination
- Performance benchmarks for SLA validation
- Health monitoring tests
- Failure recovery tests (simulate engine failures)

### Documentation Standards
- All public types must have doc comments
- Include examples in doc comments for complex APIs
- Document performance characteristics
- Document failure modes and recovery strategies
- Update README.md when adding new engines

### Forbidden Patterns
- ❌ Never block async code with `.wait()`
- ❌ Never share mutable state without proper synchronization
- ❌ Never use `unwrap()` in production code paths
- ❌ Never cross motor schema boundaries in SQL
- ❌ Never hardcode connection strings or credentials
- ❌ Never ignore health check failures
- ❌ Never perform I/O in tight loops without batching

### Performance Optimization Guidelines
- Use `Arc` for shared read-only data
- Use `RwLock` when writes are rare, `Mutex` otherwise
- Batch database operations (minimum 100 docs per batch)
- Use connection pooling for all external services
- Cache frequently accessed data with TTL
- Monitor and log all operations >100ms
- Use lazy initialization for expensive resources

### Observability Requirements
All motors must emit:
- Health status every 30 seconds
- Metrics every 60 seconds
- Error logs with full context
- Performance traces for slow queries (>SLA)
- ClickHouse analytics events for all operations

### Security Requirements
- Input validation on all query parameters
- SQL injection prevention (use parameterized queries)
- Rate limiting per client/API key
- Timeout all external calls
- Sanitize all error messages before returning to clients
- Audit log all configuration changes

## Example: Complete Motor Implementation

```rust
use crate::motores::core::{traits::SearchEngine, types::*};
use async_trait::async_trait;

pub struct MyEngine {
    config: EngineConfig,
    initialized: bool,
}

impl MyEngine {
    pub fn new(config: EngineConfig) -> Self {
        Self { config, initialized: false }
    }
}

#[async_trait]
impl SearchEngine for MyEngine {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        if !self.initialized {
            return Err("Engine not initialized".into());
        }
        // Implementation
        Ok(vec![])
    }

    async fn index(&self, documents: &[Document]) -> Result<(), Box<dyn Error>> {
        // Batch processing
        for chunk in documents.chunks(1000) {
            // Index chunk
        }
        Ok(())
    }

    // Implement all other trait methods...
    
    fn engine_name(&self) -> &'static str {
        "my_engine"
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            supports_vector_search: false,
            supports_full_text: true,
            // ... all capabilities
        }
    }

    async fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        // Setup connections, load indices, etc.
        self.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), Box<dyn Error>> {
        // Graceful cleanup
        self.initialized = false;
        Ok(())
    }
}
```

## Quick Reference

### Available Engines
1. Qdrant - Vector semantic search
2. FAISS - GPU billions-scale vectors
3. SCANN - Trillion-scale learned indexing
4. Tantivy - Single-node BM25 text
5. LNX - Distributed Raft text search
6. Toshi - Experimental distributed
7. MeiliSearch - Typo-tolerant UX
8. Julia NLP - Mathematical text analysis
9. MemoryBank - Multi-language FFI

### Key Modules
- `motores::core::traits` - Core SearchEngine trait
- `motores::core::types` - Shared types
- `motores::core::routing_ai` - Intelligent routing
- `motores::core::health_monitor` - Health checking
- `motores::factory` - Engine creation
- `motores::hybrid` - Multi-engine coordination

Remember: **Motor independence is paramount. When in doubt, isolate.**
