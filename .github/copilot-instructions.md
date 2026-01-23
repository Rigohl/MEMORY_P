---
description: 'Instrucciones específicas de desarrollo para MEMORY_P v2.0'
applyTo: '**/*.rs, **/*.jl, **/*.py, **/*.mojo, **/*.pony, **/*.zig'
---

# MEMORY_P v2.0 Development Instructions

**Servidor MCP Always-On con Cerebro Matemático Multi-Lenguaje**

---

## 🎯 Filosofía del Proyecto

MEMORY_P v2.0 es un servidor MCP revolucionario que combina 6 lenguajes de programación para crear el sistema más avanzado del mundo:

- **Rust**: Orquestación, seguridad de memoria, paralelismo
- **Julia**: Matemáticas avanzadas, teoría del caos, optimización
- **JAX/Python**: Machine learning, embeddings, reinforcement learning
- **Mojo**: Kernels SIMD, performance extremo
- **Pony**: Sistema de actores distribuidos, fault tolerance
- **Zig**: FFI bridges, zero-copy operations

---

## 📐 Multi-Language Stack Rules

### Rust (Core Orchestration)

#### Always Use
- ✅ **Async/Await**: `tokio::spawn`, `async fn`, `.await`
- ✅ **Parallelism**: `rayon::par_iter()` para procesamiento masivo
- ✅ **Error Handling**: `Result<T, E>`, `?` operator, `thiserror` para errores custom
- ✅ **Zero Warnings**: `cargo clippy -- -D warnings`
- ✅ **Documentation**: `///` rustdoc en todas las APIs públicas
- ✅ **Testing**: Tests unitarios + integration tests

#### Never Do
- ❌ **No `unwrap()` en production** - usar `expect()` con mensaje descriptivo
- ❌ **No `panic!()` en FFI boundaries** - siempre retornar `Result`
- ❌ **No blocking operations en async** - usar `spawn_blocking`
- ❌ **No locks innecesarios** - preferir lock-free structures (`DashMap`, `crossbeam`)

```rust
// ✅ GOOD
pub async fn process_data(data: &[f64]) -> Result<Vec<f64>, ProcessError> {
    let result = tokio::task::spawn_blocking(move || {
        data.par_iter()
            .map(|x| x * 2.0)
            .collect()
    }).await?;
    
    Ok(result)
}

// ❌ BAD
pub fn process_data(data: &[f64]) -> Vec<f64> {
    data.iter().map(|x| x.unwrap() * 2.0).collect()  // unwrap, no async, no parallel
}
```

### Julia (Mathematical Brain)

#### Always Use
- ✅ **Type Annotations**: Para performance y claridad
- ✅ **Broadcasting**: `.` operator para vectorización
- ✅ **DifferentialEquations.jl**: Para EDOs y sistemas dinámicos
- ✅ **DynamicalSystems.jl**: Para análisis de caos
- ✅ **Optim.jl**: Para optimización numérica
- ✅ **Documentation**: Docstrings con ejemplos

#### Optimization Guidelines
```julia
# ✅ GOOD - Type-stable, vectorized
function analyze_chaos(data::Vector{Float64})::Float64
    ds = reconstruct_system(data)
    λ = lyapunov(ds, 10000)
    return λ
end

# ❌ BAD - Type-unstable, scalar operations
function analyze_chaos(data)
    result = 0.0
    for x in data
        result += process(x)  # Scalar loop, no type info
    end
    return result
end
```

### Python/JAX (ML Engine)

#### Always Use
- ✅ **Type Hints**: `from typing import ...`
- ✅ **JAX JIT**: `@jax.jit` para funciones computacionales
- ✅ **Flax/Optax**: Para redes neuronales y optimización
- ✅ **Numpy-style**: Operaciones vectorizadas
- ✅ **Error Handling**: Try/except con tipos específicos

```python
# ✅ GOOD
import jax
import jax.numpy as jnp

@jax.jit
def generate_embedding(input_ids: jnp.ndarray) -> jnp.ndarray:
    """Generate embeddings with JIT compilation."""
    outputs = model(input_ids)
    return jnp.mean(outputs.last_hidden_state, axis=1)

# ❌ BAD
def generate_embedding(input_ids):
    return model(input_ids).mean()  # No JIT, no types, no doc
```

### Mojo (SIMD Kernels)

#### Performance Guidelines
- ✅ **Vectorize**: `@parameter fn` con SIMD width
- ✅ **Memory Alignment**: 32-byte aligned buffers
- ✅ **Zero-Copy**: `UnsafePointer` para FFI
- ✅ **Inline Functions**: `@always_inline` en hot paths

```mojo
# ✅ GOOD
fn simd_optimize(data: UnsafePointer[Float64], len: Int) -> UnsafePointer[Float64]:
    let result = UnsafePointer[Float64].alloc(len)
    
    @parameter
    fn vectorized_op[simd_width: Int](idx: Int):
        let vec = data.offset(idx).load[width=simd_width]()
        let optimized = vec * 2.0 + 1.0
        result.offset(idx).store(optimized)
    
    vectorize[vectorized_op, 8](len)  # AVX-512
    return result
```

### Pony (Actor System)

#### Concurrency Rules
- ✅ **Capabilities**: `iso`, `trn`, `ref`, `val`, `box`, `tag`
- ✅ **Actors**: Isolated state, message passing
- ✅ **Behaviors**: `be` keyword para async operations
- ✅ **Zero Data Races**: Guaranteed by type system

```pony
actor SearchWorker
  let _id: USize
  let _collector: ResultCollector
  
  new create(id: USize, collector: ResultCollector) =>
    _id = id
    _collector = collector
  
  be search_range(query: Query val, start: USize, end: USize) =>
    let results = perform_search(query, start, end)
    _collector.collect(_id, consume results)
```

### Zig (FFI Bridge)

#### Memory Safety in FFI
- ✅ **Manual Memory Management**: Explícito allocate/free
- ✅ **Error Handling**: `!` error union type
- ✅ **C Interop**: `extern` functions para FFI
- ✅ **Zero-Copy**: `@bitCast`, `@ptrCast` cuando sea seguro

```zig
// ✅ GOOD
export fn zig_preprocess(data: [*]const f64, len: usize) ![*]f64 {
    const aligned = try std.heap.c_allocator.alloc(f64, len);
    @memcpy(aligned.ptr, data, len * @sizeOf(f64));
    return aligned.ptr;
}
```

---

## 🏗️ Architecture Patterns

### Always-On Background Processing

El sistema debe ejecutar tareas continuamente en segundo plano:

```rust
pub async fn start_always_on_system() -> Result<()> {
    // Spawn all background tasks
    let handles = vec![
        tokio::spawn(filesystem_monitoring()),
        tokio::spawn(mathematical_predictions()),
        tokio::spawn(ml_inference_pipeline()),
        tokio::spawn(distributed_search()),
        tokio::spawn(learning_system()),
        tokio::spawn(performance_optimization()),
        tokio::spawn(chaos_analysis()),
        tokio::spawn(context_streaming()),
    ];
    
    // Never terminates (always-on mode)
    futures::future::join_all(handles).await;
    Ok(())
}
```

### Mathematical Decision Making

Las decisiones deben estar basadas en matemáticas, NO en heurísticas:

```julia
# ✅ GOOD - Mathematical foundation
function decide_optimization_strategy(metrics::Vector{Metric})::Strategy
    # Analyze with chaos theory
    λ = lyapunov_exponent(metrics)
    
    if λ > 0.5
        return :aggressive_refactoring
    elseif λ > 0.0
        return :gradual_improvement
    else
        return :maintain_current
    end
end

# ❌ BAD - Arbitrary heuristic
function decide_optimization_strategy(metrics)
    if length(metrics) > 100  # Magic number
        return :refactor
    else
        return :maintain
    end
end
```

### Multi-Engine Search Coordination

Coordinar 4 motores de búsqueda con fusión matemática:

```rust
pub async fn hybrid_search(query: &str) -> Result<Vec<SearchResult>> {
    // Parallel search across 4 engines
    let (qdrant, tantivy, memorybank, _) = tokio::join!(
        search_qdrant(query),
        search_tantivy(query),
        search_memorybank(query),
        search_hybrid_julia(query)  // Mathematical fusion
    );
    
    // Julia fusion algorithm
    let fused = julia_reciprocal_rank_fusion(vec![
        qdrant?,
        tantivy?,
        memorybank?
    ])?;
    
    Ok(fused)
}
```

### Continuous Learning Integration

Integrar aprendizaje en cada operación:

```rust
pub async fn execute_with_learning<T>(
    task: Task,
    executor: impl Fn(Task) -> Result<T>
) -> Result<T> {
    // 1. Retrieve similar past episodes
    let episodes = episodic_memory.retrieve_similar(&task.context, 5).await?;
    
    // 2. Optimize approach based on learning
    let optimized_approach = learning_system
        .optimize_approach(&task, &episodes)
        .await?;
    
    // 3. Execute
    let result = executor(task)?;
    
    // 4. Store for future learning
    episodic_memory.store_episode(task, result.clone()).await?;
    
    Ok(result)
}
```

### FFI Zero-Copy Between Languages

Transferir datos sin overhead de serialización:

```rust
// Rust → Julia (zero-copy)
pub fn rust_to_julia_zerocopy(data: &[f64]) -> Result<f64> {
    unsafe {
        let ptr = data.as_ptr();
        let len = data.len();
        
        // Julia wraps pointer directly
        let result = julia_ffi::analyze_chaos(ptr, len);
        Ok(result)
    }
}

// Rust → Mojo (zero-copy via Zig)
pub fn rust_to_mojo_zerocopy(data: &[f64]) -> Result<Vec<f64>> {
    unsafe {
        let ptr = data.as_ptr();
        let len = data.len();
        
        // Zig bridge → Mojo SIMD
        let result_ptr = zig_ffi::simd_optimize(ptr, len);
        let result = Vec::from_raw_parts(result_ptr, len, len);
        Ok(result)
    }
}
```

---

## 🔧 Code Quality Standards

### All Async Operations Must Be Cancellable

```rust
// ✅ GOOD
pub async fn cancellable_operation(
    cancel_token: CancellationToken
) -> Result<()> {
    loop {
        select! {
            _ = cancel_token.cancelled() => {
                info!("Operation cancelled gracefully");
                return Ok(());
            }
            result = do_work() => {
                handle_result(result)?;
            }
        }
    }
}
```

### All Mathematical Operations Must Be Vectorized

```julia
# ✅ GOOD - Vectorized
function process_batch(data::Matrix{Float64})::Matrix{Float64}
    return data .* 2.0 .+ 1.0  # Broadcasting
end

# ❌ BAD - Scalar loops
function process_batch(data)
    result = similar(data)
    for i in eachindex(data)
        result[i] = data[i] * 2.0 + 1.0
    end
    return result
end
```

### All Search Operations Must Be Cached

```rust
pub struct CachedSearch {
    cache: Arc<DashMap<String, Vec<SearchResult>>>,
    ttl: Duration,
}

impl CachedSearch {
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        // Check cache
        if let Some(cached) = self.cache.get(query) {
            return Ok(cached.clone());
        }
        
        // Execute search
        let results = self.execute_search(query).await?;
        
        // Store in cache
        self.cache.insert(query.to_string(), results.clone());
        
        Ok(results)
    }
}
```

### All Learning Must Be Persistent

```rust
pub async fn store_learning_data(
    db: &PgPool,
    episode: Episode
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO learning_sessions (user_id, context, outcomes) VALUES ($1, $2, $3)",
        episode.user_id,
        episode.context,
        episode.outcomes
    )
    .execute(db)
    .await?;
    
    Ok(())
}
```

### All Errors Must Include Recovery Strategies

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("Database connection failed: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Julia FFI call failed: {0}")]
    JuliaError(String),
    
    #[error("Optimization did not converge after {attempts} attempts")]
    ConvergenceError { attempts: u32 },
}

impl ProcessError {
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            Self::DatabaseError(_) => RecoveryStrategy::RetryWithBackoff,
            Self::JuliaError(_) => RecoveryStrategy::FallbackToRust,
            Self::ConvergenceError { .. } => RecoveryStrategy::RelaxConstraints,
        }
    }
}
```

---

## 🧪 Testing Standards

### Unit Tests (Rust)
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hybrid_search() {
        let query = "test query";
        let results = hybrid_search(query).await.unwrap();
        
        assert!(!results.is_empty());
        assert!(results[0].score > 0.0);
    }
    
    #[test]
    fn test_zero_copy_transfer() {
        let data = vec![1.0, 2.0, 3.0];
        let result = rust_to_julia_zerocopy(&data).unwrap();
        assert!(result > 0.0);
    }
}
```

### Property-Based Tests (Julia)
```julia
using Test, Random

@testset "Chaos Analysis Properties" begin
    @testset "Lyapunov always finite" begin
        for _ in 1:100
            data = randn(1000)
            λ = lyapunov_exponent(data)
            @test isfinite(λ)
        end
    end
end
```

### Integration Tests (Python/JAX)
```python
import pytest
import jax.numpy as jnp

def test_ml_pipeline_end_to_end():
    # Setup
    texts = ["test text 1", "test text 2"]
    
    # Execute
    embeddings = generate_embeddings(texts)
    
    # Verify
    assert embeddings.shape == (2, 768)
    assert jnp.all(jnp.isfinite(embeddings))
```

---

## 🚀 Performance Guidelines

### Latency Budgets

| Operation | Target | Max Acceptable |
|-----------|--------|----------------|
| MCP Request | 50ms | 300ms |
| Julia Math | 10ms | 50ms |
| JAX Inference | 15ms | 100ms |
| Search (4 engines) | 20ms | 100ms |
| Learning Update | 5ms | 20ms |

### Memory Usage Targets

- **Baseline**: ~2.4 GB
- **Working Set**: ~6.4 GB
- **Max**: 16 GB (with headroom)

### CPU Utilization

- **Target**: 75% avg (leaving headroom)
- **Cores**: Scale with available (8+ recommended)

---

## 📊 Monitoring & Observability

### Prometheus Metrics
```rust
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref REQUESTS_TOTAL: Counter = register_counter!(
        "memory_p_requests_total",
        "Total number of MCP requests"
    ).unwrap();
    
    static ref REQUEST_DURATION: Histogram = register_histogram!(
        "memory_p_request_duration_seconds",
        "Request duration in seconds"
    ).unwrap();
}
```

### Structured Logging
```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(data))]
pub async fn process_request(data: &[u8]) -> Result<Response> {
    info!(data_len = data.len(), "Processing request");
    
    let result = do_processing(data).await;
    
    match result {
        Ok(response) => {
            info!("Request processed successfully");
            Ok(response)
        }
        Err(e) => {
            error!(error = %e, "Request processing failed");
            Err(e)
        }
    }
}
```

---

## 🔐 Security Guidelines

- ✅ **Input Validation**: Validar TODOS los inputs externos
- ✅ **SQL Injection**: Usar parametrized queries (sqlx macros)
- ✅ **Memory Safety**: Rust guarantees + careful FFI
- ✅ **Secrets**: Nunca en código, usar environment variables
- ✅ **Rate Limiting**: Proteger endpoints públicos

---

## 📚 Documentation Standards

### Rustdoc Example
```rust
/// Executes hybrid search across 4 engines.
///
/// This function coordinates semantic (Qdrant), full-text (Tantivy),
/// custom (MemoryBank), and mathematical fusion searches.
///
/// # Arguments
///
/// * `query` - The search query string
///
/// # Returns
///
/// Vector of search results sorted by fused relevance score
///
/// # Errors
///
/// Returns `SearchError` if any engine fails or fusion cannot be computed
///
/// # Examples
///
/// ```
/// let results = hybrid_search("rust async").await?;
/// assert!(!results.is_empty());
/// ```
pub async fn hybrid_search(query: &str) -> Result<Vec<SearchResult>, SearchError> {
    // Implementation
}
```

---

**Last Updated**: January 2026  
**Version**: 2.0.0  
**Maintained by**: MEMORY_P Team  

**Remember**: Code with mathematical precision, not arbitrary heuristics. 🧠⚡
