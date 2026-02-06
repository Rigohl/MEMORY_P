//! Auto-Repair Module
//! 
//! Automatic fixing of common issues.

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;
use std::process::{Command, Stdio};

/// Repair results
#[derive(Debug, Default)]
pub struct RepairReport {
    pub actions: Vec<RepairAction>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RepairAction {
    pub action: String,
    pub success: bool,
    pub output: String,
}

impl RepairReport {
    pub fn print(&self) {
        println!("\n{}", "=== Repair Report ===".bold().blue());
        
        let successful = self.actions.iter().filter(|a| a.success).count();
        let failed = self.actions.iter().filter(|a| !a.success).count();
        
        println!("\n{} Actions taken: {} successful, {} failed", 
            "🔧".cyan(), 
            successful.to_string().green(),
            failed.to_string().red());
        
        for action in &self.actions {
            let status = if action.success {
                "✅".green()
            } else {
                "❌".red()
            };
            println!("\n{} {}", status, action.action.bold());
            
            if !action.output.is_empty() {
                let lines: Vec<&str> = action.output.lines().take(5).collect();
                for line in lines {
                    println!("  {}", line);
                }
                if action.output.lines().count() > 5 {
                    println!("  ... ({} more lines)", action.output.lines().count() - 5);
                }
            }
        }
        
        if !self.errors.is_empty() {
            println!("\n{} Errors:", "❌".red());
            for error in &self.errors {
                println!("  • {}", error.red());
            }
        }
    }
}

/// Auto-repair project issues
pub fn repair_project(
    path: &str, 
    fix_deps: bool, 
    format: bool, 
    fix_clippy: bool,
    _regen_schemas: bool,
    dry_run: bool
) -> Result<RepairReport> {
    let mut report = RepairReport::default();
    let project_path = Path::new(path);
    
    println!("{} Starting auto-repair for: {}", "🔧".cyan(), path);
    if dry_run {
        println!("{} DRY RUN MODE - No changes will be made", "ℹ️".yellow());
    }
    
    // Fix dependencies
    if fix_deps {
        println!("\n  📦 Fixing Rust dependencies...");
        let result = if dry_run {
            RepairAction {
                action: "cargo update (dry-run)".to_string(),
                success: true,
                output: "Would run: cargo update".to_string(),
            }
        } else {
            run_cargo_update(project_path)?
        };
        report.actions.push(result);
    }
    
    // Format code
    if format {
        println!("\n  ✨ Formatting code...");
        let result = if dry_run {
            RepairAction {
                action: "cargo fmt (dry-run)".to_string(),
                success: true,
                output: "Would run: cargo fmt --all".to_string(),
            }
        } else {
            run_cargo_fmt(project_path)?
        };
        report.actions.push(result);
    }
    
    // Fix clippy warnings
    if fix_clippy {
        println!("\n  🔍 Fixing clippy warnings...");
        let result = if dry_run {
            RepairAction {
                action: "cargo clippy --fix (dry-run)".to_string(),
                success: true,
                output: "Would run: cargo clippy --fix --allow-dirty --allow-staged".to_string(),
            }
        } else {
            run_cargo_clippy_fix(project_path)?
        };
        report.actions.push(result);
    }
    
    Ok(report)
}

fn run_cargo_update(path: &Path) -> Result<RepairAction> {
    let output = Command::new("cargo")
        .arg("update")
        .current_dir(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to run cargo update")?;
    
    let success = output.status.success();
    let output_str = if success {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };
    
    Ok(RepairAction {
        action: "cargo update".to_string(),
        success,
        output: output_str,
    })
}

fn run_cargo_fmt(path: &Path) -> Result<RepairAction> {
    let output = Command::new("cargo")
        .arg("fmt")
        .arg("--all")
        .current_dir(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to run cargo fmt")?;
    
    let success = output.status.success();
    let output_str = if success {
        "Code formatted successfully".to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };
    
    Ok(RepairAction {
        action: "cargo fmt --all".to_string(),
        success,
        output: output_str,
    })
}

fn run_cargo_clippy_fix(path: &Path) -> Result<RepairAction> {
    let output = Command::new("cargo")
        .arg("clippy")
        .arg("--fix")
        .arg("--allow-dirty")
        .arg("--allow-staged")
        .current_dir(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to run cargo clippy --fix")?;
    
    let success = output.status.success();
    let output_str = String::from_utf8_lossy(&output.stdout).to_string() 
        + &String::from_utf8_lossy(&output.stderr).to_string();
    
    Ok(RepairAction {
        action: "cargo clippy --fix".to_string(),
        success,
        output: output_str,
    })
}
