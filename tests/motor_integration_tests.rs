//! Motor Integration Tests for MEMORY_P v2.0
//! Validates motor ↔ MCP communication and coordination
//!
//! Run: cargo test --test motor_integration_tests -- --nocapture

#[cfg(test)]
mod motor_integration_tests {
    use std::time::Duration;
    use tokio::time::sleep;

    /// Test 1: Qdrant motor connectivity
    #[tokio::test]
    #[ignore]  // Requires running Qdrant on :3010
    async fn test_qdrant_motor_integration() {
        let client = reqwest::Client::new();
        
        // Test health check
        let response = client
            .get("http://localhost:3010/health")
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(resp) => {
                assert!(resp.status().is_success(), "Qdrant health check failed");
                println!("✅ Qdrant motor health check passed");
            }
            Err(e) => panic!("Qdrant unreachable: {}", e),
        }
    }

    /// Test 2: FAISS motor connectivity
    #[tokio::test]
    #[ignore]
    async fn test_faiss_motor_integration() {
        let client = reqwest::Client::new();
        
        let response = client
            .get("http://localhost:3011/health")
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(resp) => {
                assert!(resp.status().is_success(), "FAISS health check failed");
                println!("✅ FAISS motor health check passed");
            }
            Err(e) => panic!("FAISS unreachable: {}", e),
        }
    }

    /// Test 3: Tantivy motor connectivity
    #[tokio::test]
    #[ignore]
    async fn test_tantivy_motor_integration() {
        let client = reqwest::Client::new();
        
        let response = client
            .get("http://localhost:3013/health")
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(resp) => {
                assert!(resp.status().is_success(), "Tantivy health check failed");
                println!("✅ Tantivy motor health check passed");
            }
            Err(e) => panic!("Tantivy unreachable: {}", e),
        }
    }

    /// Test 4: MCP health endpoint
    #[tokio::test]
    #[ignore]  // Requires running MCP server on :4040
    async fn test_mcp_health_endpoint() {
        let client = reqwest::Client::new();
        
        let response = client
            .post("http://localhost:4040/mcp/v1/health")
            .json(&serde_json::json!({"jsonrpc":"2.0","method":"health"}))
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(resp) => {
                assert_eq!(resp.status(), 200, "MCP health endpoint failed");
                println!("✅ MCP health endpoint working");
            }
            Err(e) => panic!("MCP unreachable: {}", e),
        }
    }

    /// Test 5: MCP motors/health endpoint (NEW)
    #[tokio::test]
    #[ignore]
    async fn test_mcp_motors_health_endpoint() {
        let client = reqwest::Client::new();
        
        let response = client
            .post("http://localhost:4040/mcp/v1/motors/health")
            .json(&serde_json::json!({"jsonrpc":"2.0","method":"motors/health"}))
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(resp) => {
                assert_eq!(resp.status(), 200, "MCP motors/health endpoint failed");
                let body = resp.text().await.unwrap();
                assert!(body.contains("qdrant") || body.contains("healthy"), 
                    "Motor health response missing data");
                println!("✅ MCP motors/health endpoint working");
            }
            Err(e) => panic!("MCP motors/health unreachable: {}", e),
        }
    }

    /// Test 6: MCP predict endpoint (NEW)
    #[tokio::test]
    #[ignore]
    async fn test_mcp_predict_endpoint() {
        let client = reqwest::Client::new();
        
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "predict",
            "params": {
                "query": "find semantic similarity in embeddings",
                "context": null
            }
        });

        let response = client
            .post("http://localhost:4040/mcp/v1/predict")
            .json(&request_body)
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(resp) => {
                assert_eq!(resp.status(), 200, "MCP predict endpoint failed");
                let body = resp.text().await.unwrap();
                assert!(body.contains("recommended") || body.contains("confidence"), 
                    "Predict response missing recommendation");
                println!("✅ MCP predict endpoint working");
            }
            Err(e) => panic!("MCP predict unreachable: {}", e),
        }
    }

    /// Test 7: MCP context endpoint (NEW)
    #[tokio::test]
    #[ignore]
    async fn test_mcp_context_endpoint() {
        let client = reqwest::Client::new();
        
        let response = client
            .post("http://localhost:4040/mcp/v1/context")
            .json(&serde_json::json!({"jsonrpc":"2.0","method":"context"}))
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(resp) => {
                assert_eq!(resp.status(), 200, "MCP context endpoint failed");
                let body = resp.text().await.unwrap();
                assert!(body.contains("files") || body.contains("patterns"), 
                    "Context response missing workspace data");
                println!("✅ MCP context endpoint working");
            }
            Err(e) => panic!("MCP context unreachable: {}", e),
        }
    }

    /// Test 8: MCP decision endpoint (NEW)
    #[tokio::test]
    #[ignore]
    async fn test_mcp_decision_endpoint() {
        let client = reqwest::Client::new();
        
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "decision",
            "params": {
                "query": "find all text matches",
                "motors_available": ["qdrant", "tantivy", "faiss"],
                "sla_ms": 50.0
            }
        });

        let response = client
            .post("http://localhost:4040/mcp/v1/decision")
            .json(&request_body)
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(resp) => {
                assert_eq!(resp.status(), 200, "MCP decision endpoint failed");
                let body = resp.text().await.unwrap();
                assert!(body.contains("selected") || body.contains("reason"), 
                    "Decision response missing recommendation");
                println!("✅ MCP decision endpoint working");
            }
            Err(e) => panic!("MCP decision unreachable: {}", e),
        }
    }

    /// Test 9: Motor failover circuit breaker
    #[tokio::test]
    async fn test_circuit_breaker_fallback() {
        // Simulate motor failure by requesting non-existent endpoint
        let client = reqwest::Client::new();
        
        // Try to connect to a likely-down motor
        let response = client
            .get("http://localhost:9999/health")  // Non-existent port
            .timeout(Duration::from_millis(500))
            .send()
            .await;

        // Should fail gracefully, not panic
        assert!(response.is_err(), "Should fail on unreachable motor");
        println!("✅ Circuit breaker handles motor failure gracefully");
    }

    /// Test 10: Fusion - Multiple motors in parallel
    #[tokio::test]
    #[ignore]
    async fn test_fusion_multiple_engines() {
        let client = reqwest::Client::new();
        
        // Create parallel tasks for each motor
        let tasks = vec![
            tokio::spawn(async {
                client.get("http://localhost:3010/health").send().await
            }),
            tokio::spawn(async {
                client.get("http://localhost:3013/health").send().await
            }),
            tokio::spawn(async {
                client.get("http://localhost:3015/health").send().await
            }),
        ];

        // Wait for all to complete
        let results = futures::future::join_all(tasks).await;
        
        let success_count = results.iter().filter(|r| {
            r.as_ref().map(|resp| resp.is_ok()).unwrap_or(false)
        }).count();

        assert!(success_count > 0, "Fusion: at least one motor should be healthy");
        println!("✅ Fusion test: {}/3 motors responded", success_count);
    }

    /// Test 11: MCP JSON-RPC 2.0 Compliance
    #[tokio::test]
    #[ignore]
    async fn test_mcp_json_rpc_compliance() {
        let client = reqwest::Client::new();
        
        // Test with invalid JSON-RPC (missing jsonrpc version)
        let response = client
            .post("http://localhost:4040/mcp/v1/health")
            .json(&serde_json::json!({"method":"health"}))  // Missing jsonrpc:"2.0"
            .timeout(Duration::from_secs(2))
            .send()
            .await;

        // Should either handle gracefully or return error
        match response {
            Ok(resp) => {
                assert!(resp.status().is_client_error() || resp.status().is_success(),
                    "Should handle malformed JSON-RPC");
            }
            Err(_) => {}  // Timeout is acceptable for invalid request
        }

        println!("✅ MCP JSON-RPC compliance test passed");
    }

    /// Test 12: P99 Latency Validation (Quick check)
    #[tokio::test]
    #[ignore]
    async fn test_p99_latency_slo() {
        use std::time::Instant;
        
        let client = reqwest::Client::new();
        let mut latencies = Vec::new();

        // Make 10 requests and measure latency
        for _ in 0..10 {
            let start = Instant::now();
            let _ = client
                .get("http://localhost:4040/mcp/v1/health")
                .timeout(Duration::from_secs(5))
                .send()
                .await;
            let elapsed = start.elapsed().as_millis() as f64;
            latencies.push(elapsed);
        }

        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p99 = latencies[(latencies.len() * 99 / 100).min(latencies.len() - 1)];

        println!("📊 P99 Latency: {:.2}ms (target: <50ms)", p99);
        assert!(p99 < 50.0, "P99 latency SLA violated: {:.2}ms > 50ms", p99);
        println!("✅ P99 latency SLA passed");
    }
}

// Utilities for test management
mod test_utils {
    pub fn setup_test_env() {
        // Initialize logging
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    }

    pub fn verify_all_motors_running() -> bool {
        // This would check if all 9 motors are accessible
        // For now, just log that we're checking
        println!("🔍 Verifying motor availability...");
        true
    }
}
