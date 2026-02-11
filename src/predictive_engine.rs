//! predictive_engine.rs - Motor de Predicción Extendida
//!
//! Sistema avanzado de predicción para:
//! - Optimización de rutas de ejecución
//! - Autocorrección de resultados adversos
//! - Priorización dinámica basada en histórico

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::error::{MemoryPError as Error, Result};

/// Tipo de optimización sugerida
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationType {
    /// Optimización de ruta de ejecución
    ExecutionPath,
    /// Optimización de uso de memoria
    MemoryUsage,
    /// Optimización de I/O
    IOOperations,
    /// Optimización de concurrencia
    Concurrency,
    /// Optimización de caché
    Caching,
}

/// Sugerencia de optimización
#[derive(Debug, Clone)]
pub struct Optimization {
    /// Tipo de optimización
    pub optimization_type: OptimizationType,
    /// Descripción
    pub description: String,
    /// Prioridad (0-100)
    pub priority: u8,
    /// Impacto estimado (porcentaje de mejora)
    pub estimated_impact: f64,
    /// Confianza de la predicción (0.0-1.0)
    pub confidence: f64,
}

/// Resultado adverso detectado
#[derive(Debug, Clone)]
pub struct AdverseResult {
    /// Descripción del problema
    pub description: String,
    /// Severidad (0-10)
    pub severity: u8,
    /// Corrección sugerida
    pub suggested_correction: Option<String>,
}

/// Histórico de ejecuciones
#[derive(Debug, Clone)]
struct ExecutionHistory {
    /// Número total de ejecuciones
    total_executions: u64,
    /// Ejecuciones exitosas
    successful_executions: u64,
    /// Ejecuciones fallidas
    failed_executions: u64,
    /// Tiempo promedio de ejecución (ms)
    avg_execution_time: f64,
    /// Patrones detectados
    patterns: HashMap<String, u32>,
}

impl Default for ExecutionHistory {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            avg_execution_time: 0.0,
            patterns: HashMap::new(),
        }
    }
}

/// Motor de predicción extendida
pub struct PredictiveEngine {
    /// Histórico de ejecuciones
    history: Arc<RwLock<ExecutionHistory>>,
    /// Caché de predicciones
    #[allow(dead_code)]
    prediction_cache: Arc<RwLock<HashMap<String, Vec<Optimization>>>>,
}

impl PredictiveEngine {
    /// Crea un nuevo motor predictivo
    pub fn new() -> Self {
        info!("🔮 Inicializando Motor Predictivo...");

        Self {
            history: Arc::new(RwLock::new(ExecutionHistory::default())),
            prediction_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Sugiere optimizaciones basadas en el histórico
    pub async fn suggest_optimizations(&self) -> Result<Vec<Optimization>> {
        debug!("🎯 Generando sugerencias de optimización...");

        let history = self.history.read().await;
        let mut optimizations = Vec::new();

        // Análisis de tasa de éxito
        if history.total_executions > 0 {
            let success_rate =
                history.successful_executions as f64 / history.total_executions as f64;

            if success_rate < 0.8 {
                optimizations.push(Optimization {
                    optimization_type: OptimizationType::ExecutionPath,
                    description: format!(
                        "Tasa de éxito baja ({:.1}%). Considerar refactorización.",
                        success_rate * 100.0
                    ),
                    priority: 90,
                    estimated_impact: (1.0 - success_rate) * 50.0,
                    confidence: 0.85,
                });
            }
        }

        // Análisis de tiempo de ejecución
        if history.avg_execution_time > 1000.0 {
            optimizations.push(Optimization {
                optimization_type: OptimizationType::Concurrency,
                description: format!(
                    "Tiempo promedio alto ({:.0}ms). Considerar paralelización.",
                    history.avg_execution_time
                ),
                priority: 75,
                estimated_impact: 40.0,
                confidence: 0.78,
            });
        }

        // Análisis de patrones
        for (pattern, count) in history.patterns.iter() {
            if *count > 100 {
                optimizations.push(Optimization {
                    optimization_type: OptimizationType::Caching,
                    description: format!(
                        "Patrón '{}' detectado {} veces. Considerar caché.",
                        pattern, count
                    ),
                    priority: 60,
                    estimated_impact: 30.0,
                    confidence: 0.92,
                });
            }
        }

        // Ordenar por prioridad
        optimizations.sort_by(|a, b| b.priority.cmp(&a.priority));

        info!("✅ Generadas {} optimizaciones", optimizations.len());
        Ok(optimizations)
    }

    /// Predice y corrige resultados adversos
    pub async fn detect_and_correct_adverse_results(
        &self,
        context: &str,
    ) -> Result<Vec<AdverseResult>> {
        debug!("🔍 Detectando resultados adversos en contexto: {}", context);

        let mut adverse_results = Vec::new();

        // Análisis de contexto para detectar patrones adversos
        if context.contains("error") || context.contains("failed") {
            adverse_results.push(AdverseResult {
                description: "Patrón de error detectado en contexto".to_string(),
                severity: 7,
                suggested_correction: Some("Implementar retry con backoff exponencial".to_string()),
            });
        }

        if context.contains("timeout") {
            adverse_results.push(AdverseResult {
                description: "Timeouts detectados".to_string(),
                severity: 8,
                suggested_correction: Some("Incrementar timeout o optimizar operación".to_string()),
            });
        }

        if context.contains("memory") && context.contains("leak") {
            adverse_results.push(AdverseResult {
                description: "Posible memory leak detectado".to_string(),
                severity: 9,
                suggested_correction: Some(
                    "Revisar ciclos de vida de objetos y referencias".to_string(),
                ),
            });
        }

        if !adverse_results.is_empty() {
            warn!(
                "⚠️  Detectados {} resultados adversos",
                adverse_results.len()
            );
        }

        Ok(adverse_results)
    }

    /// Calcula prioridad dinámica basada en histórico
    pub async fn calculate_dynamic_priority(&self, task_id: &str, base_priority: u8) -> Result<u8> {
        debug!("📊 Calculando prioridad dinámica para: {}", task_id);

        let history = self.history.read().await;

        // Ajustar prioridad basado en histórico
        let mut adjusted_priority = base_priority as f64;

        // Si hay muchos fallos, aumentar prioridad de tareas de mantenimiento
        if history.total_executions > 0 {
            let failure_rate = history.failed_executions as f64 / history.total_executions as f64;
            if failure_rate > 0.2 {
                adjusted_priority *= 1.3;
            }
        }

        // Si el tiempo promedio es alto, priorizar optimizaciones
        if history.avg_execution_time > 500.0 {
            if task_id.contains("optimize") || task_id.contains("performance") {
                adjusted_priority *= 1.5;
            }
        }

        // Limitar a rango válido
        let final_priority = adjusted_priority.clamp(0.0, 100.0) as u8;

        debug!(
            "✅ Prioridad ajustada: {} -> {}",
            base_priority, final_priority
        );
        Ok(final_priority)
    }

    /// Registra ejecución en el histórico
    pub async fn record_execution(
        &self,
        success: bool,
        execution_time_ms: f64,
        pattern: Option<String>,
    ) -> Result<()> {
        let mut history = self.history.write().await;

        history.total_executions += 1;
        if success {
            history.successful_executions += 1;
        } else {
            history.failed_executions += 1;
        }

        // Actualizar tiempo promedio (media móvil)
        let alpha = 0.2; // Factor de suavizado
        history.avg_execution_time =
            alpha * execution_time_ms + (1.0 - alpha) * history.avg_execution_time;

        // Registrar patrón si existe
        if let Some(p) = pattern {
            *history.patterns.entry(p).or_insert(0) += 1;
        }

        Ok(())
    }

    /// Predice ruta óptima de ejecución
    pub async fn predict_optimal_path(&self, available_paths: Vec<&str>) -> Result<String> {
        debug!(
            "🛤️  Prediciendo ruta óptima entre {} opciones",
            available_paths.len()
        );

        if available_paths.is_empty() {
            return Err(Error::Other("No hay rutas disponibles".into()));
        }

        let history = self.history.read().await;

        // Análisis simple: preferir rutas que aparecen en patrones exitosos
        for path in &available_paths {
            if let Some(count) = history.patterns.get(*path) {
                if *count > 10 {
                    info!(
                        "✅ Ruta óptima seleccionada: {} (usada {} veces)",
                        path, count
                    );
                    return Ok(path.to_string());
                }
            }
        }

        // Si no hay histórico, retornar primera opción
        let default_path = available_paths[0].to_string();
        info!("ℹ️  Usando ruta por defecto: {}", default_path);
        Ok(default_path)
    }

    /// Obtiene estadísticas del motor predictivo
    pub async fn get_statistics(&self) -> Result<HashMap<String, f64>> {
        let history = self.history.read().await;
        let mut stats = HashMap::new();

        stats.insert(
            "total_executions".to_string(),
            history.total_executions as f64,
        );
        stats.insert(
            "successful_executions".to_string(),
            history.successful_executions as f64,
        );
        stats.insert(
            "failed_executions".to_string(),
            history.failed_executions as f64,
        );
        stats.insert(
            "avg_execution_time_ms".to_string(),
            history.avg_execution_time,
        );

        if history.total_executions > 0 {
            let success_rate =
                history.successful_executions as f64 / history.total_executions as f64;
            stats.insert("success_rate".to_string(), success_rate);
        }

        Ok(stats)
    }
}

impl Default for PredictiveEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_predictive_engine_creation() {
        let engine = PredictiveEngine::new();
        let stats = engine.get_statistics().await.unwrap();
        assert_eq!(stats.get("total_executions").unwrap(), &0.0);
    }

    #[tokio::test]
    async fn test_record_execution() {
        let engine = PredictiveEngine::new();

        engine
            .record_execution(true, 100.0, Some("test_pattern".to_string()))
            .await
            .unwrap();

        let stats = engine.get_statistics().await.unwrap();
        assert_eq!(stats.get("total_executions").unwrap(), &1.0);
        assert_eq!(stats.get("successful_executions").unwrap(), &1.0);
    }

    #[tokio::test]
    async fn test_suggest_optimizations() {
        let engine = PredictiveEngine::new();

        // Registrar varias ejecuciones lentas
        for _ in 0..10 {
            engine.record_execution(true, 1500.0, None).await.unwrap();
        }

        let optimizations = engine.suggest_optimizations().await.unwrap();
        assert!(!optimizations.is_empty());
    }

    #[tokio::test]
    async fn test_detect_adverse_results() {
        let engine = PredictiveEngine::new();

        let adverse = engine
            .detect_and_correct_adverse_results("error: timeout occurred")
            .await
            .unwrap();
        assert!(!adverse.is_empty());
    }

    #[tokio::test]
    async fn test_calculate_dynamic_priority() {
        let engine = PredictiveEngine::new();

        let priority = engine
            .calculate_dynamic_priority("test_task", 50)
            .await
            .unwrap();
        assert!(priority >= 0 && priority <= 100);
    }

    #[tokio::test]
    async fn test_predict_optimal_path() {
        let engine = PredictiveEngine::new();

        let paths = vec!["path_a", "path_b", "path_c"];
        let optimal = engine.predict_optimal_path(paths).await.unwrap();
        assert!(!optimal.is_empty());
    }
}
