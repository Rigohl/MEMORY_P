//! Agent Orchestrator - Coordina múltiples agentes paralelos
//! 
//! Los 9 motores actúan como agentes distribuidos que ejecutan tareas simultáneamente.
//! Comunicación vía canales Tokio + paralelización con Rayon para CPU-heavy work.
//!
//! Arquitectura:
//! - TaskQueue: Distribuye tareas a agentes disponibles
//! - ParallelExecutor: Ejecuta tareas con Rayon (CPU-bound)
//! - AsyncCoordinator: Coordina con Tokio (I/O-bound)
//! - ResultAggregator: Recolecta resultados con contexto

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Semaphore};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::Instant;

// ============================================================================
// TIPOS DE TAREAS Y RESULTADOS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    /// Búsqueda en motor específico
    MotorSearch { engine: String, query: String },
    
    /// Análisis matemático (Julia)
    MathAnalysis { data: Vec<f64>, operation: String },
    
    /// Predicción contextual
    PredictiveAnalysis { context_id: Uuid, lookahead: usize },
    
    /// Procesamiento paralelo de batch
    BatchProcess { documents: Vec<String>, operation: String },
    
    /// Análisis de caos (Lyapunov, entropía)
    ChaosAnalysis { metrics: Vec<f64> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub task_type: TaskType,
    pub priority: u8,  // 0-255, mayor = más prioritario
    pub created_at: i64,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub result: serde_json::Value,
    pub execution_time_ms: u64,
    pub agent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Timeout,
}

// ============================================================================
// AGENT POOL - Mantiene pool de agentes para ejecución paralela
// ============================================================================

pub struct Agent {
    pub id: String,
    pub motor_name: String,
    pub max_parallel_tasks: usize,
    pub current_tasks: Arc<RwLock<usize>>,
}

impl Agent {
    pub fn new(id: String, motor_name: String, max_parallel_tasks: usize) -> Self {
        Self {
            id,
            motor_name,
            max_parallel_tasks,
            current_tasks: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn can_accept_task(&self) -> bool {
        let current = *self.current_tasks.read().await;
        current < self.max_parallel_tasks
    }

    pub async fn increment_tasks(&self) {
        let mut current = self.current_tasks.write().await;
        *current += 1;
    }

    pub async fn decrement_tasks(&self) {
        let mut current = self.current_tasks.write().await;
        *current = current.saturating_sub(1);
    }
}

// ============================================================================
// AGENT POOL MANAGER
// ============================================================================

pub struct AgentPool {
    agents: Vec<Arc<Agent>>,
    task_queue: mpsc::UnboundedSender<Task>,
    result_tx: mpsc::UnboundedSender<TaskResult>,
}

impl AgentPool {
    /// Crea un nuevo pool de agentes basado en los 9 motores
    pub fn new(result_tx: mpsc::UnboundedSender<TaskResult>) -> (Self, mpsc::UnboundedReceiver<Task>) {
        let (task_tx, task_rx) = mpsc::unbounded_channel();
        
        let motors = vec![
            "qdrant", "faiss", "scann", "tantivy", "lnx", 
            "meilisearch", "julia_nlp", "memorybank", "toshi"
        ];

        let agents = motors
            .iter()
            .enumerate()
            .map(|(i, motor)| {
                Arc::new(Agent::new(
                    format!("agent-{}", i),
                    motor.to_string(),
                    4,  // Cada motor puede ejecutar 4 tareas paralelas
                ))
            })
            .collect();

        (
            Self {
                agents,
                task_queue: task_tx,
                result_tx,
            },
            task_rx,
        )
    }

    /// Envía una tarea al pool
    pub async fn submit_task(&self, task: Task) -> Result<Uuid, String> {
        let task_id = task.id;
        self.task_queue
            .send(task)
            .map_err(|e| e.to_string())?;
        Ok(task_id)
    }

    /// Obtiene un agente disponible (load balancing)
    pub async fn get_available_agent(&self) -> Option<Arc<Agent>> {
        for agent in &self.agents {
            if agent.can_accept_task().await {
                return Some(agent.clone());
            }
        }
        None
    }

    /// Selecciona el mejor agente para una tarea
    pub async fn select_best_agent_for_task(&self, task: &Task) -> Option<Arc<Agent>> {
        match &task.task_type {
            TaskType::MotorSearch { engine, .. } => {
                self.agents.iter().find(|a| a.motor_name == *engine).cloned()
            }
            TaskType::MathAnalysis { .. } => {
                // Julia NLP engine es mejor para análisis matemático
                self.agents
                    .iter()
                    .find(|a| a.motor_name == "julia_nlp")
                    .cloned()
            }
            TaskType::ChaosAnalysis { .. } => {
                // Julia para caos
                self.agents
                    .iter()
                    .find(|a| a.motor_name == "julia_nlp")
                    .cloned()
            }
            TaskType::BatchProcess { .. } => {
                // FAISS es bueno para procesamiento de batch con vectors
                self.agents
                    .iter()
                    .find(|a| a.motor_name == "faiss")
                    .cloned()
            }
            _ => self.get_available_agent().await,
        }
    }

    pub fn get_agent_by_motor(&self, motor_name: &str) -> Option<Arc<Agent>> {
        self.agents
            .iter()
            .find(|a| a.motor_name == motor_name)
            .cloned()
    }

    pub async fn get_poolstats(&self) -> serde_json::Value {
        let mut stats = Vec::new();
        for agent in &self.agents {
            let current = *agent.current_tasks.read().await;
            stats.push(serde_json::json!({
                "agent_id": agent.id,
                "motor": agent.motor_name,
                "active_tasks": current,
                "max_capacity": agent.max_parallel_tasks,
                "available": !current >= agent.max_parallel_tasks,
            }));
        }
        serde_json::json!({ "agents": stats })
    }
}

// ============================================================================
// ORCHESTRATOR PRINCIPAL - Coordina tareas paralelas
// ============================================================================

pub struct AgentOrchestrator {
    pool: Arc<AgentPool>,
    parallelism: usize,
    semaphore: Arc<Semaphore>,
    result_tx: mpsc::UnboundedSender<TaskResult>,
    result_rx: Arc<RwLock<mpsc::UnboundedReceiver<TaskResult>>>,
}

impl AgentOrchestrator {
    /// Crea nuevo orchestrator con control de paralelismo
    pub fn new(parallelism: usize) -> (Self, Arc<AgentPool>) {
        let (result_tx, result_rx) = mpsc::unbounded_channel();
        let pool = Arc::new(AgentPool::new(result_tx.clone()));

        (
            Self {
                pool: pool.clone(),
                parallelism,
                semaphore: Arc::new(Semaphore::new(parallelism)),
                result_tx,
                result_rx: Arc::new(RwLock::new(result_rx)),
            },
            pool,
        )
    }

    /// Ejecuta una tarea individual (puede ser paralela o secuencial)
    pub async fn execute_task(&self, task: Task) -> TaskResult {
        let task_id = task.id;
        let start = Instant::now();
        let timeout = task.timeout_ms;

        // Selecciona mejor agente
        let agent = match self.pool.select_best_agent_for_task(&task).await {
            Some(a) => a,
            None => {
                return TaskResult {
                    task_id,
                    status: TaskStatus::Failed,
                    result: serde_json::json!({"error": "No available agents"}),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    agent_id: "none".to_string(),
                };
            }
        };

        // Adquiere semáforo para controlar paralelismo global
        let _permit = match tokio::time::timeout(
            std::time::Duration::from_millis(timeout),
            self.semaphore.acquire(),
        )
        .await
        {
            Ok(Ok(p)) => p,
            _ => {
                return TaskResult {
                    task_id,
                    status: TaskStatus::Timeout,
                    result: serde_json::json!({"error": "Timeout waiting for semaphore"}),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    agent_id: agent.id.clone(),
                };
            }
        };

        agent.increment_tasks().await;

        // Ejecuta la tarea con timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(timeout),
            self.execute_task_internal(&task, agent.clone()),
        )
        .await;

        agent.decrement_tasks().await;

        match result {
            Ok(task_result) => task_result,
            Err(_) => TaskResult {
                task_id,
                status: TaskStatus::Timeout,
                result: serde_json::json!({"error": "Task execution timeout"}),
                execution_time_ms: start.elapsed().as_millis() as u64,
                agent_id: agent.id.clone(),
            },
        }
    }

    /// Ejecución interna de la tarea
    async fn execute_task_internal(&self, task: &Task, agent: Arc<Agent>) -> TaskResult {
        let task_id = task.id;
        let start = Instant::now();

        let result = match &task.task_type {
            TaskType::MotorSearch { engine, query } => {
                self.execute_motor_search(engine, query, agent.clone()).await
            }
            TaskType::MathAnalysis { data, operation } => {
                self.execute_math_analysis(data, operation).await
            }
            TaskType::ChaosAnalysis { metrics } => {
                self.execute_chaos_analysis(metrics).await
            }
            TaskType::BatchProcess { documents, operation } => {
                self.execute_batch_process(documents, operation).await
            }
            TaskType::PredictiveAnalysis { context_id, lookahead } => {
                self.execute_predictive_analysis(*context_id, *lookahead).await
            }
        };

        TaskResult {
            task_id,
            status: result.0,
            result: result.1,
            execution_time_ms: start.elapsed().as_millis() as u64,
            agent_id: agent.id.clone(),
        }
    }

    async fn execute_motor_search(
        &self,
        _engine: &str,
        query: &str,
        _agent: Arc<Agent>,
    ) -> (TaskStatus, serde_json::Value) {
        // Simulación - en producción, invocaría el motor real
        (
            TaskStatus::Completed,
            serde_json::json!({
                "query": query,
                "engine": _engine,
                "results": []
            }),
        )
    }

    async fn execute_math_analysis(
        &self,
        data: &[f64],
        operation: &str,
    ) -> (TaskStatus, serde_json::Value) {
        match operation {
            "mean" => {
                let mean = data.iter().sum::<f64>() / data.len() as f64;
                (
                    TaskStatus::Completed,
                    serde_json::json!({"operation": "mean", "result": mean}),
                )
            }
            "variance" => {
                let mean = data.iter().sum::<f64>() / data.len() as f64;
                let variance =
                    data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
                (
                    TaskStatus::Completed,
                    serde_json::json!({"operation": "variance", "result": variance}),
                )
            }
            _ => (
                TaskStatus::Failed,
                serde_json::json!({"error": format!("Unknown operation: {}", operation)}),
            ),
        }
    }

    async fn execute_chaos_analysis(&self, metrics: &[f64]) -> (TaskStatus, serde_json::Value) {
        // Llamaría a Julia FFI en producción
        let entropy = crate::ffi::julia::shannon_entropy(metrics);
        (
            TaskStatus::Completed,
            serde_json::json!({
                "chaos_metric": "entropy",
                "value": entropy,
                "metrics_count": metrics.len()
            }),
        )
    }

    async fn execute_batch_process(
        &self,
        documents: &[String],
        operation: &str,
    ) -> (TaskStatus, serde_json::Value) {
        // Usa Rayon para paralelización CPU-bound
        let results: Vec<_> = documents
            .par_iter()
            .map(|doc| {
                match operation {
                    "count_words" => doc.split_whitespace().count(),
                    "count_chars" => doc.len(),
                    _ => 0,
                }
            })
            .collect();

        (
            TaskStatus::Completed,
            serde_json::json!({
                "operation": operation,
                "processed": documents.len(),
                "results": results
            }),
        )
    }

    async fn execute_predictive_analysis(
        &self,
        _context_id: Uuid,
        _lookahead: usize,
    ) -> (TaskStatus, serde_json::Value) {
        // Placeholder - integrado con memory engine
        (
            TaskStatus::Completed,
            serde_json::json!({
                "context_id": _context_id,
                "predictions": []
            }),
        )
    }

    /// Ejecuta múltiples tareas en paralelo
    pub async fn execute_parallel(&self, tasks: Vec<Task>) -> Vec<TaskResult> {
        let futures: Vec<_> = tasks
            .into_iter()
            .map(|task| self.execute_task(task))
            .collect();

        futures::future::join_all(futures).await
    }

    /// Obtiene estadísticas del pool
    pub async fn get_stats(&self) -> serde_json::Value {
        self.pool.get_poolstats().await
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_pool_creation() {
        let (result_tx, _result_rx) = mpsc::unbounded_channel();
        let pool = AgentPool::new(result_tx);
        assert_eq!(pool.0.agents.len(), 9);
    }

    #[tokio::test]
    async fn test_parallel_execution() {
        let (orchestrator, _pool) = AgentOrchestrator::new(4);

        let tasks = vec![
            Task {
                id: Uuid::new_v4(),
                task_type: TaskType::MathAnalysis {
                    data: vec![1.0, 2.0, 3.0],
                    operation: "mean".to_string(),
                },
                priority: 100,
                created_at: 0,
                timeout_ms: 1000,
            },
            Task {
                id: Uuid::new_v4(),
                task_type: TaskType::MathAnalysis {
                    data: vec![1.0, 2.0, 3.0],
                    operation: "variance".to_string(),
                },
                priority: 100,
                created_at: 0,
                timeout_ms: 1000,
            },
        ];

        let results = orchestrator.execute_parallel(tasks).await;
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.status == TaskStatus::Completed));
    }
}
