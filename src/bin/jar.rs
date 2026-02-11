//! JAR - Intelligent DevOps CLI for MEMORY_P
//!
//! Main binary entry point for the JAR CLI tool.

use clap::Parser;
use colored::Colorize;
use std::process;

// Import from library
use memory_p::cli::{auto_repair, sql_detector, validators};
use memory_p::cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    // Print banner
    print_banner();

    if cli.verbose {
        println!("{} Verbose mode enabled", "ℹ️".cyan());
    }

    let result = match cli.command {
        Commands::Validate {
            path,
            check_dead_code,
            scan_todos,
            validate_mcp,
        } => handle_validate(&path, check_dead_code, scan_todos, validate_mcp),
        Commands::DetectSql {
            path,
            validate_syntax,
            detect_issues,
        } => handle_detect_sql(&path, validate_syntax, detect_issues),
        Commands::Repair {
            path,
            fix_deps,
            format,
            fix_clippy,
            regen_schemas,
            dry_run,
        } => handle_repair(&path, fix_deps, format, fix_clippy, regen_schemas, dry_run),
        Commands::CiCheck { path } => handle_ci_check(&path),
    };

    match result {
        Ok(code) => process::exit(code),
        Err(e) => {
            eprintln!("\n{} Error: {}", "❌".red(), e);
            process::exit(1);
        }
    }
}

fn print_banner() {
    println!(
        "{}",
        "╔══════════════════════════════════════════════════╗".bright_blue()
    );
    println!(
        "{}",
        "║      JAR - Intelligent DevOps CLI v0.1.0        ║".bright_blue()
    );
    println!(
        "{}",
        "║           For MEMORY_P Project                   ║".bright_blue()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════╝".bright_blue()
    );
}

fn handle_validate(
    path: &str,
    check_dead_code: bool,
    scan_todos: bool,
    validate_mcp: bool,
) -> Result<i32, anyhow::Error> {
    let report = validators::validate_project(path, check_dead_code, scan_todos, validate_mcp)?;
    report.print();

    Ok(if report.has_errors() { 1 } else { 0 })
}

fn handle_detect_sql(
    path: &str,
    validate_syntax: bool,
    detect_issues: bool,
) -> Result<i32, anyhow::Error> {
    let report = sql_detector::detect_sql(path, validate_syntax, detect_issues)?;
    report.print();

    Ok(if !report.syntax_errors.is_empty() {
        1
    } else {
        0
    })
}

fn handle_repair(
    path: &str,
    fix_deps: bool,
    format: bool,
    fix_clippy: bool,
    regen_schemas: bool,
    dry_run: bool,
) -> Result<i32, anyhow::Error> {
    // If no options specified, enable common ones
    let (fix_deps, format, fix_clippy) = if !fix_deps && !format && !fix_clippy && !regen_schemas {
        println!(
            "{} No specific repair actions specified, enabling: format + fix_deps",
            "ℹ️".yellow()
        );
        (true, true, false)
    } else {
        (fix_deps, format, fix_clippy)
    };

    let report =
        auto_repair::repair_project(path, fix_deps, format, fix_clippy, regen_schemas, dry_run)?;
    report.print();

    Ok(if !report.errors.is_empty() { 1 } else { 0 })
}

fn handle_ci_check(path: &str) -> Result<i32, anyhow::Error> {
    println!("{} Checking CI/CD workflows at: {}", "🔍".cyan(), path);

    let workflow_path = std::path::Path::new(path);

    if !workflow_path.exists() {
        println!(
            "\n{} Workflows directory not found at {}",
            "⚠️".yellow(),
            path
        );
        println!("  Consider creating GitHub Actions workflows for:");
        println!("  • Continuous Integration (tests, builds)");
        println!("  • SQL validation on push");
        println!("  • Auto-repair on PRs");
        return Ok(1);
    }

    // Count workflow files
    let mut count = 0;
    if workflow_path.is_dir() {
        for entry in std::fs::read_dir(workflow_path)? {
            let entry = entry?;
            if let Some(ext) = entry.path().extension() {
                if ext == "yml" || ext == "yaml" {
                    count += 1;
                    println!("  ✓ Found: {}", entry.path().display());
                }
            }
        }
    }

    println!("\n{} Found {} workflow file(s)", "📊".cyan(), count);

    if count == 0 {
        println!("\n{} No workflows found. Consider adding:", "💡".yellow());
        println!("  • ci.yml - Build, test, and validate");
        println!("  • auto-repair.yml - Automatic issue fixing");
        println!("  • sql-check.yml - SQL validation");
    }

    Ok(0)
}
