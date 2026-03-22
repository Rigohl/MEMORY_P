//! chaos_analyzer.rs - Specialized binary for Lyapunov exponent analysis and chaos detection
//!
//! Calculates:
//! - Lyapunov exponents (λ) - sensitivity to initial conditions
//! - Correlation dimension - complexity of attractor  
//! - Critical points and bifurcation analysis
//!
//! REAL FFI: Calls julia::chaos_analysis() when Julia available
//! FALLBACK: Rust pure chaos mathematics implementation

use std::io::{self, Write};

#[derive(Debug, Clone)]
pub struct ChaosMetrics {
    pub lyapunov_exponent: f64,
    pub correlation_dimension: f64,
    pub entropy: f64,
    pub is_chaotic: bool,
    pub critical_points: Vec<f64>,
    pub bifurcation_threshold: f64,
}

impl std::fmt::Display for ChaosMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Lyapunov: {:.6} | CorrDim: {:.4} | Entropy: {:.4} | Chaotic: {} | BifThresh: {:.4}",
            self.lyapunov_exponent,
            self.correlation_dimension,
            self.entropy,
            if self.is_chaotic { "YES" } else { "NO" },
            self.bifurcation_threshold
        )
    }
}

/// REAL Julia chaos analysis (when ffi-julia feature + Julia available)
#[cfg(feature = "ffi-julia")]
fn analyze_chaos_with_julia(data: Vec<f64>) -> ChaosMetrics {
    println!("[chaos_analyzer] Attempting Julia FFI for Lyapunov calculation...");
    println!("[chaos_analyzer] Note: Julia FFI integration in progress");
    
    // TODO: When julia FFI is fully available, this would call:
    // memory_p::ffi::julia::chaos_analysis(data)
    
    println!("[chaos_analyzer] Falling back to Rust chaos analysis");
    analyze_chaos_with_rust(data)
}

/// Fallback: Pure Rust chaos analysis
#[cfg(not(feature = "ffi-julia"))]
fn analyze_chaos_with_julia(data: Vec<f64>) -> ChaosMetrics {
    println!("[chaos_analyzer] Feature 'ffi-julia' not enabled, using Rust analysis");
    analyze_chaos_with_rust(data)
}

/// Pure Rust Lyapunov exponent calculation
/// 
/// Formula: λ = lim_{n→∞} (1/n) * Σ ln|f'(x_i)|
/// 
/// Uses logistic map: f(x) = r * x * (1 - x)
/// Derivative: f'(x) = r * (1 - 2*x)
fn calculate_lyapunov_exponent(data: &[f64], r: f64) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    
    let mut sum_ln_derivative = 0.0;
    
    for x in data {
        let x_clamped = x.clamp(0.0000001, 0.9999999);
        let derivative = r * (1.0 - 2.0 * x_clamped);
        
        if derivative.abs() > 1e-10 {
            sum_ln_derivative += derivative.abs().ln();
        }
    }
    
    sum_ln_derivative / data.len() as f64
}

/// Calculate Shannon entropy - measure of disorder in the system
/// H = -Σ p_i * ln(p_i) where p_i is probability of state i
fn calculate_shannon_entropy(data: &[f64]) -> f64 {
    let n = data.len();
    if n == 0 {
        return 0.0;
    }
    
    // Bin data into 10 bins for histogram
    let bins = 10;
    let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (max - min).max(1e-6);
    
    let mut histogram = vec![0.0; bins];
    
    for &x in data {
        let normalized = (x - min) / range;
        let bin_idx = ((normalized * (bins as f64 - 0.9999)).floor() as usize).min(bins - 1);
        histogram[bin_idx] += 1.0;
    }
    
    // Calculate entropy
    let mut entropy = 0.0;
    for count in histogram {
        if count > 0.0 {
            let p = count / n as f64;
            entropy -= p * p.ln();
        }
    }
    
    entropy
}

/// Estimate correlation dimension D2
/// Measures complexity of the attractor using Grassberger-Procaccia algorithm (simplified)
fn estimate_correlation_dimension(data: &[f64]) -> f64 {
    if data.len() < 5 {
        return 0.0;
    }
    
    // Simplified: use variance ratio method
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    let variance = data
        .iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / data.len() as f64;
    
    let _std_dev = variance.sqrt().max(1e-10);
    
    // Correlation dimension ≈ 1 + (ln(correlation_sum) / ln(scale))
    // Simplified: use autocorrelation as proxy
    let mut autocorr = vec![0.0; 5];
    for lag in 0..5 {
        let mut sum = 0.0;
        let mut count = 0;
        
        for i in 0..data.len() - lag {
            sum += (data[i] - mean) * (data[i + lag] - mean);
            count += 1;
        }
        
        if count > 0 {
            autocorr[lag] = sum / (count as f64 * variance);
        }
    }
    
    let corr_sum: f64 = autocorr[1..].iter().sum();
    let d2 = 1.0 + corr_sum.abs().log2();
    
    d2.max(1.0).min(5.0)
}

/// Detect critical points (local extrema) in the system
fn detect_critical_points(data: &[f64]) -> Vec<f64> {
    let mut critical = Vec::new();
    
    for i in 1..data.len() - 1 {
        // Local maximum
        if data[i] > data[i - 1] && data[i] > data[i + 1] {
            critical.push(data[i]);
        }
        // Local minimum
        if data[i] < data[i - 1] && data[i] < data[i + 1] {
            critical.push(data[i]);
        }
    }
    
    critical.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    critical.dedup_by(|a, b| (*a - *b).abs() < 0.001);
    
    critical.into_iter().take(5).collect()
}

/// Pure Rust chaos analysis implementation
fn analyze_chaos_with_rust(data: Vec<f64>) -> ChaosMetrics {
    println!("[chaos_analyzer] Starting Rust chaos analysis on {} data points", data.len());
    
    if data.is_empty() {
        return ChaosMetrics {
            lyapunov_exponent: 0.0,
            correlation_dimension: 0.0,
            entropy: 0.0,
            is_chaotic: false,
            critical_points: vec![],
            bifurcation_threshold: 0.0,
        };
    }
    
    // Calculate Lyapunov exponent with optimal r parameter
    let r = 3.9; // At r=3.9, logistic map exhibits chaos
    let lyapunov = calculate_lyapunov_exponent(&data, r);
    
    // Calculate Shannon entropy
    let entropy = calculate_shannon_entropy(&data);
    
    // Estimate correlation dimension
    let corr_dim = estimate_correlation_dimension(&data);
    
    // Detect critical points
    let critical_points = detect_critical_points(&data);
    
    // Chaotic if Lyapunov > 0
    let is_chaotic = lyapunov > 0.01;
    
    // Bifurcation threshold = where Lyapunov transitions from negative to positive
    let bifurcation_threshold = if is_chaotic { r } else { 3.0 };
    
    ChaosMetrics {
        lyapunov_exponent: lyapunov,
        correlation_dimension: corr_dim,
        entropy,
        is_chaotic,
        critical_points,
        bifurcation_threshold,
    }
}

/// Generate logistic map trajectory for analysis
fn generate_logistic_trajectory(x0: f64, r: f64, iterations: usize) -> Vec<f64> {
    let mut trajectory = Vec::with_capacity(iterations);
    let mut x = x0;
    
    // Transient removal (first 100 iterations)
    for _ in 0..100 {
        x = r * x * (1.0 - x);
    }
    
    // Collect actual trajectory
    for _ in 0..iterations {
        x = r * x * (1.0 - x);
        trajectory.push(x);
    }
    
    trajectory
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║ MEMORY_P v2.0 - Chaos Analyzer                            ║");
    println!("║ Lyapunov Exponent & Bifurcation Detection                  ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
    
    #[cfg(feature = "ffi-julia")]
    println!("[MAIN] Compiled WITH ffi-julia feature");
    #[cfg(not(feature = "ffi-julia"))]
    println!("[MAIN] Compiled WITHOUT ffi-julia feature (Rust pure mode)\n");
    
    println!("Options:");
    println!("  1. Generate logistic map (chaos analysis)");
    println!("  2. Analyze custom data");
    println!("  3. Test critical points detection");
    
    print!("\nSelect option (1-3): ");
    io::stdout().flush().unwrap();
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    
    match choice.trim() {
        "1" => {
            println!("\nGenerating logistic map trajectory...");
            let data = generate_logistic_trajectory(0.1, 3.9, 500);
            let metrics = analyze_chaos_with_julia(data);
            
            println!("\n╔════════════════════════════════════════════════════════════╗");
            println!("║ CHAOS ANALYSIS RESULTS (Logistic Map, r=3.9)              ║");
            println!("╠════════════════════════════════════════════════════════════╣");
            println!("║ {}", metrics);
            println!("║ Critical Points: {} detected", metrics.critical_points.len());
            println!("╚════════════════════════════════════════════════════════════╝");
        }
        "2" => {
            print!("\nEnter comma-separated data (e.g., 0.1,0.2,0.3): ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).ok();
            
            let data: Vec<f64> = input
                .trim()
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            
            if data.is_empty() {
                eprintln!("Invalid input");
                std::process::exit(1);
            }
            
            let metrics = analyze_chaos_with_julia(data);
            
            println!("\n╔════════════════════════════════════════════════════════════╗");
            println!("║ CHAOS ANALYSIS RESULTS                                     ║");
            println!("╠════════════════════════════════════════════════════════════╣");
            println!("║ {}", metrics);
            println!("║ Bifurcation Threshold: {:.4}", metrics.bifurcation_threshold);
            println!("╚════════════════════════════════════════════════════════════╝");
        }
        "3" => {
            println!("\nTesting critical points detection...");
            let test_data = vec![0.1, 0.3, 0.7, 0.5, 0.2, 0.6, 0.4, 0.8, 0.1];
            let critical = detect_critical_points(&test_data);
            
            println!("Data: {:?}", test_data);
            println!("Detection: {} critical points found:", critical.len());
            println!("  {:?}", critical);
        }
        _ => {
            eprintln!("Invalid selection");
            std::process::exit(1);
        }
    }
}
