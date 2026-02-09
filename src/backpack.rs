//! src/backpack.rs - La "Mochila" del Agente: Contexto denso y herramientas listas.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::shared_memory::types::MemoryStats;
use crate::prediction_engine::NextAgentMoves;

/// La Mochila es el objeto de contexto definitivo que acompaña al agente.
/// Contiene todo lo necesario para que el agente no tenga que buscar contexto.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backpack {
    /// ID de la sesión actual
    pub session_id: String,

    /// Predicciones matemáticas del siguiente paso (2 pasos adelante)
    pub predictions: NextAgentMoves,

    /// Herramientas recomendadas basadas en el estado actual
    pub recommended_tools: Vec<String>,

    /// Resumen del estado de salud del proyecto
    pub project_health: ProjectHealth,

    /// Memoria de corto plazo (últimas acciones y resultados)
    pub short_term_memory: Vec<String>,

    /// Estadísticas de agilidad de memoria
    pub memory_agility: MemoryStats,

    /// Alertas proactivas del sistema autónomo
    pub proactive_alerts: Vec<String>,

    /// Fragmentos de código o documentación de alta relevancia
    pub high_utility_snippets: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectHealth {
    pub compilation_status: String,
    pub test_coverage: f64,
    pub critical_issues: usize,
    pub technical_debt_score: u8,
}

impl Backpack {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            predictions: NextAgentMoves {
                next_step: "Analizar requerimientos".to_string(),
                following_step: "Implementar solución".to_string(),
                confidence: 0.9,
                rationale: "Inicialización de mochila".to_string(),
            },
            recommended_tools: vec!["read_file".to_string(), "list_files".to_string()],
            project_health: ProjectHealth {
                compilation_status: "Ok".to_string(),
                test_coverage: 0.0,
                critical_issues: 0,
                technical_debt_score: 10,
            },
            short_term_memory: Vec::new(),
            memory_agility: MemoryStats::new(),
            proactive_alerts: Vec::new(),
            high_utility_snippets: HashMap::new(),
        }
    }

    /// Añade una alerta proactiva a la mochila
    pub fn add_alert(&mut self, alert: String) {
        if !self.proactive_alerts.contains(&alert) {
            self.proactive_alerts.push(alert);
        }
    }

    /// Actualiza el estado de salud basado en inspección rápida
    pub fn update_health(&mut self, status: String, issues: usize) {
        self.project_health.compilation_status = status;
        self.project_health.critical_issues = issues;
    }
}
