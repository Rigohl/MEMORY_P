//! CLI Module - Intelligent DevOps CLI for MEMORY_P
//!
//! Provides commands for validation, SQL detection, and auto-repair.

pub mod auto_repair;
pub mod commands;
pub mod sql_detector;
pub mod validators;

pub use commands::{Cli, Commands};
