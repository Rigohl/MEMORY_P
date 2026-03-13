//! predictive_nodes.rs - Nodos predictivos para auto-corrección
//! Auto-corrección de búsquedas fallidas

use crate::error::{MemoryPError, Result};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Predicción de búsqueda
#[derive(Debug, Clone)]
struct SearchPrediction {
    #[allow(dead_code)]
    original_query: String,
    corrected_query: String,
    confidence: f64,
}

/// Sistema de nodos predictivos
pub struct PredictiveNodes {
    predictions: Arc<DashMap<String, SearchPrediction>>,
    success_rate: Arc<RwLock<f64>>,
    running: Arc<RwLock<bool>>,
}

impl Default for PredictiveNodes {
    fn default() -> Self {
        Self::new()
    }
}

impl PredictiveNodes {
    pub fn new() -> Self {
        Self {
            predictions: Arc::new(DashMap::new()),
            success_rate: Arc::new(RwLock::new(0.0)),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("🧠 Iniciando Predictive Nodes...");

        let mut running = self.running.write().await;
        if *running {
            warn!("Predictive Nodes ya está ejecutándose");
            return Ok(());
        }
        *running = true;
        drop(running);

        // Iniciar aprendizaje continuo
        self.start_learning_task().await;

        info!("✅ Predictive Nodes iniciado");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Deteniendo Predictive Nodes...");

        let mut running = self.running.write().await;
        *running = false;

        info!("✅ Predictive Nodes detenido");
        Ok(())
    }

    /// Tarea de aprendizaje continuo
    async fn start_learning_task(&self) {
        let predictions = self.predictions.clone();
        let success_rate = self.success_rate.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            loop {
                if !*running.read().await {
                    break;
                }

                // Analizar predicciones exitosas
                let total = predictions.len();
                if total > 0 {
                    let successful = predictions
                        .iter()
                        .filter(|e| e.value().confidence > 0.7)
                        .count();

                    let rate = successful as f64 / total as f64;
                    *success_rate.write().await = rate;

                    info!("📊 Tasa de éxito predictiva: {:.2}%", rate * 100.0);
                }

                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });
    }

    /// Predice y busca con auto-corrección
    pub async fn predict_and_search(&self, query: &str) -> Result<Vec<String>> {
        // Verificar si hay predicción previa
        if let Some(prediction) = self.predictions.get(query) {
            info!(
                "🔮 Usando predicción para: {} -> {}",
                query, prediction.corrected_query
            );
            return self.execute_search(&prediction.corrected_query).await;
        }

        // Búsqueda directa
        self.execute_search(query).await
    }

    /// Auto-corrección y reintento
    pub async fn auto_correct_and_retry(&self, query: &str) -> Result<Vec<String>> {
        info!("🔧 Auto-corrección activada para: {}", query);

        // Generar correcciones predictivas
        let corrections = self.generate_corrections(query);

        for corrected in corrections {
            info!("🔄 Intentando con: {}", corrected);

            match self.execute_search(&corrected).await {
                Ok(results) if !results.is_empty() => {
                    // Guardar predicción exitosa
                    self.predictions.insert(
                        query.to_string(),
                        SearchPrediction {
                            original_query: query.to_string(),
                            corrected_query: corrected.clone(),
                            confidence: 0.8,
                        },
                    );

                    info!("✅ Auto-corrección exitosa: {} -> {}", query, corrected);
                    return Ok(results);
                }
                _ => continue,
            }
        }

        Err(MemoryPError::Other(format!(
            "No se pudo auto-corregir búsqueda: {}",
            query
        )))
    }

    /// Genera correcciones predictivas
    fn generate_corrections(&self, query: &str) -> Vec<String> {
        let mut corrections = vec![query.to_string()];

        // Corrección 1: Lowercase
        corrections.push(query.to_lowercase());

        // Corrección 2: Sin espacios extra
        corrections.push(query.trim().replace("  ", " "));

        // Corrección 3: Sin caracteres especiales
        corrections.push(
            query
                .chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                .collect(),
        );

        corrections
    }

    /// Ejecuta búsqueda (simulada)
    async fn execute_search(&self, query: &str) -> Result<Vec<String>> {
        info!("🔍 Ejecutando búsqueda: {}", query);

        let indices = vec![
            "tantivy".to_string(),
            "memory_bank".to_string(),
            "julia_nlp".to_string(),
        ];

        let results = crate::ffi::pony::distributed_search(query, &indices)
            .await
            .map_err(|e| MemoryPError::Other(format!("Pony predictive search failed: {}", e)))?;

        if results.is_empty() {
            Err(MemoryPError::Other(format!(
                "No se encontraron resultados distribuidos para '{}'",
                query
            )))
        } else {
            Ok(results)
        }
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let total_predictions = self.predictions.len();
        let success_rate = *self.success_rate.blocking_read();

        serde_json::json!({
            "total_predictions": total_predictions,
            "success_rate": format!("{:.2}%", success_rate * 100.0),
        })
    }
}
