//! Core module for all search engines
//! 
//! Provides traits, types, and utilities shared across all 9 engines

pub mod health_monitor;
pub mod routing_ai;
pub mod traits;
pub mod types;

pub use health_monitor::HealthMonitor;
pub use routing_ai::RoutingAI;
pub use traits::*;
pub use types::*;
