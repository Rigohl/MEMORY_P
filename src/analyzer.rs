// analyzer.rs - Analizador de código con métricas y sugerencias
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use lazy_static::lazy_static;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileAnalysis {
    pub file_path: String,
    pub lines_of_code: usize,
    pub lines_with_code: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub complexity_estimate: f32,
    pub functions: usize,
    pub structs: usize,
    pub imports: usize,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
    pub security_score: u8,
}

pub struct CodeAnalyzer;

impl CodeAnalyzer {
    pub fn scan_files(root_path: &str, extension: &str, use_gitignore: bool, include_hidden: bool) -> crate::error::Result<Vec<PathBuf>> {
        let root = Path::new(root_path);
        if !root.exists() || !root.is_dir() {
            return Err(crate::error::MemoryPError::InvalidDirectory(root_path.to_string()));
        }
        let walker = ignore::WalkBuilder::new(root_path)
            .hidden(!include_hidden)
            .git_ignore(use_gitignore)
            .threads(num_cpus::get())
            .build();
        let files: Vec<PathBuf> = walker.into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map_or(false, |ft| ft.is_file()))
            .filter(|e| e.path().extension().map_or(false, |ext| ext == extension))
            .map(|e| e.path().to_path_buf())
            .collect();
        Ok(files)
    }

    pub fn analyze_file(file_path: &Path) -> crate::error::Result<FileAnalysis> {
        let content = fs::read_to_string(file_path)?;
        let lines: Vec<&str> = content.lines().collect();
        let blank_lines = lines.iter().filter(|l| l.trim().is_empty()).count();
        let comment_lines = count_regex(&content, &RE_COMMENT);
        let lines_with_code = lines.len() - blank_lines - comment_lines;

        let functions = count_regex(&content, &RE_FN);
        let structs = count_regex(&content, &RE_STRUCT);
        let imports = count_regex(&content, &RE_USE);
        let complexity_estimate = estimate_complexity(&content, 1.0);
        let warnings = detect_warnings(&content, file_path);
        let suggestions = generate_suggestions(&warnings);

        Ok(FileAnalysis {
            file_path: file_path.to_string_lossy().to_string(),
            lines_of_code: lines.len(),
            lines_with_code,
            blank_lines,
            comment_lines,
            complexity_estimate,
            functions,
            structs,
            imports,
            warnings,
            suggestions,
            security_score: 100,
        })
    }
}

lazy_static! {
    static ref RE_FN: Regex = Regex::new(r"fn\s+\w+").unwrap();
    static ref RE_STRUCT: Regex = Regex::new(r"struct\s+\w+").unwrap();
    static ref RE_USE: Regex = Regex::new(r"use\s+").unwrap();
    static ref RE_COMMENT: Regex = Regex::new(r"//").unwrap();
    static ref RE_SEC_API_KEY: Regex = Regex::new(r"sk-[a-zA-Z0-9]{48}").unwrap();
}

fn count_regex(content: &str, re: &Regex) -> usize { re.find_iter(content).count() }
fn estimate_complexity(content: &str, base: f32) -> f32 { base + (count_regex(content, &RE_FN) as f32 * 1.5) }
fn detect_warnings(content: &str, _file_path: &Path) -> Vec<String> {
    let mut warnings = Vec::new();
    if content.contains("unwrap()") { warnings.push("⚠️ RUST: Uso de unwrap() detectado".into()); }
    if RE_SEC_API_KEY.is_match(content) { warnings.push("🛡️ SEGURIDAD: API Key detectada".into()); }
    warnings
}
fn generate_suggestions(warnings: &[String]) -> Vec<String> {
    warnings.iter().map(|w| if w.contains("unwrap") { "Usa expect() o manejo de errores." } else { "Revisar código." }.to_string()).collect()
}
