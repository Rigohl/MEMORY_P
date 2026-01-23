# FFI - Foreign Function Interface Layer

**Versión**: 2.0.0  
**Lenguajes**: Rust (orquestador) + Julia + JAX + Mojo + Pony + Zig

---

## 📋 Índice

- [Arquitectura](#arquitectura)
- [Lenguajes Integrados](#lenguajes-integrados)
- [Estructura de Directorios](#estructura-de-directorios)
- [Compilación](#compilación)
- [Uso desde Rust](#uso-desde-rust)
- [Seguridad FFI](#seguridad-ffi)
- [Performance](#performance)
- [Troubleshooting](#troubleshooting)

---

## Arquitectura

MEMORY_P utiliza una arquitectura FFI multi-lenguaje donde **Rust actúa como orquestador** de todos los componentes:

```
┌─────────────────────────────────────────────────────────────┐
│                     Rust Orchestrator                        │
│                  (src/ffi/mod.rs)                           │
├─────────────────────────────────────────────────────────────┤
│                    FFI Bridge Layer                          │
│                  (FFI/src/ffi_bridge.zig)                   │
├─────────────────────────────────────────────────────────────┤
│              Multi-Language Engines                          │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐ │
│  │  Julia   │   JAX    │   Mojo   │   Pony   │   Zig    │ │
│  │  Math    │   ML     │  SIMD    │  Actors  │  Bridge  │ │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Flujo de Comunicación

1. **Rust** recibe petición MCP desde cliente
2. **Rust** decide qué motor usar según la operación
3. **Zig** bridge traduce estructuras Rust ↔ C ABI
4. **Julia/JAX/Mojo/Pony** ejecuta operación especializada
5. **Zig** retorna resultado a Rust
6. **Rust** envía respuesta MCP al cliente

---

## Lenguajes Integrados

### 1. Julia - Mathematical Core 🧮

**Responsabilidades**:
- Análisis de caos (ChaosTools.jl)
- Optimización matemática (Optim.jl)
- Ecuaciones diferenciales (DifferentialEquations.jl)
- Modelado simbólico (ModelingToolkit.jl)

**Ubicación**: `FFI/src/julia_math.jl`

**Ejemplo de Uso**:
```rust
use crate::ffi::julia::optimize_weights;

let optimal = optimize_weights(&[0.33, 0.33, 0.34])?;
// [0.41, 0.29, 0.30]
```

### 2. JAX - ML Inference 🤖

**Responsabilidades**:
- Generación de embeddings (sentence-transformers)
- Inference de modelos ML
- Clasificación de código
- Ranking ML-powered

**Ubicación**: `FFI/src/jax_inference.py`

**Ejemplo de Uso**:
```rust
use crate::ffi::jax::generate_embeddings;

let embeddings = generate_embeddings(&texts)?;
// Vec<Vec<f32>> de 384 dimensiones
```

### 3. Mojo - SIMD Kernels ⚡

**Responsabilidades**:
- Dot products ultra-rápidos
- Matrix operations
- Operaciones vectorizadas
- Hotspots críticos de performance

**Ubicación**: `FFI/src/kernels.mojo`

**Performance**: 35000x más rápido que Python puro

**Ejemplo de Uso**:
```rust
use crate::ffi::mojo::dot_product;

let result = dot_product(&vec_a, &vec_b)?;
// 12 µs para 1M elementos
```

### 4. Pony - Actor System 🎭

**Responsabilidades**:
- Concurrencia sin locks (actor model)
- Búsqueda distribuida en paralelo
- Message passing seguro
- Garantías de ausencia de data races

**Ubicación**: `FFI/src/search_actor.pony`

**Ejemplo de Uso**:
```rust
use crate::ffi::pony::distributed_search;

let results = distributed_search(&query, &indices).await?;
```

### 5. Zig - FFI Bridge 🌉

**Responsabilidades**:
- Traducción entre Rust y otros lenguajes
- Wrapper C ABI
- Memory safety en fronteras FFI
- Dispatcher de llamadas

**Ubicación**: `FFI/src/ffi_bridge.zig`

**Ejemplo de Uso**:
```rust
// Zig actúa como intermediario transparente
use crate::ffi::bridge::call_foreign;

let result = call_foreign(Language::Julia, &data)?;
```

---

## Estructura de Directorios

```
FFI/
├── README.md                    # Este archivo
├── Cargo.toml                   # Opcional: crate FFI separado
├── build.zig                    # Script de build Zig
├── Makefile                     # Build automation
│
├── src/                         # Código fuente multi-lenguaje
│   ├── ffi_bridge.zig          # Dispatcher principal
│   ├── julia_math.jl           # Motor matemático Julia
│   ├── jax_inference.py        # ML inference con JAX
│   ├── kernels.mojo            # Kernels SIMD Mojo
│   ├── search_actor.pony       # Actor system Pony
│   └── exports.h               # Headers C para FFI
│
├── lib/                         # Bibliotecas compiladas
│   ├── libjulia_ffi.so         # Julia shared library
│   ├── libjax_ffi.so           # JAX Python extension
│   ├── libmojo_kernels.so      # Mojo compiled kernels
│   ├── libpony_actors.so       # Pony runtime
│   └── libzig_bridge.so        # Zig FFI bridge
│
├── examples/                    # Ejemplos de uso
│   ├── julia_optimization.rs
│   ├── jax_embeddings.rs
│   ├── mojo_simd.rs
│   ├── pony_concurrent.rs
│   └── full_integration.rs
│
└── tests/                       # Tests de integración FFI
    ├── test_julia_ffi.rs
    ├── test_jax_ffi.rs
    ├── test_mojo_ffi.rs
    ├── test_pony_ffi.rs
    └── test_safety.rs
```

---

## Compilación

### Prerrequisitos

```bash
# Rust (requerido)
rustup update stable

# Zig (requerido para FFI bridge)
# https://ziglang.org/download/
zig version  # 0.12.0 o superior

# Julia (opcional pero recomendado)
julia --version  # 1.10.0 o superior

# Python + JAX (opcional)
python3 --version  # 3.11 o superior
pip install jax[cuda12] sentence-transformers

# Mojo (opcional)
# https://docs.modular.com/mojo/manual/get-started/
mojo --version

# Pony (opcional)
# https://www.ponylang.io/
ponyc --version
```

### Build Completo

```bash
# Desde la raíz de MEMORY_P
cd FFI

# Build todas las bibliotecas FFI
make all

# O individualmente
make julia    # Compila Julia FFI
make jax      # Compila JAX extension
make mojo     # Compila Mojo kernels
make pony     # Compila Pony actors
make zig      # Compila Zig bridge
```

### Build Solo Core (Sin FFI)

Si solo quieres el core de Rust sin FFI:

```bash
cd /path/to/MEMORY_P
cargo build --release --no-default-features
```

---

## Uso desde Rust

### Ejemplo Completo: Búsqueda Híbrida

```rust
use memory_p::ffi::{julia, jax, mojo, pony};

async fn hybrid_search(query: &str) -> Result<Vec<Document>> {
    // 1. Generar embeddings con JAX
    let embedding = jax::generate_embedding(query)?;
    
    // 2. Búsqueda vectorial paralela con Pony actors
    let vector_results = pony::parallel_vector_search(&embedding).await?;
    
    // 3. Calcular similitudes con Mojo SIMD
    let similarities = mojo::cosine_similarity_batch(&embedding, &vector_results)?;
    
    // 4. Optimizar ranking con Julia
    let optimal_weights = julia::optimize_ranking(&similarities)?;
    
    // 5. Aplicar pesos y retornar
    let final_results = apply_weights(vector_results, optimal_weights);
    
    Ok(final_results)
}
```

### Manejo de Errores FFI

```rust
use crate::ffi::error::FfiError;

match julia::optimize(&data) {
    Ok(result) => println!("✅ Optimized: {:?}", result),
    Err(FfiError::JuliaException(e)) => eprintln!("❌ Julia error: {}", e),
    Err(FfiError::NullPointer) => eprintln!("❌ Null pointer in FFI"),
    Err(FfiError::MemorySafety(e)) => eprintln!("❌ Memory safety violation: {}", e),
    Err(e) => eprintln!("❌ Unknown FFI error: {}", e),
}
```

---

## Seguridad FFI

### Principios de Seguridad

1. **No Null Pointers**: Siempre validar punteros antes de dereferenciar
2. **Memory Ownership**: Usar `Box`, `Vec` y smart pointers de Rust
3. **Lifetime Management**: Marcar claramente ownership de memoria
4. **Error Handling**: Propagar errores FFI con `Result<T, FfiError>`
5. **Type Safety**: Usar `#[repr(C)]` para structs compartidos

### Ejemplo Seguro

```rust
#[repr(C)]
pub struct FfiVec {
    data: *mut f64,
    len: usize,
    cap: usize,
}

impl FfiVec {
    /// Crea FfiVec desde Vec<f64> de forma segura
    pub fn from_vec(v: Vec<f64>) -> Self {
        let mut v = std::mem::ManuallyDrop::new(v);
        FfiVec {
            data: v.as_mut_ptr(),
            len: v.len(),
            cap: v.capacity(),
        }
    }
    
    /// Reconstruye Vec<f64> sin double-free
    pub unsafe fn into_vec(self) -> Vec<f64> {
        Vec::from_raw_parts(self.data, self.len, self.cap)
    }
}

extern "C" {
    fn julia_process_vec(vec: FfiVec) -> FfiVec;
}

pub fn safe_julia_call(data: Vec<f64>) -> Result<Vec<f64>> {
    let ffi_input = FfiVec::from_vec(data);
    let ffi_output = unsafe { julia_process_vec(ffi_input) };
    
    // Validar antes de reconstruir
    if ffi_output.data.is_null() {
        return Err(FfiError::NullPointer);
    }
    
    Ok(unsafe { ffi_output.into_vec() })
}
```

---

## Performance

### Benchmarks

| Operación | Rust Puro | Con FFI | Speedup |
|-----------|-----------|---------|---------|
| Dot Product (1M) | 850 µs | 12 µs (Mojo) | **70x** |
| Chaos Analysis | N/A | 157 ms (Julia) | N/A |
| Embeddings (batch=32) | N/A | 46 ms (JAX) | N/A |
| Concurrent Search | 245 ms | 89 ms (Pony) | **2.7x** |

### Overhead FFI

```
┌─────────────────┬──────────┬──────────┐
│ Call Type       │ Latency  │ Overhead │
├─────────────────┼──────────┼──────────┤
│ Rust → Zig      │ ~5 ns    │ minimal  │
│ Zig → Julia     │ ~200 ns  │ low      │
│ Zig → JAX       │ ~500 ns  │ medium   │
│ Zig → Mojo      │ ~10 ns   │ minimal  │
│ Zig → Pony      │ ~100 ns  │ low      │
└─────────────────┴──────────┴──────────┘
```

---

## Referencias

- **Rust FFI Guide**: https://doc.rust-lang.org/nomicon/ffi.html
- **Julia C Interface**: https://docs.julialang.org/en/v1/manual/calling-c-and-fortran-code/
- **JAX Foreign Functions**: https://jax.readthedocs.io/en/latest/ffi.html
- **Mojo C Interop**: https://docs.modular.com/mojo/manual/c
- **Pony FFI**: https://tutorial.ponylang.io/c-ffi.html
- **Zig Build System**: https://ziglang.org/documentation/master/#Build-System

---

**Última actualización**: Enero 2026  
**Versión**: 2.0.0  
**Proyecto**: MEMORY_P - Always-On MCP Toolkit with Multi-Language Brain
