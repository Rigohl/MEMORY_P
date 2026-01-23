---
name: "9-motor-coordination"
description: "Advanced coordination of all 9 search engines in MEMORY_P. Use when implementing multi-engine search strategies, optimizing cross-engine performance, or building intelligent routing systems."
tools: ["codebase", "terminalCommand"]
---

# 9-Motor Coordination Skill

## Motor Categories

### Vector Search Engines (3)
- **Qdrant**: Semantic similarity, real-time updates (<100ms for <1M vectors)
- **FAISS-GPU**: Billions-scale GPU acceleration (<50ms)
- **SCANN**: Google-grade trillion-scale enterprise (<200ms)

### Text Search Engines (4)
- **Tantivy**: Single-node BM25 champion (<10ms)
- **LNX**: Production distributed with Raft (<150ms)
- **Toshi**: Experimental distributed testing (<300ms acceptable)
- **MeiliSearch**: User-friendly typo-tolerant (<80ms)

### Specialized Engines (2)
- **Julia NLP**: Mathematical text analysis (<500ms)
- **MemoryBank**: Multi-language FFI innovation (<200ms)

## Coordination Strategies

### Basic Multi-Engine Setup
```rust
use memory_p::motores::{
    factory::EngineFactory,
    hybrid::FusionEngine,
    core::{HealthMonitor, RoutingAI, types::*},
};
use std::sync::Arc;
use std::time::Duration;

// Create health monitor
let health_monitor = Arc::new(HealthMonitor::new(Duration::from_secs(30)));

// Create fusion engine for coordination
let fusion = Arc::new(FusionEngine::new());

// Initialize all 9 engines
let engine_names = EngineFactory::available_engines();

for name in engine_names {
    let config = create_engine_config(name);
    let mut engine = EngineFactory::create_engine(name, config)?;
    
    engine.initialize().await?;
    
    let engine_arc = Arc::clone(&engine);
    fusion.register_engine(name.to_string(), engine_arc.clone()).await;
    health_monitor.register_engine(name.to_string(), engine_arc).await;
}

// Start background health checking
health_monitor.clone().start_background_checks();
```

### Query Routing Algorithm
```rust
use memory_p::motores::core::{RoutingAI, types::*};

let router = RoutingAI::new();

// Route a semantic search query
let query = SearchQuery {
    text: "find similar documents about machine learning".to_string(),
    vector: Some(embedding_vector),
    query_type: QueryType::Vector,
    limit: 10,
    offset: 0,
    filters: HashMap::new(),
    min_score: 0.7,
};

// Get optimal engine selection
let engines = router.route_query(&query);
// Returns: Primary("qdrant"), Fallback("faiss")
```

### Multi-Engine Search
```rust
// Perform search across multiple engines
let results = fusion.search_multi(&query).await?;

// Results are merged from multiple engines
for result in results {
    println!("Doc: {} from {} (score: {})", 
        result.id, 
        result.engine,
        result.score
    );
}
```

### Health Monitoring
```rust
// Check system health
let system_health = health_monitor.get_system_health().await;

println!("Healthy engines: {}/{}", 
    system_health.healthy_engines,
    system_health.total_engines
);

// Check specific engine
if let Some(health) = health_monitor.check_engine("qdrant").await {
    if !health.healthy {
        eprintln!("Qdrant is unhealthy: {}", health.status);
    }
}
```

## Performance Optimization

### 1. Engine-Specific Tuning
```rust
match engine_name {
    "faiss" => {
        // GPU-specific optimizations
        config.settings.insert("use_gpu".to_string(), json!(true));
        config.limits.max_latency_ms = 50;
    },
    "tantivy" => {
        // Single-node optimizations
        config.limits.max_concurrent_queries = 1000;
        config.limits.max_latency_ms = 10;
    },
    "scann" => {
        // Trillion-scale optimizations
        config.limits.max_latency_ms = 200;
        config.settings.insert("quantization".to_string(), json!("anisotropic"));
    },
    _ => {}
}
```

### 2. Load Balancing
```rust
use memory_p::motores::hybrid::LoadBalancer;

let load_balancer = LoadBalancer::new();

// Select engine with lowest load
let candidates = vec!["qdrant".to_string(), "faiss".to_string()];
if let Some(engine) = load_balancer.select_engine(&candidates) {
    println!("Using engine: {}", engine);
}
```

### 3. Fallback Strategy
```rust
async fn search_with_fallback(
    query: &SearchQuery,
    fusion: &FusionEngine,
) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    // Try primary engine
    match fusion.search_multi(query).await {
        Ok(results) if !results.is_empty() => Ok(results),
        _ => {
            // Fallback to secondary engines
            eprintln!("Primary failed, trying fallback");
            // Retry with different routing
            fusion.search_multi(query).await
        }
    }
}
```

## Database Integration

### PostgreSQL Per-Motor Queries
```sql
-- Query Qdrant collections
SELECT * FROM motor_qdrant.collections WHERE name = 'documents';

-- Check Toshi shards
SELECT * FROM motor_toshi.shards WHERE document_count > 1000;

-- Monitor all motors
SELECT motor_name, healthy, checked_at 
FROM public.motor_health 
ORDER BY checked_at DESC 
LIMIT 100;
```

### ClickHouse Analytics
```sql
-- Average latency per motor
SELECT 
    motor_name,
    operation_type,
    avg(latency_ms) AS avg_latency
FROM analytics.motor_performance
WHERE timestamp > now() - INTERVAL 1 HOUR
GROUP BY motor_name, operation_type
ORDER BY avg_latency DESC;

-- Query success rates
SELECT 
    motor_name,
    success_rate
FROM analytics.query_success_rate
WHERE timestamp > now() - INTERVAL 24 HOUR;
```

## Best Practices

1. **Always use RoutingAI** for engine selection
2. **Monitor health continuously** with HealthMonitor
3. **Use FusionEngine** for coordinated searches
4. **Implement fallback strategies** for reliability
5. **Track metrics** in ClickHouse for optimization
6. **Isolate schemas** in PostgreSQL per motor
7. **Test with Toshi** before LNX production deployment
8. **GPU acceleration** for FAISS when available
9. **Mathematical analysis** delegate to Julia NLP
10. **Multi-language** coordination via MemoryBank

## Troubleshooting

### Engine Not Responding
```rust
// Check engine health
let health = health_monitor.check_engine("problematic_engine").await;
if let Some(h) = health {
    if !h.healthy {
        // Attempt restart or use fallback
        println!("Engine unhealthy: {:?}", h.details);
    }
}
```

### Performance Degradation
```rust
// Check metrics
let metrics = engine.metrics().await?;
if metrics.avg_query_latency_ms > expected_sla {
    // Scale up or redistribute load
    eprintln!("Latency exceeds SLA: {}ms", metrics.avg_query_latency_ms);
}
```

## Example: Complete Integration
```rust
use memory_p::motores::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup all 9 motors
    let health_monitor = Arc::new(HealthMonitor::default());
    let fusion = Arc::new(FusionEngine::new());
    let router = RoutingAI::new();
    
    // Initialize engines
    for name in EngineFactory::available_engines() {
        let config = create_config(name);
        let mut engine = EngineFactory::create_engine(name, config)?;
        engine.initialize().await?;
        
        let arc = Arc::clone(&engine);
        fusion.register_engine(name.to_string(), arc.clone()).await;
        health_monitor.register_engine(name.to_string(), arc).await;
    }
    
    // Start monitoring
    health_monitor.clone().start_background_checks();
    
    // Perform intelligent search
    let query = SearchQuery { /* ... */ };
    let results = fusion.search_multi(&query).await?;
    
    println!("Found {} results across all engines", results.len());
    
    Ok(())
}
```
