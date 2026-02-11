//! shared_memory/mod.rs - Sistema de Memoria Compartida para Agentes MCP
//!
//! Este módulo proporciona infraestructura de memoria compartida de alta velocidad
//! para coordinación entre agentes en MEMORY_P v2.0.
//!
//! Características:
//! - Buffers de alta velocidad con Zig FFI (zero-latency)
//! - Concurrencia segura con DashMap y SCC
//! - Sincronización proactiva entre agentes
//! - Persistencia en PostgreSQL + Redis cache
//! - Monitoreo en tiempo real

pub mod buffer;
pub mod cleanup;
pub mod context;
pub mod engine_integration;
pub mod monitor;
pub mod sync;
pub mod types;

use crate::hyper_memory::HyperMemoryManager;
pub use buffer::SharedMemoryBuffer;
pub use cleanup::CleanupManager;
pub use context::ContextManager;
pub use engine_integration::{EngineIntegration, EngineIntegrationConfig, IntegrationStats};
pub use monitor::MemoryMonitor;
pub use sync::SyncCoordinator;
pub use types::{AgentId, ContextId, ContextMetadata, MemoryStats, SharedContext};

use crate::error::Result;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Sistema central de memoria compartida
#[derive(Clone)]
pub struct SharedMemorySystem {
    /// Gestor de contextos
    context_manager: Arc<ContextManager>,

    /// Buffer de alta velocidad (Zig FFI)
    buffer: Arc<SharedMemoryBuffer>,

    /// Coordinador de sincronización
    sync_coordinator: Arc<SyncCoordinator>,

    /// Monitor de memoria
    monitor: Arc<MemoryMonitor>,

    /// Gestor de limpieza
    cleanup_manager: Arc<CleanupManager>,

    /// Integración con motores de búsqueda
    engine_integration: Arc<EngineIntegration>,

    /// Gestor de memoria hiperestructurada (Vectorial + Textual)
    hyper_memory: Arc<HyperMemoryManager>,

    /// Grafo de memoria relacional
    graph: Arc<RelationalMemoryGraph>,

    /// Cache de contextos activos (AgentId -> Context)
    active_contexts: Arc<DashMap<AgentId, SharedContext>>,

    /// Estado de inicialización
    initialized: Arc<RwLock<bool>>,
}

impl SharedMemorySystem {
    /// Crea una nueva instancia del sistema de memoria compartida
    pub async fn new() -> Result<Self> {
        info!("🧠 Inicializando sistema de memoria compartida");

        let context_manager = Arc::new(ContextManager::new().await?);
        let buffer = Arc::new(SharedMemoryBuffer::new()?);
        let sync_coordinator = Arc::new(SyncCoordinator::new().await?);
        let monitor = Arc::new(MemoryMonitor::new());
        let cleanup_manager = Arc::new(CleanupManager::new());
        let engine_integration =
            Arc::new(EngineIntegration::new(EngineIntegrationConfig::default()));
        let hyper_memory = Arc::new(HyperMemoryManager::new(384)); // Dimensión para embeddings BERT/JAX
        let graph = Arc::new(RelationalMemoryGraph::new());
        let active_contexts = Arc::new(DashMap::new());

        Ok(Self {
            context_manager,
            buffer,
            sync_coordinator,
            monitor,
            cleanup_manager,
            engine_integration,
            hyper_memory,
            graph,
            active_contexts,
            initialized: Arc::new(RwLock::new(false)),
        })
    }

    /// Inicializa el sistema completo
    pub async fn initialize(&self) -> Result<()> {
        let mut init = self.initialized.write().await;
        if *init {
            warn!("⚠️  Sistema de memoria compartida ya inicializado");
            return Ok(());
        }

        info!("🔧 Inicializando componentes del sistema de memoria compartida");

        // Inicializar buffer de alta velocidad
        self.buffer.initialize()?;

        // Inicializar coordinador de sincronización
        self.sync_coordinator.initialize().await?;

        // Inicializar integración con motores
        self.engine_integration.initialize().await?;

        // Inicializar monitor
        self.monitor.start().await;

        // Inicializar limpieza automática
        self.cleanup_manager
            .start(self.context_manager.clone(), self.active_contexts.clone())
            .await;

        *init = true;
        info!("✅ Sistema de memoria compartida inicializado correctamente");
        Ok(())
    }

    /// Obtiene o crea un contexto para un agente
    pub async fn get_or_create_context(&self, agent_id: AgentId) -> Result<SharedContext> {
        // Buscar en cache primero
        if let Some(ctx) = self.active_contexts.get(&agent_id) {
            self.monitor.record_cache_hit().await;
            return Ok(ctx.clone());
        }

        self.monitor.record_cache_miss().await;

        // Crear o recuperar desde persistencia
        let context = self.context_manager.get_or_create(agent_id.clone()).await?;

        // Guardar en cache
        self.active_contexts.insert(agent_id, context.clone());

        Ok(context)
    }

    /// Actualiza el contexto de un agente
    pub async fn update_context(&self, agent_id: AgentId, context: SharedContext) -> Result<()> {
        // Actualizar en cache
        self.active_contexts
            .insert(agent_id.clone(), context.clone());

        // Persistir
        self.context_manager.update(context.clone()).await?;

        // Indexar en motores de búsqueda
        self.engine_integration.index_context(&context).await?;

        // Notificar a otros agentes
        self.sync_coordinator
            .broadcast_update(agent_id, context)
            .await?;

        self.monitor.record_update().await;
        Ok(())
    }

    /// Sincroniza contextos entre agentes
    pub async fn sync_contexts(
        &self,
        source_agent: AgentId,
        target_agents: Vec<AgentId>,
    ) -> Result<()> {
        self.sync_coordinator
            .sync_contexts(source_agent, target_agents)
            .await
    }

    /// Obtiene estadísticas del sistema
    pub async fn get_stats(&self) -> MemoryStats {
        self.monitor.get_stats().await
    }

    /// Limpia contextos inactivos
    pub async fn cleanup_inactive(&self, max_age_secs: u64) -> Result<usize> {
        self.cleanup_manager.cleanup_inactive(max_age_secs).await
    }

    /// Busca contextos similares usando vector search
    pub async fn search_similar_contexts(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SharedContext>> {
        self.engine_integration
            .search_similar_contexts(query, limit)
            .await
    }

    /// Busca contextos por texto usando full-text search
    pub async fn search_contexts_by_text(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SharedContext>> {
        self.engine_integration
            .search_contexts_by_text(query, limit)
            .await
    }

    /// Pre-carga contextos relevantes para un agente
    pub async fn preload_relevant_contexts(
        &self,
        agent_id: &AgentId,
    ) -> Result<Vec<SharedContext>> {
        self.engine_integration
            .preload_relevant_contexts(agent_id)
            .await
    }

    /// Obtiene estadísticas de integración con motores
    pub async fn get_integration_stats(&self) -> IntegrationStats {
        self.engine_integration.get_integration_stats().await
    }

    /// Obtiene acceso al gestor de memoria hiperestructurada
    pub fn hyper_memory(&self) -> Arc<HyperMemoryManager> {
        self.hyper_memory.clone()
    }

    /// Sistema de autogestión de memoria (Auto-moving Context)
    /// Mueve y optimiza los contextos según su relevancia y uso
    pub async fn auto_manage_memory(&self) -> Result<()> {
        info!("🧠 Ejecutando autogestión de memoria (auto-moving context)");

        let contexts = self.context_manager.get_all_contexts();
        for context in contexts {
            // Lógica de migración inteligente:
            // - Memorias con alta frecuencia de acceso -> Asegurar en lóbulos rápidos (Redis)
            // - Memorias antiguas o frías -> Archivar en Postgres/ClickHouse
            // - Hechos detectados -> Indexar en Tantivy (Episódica)
            // - Conceptos abstractos -> Indexar en Qdrant (Semántica)

            self.engine_integration.index_context(&context).await?;
        }

        Ok(())
    }

    /// Finaliza el sistema de memoria compartida
    pub async fn shutdown(&self) -> Result<()> {
        info!("🔧 Finalizando sistema de memoria compartida");

        // Detener integración con motores
        self.engine_integration.shutdown().await?;

        // Detener limpieza automática
        self.cleanup_manager.stop().await;

        // Detener monitor
        self.monitor.stop().await;

        // Finalizar sincronización
        self.sync_coordinator.shutdown().await?;

        // Persistir todos los contextos activos
        for entry in self.active_contexts.iter() {
            let (_, context) = entry.pair();
            if let Err(e) = self.context_manager.update(context.clone()).await {
                error!("Error persistiendo contexto: {}", e);
            }
        }

        // Limpiar cache
        self.active_contexts.clear();

        let mut init = self.initialized.write().await;
        *init = false;

        info!("✅ Sistema de memoria compartida finalizado");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_initialization() {
        let system = SharedMemorySystem::new().await.unwrap();
        assert!(system.initialize().await.is_ok());
        assert!(system.shutdown().await.is_ok());
    }

    #[tokio::test]
    async fn test_context_creation() {
        let system = SharedMemorySystem::new().await.unwrap();
        system.initialize().await.unwrap();

        let agent_id = AgentId::new("test-agent".to_string());
        let context = system
            .get_or_create_context(agent_id.clone())
            .await
            .unwrap();

        assert_eq!(context.agent_id, agent_id);

        system.shutdown().await.unwrap();
    }
}

impl SharedMemorySystem {
    /// Obtiene el gestor de contextos
    pub fn get_graph(&self) -> Arc<RelationalMemoryGraph> {
        self.graph.clone()
    }

    pub fn get_context_manager(&self) -> Arc<ContextManager> {
        self.context_manager.clone()
    }
}
pub mod graph;
use crate::shared_memory::graph::RelationalMemoryGraph;
