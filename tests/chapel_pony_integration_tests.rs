#[cfg(test)]
mod chapel_pony_integration_tests {
    use super::super::*;
    use rayon::prelude::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_chapel_search_basic() {
        let req = MultiLanguageSearchRequest {
            query: "test search".to_string(),
            search_type: SearchType::FullText,
            limit: Some(10),
            timeout_ms: Some(250),
            metadata: None,
        };

        let (results, parallelism) = execute_chapel_search(&req).await;

        assert!(!results.is_empty(), "Chapel should return results");
        assert!(parallelism > 0, "Chapel should report parallelism level");
        assert!(
            results.iter().all(|r| r.score >= 0.0 && r.score <= 1.0),
            "Scores must be in [0,1]"
        );
    }

    #[tokio::test]
    async fn test_pony_search_basic() {
        let req = MultiLanguageSearchRequest {
            query: "actor isolation test".to_string(),
            search_type: SearchType::Hybrid,
            limit: Some(20),
            timeout_ms: Some(300),
            metadata: None,
        };

        let (results, actor_count) = execute_pony_search(&req).await;

        assert!(!results.is_empty(), "Pony should return results");
        assert!(actor_count > 0, "Pony should report actor count");
        assert!(
            results.len() <= 20,
            "Results should respect limit"
        );
    }

    #[tokio::test]
    async fn test_chapel_sla_compliance() {
        let test_cases = vec![100, 250, 500, 1000];

        for limit in test_cases.par_iter() {
            let req = MultiLanguageSearchRequest {
                query: "benchmark search".to_string(),
                search_type: SearchType::SemanticVector,
                limit: Some(*limit),
                timeout_ms: Some(250),
                metadata: None,
            };

            let start = Instant::now();
            let (results, _) = execute_chapel_search(&req).await;
            let elapsed = start.elapsed().as_millis();

            assert!(
                elapsed < 250,
                "Chapel SLA violated: {} ms > 250 ms for {} results",
                elapsed,
                results.len()
            );
        }
    }

    #[tokio::test]
    async fn test_pony_concurrent_searches() {
        let queries = vec![
            "search 1".to_string(),
            "search 2".to_string(),
            "search 3".to_string(),
            "search 4".to_string(),
        ];

        let start = Instant::now();

        let handles: Vec<_> = queries
            .into_par_iter()
            .map(|query| {
                tokio::spawn(async move {
                    let req = MultiLanguageSearchRequest {
                        query,
                        search_type: SearchType::FullText,
                        limit: Some(10),
                        timeout_ms: Some(300),
                        metadata: None,
                    };
                    execute_pony_search(&req).await
                })
            })
            .collect();

        for handle in handles {
            let (results, _) = handle.await.unwrap();
            assert!(!results.is_empty(), "Concurrent search should succeed");
        }

        let total_time = start.elapsed().as_millis();
        assert!(
            total_time < 300,
            "Concurrent Pony searches should complete under SLA"
        );
    }

    #[tokio::test]
    async fn test_hybrid_multi_language_aggregation() {
        let req = MultiLanguageSearchRequest {
            query: "multi-language hybrid".to_string(),
            search_type: SearchType::Hybrid,
            limit: Some(25),
            timeout_ms: Some(250),
            metadata: Some(serde_json::json!({"source": "test"})),
        };

        let start = Instant::now();
        let (results, final_engine, sla_met) =
            match execute_chapel_search(&req).await {
                (r, _) if start.elapsed().as_millis() < 250 => (r, "chapel", true),
                _ => {
                    let (r, _) = execute_pony_search(&req).await;
                    (r, "pony", false)
                }
            };

        assert!(
            !results.is_empty(),
            "Hybrid search should return results from {}",
            final_engine
        );
        println!(
            "Hybrid aggregation completed in {}ms using {}",
            start.elapsed().as_millis(),
            final_engine
        );
    }

    #[tokio::test]
    async fn test_search_result_ranking() {
        let req = MultiLanguageSearchRequest {
            query: "ranking test".to_string(),
            search_type: SearchType::SemanticVector,
            limit: Some(10),
            timeout_ms: Some(250),
            metadata: None,
        };

        let (results, _) = execute_chapel_search(&req).await;

        // Verify ranking is monotonic (scores descending)
        for i in 0..(results.len() - 1) {
            assert!(
                results[i].score >= results[i + 1].score,
                "Results must be ranked by score (descending)"
            );
        }
    }

    #[tokio::test]
    async fn test_search_timeout_handling() {
        let req = MultiLanguageSearchRequest {
            query: "timeout test".to_string(),
            search_type: SearchType::FullText,
            limit: Some(1000),
            timeout_ms: Some(10), // Very short timeout
            metadata: None,
        };

        let start = Instant::now();
        let _ = execute_chapel_search(&req).await;
        let elapsed = start.elapsed().as_millis();

        println!(
            "Execution completed in {} ms (requested timeout: 10 ms)",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_parallel_engine_consistency() {
        let query = "consistency test".to_string();
        let runs = 5;

        let mut all_results = Vec::new();

        for _ in 0..runs {
            let req = MultiLanguageSearchRequest {
                query: query.clone(),
                search_type: SearchType::FullText,
                limit: Some(5),
                timeout_ms: Some(250),
                metadata: None,
            };

            let (results, _) = execute_chapel_search(&req).await;
            all_results.push(results);
        }

        // All runs should return same number of results
        assert!(
            all_results.iter().all(|r| r.len() == all_results[0].len()),
            "All runs should return consistent result count"
        );
    }

    #[tokio::test]
    async fn test_metadata_serialization() {
        let metadata = SearchMetadata {
            timestamp: chrono::Utc::now().timestamp(),
            parallelism_level: 8,
            engine_version: "2.0.0".to_string(),
            sla_target_ms: 250,
            sla_met: true,
        };

        let serialized = serde_json::to_string(&metadata).unwrap();
        let deserialized: SearchMetadata = serde_json::from_str(&serialized).unwrap();

        assert_eq!(metadata.parallelism_level, deserialized.parallelism_level);
        assert_eq!(metadata.sla_target_ms, deserialized.sla_target_ms);
    }
}
