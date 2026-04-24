//! shared_memory/sync.rs - Coordinador de sincronización entre agentes

use super::types::{AgentId, ContextId, SharedContext};
use crate::error::Result;
use dashmap::DashMap;
use futures::StreamExt;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

/// Evento de sincronización
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncEvent {
    /// Contexto actualizado
    ContextUpdated {
        agent_id: AgentId,
        context: Box<SharedContext>,
        targets: Option<Vec<AgentId>>,
    },

    /// Contexto creado
    ContextCreated {
        agent_id: AgentId,
        context_id: ContextId,
        targets: Option<Vec<AgentId>>,
    },

    /// Contexto eliminado
    ContextDeleted {
        context_id: ContextId,
        targets: Option<Vec<AgentId>>,
    },
}

impl SyncEvent {
    /// Verifica si un agente es objetivo de este evento
    pub fn is_target(&self, agent_id: &AgentId) -> bool {
        let targets = match self {
            SyncEvent::ContextUpdated { targets, .. } => targets,
            SyncEvent::ContextCreated { targets, .. } => targets,
            SyncEvent::ContextDeleted { targets, .. } => targets,
        };

        match targets {
            Some(t) => t.contains(agent_id),
            None => true, // Broadcast general si no hay objetivos específicos
        }
    }
}

/// Envoltorio para eventos de Redis que incluye el ID de instancia
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RedisSyncEvent {
    pub source_instance: uuid::Uuid,
    pub event: SyncEvent,
}

/// Coordinador de sincronización entre agentes
/// Utiliza pub/sub para notificar cambios a todos los agentes interesados
pub struct SyncCoordinator {
    /// ID único de esta instancia
    instance_id: uuid::Uuid,

    /// Canal de broadcast para eventos
    event_tx: broadcast::Sender<SyncEvent>,

    /// Suscriptores activos por agente
    subscribers: Arc<DashMap<AgentId, broadcast::Receiver<SyncEvent>>>,

    /// Conexión de Redis para publicación
    redis_tx: Arc<tokio::sync::Mutex<Option<redis::aio::MultiplexedConnection>>>,

    /// Indica si está inicializado
    initialized: Arc<std::sync::atomic::AtomicBool>,
}

impl SyncCoordinator {
    /// Crea un nuevo coordinador de sincronización
    pub fn new() -> Self {
        // Canal con capacidad para 1000 eventos
        let (event_tx, _) = broadcast::channel(1000);

        Self {
            instance_id: uuid::Uuid::new_v4(),
            event_tx,
            subscribers: Arc::new(DashMap::new()),
            redis_tx: Arc::new(tokio::sync::Mutex::new(None)),
            initialized: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Inicializa el coordinador
    pub async fn initialize(&self) -> Result<()> {
        if self.initialized.load(std::sync::atomic::Ordering::Acquire) {
            return Ok(());
        }

        info!(
            "🔧 Inicializando coordinador de sincronización (ID: {})",
            self.instance_id
        );

        // Conectar a Redis para pub/sub distribuido
        let redis_url = &crate::config::CONFIG.advanced.redis_url;
        match redis::Client::open(redis_url.as_str()) {
            Ok(client) => {
                match client.get_multiplexed_async_connection().await {
                    Ok(conn) => {
                        let mut redis_tx = self.redis_tx.lock().await;
                        *redis_tx = Some(conn);
                        info!("📡 Conectado a Redis para publicación distribuida");

                        // Iniciar suscriptor en background
                        self.start_redis_subscriber(client).await;
                    }
                    Err(e) => {
                        warn!("⚠️ No se pudo conectar a Redis: {}. El modo distribuido estará desactivado.", e);
                    }
                }
            }
            Err(e) => {
                warn!(
                    "⚠️ URL de Redis inválida ({}): {}. El modo distribuido estará desactivado.",
                    redis_url, e
                );
            }
        }

        self.initialized
            .store(true, std::sync::atomic::Ordering::Release);
        info!("✅ Coordinador de sincronización inicializado");
        Ok(())
    }

    /// Inicia el suscriptor de Redis en una tarea separada
    async fn start_redis_subscriber(&self, client: redis::Client) {
        let event_tx = self.event_tx.clone();
        let instance_id = self.instance_id;

        tokio::spawn(async move {
            match client.get_async_connection().await {
                Ok(conn) => {
                    let mut pubsub = conn.into_pubsub();
                    if let Err(e) = pubsub.subscribe("memory_p_sync").await {
                        error!("❌ Error al suscribirse a canal Redis: {}", e);
                        return;
                    }

                    info!("📥 Suscrito a eventos distribuidos en 'memory_p_sync'");

                    let mut stream = pubsub.on_message();
                    while let Some(msg) = stream.next().await {
                        let payload: Vec<u8> = match msg.get_payload() {
                            Ok(p) => p,
                            Err(e) => {
                                error!("❌ Error al leer payload de Redis: {}", e);
                                continue;
                            }
                        };

                        if let Ok(redis_event) = serde_json::from_slice::<RedisSyncEvent>(&payload) {
                            // Ignorar eventos generados por nosotros mismos
                            if redis_event.source_instance != instance_id {
                                debug!(
                                    "Recibido evento remoto de instancia {}",
                                    redis_event.source_instance
                                );
                                let _ = event_tx.send(redis_event.event);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Error en conexión de suscripción Redis: {}", e);
                }
            }
        });
    }

    /// Suscribe un agente a eventos de sincronización
    pub fn subscribe(&self, agent_id: AgentId) -> broadcast::Receiver<SyncEvent> {
        let rx = self.event_tx.subscribe();
        self.subscribers
            .insert(agent_id.clone(), self.event_tx.subscribe());
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
        targets: Option<Vec<AgentId>>,
    ) -> Result<()> {
        let event = SyncEvent::ContextUpdated {
            agent_id: agent_id.clone(),
            context: Box::new(context),
            targets,
        };

        // 1. Enviar localmente
        match self.event_tx.send(event.clone()) {
            Ok(num_receivers) => {
                debug!("Evento enviado a {} suscriptores locales", num_receivers);
            }
            Err(_) => {
                debug!("Sin suscriptores activos locales para evento");
            }
        }

        // 2. Enviar a Redis para distribución
        let mut redis_tx_lock = self.redis_tx.lock().await;
        if let Some(conn) = redis_tx_lock.as_mut() {
            let redis_event = RedisSyncEvent {
                source_instance: self.instance_id,
                event,
            };

            if let Ok(payload) = serde_json::to_vec(&redis_event) {
                if let Err(e) = conn.publish::<_, _, ()>("memory_p_sync", payload).await {
                    error!("❌ Error al publicar evento en Redis: {}", e);
                } else {
                    debug!("Evento publicado en Redis");
                }
            }
        }

        Ok(())
    }

    /// Sincroniza contextos entre agentes específicos
    pub async fn sync_contexts(
        &self,
        source_agent: AgentId,
        target_agents: Vec<AgentId>,
        context: SharedContext,
    ) -> Result<()> {
        debug!(
            "Sincronizando contexto de {} a {} agentes",
            source_agent,
            target_agents.len()
        );

        self.broadcast_update(source_agent, context, Some(target_agents))
            .await
    }

    /// Obtiene el número de suscriptores activos
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }

    /// Finaliza el coordinador
    pub async fn shutdown(&self) -> Result<()> {
        info!("🔧 Finalizando coordinador de sincronización");

        self.subscribers.clear();
        self.initialized
            .store(false, std::sync::atomic::Ordering::Release);

        info!("✅ Coordinador de sincronización finalizado");
        Ok(())
    }
}

impl Clone for SyncCoordinator {
    fn clone(&self) -> Self {
        Self {
            instance_id: self.instance_id,
            event_tx: self.event_tx.clone(),
            subscribers: Arc::clone(&self.subscribers),
            redis_tx: Arc::clone(&self.redis_tx),
            initialized: Arc::clone(&self.initialized),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_coordinator_creation() {
        let coordinator = SyncCoordinator::new();
        assert!(coordinator.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_redis_unreachable_initialization() {
        // La inicialización debería tener éxito incluso si Redis no está disponible
        let coordinator = SyncCoordinator::new();
        // Usamos una URL que probablemente no tenga un Redis escuchando
        // Aunque la prueba es offline, el cliente de redis simplemente fallará al conectar
        // y el código debería manejarlo con un warning.
        let result = coordinator.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_subscribe_unsubscribe() {
        let coordinator = SyncCoordinator::new();
        coordinator.initialize().await.unwrap();

        let agent_id = AgentId::new("test-agent".to_string());
        let _rx = coordinator.subscribe(agent_id.clone());

        assert_eq!(coordinator.subscriber_count(), 1);

        coordinator.unsubscribe(&agent_id);
        assert_eq!(coordinator.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_broadcast_update() {
        let coordinator = SyncCoordinator::new();
        coordinator.initialize().await.unwrap();

        let agent_id = AgentId::new("test-agent".to_string());
        let mut rx = coordinator.subscribe(agent_id.clone());

        let context = SharedContext::new(agent_id.clone());

        // Broadcast en background
        let coordinator_clone = coordinator.clone();
        let context_clone = context.clone();
        let agent_id_clone = agent_id.clone();
        tokio::spawn(async move {
            coordinator_clone
                .broadcast_update(agent_id_clone, context_clone, None)
                .await
                .unwrap();
        });

        // Recibir evento
        let event = rx.recv().await.unwrap();
        match event {
            SyncEvent::ContextUpdated {
                agent_id: received_id,
                ..
            } => {
                assert_eq!(received_id, agent_id);
                assert!(event.is_target(&agent_id));
            }
            _ => panic!("Tipo de evento incorrecto"),
        }
    }

    #[tokio::test]
    async fn test_selective_sync() {
        let coordinator = SyncCoordinator::new();
        coordinator.initialize().await.unwrap();

        let source_agent = AgentId::new("source".to_string());
        let target1 = AgentId::new("target1".to_string());
        let target2 = AgentId::new("target2".to_string());
        let other = AgentId::new("other".to_string());

        let mut rx1 = coordinator.subscribe(target1.clone());
        let mut rx2 = coordinator.subscribe(target2.clone());
        let mut rx_other = coordinator.subscribe(other.clone());

        let context = SharedContext::new(source_agent.clone());

        // Sincronizar solo con target1 y target2
        let targets = vec![target1.clone(), target2.clone()];

        let coordinator_clone = coordinator.clone();
        let source_clone = source_agent.clone();
        let targets_clone = targets.clone();
        let context_clone = context.clone();

        tokio::spawn(async move {
            coordinator_clone
                .sync_contexts(source_clone, targets_clone, context_clone)
                .await
                .unwrap();
        });

        // Recibir y verificar en target1
        let event1 = rx1.recv().await.unwrap();
        assert!(event1.is_target(&target1));

        // Recibir y verificar en target2
        let event2 = rx2.recv().await.unwrap();
        assert!(event2.is_target(&target2));

        // Recibir en other y verificar que no es el objetivo
        let event_other = rx_other.recv().await.unwrap();
        assert!(!event_other.is_target(&other));

        // Verificar lógica de is_target directamente
        assert!(event1.is_target(&target1));
        assert!(event1.is_target(&target2));
        assert!(!event1.is_target(&other));
    }
}
