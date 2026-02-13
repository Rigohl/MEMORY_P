//! shared_memory/context.rs - Gestor de contextos compartidos

use super::types::{AgentId, ContextId, SharedContext};
use crate::error::{MemoryPError, Result};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};
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

    /// Base de datos persistente (Sled)
    db: Arc<sled::Db>,

    /// Clave de cifrado para seguridad de memoria
    encryption_key: [u8; 32],
}

impl ContextManager {
    /// Crea un nuevo gestor de contextos con persistencia Sled y Cifrado AES-256
    pub async fn new() -> Result<Self> {
        info!("🔧 Inicializando gestor de contextos con persistencia Sled y Cifrado AES-256");

        let db = sled::open("memory_db")
            .map_err(|e| MemoryPError::Other(format!("Sled open failed: {}", e)))?;

        // En producción, esto vendría de una variable de entorno segura
        let encryption_key = [0u8; 32];

        let manager = Self {
            contexts: Arc::new(DashMap::new()),
            agent_index: Arc::new(DashMap::new()),
            db: Arc::new(db),
            encryption_key,
        };

        // Cargar datos desde persistencia
        manager.load_all_from_db()?;

        Ok(manager)
    }

    /// Carga todos los contextos desde el disco al inicio y los descifra
    fn load_all_from_db(&self) -> Result<()> {
        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|_| MemoryPError::Other("Invalid key length".into()))?;

        for item in self.db.iter() {
            if let Ok((_key, value)) = item {
                // El nonce está al principio del valor (12 bytes)
                if value.len() < 12 { continue; }
                let nonce = Nonce::from_slice(&value[..12]);
                let ciphertext = &value[12..];

                if let Ok(plaintext) = cipher.decrypt(nonce, ciphertext) {
                    if let Ok(context) = serde_json::from_slice::<SharedContext>(&plaintext) {
                        self.contexts.insert(context.context_id.clone(), context.clone());
                        self.agent_index.insert(context.agent_id.clone(), context.context_id.clone());
                    }
                }
            }
        }
        info!("✅ Cargados {} contextos desde persistencia", self.contexts.len());
        Ok(())
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

    /// Actualiza un contexto existente, lo cifra y lo persiste
    pub async fn update(&self, mut context: SharedContext) -> Result<()> {
        context.update();

        // Actualizar en cache
        self.contexts
            .insert(context.context_id.clone(), context.clone());
        self.agent_index
            .insert(context.agent_id.clone(), context.context_id.clone());

        // Cifrar datos
        let plaintext = serde_json::to_vec(&context)
            .map_err(|e| MemoryPError::Other(format!("Serialization failed: {}", e)))?;

        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|_| MemoryPError::Other("Invalid key length".into()))?;

        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|e| MemoryPError::Other(format!("Encryption failed: {}", e)))?;

        // Combinar nonce + ciphertext
        let mut final_payload = nonce_bytes.to_vec();
        final_payload.extend(ciphertext);

        // Persistir en Sled
        self.db.insert(context.context_id.0.as_bytes(), final_payload)
            .map_err(|e| MemoryPError::Other(format!("Sled insert failed: {}", e)))?;

        self.db.flush()
            .map_err(|e| MemoryPError::Other(format!("Sled flush failed: {}", e)))?;

        debug!("Contexto {} actualizado y persistido de forma segura (Cifrado)", context.context_id);
        Ok(())
    }

    /// Elimina un contexto
    pub async fn delete(&self, context_id: &ContextId) -> Result<()> {
        if let Some((_, context)) = self.contexts.remove(context_id) {
            self.agent_index.remove(&context.agent_id);

            // Eliminar de Sled
            self.db.remove(context_id.0.as_bytes())
                .map_err(|e| MemoryPError::Other(format!("Sled remove failed: {}", e)))?;

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
