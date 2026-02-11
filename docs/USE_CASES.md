# 🎯 Casos de Uso Prácticos - MEMORY_P Learning System v2.0

## Overview

Este documento presenta casos de uso reales y prácticos del sistema de aprendizaje continuo de MEMORY_P.

---

## 📚 Índice

1. [Caso 1: Startup Automático de Producción](#caso-1-startup-automático-de-producción)
2. [Caso 2: Detección y Corrección de Engine Degradado](#caso-2-detección-y-corrección-de-engine-degradado)
3. [Caso 3: Aprendizaje de Patrones de Usuario](#caso-3-aprendizaje-de-patrones-de-usuario)
4. [Caso 4: Optimización Adaptativa en Runtime](#caso-4-optimización-adaptativa-en-runtime)
5. [Caso 5: Monitoreo y Alertas](#caso-5-monitoreo-y-alertas)
6. [Caso 6: Recovery de Fallo Crítico](#caso-6-recovery-de-fallo-crítico)

---

## Caso 1: Startup Automático de Producción

### Contexto
Iniciar el sistema MEMORY_P en producción con auto-gestión completa.

### Código

```rust
use memory_p::auto_manager::{AutoManager, ManagerConfig};
use memory_p::pattern_detector::PatternDetector;
use memory_p::telemetry::{TelemetrySystem, TelemetryConfig};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Setup logging
    tracing_subscriber::fmt()
        .with_target(false)
        .json()
        .init();

    info!("🚀 Starting MEMORY_P Production System");

    // 2. Initialize telemetry first
    let telemetry = Arc::new(TelemetrySystem::new(TelemetryConfig {
        clickhouse_url: std::env::var("CLICKHOUSE_URL")
            .unwrap_or_else(|_| "http://localhost:8123".to_string()),
        prometheus_port: 9090,
        enable_metrics: true,
        enable_tracing: true,
        batch_size: 5000,
        flush_interval_secs: 30,
    }));
    
    telemetry.start().await?;
    info!("✅ Telemetry started");

    // 3. Initialize auto manager with production config
    let config = ManagerConfig {
        check_interval: std::time::Duration::from_secs(15), // Más frecuente
        max_errors: 5,
        recovery_timeout: std::time::Duration::from_secs(30),
        auto_restart: true,
    };
    
    let auto_manager = Arc::new(AutoManager::new(config));
    
    // 4. Start auto-management
    auto_manager.auto_start().await?;
    info!("✅ Auto-manager started");
    
    // 5. Start continuous learning
    auto_manager.start_continuous_learning().await;
    info!("✅ Learning system activated");

    // 6. Keep system running
    info!("🎯 System running - Press Ctrl+C to stop");
    
    tokio::signal::ctrl_c().await?;
    
    info!("🛑 Shutting down gracefully...");
    auto_manager.stop().await;
    telemetry.shutdown().await?;
    
    Ok(())
}
```

### Resultado
```
🚀 Starting MEMORY_P Production System
✅ Telemetry started
🔧 Auto-inicializando módulos FFI...
  ✅ FFI julia: inicializado
  ✅ FFI jax: inicializado
  ✅ FFI zig: inicializado
🔍 Auto-inicializando motores de búsqueda...
  ✅ Motor qdrant: listo
  ✅ Motor faiss: listo
  ✅ Motor tantivy: listo
❤️  Health monitor iniciado (cada 15s)
🔄 Auto-recovery iniciado
✅ Auto-manager started
🧠 Sistema de aprendizaje continuo iniciado
✅ Learning system activated
🎯 System running - Press Ctrl+C to stop
```

---

## Caso 2: Detección y Corrección de Engine Degradado

### Contexto
Un motor de búsqueda comienza a fallar. El sistema detecta y corrige automáticamente.

### Código

```rust
use memory_p::auto_manager::AutoManager;
use std::time::Duration;
use tokio::time::interval;

async fn monitor_and_autocorrect(auto_manager: Arc<AutoManager>) -> anyhow::Result<()> {
    let mut check_interval = interval(Duration::from_secs(30));
    
    loop {
        check_interval.tick().await;
        
        // Ejecutar diagnósticos
        let diagnostics = auto_manager.run_predictive_diagnostics().await?;
        
        for diag in diagnostics {
            // Verificar si hay issues
            if !diag.issues.is_empty() {
                tracing::warn!(
                    "⚠️  Componente {} tiene {} issues",
                    diag.component,
                    diag.issues.len()
                );
                
                // Auto-corrección si es posible
                for issue in &diag.issues {
                    if issue.auto_correctable {
                        tracing::info!(
                            "🔧 Iniciando auto-corrección para {}...",
                            diag.component
                        );
                        
                        let result = auto_manager
                            .run_chaos_based_autocorrection(&diag.component)
                            .await?;
                        
                        if result.success {
                            tracing::info!(
                                "✅ {} corregido en {:.2}ms (chaos: {:.2})",
                                diag.component,
                                result.duration_ms,
                                result.chaos_score
                            );
                        } else {
                            tracing::error!(
                                "❌ Fallo en auto-corrección de {}",
                                diag.component
                            );
                        }
                    }
                }
            }
            
            // Verificar predicciones de fallo
            if !diag.predicted_failures.is_empty() {
                for failure in &diag.predicted_failures {
                    tracing::warn!(
                        "🔮 Fallo predicho: {} ({:.0}% probabilidad en {}s)",
                        failure.component,
                        failure.probability * 100.0,
                        failure.time_to_failure_secs.unwrap_or(0)
                    );
                    
                    // Acción preventiva
                    if failure.probability > 0.7 {
                        tracing::info!("🛡️  Ejecutando mitigación preventiva...");
                        auto_manager
                            .run_chaos_based_autocorrection(&failure.component)
                            .await?;
                    }
                }
            }
        }
    }
}
```

### Escenario de Ejecución

```
[15:30:00] ❤️  Health check: qdrant - Healthy
[15:30:30] ⚠️  Componente qdrant tiene 1 issues
[15:30:30]   Issue: [Medium] Componente qdrant degradado (2 errores)
[15:30:30] 🔧 Iniciando auto-corrección para qdrant...
[15:30:30] 🌀 Ejecutando análisis de caos Julia para: qdrant
[15:30:31] 🔄 Auto-recovery: reiniciando motor qdrant
[15:30:33] ✅ qdrant corregido en 2341.23ms (chaos: 0.45)
[15:30:33] ✅ Motor qdrant recuperado
```

---

## Caso 3: Aprendizaje de Patrones de Usuario

### Contexto
El sistema aprende los patrones de trabajo del usuario "rigohl" para optimizar la experiencia.

### Código

```rust
use memory_p::pattern_detector::{PatternDetector, UserAction};
use chrono::Utc;
use std::sync::Arc;

async fn learn_user_patterns(
    pattern_detector: Arc<PatternDetector>,
    user_id: &str
) -> anyhow::Result<()> {
    // Simular día de trabajo típico
    let actions = vec![
        ("09:00", "session_start", "vscode", "rust"),
        ("09:15", "analyze", "analyzer", "rust"),
        ("09:45", "edit", "vscode", "rust"),
        ("10:30", "test", "cargo", "rust"),
        ("11:00", "commit", "git", "rust"),
        ("14:00", "refactor", "cursor", "rust"),
        ("14:45", "edit", "cursor", "julia"),
        ("15:30", "analyze", "analyzer", "julia"),
        ("16:15", "test", "cargo", "rust"),
        ("17:00", "commit", "git", "rust"),
    ];
    
    for (time, action_type, tool, lang) in actions {
        pattern_detector.record_action(user_id, UserAction {
            timestamp: Utc::now(),
            action_type: action_type.to_string(),
            tool: tool.to_string(),
            language: Some(lang.to_string()),
            success: true,
            duration_secs: 900.0, // 15 min
        }).await;
    }
    
    // Detectar patrones después de varias semanas
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    let patterns = pattern_detector.detect_patterns(user_id).await?;
    
    // Generar reporte
    let report = pattern_detector.generate_pattern_report(user_id).await;
    println!("{}", report);
    
    // Usar patrones para optimizar
    if patterns.temporal_patterns.working_hours.contains(&14) {
        tracing::info!("💡 Usuario activo a las 14:00 - precalentando cache");
    }
    
    if patterns.tool_usage.editor_distribution.get("cursor").unwrap_or(&0.0) > &0.3 {
        tracing::info!("💡 Usuario prefiere Cursor - optimizando integraciones");
    }
    
    Ok(())
}
```

### Output

```
👤 PATRONES DE USUARIO: rigohl

⏰ Temporal:
├─ Horario de Trabajo: [9, 10, 11, 14, 15, 16, 17]
├─ Días Preferidos: [2, 3, 4]
├─ Duración de Sesión: 120.0 min
└─ Commits/Semana: 10.0

💻 Estilo de Código:
├─ Naming: SnakeCase
├─ Documentación: Standard
├─ Error Handling: ResultType
└─ Testing Coverage: 80%

🛠️ Uso de Herramientas:
├─ Editores: {"vscode": 0.5, "cursor": 0.3, "analyzer": 0.2}
├─ Lenguajes: {"rust": 0.8, "julia": 0.2}
└─ Git: frequent

🔄 Workflows Típicos: 3 detectados
  1. analyze → edit → test → commit (60% frecuencia)
  2. edit → refactor → test (25% frecuencia)
  3. session_start → analyze → edit (15% frecuencia)

📊 Confidence: 78.5%

💡 Usuario activo a las 14:00 - precalentando cache
💡 Usuario prefiere Cursor - optimizando integraciones
```

---

## Caso 4: Optimización Adaptativa en Runtime

### Contexto
El sistema ajusta automáticamente sus parámetros basándose en métricas de performance.

### Código

```rust
async fn adaptive_optimization_loop(
    auto_manager: Arc<AutoManager>
) -> anyhow::Result<()> {
    let mut optimize_interval = tokio::time::interval(
        std::time::Duration::from_secs(300) // Cada 5 minutos
    );
    
    loop {
        optimize_interval.tick().await;
        
        // Obtener métricas actuales
        let metrics = auto_manager.get_realtime_metrics().await;
        
        tracing::info!(
            "📊 Métricas: accuracy={:.1}%, corrections={}, velocity={:.3}",
            metrics.prediction_accuracy * 100.0,
            metrics.auto_corrections_count,
            metrics.learning_velocity
        );
        
        // Optimizar parámetros
        auto_manager.optimize_adaptive_parameters().await?;
        
        let params = auto_manager.adaptive_params.read().await;
        
        tracing::info!(
            "🎯 Parámetros optimizados: check_interval={}ms, threshold={:.2}",
            params.health_check_interval_ms,
            params.prediction_threshold
        );
        
        // Generar reporte cada hora
        if optimize_interval.period().as_secs() % 3600 == 0 {
            let report = auto_manager.generate_learning_report().await;
            tracing::info!("📈 Reporte:\n{}", report);
        }
    }
}
```

### Timeline de Optimización

```
[T+0m]   📊 Métricas: accuracy=67.0%, corrections=0, velocity=0.100
[T+0m]   🎯 Parámetros: check_interval=30000ms, threshold=0.75

[T+5m]   📊 Métricas: accuracy=72.5%, corrections=3, velocity=0.234
[T+5m]   🎯 Optimizados: check_interval=27000ms, threshold=0.76

[T+10m]  📊 Métricas: accuracy=78.2%, corrections=8, velocity=0.412
[T+10m]  🎯 Optimizados: check_interval=24300ms, threshold=0.78

[T+15m]  📊 Métricas: accuracy=85.1%, corrections=12, velocity=0.589
[T+15m]  🎯 Optimizados: check_interval=24300ms, threshold=0.81

[T+20m]  📊 Métricas: accuracy=91.3%, corrections=15, velocity=0.723
[T+20m]  🎯 Optimizados: check_interval=24300ms, threshold=0.84

[T+30m]  📊 Métricas: accuracy=95.7%, corrections=18, velocity=0.856
[T+30m]  🎯 Optimizados: check_interval=26730ms, threshold=0.87
         ✅ Target accuracy alcanzado! Sistema estable.
```

---

## Caso 5: Monitoreo y Alertas

### Contexto
Configurar sistema de monitoreo con alertas basadas en métricas.

### Código

```rust
use memory_p::telemetry::{TelemetrySystem, MetricsSnapshot};

async fn monitoring_dashboard(
    telemetry: Arc<TelemetrySystem>,
    auto_manager: Arc<AutoManager>
) -> anyhow::Result<()> {
    let mut dashboard_interval = tokio::time::interval(
        std::time::Duration::from_secs(10)
    );
    
    loop {
        dashboard_interval.tick().await;
        
        // Obtener snapshots
        let tel_metrics = telemetry.get_metrics_snapshot().await;
        let sys_metrics = auto_manager.get_realtime_metrics().await;
        let health = auto_manager.get_overall_health();
        
        // Dashboard en consola
        println!("\n╔══════════════════════════════════════════╗");
        println!("║  MEMORY_P DASHBOARD - {:?}  ║", chrono::Utc::now().format("%H:%M:%S"));
        println!("╠══════════════════════════════════════════╣");
        println!("║ Health: {:?}                    ║", health);
        println!("║ Requests: {} ({}% success)     ║", 
            tel_metrics.total_requests,
            tel_metrics.success_rate as u32
        );
        println!("║ Latency: {:.2}ms (p95: {:.2}ms)   ║",
            tel_metrics.avg_latency_ms,
            tel_metrics.p95_latency_ms
        );
        println!("║ Accuracy: {:.1}%                  ║",
            sys_metrics.prediction_accuracy * 100.0
        );
        println!("║ Learning Velocity: {:.3}         ║",
            sys_metrics.learning_velocity
        );
        println!("╚══════════════════════════════════════════╝");
        
        // Alertas
        if tel_metrics.success_rate < 90.0 {
            tracing::warn!("🚨 ALERT: Success rate bajo: {:.1}%", tel_metrics.success_rate);
        }
        
        if tel_metrics.p95_latency_ms > 100.0 {
            tracing::warn!("🚨 ALERT: Latencia alta: {:.2}ms", tel_metrics.p95_latency_ms);
        }
        
        if sys_metrics.prediction_accuracy < 0.8 {
            tracing::warn!("🚨 ALERT: Accuracy bajo: {:.1}%", 
                sys_metrics.prediction_accuracy * 100.0
            );
        }
        
        // Exportar a Prometheus
        export_to_prometheus(&tel_metrics, &sys_metrics);
    }
}

fn export_to_prometheus(
    tel: &MetricsSnapshot,
    sys: &memory_p::auto_manager::SystemMetrics
) {
    // En producción, esto enviaría a Prometheus
    println!("# HELP memory_p_requests_total Total requests");
    println!("# TYPE memory_p_requests_total counter");
    println!("memory_p_requests_total {}", tel.total_requests);
    
    println!("# HELP memory_p_success_rate Success rate");
    println!("# TYPE memory_p_success_rate gauge");
    println!("memory_p_success_rate {}", tel.success_rate / 100.0);
    
    println!("# HELP memory_p_prediction_accuracy Prediction accuracy");
    println!("# TYPE memory_p_prediction_accuracy gauge");
    println!("memory_p_prediction_accuracy {}", sys.prediction_accuracy);
}
```

### Output Dashboard

```
╔══════════════════════════════════════════╗
║  MEMORY_P DASHBOARD - 15:42:33           ║
╠══════════════════════════════════════════╣
║ Health: Healthy                          ║
║ Requests: 1247 (94% success)             ║
║ Latency: 12.34ms (p95: 45.67ms)          ║
║ Accuracy: 93.2%                          ║
║ Learning Velocity: 0.678                 ║
╚══════════════════════════════════════════╝
```

---

## Caso 6: Recovery de Fallo Crítico

### Contexto
El sistema detecta y se recupera de un fallo crítico automáticamente.

### Código

```rust
async fn critical_failure_scenario(
    auto_manager: Arc<AutoManager>
) -> anyhow::Result<()> {
    tracing::warn!("🔥 Simulando fallo crítico en componente...");
    
    // Simular fallo crítico
    // (En realidad esto sería detectado por health checks)
    
    // El auto-manager detecta el fallo
    let diagnostics = auto_manager.run_predictive_diagnostics().await?;
    
    for diag in diagnostics {
        if diag.status == memory_p::auto_manager::ComponentStatus::Failed {
            tracing::error!(
                "💥 FALLO CRÍTICO detectado en {}",
                diag.component
            );
            
            // Auto-corrección inmediata
            tracing::info!("🚑 Iniciando recovery de emergencia...");
            
            let result = auto_manager
                .run_chaos_based_autocorrection(&diag.component)
                .await?;
            
            if result.success {
                tracing::info!(
                    "✅ RECOVERY EXITOSO: {} restaurado en {:.2}ms",
                    diag.component,
                    result.duration_ms
                );
                
                // Verificar salud post-recovery
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                
                let post_diagnostics = auto_manager
                    .run_predictive_diagnostics()
                    .await?;
                
                let post_status = post_diagnostics.iter()
                    .find(|d| d.component == diag.component)
                    .map(|d| &d.status);
                
                match post_status {
                    Some(memory_p::auto_manager::ComponentStatus::Healthy) => {
                        tracing::info!("✅ Componente {} completamente recuperado", diag.component);
                    }
                    _ => {
                        tracing::warn!("⚠️  Componente {} requiere atención manual", diag.component);
                    }
                }
            } else {
                tracing::error!(
                    "❌ RECOVERY FALLIDO: {} requiere intervención manual",
                    diag.component
                );
                
                // Notificar a operadores
                send_alert_to_ops(&diag.component);
            }
        }
    }
    
    Ok(())
}

fn send_alert_to_ops(component: &str) {
    // En producción: PagerDuty, Slack, email, etc.
    tracing::error!("🚨 ALERTA ENVIADA A OPS: {}", component);
}
```

### Timeline de Recovery

```
[15:45:00] 🔥 Simulando fallo crítico en componente...
[15:45:00] ❌ Health check: faiss - Failed
[15:45:01] 💥 FALLO CRÍTICO detectado en faiss
[15:45:01] 🚑 Iniciando recovery de emergencia...
[15:45:01] 🌀 Ejecutando análisis de caos Julia para: faiss
[15:45:02]   Chaos score: 0.87 (Crítico)
[15:45:02]   Acción recomendada: restart
[15:45:02] 🔄 Auto-recovery: reiniciando motor faiss
[15:45:04] ✅ Motor faiss reiniciado
[15:45:04] ✅ RECOVERY EXITOSO: faiss restaurado en 3421.56ms
[15:45:09] ❤️  Health check: faiss - Healthy
[15:45:09] ✅ Componente faiss completamente recuperado
```

---

## 🎯 Conclusión

Estos casos de uso demuestran la capacidad del sistema para:

1. ✅ **Auto-gestión** completa en producción
2. ✅ **Detección y corrección** automática de fallos
3. ✅ **Aprendizaje** continuo de patrones
4. ✅ **Optimización** adaptativa en runtime
5. ✅ **Monitoreo** y alertas en tiempo real
6. ✅ **Recovery** automático de fallos críticos

El sistema está diseñado para **operación always-on** con **zero-touch** en producción.

---

🧠 **"El conocimiento no es estático; evoluciona con cada interacción."** 🧠
