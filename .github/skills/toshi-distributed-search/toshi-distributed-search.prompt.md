---
name: "toshi-distributed-search"
description: "Toshi experimental distributed search engine setup and optimization. Use when experimenting with distributed search, need custom replication strategies, or exploring alternative distributed architectures."
tools: ["codebase", "terminalCommand"]
---

# Toshi Distributed Search Engine Skill

## When to Use This Skill
- Experimental distributed search projects
- Custom replication strategy needs
- Alternative to LNX for comparison
- Learning distributed search architectures
- Prototype distributed indexing

## Prerequisites
- Rust toolchain installed
- Multiple nodes available for cluster
- Network connectivity between nodes
- Understanding of distributed systems concepts

## Setup Steps

### 1. Install Toshi (Conceptual - as it's experimental)
```bash
# Toshi is experimental and would need custom build
cargo install toshi
```

### 2. Configure Cluster
```toml
[cluster]
nodes = ["node1:8080", "node2:8080", "node3:8080"]
replication_factor = 2
shard_count = 4

[storage]
data_directory = "/var/lib/toshi"
max_memory_mb = 2048
```

### 3. Initialize Distributed Index
```rust
use memory_p::motores::text_search::ToshiEngine;
use memory_p::motores::core::types::*;

let config = EngineConfig {
    name: "toshi".to_string(),
    enabled: true,
    endpoints: vec!["http://node1:8080".to_string()],
    database: DatabaseConfig {
        storage_type: "toshi_native".to_string(),
        storage_path: "/var/lib/toshi/data".to_string(),
        postgres_schema: Some("motor_toshi".to_string()),
        metadata_storage: Some("postgresql".to_string()),
    },
    // ... other config
};

let engine = ToshiEngine::new(config);
```

## Performance Considerations
- Experimental status - use for testing only
- Replication can be resource intensive
- Monitor shard distribution across nodes
- Consider network latency between nodes

## Comparison with LNX
- **Toshi**: More experimental, fewer features
- **LNX**: Production-ready, Raft consensus
- **Use Toshi for**: Learning and experimentation
- **Use LNX for**: Production workloads

## Monitoring Toshi
```rust
let health = engine.health().await?;
let metrics = engine.metrics().await?;

if let Ok(cluster_info) = engine.cluster_info().await {
    println!("Nodes: {}", cluster_info.node_count);
}
```
