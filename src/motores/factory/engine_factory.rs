//! Engine factory for creating search engine instances

use crate::motores::{
    core::types::EngineConfig,
    specialized::{JuliaNlpEngine, MemoryBankEngine},
    text_search::{LnxEngine, MeiliSearchEngine, TantivyEngine, ToshiEngine},
    vector_search::{FaissEngine, QdrantEngine, ScannEngine},
};
use std::error::Error;
use std::sync::Arc;

/// Factory for creating search engine instances
pub struct EngineFactory;

impl EngineFactory {
    /// Create an engine by name
    pub fn create_engine(
        name: &str,
        config: EngineConfig,
    ) -> Result<Arc<dyn crate::motores::core::traits::SearchEngine>, Box<dyn Error>> {
        match name {
            // Vector search engines
            "qdrant" => Ok(Arc::new(QdrantEngine::new(config))),
            "faiss" => Ok(Arc::new(FaissEngine::new(config))),
            "scann" => Ok(Arc::new(ScannEngine::new(config))),

            // Text search engines
            "tantivy" => Ok(Arc::new(TantivyEngine::new(config))),
            "lnx" => Ok(Arc::new(LnxEngine::new(config))),
            "toshi" => Ok(Arc::new(ToshiEngine::new(config))),
            "meilisearch" => Ok(Arc::new(MeiliSearchEngine::new(config))),

            // Specialized engines
            "julia_nlp" => Ok(Arc::new(JuliaNlpEngine::new(config))),
            "memory_bank" => Ok(Arc::new(MemoryBankEngine::new(config))),

            _ => Err(format!("Unknown engine: {}", name).into()),
        }
    }

    /// Get list of all available engine names
    pub fn available_engines() -> Vec<&'static str> {
        vec![
            "qdrant",
            "faiss",
            "scann",
            "tantivy",
            "lnx",
            "toshi",
            "meilisearch",
            "julia_nlp",
            "memory_bank",
        ]
    }

    /// Check if an engine name is valid
    pub fn is_valid_engine(name: &str) -> bool {
        Self::available_engines().contains(&name)
    }
}
