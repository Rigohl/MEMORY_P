//! pattern_detector.rs - Detector de Patrones de Usuario
//!
//! Detecta automáticamente patrones de comportamiento del usuario:
//! - Patrones temporales (horarios, frecuencias)
//! - Estilo de código
//! - Uso de herramientas
//! - Workflows típicos

use anyhow::Result;
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Detector de patrones de usuario
pub struct PatternDetector {
    /// Caché de patrones detectados por usuario
    patterns_cache: Arc<RwLock<HashMap<String, UserPatterns>>>,
    
    /// Historial de acciones por usuario
    action_history: Arc<RwLock<HashMap<String, Vec<UserAction>>>>,
}

/// Patrones de usuario detectados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPatterns {
    pub user_id: String,
    pub temporal_patterns: TemporalPatterns,
    pub code_style: CodeStylePatterns,
    pub tool_usage: ToolUsagePatterns,
    pub typical_workflows: Vec<WorkflowPattern>,
    pub confidence: f64,
    pub last_updated: DateTime<Utc>,
}

/// Patrones temporales
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalPatterns {
    pub working_hours: Vec<u8>,           // Horas del día (0-23)
    pub preferred_days: Vec<u8>,          // Días de la semana (1-7)
    pub activity_distribution: Vec<f64>,  // Distribución horaria
    pub session_duration_avg: f64,        // Duración promedio de sesión (minutos)
    pub commits_per_week: f64,
}

impl Default for TemporalPatterns {
    fn default() -> Self {
        Self {
            working_hours: vec![],
            preferred_days: vec![],
            activity_distribution: vec![0.0; 24],
            session_duration_avg: 0.0,
            commits_per_week: 0.0,
        }
    }
}

/// Patrones de estilo de código
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeStylePatterns {
    pub naming_convention: NamingConvention,
    pub documentation_level: DocumentationLevel,
    pub error_handling_style: ErrorHandlingStyle,
    pub testing_coverage_target: f64,
    pub preferred_async_runtime: Option<String>,
}

impl Default for CodeStylePatterns {
    fn default() -> Self {
        Self {
            naming_convention: NamingConvention::SnakeCase,
            documentation_level: DocumentationLevel::Standard,
            error_handling_style: ErrorHandlingStyle::ResultType,
            testing_coverage_target: 0.8,
            preferred_async_runtime: Some("tokio".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NamingConvention {
    SnakeCase,
    CamelCase,
    PascalCase,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentationLevel {
    Minimal,
    Standard,
    Comprehensive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorHandlingStyle {
    ResultType,
    Unwrap,
    Panic,
    Mixed,
}

/// Patrones de uso de herramientas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsagePatterns {
    pub editor_distribution: HashMap<String, f64>,
    pub mcp_tool_frequency: HashMap<String, usize>,
    pub language_distribution: HashMap<String, f64>,
    pub git_habits: GitHabits,
}

impl Default for ToolUsagePatterns {
    fn default() -> Self {
        let mut editor_dist = HashMap::new();
        editor_dist.insert("vscode".to_string(), 0.6);
        editor_dist.insert("cursor".to_string(), 0.4);
        
        let mut lang_dist = HashMap::new();
        lang_dist.insert("rust".to_string(), 0.7);
        lang_dist.insert("julia".to_string(), 0.2);
        lang_dist.insert("python".to_string(), 0.1);
        
        Self {
            editor_distribution: editor_dist,
            mcp_tool_frequency: HashMap::new(),
            language_distribution: lang_dist,
            git_habits: GitHabits::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHabits {
    pub commit_frequency: String,
    pub commit_message_style: String,
    pub branching_strategy: Option<String>,
}

impl Default for GitHabits {
    fn default() -> Self {
        Self {
            commit_frequency: "frequent".to_string(),
            commit_message_style: "descriptive".to_string(),
            branching_strategy: Some("feature-branches".to_string()),
        }
    }
}

/// Patrón de workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPattern {
    pub sequence: Vec<String>,
    pub frequency: usize,
    pub avg_duration_secs: f64,
    pub success_rate: f64,
}

/// Acción de usuario para análisis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAction {
    pub timestamp: DateTime<Utc>,
    pub action_type: String,
    pub tool: String,
    pub language: Option<String>,
    pub success: bool,
    pub duration_secs: f64,
}

impl PatternDetector {
    /// Crea un nuevo detector de patrones
    pub fn new() -> Self {
        info!("🔍 Inicializando detector de patrones");
        
        Self {
            patterns_cache: Arc::new(RwLock::new(HashMap::new())),
            action_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Registra una acción de usuario
    pub async fn record_action(&self, user_id: &str, action: UserAction) {
        let mut history = self.action_history.write().await;
        let user_history = history.entry(user_id.to_string()).or_insert_with(Vec::new);
        
        user_history.push(action);
        
        // Mantener solo últimas 10000 acciones por usuario
        if user_history.len() > 10000 {
            user_history.remove(0);
        }
    }
    
    /// Detecta patrones de usuario
    pub async fn detect_patterns(&self, user_id: &str) -> Result<UserPatterns> {
        debug!("🔍 Detectando patrones para usuario: {}", user_id);
        
        let history = self.action_history.read().await;
        let user_actions = history.get(user_id)
            .cloned()
            .unwrap_or_default();
        
        if user_actions.is_empty() {
            // Retornar patrones por defecto si no hay historial
            return Ok(UserPatterns {
                user_id: user_id.to_string(),
                temporal_patterns: TemporalPatterns::default(),
                code_style: CodeStylePatterns::default(),
                tool_usage: ToolUsagePatterns::default(),
                typical_workflows: vec![],
                confidence: 0.0,
                last_updated: Utc::now(),
            });
        }
        
        // Detectar patrones temporales
        let temporal = self.detect_temporal_patterns(&user_actions);
        
        // Detectar estilo de código (análisis básico)
        let code_style = self.detect_code_style(&user_actions);
        
        // Detectar uso de herramientas
        let tool_usage = self.detect_tool_usage(&user_actions);
        
        // Detectar workflows
        let workflows = self.detect_workflows(&user_actions);
        
        // Calcular confianza basada en cantidad de datos
        let confidence = (user_actions.len() as f64 / 1000.0).min(1.0);
        
        let patterns = UserPatterns {
            user_id: user_id.to_string(),
            temporal_patterns: temporal,
            code_style,
            tool_usage,
            typical_workflows: workflows,
            confidence,
            last_updated: Utc::now(),
        };
        
        // Cachear patrones
        let mut cache = self.patterns_cache.write().await;
        cache.insert(user_id.to_string(), patterns.clone());
        
        info!("✅ Patrones detectados para {} (confidence: {:.2})", user_id, confidence);
        
        Ok(patterns)
    }
    
    /// Detecta patrones temporales
    fn detect_temporal_patterns(&self, actions: &[UserAction]) -> TemporalPatterns {
        let mut hour_counts = vec![0usize; 24];
        let mut day_counts = vec![0usize; 7];
        let mut total_duration = 0.0;
        
        for action in actions {
            let hour = action.timestamp.hour() as usize;
            hour_counts[hour] += 1;
            
            let day = action.timestamp.weekday().num_days_from_monday() as usize;
            day_counts[day] += 1;
            
            total_duration += action.duration_secs;
        }
        
        // Calcular distribución de actividad
        let total_actions = actions.len() as f64;
        let activity_distribution: Vec<f64> = hour_counts.iter()
            .map(|&count| count as f64 / total_actions)
            .collect();
        
        // Identificar horas de trabajo (>5% de actividad)
        let working_hours: Vec<u8> = hour_counts.iter()
            .enumerate()
            .filter(|(_, &count)| count as f64 / total_actions > 0.05)
            .map(|(hour, _)| hour as u8)
            .collect();
        
        // Identificar días preferidos (>10% de actividad)
        let preferred_days: Vec<u8> = day_counts.iter()
            .enumerate()
            .filter(|(_, &count)| count as f64 / total_actions > 0.1)
            .map(|(day, _)| (day + 1) as u8) // 1-7 instead of 0-6
            .collect();
        
        let avg_session_duration = if !actions.is_empty() {
            (total_duration / actions.len() as f64) / 60.0 // En minutos
        } else {
            0.0
        };
        
        TemporalPatterns {
            working_hours,
            preferred_days,
            activity_distribution,
            session_duration_avg: avg_session_duration,
            commits_per_week: (actions.len() as f64 / 7.0), // Aproximación
        }
    }
    
    /// Detecta estilo de código (análisis básico)
    fn detect_code_style(&self, _actions: &[UserAction]) -> CodeStylePatterns {
        // Por ahora retornar valores por defecto
        // En producción, analizaríamos el código real
        CodeStylePatterns::default()
    }
    
    /// Detecta patrones de uso de herramientas
    fn detect_tool_usage(&self, actions: &[UserAction]) -> ToolUsagePatterns {
        let mut tool_counts: HashMap<String, usize> = HashMap::new();
        let mut lang_counts: HashMap<String, usize> = HashMap::new();
        
        for action in actions {
            *tool_counts.entry(action.tool.clone()).or_insert(0) += 1;
            
            if let Some(ref lang) = action.language {
                *lang_counts.entry(lang.clone()).or_insert(0) += 1;
            }
        }
        
        // Convertir a distribuciones
        let total_actions = actions.len() as f64;
        
        let editor_distribution: HashMap<String, f64> = tool_counts.iter()
            .map(|(tool, &count)| (tool.clone(), count as f64 / total_actions))
            .collect();
        
        let language_distribution: HashMap<String, f64> = lang_counts.iter()
            .map(|(lang, &count)| (lang.clone(), count as f64 / total_actions))
            .collect();
        
        ToolUsagePatterns {
            editor_distribution,
            mcp_tool_frequency: tool_counts,
            language_distribution,
            git_habits: GitHabits::default(),
        }
    }
    
    /// Detecta workflows típicos
    fn detect_workflows(&self, actions: &[UserAction]) -> Vec<WorkflowPattern> {
        let mut workflows = Vec::new();
        
        // Detección simple de secuencias comunes
        // En producción usaríamos algoritmos de sequence mining más sofisticados
        
        if actions.len() < 3 {
            return workflows;
        }
        
        // Buscar secuencias frecuentes de 3 acciones
        let mut sequences: HashMap<Vec<String>, usize> = HashMap::new();
        
        for window in actions.windows(3) {
            let sequence: Vec<String> = window.iter()
                .map(|a| a.action_type.clone())
                .collect();
            *sequences.entry(sequence).or_insert(0) += 1;
        }
        
        // Convertir a workflows (solo los que aparecen >5 veces)
        for (sequence, frequency) in sequences {
            if frequency >= 5 {
                workflows.push(WorkflowPattern {
                    sequence,
                    frequency,
                    avg_duration_secs: 120.0, // Estimación
                    success_rate: 0.85, // Estimación
                });
            }
        }
        
        workflows
    }
    
    /// Obtiene patrones cacheados
    pub async fn get_cached_patterns(&self, user_id: &str) -> Option<UserPatterns> {
        let cache = self.patterns_cache.read().await;
        cache.get(user_id).cloned()
    }
    
    /// Genera reporte de patrones
    pub async fn generate_pattern_report(&self, user_id: &str) -> String {
        let patterns = match self.detect_patterns(user_id).await {
            Ok(p) => p,
            Err(_) => return "❌ Error generando reporte de patrones".to_string(),
        };
        
        format!(
            r#"
👤 PATRONES DE USUARIO: {}

⏰ Temporal:
├─ Horario de Trabajo: {:?}
├─ Días Preferidos: {:?}
├─ Duración de Sesión: {:.1} min
└─ Commits/Semana: {:.1}

💻 Estilo de Código:
├─ Naming: {:?}
├─ Documentación: {:?}
├─ Error Handling: {:?}
└─ Testing Coverage: {:.0}%

🛠️ Uso de Herramientas:
├─ Editores: {:#?}
├─ Lenguajes: {:#?}
└─ Git: {}

🔄 Workflows Típicos: {} detectados

📊 Confidence: {:.1}%
"#,
            user_id,
            patterns.temporal_patterns.working_hours,
            patterns.temporal_patterns.preferred_days,
            patterns.temporal_patterns.session_duration_avg,
            patterns.temporal_patterns.commits_per_week,
            patterns.code_style.naming_convention,
            patterns.code_style.documentation_level,
            patterns.code_style.error_handling_style,
            patterns.code_style.testing_coverage_target * 100.0,
            patterns.tool_usage.editor_distribution,
            patterns.tool_usage.language_distribution,
            patterns.tool_usage.git_habits.commit_frequency,
            patterns.typical_workflows.len(),
            patterns.confidence * 100.0,
        )
    }
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_pattern_detection() {
        let detector = PatternDetector::new();
        
        // Record some actions
        for i in 0..10 {
            detector.record_action("test_user", UserAction {
                timestamp: Utc::now(),
                action_type: "edit".to_string(),
                tool: "vscode".to_string(),
                language: Some("rust".to_string()),
                success: true,
                duration_secs: 60.0,
            }).await;
        }
        
        let patterns = detector.detect_patterns("test_user").await.unwrap();
        assert_eq!(patterns.user_id, "test_user");
        assert!(patterns.confidence > 0.0);
    }
    
    #[tokio::test]
    async fn test_pattern_caching() {
        let detector = PatternDetector::new();
        
        detector.record_action("user1", UserAction {
            timestamp: Utc::now(),
            action_type: "test".to_string(),
            tool: "test".to_string(),
            language: None,
            success: true,
            duration_secs: 1.0,
        }).await;
        
        let _ = detector.detect_patterns("user1").await;
        let cached = detector.get_cached_patterns("user1").await;
        
        assert!(cached.is_some());
    }
}
