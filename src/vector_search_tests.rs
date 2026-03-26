//! Integration tests for Vector Search
//!
//! Para ejecutar: cargo test --package memory_p --lib vector_search_integration

#[cfg(test)]
mod vector_search_integration {
    use crate::ffi::jax::{EmbeddingConfig, EmbeddingGenerator};
    use crate::motores::vector_search::{
        AdvancedVectorEngine, DistanceMetric, HnswConfig, VectorDocument, VectorFilter,
    };
    use serde_json::json;

    #[tokio::test]
    async fn test_complete_workflow() {
        // 1. Inicializar motor vectorial
        let config = HnswConfig::default().with_dimension(384);
        let engine = AdvancedVectorEngine::new(config);

        // 2. Inicializar generador de embeddings
        let emb_config = EmbeddingConfig::default();
        let generator = EmbeddingGenerator::new(emb_config);

        // 3. Preparar documentos
        let docs_text = vec![
            ("doc1", "Rust programming language for systems development"),
            ("doc2", "Python is great for machine learning and AI"),
            ("doc3", "JavaScript for web development and Node.js"),
            ("doc4", "Go language for concurrent programming"),
            ("doc5", "C++ for high performance computing"),
        ];

        // 4. Generar embeddings
        let texts: Vec<String> = docs_text.iter().map(|(_, text)| text.to_string()).collect();
        let embeddings = generator
            .generate_embeddings_batch(&texts)
            .await
            .expect("Failed to generate embeddings");

        // 5. Indexar documentos
        let vector_docs: Vec<VectorDocument> = docs_text
            .iter()
            .zip(embeddings.iter())
            .map(|((id, text), embedding)| {
                VectorDocument::new(
                    id.to_string(),
                    embedding.clone(),
                    json!({
                        "text": text,
                        "category": if text.contains("machine learning") || text.contains("AI") {
                            "ml"
                        } else {
                            "programming"
                        }
                    }),
                )
            })
            .collect();

        let indexed = engine
            .index_batch(vector_docs)
            .await
            .expect("Failed to index documents");
        assert_eq!(indexed, 5, "All documents should be indexed");

        // 6. Búsqueda básica
        let query_text = "programming languages for software";
        let query_embedding = generator
            .generate_embedding(query_text)
            .await
            .expect("Failed to generate query embedding");

        let results = engine
            .search(&query_embedding, 3, None)
            .await
            .expect("Search failed");

        assert!(!results.is_empty(), "Should find results");
        assert!(results.len() <= 3, "Should respect limit");

        // 7. Búsqueda con filtros
        let mut filter = VectorFilter::new();
        filter.must = Some(serde_json::Map::from_iter(vec![(
            "category".to_string(),
            json!("programming"),
        )]));

        let filtered_results = engine
            .search(&query_embedding, 5, Some(filter))
            .await
            .expect("Filtered search failed");

        assert!(!filtered_results.is_empty(), "Should find filtered results");
        for result in &filtered_results {
            assert_eq!(
                result.metadata.get("category").unwrap().as_str().unwrap(),
                "programming"
            );
        }

        // 8. Verificar estadísticas
        let stats = engine.get_stats();
        assert_eq!(stats.total_documents, 5);
        assert!(stats.total_queries > 0);

        println!("✅ Complete workflow test passed!");
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let config = HnswConfig::default();
        let engine = AdvancedVectorEngine::new(config);
        let emb_gen = EmbeddingGenerator::new(EmbeddingConfig::default());

        // Batch indexing
        let texts: Vec<String> = (0..100)
            .map(|i| format!("Document number {} with some content", i))
            .collect();

        let embeddings = emb_gen
            .generate_embeddings_batch(&texts)
            .await
            .expect("Batch embedding failed");

        let docs: Vec<VectorDocument> = texts
            .iter()
            .zip(embeddings.iter())
            .enumerate()
            .map(|(i, (text, emb))| {
                VectorDocument::new(
                    format!("doc{}", i),
                    emb.clone(),
                    json!({"index": i, "text": text}),
                )
            })
            .collect();

        let indexed = engine.index_batch(docs).await.expect("Batch index failed");
        assert_eq!(indexed, 100, "All documents should be indexed");

        println!("✅ Batch operations test passed!");
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        use std::sync::Arc;
        use tokio::task;

        let config = HnswConfig::default();
        let engine = Arc::new(AdvancedVectorEngine::new(config));

        // Spawn multiple tasks concurrently
        let mut handles = vec![];

        for i in 0..10 {
            let engine_clone = Arc::clone(&engine);
            let handle = task::spawn(async move {
                let doc = VectorDocument::new(
                    format!("concurrent_doc_{}", i),
                    vec![i as f32 / 10.0; 384],
                    json!({"index": i}),
                );
                engine_clone.index_document(doc).await.unwrap();
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all documents were indexed
        let stats = engine.get_stats();
        assert_eq!(stats.total_documents, 10);

        println!("✅ Concurrent operations test passed!");
    }
}
