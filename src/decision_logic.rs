use crate::error::{MemoryPError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRationale {
    pub situation: String,
    pub recommended_action: String,
    pub confidence: f64,
    pub factors: Vec<String>,
}

pub struct DecisionEngine;

impl DecisionEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn analyze_decision(
        &self,
        situation: &str,
        context_data: &HashMap<String, String>,
    ) -> Result<DecisionRationale> {
        if situation.trim().is_empty() {
            return Err(MemoryPError::InvalidParams(
                "Decision analysis requires a non-empty situation".into(),
            ));
        }

        let factors: Vec<String> = context_data
            .iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect();
        let confidence = if factors.is_empty() { 0.62 } else { 0.78 };
        let recommended_action = if situation.contains("error") || situation.contains("fail") {
            "stabilize_and_retry"
        } else if factors
            .iter()
            .any(|entry| entry.contains("chaos") || entry.contains("entropy"))
        {
            "analyze_then_parallelize"
        } else {
            "continue_with_guardrails"
        };

        Ok(DecisionRationale {
            situation: situation.to_string(),
            recommended_action: recommended_action.to_string(),
            confidence,
            factors,
        })
    }
}
