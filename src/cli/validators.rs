//! Validators Module
//! 
//! Structural validation, dead code detection, and TODO/FIXME scanning.

use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use jwalk::WalkDir;

/// Validation results
#[derive(Debug, Default)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub todos: Vec<TodoItem>,
    pub dead_code_suspects: Vec<String>,
    pub mcp_issues: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TodoItem {
    pub file: PathBuf,
    pub line: usize,
    pub kind: TodoKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TodoKind {
    Todo,
    Fixme,
    Hack,
    Xxx,
    Note,
}

impl ValidationReport {
    pub fn print(&self) {
        println!("\n{}", "=== Validation Report ===".bold().blue());
        
        if !self.errors.is_empty() {
            println!("\n{} Errors:", "❌".red());
            for error in &self.errors {
                println!("  • {}", error.red());
            }
        }
        
        if !self.warnings.is_empty() {
            println!("\n{} Warnings:", "⚠️".yellow());
            for warning in &self.warnings {
                println!("  • {}", warning.yellow());
            }
        }
        
        if !self.todos.is_empty() {
            println!("\n{} TODOs/FIXMEs found:", "📝".cyan());
            let mut by_kind: HashMap<String, Vec<&TodoItem>> = HashMap::new();
            for todo in &self.todos {
                by_kind.entry(format!("{:?}", todo.kind))
                    .or_default()
                    .push(todo);
            }
            
            for (kind, items) in by_kind.iter() {
                println!("  {} ({}):", kind, items.len());
                for item in items.iter().take(5) {
                    println!("    {}:{} - {}", 
                        item.file.display(), 
                        item.line, 
                        item.message.trim());
                }
                if items.len() > 5 {
                    println!("    ... and {} more", items.len() - 5);
                }
            }
        }
        
        if !self.dead_code_suspects.is_empty() {
            println!("\n{} Potential dead code:", "🔍".magenta());
            for suspect in self.dead_code_suspects.iter().take(10) {
                println!("  • {}", suspect);
            }
            if self.dead_code_suspects.len() > 10 {
                println!("  ... and {} more", self.dead_code_suspects.len() - 10);
            }
        }
        
        if !self.mcp_issues.is_empty() {
            println!("\n{} MCP Issues:", "🔌".yellow());
            for issue in &self.mcp_issues {
                println!("  • {}", issue.yellow());
            }
        }
        
        if self.errors.is_empty() && self.warnings.is_empty() {
            println!("\n{} {}", "✅".green(), "Validation passed!".green().bold());
        }
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Validate project structure
pub fn validate_project(path: &str, check_dead_code: bool, scan_todos: bool, validate_mcp: bool) -> Result<ValidationReport> {
    let mut report = ValidationReport::default();
    let project_path = Path::new(path);
    
    println!("{} Validating project at: {}", "🔍".cyan(), path);
    
    // Check Cargo.toml exists
    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        report.errors.push(format!("Cargo.toml not found at {}", cargo_toml.display()));
        return Ok(report);
    }
    
    // Parse Cargo.toml
    let cargo_content = fs::read_to_string(&cargo_toml)
        .context("Failed to read Cargo.toml")?;
    
    // Check for package name
    if !cargo_content.contains("[package]") {
        report.errors.push("Cargo.toml missing [package] section".to_string());
    }
    
    // Validate src directory
    let src_dir = project_path.join("src");
    if !src_dir.exists() {
        report.errors.push(format!("src/ directory not found at {}", src_dir.display()));
    } else {
        // Check for main.rs or lib.rs
        let has_main = src_dir.join("main.rs").exists();
        let has_lib = src_dir.join("lib.rs").exists();
        
        if !has_main && !has_lib {
            report.warnings.push("Neither main.rs nor lib.rs found in src/".to_string());
        }
    }
    
    // Scan for TODOs if requested
    if scan_todos {
        println!("  📝 Scanning for TODOs/FIXMEs...");
        report.todos = scan_todos_in_project(project_path)?;
    }
    
    // Check for dead code if requested
    if check_dead_code {
        println!("  🔍 Checking for potential dead code...");
        report.dead_code_suspects = check_dead_code_patterns(project_path)?;
    }
    
    // Validate MCP structure if requested
    if validate_mcp {
        println!("  🔌 Validating MCP structure...");
        validate_mcp_structure(project_path, &mut report)?;
    }
    
    Ok(report)
}

/// Scan for TODO, FIXME, HACK, XXX, NOTE comments
fn scan_todos_in_project(path: &Path) -> Result<Vec<TodoItem>> {
    let mut todos = Vec::new();
    
    let todo_re = Regex::new(r"(?i)(TODO|FIXME|HACK|XXX|NOTE)[\s:]*(.*)").unwrap();
    
    for entry in WalkDir::new(path)
        .skip_hidden(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // Only scan source files
        if let Some(ext) = path.extension() {
            if !matches!(ext.to_str(), Some("rs" | "toml" | "md" | "yml" | "yaml" | "json")) {
                continue;
            }
        } else {
            continue;
        }
        
        if let Ok(content) = fs::read_to_string(&path) {
            for (line_num, line) in content.lines().enumerate() {
                if let Some(caps) = todo_re.captures(line) {
                    let kind = match caps.get(1).unwrap().as_str().to_uppercase().as_str() {
                        "TODO" => TodoKind::Todo,
                        "FIXME" => TodoKind::Fixme,
                        "HACK" => TodoKind::Hack,
                        "XXX" => TodoKind::Xxx,
                        "NOTE" => TodoKind::Note,
                        _ => TodoKind::Todo,
                    };
                    
                    let message = caps.get(2)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default();
                    
                    todos.push(TodoItem {
                        file: path.to_path_buf(),
                        line: line_num + 1,
                        kind,
                        message,
                    });
                }
            }
        }
    }
    
    Ok(todos)
}

/// Check for potential dead code patterns
fn check_dead_code_patterns(path: &Path) -> Result<Vec<String>> {
    let mut suspects = Vec::new();
    
    // Look for unused function patterns (simplified heuristic)
    let _unused_fn_re = Regex::new(r"(?m)^[\s]*(?:pub\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(").unwrap();
    
    for entry in WalkDir::new(path)
        .skip_hidden(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();
        
        if entry_path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        
        if let Ok(content) = fs::read_to_string(&entry_path) {
            // Look for #[allow(dead_code)] or #[cfg(test)]
            if content.contains("#[allow(dead_code)]") {
                suspects.push(format!("{}: contains #[allow(dead_code)]", entry_path.display()));
            }
        }
    }
    
    Ok(suspects)
}

/// Validate MCP structure (endpoints, schemas)
fn validate_mcp_structure(path: &Path, report: &mut ValidationReport) -> Result<()> {
    // Check for MCP-related files
    let has_mcp_api = path.join("src/mcp_api.rs").exists() || 
                      path.join("src/mcp.rs").exists() ||
                      path.join("src/mcp").is_dir();
    
    if !has_mcp_api {
        report.warnings.push("No MCP API implementation found (src/mcp_api.rs or src/mcp/)".to_string());
    }
    
    // Check for required dependencies
    let cargo_toml = path.join("Cargo.toml");
    if cargo_toml.exists() {
        let content = fs::read_to_string(&cargo_toml)?;
        
        if !content.contains("mcp-sdk") && !content.contains("mcpkit") {
            report.mcp_issues.push("No MCP SDK dependency found in Cargo.toml".to_string());
        }
        
        if !content.contains("axum") && !content.contains("actix-web") {
            report.mcp_issues.push("No web framework found (axum/actix-web)".to_string());
        }
    }
    
    Ok(())
}
