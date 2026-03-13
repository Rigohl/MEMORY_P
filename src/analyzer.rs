//! src/analyzer.rs - Code analysis module for MEMORY_P v2.0
//!
//! Analyzes code files for metrics (LOC, complexity), security issues, and patterns.
//! Used by MCP handlers and parallel engine for comprehensive code inspection.
//! 
//! Features:
//! - Line counting (LOC, blank, comments)
//! - Cyclomatic complexity estimation
//! - Function/struct/import detection
//! - Security vulnerability scanning (unsafe, unwrap, shell commands)
//! - Pattern detection (async, serde, tokio)
//! - Recursive file scanning with filtering

use crate::error::{MemoryPError, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Analysis results for a single file
#[derive(Debug, Clone)]
pub struct FileAnalysis {
	/// File path that was analyzed
	pub file_path: String,
	/// Total lines of code (excluding comments and blanks)
	pub lines_of_code: usize,
	/// Lines with actual code
	pub lines_with_code: usize,
	/// Blank lines
	pub blank_lines: usize,
	/// Comment lines (doc comments, multiline comments)
	pub comment_lines: usize,
	/// Number of functions/methods detected
	pub functions: usize,
	/// Number of struct/trait definitions
	pub structs: usize,
	/// Number of imports
	pub imports: usize,
	/// Estimated cyclomatic complexity (1.0 = simple)
	pub complexity_estimate: f64,
	/// Security score (0-100) based on security analysis
	pub security_score: f64,
	/// Security/style warnings detected (can be repeated)
	pub warnings: Vec<String>,
}

/// Code analyzer for static analysis of source files
pub struct CodeAnalyzer;

impl CodeAnalyzer {
	/// Scan directory for files with specific extension
	///
	/// # Arguments
	/// - `path`: Directory path to scan (or file path)
	/// - `ext`: File extension filter (e.g., "rs" for Rust files)
	/// - `recursive`: Whether to scan subdirectories recursively
	/// - `_follow_links`: Whether to follow symbolic links (ignored for now)
	///
	/// # Returns
	/// Vector of PathBuf for matching files, sorted
	///
	/// # Errors
	/// Returns error if path doesn't exist or can't be read
	pub fn scan_files(
		path: &str,
		ext: &str,
		recursive: bool,
		_follow_links: bool,
	) -> Result<Vec<PathBuf>> {
		let root_path = Path::new(path);
		
		if !root_path.exists() {
			return Err(MemoryPError::Other(format!("Path does not exist: {}", path)));
		}

		let ext_lower = ext.trim_start_matches('.').to_ascii_lowercase();
		let mut results = Vec::new();

		// Single file case
		if root_path.is_file() {
			if let Some(file_ext) = root_path.extension() {
				let ext_str = file_ext.to_str().unwrap_or("");
				if ext_str.eq_ignore_ascii_case(&ext_lower) {
					results.push(root_path.to_path_buf());
				}
			}
			return Ok(results);
		}

		// Directory scanning
		if recursive {
			for entry_result in WalkDir::new(root_path) {
				if let Ok(entry) = entry_result {
					if entry.file_type().is_file() {
						if let Some(file_ext) = entry.path().extension() {
							let ext_str = file_ext.to_str().unwrap_or("");
							if ext_str.eq_ignore_ascii_case(&ext_lower) {
								// Skip hidden and cache dirs
								let skip = if let Some(parent) = entry.path().parent() {
									if let Some(parent_name) = parent.file_name() {
										let pname = parent_name.to_str().unwrap_or("");
										pname.starts_with('.') || pname == "target" || pname == "node_modules"
									} else {
										false
									}
								} else {
									false
								};

								if !skip {
									results.push(entry.path().to_path_buf());
								}
							}
						}
					}
				}
			}
		} else {
			// Non-recursive: only immediate children
			if let Ok(entries) = fs::read_dir(root_path) {
				for entry_result in entries {
					if let Ok(entry) = entry_result {
						let path = entry.path();
						if path.is_file() {
							if let Some(file_ext) = path.extension() {
								let ext_str = file_ext.to_str().unwrap_or("");
								if ext_str.eq_ignore_ascii_case(&ext_lower) {
									results.push(path);
								}
							}
						}
					}
				}
			}
		}

		results.sort();
		Ok(results)
	}

	/// Analyze a single file for metrics and issues
	///
	/// # Arguments
	/// - `path`: Path to file to analyze
	///
	/// # Returns
	/// FileAnalysis with metrics, counts, and warnings
	pub fn analyze_file(path: &Path) -> Result<FileAnalysis> {
		let content = fs::read_to_string(path)
			.map_err(|e| MemoryPError::Other(format!("Failed to read file {:?}: {}", path, e)))?;

		let path_str = path.to_string_lossy().to_string();
		let mut analysis = FileAnalysis {
			file_path: path_str,
			lines_of_code: 0,
			lines_with_code: 0,
			blank_lines: 0,
			comment_lines: 0,
			functions: 0,
			structs: 0,
			imports: 0,
			complexity_estimate: 1.0,
			security_score: 100.0,
			warnings: Vec::new(),
		};

		Self::analyze_content(&content, &mut analysis);

		Ok(analysis)
	}

	/// Analyze file content and populate FileAnalysis
	fn analyze_content(content: &str, analysis: &mut FileAnalysis) {
		let mut in_multiline_comment = false;

		for line in content.lines() {
			let trimmed = line.trim();

			// Track multiline comments
			if trimmed.contains("/*") {
				in_multiline_comment = true;
			}
			if trimmed.contains("*/") {
				in_multiline_comment = false;
				analysis.comment_lines += 1;
				continue;
			}

			// Skip lines inside multiline comments
			if in_multiline_comment {
				analysis.comment_lines += 1;
				continue;
			}

			// Blank line
			if trimmed.is_empty() {
				analysis.blank_lines += 1;
				continue;
			}

			// Comment line
			if trimmed.starts_with("//") {
				analysis.comment_lines += 1;
				continue;
			}

			// Code line
			analysis.lines_with_code += 1;
			analysis.lines_of_code += 1;

			// Count constructs
			if trimmed.contains("fn ") && !trimmed.starts_with("//") {
				analysis.functions += 1;
			}
			if (trimmed.contains("struct ") || trimmed.contains("trait ") || trimmed.contains("impl "))
				&& !trimmed.starts_with("//")
			{
				analysis.structs += 1;
			}
			if (trimmed.starts_with("use ") || trimmed.starts_with("mod "))
				&& !trimmed.starts_with("//")
			{
				analysis.imports += 1;
			}
		}

		// Calculate complexity
		analysis.complexity_estimate = Self::calculate_complexity(content);

		// Security analysis
		Self::analyze_security(content, analysis);

		// Pattern detection
		Self::detect_patterns(content, analysis);
	}

	/// Calculate cyclomatic complexity estimate
	fn calculate_complexity(content: &str) -> f64 {
		let mut complexity = 1.0;

		// Count control flow constructs
		let keywords = [
			("if ", 1.0),
			("else ", 1.0),
			("match ", 1.5),
			("for ", 1.0),
			("while ", 1.0),
			("&&", 0.5),
			("||", 0.5),
			("?", 0.3),
			("=>", 0.1),
		];

		for (keyword, weight) in &keywords {
			complexity += content.matches(keyword).count() as f64 * weight;
		}

		// Find max nesting depth and penalize
		let mut max_depth: i32 = 0;
		let mut current_depth: i32 = 0;
		for ch in content.chars() {
			if ch == '{' {
				current_depth += 1;
				max_depth = max_depth.max(current_depth);
			} else if ch == '}' {
				current_depth = (current_depth - 1).max(0);
			}
		}

		// Add nesting penalty
		if max_depth > 3 {
			complexity += (max_depth - 3) as f64 * 0.2;
		}

		complexity.min(100.0) // Cap at 100
	}

	/// Analyze for security issues and style violations
	fn analyze_security(content: &str, analysis: &mut FileAnalysis) {
		let mut security_score = 100.0_f64;

		// Unsafe blocks
		let unsafe_count = content.matches("unsafe").count();
		if unsafe_count > 0 {
			analysis.warnings.push(format!("⚠️ {} unsafe blocks detected", unsafe_count));
			security_score -= (unsafe_count as f64) * 20.0;
		}

		// Unwrap/expect calls
		let unwrap_count = content.matches(".unwrap()").count()
			+ content.matches("unwrap((").count()
			+ content.matches(".expect(").count();
		if unwrap_count > 5 {
			analysis
				.warnings
				.push(format!("⚠️ {} unwrap/expect calls", unwrap_count));
			security_score -= (unwrap_count.saturating_sub(5)) as f64 * 2.0;
		}

		// Shell command execution
		if content.contains("std::process::") || content.contains("Command::new(") {
			analysis
				.warnings
				.push("⚠️ Shell command execution (injection risk)".into());
			security_score -= 30.0;
		}

		// Panic/unreachable
		let panic_count = content.matches("panic!(").count() + content.matches("unreachable!(").count();
		if panic_count > 0 {
			analysis
				.warnings
				.push(format!("ℹ️ {} panic/unreachable calls", panic_count));
			security_score -= (panic_count as f64) * 5.0;
		}

		// Mutex/lock contention
		let lock_count = content.matches("Mutex").count() + content.matches("RwLock").count();
		if lock_count > 5 {
			analysis
				.warnings
				.push(format!("ℹ️ {} mutex/rwlock instances (check contention)", lock_count));
			security_score -= (lock_count.saturating_sub(5)) as f64 * 1.0;
		}

		// Deprecated APIs
		if content.contains("use_new") || content.contains("from_raw_parts") {
			analysis
				.warnings
				.push("⚠️ Potentially deprecated/unsafe API usage".into());
			security_score -= 10.0;
		}

		analysis.security_score = security_score.max(0.0);
	}

	/// Detect architectural patterns
	fn detect_patterns(content: &str, analysis: &mut FileAnalysis) {
		let patterns = [
			("async fn", "async"),
			("#[tokio::main]", "tokio runtime"),
			("impl Future", "Future"),
			("pub trait", "public traits"),
			("serde::", "serde"),
			("rayon::", "rayon parallelism"),
			("dashmap::", "dashmap concurrent"),
		];

		let mut detected = HashSet::new();

		for (pat, name) in &patterns {
			if content.contains(pat) {
				detected.insert(name.to_string());
			}
		}

		if !detected.is_empty() {
			let mut list: Vec<_> = detected.into_iter().collect();
			list.sort();
			analysis.warnings.push(format!("✓ Patterns: {}", list.join(", ")));
		}
	}
}
