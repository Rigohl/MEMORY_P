---
name: "performance-benchmark"
description: "Crea y ejecuta benchmarks con criterion para MEMORY_P"
version: "1.0.0"
author: "MEMORY_P Team"
tags: ["rust", "benchmark", "performance", "criterion"]
---

# Performance Benchmark Skill

## Descripción
Crea benchmarks de rendimiento con Criterion para medir y validar optimizaciones en MEMORY_P.

## Cuándo Usar
- Validar optimizaciones
- Detectar regresiones de performance
- Comparar algoritmos
- Medir impacto de cambios

## Template de Benchmark
```rust
// benches/analyzer_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memory_p::analyzer::CodeAnalyzer;

fn bench_analyze(c: &mut Criterion) {
    use std::path::PathBuf;
    let analyzer = CodeAnalyzer::new();
    let path = PathBuf::from("src/main.rs");
    
    c.bench_function("analyze_file", |b| {
        b.iter(|| {
            analyzer.analyze_file(black_box(&path))
        });
    });
}

criterion_group!(benches, bench_analyze);
criterion_main!(benches);
```

## Comandos
```bash
# Ejecutar benchmarks
cargo bench

# Con baseline para comparar
cargo bench -- --save-baseline before
# hacer cambios...
cargo bench -- --baseline before

# Ver reportes
open target/criterion/report/index.html
```

## Mejores Prácticas
- Usar `black_box()` para evitar optimizaciones
- Medir solo lo importante (sin I/O)
- Documentar condiciones del benchmark
- Comparar con baselines

## Métricas Objetivo MEMORY_P
- Análisis: >1000 archivos/segundo
- Operaciones MCP: <100ms
- Throughput: Escalar linealmente hasta 8 cores
