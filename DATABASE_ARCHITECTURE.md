# Database Architecture - MEMORY_P v2.0

## 🗄️ Multi-Database Strategy

Cada motor usa la base de datos más adecuada para su tipo de operación, con opción de compartir cuando sea beneficioso.

---

## 📊 Database Assignments per Motor

### Vector Search Engines

#### 1. Qdrant Motor
**Primary DB**: **Qdrant Native Storage**
- Built-in vector database
- HNSW + Product Quantization
- gRPC/HTTP API
- Persistent on-disk storage

**Shared**: 
- PostgreSQL + pgvector (backup/export)
- Redis (query cache, <5min TTL)

**Why**: Optimizado para búsqueda vectorial a escala

---

#### 2. FAISS Motor
**Primary DB**: **In-Memory + RocksDB**
- FAISS indices en memoria (GPU/CPU)
- RocksDB para persistencia de metadatos
- Checkpoints periódicos a disco

**Shared**:
- PostgreSQL + pgvector (fallback search)
- Redis (hot vectors cache)

**Why**: FAISS es in-memory, RocksDB para durabilidad

---

#### 3. SCANN Motor
**Primary DB**: **LevelDB + Parquet**
- LevelDB para índices aprendidos
- Parquet para vectores (columnar)
- TensorFlow SavedModel para learned index

**Shared**:
- PostgreSQL + pgvector (training data)
- Redis (query cache)

**Why**: LevelDB eficiente para learned indices, Parquet para compresión

---

### Text Search Engines

#### 4. Tantivy Motor
**Primary DB**: **Tantivy Native Segments**
- Lucene-like segment storage
- FST (Finite State Transducers)
- Inverted index on disk
- Lock-free writes

**Shared**:
- Redis (autocomplete cache, facets)
- PostgreSQL (document metadata)

**Why**: Tantivy es self-contained, no necesita DB externa

---

#### 5. LNX Motor
**Primary DB**: **RocksDB + Raft Log**
- RocksDB para índices distribuidos
- Raft consensus log (etcd-like)
- Multi-node replication

**Shared**:
- Redis (coordination, leader election)
- PostgreSQL (cluster metadata)

**Why**: RocksDB óptimo para Raft, Redis para coordinación

---

#### 6. Toshi Motor
**Primary DB**: **Sled DB**
- Sled (Rust embedded DB)
- Experimental lock-free B+ trees
- Crash-safe MVCC

**Shared**:
- PostgreSQL (fallback)
- Redis (experimental cache)

**Why**: Sled es experimental como Toshi

---

#### 7. MeiliSearch Motor
**Primary DB**: **LMDB (Memory-Mapped)**
- Lightning Memory-Mapped Database
- Copy-on-write B+ trees
- Typo-tolerant FSTs

**Shared**:
- Redis (typo suggestions cache)
- PostgreSQL (synonym dictionaries)

**Why**: LMDB ultra-rápido para lecturas, ideal para typo-tolerance

---

### Specialized Engines

#### 8. Julia NLP Motor
**Primary DB**: **PostgreSQL + TimescaleDB**
- TimescaleDB para series temporales
- PostgreSQL para chaos analysis results
- JLD2 (Julia Data Format) para arrays

**Shared**:
- Redis (optimization cache)
- ClickHouse (analytics)

**Why**: TimescaleDB perfecto para análisis temporal, Julia nativo con JLD2

---

#### 9. MemoryBank Motor
**Primary DB**: **PostgreSQL + Multiple Extensions**
- pgvector (vectores)
- pg_trgm (fuzzy text)
- jsonb (flexible schemas)
- Full-text search (tsvector)

**Shared**: 
- Redis (unified cache layer)
- All other motors (dispatcher)

**Why**: PostgreSQL como "hub" central, accede a todos los motores

---

#### 10. Six Sigma Motor
**Primary DB**: **ClickHouse OLAP**
- Columnar storage
- Real-time aggregations
- Time-series metrics
- Statistical functions built-in

**Shared**:
- Redis (hot metrics, <1min)
- PostgreSQL (control limits, historical)

**Why**: ClickHouse diseñado para analytics en tiempo real

---

#### 11. ONNX Motor
**Primary DB**: **Redis + Object Storage**
- Redis (model cache, embeddings)
- MinIO/S3 (ONNX models storage)
- RocksDB (embedding index)

**Shared**:
- PostgreSQL (model metadata)
- Qdrant/FAISS (embeddings routing)

**Why**: Redis para baja latencia ML, S3 para models grandes

---

## 🔄 Event-Driven Continuous Architecture

### Eliminar Polling de 30 segundos → Event-Driven Continuo

#### Event Bus Central
**Technology**: **Apache Kafka / NATS Jetstream**

**Event Types**:
```rust
enum SystemEvent {
    // File System
    FileCreated(PathBuf),
    FileModified(PathBuf),
    FileDeleted(PathBuf),
    DirectoryChanged(PathBuf),
    
    // Search Operations
    SearchRequested(SearchQuery),
    SearchCompleted(SearchResult),
    IndexUpdateNeeded(EngineId),
    
    // Quality Events
    DefectDetected(DefectInfo),
    SigmaLevelChanged(f64),
    ControlLimitBreached(Metric),
    
    // Agent Collaboration
    AgentMessageSent(AgentId, Message),
    ContextUpdated(Context),
    MemoryStored(MemoryEntry),
    
    // System Health
    EngineHealthChanged(EngineId, HealthStatus),
    PerformanceDegraded(EngineId, Metrics),
    
    // Documentation
    CodeChanged(FilePath, ChangeType),
    SkillGenerated(Skill),
    AgentUpdated(Agent),
    
    // CI/CD
    TestFailed(TestInfo),
    BuildCompleted(BuildStatus),
    DeploymentTriggered(Environment),
}
```

---

### Reactive Triggers por Componente

#### 1. Auto-Index System
**Old**: Poll every 30s  
**New**: Event-driven continuous

```rust
// Subscribe to file system events
event_bus.subscribe("fs.*", |event| {
    match event {
        FileCreated(path) | FileModified(path) => {
            // Immediate indexing (< 100ms)
            auto_indexer.index_file(path).await;
        },
        FileDeleted(path) => {
            auto_indexer.remove_from_index(path).await;
        },
        DirectoryChanged(dir) => {
            auto_indexer.scan_directory(dir).await;
        }
    }
});

// Also subscribe to search patterns
event_bus.subscribe("search.completed", |event| {
    if event.result.is_miss() {
        // Proactive indexing of related content
        auto_indexer.expand_index(event.query).await;
    }
});
```

**Triggers**:
- ✅ `notify` crate (inotify/FSEvents) → <1ms latency
- ✅ Git hooks → on commit
- ✅ Search misses → reactive expansion
- ✅ User focus changes → background indexing

---

#### 2. Auto-Quality System
**Old**: Poll every operation  
**New**: Stream processing

```rust
// Real-time stream of all operations
event_bus.subscribe("motor.*.operation", |event| {
    let start = Instant::now();
    let result = event.result;
    let latency = start.elapsed();
    
    // Stream to Six Sigma engine
    six_sigma.record_operation(
        success: result.is_ok(),
        latency_ms: latency.as_millis(),
        motor: event.motor_id,
    );
    
    // Immediate alert if quality drops
    if six_sigma.sigma_level() < 4.0 {
        event_bus.publish(SystemEvent::QualityAlert {
            motor: event.motor_id,
            sigma: six_sigma.sigma_level(),
        });
    }
});
```

**Triggers**:
- ✅ Every operation (stream processing)
- ✅ Control limit breach → immediate alert
- ✅ Sigma level drop → automatic improvement
- ✅ Pattern detection → proactive optimization

---

#### 3. Auto-Optimize System
**Old**: Poll every 5 minutes  
**New**: Reactive + predictive

```rust
// Predictive optimization triggers
event_bus.subscribe("metrics.performance", |metrics| {
    if metrics.p99_latency > SLA * 0.8 {
        // 80% of SLA → proactive optimization
        optimizer.optimize_before_breach(metrics).await;
    }
});

// Reactive to degradation
event_bus.subscribe("engine.degraded", |event| {
    // Immediate optimization on degradation
    optimizer.emergency_optimize(event.engine_id).await;
});

// Learn from patterns
event_bus.subscribe("search.pattern", |pattern| {
    if pattern.frequency > THRESHOLD {
        // Pre-optimize common patterns
        optimizer.create_fast_path(pattern).await;
    }
});
```

**Triggers**:
- ✅ Performance approaching SLA (80% threshold)
- ✅ Degradation detected immediately
- ✅ Query patterns learned → pre-optimization
- ✅ Load spike predicted → scale proactively

---

#### 4. Auto-Backup System
**Old**: Poll every hour  
**New**: Event-driven + continuous

```rust
// Continuous WAL (Write-Ahead Log) backup
event_bus.subscribe("db.*.write", |event| {
    // Stream writes to WAL immediately
    backup_service.append_wal(event).await;
});

// Snapshot on significant events
event_bus.subscribe("system.milestone", |event| {
    match event {
        LargeIndexUpdate => backup_service.snapshot("index").await,
        ModelUpdated => backup_service.snapshot("models").await,
        ConfigChanged => backup_service.snapshot("config").await,
    }
});

// Incremental continuous backup (CDP)
backup_service.enable_continuous_data_protection();
```

**Triggers**:
- ✅ Every write → WAL (< 1ms overhead)
- ✅ Milestones → snapshots
- ✅ Crashes detected → automatic restore
- ✅ Corruption → rollback to last good state

---

#### 5. Auto-Update-Docs System
**Old**: Poll on commits  
**New**: Real-time on code changes

```rust
// Git hooks integration
event_bus.subscribe("git.pre-commit", |event| {
    // Update docs BEFORE commit
    doc_manager.update_affected_docs(event.changed_files).await;
    doc_manager.regenerate_tables().await;
});

// Real-time code analysis
event_bus.subscribe("code.changed", |event| {
    match event.file_type {
        "src/motores/*.rs" => {
            doc_manager.update_motors_table().await;
        },
        ".github/agents/*.agent.md" => {
            doc_manager.update_agents_table().await;
        },
        ".github/skills/*.skill.md" => {
            doc_manager.update_skills_table().await;
        }
    }
});
```

**Triggers**:
- ✅ Pre-commit hook → docs updated before commit
- ✅ File save → incremental doc update
- ✅ New motor → auto-add to matrix
- ✅ Feature implemented → update roadmap

---

## 📊 Database Technology Stack

### Databases Used

| Database | Motors Using | Purpose | Size Estimate |
|----------|-------------|---------|---------------|
| **PostgreSQL + pgvector** | Qdrant, FAISS, SCANN, Tantivy, LNX, Julia NLP, MemoryBank, ONNX | Backup vectors, metadata, relations | 10-100 GB |
| **Redis** | ALL (shared cache) | Hot cache, coordination, pub/sub | 1-10 GB |
| **ClickHouse** | Six Sigma, Julia NLP | Analytics, metrics, time-series | 5-50 GB |
| **RocksDB** | FAISS, LNX | Fast key-value, Raft log | 1-20 GB |
| **Qdrant Native** | Qdrant | Vector search storage | 10-100 GB |
| **Tantivy Segments** | Tantivy | Inverted index | 5-50 GB |
| **LMDB** | MeiliSearch | Memory-mapped fast reads | 1-10 GB |
| **LevelDB** | SCANN | Learned index storage | 5-20 GB |
| **Sled** | Toshi | Experimental embedded | 1-5 GB |
| **TimescaleDB** | Julia NLP | Time-series for chaos analysis | 1-10 GB |
| **Parquet Files** | SCANN | Columnar vector storage | 10-50 GB |
| **MinIO/S3** | ONNX | Model storage | 1-5 GB |

**Total Estimated**: 50-400 GB depending on data size

---

## 🔧 Dependencies to Add

```toml
# Event Bus
rdkafka = "0.36"              # Apache Kafka client
nats = "0.24"                 # NATS alternative

# Databases
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls"] }
pgvector = "0.3"              # PostgreSQL vector extension
clickhouse = "0.11"           # ClickHouse client
rocksdb = "0.21"              # RocksDB embedded
sled = "0.34"                 # Sled embedded DB
lmdb-rs = "0.15"              # LMDB bindings

# File System Events
notify = { version = "6.1", features = ["serde"] }

# WAL & Backup
wal = "0.1"                   # Write-Ahead Log
tempfile = "3.8"              # Temporary files for backups

# Stream Processing
async-stream = "0.3"          # Async stream utilities
futures-util = "0.3"          # Stream combinators
```

---

## 🚀 Performance Expectations

### Event-Driven vs Polling

| Metric | Polling (30s) | Event-Driven |
|--------|--------------|--------------|
| **Index Latency** | 0-30s | <100ms |
| **Quality Detection** | 30s average | <1ms |
| **Optimization Trigger** | 5min average | Immediate |
| **Backup Lag** | 1h | <10ms (WAL) |
| **Doc Update** | Next commit | Real-time |
| **CPU Usage** | 5-10% constant | 1-2% idle, burst on events |
| **Responsiveness** | Poor | Excellent |

### Database Performance

| Database | Read Latency | Write Latency | Throughput |
|----------|-------------|---------------|------------|
| Redis | <1ms | <1ms | 100K ops/sec |
| PostgreSQL | 1-5ms | 5-10ms | 10K ops/sec |
| ClickHouse | 10-50ms | 1-5ms | 1M rows/sec |
| RocksDB | <1ms | <1ms | 100K ops/sec |
| Qdrant | 5-20ms | 10-50ms | 1K vectors/sec |
| Tantivy | 5-10ms | 50-100ms | 10K docs/sec |

---

## 📐 Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Event Bus (Kafka/NATS)                    │
│  fs.* | search.* | quality.* | agent.* | system.* | ci.*    │
└──────────────────────┬──────────────────────────────────────┘
                       │
         ┌─────────────┴─────────────┐
         │                           │
         ▼                           ▼
┌─────────────────┐         ┌─────────────────┐
│  Auto-Indexer   │         │  Auto-Quality   │
│  (Reactive)     │         │  (Stream)       │
└────────┬────────┘         └────────┬────────┘
         │                           │
         ▼                           ▼
┌─────────────────────────────────────────────┐
│             Database Layer                   │
├──────────────┬──────────────┬────────────────┤
│  PostgreSQL  │  ClickHouse  │     Redis      │
│  (Relations) │  (Analytics) │    (Cache)     │
├──────────────┼──────────────┼────────────────┤
│   RocksDB    │   Qdrant    │    Tantivy     │
│  (KV Store)  │  (Vectors)  │    (Text)      │
└──────────────┴──────────────┴────────────────┘
         │                           │
         └────────────┬──────────────┘
                      ▼
         ┌─────────────────────────┐
         │    11 Search Motors     │
         │  (Event Subscribers)    │
         └─────────────────────────┘
```

---

## ✅ Implementation Priority

### Phase 1 (3 days)
- ✅ Event bus setup (Kafka or NATS)
- ✅ File system event subscription
- ✅ Convert auto-indexer to event-driven
- ✅ Redis shared cache layer

### Phase 2 (3 days)
- ✅ PostgreSQL + pgvector setup
- ✅ ClickHouse for Six Sigma
- ✅ Stream processing for quality
- ✅ Event-driven optimization

### Phase 3 (2 days)
- ✅ Specialized DBs (RocksDB, LMDB, etc.)
- ✅ WAL continuous backup
- ✅ Real-time doc updates
- ✅ Complete event integration

**Total**: 8 days for full event-driven multi-DB architecture

---

## 🎯 Benefits

### Event-Driven Architecture
- ✅ **100-300x faster response** (30s → <100ms)
- ✅ **Lower CPU usage** when idle
- ✅ **Immediate reactions** to changes
- ✅ **Predictive optimizations**
- ✅ **Better user experience**

### Multi-Database Strategy
- ✅ **Each motor uses optimal DB** for its workload
- ✅ **Shared Redis cache** reduces duplication
- ✅ **PostgreSQL as backup/hub** for reliability
- ✅ **Specialized performance** (10-100x per motor)
- ✅ **Horizontal scalability** per motor

---

**Total Estimated Storage**: 50-400 GB  
**Total Estimated Time**: 8 days implementation  
**Performance Gain**: 100-300x response time improvement  
**Status**: ✅ Architecture documented, ready for implementation
