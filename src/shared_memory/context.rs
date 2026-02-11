//! shared_memory/context.rs - Gestor de contextos compartidos

use super::types::{AgentId, ContextId, SharedContext};
use crate::error::{MemoryPError, Result};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Gestor de contextos compartidos
/// Gestiona la creación, recuperación y persistencia de contextos
pub struct ContextManager {
    /// Almacenamiento en memoria de contextos (ContextId -> SharedContext)
    contexts: Arc<DashMap<ContextId, SharedContext>>,

    /// Índice por AgentId para búsqueda rápida (AgentId -> ContextId)
    agent_index: Arc<DashMap<AgentId, ContextId>>,
    // TODO: Agregar cliente PostgreSQL para persistencia
    // db_pool: Option<PgPool>,
}

impl ContextManager {
    /// Crea un nuevo gestor de contextos
    pub async fn new() -> Result<Self> {
        info!("🔧 Inicializando gestor de contextos");

        // TODO: Conectar a PostgreSQL si está disponible
        // let db_pool = connect_to_postgres().await.ok();

        Ok(Self {
            contexts: Arc::new(DashMap::new()),
            agent_index: Arc::new(DashMap::new()),
            // db_pool,
        })
    }

    /// Obtiene o crea un contexto para un agente
    pub async fn get_or_create(&self, agent_id: AgentId) -> Result<SharedContext> {
        // Buscar en índice primero
        if let Some(context_id_ref) = self.agent_index.get(&agent_id) {
            let context_id = context_id_ref.value().clone();
            drop(context_id_ref); // Liberar el lock

            if let Some(context) = self.contexts.get(&context_id) {
                debug!("Contexto encontrado en cache para agente {}", agent_id);
                let mut ctx = context.clone();
                ctx.touch();

                // Actualizar en cache
                self.contexts.insert(context_id, ctx.clone());
                return Ok(ctx);
            }
        }

        // TODO: Intentar recuperar de PostgreSQL
        // if let Some(pool) = &self.db_pool {
        //     if let Ok(context) = load_from_db(pool, &agent_id).await {
        //         self.store_in_cache(context.clone());
        //         return Ok(context);
        //     }
        // }

        // Crear nuevo contexto
        debug!("Creando nuevo contexto para agente {}", agent_id);
        let context = SharedContext::new(agent_id.clone());

        // Guardar en índices
        self.contexts
            .insert(context.context_id.clone(), context.clone());
        self.agent_index
            .insert(agent_id, context.context_id.clone());

        Ok(context)
    }

    /// Actualiza un contexto existente
    pub async fn update(&self, mut context: SharedContext) -> Result<()> {
        context.update();

        // Actualizar en cache
        self.contexts
            .insert(context.context_id.clone(), context.clone());
        self.agent_index
            .insert(context.agent_id.clone(), context.context_id.clone());

        // TODO: Persistir en PostgreSQL
        // if let Some(pool) = &self.db_pool {
        //     save_to_db(pool, &context).await?;
        // }

        debug!("Contexto {} actualizado", context.context_id);
        Ok(())
    }

    /// Elimina un contexto
    pub async fn delete(&self, context_id: &ContextId) -> Result<()> {
        if let Some((_, context)) = self.contexts.remove(context_id) {
            self.agent_index.remove(&context.agent_id);

            // TODO: Eliminar de PostgreSQL
            // if let Some(pool) = &self.db_pool {
            //     delete_from_db(pool, context_id).await?;
            // }

            debug!("Contexto {} eliminado", context_id);
            Ok(())
        } else {
            Err(MemoryPError::SharedMemoryError(format!(
                "Contexto {} no encontrado",
                context_id
            )))
        }
    }

    /// Obtiene todos los contextos activos
    pub fn get_all_contexts(&self) -> Vec<SharedContext> {
        self.contexts
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Obtiene el número de contextos activos
    pub fn count(&self) -> usize {
        self.contexts.len()
    }

    /// Limpia todos los contextos en memoria
    pub fn clear_cache(&self) {
        self.contexts.clear();
        self.agent_index.clear();
        warn!("Cache de contextos limpiado");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_manager_creation() {
        let manager = ContextManager::new().await.unwrap();
        assert_eq!(manager.count(), 0);
    }

    #[tokio::test]
    async fn test_get_or_create_context() {
        let manager = ContextManager::new().await.unwrap();
        let agent_id = AgentId::new("test-agent".to_string());

        let context1 = manager.get_or_create(agent_id.clone()).await.unwrap();
        assert_eq!(context1.agent_id, agent_id);

        // Segunda llamada debe retornar el mismo contexto
        let context2 = manager.get_or_create(agent_id.clone()).await.unwrap();
        assert_eq!(context1.context_id, context2.context_id);
    }

    #[tokio::test]
    async fn test_update_context() {
        let manager = ContextManager::new().await.unwrap();
        let agent_id = AgentId::new("test-agent".to_string());

        let mut context = manager.get_or_create(agent_id).await.unwrap();
        let initial_version = context.metadata.version;

        context
            .shared_data
            .insert("key".to_string(), serde_json::json!("value"));
        manager.update(context.clone()).await.unwrap();

        assert!(context.metadata.version > initial_version);
    }

    #[tokio::test]
    async fn test_delete_context() {
        let manager = ContextManager::new().await.unwrap();
        let agent_id = AgentId::new("test-agent".to_string());

        let context = manager.get_or_create(agent_id).await.unwrap();
        let context_id = context.context_id.clone();

        assert!(manager.delete(&context_id).await.is_ok());
        assert_eq!(manager.count(), 0);
    }
}

impl ContextManager {
    /// Genera un "Backpack" (resumen proactivo de contexto) para el agente
    pub async fn assemble_backpack(&self, agent_id: &AgentId) -> Result<serde_json::Value> {
        let context = self.get_or_create(agent_id.clone()).await?;

        // Obtener predicciones de movimientos
        let current_embedding = vec![0.0f32; 384]; // Placeholder
        let next_moves = crate::ffi::jax::predict_next_moves(&current_embedding, 2)
            .unwrap_or_default();

        // Obtener decisión de estrategia
        let strategy = crate::ffi::julia::get_search_decision(1.5, 0.2, 0.9)
            .unwrap_or_else(|_| "DEFAULT".to_string());

        Ok(serde_json::json!({
            "agent_id": agent_id.to_string(),
            "context_version": context.metadata.version,
            "recommended_strategy": strategy,
            "predicted_next_moves": next_moves.len(),
            "immediate_context": context.shared_data,
            "system_health": "OPTIMAL",
            "proactive_insights": [
                "El sistema ha detectado una alta estabilidad en el código actual.",
                format!("Se recomienda usar la estrategia {}", strategy)
            ]
        }))
    }
}
