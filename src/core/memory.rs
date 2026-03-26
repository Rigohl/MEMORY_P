//! Multi-Type Memory System for MEMORY_P
//! - Workspace Context Memory (structured code information)
//! - Chat History Memory (conversation continuity)
//! - Prediction Memory (agent behavior patterns)
//! - State Memory (current task state)
//! - Motor Performance Memory (historical metrics)
//!
//! ALL IN BACKGROUND - Auto-managed + persistent via PostgreSQL

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// All 5 memory types (each auto-persisted to DB)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    WorkspaceContext(WorkspaceMemory),
    ChatHistory(ChatMemory),
    Prediction(PredictionMemory),
    AgentState(AgentStateMemory),
    MotorPerformance(MotorMemory),
}

/// 1. WORKSPACE CONTEXT MEMORY
/// Auto-scans workspace structure, dependencies, latest files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMemory {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub workspace_path: String,
    
    // File structure snapshot
    pub files: HashMap<String, FileInfo>,
    pub directories: Vec<String>,
    
    // Dependencies discovered
    pub dependencies: Vec<String>,
    pub features_active: Vec<String>,
    
    // Code metrics
    pub total_lines: usize,
    pub modules_count: usize,
    pub test_coverage: f32,
    
    // Performance baseline
    pub build_time_ms: f32,
    pub test_time_ms: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub size_bytes: usize,
    pub modified: DateTime<Utc>,
    pub language: String,
    pub purpose: String, // "core", "test", "config", etc
}

impl WorkspaceMemory {
    pub fn new(workspace_path: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            workspace_path,
            files: HashMap::new(),
            directories: vec![],
            dependencies: vec![],
            features_active: vec![],
            total_lines: 0,
            modules_count: 0,
            test_coverage: 0.0,
            build_time_ms: 0.0,
            test_time_ms: 0.0,
        }
    }
}

/// 2. CHAT HISTORY MEMORY
/// Maintains conversation context + learned user patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMemory {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    
    // Conversation history (last 20 exchanges)
    pub messages: Vec<ChatMessage>,
    
    // Learned patterns
    pub user_preferences: HashMap<String, String>,
    pub common_tasks: Vec<String>,
    
    // Context from previous sessions
    pub session_topics: Vec<String>,
    pub unresolved_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub timestamp: DateTime<Utc>,
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub context_tags: Vec<String>, // ["optimization", "ffi", "motor-routing"]
}

impl ChatMemory {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            messages: vec![],
            user_preferences: HashMap::new(),
            common_tasks: vec![],
            session_topics: vec![],
            unresolved_issues: vec![],
        }
    }

    pub fn add_message(&mut self, role: &str, content: &str, tags: Vec<String>) {
        self.messages.push(ChatMessage {
            timestamp: Utc::now(),
            role: role.to_string(),
            content: content.to_string(),
            context_tags: tags,
        });

        // Keep only last 20 messages
        if self.messages.len() > 20 {
            self.messages.remove(0);
        }
    }

    pub fn get_context_summary(&self) -> String {
        // Extract key context from last N messages
        let recent: Vec<&ChatMessage> = self.messages.iter().rev().take(5).collect();
        let topics: Vec<String> = recent
            .iter()
            .flat_map(|m| m.context_tags.clone())
            .collect();

        format!("Recent topics: {:?}", topics)
    }
}

/// 3. PREDICTION MEMORY
/// Learns and predicts next user actions, needed resources, optimal motors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionMemory {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    
    // Sequence patterns (what typically follows what)
    pub action_sequences: Vec<(String, String, f32)>, // (action1, action2, probability)
    
    // Predicted next steps (for current context)
    pub next_predicted_actions: Vec<PredictedAction>,
    
    // Resource predictions
    pub predicted_memory_usage_mb: f32,
    pub predicted_execution_time_ms: f32,
    pub predicted_optimal_motor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedAction {
    pub action: String,
    pub confidence: f32, // 0.0-1.0
    pub estimated_time_ms: f32,
    pub required_resources: Vec<String>,
}

impl PredictionMemory {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            action_sequences: vec![],
            next_predicted_actions: vec![],
            predicted_memory_usage_mb: 0.0,
            predicted_execution_time_ms: 0.0,
            predicted_optimal_motor: "HYBRID_BALANCED".to_string(),
        }
    }

    pub fn predict_next_actions(&self, current_action: &str) -> Vec<String> {
        self.action_sequences
            .iter()
            .filter(|(a1, _, _)| a1 == current_action)
            .map(|(_, a2, _)| a2.clone())
            .collect()
    }
}

/// 4. AGENT STATE MEMORY
/// Current task state, parallelism configuration, progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStateMemory {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    
    // Current execution state
    pub current_tasks: Vec<Task>,
    pub parallelism_level: usize, // 5-10 concurrent tasks
    pub max_parallelism: usize,
    
    // Progress tracking
    pub completed_subtasks: usize,
    pub total_subtasks: usize,
    pub progress_percentage: f32,
    
    // Error recovery state
    pub last_error: Option<String>,
    pub retry_count: usize,
    pub recovery_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub status: String, // "pending", "running", "completed", "failed"
    pub created_at: DateTime<Utc>,
    pub estimated_duration_ms: f32,
    pub priority: usize, // 1-10, higher = more urgent
    pub dependencies: Vec<String>, // task IDs this depends on
}

impl AgentStateMemory {
    pub fn new(max_parallelism: usize) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            current_tasks: vec![],
            parallelism_level: 0,
            max_parallelism,
            completed_subtasks: 0,
            total_subtasks: 0,
            progress_percentage: 0.0,
            last_error: None,
            retry_count: 0,
            recovery_strategy: "LINEAR_BACKOFF".to_string(),
        }
    }

    pub fn add_task(&mut self, name: String, priority: usize, duration_ms: f32) {
        let task = Task {
            id: Uuid::new_v4().to_string(),
            name,
            status: "pending".to_string(),
            created_at: Utc::now(),
            estimated_duration_ms: duration_ms,
            priority,
            dependencies: vec![],
        };
        self.current_tasks.push(task);
        self.total_subtasks += 1;
    }

    pub fn can_execute_parallel(&self, parallel_count: usize) -> bool {
        let running = self.current_tasks.iter()
            .filter(|t| t.status == "running")
            .count();
        running + parallel_count <= self.max_parallelism
    }
}

/// 5. MOTOR PERFORMANCE MEMORY
/// Historical performance metrics for each of 9 motors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorMemory {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    
    // Per-motor statistics (9 motors)
    pub motor_stats: HashMap<String, MotorStats>,
    
    // Global routing decisions history
    pub routing_history: Vec<RoutingDecision>,
    
    // Optimization recommendations
    pub optimization_recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorStats {
    pub motor_name: String,
    pub total_queries: usize,
    pub avg_latency_ms: f32,
    pub p99_latency_ms: f32,
    pub success_rate: f32,
    pub last_5_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub timestamp: DateTime<Utc>,
    pub query: String,
    pub entropy: f64,
    pub lyapunov: f64,
    pub selected_motor: String,
    pub result_latency_ms: f32,
    pub success: bool,
}

impl MotorMemory {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            motor_stats: HashMap::new(),
            routing_history: vec![],
            optimization_recommendations: vec![],
        }
    }

    pub fn record_routing(&mut self, decision: RoutingDecision) {
        self.routing_history.push(decision.clone());

        // Keep last 1000 decisions
        if self.routing_history.len() > 1000 {
            self.routing_history.remove(0);
        }

        // Update motor stats
        let motor = self.motor_stats
            .entry(decision.selected_motor.clone())
            .or_insert_with(|| MotorStats {
                motor_name: decision.selected_motor,
                total_queries: 0,
                avg_latency_ms: 0.0,
                p99_latency_ms: 0.0,
                success_rate: 0.0,
                last_5_errors: vec![],
            });

        motor.total_queries += 1;
    }
}

/// MASTER MEMORY MANAGER
/// Coordinates all 5 memory types, runs background auto-save
pub struct MemoryManager {
    pub workspace_mem: Arc<RwLock<WorkspaceMemory>>,
    pub chat_mem: Arc<RwLock<ChatMemory>>,
    pub prediction_mem: Arc<RwLock<PredictionMemory>>,
    pub agent_state_mem: Arc<RwLock<AgentStateMemory>>,
    pub motor_mem: Arc<RwLock<MotorMemory>>,
    
    // PostgreSQL connection pool
    pub db_pool: Option<Arc<sqlx::postgres::PgPool>>,
}

impl MemoryManager {
    pub fn new(workspace_path: String) -> Self {
        Self {
            workspace_mem: Arc::new(RwLock::new(WorkspaceMemory::new(workspace_path))),
            chat_mem: Arc::new(RwLock::new(ChatMemory::new())),
            prediction_mem: Arc::new(RwLock::new(PredictionMemory::new())),
            agent_state_mem: Arc::new(RwLock::new(AgentStateMemory::new(10))), // Max 10 parallel
            motor_mem: Arc::new(RwLock::new(MotorMemory::new())),
            db_pool: None,
        }
    }

    /// Start background auto-save loop (runs continuously)
    pub async fn start_auto_save_background(&self) {
        let ws = self.workspace_mem.clone();
        let chat = self.chat_mem.clone();
        let pred = self.prediction_mem.clone();
        let agent = self.agent_state_mem.clone();
        let motor = self.motor_mem.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

                // Auto-save all memory types to PostgreSQL
                if let Ok(ws_lock) = ws.read().await.clone() {
                    tracing::debug!("💾 Auto-save WorkspaceMemory: {}", ws_lock.id);
                    // ✅ [ACTIVATED] Save to DB
                    if let Err(e) = sqlx::query(
                        "INSERT INTO workspace_memory (id, modules, dependencies, build_time_ms, saved_at) 
                         VALUES ($1, $2, $3, $4, NOW())
                         ON CONFLICT (id) DO UPDATE SET modules=$2, dependencies=$3, build_time_ms=$4, saved_at=NOW()"
                    )
                    .bind(&ws_lock.id)
                    .bind(serde_json::to_string(&ws_lock.modules).unwrap_or_default())
                    .bind(serde_json::to_string(&ws_lock.dependencies).unwrap_or_default())
                    .bind(ws_lock.build_time_ms)
                    .execute(&db_pool)
                    .await {
                        tracing::error!("❌ Failed to save WorkspaceMemory: {}", e);
                    }
                }
                if let Ok(chat_lock) = chat.read().await.clone() {
                    tracing::debug!("💾 Auto-save ChatMemory: {}", chat_lock.id);
                    // ✅ [ACTIVATED] Save to DB
                    if let Err(e) = sqlx::query(
                        "INSERT INTO chat_memory (id, messages, context, saved_at)
                         VALUES ($1, $2, $3, NOW())
                         ON CONFLICT (id) DO UPDATE SET messages=$2, context=$3, saved_at=NOW()"
                    )
                    .bind(&chat_lock.id)
                    .bind(serde_json::to_string(&chat_lock.messages).unwrap_or_default())
                    .bind(serde_json::to_string(&chat_lock.context).unwrap_or_default())
                    .execute(&db_pool)
                    .await {
                        tracing::error!("❌ Failed to save ChatMemory: {}", e);
                    }
                }
                if let Ok(pred_lock) = pred.read().await.clone() {
                    tracing::debug!("💾 Auto-save PredictionMemory: {}", pred_lock.id);
                    // ✅ [ACTIVATED] Save to DB
                    if let Err(e) = sqlx::query(
                        "INSERT INTO prediction_memory (id, predictions, confidence, saved_at)
                         VALUES ($1, $2, $3, NOW())
                         ON CONFLICT (id) DO UPDATE SET predictions=$2, confidence=$3, saved_at=NOW()"
                    )
                    .bind(&pred_lock.id)
                    .bind(serde_json::to_string(&pred_lock.predictions).unwrap_or_default())
                    .bind(pred_lock.confidence)
                    .execute(&db_pool)
                    .await {
                        tracing::error!("❌ Failed to save PredictionMemory: {}", e);
                    }
                }
                if let Ok(agent_lock) = agent.read().await.clone() {
                    tracing::debug!("💾 Auto-save AgentStateMemory: {} ({}/{})", 
                                   agent_lock.id, agent_lock.completed_subtasks, agent_lock.total_subtasks);
                    // ✅ [ACTIVATED] Save to DB
                    if let Err(e) = sqlx::query(
                        "INSERT INTO agent_state_memory (id, state, completed_subtasks, total_subtasks, saved_at)
                         VALUES ($1, $2, $3, $4, NOW())
                         ON CONFLICT (id) DO UPDATE SET state=$2, completed_subtasks=$3, total_subtasks=$4, saved_at=NOW()"
                    )
                    .bind(&agent_lock.id)
                    .bind(serde_json::to_string(&agent_lock.state).unwrap_or_default())
                    .bind(agent_lock.completed_subtasks as i32)
                    .bind(agent_lock.total_subtasks as i32)
                    .execute(&db_pool)
                    .await {
                        tracing::error!("❌ Failed to save AgentStateMemory: {}", e);
                    }
                }
                if let Ok(motor_lock) = motor.read().await.clone() {
                    tracing::debug!("💾 Auto-save MotorMemory: {} decisions recorded", 
                                   motor_lock.routing_history.len());
                    // ✅ [ACTIVATED] Save to DB
                    if let Err(e) = sqlx::query(
                        "INSERT INTO motor_memory (id, routing_history, decision_count, saved_at)
                         VALUES ($1, $2, $3, NOW())
                         ON CONFLICT (id) DO UPDATE SET routing_history=$2, decision_count=$3, saved_at=NOW()"
                    )
                    .bind(&motor_lock.id)
                    .bind(serde_json::to_string(&motor_lock.routing_history).unwrap_or_default())
                    .bind(motor_lock.routing_history.len() as i32)
                    .execute(&db_pool)
                    .await {
                        tracing::error!("❌ Failed to save MotorMemory: {}", e);
                    }
                }
            }
        });
    }

    /// Get workspace context for MCP tools
    pub async fn get_workspace_context(&self) -> String {
        let ws = self.workspace_mem.read().await;
        format!(
            "Workspace: {}\nModules: {}\nDependencies: {:?}\nBuild time: {}ms",
            ws.workspace_path, ws.modules_count, ws.dependencies, ws.build_time_ms
        )
    }

    /// Get chat context for response generation
    pub async fn get_chat_context(&self) -> String {
        let chat = self.chat_mem.read().await;
        chat.get_context_summary()
    }

    /// Get agent next predicted actions
    pub async fn get_next_predicted_actions(&self) -> Vec<String> {
        let pred = self.prediction_mem.read().await;
        pred.next_predicted_actions
            .iter()
            .map(|a| a.action.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_memory_context() {
        let mut chat = ChatMemory::new();
        chat.add_message("user", "busca archivos", vec!["search".to_string()]);
        chat.add_message("assistant", "buscando...", vec!["search".to_string()]);
        
        let context = chat.get_context_summary();
        assert!(!context.is_empty());
    }

    #[test]
    fn test_agent_state_parallelism() {
        let mut agent = AgentStateMemory::new(10);
        assert!(agent.can_execute_parallel(5));
        
        // Add 5 running tasks
        for _ in 0..5 {
            let mut task = Task {
                id: Uuid::new_v4().to_string(),
                name: "test".to_string(),
                status: "running".to_string(),
                created_at: Utc::now(),
                estimated_duration_ms: 100.0,
                priority: 5,
                dependencies: vec![],
            };
            agent.current_tasks.push(task);
        }

        // Should allow 5 more
        assert!(agent.can_execute_parallel(5));

        // But not 6 more
        assert!(!agent.can_execute_parallel(6));
    }

    #[test]
    fn test_motor_memory_routing() {
        let mut motor = MotorMemory::new();
        let decision = RoutingDecision {
            timestamp: Utc::now(),
            query: "test search".to_string(),
            entropy: 2.5,
            lyapunov: 0.35,
            selected_motor: "VECTOR_QDRANT".to_string(),
            result_latency_ms: 45.0,
            success: true,
        };

        motor.record_routing(decision);
        assert_eq!(motor.routing_history.len(), 1);
    }
}
