//! shared_memory/sync.rs - Coordinador de sincronización entre agentes

use super::types::{SharedContext, AgentId, ContextId};
use crate::error::Result;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, debug};

/// Evento de sincronización
#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// Contexto actualizado
    ContextUpdated {
        agent_id: AgentId,
        context: SharedContext,
    },
    
    /// Contexto creado
    ContextCreated {
        agent_id: AgentId,
        context_id: ContextId,
    },
    
    /// Contexto eliminado
    ContextDeleted {
        context_id: ContextId,
    },
}

/// Coordinador de sincronización entre agentes
/// Utiliza pub/sub para notificar cambios a todos los agentes interesados
pub struct SyncCoordinator {
    /// Canal de broadcast para eventos
    event_tx: broadcast::Sender<SyncEvent>,
    
    /// Suscriptores activos por agente
    subscribers: Arc<DashMap<AgentId, broadcast::Receiver<SyncEvent>>>,
    
    /// Indica si está inicializado
    initialized: Arc<std::sync::atomic::AtomicBool>,
}

impl SyncCoordinator {
    /// Crea un nuevo coordinador de sincronización
    pub async fn new() -> Result<Self> {
        // Canal con capacidad para 1000 eventos
        let (event_tx, _) = broadcast::channel(1000);
        
        Ok(Self {
            event_tx,
            subscribers: Arc::new(DashMap::new()),
            initialized: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }
    
    /// Inicializa el coordinador
    pub async fn initialize(&self) -> Result<()> {
        if self.initialized.load(std::sync::atomic::Ordering::Acquire) {
            return Ok(());
        }
        
        info!("🔧 Inicializando coordinador de sincronización");
        
        // TODO: Conectar a Redis para pub/sub distribuido
        // let redis_client = connect_to_redis().await.ok();
        
        self.initialized.store(true, std::sync::atomic::Ordering::Release);
        info!("✅ Coordinador de sincronización inicializado");
        Ok(())
    }
    
    /// Suscribe un agente a eventos de sincronización
    pub fn subscribe(&self, agent_id: AgentId) -> broadcast::Receiver<SyncEvent> {
        let rx = self.event_tx.subscribe();
        self.subscribers.insert(agent_id.clone(), self.event_tx.subscribe());
        debug!("Agente {} suscrito a eventos", agent_id);
        rx
    }
    
    /// Desuscribe un agente
    pub fn unsubscribe(&self, agent_id: &AgentId) {
        self.subscribers.remove(agent_id);
        debug!("Agente {} desuscrito de eventos", agent_id);
    }
    
    /// Transmite una actualización de contexto a todos los suscriptores
    pub async fn broadcast_update(
        &self,
        agent_id: AgentId,
        context: SharedContext,
    ) -> Result<()> {
        let event = SyncEvent::ContextUpdated {
            agent_id: agent_id.clone(),
            context,
        };
        
        match self.event_tx.send(event) {
            Ok(num_receivers) => {
                debug!("Evento enviado a {} suscriptores", num_receivers);
                Ok(())
            }
            Err(_) => {
                // No hay receptores activos, no es un error
                debug!("Sin suscriptores activos para evento");
                Ok(())
            }
        }
    }
    
    /// Sincroniza contextos entre agentes específicos
    pub async fn sync_contexts(
        &self,
        source_agent: AgentId,
        target_agents: Vec<AgentId>,
    ) -> Result<()> {
        debug!(
            "Sincronizando contexto de {} a {} agentes",
            source_agent,
            target_agents.len()
        );
        
        // TODO: Implementar sincronización selectiva
        // Por ahora, usar broadcast general
        
        Ok(())
    }
    
    /// Obtiene el número de suscriptores activos
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }
    
    /// Finaliza el coordinador
    pub async fn shutdown(&self) -> Result<()> {
        info!("🔧 Finalizando coordinador de sincronización");
        
        self.subscribers.clear();
        self.initialized.store(false, std::sync::atomic::Ordering::Release);
        
        info!("✅ Coordinador de sincronización finalizado");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_sync_coordinator_creation() {
        let coordinator = SyncCoordinator::new().await.unwrap();
        assert!(coordinator.initialize().await.is_ok());
    }
    
    #[tokio::test]
    async fn test_subscribe_unsubscribe() {
        let coordinator = SyncCoordinator::new().await.unwrap();
        coordinator.initialize().await.unwrap();
        
        let agent_id = AgentId::new("test-agent".to_string());
        let _rx = coordinator.subscribe(agent_id.clone());
        
        assert_eq!(coordinator.subscriber_count(), 1);
        
        coordinator.unsubscribe(&agent_id);
        assert_eq!(coordinator.subscriber_count(), 0);
    }
    
    #[tokio::test]
    async fn test_broadcast_update() {
        let coordinator = SyncCoordinator::new().await.unwrap();
        coordinator.initialize().await.unwrap();
        
        let agent_id = AgentId::new("test-agent".to_string());
        let mut rx = coordinator.subscribe(agent_id.clone());
        
        let context = SharedContext::new(agent_id.clone());
        
        // Broadcast en background
        let coordinator_clone = coordinator.clone();
        let context_clone = context.clone();
        let agent_id_clone = agent_id.clone();
        tokio::spawn(async move {
            coordinator_clone.broadcast_update(agent_id_clone, context_clone).await.unwrap();
        });
        
        // Recibir evento
        let event = rx.recv().await.unwrap();
        match event {
            SyncEvent::ContextUpdated { agent_id: received_id, .. } => {
                assert_eq!(received_id, agent_id);
            }
            _ => panic!("Tipo de evento incorrecto"),
        }
    }
}

impl Clone for SyncCoordinator {
    fn clone(&self) -> Self {
        Self {
            event_tx: self.event_tx.clone(),
            subscribers: Arc::clone(&self.subscribers),
            initialized: Arc::clone(&self.initialized),
        }
    }
}
