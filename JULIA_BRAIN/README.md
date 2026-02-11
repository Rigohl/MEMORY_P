# JULIA_BRAIN - Mathematical Core for MEMORY_P v2.0

Julia mathematical brain providing predictive optimization, chaos analysis, and differential equations for MEMORY_P.

## 🧠 Core Modules

### 1. Chaos Analysis (`chaos_metrics.jl`)
Analyzes system complexity using chaos theory:
- Lyapunov exponents (sensitivity to initial conditions)
- Correlation dimension (attractor complexity)
- Bifurcation detection

### 2. Predictive Optimization
Forecasts and optimizes system metrics:
- Time series forecasting (ARIMA, SARIMA)
- Multi-objective optimization (NSGA-II)
- Resource allocation optimization

### 3. Differential Systems
Solves system dynamics equations:
- ODEs for performance modeling
- Control systems for auto-scaling
- Stability analysis

## 📦 Dependencies

```julia
using Pkg
Pkg.add([
    "DifferentialEquations",
    "ChaosTools", 
    "Optim",
    "Forecasting",
    "ModelingToolkit",
    "Statistics"
])
```

## 🚀 Usage from Rust

```rust
use std::process::Command;

// Call Julia script
let output = Command::new("julia")
    .arg("JULIA_BRAIN/chaos_metrics.jl")
    .arg("--metrics")
    .arg(metrics_json)
    .output()
    .expect("Failed to execute Julia");

let chaos_score = parse_output(output.stdout);
```

## 🔧 Integration with Docker

Julia is pre-installed in the MEMORY_P Docker image:

```yaml
environment:
  - JULIA_ENABLED=true
  - JULIA_THREADS=4
volumes:
  - ./JULIA_BRAIN:/app/JULIA_BRAIN:ro
```

## 📊 Example Outputs

### Chaos Metrics
```json
{
  "lyapunov_exponent": 0.23,
  "correlation_dimension": 2.5,
  "stability": "semi-chaotic",
  "recommendation": "Adjust batching parameters"
}
```

### Predictive Optimization
```json
{
  "predicted_cpu_usage": [45.2, 48.1, 52.3],
  "predicted_memory_usage": [6.2, 6.5, 7.1],
  "optimal_workers": 8,
  "optimal_batch_size": 1000
}
```

## 🎯 Use Cases

1. **Auto-scaling**: Predict load and scale services proactively
2. **Performance Tuning**: Find optimal parameters via optimization
3. **Anomaly Detection**: Detect chaotic behavior in metrics
4. **Resource Planning**: Forecast resource needs for capacity planning

## 📚 Documentation

- See individual `.jl` files for detailed module documentation
- DEVOPS.md for integration with monitoring stack
- AGENTS.md for intelligent agent usage of Julia brain
