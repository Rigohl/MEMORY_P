//! Qdrant Data Models + MCP Context7 Integration
//!
//! Maps MEMORY_P documents → Qdrant Points + semantic search via Context7
//! Integrates with GitHub API for code examples

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::motores::core::types::{Document, SearchQuery, SearchResult, QueryType};

/// Qdrant Point - native format for vector storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantPoint {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: QdrantPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QdrantPayload {
    pub content: String,
    pub type_: String,
    pub source: String,
    pub language: Option<String>,
    pub collections: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: i64,
    pub semantic_tags: Vec<String>,
    pub indexed_fields: HashMap<String, String>,
}

impl QdrantPayload {
    pub fn new(content: String) -> Self {
        Self {
            content,
            type_: "document".to_string(),
            source: "memory_p".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            ..Default::default()
        }
    }
    
    pub fn with_type(mut self, t: &str) -> Self {
        self.type_ = t.to_string();
        self
    }
    
    pub fn with_language(mut self, lang: &str) -> Self {
        self.language = Some(lang.to_string());
        self
    }
    
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.semantic_tags = tags;
        self
    }
}

pub struct DocumentToQdrantMapper;

impl DocumentToQdrantMapper {
    pub fn map(doc: &Document, collection: &str) -> Result<QdrantPoint, String> {
        let vector = doc.vector.clone()
            .ok_or("Document must have vector embeddings")?;
        
        let language = Self::infer_language(&doc.content, &doc.metadata);
        let semantic_tags = Self::extract_semantic_tags(&doc.content, &language);
        
        let mut indexed_fields = HashMap::new();
        indexed_fields.insert("id".to_string(), doc.id.clone());
        if let Some(lang) = &language {
            indexed_fields.insert("language".to_string(), lang.clone());
        }
        
        let mut payload = QdrantPayload::new(doc.content.clone())
            .with_tags(semantic_tags);
        
        if let Some(lang) = language {
            payload = payload.with_language(&lang);
        }
        payload.collections.push(collection.to_string());
        payload.indexed_fields = indexed_fields;
        payload.metadata = doc.metadata.clone();
        
        Ok(QdrantPoint {
            id: doc.id.clone(),
            vector,
            payload,
        })
    }
    
    pub fn map_batch(docs: &[Document], collection: &str) -> Vec<QdrantPoint> {
        docs.iter()
            .filter_map(|doc| Self::map(doc, collection).ok())
            .collect()
    }
    
    fn infer_language(content: &str, _metadata: &HashMap<String, serde_json::Value>) -> Option<String> {
        if content.contains("fn ") && content.contains("async") {
            Some("rust".to_string())
        } else if content.contains("function ") || content.contains("=>") {
            Some("julia".to_string())
        } else if content.contains("def ") || content.contains("import ") {
            Some("python".to_string())
        } else if content.contains("const ") || content.contains("var ") {
            Some("zig".to_string())
        } else {
            None
        }
    }
    
    fn extract_semantic_tags(content: &str, language: &Option<String>) -> Vec<String> {
        let mut tags = vec![];
        if let Some(lang) = language { tags.push(format!("lang:{}", lang)); }
        if content.contains("search") || content.contains("query") { tags.push("search".to_string()); }
        if content.contains("vector") || content.contains("embedding") { tags.push("vector".to_string()); }
        if content.contains("async") || content.contains("await") { tags.push("async".to_string()); }
        if content.contains("test") || content.contains("bench") { tags.push("test".to_string()); }
        if content.contains("memory") || content.contains("cache") { tags.push("memory".to_string()); }
        if content.contains("optimization") || content.contains("performance") { tags.push("optimization".to_string()); }
        if content.contains("ffi") || content.contains("interop") { tags.push("ffi".to_string()); }
        tags
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SemanticRoute {
    pub tags: Vec<String>,
    pub suggested_motors: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantFilter {
    pub semantic_tags: Vec<String>,
    pub collections: Vec<String>,
    pub min_confidence: f32,
    pub intent: String,
}

pub struct Context7Query {
    pub intent: String,
    pub tags: Vec<String>,
    pub collections: Vec<String>,
    pub limit: usize,
    pub confidence: f32,
}

impl Context7Query {
    pub fn new(intent: &str) -> Self {
        Self {
            intent: intent.to_string(),
            tags: vec![],
            collections: vec!["default".to_string()],
            limit: 10,
            confidence: 0.7,
        }
    }
    
    pub fn with_collection(mut self, collection: &str) -> Self {
        self.collections = vec![collection.to_string()];
        self
    }
    
    pub fn to_qdrant_filter(&self) -> QdrantFilter {
        QdrantFilter {
            semantic_tags: self.tags.clone(),
            collections: self.collections.clone(),
            min_confidence: self.confidence,
            intent: self.intent.clone(),
        }
    }
}

pub struct MpcContext7Router {
    pub intent_mappings: HashMap<String, SemanticRoute>,
}

impl MpcContext7Router {
    pub fn new() -> Self {
        let mut intent_mappings = HashMap::new();
        
        intent_mappings.insert("find_similar_vectors".to_string(), SemanticRoute {
            tags: vec!["vector".to_string(), "semantic".to_string()],
            suggested_motors: vec!["qdrant".to_string(), "faiss".to_string()],
            description: "Vector similarity search".to_string(),
        });
        
        intent_mappings.insert("find_text_match".to_string(), SemanticRoute {
            tags: vec!["fulltext".to_string(), "text".to_string()],
            suggested_motors: vec!["tantivy".to_string(), "lnx".to_string()],
            description: "Full-text search".to_string(),
        });
        
        intent_mappings.insert("find_code_pattern".to_string(), SemanticRoute {
            tags: vec!["code".to_string(), "pattern".to_string()],
            suggested_motors: vec!["tantivy".to_string(), "memory_bank".to_string()],
            description: "Code pattern search".to_string(),
        });
        
        intent_mappings.insert("find_optimization".to_string(), SemanticRoute {
            tags: vec!["optimization".to_string(), "performance".to_string()],
            suggested_motors: vec!["memory_bank".to_string(), "qdrant".to_string()],
            description: "Performance optimization search".to_string(),
        });
        
        intent_mappings.insert("find_ffi_example".to_string(), SemanticRoute {
            tags: vec!["ffi".to_string(), "interop".to_string()],
            suggested_motors: vec!["memory_bank".to_string()],
            description: "FFI integration examples".to_string(),
        });
        
        Self { intent_mappings }
    }
    
    pub fn route(&self, intent: &str) -> Option<SemanticRoute> {
        self.intent_mappings.get(intent).cloned()
    }
    
    pub fn get_motor_for_intent(&self, intent: &str) -> Option<String> {
        self.route(intent).and_then(|r| r.suggested_motors.first().cloned())
    }
}

impl Default for MpcContext7Router {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub intent: String,
    pub selected_motors: Vec<String>,
    pub semantic_tags: Vec<String>,
    pub qdrant_filter: QdrantFilter,
    pub github_search: Option<String>,
    pub description: String,
}

pub struct QdrantContext7Workflow {
    router: MpcContext7Router,
}

impl QdrantContext7Workflow {
    pub fn new() -> Self {
        Self { router: MpcContext7Router::new() }
    }
    
    pub fn process_query(&self, intent: &str, query_text: &str) -> WorkflowResult {
        let route = self.router.route(intent).unwrap_or_else(|| SemanticRoute {
            tags: vec![],
            suggested_motors: vec!["qdrant".to_string()],
            description: "Generic search".to_string(),
        });
        
        let context7_query = Context7Query::new(query_text).with_collection("default");
        let qdrant_filter = context7_query.to_qdrant_filter();
        
        let github_search = if route.tags.contains(&"code".to_string()) {
            Some(format!("\"{}\" language:rust stars:>100", intent))
        } else { None };
        
        WorkflowResult {
            intent: intent.to_string(),
            selected_motors: route.suggested_motors,
            semantic_tags: route.tags,
            qdrant_filter,
            github_search,
            description: route.description,
        }
    }
}

impl Default for QdrantContext7Workflow {
    fn default() -> Self { Self::new() }
}

// Suppress unused import warnings for types brought in via pub use above
#[allow(unused_imports)]
use crate::motores::core::types::SearchResult as _SearchResult;
#[allow(unused_imports)]
use crate::motores::core::types::SearchQuery as _SearchQuery;
#[allow(unused_imports)]
use crate::motores::core::types::QueryType as _QueryType;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_to_qdrant() {
        let doc = Document {
            id: "doc-1".to_string(),
            content: "async fn search() {}".to_string(),
            vector: Some(vec![0.1, 0.2, 0.3]),
            metadata: Default::default(),
        };
        
        let point = DocumentToQdrantMapper::map(&doc, "test").unwrap();
        assert_eq!(point.id, "doc-1");
        assert_eq!(point.vector.len(), 3);
    }

    #[test]
    fn test_context7_router() {
        let router = MpcContext7Router::new();
        let route = router.route("find_similar_vectors");
        assert!(route.is_some());
        assert!(route.unwrap().tags.contains(&"vector".to_string()));
    }

    #[test]
    fn test_workflow() {
        let workflow = QdrantContext7Workflow::new();
        let result = workflow.process_query("find_similar_vectors", "test query");
        assert_eq!(result.intent, "find_similar_vectors");
        assert!(result.selected_motors.contains(&"qdrant".to_string()));
    }
}
