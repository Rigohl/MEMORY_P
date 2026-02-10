# Troubleshooting: High CPU Usage

> **Quick diagnostic and resolution guide for CPU overload issues**

## 🔍 Symptoms

- Server becomes unresponsive
- Search queries timeout
- High load average (>4.0 on 4-core system)
- `docker stats` shows CPU at 100%

---

## 📊 Diagnosis

### Step 1: Identify the Culprit

```bash
# Check overall system load
uptime
# Output: load average: 8.45, 6.32, 5.11

# Check per-container CPU usage
docker stats --no-stream

# Check specific processes
top -o %CPU
# Press '1' to see per-core usage

# Check which container is the problem
docker ps --format "table {{.Names}}\t{{.CPUPerc}}\t{{.MemPerc}}"
```

### Step 2: Analyze Logs

```bash
# Check for errors in MEMORY_P
docker logs memory-p-app --tail 100

# Check for slow queries
docker logs memory-p-app | grep "SLOW QUERY"

# Check Qdrant logs
docker logs qdrant | grep -i "error\|warn"
```

### Step 3: Profile the Application

```bash
# Rust flamegraph (if running from source)
cargo flamegraph --bin memory_p

# Check syscalls
strace -c -p $(pgrep memory_p)

# Check I/O wait
iostat -x 1 10
```

---

## 🔧 Quick Fixes

### Fix 1: Restart Heavy Services

```bash
# Restart Qdrant (often the culprit for vector search)
docker-compose restart qdrant

# Wait 30 seconds for startup
sleep 30

# Verify health
curl http://localhost:6333/health
```

### Fix 2: Clear Caches

```bash
# Clear Redis cache
docker-compose exec redis redis-cli FLUSHDB

# Clear system cache (if running bare metal)
sync; echo 3 > /proc/sys/vm/drop_caches

# Restart MEMORY_P
docker-compose restart memory-p
```

### Fix 3: Adjust Resource Limits

```yaml
# Edit docker-compose.yml
services:
  memory-p:
    # Add CPU limits
    cpus: '2.0'  # Max 2 cores
    mem_limit: 8g
    
  qdrant:
    cpus: '1.5'  # Max 1.5 cores
    mem_limit: 6g
```

```bash
# Apply changes
docker-compose up -d
```

---

## 🎯 Root Cause Analysis

### Cause 1: Expensive Vector Search

**Symptoms**:
- High CPU during search operations
- Qdrant container at 100% CPU

**Solution**:
```toml
# config/production.toml
[qdrant]
# Reduce HNSW search precision
search_params = { ef = 64 }  # Default is 128

# Enable quantization (trades accuracy for speed)
quantization = "scalar"

# Limit concurrent searches
max_concurrent_searches = 4
```

### Cause 2: Unoptimized Queries

**Symptoms**:
- Tantivy at high CPU
- Slow full-text searches

**Solution**:
```rust
// Add query limits in code
let query = SearchQuery {
    limit: 100,  // Reduce from 1000
    timeout: Duration::from_millis(100),  // Add timeout
    ..default
};
```

### Cause 3: Julia Mathematical Operations

**Symptoms**:
- Julia process at 100% CPU
- Chaos analysis taking too long

**Solution**:
```julia
# FFI/JULIA_BRAIN/optimize_config.jl

# Reduce Julia threads
ENV["JULIA_NUM_THREADS"] = "2"  # From 4

# Use lower precision
setprecision(32)  # From 64

# Simplify chaos calculations
const MAX_ITERATIONS = 1000  # From 10000
```

### Cause 4: Too Many Concurrent Requests

**Symptoms**:
- Many connections in `netstat`
- Axum server threads maxed out

**Solution**:
```rust
// src/main.rs
let app = Router::new()
    .layer(
        ServiceBuilder::new()
            .layer(
                // Limit concurrent requests
                tower::limit::ConcurrencyLimitLayer::new(100)
            )
            .layer(
                // Rate limiting per IP
                tower::limit::RateLimitLayer::new(10, Duration::from_secs(1))
            )
    );
```

---

## 🚀 Performance Optimizations

### Optimization 1: Enable CPU Affinity

```bash
# Pin containers to specific cores
docker update --cpuset-cpus="0,1" memory-p-app
docker update --cpuset-cpus="2,3" qdrant
```

### Optimization 2: Optimize Rust Build

```bash
# Rebuild with CPU-specific optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release

# For ARM (Oracle Cloud)
RUSTFLAGS="-C target-cpu=neoverse-n1" cargo build --release
```

### Optimization 3: Use Rayon More Efficiently

```rust
// Reduce Rayon thread pool size
use rayon::ThreadPoolBuilder;

ThreadPoolBuilder::new()
    .num_threads(4)  // Match physical cores
    .build_global()
    .unwrap();
```

### Optimization 4: Database Connection Pooling

```toml
# config/production.toml
[database]
# Reduce PostgreSQL connections
max_connections = 10  # From 20

[redis]
# Reduce Redis pool
pool_size = 5  # From 10
```

---

## 📈 Monitoring Setup

### Prometheus Alerts

```yaml
# monitoring/prometheus/alerts.yml
groups:
  - name: cpu_alerts
    rules:
      - alert: HighCPUUsage
        expr: rate(process_cpu_seconds_total[5m]) > 0.8
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage detected"
          description: "CPU usage above 80% for 5 minutes"
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "CPU Usage",
    "panels": [
      {
        "title": "CPU by Container",
        "targets": [
          {
            "expr": "rate(container_cpu_usage_seconds_total[5m])"
          }
        ]
      }
    ]
  }
}
```

---

## 🔄 Long-Term Solutions

### Solution 1: Horizontal Scaling

```yaml
# kubernetes/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: memory-p
spec:
  replicas: 3  # Distribute load
  strategy:
    type: RollingUpdate
```

### Solution 2: Caching Layer

```rust
// Add LRU cache for expensive operations
use lru::LruCache;

lazy_static! {
    static ref QUERY_CACHE: Mutex<LruCache<String, Vec<SearchResult>>> =
        Mutex::new(LruCache::new(1000));
}

pub async fn cached_search(query: &str) -> Result<Vec<SearchResult>> {
    // Check cache first
    if let Some(results) = QUERY_CACHE.lock().unwrap().get(query) {
        return Ok(results.clone());
    }
    
    // Execute search
    let results = execute_search(query).await?;
    
    // Cache results
    QUERY_CACHE.lock().unwrap().put(query.to_string(), results.clone());
    
    Ok(results)
}
```

### Solution 3: Async Processing Queue

```rust
// Use message queue for heavy operations
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel(100);

// Producer
tx.send(SearchRequest::new(query)).await?;

// Consumer (separate task)
tokio::spawn(async move {
    while let Some(req) = rx.recv().await {
        process_search(req).await;
    }
});
```

---

## 📋 Prevention Checklist

- [ ] Set CPU limits in docker-compose.yml
- [ ] Enable query timeouts
- [ ] Implement rate limiting
- [ ] Monitor CPU usage with Prometheus
- [ ] Set up alerts for >80% CPU
- [ ] Regular performance profiling
- [ ] Cache frequently accessed data
- [ ] Optimize database queries
- [ ] Use connection pooling
- [ ] Review and optimize hot paths

---

## 🔗 Related Guides

- [Memory Leaks](./memory-leaks.md) - High memory usage
- [Slow Search Queries](./slow-search.md) - Query optimization
- [Performance Tuning](./performance-tuning.md) - General optimization

---

## 📞 Need Help?

If CPU usage remains high after these fixes:

1. Collect diagnostics:
```bash
./scripts/collect_diagnostics.sh
```

2. Open an issue with:
   - Output of `docker stats`
   - Flamegraph (if available)
   - Logs from last 1 hour
   - Configuration file

**Open issue**: https://github.com/Rigohl/MEMORY_P/issues/new
