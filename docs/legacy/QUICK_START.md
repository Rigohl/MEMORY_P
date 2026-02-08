# ⚡ Quick Integration Guide - MEMORY_P Learning System

## 🚀 5-Minute Integration

### Step 1: Add Dependency

```toml
# Cargo.toml
[dependencies]
memory_p = "0.2.0"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Step 2: Initialize System

```rust
// src/main.rs
use memory_p::auto_manager::{AutoManager, ManagerConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logging
    tracing_subscriber::fmt::init();
    
    // Create auto manager
    let auto_manager = Arc::new(AutoManager::new(ManagerConfig::default()));
    
    // Start always-on system
    auto_manager.auto_start().await?;
    auto_manager.start_continuous_learning().await;
    
    println!("✅ MEMORY_P Learning System running!");
    
    // Your application code here...
    
    Ok(())
}
```

### Step 3: Run

```bash
cargo run --release
```

**That's it!** 🎉 The system is now:
- ✅ Auto-managing itself
- ✅ Detecting and fixing issues
- ✅ Learning user patterns
- ✅ Optimizing parameters

---

## 📊 Optional: Add Monitoring

```rust
use memory_p::telemetry::{TelemetrySystem, TelemetryConfig};

// Add telemetry
let telemetry = Arc::new(TelemetrySystem::new(TelemetryConfig::default()));
telemetry.start().await?;

// Get metrics anytime
let metrics = telemetry.get_metrics_snapshot().await;
println!("Requests: {}, Success: {:.1}%", 
    metrics.total_requests,
    metrics.success_rate
);
```

---

## 🔍 Optional: Add Pattern Detection

```rust
use memory_p::pattern_detector::{PatternDetector, UserAction};
use chrono::Utc;

let detector = Arc::new(PatternDetector::new());

// Record user actions
detector.record_action("user_id", UserAction {
    timestamp: Utc::now(),
    action_type: "edit".to_string(),
    tool: "vscode".to_string(),
    language: Some("rust".to_string()),
    success: true,
    duration_secs: 45.0,
}).await;

// Get patterns
let patterns = detector.detect_patterns("user_id").await?;
println!("Confidence: {:.0}%", patterns.confidence * 100.0);
```

---

## 🔧 Configuration

### Minimal Config
```rust
use std::time::Duration;

let config = ManagerConfig {
    check_interval: Duration::from_secs(30),
    max_errors: 3,
    recovery_timeout: Duration::from_secs(10),
    auto_restart: true,
};
```

### Production Config
```rust
let config = ManagerConfig {
    check_interval: Duration::from_secs(15),  // More frequent
    max_errors: 5,                            // More tolerant
    recovery_timeout: Duration::from_secs(30),// Longer timeout
    auto_restart: true,
};
```

---

## 📈 Access Metrics

```rust
// Real-time metrics
let metrics = auto_manager.get_realtime_metrics().await;
println!("Accuracy: {:.1}%", metrics.prediction_accuracy * 100.0);
println!("Learning Velocity: {:.3}", metrics.learning_velocity);

// Generate report
let report = auto_manager.generate_learning_report().await;
println!("{}", report);
```

---

## 🔍 Run Diagnostics

```rust
// Run diagnostics
let diagnostics = auto_manager.run_predictive_diagnostics().await?;

for diag in diagnostics {
    if !diag.issues.is_empty() {
        println!("⚠️  {} has {} issues", diag.component, diag.issues.len());
    }
}
```

---

## 🔧 Auto-correct Issues

```rust
// Auto-correct specific component
let result = auto_manager
    .run_chaos_based_autocorrection("component_name")
    .await?;

if result.success {
    println!("✅ Corrected in {:.2}ms", result.duration_ms);
}
```

---

## 🎯 Full Example

```rust
use memory_p::auto_manager::{AutoManager, ManagerConfig};
use memory_p::telemetry::{TelemetrySystem, TelemetryConfig};
use memory_p::pattern_detector::PatternDetector;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    // 1. Initialize components
    let telemetry = Arc::new(TelemetrySystem::new(TelemetryConfig::default()));
    let pattern_detector = Arc::new(PatternDetector::new());
    let auto_manager = Arc::new(AutoManager::new(ManagerConfig::default()));
    
    // 2. Start systems
    telemetry.start().await?;
    auto_manager.auto_start().await?;
    auto_manager.start_continuous_learning().await;
    
    println!("✅ All systems running!");
    
    // 3. Background monitoring
    let am = auto_manager.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(60));
        loop {
            ticker.tick().await;
            
            // Auto-diagnostics and correction
            if let Ok(diagnostics) = am.run_predictive_diagnostics().await {
                for diag in diagnostics {
                    if !diag.issues.is_empty() && diag.issues[0].auto_correctable {
                        let _ = am.run_chaos_based_autocorrection(&diag.component).await;
                    }
                }
            }
            
            // Optimize parameters every 5 minutes
            if ticker.period().as_secs() % 300 == 0 {
                let _ = am.optimize_adaptive_parameters().await;
            }
        }
    });
    
    // 4. Your application logic here
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // System is self-managing while your app runs
    }
}
```

---

## 🐳 Docker Integration

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/memory_p /usr/local/bin/
CMD ["memory_p"]
```

```yaml
# docker-compose.yml
version: '3.8'
services:
  memory_p:
    build: .
    ports:
      - "9090:9090"  # Prometheus
    environment:
      - RUST_LOG=info
      - CLICKHOUSE_URL=http://clickhouse:8123
    depends_on:
      - postgres
      - redis
      - clickhouse
  
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: memory_p
  
  redis:
    image: redis:7-alpine
  
  clickhouse:
    image: clickhouse/clickhouse-server:latest
```

---

## 🧪 Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test '*'

# Benchmarks
cargo bench

# With logging
cargo test -- --nocapture
```

---

## 📚 Next Steps

1. Read full docs: [LEARNING_SYSTEM.md](LEARNING_SYSTEM.md)
2. See use cases: [USE_CASES.md](USE_CASES.md)
3. Deep dive: [auto_manager.md](auto_manager.md)
4. Join community: [GitHub Discussions](https://github.com/memory-p/discussions)

---

## 🆘 Troubleshooting

### System not starting?
```rust
// Add detailed logging
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

### Julia not available?
```
Don't worry! The system uses fallback heuristics automatically.
Julia is optional for advanced chaos analysis.
```

### High memory usage?
```rust
// Reduce cache size
let config = TelemetryConfig {
    batch_size: 500,  // Smaller batches
    ..Default::default()
};
```

---

## ✅ Checklist

- [ ] Dependencies added
- [ ] AutoManager initialized
- [ ] System started
- [ ] Logging configured
- [ ] Metrics accessible
- [ ] Tests passing
- [ ] Ready for production! 🚀

---

🧠 **"The simplest integration, the most powerful system."** 🧠

Questions? Open an issue on GitHub!
