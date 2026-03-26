//! Qdrant Data Models + MCP Context7 Integration
//!
//! Maps MEMORY_P documents → Qdrant Points + semantic search via Context7
//! Integrates with GitHub API for code examples

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::core::types::{Document, SearchQuery, SearchResult, QueryType};

/// ════════════════════════════════════════════════════════════════
/// QDRANT DATA MODELS (Point = Vector + Payload)
/// ════════════════════════════════════════════════════════════════

/// Qdrant Point - native format for vector storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantPoint {
    /// Unique ID (from Document)
    pub id: String,
    
    /// Vector (embeddings)
    pub vector: Vec<f32>,
    
    /// Metadata (Qdrant "payload" - flexible JSON)
    pub payload: QdrantPayload,
}

/// Qdrant Payload - flexible metadata structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QdrantPayload {
    /// Document content (indexed for full-text)
    pub content: String,
    
    /// Content type (code, doc, memory, pattern)
    pub type_: String,
    
    /// Source (file path, github url, memory_bank)
    pub source: String,
    
    /// Language (rust, julia, python, zig)
    pub language: Option<String>,
    
    /// Collections this belongs to
    pub collections: Vec<String>,
    
    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Timestamp indexed for recency
    pub timestamp: i64,
    
    /// Context7 semantic tags for search
    pub semantic_tags: Vec<String>,
    
    /// Indexed for filtering (motor, module, pattern)
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

/// ════════════════════════════════════════════════════════════════
/// MAPPERS: MEMORY_P → Qdrant
/// ════════════════════════════════════════════════════════════════

pub struct DocumentToQdrantMapper;

impl DocumentToQdrantMapper {
    /// Convert MEMORY_P Document → Qdrant Point
    pub fn map(doc: &Document, collection: &str) -> Result<QdrantPoint, String> {
        let vector = doc.vector.clone()
            .ok_or("Document must have vector embeddings")?;
        
        // Infer language from content/metadata
        let language = Self::infer_language(&doc.content, &doc.metadata);
        
        // Extract semantic tags
        let semantic_tags = Self::extract_semantic_tags(&doc.content, &language);
        
        // Build indexed fields
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
    
    /// Batch conversion
    pub fn map_batch(docs: &[Document], collection: &str) -> Vec<QdrantPoint> {
        docs.iter()
            .filter_map(|doc| Self::map(doc, collection).ok())
            .collect()
    }
    
    /// Infer programming language from content
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
    
    /// Extract semantic tags for Context7
    fn extract_semantic_tags(content: &str, language: &Option<String>) -> Vec<String> {
        let mut tags = vec![];
        
        // Language tag
        if let Some(lang) = language {
            tags.push(format!("lang:{}", lang));
        }
        
        // Domain tags
        if content.contains("search") || content.contains("query") {
            tags.push("search".to_string());
        }
        if content.contains("vector") || content.contains("embedding") {
            tags.push("vector".to_string());
        }
        if content.contains("async") || content.contains("await") {
            tags.push("async".to_string());
        }
        if content.contains("test") || content.contains("bench") {
            tags.push("test".to_string());
        }
        if content.contains("memory") || content.contains("cache") {
            tags.push("memory".to_string());
        }
        if content.contains("optimization") || content.contains("performance") {
            tags.push("optimization".to_string());
        }
        if content.contains("ffi") || content.contains("interop") {
            tags.push("ffi".to_string());
        }
        
        tags
    }
}

/// ════════════════════════════════════════════════════════════════
/// CONTEXT7 SEMANTIC SEARCH
/// ════════════════════════════════════════════════════════════════

/// Context7 semantic query builder
pub struct Context7Query {
    /// Natural language intent
    pub intent: String,
    
    /// Semantic tags to filter
    pub tags: Vec<String>,
    
    /// Collections to search
    pub collections: Vec<String>,
    
    /// Max results
    pub limit: usize,
    
    /// Query confidence threshold
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
    
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }
    
    pub fn with_collection(mut self, collection: &str) -> Self {
        self.collections = vec![collection.to_string()];
        self
    }
    
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
    
    /// Convert to Qdrant filter + search query
    pub fn to_qdrant_filter(&self) -> QdrantFilter {
        QdrantFilter {
            semantic_tags: self.tags.clone(),
            collections: self.collections.clone(),
            min_confidence: self.confidence,
            intent: self.intent.clone(),
        }
    }
}

/// Qdrant Filter (to apply with Context7 intent)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantFilter {
    pub semantic_tags: Vec<String>,
    pub collections: Vec<String>,
    pub min_confidence: f32,
    pub intent: String,
}

/// ════════════════════════════════════════════════════════════════
/// MCP CONTEXT7 SEMANTIC ROUTER
/// ════════════════════════════════════════════════════════════════

pub struct MpcContext7Router {
    /// Maps natural language intent → semantic tags + optimal motor
    pub intent_mappings: HashMap<String, SemanticRoute>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SemanticRoute {
    pub tags: Vec<String>,
    pub suggested_motors: Vec<String>,
    pub description: String,
}

impl MpcContext7Router {
    pub fn new() -> Self {
        let mut intent_mappings = HashMap::new();
        
        // Vector-specific queries
        intent_mappings.insert(
            "find_similar_vectors".to_string(),
            SemanticRoute {
                tags: vec!["vector".to_string(), "semantic".to_string()],
                suggested_motors: vec!["qdrant".to_string(), "faiss".to_string()],
                description: "Vector similarity search".to_string(),
            },
        );
        
        // Full-text queries
        intent_mappings.insert(
            "find_text_match".to_string(),
            SemanticRoute {
                tags: vec!["fulltext".to_string(), "text".to_string()],
                suggested_motors: vec!["tantivy".to_string(), "lnx".to_string()],
                description: "Full-text search".to_string(),
            },
        );
        
        // Code-specific queries
        intent_mappings.insert(
            "find_code_pattern".to_string(),
            SemanticRoute {
                tags: vec!["code".to_string(), "pattern".to_string()],
                suggested_motors: vec!["tantivy".to_string(), "memory_bank".to_string()],
                description: "Code pattern search".to_string(),
            },
        );
        
        // Optimization queries
        intent_mappings.insert(
            "find_optimization".to_string(),
            SemanticRoute {
                tags: vec!["optimization".to_string(), "performance".to_string()],
                suggested_motors: vec!["memory_bank".to_string(), "qdrant".to_string()],
                description: "Performance optimization search".to_string(),
            },
        );
        
        // FFI queries
        intent_mappings.insert(
            "find_ffi_example".to_string(),
            SemanticRoute {
                tags: vec!["ffi".to_string(), "interop".to_string()],
                suggested_motors: vec!["memory_bank".to_string()],
                description: "FFI integration examples".to_string(),
            },
        );
        
        Self { intent_mappings }
    }
    
    /// Route query based on semantic intent
    pub fn route(&self, intent: &str) -> Option<SemanticRoute> {
        self.intent_mappings.get(intent).cloned()
    }
    
    /// Get optimal motor for intent
    pub fn get_motor_for_intent(&self, intent: &str) -> Option<String> {
        self.route(intent)
            .and_then(|r| r.suggested_motors.first().cloned())
    }
}

impl Default for MpcContext7Router {
    fn default() -> Self {
        Self::new()
    }
}

/// ════════════════════════════════════════════════════════════════
/// GITHUB CODE SEARCH INTEGRATION (via MCP)
/// ════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCodeExample {
    pub url: String,
    pub file_path: String,
    pub language: String,
    pub code_snippet: String,
    pub relevance_score: f32,
    pub stars: u32,
}

pub struct MpcGithubSearcher;

impl MpcGithubSearcher {
    /// Build GitHub search query for code examples
    /// Uses Context7 semantics to find relevant patterns
    pub fn build_github_search(
        intent: &str,
        tags: &[String],
        language: Option<&str>,
    ) -> String {
        let mut query = String::new();
        
        // Intent keyword
        query.push_str(&format!("\"{}\"", intent));
        
        // Language filter
        if let Some(lang) = language {
            query.push_str(&format!(" language:{}", lang));
        }
        
        // Tags as code search keywords
        for tag in tags {
            query.push_str(&format!(" {}", tag));
        }
        
        // Restrict to popular repos (stars > 100)
        query.push_str(" stars:>100");
        
        query
    }
    
    /// Example MCP tool call to fetch GitHub examples
    pub fn example_mcp_call(search_query: &str) -> String {
        format!(
            r#"
mcp.tool_call("github_code_search", {{
    "query": "{}",
    "language": "rust",
    "per_page": 5,
    "sort": "stars"
}})
        "#,
            search_query
        )
    }
}

/// ════════════════════════════════════════════════════════════════
/// INTEGRATED WORKFLOW
/// ════════════════════════════════════════════════════════════════

pub struct QdrantContext7Workflow {
    router: MpcContext7Router,
}

impl QdrantContext7Workflow {
    pub fn new() -> Self {
        Self {
            router: MpcContext7Router::new(),
        }
    }
    
    /// Full workflow: Intent → Context7 → Qdrant + GitHub search
    pub fn process_query(
        &self,
        intent: &str,
        query_text: &str,
    ) -> WorkflowResult {
        // Step 1: Route using Context7
        let route = self.router.route(intent)
            .unwrap_or_else(|| SemanticRoute {
                tags: vec![],
                suggested_motors: vec!["qdrant".to_string()],
                description: "Generic search".to_string(),
            });
        
        // Step 2: Build Qdrant context filter
        let context7_query = Context7Query::new(query_text)
            .with_collection("default");
        
        let qdrant_filter = context7_query.to_qdrant_filter();
        
        // Step 3: Build GitHub search (for code examples)
        let github_search = if route.tags.contains(&"code".to_string()) {
            Some(MpcGithubSearcher::build_github_search(
                intent,
                &route.tags,
                Some("rust"),
            ))
        } else {
            None
        };
        
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
    fn default() -> Self {
        Self::new()
    }
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

/// ════════════════════════════════════════════════════════════════
/// TESTS
/// ════════════════════════════════════════════════════════════════

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
        assert_eq!(point.payload.type_, "document");
    }

    #[test]
    fn test_language_inference() {
        let rust_code = "fn main() { println!(\"hello\"); }".to_string();
        let lang = DocumentToQdrantMapper::infer_language(&rust_code, &HashMap::new());
        assert_eq!(lang, Some("rust".to_string()));
    }

    #[test]
    fn test_semantic_tags() {
        let content = "vector search optimization".to_string();
        let tags = DocumentToQdrantMapper::extract_semantic_tags(&content, &Some("rust".to_string()));
        assert!(tags.contains(&"lang:rust".to_string()));
        assert!(tags.contains(&"vector".to_string()));
        assert!(tags.contains(&"optimization".to_string()));
    }

    #[test]
    fn test_context7_router() {
        let router = MpcContext7Router::new();
        let route = router.route("find_similar_vectors");
        assert!(route.is_some());
        assert!(route.unwrap().tags.contains(&"vector".to_string()));
    }

    #[test]
    fn test_github_search_builder() {
        let query = MpcGithubSearcher::build_github_search(
            "vector_search",
            &vec!["optimization".to_string()],
            Some("rust"),
        );
        assert!(query.contains("vector_search"));
        assert!(query.contains("language:rust"));
        assert!(query.contains("optimization"));
    }

    #[test]
    fn test_workflow() {
        let workflow = QdrantContext7Workflow::new();
        let result = workflow.process_query(
            "find_similar_vectors",
            "Find documents similar to my query",
        );
        assert_eq!(result.intent, "find_similar_vectors");
        assert!(result.selected_motors.contains(&"qdrant".to_string()));
    }
}
