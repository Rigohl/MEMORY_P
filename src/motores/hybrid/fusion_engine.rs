//! Fusion engine for coordinating multiple search engines

use crate::motores::core::{traits::SearchEngine, types::*, RoutingAI};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Fusion engine that coordinates queries across multiple engines
pub struct FusionEngine {
    engines: Arc<RwLock<HashMap<String, Arc<dyn SearchEngine>>>>,
    router: Arc<RoutingAI>,
}

impl FusionEngine {
    pub fn new() -> Self {
        Self {
            engines: Arc::new(RwLock::new(HashMap::new())),
            router: Arc::new(RoutingAI::new()),
        }
    }

    pub async fn register_engine(&self, name: String, engine: Arc<dyn SearchEngine>) {
        let mut engines = self.engines.write().await;
        engines.insert(name, engine);
    }

    pub async fn search_multi(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        let engine_selections = self.router.route_query(query);
        let engines = self.engines.read().await;

        let mut all_results = Vec::new();

        for selection in engine_selections {
            if let EngineSelection::Primary(name) = selection {
                if let Some(engine) = engines.get(name) {
                    match engine.search(query).await {
                        Ok(mut results) => all_results.append(&mut results),
                        Err(_) => continue,
                    }
                }
            }
        }

        Ok(all_results)
    }
}

impl Default for FusionEngine {
    fn default() -> Self {
        Self::new()
    }
}
