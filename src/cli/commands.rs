//! CLI Commands Definition
//!
//! Defines the command structure for the JAR CLI tool.

use clap::{Parser, Subcommand};

/// JAR - Intelligent DevOps CLI for MEMORY_P
///
/// Automated validation, SQL detection, and repair for CI/CD workflows
#[derive(Parser, Debug)]
#[command(name = "jar")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Validate project structure and detect errors
    Validate {
        /// Path to project directory
        #[arg(short, long, default_value = ".")]
        path: String,

        /// Check for dead code
        #[arg(long)]
        check_dead_code: bool,

        /// Scan for TODOs and FIXMEs
        #[arg(long)]
        scan_todos: bool,

        /// Validate MCP endpoints
        #[arg(long)]
        validate_mcp: bool,
    },

    /// Deep scan for SQL queries and schemas
    DetectSql {
        /// Path to scan
        #[arg(short, long, default_value = ".")]
        path: String,

        /// Check syntax of detected queries
        #[arg(long)]
        validate_syntax: bool,

        /// Detect common SQL issues (N+1, missing indexes)
        #[arg(long)]
        detect_issues: bool,
    },

    /// Auto-repair common issues
    Repair {
        /// Path to project
        #[arg(short, long, default_value = ".")]
        path: String,

        /// Fix Rust dependencies
        #[arg(long)]
        fix_deps: bool,

        /// Format code
        #[arg(long)]
        format: bool,

        /// Auto-fix clippy warnings
        #[arg(long)]
        fix_clippy: bool,

        /// Regenerate SQL schemas
        #[arg(long)]
        regen_schemas: bool,

        /// Dry run (show what would be fixed)
        #[arg(long)]
        dry_run: bool,
    },

    /// Check CI/CD workflow health
    CiCheck {
        /// Path to .github/workflows
        #[arg(short, long, default_value = ".github/workflows")]
        path: String,
    },
}
