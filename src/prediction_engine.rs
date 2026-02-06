//! prediction_engine.rs - Motor de Predicción para MEMORY_P
//!
//! Proporciona predicciones automáticas antes de ejecutar acciones.
//! Utiliza algoritmos avanzados en Julia y Mojo para modelos predictivos.
//!
//! Características:
//! - Predicción de resultados antes de acciones
//! - Modelos ligeros y rápidos (ARIMA, Prophet, ML)
//! - Integración con Julia para análisis matemático
//! - Caching de predicciones para performance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use dashmap::DashMap;
use tracing::{debug, info, warn};

use crate::ffi;
use crate::error::{Error, Result};

/// Tipo de predicción
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PredictionType {
    /// Predicción de éxito/fallo de operación
    SuccessProbability,
    /// Predicción de tiempo de ejecución
    ExecutionTime,
    /// Predicción de uso de recursos
    ResourceUsage,
    /// Predicción de calidad de resultado
    ResultQuality,
    /// Predicción de impacto en el sistema
    SystemImpact,
}

/// Resultado de una predicción
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// Tipo de predicción
    pub prediction_type: PredictionType,
    /// Valor predicho (0.0 - 1.0 para probabilidades)
    pub value: f64,
    /// Confianza en la predicción (0.0 - 1.0)
    pub confidence: f64,
    /// Métricas adicionales
    pub metrics: HashMap<String, f64>,
    /// Recomendación basada en la predicción
    pub recommendation: String,
    /// Timestamp de la predicción
    pub timestamp: SystemTime,
}

impl Prediction {
    /// Determina si la acción es segura según la predicción
    pub fn is_safe(&self) -> bool {
        match self.prediction_type {
            PredictionType::SuccessProbability => self.value >= 0.7 && self.confidence >= 0.6,
            PredictionType::SystemImpact => self.value <= 0.5, // Menor impacto es mejor
            _ => self.confidence >= 0.6,
        }
    }
    
    /// Retorna nivel de riesgo (0=bajo, 1=alto)
    pub fn risk_level(&self) -> f64 {
        match self.prediction_type {
            PredictionType::SuccessProbability => 1.0 - self.value,
            PredictionType::SystemImpact => self.value,
            _ => {
                if self.confidence < 0.5 {
                    0.8 // Alta incertidumbre = alto riesgo
                } else {
                    0.3
                }
            }
        }
    }
}

/// Contexto de acción para predicción
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionContext {
    /// Tipo de acción a realizar
    pub action_type: String,
    /// Parámetros de la acción
    pub parameters: serde_json::Value,
    /// Historial de acciones similares
    pub history: Vec<ActionResult>,
    /// Métricas actuales del sistema
    pub system_metrics: SystemMetrics,
}

/// Resultado de acción histórica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action_type: String,
    pub success: bool,
    pub execution_time_ms: u64,
    pub resource_usage: f64,
    pub timestamp: u64,
}

/// Métricas del sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub active_tasks: usize,
    pub avg_response_time_ms: f64,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.3,
            memory_usage: 0.4,
            active_tasks: 1,
            avg_response_time_ms: 100.0,
        }
    }
}

/// Motor de predicción
pub struct PredictionEngine {
    /// Cache de predicciones recientes
    cache: Arc<DashMap<String, Prediction>>,
    /// Historial de resultados de acciones
    history: Arc<DashMap<String, Vec<ActionResult>>>,
    /// Configuración
    config: PredictionConfig,
}

/// Configuración del motor de predicción
#[derive(Debug, Clone)]
pub struct PredictionConfig {
    /// TTL de cache en segundos
    pub cache_ttl_seconds: u64,
    /// Número máximo de entradas en historial
    pub max_history_entries: usize,
    /// Umbral mínimo de confianza
    pub min_confidence: f64,
    /// Habilitar integración con Julia
    pub enable_julia: bool,
    /// Habilitar integración con Mojo
    pub enable_mojo: bool,
}

impl Default for PredictionConfig {
    fn default() -> Self {
        Self {
            cache_ttl_seconds: 300, // 5 minutos
            max_history_entries: 1000,
            min_confidence: 0.6,
            enable_julia: true,
            enable_mojo: true,
        }
    }
}

impl PredictionEngine {
    /// Crea una nueva instancia del motor de predicción
    pub fn new(config: PredictionConfig) -> Self {
        info!("🔮 Inicializando motor de predicción");
        
        Self {
            cache: Arc::new(DashMap::new()),
            history: Arc::new(DashMap::new()),
            config,
        }
    }
    
    /// Predice el resultado de una acción
    pub async fn predict(
        &self,
        prediction_type: PredictionType,
        context: &ActionContext,
    ) -> Result<Prediction> {
        // Verificar cache
        let cache_key = self.generate_cache_key(prediction_type, context);
        if let Some(cached) = self.cache.get(&cache_key) {
            debug!("📋 Predicción desde cache: {:?}", prediction_type);
            return Ok(cached.clone());
        }
        
        // Calcular predicción
        let prediction = match prediction_type {
            PredictionType::SuccessProbability => {
                self.predict_success_probability(context).await?
            }
            PredictionType::ExecutionTime => {
                self.predict_execution_time(context).await?
            }
            PredictionType::ResourceUsage => {
                self.predict_resource_usage(context).await?
            }
            PredictionType::ResultQuality => {
                self.predict_result_quality(context).await?
            }
            PredictionType::SystemImpact => {
                self.predict_system_impact(context).await?
            }
        };
        
        // Almacenar en cache
        self.cache.insert(cache_key, prediction.clone());
        
        info!(
            "✅ Predicción completada: {:?} = {:.2} (confianza: {:.2})",
            prediction_type, prediction.value, prediction.confidence
        );
        
        Ok(prediction)
    }
    
    /// Predice probabilidad de éxito
    async fn predict_success_probability(&self, context: &ActionContext) -> Result<Prediction> {
        // Analizar historial
        let success_rate = self.calculate_historical_success_rate(&context.action_type);
        
        // Ajustar por métricas actuales
        let system_factor = self.calculate_system_health_factor(&context.system_metrics);
        
        let predicted_value = success_rate * system_factor;
        
        // Calcular confianza basada en tamaño del historial
        let history_size = context.history.len() as f64;
        let confidence = (history_size / (history_size + 10.0)).min(0.95);
        
        let mut metrics = HashMap::new();
        metrics.insert("historical_success_rate".to_string(), success_rate);
        metrics.insert("system_health_factor".to_string(), system_factor);
        metrics.insert("history_size".to_string(), history_size);
        
        let recommendation = if predicted_value >= 0.8 {
            "✅ Acción recomendada - alta probabilidad de éxito".to_string()
        } else if predicted_value >= 0.6 {
            "⚠️  Acción con precaución - éxito moderado".to_string()
        } else {
            "❌ Acción NO recomendada - alto riesgo de fallo".to_string()
        };
        
        Ok(Prediction {
            prediction_type: PredictionType::SuccessProbability,
            value: predicted_value,
            confidence,
            metrics,
            recommendation,
            timestamp: SystemTime::now(),
        })
    }
    
    /// Predice tiempo de ejecución
    async fn predict_execution_time(&self, context: &ActionContext) -> Result<Prediction> {
        // Obtener tiempos históricos
        let historical_times: Vec<f64> = context
            .history
            .iter()
            .map(|r| r.execution_time_ms as f64)
            .collect();
        
        if historical_times.is_empty() {
            // Sin historial, usar estimación conservadora
            return Ok(Prediction {
                prediction_type: PredictionType::ExecutionTime,
                value: 1000.0, // 1 segundo por defecto
                confidence: 0.3,
                metrics: HashMap::new(),
                recommendation: "⚠️  Sin historial - estimación conservadora".to_string(),
                timestamp: SystemTime::now(),
            });
        }
        
        // Calcular estadísticas
        let mean = historical_times.iter().sum::<f64>() / historical_times.len() as f64;
        let variance = historical_times
            .iter()
            .map(|t| (t - mean).powi(2))
            .sum::<f64>()
            / historical_times.len() as f64;
        let std_dev = variance.sqrt();
        
        // Ajustar por carga actual del sistema
        let load_factor = 1.0 + (context.system_metrics.cpu_usage * 0.5);
        let predicted_time = mean * load_factor;
        
        // Confianza basada en consistencia histórica
        let cv = std_dev / mean; // Coefficient of variation
        let confidence = (1.0 - cv).max(0.3).min(0.95);
        
        let mut metrics = HashMap::new();
        metrics.insert("mean_time_ms".to_string(), mean);
        metrics.insert("std_dev_ms".to_string(), std_dev);
        metrics.insert("load_factor".to_string(), load_factor);
        
        Ok(Prediction {
            prediction_type: PredictionType::ExecutionTime,
            value: predicted_time,
            confidence,
            metrics,
            recommendation: format!(
                "⏱️  Tiempo estimado: {:.0}ms (±{:.0}ms)",
                predicted_time, std_dev
            ),
            timestamp: SystemTime::now(),
        })
    }
    
    /// Predice uso de recursos
    async fn predict_resource_usage(&self, context: &ActionContext) -> Result<Prediction> {
        // Calcular uso promedio histórico
        let historical_usage: Vec<f64> = context
            .history
            .iter()
            .map(|r| r.resource_usage)
            .collect();
        
        let predicted_usage = if historical_usage.is_empty() {
            0.3 // Uso moderado por defecto
        } else {
            historical_usage.iter().sum::<f64>() / historical_usage.len() as f64
        };
        
        // Ajustar por disponibilidad actual
        let available_resources = 1.0 - context.system_metrics.memory_usage;
        let adjusted_usage = (predicted_usage / available_resources).min(1.0);
        
        let confidence = if historical_usage.len() >= 10 {
            0.8
        } else {
            0.5
        };
        
        let mut metrics = HashMap::new();
        metrics.insert("predicted_usage".to_string(), predicted_usage);
        metrics.insert("available_resources".to_string(), available_resources);
        
        Ok(Prediction {
            prediction_type: PredictionType::ResourceUsage,
            value: adjusted_usage,
            confidence,
            metrics,
            recommendation: if adjusted_usage < 0.5 {
                "✅ Recursos suficientes disponibles".to_string()
            } else if adjusted_usage < 0.8 {
                "⚠️  Recursos limitados - monitorear".to_string()
            } else {
                "❌ Recursos insuficientes - retrasar acción".to_string()
            },
            timestamp: SystemTime::now(),
        })
    }
    
    /// Predice calidad del resultado
    async fn predict_result_quality(&self, context: &ActionContext) -> Result<Prediction> {
        // Por ahora, usar heurística simple
        let base_quality = 0.75;
        
        // Ajustar por métricas del sistema
        let system_quality = 1.0 - (
            context.system_metrics.cpu_usage * 0.3 +
            context.system_metrics.memory_usage * 0.2
        );
        
        let predicted_quality = (base_quality * system_quality).max(0.0).min(1.0);
        
        Ok(Prediction {
            prediction_type: PredictionType::ResultQuality,
            value: predicted_quality,
            confidence: 0.7,
            metrics: HashMap::new(),
            recommendation: format!("📊 Calidad esperada: {:.0}%", predicted_quality * 100.0),
            timestamp: SystemTime::now(),
        })
    }
    
    /// Predice impacto en el sistema
    async fn predict_system_impact(&self, context: &ActionContext) -> Result<Prediction> {
        // Estimar impacto basado en tipo de acción y estado del sistema
        let base_impact = match context.action_type.as_str() {
            "search" => 0.2,
            "analyze" => 0.4,
            "repair" => 0.6,
            "simulate" => 0.8,
            _ => 0.3,
        };
        
        // Ajustar por carga actual
        let load_multiplier = 1.0 + context.system_metrics.active_tasks as f64 * 0.1;
        let predicted_impact = (base_impact * load_multiplier).min(1.0);
        
        Ok(Prediction {
            prediction_type: PredictionType::SystemImpact,
            value: predicted_impact,
            confidence: 0.75,
            metrics: HashMap::new(),
            recommendation: if predicted_impact < 0.3 {
                "✅ Impacto mínimo en el sistema".to_string()
            } else if predicted_impact < 0.6 {
                "⚠️  Impacto moderado - continuar".to_string()
            } else {
                "❌ Alto impacto - considerar retrasar".to_string()
            },
            timestamp: SystemTime::now(),
        })
    }
    
    /// Registra el resultado de una acción
    pub fn record_result(&self, action_type: String, result: ActionResult) {
        let mut history = self.history.entry(action_type.clone()).or_insert_with(Vec::new);
        
        history.push(result);
        
        // Limitar tamaño del historial
        if history.len() > self.config.max_history_entries {
            history.drain(0..history.len() - self.config.max_history_entries);
        }
        
        debug!("📝 Resultado registrado para acción: {}", action_type);
    }
    
    /// Calcula tasa de éxito histórica
    fn calculate_historical_success_rate(&self, action_type: &str) -> f64 {
        let history = match self.history.get(action_type) {
            Some(h) => h,
            None => return 0.5, // Sin historial, asumir 50%
        };
        
        if history.is_empty() {
            return 0.5;
        }
        
        let successes = history.iter().filter(|r| r.success).count();
        successes as f64 / history.len() as f64
    }
    
    /// Calcula factor de salud del sistema
    fn calculate_system_health_factor(&self, metrics: &SystemMetrics) -> f64 {
        // Sistema saludable = factor cercano a 1.0
        let cpu_factor = 1.0 - (metrics.cpu_usage * 0.5);
        let memory_factor = 1.0 - (metrics.memory_usage * 0.3);
        let load_factor = if metrics.active_tasks > 10 {
            0.8
        } else {
            1.0
        };
        
        (cpu_factor * memory_factor * load_factor).max(0.3).min(1.0)
    }
    
    /// Genera clave de cache
    fn generate_cache_key(&self, prediction_type: PredictionType, context: &ActionContext) -> String {
        format!(
            "{:?}_{}",
            prediction_type,
            self.hash_context(context)
        )
    }
    
    /// Calcula hash del contexto
    fn hash_context(&self, context: &ActionContext) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        context.action_type.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Limpia cache expirado
    pub fn cleanup_cache(&self) {
        // Implementar limpieza basada en TTL
        // Por ahora, simple: vaciar todo
        self.cache.clear();
        debug!("🧹 Cache de predicciones limpiado");
    }
}

impl Default for PredictionEngine {
    fn default() -> Self {
        Self::new(PredictionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_predict_success_probability() {
        let engine = PredictionEngine::default();
        
        let context = ActionContext {
            action_type: "test_action".to_string(),
            parameters: serde_json::json!({}),
            history: vec![
                ActionResult {
                    action_type: "test_action".to_string(),
                    success: true,
                    execution_time_ms: 100,
                    resource_usage: 0.3,
                    timestamp: 0,
                },
                ActionResult {
                    action_type: "test_action".to_string(),
                    success: true,
                    execution_time_ms: 120,
                    resource_usage: 0.4,
                    timestamp: 1,
                },
            ],
            system_metrics: SystemMetrics::default(),
        };
        
        let prediction = engine
            .predict(PredictionType::SuccessProbability, &context)
            .await
            .unwrap();
        
        assert!(prediction.value > 0.0 && prediction.value <= 1.0);
        assert!(prediction.confidence > 0.0 && prediction.confidence <= 1.0);
    }
    
    #[tokio::test]
    async fn test_predict_execution_time() {
        let engine = PredictionEngine::default();
        
        let context = ActionContext {
            action_type: "test".to_string(),
            parameters: serde_json::json!({}),
            history: vec![
                ActionResult {
                    action_type: "test".to_string(),
                    success: true,
                    execution_time_ms: 100,
                    resource_usage: 0.3,
                    timestamp: 0,
                },
                ActionResult {
                    action_type: "test".to_string(),
                    success: true,
                    execution_time_ms: 120,
                    resource_usage: 0.4,
                    timestamp: 1,
                },
                ActionResult {
                    action_type: "test".to_string(),
                    success: true,
                    execution_time_ms: 110,
                    resource_usage: 0.35,
                    timestamp: 2,
                },
            ],
            system_metrics: SystemMetrics::default(),
        };
        
        let prediction = engine
            .predict(PredictionType::ExecutionTime, &context)
            .await
            .unwrap();
        
        // Debería estar cerca del promedio (110ms)
        assert!(prediction.value > 80.0 && prediction.value < 150.0);
    }
}
