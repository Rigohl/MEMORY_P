//! mcp/shared_memory_tools.rs - MCP tools para sistema de memoria compartida

use crate::error::Result;
use crate::shared_memory::{AgentId, MemoryStats, SharedMemorySystem};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Parámetros para obtener/crear contexto de agente
#[derive(Debug, Deserialize)]
pub struct GetContextParams {
    pub agent_id: String,
}

/// Parámetros para actualizar contexto de agente
#[derive(Debug, Deserialize)]
pub struct UpdateContextParams {
    pub agent_id: String,
    pub shared_data: Value,
}

/// Parámetros para sincronizar contextos
#[derive(Debug, Deserialize)]
pub struct SyncContextsParams {
    pub source_agent: String,
    pub target_agents: Vec<String>,
}

/// Parámetros para limpieza de contextos inactivos
#[derive(Debug, Deserialize)]
pub struct CleanupParams {
    pub max_age_seconds: u64,
}

/// Respuesta de estadísticas de memoria
#[derive(Debug, Serialize)]
pub struct MemoryStatsResponse {
    pub active_contexts: usize,
    pub persisted_contexts: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_rate: f64,
    pub total_updates: u64,
    pub memory_usage_bytes: u64,
    pub avg_latency_ms: f64,
}

impl From<MemoryStats> for MemoryStatsResponse {
    fn from(stats: MemoryStats) -> Self {
        Self {
            active_contexts: stats.active_contexts,
            persisted_contexts: stats.persisted_contexts,
            cache_hits: stats.cache_hits,
            cache_misses: stats.cache_misses,
            cache_hit_rate: stats.cache_hit_rate,
            total_updates: stats.total_updates,
            memory_usage_bytes: stats.memory_usage_bytes,
            avg_latency_ms: stats.avg_latency_ms,
        }
    }
}

/// Handler de herramientas MCP para memoria compartida
pub struct SharedMemoryToolHandler {
    system: Arc<SharedMemorySystem>,
}

impl SharedMemoryToolHandler {
    pub fn new(system: Arc<SharedMemorySystem>) -> Self {
        Self { system }
    }

    /// Obtiene o crea contexto para un agente
    pub async fn get_agent_context(&self, params: GetContextParams) -> Result<Value> {
        let agent_id = AgentId::new(params.agent_id);
        let context = self.system.get_or_create_context(agent_id).await?;

        Ok(serde_json::to_value(context)?)
    }

    /// Actualiza contexto de un agente
    pub async fn update_agent_context(&self, params: UpdateContextParams) -> Result<Value> {
        let agent_id = AgentId::new(params.agent_id);
        let mut context = self.system.get_or_create_context(agent_id.clone()).await?;

        // Actualizar shared_data
        if let Value::Object(map) = params.shared_data {
            for (key, value) in map {
                context.shared_data.insert(key, value);
            }
        }

        self.system
            .update_context(agent_id, context.clone())
            .await?;

        Ok(serde_json::json!({
            "success": true,
            "context_id": context.context_id.to_string(),
            "version": context.metadata.version
        }))
    }

    /// Sincroniza contextos entre agentes
    pub async fn sync_contexts(&self, params: SyncContextsParams) -> Result<Value> {
        let source_agent = AgentId::new(params.source_agent);
        let target_agents: Vec<AgentId> =
            params.target_agents.into_iter().map(AgentId::new).collect();

        self.system
            .sync_contexts(source_agent, target_agents.clone())
            .await?;

        Ok(serde_json::json!({
            "success": true,
            "synced_agents": target_agents.len()
        }))
    }

    /// Obtiene estadísticas del sistema de memoria
    pub async fn get_memory_stats(&self) -> Result<Value> {
        let stats = self.system.get_stats().await;
        let response = MemoryStatsResponse::from(stats);

        Ok(serde_json::to_value(response)?)
    }

    /// Limpia contextos inactivos
    pub async fn cleanup_inactive_contexts(&self, params: CleanupParams) -> Result<Value> {
        let cleaned: usize = self.system.cleanup_inactive(params.max_age_seconds).await?;

        Ok(serde_json::json!({
            "success": true,
            "cleaned_contexts": cleaned
        }))
    }
}

/// Registra las herramientas MCP para memoria compartida
pub fn register_shared_memory_tools() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "name": "get_agent_context",
            "description": "Obtiene o crea el contexto de memoria compartida para un agente",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "agent_id": {
                        "type": "string",
                        "description": "ID único del agente"
                    }
                },
                "required": ["agent_id"]
            }
        }),
        serde_json::json!({
            "name": "update_agent_context",
            "description": "Actualiza el contexto de memoria compartida de un agente",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "agent_id": {
                        "type": "string",
                        "description": "ID único del agente"
                    },
                    "shared_data": {
                        "type": "object",
                        "description": "Datos compartidos a actualizar (clave-valor)"
                    }
                },
                "required": ["agent_id", "shared_data"]
            }
        }),
        serde_json::json!({
            "name": "sync_agent_contexts",
            "description": "Sincroniza el contexto de un agente con otros agentes",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source_agent": {
                        "type": "string",
                        "description": "ID del agente fuente"
                    },
                    "target_agents": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "IDs de los agentes destino"
                    }
                },
                "required": ["source_agent", "target_agents"]
            }
        }),
        serde_json::json!({
            "name": "get_memory_stats",
            "description": "Obtiene estadísticas del sistema de memoria compartida",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        serde_json::json!({
            "name": "cleanup_inactive_contexts",
            "description": "Limpia contextos inactivos del sistema de memoria",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "max_age_seconds": {
                        "type": "number",
                        "description": "Edad máxima en segundos para considerar un contexto como inactivo"
                    }
                },
                "required": ["max_age_seconds"]
            }
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shared_memory_tools() {
        let system = Arc::new(SharedMemorySystem::new().await.unwrap());
        system.initialize().await.unwrap();

        let handler = SharedMemoryToolHandler::new(system.clone());

        // Test get context
        let params = GetContextParams {
            agent_id: "test-agent".to_string(),
        };
        let result = handler.get_agent_context(params).await;
        assert!(result.is_ok());

        // Test update context
        let params = UpdateContextParams {
            agent_id: "test-agent".to_string(),
            shared_data: serde_json::json!({"key": "value"}),
        };
        let result = handler.update_agent_context(params).await;
        assert!(result.is_ok());

        // Test get stats
        let result = handler.get_memory_stats().await;
        assert!(result.is_ok());

        system.shutdown().await.unwrap();
    }
}
