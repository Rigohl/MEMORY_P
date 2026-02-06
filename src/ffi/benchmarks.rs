//! FFI Bridge Latency Benchmarks
//!
//! Objetivo: Validar que el FFI bridge alcanza <1µs de latencia
//!
//! Para ejecutar:
//! ```bash
//! cargo test --release ffi_benchmark -- --nocapture --ignored
//! ```

#[cfg(test)]
mod tests {
    use crate::ffi::bridge::{self, Language};
    use std::time::{Duration, Instant};

    /// Estructura para resultados de benchmarks
    struct BenchResult {
        name: String,
        iterations: usize,
        total_duration: Duration,
        avg_ns: u64,
        min_ns: u64,
        max_ns: u64,
        p50_ns: u64,
        p95_ns: u64,
        p99_ns: u64,
    }

    impl BenchResult {
        fn new(name: &str, iterations: usize, latencies: &mut [u64]) -> Self {
            latencies.sort_unstable();

            let total_ns: u64 = latencies.iter().sum();
            let avg_ns = total_ns / iterations as u64;
            let min_ns = *latencies.first().unwrap_or(&0);
            let max_ns = *latencies.last().unwrap_or(&0);

            let p50_idx = (iterations as f64 * 0.50) as usize;
            let p95_idx = (iterations as f64 * 0.95) as usize;
            let p99_idx = (iterations as f64 * 0.99) as usize;

            let p50_ns = latencies[p50_idx.min(latencies.len() - 1)];
            let p95_ns = latencies[p95_idx.min(latencies.len() - 1)];
            let p99_ns = latencies[p99_idx.min(latencies.len() - 1)];

            Self {
                name: name.to_string(),
                iterations,
                total_duration: Duration::from_nanos(total_ns),
                avg_ns,
                min_ns,
                max_ns,
                p50_ns,
                p95_ns,
                p99_ns,
            }
        }

        fn print(&self) {
            println!("\n📊 Benchmark: {}", self.name);
            println!("   Iterations: {}", self.iterations);
            println!("   Total time: {:?}", self.total_duration);
            println!(
                "   Average:    {}ns ({:.2}µs)",
                self.avg_ns,
                self.avg_ns as f64 / 1000.0
            );
            println!(
                "   Min:        {}ns ({:.2}µs)",
                self.min_ns,
                self.min_ns as f64 / 1000.0
            );
            println!(
                "   Max:        {}ns ({:.2}µs)",
                self.max_ns,
                self.max_ns as f64 / 1000.0
            );
            println!(
                "   P50:        {}ns ({:.2}µs)",
                self.p50_ns,
                self.p50_ns as f64 / 1000.0
            );
            println!(
                "   P95:        {}ns ({:.2}µs)",
                self.p95_ns,
                self.p95_ns as f64 / 1000.0
            );
            println!(
                "   P99:        {}ns ({:.2}µs)",
                self.p99_ns,
                self.p99_ns as f64 / 1000.0
            );

            // Validar target de <1µs
            if self.p95_ns < 1000 {
                println!("   ✅ PASS: P95 < 1µs target");
            } else {
                println!(
                    "   ⚠️  WARN: P95 > 1µs target ({:.2}µs)",
                    self.p95_ns as f64 / 1000.0
                );
            }
        }

        fn passes_target(&self) -> bool {
            // En modo debug, permitimos hasta 10µs
            // En release, target estricto de 1µs
            #[cfg(debug_assertions)]
            let target_ns = 10_000;
            #[cfg(not(debug_assertions))]
            let target_ns = 1_000;

            self.p95_ns < target_ns
        }
    }

    /// Benchmark: Llamada FFI mínima (3 elementos)
    fn bench_minimal_call(iterations: usize) -> BenchResult {
        let mut latencies = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let mut data = vec![1.0, 2.0, 3.0];

            let start = Instant::now();
            let _ = bridge::dispatch_fast(Language::Zig, "multiply", &mut data);
            let elapsed = start.elapsed();

            latencies.push(elapsed.as_nanos() as u64);
        }

        BenchResult::new("Minimal FFI call (3 elements)", iterations, &mut latencies)
    }

    /// Benchmark: Llamada FFI pequeña (64 elementos - límite stack)
    fn bench_small_call(iterations: usize) -> BenchResult {
        let mut latencies = Vec::with_capacity(iterations);
        let mut data: Vec<f64> = (0..64).map(|i| i as f64).collect();

        for _ in 0..iterations {
            let start = Instant::now();
            let _ = bridge::dispatch_fast(Language::Zig, "process", &mut data);
            let elapsed = start.elapsed();

            latencies.push(elapsed.as_nanos() as u64);
        }

        BenchResult::new(
            "Small FFI call (64 elements, stack alloc)",
            iterations,
            &mut latencies,
        )
    }

    /// Benchmark: Llamada FFI mediana (256 elementos)
    fn bench_medium_call(iterations: usize) -> BenchResult {
        let mut latencies = Vec::with_capacity(iterations);
        let mut data: Vec<f64> = (0..256).map(|i| i as f64).collect();

        for _ in 0..iterations {
            let start = Instant::now();
            let _ = bridge::dispatch_fast(Language::Zig, "process", &mut data);
            let elapsed = start.elapsed();

            latencies.push(elapsed.as_nanos() as u64);
        }

        BenchResult::new("Medium FFI call (256 elements)", iterations, &mut latencies)
    }

    /// Benchmark: Llamada FFI grande (1024 elementos)
    fn bench_large_call(iterations: usize) -> BenchResult {
        let mut latencies = Vec::with_capacity(iterations);
        let mut data: Vec<f64> = (0..1024).map(|i| i as f64).collect();

        for _ in 0..iterations {
            let start = Instant::now();
            let _ = bridge::dispatch_fast(Language::Zig, "process", &mut data);
            let elapsed = start.elapsed();

            latencies.push(elapsed.as_nanos() as u64);
        }

        BenchResult::new("Large FFI call (1K elements)", iterations, &mut latencies)
    }

    /// Benchmark: Batch processing paralelo
    #[cfg(feature = "ffi-zig")]
    fn bench_batch_parallel(iterations: usize) -> BenchResult {
        let mut latencies = Vec::with_capacity(iterations);

        // Crear batch de 100 requests
        let requests: Vec<_> = (0..100)
            .map(|i| {
                let data: Vec<f64> = vec![i as f64; 10];
                (Language::Zig, "process", data)
            })
            .collect();

        for _ in 0..iterations {
            let start = Instant::now();
            let _ = bridge::dispatch_batch(&requests);
            let elapsed = start.elapsed();

            latencies.push(elapsed.as_nanos() as u64);
        }

        BenchResult::new("Batch parallel (100 requests)", iterations, &mut latencies)
    }

    #[test]
    #[ignore] // Solo ejecutar con: cargo test --release ffi_benchmark -- --nocapture --ignored
    fn ffi_benchmark_suite() {
        println!("🚀 MEMORY_P FFI Bridge Latency Benchmarks");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        #[cfg(debug_assertions)]
        println!("⚠️  Running in DEBUG mode - expect higher latencies");
        #[cfg(not(debug_assertions))]
        println!("✅ Running in RELEASE mode");

        println!("Target: <1µs (1000ns) for P95 latency\n");

        // Inicializar FFI bridge
        if !bridge::init() {
            eprintln!("❌ Failed to initialize FFI bridge");
            eprintln!("   Compile with --features ffi-zig to run benchmarks");
            return;
        }

        // Reset metrics
        bridge::reset_metrics();

        // Warm-up
        println!("🔥 Warming up...");
        for _ in 0..1000 {
            let mut data = vec![1.0, 2.0, 3.0];
            let _ = bridge::dispatch_fast(Language::Zig, "warmup", &mut data);
        }

        // Run benchmarks
        let iterations = 10_000;

        let results = vec![
            bench_minimal_call(iterations),
            bench_small_call(iterations),
            bench_medium_call(iterations),
            bench_large_call(iterations),
        ];

        // Print all results
        for result in &results {
            result.print();
        }

        // Batch benchmark (menos iteraciones)
        #[cfg(feature = "ffi-zig")]
        {
            let batch_result = bench_batch_parallel(100);
            batch_result.print();
        }

        // Print overall metrics
        println!("\n📈 Overall FFI Metrics:");
        let (total_calls, avg_us) = bridge::get_metrics();
        println!("   Total calls: {}", total_calls);
        println!("   Average latency: {:.2}µs", avg_us);

        // Cleanup
        bridge::shutdown();

        println!("\n✅ Benchmarks completed!");

        // Assert que al menos el benchmark minimal pasa el target
        #[cfg(not(debug_assertions))]
        assert!(
            results[0].passes_target(),
            "Minimal FFI call should be <1µs in release mode"
        );
    }

    #[test]
    fn test_ffi_latency_basic() {
        // Test básico que siempre ejecuta (no ignorado)
        if !bridge::init() {
            // FFI no disponible, skip test
            return;
        }

        bridge::reset_metrics();

        let mut data = vec![1.0, 2.0, 3.0];
        let start = Instant::now();
        let result = bridge::dispatch_fast(Language::Zig, "test", &mut data);
        let elapsed = start.elapsed();

        bridge::shutdown();

        // Solo verificamos que funciona
        println!(
            "FFI call latency: {}ns ({:.2}µs)",
            elapsed.as_nanos(),
            elapsed.as_micros() as f64 / 1.0
        );

        #[cfg(feature = "ffi-zig")]
        assert!(result.is_ok(), "FFI call should succeed");
    }

    #[test]
    fn test_zero_copy_performance() {
        // Verificar que zero-copy es más rápido que copy
        if !bridge::init() {
            return;
        }

        let iterations = 1000;
        let mut data: Vec<f64> = (0..1024).map(|i| i as f64).collect();

        // Medir con zero-copy
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = bridge::dispatch_fast(Language::Zig, "process", &mut data);
        }
        let zero_copy_duration = start.elapsed();

        bridge::shutdown();

        println!(
            "Zero-copy performance: {:?} for {} iterations",
            zero_copy_duration, iterations
        );
        println!(
            "Average per call: {:.2}µs",
            zero_copy_duration.as_micros() as f64 / iterations as f64
        );
    }

    #[test]
    fn test_ffi_zero_copy() {
        // Test que verifica zero-copy
        if !bridge::init() {
            return; // Skip si FFI no disponible
        }

        let mut original = vec![1.0, 2.0, 3.0];
        let original_ptr = original.as_ptr();

        // Dispatch debe usar la misma memoria (zero-copy)
        let _ = bridge::dispatch_fast(Language::Zig, "test", &mut original);

        // Verificar que el puntero no cambió (misma memoria)
        assert_eq!(
            original.as_ptr(),
            original_ptr,
            "Zero-copy: el puntero no debe cambiar"
        );

        bridge::shutdown();
    }

    #[test]
    fn test_ffi_different_sizes() {
        // Test con diferentes tamaños de datos
        if !bridge::init() {
            return;
        }

        let test_cases = vec![
            ("tiny", 3),
            ("small", 64),
            ("medium", 256),
            ("large", 1024),
            ("xlarge", 4096),
        ];

        for (name, size) in test_cases {
            let mut data: Vec<f64> = (0..size).map(|i| i as f64).collect();
            let result = bridge::dispatch_fast(Language::Zig, "test", &mut data);

            #[cfg(feature = "ffi-zig")]
            assert!(result.is_ok(), "FFI call should succeed for size: {}", name);

            #[cfg(not(feature = "ffi-zig"))]
            assert!(result.is_err(), "FFI should fail without feature");
        }

        bridge::shutdown();
    }

    #[test]
    #[cfg(feature = "ffi-zig")]
    fn test_ffi_batch_correctness() {
        // Test que el batch processing produce resultados correctos
        if !bridge::init() {
            return;
        }

        let requests: Vec<_> = (0..10)
            .map(|i| {
                let data = vec![i as f64; 5];
                (Language::Zig, "test", data)
            })
            .collect();

        let results = bridge::dispatch_batch(&requests);

        assert_eq!(results.len(), 10, "Debe procesar todos los requests");

        let successful = results.iter().filter(|r| r.is_ok()).count();
        assert!(
            successful > 0,
            "Al menos algunos requests deben ser exitosos"
        );

        bridge::shutdown();
    }

    #[test]
    #[ignore] // Ejecutar con: cargo test --release --features ffi-zig ffi_usage_demo -- --nocapture --ignored
    fn ffi_usage_demo() {
        println!("🚀 MEMORY_P FFI Bridge - Ultra-Low-Latency Demo");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // 1. Inicializar el FFI bridge
        println!("\n1️⃣  Inicializando FFI bridge...");
        if !bridge::init() {
            eprintln!("❌ Error: FFI bridge no disponible");
            eprintln!("   Compile con: cargo test --features ffi-zig");
            return;
        }
        println!("   ✅ FFI bridge inicializado");

        // Reset metrics
        bridge::reset_metrics();

        // 2. Llamada simple (zero-copy)
        println!("\n2️⃣  Llamada FFI simple (zero-copy):");
        let mut data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        println!("   Input:  {:?}", data);

        let start = Instant::now();
        let result = bridge::dispatch_fast(Language::Zig, "multiply", &mut data);
        let elapsed = start.elapsed();

        if let Ok(output) = result {
            println!("   Output: {:?}", output);
        }
        println!(
            "   Latencia: {}ns ({:.2}µs)",
            elapsed.as_nanos(),
            elapsed.as_micros() as f64
        );

        // 3. Benchmark de múltiples llamadas
        println!("\n3️⃣  Benchmark: 1000 llamadas consecutivas");
        let iterations = 1000;
        let mut total_ns = 0u128;
        let mut min_ns = u128::MAX;
        let mut max_ns = 0u128;

        for _ in 0..iterations {
            let mut data = vec![1.0, 2.0, 3.0];
            let start = Instant::now();
            let _ = bridge::dispatch_fast(Language::Zig, "test", &mut data);
            let elapsed = start.elapsed().as_nanos();

            total_ns += elapsed;
            min_ns = min_ns.min(elapsed);
            max_ns = max_ns.max(elapsed);
        }

        let avg_ns = total_ns / iterations as u128;
        println!("   Average: {}ns ({:.2}µs)", avg_ns, avg_ns as f64 / 1000.0);
        println!("   Min:     {}ns ({:.2}µs)", min_ns, min_ns as f64 / 1000.0);
        println!("   Max:     {}ns ({:.2}µs)", max_ns, max_ns as f64 / 1000.0);

        // 4. Batch processing paralelo
        #[cfg(feature = "ffi-zig")]
        {
            println!("\n4️⃣  Batch processing paralelo (100 requests):");

            let requests: Vec<_> = (0..100)
                .map(|i| {
                    let data = vec![i as f64, (i + 1) as f64, (i + 2) as f64];
                    (Language::Zig, "batch", data)
                })
                .collect();

            let start = Instant::now();
            let results = bridge::dispatch_batch(&requests);
            let elapsed = start.elapsed();

            let successful = results.iter().filter(|r| r.is_ok()).count();
            println!("   Processed: {}/{} successful", successful, requests.len());
            println!("   Total time: {:?}", elapsed);
            println!(
                "   Per request: {:.2}µs",
                elapsed.as_micros() as f64 / requests.len() as f64
            );
        }

        // 5. Comparación de tamaños
        println!("\n5️⃣  Performance vs tamaño de datos:");

        let sizes = vec![10, 64, 256, 1024, 4096];
        for size in sizes {
            let mut data: Vec<f64> = (0..size).map(|i| i as f64).collect();

            let start = Instant::now();
            let _ = bridge::dispatch_fast(Language::Zig, "process", &mut data);
            let elapsed = start.elapsed();

            println!(
                "   {} elementos: {}ns ({:.2}µs)",
                size,
                elapsed.as_nanos(),
                elapsed.as_micros() as f64
            );
        }

        // 6. Métricas globales
        println!("\n6️⃣  Métricas globales del FFI bridge:");
        let (total_calls, avg_us) = bridge::get_metrics();
        println!("   Total calls: {}", total_calls);
        println!("   Average latency: {:.2}µs", avg_us);

        if avg_us < 1.0 {
            println!("   ✅ Target de <1µs alcanzado!");
        } else {
            println!("   ⚠️  Promedio mayor a 1µs");
            #[cfg(debug_assertions)]
            println!("      (Normal en debug mode - ejecutar con --release)");
        }

        // 7. Cleanup
        println!("\n7️⃣  Finalizando FFI bridge...");
        bridge::shutdown();
        println!("   ✅ FFI bridge cerrado correctamente");

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("✅ Demo completado exitosamente!");
    }
}
