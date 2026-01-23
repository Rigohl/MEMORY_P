//! Load balancer for distributing queries across engines

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct LoadBalancer {
    engine_loads: HashMap<String, AtomicUsize>,
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            engine_loads: HashMap::new(),
        }
    }

    pub fn select_engine(&self, candidates: &[String]) -> Option<String> {
        candidates
            .iter()
            .min_by_key(|name| {
                self.engine_loads
                    .get(*name)
                    .map(|load| load.load(Ordering::Relaxed))
                    .unwrap_or(0)
            })
            .cloned()
    }

    pub fn increment_load(&self, engine: &str) {
        if let Some(load) = self.engine_loads.get(engine) {
            load.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn decrement_load(&self, engine: &str) {
        if let Some(load) = self.engine_loads.get(engine) {
            load.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

impl Default for LoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}
