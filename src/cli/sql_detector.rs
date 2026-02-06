//! SQL Detector Module
//! 
//! Deep scan for SQL queries and validation.

use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use sqlparser::ast::Statement;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use jwalk::WalkDir;

/// SQL detection results
#[derive(Debug, Default)]
pub struct SqlReport {
    pub queries: Vec<SqlQuery>,
    pub syntax_errors: Vec<SqlError>,
    pub issues: Vec<SqlIssue>,
}

#[derive(Debug, Clone)]
pub struct SqlQuery {
    pub file: PathBuf,
    pub line: usize,
    pub query: String,
    pub query_type: QueryType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
    Create,
    Drop,
    Alter,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct SqlError {
    pub file: PathBuf,
    pub line: usize,
    pub query: String,
    pub error: String,
}

#[derive(Debug, Clone)]
pub struct SqlIssue {
    pub file: PathBuf,
    pub line: usize,
    pub severity: Severity,
    pub issue_type: IssueType,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IssueType {
    PotentialN1,
    MissingIndex,
    NoWhereClause,
    SelectStar,
    LargeIn,
    NonParameterized,
}

impl SqlReport {
    pub fn print(&self) {
        println!("\n{}", "=== SQL Analysis Report ===".bold().blue());
        
        println!("\n{} SQL queries found: {}", "📊".cyan(), self.queries.len());
        
        // Group by type
        let mut by_type: HashMap<String, usize> = HashMap::new();
        for query in &self.queries {
            *by_type.entry(format!("{:?}", query.query_type)).or_insert(0) += 1;
        }
        
        for (qtype, count) in by_type.iter() {
            println!("  • {}: {}", qtype, count);
        }
        
        if !self.syntax_errors.is_empty() {
            println!("\n{} Syntax Errors:", "❌".red());
            for error in self.syntax_errors.iter().take(5) {
                println!("  {}:{}", error.file.display(), error.line);
                println!("    Query: {}", error.query.trim().chars().take(80).collect::<String>());
                println!("    Error: {}", error.error.red());
            }
            if self.syntax_errors.len() > 5 {
                println!("  ... and {} more", self.syntax_errors.len() - 5);
            }
        }
        
        if !self.issues.is_empty() {
            println!("\n{} Potential Issues:", "⚠️".yellow());
            
            let errors: Vec<_> = self.issues.iter().filter(|i| i.severity == Severity::Error).collect();
            let warnings: Vec<_> = self.issues.iter().filter(|i| i.severity == Severity::Warning).collect();
            
            if !errors.is_empty() {
                println!("  Errors ({}):", errors.len());
                for issue in errors.iter().take(3) {
                    println!("    {}:{} - {} ({})", 
                        issue.file.display(), 
                        issue.line,
                        issue.message,
                        format!("{:?}", issue.issue_type).red());
                }
            }
            
            if !warnings.is_empty() {
                println!("  Warnings ({}):", warnings.len());
                for issue in warnings.iter().take(3) {
                    println!("    {}:{} - {} ({})", 
                        issue.file.display(), 
                        issue.line,
                        issue.message,
                        format!("{:?}", issue.issue_type).yellow());
                }
            }
        }
        
        if self.syntax_errors.is_empty() && self.issues.is_empty() {
            println!("\n{} {}", "✅".green(), "No SQL issues detected!".green().bold());
        }
    }
}

/// Detect SQL queries in project
pub fn detect_sql(path: &str, validate_syntax: bool, detect_issues: bool) -> Result<SqlReport> {
    let mut report = SqlReport::default();
    let project_path = Path::new(path);
    
    println!("{} Scanning for SQL queries in: {}", "🔍".cyan(), path);
    
    // Patterns to detect SQL queries
    let sql_patterns = vec![
        // Rust string literals with SQL keywords
        Regex::new(r#"(?i)["'][\s]*(SELECT|INSERT|UPDATE|DELETE|CREATE|DROP|ALTER)[\s]+.*?["']"#).unwrap(),
        // sqlx macros
        Regex::new(r#"(?i)query!?\s*\(\s*["'](.+?)["']"#).unwrap(),
        // Raw SQL strings
        Regex::new(r"(?i)r#[\s]*(SELECT|INSERT|UPDATE|DELETE|CREATE|DROP|ALTER)[\s]+.*?#").unwrap(),
    ];
    
    for entry in WalkDir::new(project_path)
        .skip_hidden(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();
        
        // Only scan Rust files
        if entry_path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        
        if let Ok(content) = fs::read_to_string(&entry_path) {
            for (line_num, line) in content.lines().enumerate() {
                for pattern in &sql_patterns {
                    if let Some(caps) = pattern.captures(line) {
                        // Try to extract the query
                        let query_str = caps.get(1)
                            .or_else(|| caps.get(0))
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default();
                        
                        if query_str.len() < 10 {
                            continue; // Too short to be a real query
                        }
                        
                        let query_type = determine_query_type(&query_str);
                        
                        let query = SqlQuery {
                            file: entry_path.to_path_buf(),
                            line: line_num + 1,
                            query: query_str.clone(),
                            query_type,
                        };
                        
                        // Validate syntax if requested
                        if validate_syntax {
                            if let Err(e) = validate_sql_syntax(&query_str) {
                                report.syntax_errors.push(SqlError {
                                    file: entry_path.to_path_buf(),
                                    line: line_num + 1,
                                    query: query_str.clone(),
                                    error: e.to_string(),
                                });
                            }
                        }
                        
                        // Detect issues if requested
                        if detect_issues {
                            let issues = detect_sql_issues(&query);
                            report.issues.extend(issues);
                        }
                        
                        report.queries.push(query);
                    }
                }
            }
        }
    }
    
    Ok(report)
}

fn determine_query_type(query: &str) -> QueryType {
    let upper = query.to_uppercase();
    if upper.contains("SELECT") {
        QueryType::Select
    } else if upper.contains("INSERT") {
        QueryType::Insert
    } else if upper.contains("UPDATE") {
        QueryType::Update
    } else if upper.contains("DELETE") {
        QueryType::Delete
    } else if upper.contains("CREATE") {
        QueryType::Create
    } else if upper.contains("DROP") {
        QueryType::Drop
    } else if upper.contains("ALTER") {
        QueryType::Alter
    } else {
        QueryType::Unknown
    }
}

fn validate_sql_syntax(query: &str) -> Result<()> {
    let dialect = GenericDialect {};
    let _statements = Parser::parse_sql(&dialect, query)
        .context("Failed to parse SQL")?;
    Ok(())
}

fn detect_sql_issues(query: &SqlQuery) -> Vec<SqlIssue> {
    let mut issues = Vec::new();
    let upper = query.query.to_uppercase();
    
    // Check for SELECT *
    if upper.contains("SELECT *") || upper.contains("SELECT*") {
        issues.push(SqlIssue {
            file: query.file.clone(),
            line: query.line,
            severity: Severity::Warning,
            issue_type: IssueType::SelectStar,
            message: "SELECT * detected - consider specifying columns explicitly".to_string(),
        });
    }
    
    // Check for DELETE/UPDATE without WHERE
    if (upper.contains("DELETE FROM") || upper.contains("UPDATE ")) && !upper.contains("WHERE") {
        issues.push(SqlIssue {
            file: query.file.clone(),
            line: query.line,
            severity: Severity::Error,
            issue_type: IssueType::NoWhereClause,
            message: "DELETE/UPDATE without WHERE clause - could affect all rows!".to_string(),
        });
    }
    
    // Check for non-parameterized queries (simple heuristic)
    if query.query.contains("format!") || query.query.contains("&format") {
        issues.push(SqlIssue {
            file: query.file.clone(),
            line: query.line,
            severity: Severity::Warning,
            issue_type: IssueType::NonParameterized,
            message: "Potential SQL injection risk - use parameterized queries".to_string(),
        });
    }
    
    issues
}
