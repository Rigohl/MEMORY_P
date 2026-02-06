# Auto Manager Guide - MEMORY_P v2.0

## 🎯 Overview

El **AutoManager** es el coordinador central del sistema de aprendizaje continuo de MEMORY_P. Proporciona:

- ✅ **Diagnósticos en tiempo real** del sistema
- 🔮 **Detección predictiva** de inconsistencias
- 🔧 **Auto-corrección** basada en teoría del caos (Julia)
- 📊 **Telemetría** con métricas clave
- 🧠 **Aprendizaje continuo** con patrones de usuario
- ⚡ **Optimización adaptativa** de parámetros

## 🏗️ Arquitectura

```
┌──────────────────────────────────────┐
│         AutoManager                  │
├──────────────────────────────────────┤
│ - Health Monitoring                  │
│ - Predictive Diagnostics             │
│ - Chaos-based Auto-correction        │
│ - Adaptive Parameter Optimization    │
│ - Continuous Learning Loop           │
│ - Telemetry Integration              │
└──────────────────────────────────────┘
           │
     ┌─────┼─────┬─────────┬────────┐
     │     │     │         │        │
┌────▼┐ ┌──▼──┐ ┌▼────┐ ┌─▼──┐ ┌───▼───┐
│Pred │ │Shar │ │Telem│ │Julia│ │Pattern│
│Engi │ │Memo │ │etry │ │ FFI │ │Detect │
└─────┘ └─────┘ └─────┘ └────┘ └───────┘
```

## 🚀 Quick Start

### Inicialización Básica

```rust
use memory_p::auto_manager::{AutoManager, ManagerConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ManagerConfig::default();
    let auto_manager = AutoManager::new(config);
    
    // Iniciar sistema always-on
    auto_manager.auto_start().await?;
    
    // El sistema ahora está auto-gestionándose
    // No requiere intervención manual
    
    Ok(())
}
```

### Con Dependencias Personalizadas

```rust
use memory_p::auto_manager::AutoManager;
use memory_p::prediction_engine::PredictionEngine;
use memory_p::shared_memory::SharedMemory;
use std::sync::Arc;

let prediction_engine = Arc::new(PredictionEngine::new());
let shared_memory = Arc::new(SharedMemory::new());

let auto_manager = AutoManager::with_dependencies(
    config,
    prediction_engine,
    shared_memory,
);

auto_manager.auto_start().await?;
```

## 📋 Configuración

### ManagerConfig

```rust
use std::time::Duration;

let config = ManagerConfig {
    // Intervalo de health checks
    check_interval: Duration::from_secs(30),
    
    // Máximo de errores antes de recovery
    max_errors: 3,
    
    // Timeout para recovery
    recovery_timeout: Duration::from_secs(10),
    
    // Auto-restart habilitado
    auto_restart: true,
};
```

## 🔬 Diagnósticos Predictivos

### Ejecutar Diagnósticos

```rust
// Ejecutar diagnóstico completo del sistema
let diagnostics = auto_manager
    .run_predictive_diagnostics()
    .await?;

for diag in diagnostics {
    println!("Componente: {}", diag.component);
    println!("Estado: {:?}", diag.status);
    
    for issue in diag.issues {
        println!("  ⚠️  {}: {}", issue.severity, issue.description);
        
        if issue.auto_correctable {
            println!("    ✅ Auto-corrección disponible");
        }
    }
    
    for failure in diag.predicted_failures {
        println!("  🔮 Fallo predicho: {} ({:.0}% probabilidad)",
            failure.component,
            failure.probability * 100.0
        );
    }
}
```

### Resultado de Diagnóstico

```rust
pub struct DiagnosticResult {
    pub component: String,
    pub status: ComponentStatus,           // Healthy/Warning/Degraded/Failed
    pub issues: Vec<Issue>,                // Issues detectados
    pub recommendations: Vec<String>,       // Recomendaciones
    pub predicted_failures: Vec<PredictedFailure>, // Fallos predichos
}
```

## 🔧 Auto-corrección con Caos

### Ejecutar Auto-corrección

```rust
// Auto-corrección basada en análisis de caos (Julia)
let result = auto_manager
    .run_chaos_based_autocorrection("engine_name")
    .await?;

if result.success {
    println!("✅ Componente corregido: {}", result.component);
    println!("   Acción: {}", result.action);
    println!("   Duración: {:.2}ms", result.duration_ms);
    println!("   Chaos Score: {:.2}", result.chaos_score);
} else {
    println!("❌ Auto-corrección falló para: {}", result.component);
}
```

### Interpretación de Chaos Score

| Score | Interpretación | Acción Recomendada |
|-------|----------------|-------------------|
| 0.0 - 0.2 | Estable | Monitor |
| 0.2 - 0.5 | Inestabilidad leve | Reset State |
| 0.5 - 0.8 | Inestabilidad alta | Restart |
| 0.8 - 1.0 | Caótico | Escalate |

### Análisis de Caos con Julia

El sistema utiliza el módulo Julia FFI para análisis avanzado:

```julia
function analyze_chaos(error_counts::Vector{Int})
    # Calcular exponente de Lyapunov
    lyapunov = calculate_lyapunov_exponent(error_counts)
    
    # Detectar atractores extraños
    attractors = detect_strange_attractors(error_counts)
    
    return Dict(
        "lyapunov_exponent" => lyapunov,
        "attractors" => attractors,
        "chaos_level" => classify_chaos(lyapunov)
    )
end
```

Si Julia no está disponible, usa heurística de fallback.

## 🎯 Optimización Adaptativa

### Auto-optimización de Parámetros

```rust
// El sistema optimiza automáticamente sus parámetros
auto_manager.optimize_adaptive_parameters().await?;

// Ver parámetros actuales
let params = auto_manager.adaptive_params.read().await;
println!("Health check interval: {}ms", params.health_check_interval_ms);
println!("Prediction threshold: {:.2}", params.prediction_threshold);
println!("Learning rate: {:.4}", params.learning_rate);
```

### Parámetros Adaptativos

```rust
pub struct AdaptiveParameters {
    pub health_check_interval_ms: u64,        // Ajusta frecuencia de checks
    pub prediction_threshold: f64,             // Ajusta sensibilidad predicción
    pub auto_correction_aggressiveness: f64,   // Ajusta agresividad corrección
    pub learning_rate: f64,                    // Ajusta velocidad aprendizaje
    pub pattern_detection_sensitivity: f64,    // Ajusta detección de patrones
}
```

**Optimización automática basada en**:
- Tasa de auto-correcciones exitosas
- Accuracy de predicciones
- Actividad del sistema
- Feedback histórico

## 🧠 Aprendizaje Continuo

### Iniciar Learning Loop

```rust
// Inicia aprendizaje continuo en background
auto_manager.start_continuous_learning().await;

// El sistema ahora:
// - Detecta patrones cada 60 segundos
// - Almacena en SharedMemory
// - Actualiza métricas de aprendizaje
// - Evoluciona automáticamente
```

### Obtener Métricas en Tiempo Real

```rust
let metrics = auto_manager.get_realtime_metrics().await;

println!("📊 Metrics:");
println!("  Prediction Accuracy: {:.1}%", metrics.prediction_accuracy * 100.0);
println!("  Avg Response Time: {:.2}ms", metrics.avg_response_time_ms);
println!("  Auto-corrections: {}", metrics.auto_corrections_count);
println!("  Success Rate: {:.1}%", metrics.successful_corrections_rate * 100.0);
println!("  Learning Velocity: {:.3}", metrics.learning_velocity);
println!("  Active Patterns: {}", metrics.active_patterns);
```

### Generar Reporte de Aprendizaje

```rust
let report = auto_manager.generate_learning_report().await;
println!("{}", report);
```

Output:
```
🧠 MEMORY_P LEARNING SYSTEM REPORT

📊 Métricas del Sistema:
├─ Prediction Accuracy: 93.2%
├─ Avg Response Time: 12.34ms
├─ Auto-corrections: 45 (éxito: 91.1%)
├─ System Uptime: 86400s
├─ Learning Velocity: 0.234
├─ Events Processed: 1247
└─ Active Patterns: 12

🎯 Parámetros Adaptativos:
├─ Health Check Interval: 25000ms
├─ Prediction Threshold: 0.78
├─ Auto-correction Aggressiveness: 0.65
├─ Learning Rate: 0.0012
└─ Pattern Sensitivity: 0.70

✅ Estado: Sistema aprendiendo activamente
```

## 📊 Monitoreo

### Estado de Salud General

```rust
let health = auto_manager.get_overall_health();

match health {
    HealthStatus::Healthy => println!("✅ Sistema saludable"),
    HealthStatus::Degraded => println!("⚠️  Sistema degradado"),
    HealthStatus::Unhealthy => println!("❌ Sistema no saludable"),
    HealthStatus::Recovering => println!("🔄 Sistema recuperándose"),
}
```

### Estado Detallado

```rust
let status = auto_manager.get_detailed_status();
println!("{:#}", status);
```

Output JSON:
```json
{
  "protocol_version": "2026.1.0",
  "auto_managed": true,
  "always_on": true,
  "overall_health": "Healthy",
  "engines": [
    {
      "name": "qdrant",
      "status": "Healthy",
      "last_check": 5,
      "error_count": 0
    }
  ],
  "ffi_modules": [
    {
      "language": "julia",
      "status": "Healthy",
      "last_check": 5,
      "error_count": 0
    }
  ]
}
```

## 🔄 Workflows de Auto-mejora

### Workflow Completo

```rust
use std::time::Duration;
use tokio::time::interval;

// 1. Iniciar sistema
auto_manager.auto_start().await?;
auto_manager.start_continuous_learning().await;

// 2. Loop de auto-mejora
let mut check_interval = interval(Duration::from_secs(60));

loop {
    check_interval.tick().await;
    
    // Diagnósticos predictivos
    if let Ok(diagnostics) = auto_manager.run_predictive_diagnostics().await {
        for diag in diagnostics {
            // Auto-corrección si hay issues
            if !diag.issues.is_empty() && diag.issues[0].auto_correctable {
                let _ = auto_manager
                    .run_chaos_based_autocorrection(&diag.component)
                    .await;
            }
        }
    }
    
    // Optimización adaptativa (cada 5 minutos)
    if check_interval.period().as_secs() % 300 == 0 {
        let _ = auto_manager.optimize_adaptive_parameters().await;
    }
    
    // Reporte (cada hora)
    if check_interval.period().as_secs() % 3600 == 0 {
        let report = auto_manager.generate_learning_report().await;
        println!("{}", report);
    }
}
```

## 🔐 Seguridad y Rollback

### Rollback Automático

El sistema soporta rollback automático si una auto-corrección falla:

```rust
let config = ManagerConfig {
    rollback_on_failure: true,  // Habilitar rollback automático
    ..Default::default()
};
```

Cuando está habilitado:
1. Guarda snapshot antes de corrección
2. Ejecuta corrección
3. Si falla, restaura snapshot
4. Log del rollback en telemetría

### Límites de Auto-corrección

```rust
// Limitar auto-correcciones por hora
let config = AutoManagerConfig {
    max_auto_corrections_per_hour: 10,
    ..Default::default()
};
```

## 📈 Métricas y KPIs

### Métricas Clave

| Métrica | Descripción | Target |
|---------|-------------|--------|
| **Prediction Accuracy** | % de predicciones correctas | >95% |
| **Context Switch Time** | Tiempo de cambio de contexto | <10ms |
| **Auto-correction Rate** | % de correcciones exitosas | >90% |
| **Learning Velocity** | Velocidad de aprendizaje | >0.5 |
| **System Uptime** | Tiempo sin fallos | 99.9%+ |

### Exportar Métricas

```rust
// Integración con Prometheus
let metrics = auto_manager.get_realtime_metrics().await;

// Exportar a formato Prometheus
println!("# HELP memory_p_prediction_accuracy Prediction accuracy");
println!("# TYPE memory_p_prediction_accuracy gauge");
println!("memory_p_prediction_accuracy {}", metrics.prediction_accuracy);

println!("# HELP memory_p_auto_corrections Total auto-corrections");
println!("# TYPE memory_p_auto_corrections counter");
println!("memory_p_auto_corrections {}", metrics.auto_corrections_count);
```

## 🛠️ Troubleshooting

### Problema: Auto-correcciones fallando

**Solución 1**: Reducir agresividad
```rust
let mut params = auto_manager.adaptive_params.write().await;
params.auto_correction_aggressiveness = 0.3; // Más conservador
```

**Solución 2**: Verificar Julia FFI
```rust
// Si Julia no está disponible, usa fallback
// El sistema detecta automáticamente y usa heurística
```

### Problema: Baja Learning Velocity

**Solución**: Aumentar sensibilidad de detección
```rust
let mut params = auto_manager.adaptive_params.write().await;
params.pattern_detection_sensitivity = 0.4; // Más sensible
params.learning_rate = 0.002; // Aprender más rápido
```

### Problema: Demasiados health checks

**Solución**: Aumentar intervalo
```rust
let mut params = auto_manager.adaptive_params.write().await;
params.health_check_interval_ms = 60000; // Cada minuto
```

## 📚 Referencias

- [Prediction Engine Guide](prediction_engine.md)
- [Telemetry System Guide](telemetry.md)
- [Pattern Detection Guide](pattern_detection.md)
- [Julia FFI Integration](julia_ffi.md)
- [API Reference](api.md)

## 🤝 Contributing

Para contribuir al AutoManager:

1. Fork el repositorio
2. Crear feature branch
3. Añadir tests
4. Submit PR

Ver [CONTRIBUTING.md](../CONTRIBUTING.md) para más detalles.

---

**¿Preguntas?** Abre un issue en GitHub o contacta al equipo.

🧠 **"El sistema que nunca deja de mejorar."** 🧠
