# 🧠 MEMORY_P Learning System v2.0 - Sistema de Aprendizaje Continuo

## 🎯 Overview

El **Sistema de Aprendizaje Continuo** es una extensión avanzada de MEMORY_P que garantiza que el sistema mejore automáticamente con cada interacción, adaptándose a los patrones del usuario **sin intervención manual**.

### Características Principales

✅ **Always-On**: Sistema que nunca deja de funcionar ni de aprender  
✅ **Auto-diagnóstico**: Detecta y resuelve problemas predictivamente  
✅ **Aprendizaje Continuo**: Detecta patrones de usuario automáticamente  
✅ **Auto-corrección**: Repara issues usando teoría del caos (Julia)  
✅ **Optimización Adaptativa**: Ajusta parámetros en tiempo real  
✅ **Telemetría Completa**: ClickHouse + Prometheus para analytics  
✅ **Zero-Touch**: No requiere intervención manual en producción  

## 🏗️ Arquitectura

```
┌─────────────────────────────────────────────────────────┐
│                    AutoManager                          │
│  (Orquestación, Diagnósticos, Auto-corrección)         │
└─────────────────┬───────────────────────────────────────┘
                  │
     ┌────────────┼────────────┬─────────────┬───────────┐
     │            │            │             │           │
┌────▼────┐ ┌────▼────┐ ┌─────▼─────┐ ┌────▼────┐ ┌────▼────┐
│ Pattern │ │Episodic │ │Prediction │ │Telemetry│ │Knowledge│
│Detector │ │ Memory  │ │  Engine   │ │ System  │ │  Graph  │
└─────────┘ └─────────┘ └───────────┘ └─────────┘ └─────────┘
```

## 🚀 Quick Start

### Instalación

```toml
[dependencies]
memory_p = "0.2.0"
tokio = { version = "1", features = ["full"] }
```

### Uso Básico

```rust
use memory_p::auto_manager::{AutoManager, ManagerConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Crear configuración
    let config = ManagerConfig::default();
    
    // 2. Inicializar AutoManager
    let auto_manager = AutoManager::new(config);
    
    // 3. Iniciar sistema always-on
    auto_manager.auto_start().await?;
    
    // 4. Iniciar aprendizaje continuo
    auto_manager.start_continuous_learning().await;
    
    // ✅ El sistema ahora se auto-gestiona y aprende continuamente
    
    Ok(())
}
```

## 📦 Componentes

### 1. AutoManager 🤖

**Orquestador principal** que coordina todos los subsistemas.

**Responsabilidades**:
- Health monitoring de todos los componentes
- Diagnósticos predictivos en tiempo real
- Auto-corrección basada en caos (Julia FFI)
- Optimización adaptativa de parámetros
- Coordinación del learning loop

**API Principal**:
```rust
// Iniciar sistema
auto_manager.auto_start().await?;

// Diagnósticos
let diagnostics = auto_manager.run_predictive_diagnostics().await?;

// Auto-corrección
let result = auto_manager.run_chaos_based_autocorrection("component").await?;

// Optimización
auto_manager.optimize_adaptive_parameters().await?;

// Métricas
let metrics = auto_manager.get_realtime_metrics().await;

// Reporte
let report = auto_manager.generate_learning_report().await;
```

📚 [Ver Guía Completa](docs/auto_manager.md)

### 2. PatternDetector 🔍

**Detecta patrones de usuario** automáticamente.

**Detecta**:
- **Patrones temporales**: Horarios de trabajo, días preferidos
- **Estilo de código**: Convenciones, documentación, error handling
- **Uso de herramientas**: Editores, lenguajes, MCP tools
- **Workflows**: Secuencias típicas de acciones

**API**:
```rust
use memory_p::pattern_detector::{PatternDetector, UserAction};

let detector = PatternDetector::new();

// Registrar acción
detector.record_action("user_id", UserAction {
    timestamp: Utc::now(),
    action_type: "edit".to_string(),
    tool: "vscode".to_string(),
    language: Some("rust".to_string()),
    success: true,
    duration_secs: 45.0,
}).await;

// Detectar patrones
let patterns = detector.detect_patterns("user_id").await?;

// Generar reporte
let report = detector.generate_pattern_report("user_id").await;
```

**Output ejemplo**:
```
👤 PATRONES DE USUARIO: rigohl

⏰ Temporal:
├─ Horario de Trabajo: [9, 10, 11, 14, 15, 16, 17]
├─ Días Preferidos: [2, 3, 4] (Martes, Miércoles, Jueves)
├─ Duración de Sesión: 120.5 min
└─ Commits/Semana: 45.2

💻 Estilo de Código:
├─ Naming: SnakeCase
├─ Documentación: Comprehensive
├─ Error Handling: ResultType
└─ Testing Coverage: 80%

🛠️ Uso de Herramientas:
├─ Editores: {"vscode": 0.6, "cursor": 0.4}
├─ Lenguajes: {"rust": 0.7, "julia": 0.2, "python": 0.1}
└─ Git: frequent commits

🔄 Workflows Típicos: 5 detectados

📊 Confidence: 87.3%
```

### 3. TelemetrySystem 📊

**Telemetría completa** con ClickHouse y Prometheus.

**Features**:
- Batch processing de eventos
- Métricas en tiempo real (Prometheus)
- Analytics histórico (ClickHouse)
- Auto-flush configurable

**API**:
```rust
use memory_p::telemetry::{TelemetrySystem, TelemetryConfig, TelemetryEvent};

let telemetry = TelemetrySystem::new(TelemetryConfig::default());
telemetry.start().await?;

// Registrar evento
telemetry.record_event(TelemetryEvent {
    timestamp: now(),
    event_type: "user_action".to_string(),
    component: "pattern_detector".to_string(),
    metrics: json!({"count": 1}),
    tags: vec![],
}).await;

// Métricas
telemetry.increment_requests(true).await;
telemetry.record_latency(12.5).await;

// Snapshot
let snapshot = telemetry.get_metrics_snapshot().await;
```

### 4. PredictionEngine 🔮

**Motor de predicción** integrado (ya existente, extendido).

**Predice**:
- Probabilidad de éxito de operaciones
- Tiempo de ejecución estimado
- Uso de recursos
- Impacto en el sistema

**Integración**:
```rust
use memory_p::prediction_engine::{PredictionEngine, PredictionType};

let engine = PredictionEngine::new();

let prediction = engine.predict(&context, PredictionType::SuccessProbability).await?;

if prediction.is_safe() {
    println!("✅ Operación segura ({}% confidence)", prediction.confidence * 100.0);
} else {
    println!("⚠️  Riesgo alto: {}", prediction.recommendation);
}
```

### 5. SharedMemory 💾

**Memoria compartida** entre agentes (ya existente, extendido).

Ahora almacena:
- Contextos de usuario
- Patrones detectados
- Métricas de aprendizaje
- Knowledge graph

## 📊 Métricas y KPIs

### Métricas Clave

| Métrica | Descripción | Baseline | Target | Actual |
|---------|-------------|----------|--------|--------|
| **Prediction Accuracy** | % predicciones correctas | 67% | 95%+ | - |
| **Context Switch Time** | Tiempo cambio contexto | 89ms | <10ms | - |
| **Auto-correction Rate** | % correcciones exitosas | - | 90%+ | - |
| **Learning Velocity** | Velocidad de aprendizaje | - | >0.5 | - |
| **User Satisfaction** | Satisfacción del usuario | 3.2/5 | 4.5/5+ | - |

### Evolución Esperada

```
Prediction Accuracy
100% ┤                                        ╭─────
 95% ┤                               ╭────────╯
 90% ┤                      ╭────────╯
 85% ┤              ╭───────╯
 80% ┤      ╭───────╯
 75% ┤  ╭───╯
 70% ┼──╯
     └────────────────────────────────────────────>
      0    1w    1m    3m    6m   12m   18m   24m
```

## 🔄 Workflows

### Workflow Completo de Auto-mejora

```rust
use tokio::time::{interval, Duration};

// 1. Inicializar sistema
auto_manager.auto_start().await?;
auto_manager.start_continuous_learning().await;

// 2. Loop de auto-mejora
let mut ticker = interval(Duration::from_secs(60));

loop {
    ticker.tick().await;
    
    // Diagnósticos predictivos
    let diagnostics = auto_manager.run_predictive_diagnostics().await?;
    
    // Auto-corrección si necesario
    for diag in diagnostics {
        if !diag.issues.is_empty() && diag.issues[0].auto_correctable {
            auto_manager.run_chaos_based_autocorrection(&diag.component).await?;
        }
    }
    
    // Optimización adaptativa (cada 5 min)
    if ticker.period().as_secs() % 300 == 0 {
        auto_manager.optimize_adaptive_parameters().await?;
    }
    
    // Reporte (cada hora)
    if ticker.period().as_secs() % 3600 == 0 {
        let report = auto_manager.generate_learning_report().await;
        println!("{}", report);
    }
}
```

## 🔧 Configuración

### ManagerConfig

```rust
use std::time::Duration;
use memory_p::auto_manager::ManagerConfig;

let config = ManagerConfig {
    check_interval: Duration::from_secs(30),  // Health checks
    max_errors: 3,                            // Antes de recovery
    recovery_timeout: Duration::from_secs(10),// Timeout recovery
    auto_restart: true,                       // Auto-restart habilitado
};
```

### TelemetryConfig

```rust
use memory_p::telemetry::TelemetryConfig;

let config = TelemetryConfig {
    clickhouse_url: "http://localhost:8123".to_string(),
    prometheus_port: 9090,
    enable_metrics: true,
    enable_tracing: true,
    batch_size: 1000,
    flush_interval_secs: 10,
};
```

## 🧪 Testing

```bash
# Tests unitarios
cargo test

# Tests de integración
cargo test --test '*'

# Tests con output
cargo test -- --nocapture

# Benchmarks
cargo bench
```

## 📈 Monitoreo en Producción

### Prometheus Metrics

```prometheus
# Prediction accuracy
memory_p_prediction_accuracy{component="auto_manager"} 0.932

# Auto-corrections
memory_p_auto_corrections_total{result="success"} 45
memory_p_auto_corrections_total{result="failure"} 5

# Learning velocity
memory_p_learning_velocity{user="rigohl"} 0.567

# Response times
memory_p_response_time_seconds{quantile="0.5"} 0.012
memory_p_response_time_seconds{quantile="0.95"} 0.045
memory_p_response_time_seconds{quantile="0.99"} 0.089
```

### ClickHouse Queries

```sql
-- Eventos por tipo (últimas 24h)
SELECT 
    event_type,
    count() as total,
    avg(JSONExtractFloat(metrics, 'duration_ms')) as avg_duration
FROM telemetry_events
WHERE timestamp > now() - INTERVAL 1 DAY
GROUP BY event_type
ORDER BY total DESC;

-- Patrones de usuario más comunes
SELECT 
    JSONExtractString(metrics, 'user_id') as user,
    JSONExtractString(metrics, 'pattern_type') as pattern,
    count() as frequency
FROM telemetry_events
WHERE event_type = 'pattern_detected'
GROUP BY user, pattern
ORDER BY frequency DESC
LIMIT 10;
```

## 🛠️ Troubleshooting

### Problema: Auto-correcciones fallando

```rust
// Reducir agresividad
let mut params = auto_manager.adaptive_params.write().await;
params.auto_correction_aggressiveness = 0.3;
```

### Problema: Baja learning velocity

```rust
// Aumentar sensibilidad
let mut params = auto_manager.adaptive_params.write().await;
params.pattern_detection_sensitivity = 0.4;
params.learning_rate = 0.002;
```

### Problema: Julia FFI no disponible

El sistema detecta automáticamente y usa fallback heurístico. Para habilitar Julia:

```bash
# Instalar Julia
wget https://julialang-s3.julialang.org/bin/linux/x64/1.9/julia-1.9.0-linux-x86_64.tar.gz
tar -xvzf julia-1.9.0-linux-x86_64.tar.gz
export PATH="$PATH:$(pwd)/julia-1.9.0/bin"

# Verificar
julia --version
```

## 📚 Documentación Completa

- **[Auto Manager Guide](docs/auto_manager.md)** - Guía completa del AutoManager
- **[Telemetry System](docs/telemetry.md)** - Sistema de telemetría
- **[Pattern Detection](docs/pattern_detection.md)** - Detección de patrones
- **[API Reference](docs/api.md)** - Referencia completa de la API

## 🎓 Conceptos Avanzados

### Teoría del Caos en Auto-corrección

El sistema usa análisis de caos (exponente de Lyapunov) para determinar la estabilidad de componentes:

```julia
function analyze_chaos(error_counts::Vector{Int})
    # Calcular exponente de Lyapunov
    λ = lyapunov_exponent(error_counts)
    
    # Clasificar
    if λ < 0
        return "stable"      # Sistema converge
    elseif λ ≈ 0
        return "neutral"     # Sistema periódico
    else
        return "chaotic"     # Sistema diverge → restart
    end
end
```

### Optimización Adaptativa

Usa gradiente descendente con momentum para optimizar parámetros:

```rust
// Momentum update
new_value = momentum * old_value + (1 - momentum) * optimized_value

// Con constraints
new_value = new_value.clamp(min_value, max_value)
```

### Pattern Mining

Usa sequence mining para detectar workflows:

```rust
// Detectar secuencias frecuentes
let sequences = mine_frequent_sequences(actions, min_support=5);

// Filtrar por confidence
let patterns = sequences.filter(|s| s.confidence > 0.75);
```

## 🤝 Contributing

Contribuciones bienvenidas! Ver [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

MIT License - Ver [LICENSE](LICENSE)

## 🙏 Credits

- **Rigohl** - Arquitectura y implementación principal
- **MEMORY_P Team** - Desarrollo y testing
- **Julia Community** - FFI y chaos analysis
- **Rust Community** - Async runtime y tooling

---

🧠 **"El conocimiento no es estático; evoluciona con cada interacción."** 🧠

**MEMORY_P Learning System v2.0** - Always learning, always improving.
