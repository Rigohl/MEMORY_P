use super::types::{AgentId, ContextId, SharedContext};
use crate::error::Result;
use dashmap::DashMap;
use std::sync::Arc;

pub struct ContextManager {
    contexts: Arc<DashMap<String, SharedContext>>,
}

impl ContextManager {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            contexts: Arc::new(DashMap::new()),
        })
    }

    pub async fn get_or_create(&self, agent_id: AgentId) -> Result<SharedContext> {
        if let Some(existing) = self.contexts.get(&agent_id.0) {
            let mut context = existing.clone();
            context.touch();
            self.contexts.insert(agent_id.0.clone(), context.clone());
            return Ok(context);
        }

        let context = SharedContext::new(agent_id.clone());
        self.contexts.insert(agent_id.0.clone(), context.clone());
        Ok(context)
    }

    pub async fn update(&self, mut context: SharedContext) -> Result<()> {
        context.update();
        self.contexts.insert(context.agent_id.0.clone(), context);
        Ok(())
    }

    pub fn count(&self) -> usize {
        self.contexts.len()
    }

    pub async fn cleanup_inactive(&self, max_age_secs: u64) -> Result<usize> {
        let now = chrono::Utc::now().timestamp();
        let mut removed = 0_usize;
        let keys: Vec<String> = self
            .contexts
            .iter()
            .filter_map(|entry| {
                let age = now.saturating_sub(entry.metadata.last_accessed);
                if age > max_age_secs as i64 {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        for key in keys {
            if self.contexts.remove(&key).is_some() {
                removed += 1;
            }
        }

        Ok(removed)
    }

    pub async fn delete(&self, context_id: &ContextId) -> Result<()> {
        let key = self.contexts.iter().find_map(|entry| {
            if entry.context_id == *context_id {
                Some(entry.key().clone())
            } else {
                None
            }
        });

        if let Some(key) = key {
            self.contexts.remove(&key);
        }

        Ok(())
    }
}
