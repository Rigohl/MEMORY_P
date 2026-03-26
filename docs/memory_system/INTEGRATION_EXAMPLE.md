# Memory MCP Integration Example

This example shows how to integrate the advanced memory MCP system into your MEMORY_P server.

## Quick Start

### 1. Add Memory Routes to Your Server

```rust
use memory_p::mcp::memory_handlers::*;
use memory_p::mcp::memory_models::MemoryEngineConfig;
use axum::{Router, routing::{get, post}};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Initialize memory state
    let config = MemoryEngineConfig::default();
    let memory_state = Arc::new(MemoryState::new(config));

    // Create router with memory endpoints
    let app = Router::new()
        // Existing routes...
        .route("/mcp/memory/store", post(store_context_handler))
        .route("/mcp/memory/context/:id", get(get_context_handler))
        .route("/mcp/memory/predict", post(predict_next_handler))
        .route("/mcp/memory/reorder", post(reorder_contexts_handler))
        .route("/mcp/memory/cleanup", post(cleanup_stale_handler))
        .route("/mcp/memory/stats", get(get_stats_handler))
        .with_state(memory_state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Memory MCP server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
```

### 2. Using the Memory API

#### Store a Context

```bash
curl -X POST http://localhost:3000/mcp/memory/store \
  -H "Content-Type: application/json" \
  -d '{
    "content": "User prefers dark mode and TypeScript over JavaScript",
    "embedding": [0.1, 0.2, 0.3],
    "metadata": {
      "source": "user_preferences",
      "priority": "high"
    }
  }'

# Response:
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "stored"
}
```

#### Predict Next Contexts

```bash
curl -X POST http://localhost:3000/mcp/memory/predict \
  -H "Content-Type: application/json" \
  -d '{
    "context_id": "550e8400-e29b-41d4-a716-446655440000",
    "lookahead": 3
  }'

# Response:
{
  "predicted_contexts": [
    {
      "id": "...",
      "content": "Last used TypeScript config...",
      "prediction_score": 0.92
    }
  ],
  "confidence": 0.85,
  "computation_time_ms": 7,
  "predictor_used": "heuristic"
}
```

#### Auto-Reorder by Strategy

```bash
curl -X POST http://localhost:3000/mcp/memory/reorder \
  -H "Content-Type: application/json" \
  -d '{"strategy": "combined"}'

# Response:
{
  "reordered": 127,
  "strategy": "combined"
}
```

#### Get Statistics

```bash
curl http://localhost:3000/mcp/memory/stats

# Response:
{
  "total_contexts": 1523,
  "cache_hit_rate": 0.87,
  "avg_prediction_time_ms": 7.3,
  "total_predictions": 4512,
  "total_events": 8924
}
```

### 3. Advanced: Enable FFI Predictors

To enable Julia chaos analysis or MOJO inference:

```rust
let config = MemoryEngineConfig {
    enable_julia: true,  // Requires Julia installed
    enable_mojo: false,  // Requires MOJO SDK
    enable_zig_buffers: false,  // Optional zero-copy buffers
    prediction_cache_size: 1000,
    max_context_age_hours: 24,
    auto_cleanup_interval_secs: 3600,
};

let memory_state = Arc::new(MemoryState::new(config));
```

### 4. Using with PostgreSQL (Optional)

For persistent storage, set up PostgreSQL:

```bash
# Apply migrations
psql -d your_database -f migrations/001_memory_system.sql

# Set environment variable
export DATABASE_URL=postgresql://user:pass@localhost/your_database

# Update config to use database (future feature)
```

## Integration Patterns

### Pattern 1: Pre-load Context Before Agent Action

```rust
async fn handle_agent_request(
    memory: &PredictiveMemoryEngine,
    current_context_id: Uuid
) -> Result<Vec<MemoryContext>> {
    // Get current context
    let current = memory.get_context(current_context_id).await?
        .ok_or("Context not found")?;

    // Predict and pre-load next contexts
    let prediction = memory.predict_next(&current, 5).await?;

    // Contexts are now in cache, ready for fast access
    Ok(prediction.predicted_contexts)
}
```

### Pattern 2: Automatic Context Cleanup

```rust
// In your background task
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;

        // Clean contexts older than 24 hours
        let threshold = chrono::Duration::hours(24);
        let removed = memory.cleanup_stale(threshold).await?;

        println!("Cleaned {} stale contexts", removed);
    }
});
```

### Pattern 3: Smart Context Reordering

```rust
// Reorder daily based on usage patterns
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(86400)).await;

        // Use combined strategy (access count + prediction score)
        memory.auto_reorder(ReorderStrategy::Combined).await?;

        let stats = memory.get_stats().await?;
        println!("Cache hit rate: {:.2}%", stats.cache_hit_rate * 100.0);
    }
});
```

## Performance Tips

1. **Cache Size**: Increase `prediction_cache_size` for better hit rates
   - Default: 1000
   - Recommended for production: 5000-10000

2. **Cleanup Threshold**: Balance memory usage vs context retention
   - Default: 24 hours
   - Adjust based on your use case

3. **Prediction Lookahead**: Smaller values = faster predictions
   - Default: 5
   - For real-time: 3
   - For batch: 10

4. **Reorder Strategy**: Choose based on access patterns
   - `MostAccessed`: For frequently reused contexts
   - `MostRecent`: For time-sensitive contexts
   - `HighestScore`: For ML-optimized ordering
   - `Combined`: Best all-around (recommended)

## Monitoring

Check memory system health:

```bash
# Get stats every 10 seconds
while true; do
  curl -s http://localhost:3000/mcp/memory/stats | jq '.cache_hit_rate'
  sleep 10
done
```

## Troubleshooting

### High Memory Usage
```rust
// Reduce cache size
let config = MemoryEngineConfig {
    prediction_cache_size: 500,  // Reduced from 1000
    max_context_age_hours: 12,   // More aggressive cleanup
    ..Default::default()
};
```

### Slow Predictions
```rust
// Disable FFI predictors
let config = MemoryEngineConfig {
    enable_julia: false,
    enable_mojo: false,
    ..Default::default()
};
```

### Low Cache Hit Rate
```rust
// Increase cache and use better strategy
let config = MemoryEngineConfig {
    prediction_cache_size: 5000,
    ..Default::default()
};

// Use combined reordering more frequently
memory.auto_reorder(ReorderStrategy::Combined).await?;
```

## Next Steps

- Read the [full documentation](../docs/memory_system/MEMORY_MCP_GUIDE.md)
- Check out the [API reference](../docs/memory_system/MEMORY_MCP_GUIDE.md#api-reference)
- Explore [FFI integration](../docs/memory_system/MEMORY_MCP_GUIDE.md#ffi-integration)
- Review [benchmarks](../docs/memory_system/MEMORY_MCP_GUIDE.md#performance-benchmarks)
