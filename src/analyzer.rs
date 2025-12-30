// analyzer.rs - Analizador de código con métricas básicas
// Proporciona análisis de archivos Rust individuales

#[allow(unused_imports)]
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
// walkdir replaced by 'ignore' crate (Ripgrep engine)

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
    pub security_score: u8,
}

pub struct CodeAnalyzer;

impl CodeAnalyzer {
    /// Escanea un directorio recursivamente y retorna paths de archivos con extensión dada de forma paralela (Ripgrep Engine)
    pub fn scan_files(
        root_path: &str,
        extension: &str,
        use_gitignore: bool,
        include_hidden: bool,
    ) -> crate::error::Result<Vec<PathBuf>> {
        let root = Path::new(root_path);
        if !root.exists() || !root.is_dir() {
            return Err(crate::error::MemoryPError::InvalidDirectory(
                root_path.to_string(),
            ));
        }

        let walker = ignore::WalkBuilder::new(root_path)
            .hidden(!include_hidden)
            .git_ignore(use_gitignore)
            .threads(num_cpus::get())
            .build();

        let files: Vec<PathBuf> = walker
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map_or(false, |ft| ft.is_file()))
            .filter(|e| e.path().extension().map_or(false, |ext| ext == extension))
            .map(|e| e.path().to_path_buf())
            .collect();

        if files.is_empty() {
            // Silencioso o debug log
            // tracing::debug!(...)
        }

        Ok(files)
    }

    /// Analiza un archivo individual y retorna métricas
    pub fn analyze_file(file_path: &Path) -> crate::error::Result<FileAnalysis> {
        if !file_path.exists() {
            return Err(crate::error::MemoryPError::FileNotFound(
                file_path.to_path_buf(),
            ));
        }

        // ⚡ CACHE CHECK (Wait-free Read)
        let metadata = fs::metadata(file_path)?;
        let modified = metadata.modified()?;
        let path_key = file_path.to_string_lossy().to_string();

        if let Some(entry) = ANALYSIS_CACHE.get(&path_key) {
            if entry.0 == modified {
                return Ok(entry.1.clone());
            }
        }

        // ⚡ MMAP READ (Zero-copy I/O)
        let file = fs::File::open(file_path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let content = String::from_utf8_lossy(&mmap);
        let lines: Vec<&str> = content.lines().collect();

        let blank_lines = lines.iter().filter(|l| l.trim().is_empty()).count();
        let comment_lines = count_regex(&content, &RE_COMMENT);
        let lines_with_code = lines.len() - blank_lines - comment_lines;

        // Dynamic Syntax Analysis
        let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let (re_fn, re_struct): (&Regex, &Regex) = match ext {
            "py" => (&*RE_DEF, &*RE_CLASS),
            "go" => (&*RE_FUNC, &*RE_TYPE),
            "bend" | "hvm" => (&*RE_DEF, &*RE_TYPE),
            _ => (&*RE_FN, &*RE_STRUCT),
        };

        let functions = count_regex(&content, re_fn);
        let structs = count_regex(&content, re_struct);
        let imports = count_regex(&content, &RE_USE);

        let complexity_estimate = estimate_complexity(&content, 1.0);
        let warnings = detect_warnings(&content, file_path);

        let result = FileAnalysis {
            file_path: file_path.to_string_lossy().to_string(),
            lines_of_code: lines.len(),
            lines_with_code,
            blank_lines,
            comment_lines,
            complexity_estimate,
            functions,
            structs,
            imports,
            warnings: warnings.clone(),
            security_score: calculate_security_score(&warnings),
        };

        let _ = ANALYSIS_CACHE.insert(path_key, (modified, result.clone()));
        Ok(result)
    }
}

use lazy_static::lazy_static;

// use dashmap::DashMap; // REMOVED as SCC is used

lazy_static! {
    // Cache Concurrente Maestramiente (SCC: Scalable Concurrent Cache)
    static ref ANALYSIS_CACHE: scc::HashMap<String, (std::time::SystemTime, FileAnalysis)> = scc::HashMap::new();

    static ref RE_FN: Regex = Regex::new(r"fn\s+\w+").unwrap();
    static ref RE_STRUCT: Regex = Regex::new(r"struct\s+\w+").unwrap();
    static ref RE_USE: Regex = Regex::new(r"use\s+").unwrap();
    static ref RE_COMMENT: Regex = Regex::new(r"//").unwrap();

    // Complejidad
    static ref RE_COMPLEX_IF: Regex = Regex::new(r"if\s*[({]").unwrap();
    static ref RE_COMPLEX_ELSE: Regex = Regex::new(r"else").unwrap();
    static ref RE_COMPLEX_MATCH: Regex = Regex::new(r"match\s*").unwrap();
    static ref RE_COMPLEX_FOR: Regex = Regex::new(r"for\s+").unwrap();
    static ref RE_COMPLEX_WHILE: Regex = Regex::new(r"while\s+").unwrap();

    // Seguridad
    static ref RE_SEC_API_KEY_GOOGLE: Regex = Regex::new(r"AIza[0-9A-Za-z-_]{35}").unwrap();
    static ref RE_SEC_API_KEY_OPENAI: Regex = Regex::new(r"sk-[a-zA-Z0-9]{48}").unwrap();
    static ref RE_SEC_PASSWORD: Regex = Regex::new(r"(?i)password\s*[:=]").unwrap();

    // Multilingual Support
    static ref RE_DEF: Regex = Regex::new(r"def\s+\w+").unwrap();    // Python, Bend
    static ref RE_CLASS: Regex = Regex::new(r"class\s+\w+").unwrap(); // Python, TS
    static ref RE_FUNC: Regex = Regex::new(r"func\s+\w+").unwrap();   // Go
    static ref RE_TYPE: Regex = Regex::new(r"type\s+\w+").unwrap();   // Go, Bend
}

// --- FUNCIONES AUXILIARES OPTIMIZADAS (MAX JUICE) ---

fn count_regex(content: &str, re: &Regex) -> usize {
    re.find_iter(content).count()
}

/// Estima complejidad del código usando heurísticas pre-compiladas
fn estimate_complexity(content: &str, base: f32) -> f32 {
    let mut complexity = base;

    // Contar puntos de decisión (Zero additional allocations)
    complexity += count_regex(content, &RE_COMPLEX_IF) as f32 * 1.5;
    complexity += count_regex(content, &RE_COMPLEX_ELSE) as f32 * 0.5;
    complexity += count_regex(content, &RE_COMPLEX_MATCH) as f32 * 2.0;
    complexity += count_regex(content, &RE_COMPLEX_FOR) as f32 * 1.5;
    complexity += count_regex(content, &RE_COMPLEX_WHILE) as f32 * 1.5;
    complexity += count_regex(content, &RE_FN) as f32 * 0.5;

    complexity
}

/// Detecta warnings potenciales en el código de forma eficiente
fn detect_warnings(content: &str, file_path: &Path) -> Vec<String> {
    let mut warnings = Vec::new();

    // 4. Analizador Multilingüe Dinámico
    let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");

    match ext {
        "mojo" | "🔥" => {
            if content.contains("Python.import") {
                warnings.push("📦 MOJO: Interoperabilidad con Python detectada".into());
            }
            if !content.contains("struct") && content.contains("fn ") {
                warnings.push(
                    "⚠️ MOJO: Considera usar 'struct' para performance en lugar de solo funciones"
                        .into(),
                );
            }
        }
        "py" => {
            if content.contains("eval(") {
                warnings.push("🛡️ SEGURIDAD (Python): Uso de eval() detectado".into());
            }
            if content.contains("pickle.load") {
                warnings.push("🛡️ SEGURIDAD (Python): Deserialización insegura con pickle".into());
            }
            if !content.contains("def main():") && !content.contains("if __name__") {
                warnings.push("⚠️ Python: Script sin entry point claro (main)".into());
            }
        }
        "rs" => {
            if content.contains("unsafe {") {
                warnings.push("☢️ RUST: Bloque unsafe detectado".into());
            }
            if content.contains("unwrap()") {
                warnings.push("⚠️ RUST: Uso de unwrap() en producción".into());
            }
            // Performance Anti-patterns
            if content.contains(".clone()") && content.len() > 5000 {
                warnings.push("🧬 RUST: Heavy cloning detectado en archivo grande".into());
            }
            if content.contains("Mutex<") {
                warnings.push("🔒 RUST: Mutex lock (contención potencial)".into());
            }
            if content.contains("static mut") {
                warnings.push("🦠 RUST: static mut (estado global inseguro)".into());
            }
            // Zero-copy opportunities
            if content.contains("to_string()") && content.matches("to_string()").count() > 10 {
                warnings.push("📦 RUST: Múltiples to_string() - considerar Cow<str>".into());
            }
            if content.contains("Vec::new()") && !content.contains("with_capacity") {
                warnings.push("📐 RUST: Vec sin with_capacity - optimización posible".into());
            }
        }
        "go" => {
            if content.contains("interface{}") {
                warnings.push("⚠️ GO: Uso de interface{} vacía (Any). Tipado débil.".into());
            }
        }
        "bend" | "hvm" => {
            if content.contains("fold") && !content.contains("case") {
                warnings.push("⚠️ BEND: 'fold' recursivo sin pattern matching 'case'".into());
            }
            if !content.contains("def main:") {
                warnings.push("⚠️ BEND: Falta 'def main:'".into());
            }
            // Bend GPU optimization hints
            if content.contains("return") && !content.contains("bend run-cu") {
                warnings.push("🚀 BEND: Código paralelizable - considerar run-cu para GPU".into());
            }
        }
        "chpl" => {
            if content.contains("forall") && !content.contains("with") {
                warnings
                    .push("⚠️ CHAPEL: 'forall' paralelo. Verificar data race o usar 'with'".into());
            }
        }
        "jl" => {
            // Julia analysis
            if content.contains("@threads") && !content.contains("Threads.nthreads()") {
                warnings.push("⚠️ JULIA: @threads sin verificar nthreads()".into());
            }
            if content.contains("global ") {
                warnings.push("🦠 JULIA: Variable global detectada".into());
            }
        }
        "ts" | "tsx" => {
            // TypeScript analysis
            if content.contains("any") {
                warnings.push("⚠️ TS: Tipo 'any' detectado - tipado débil".into());
            }
            if content.contains("// @ts-ignore") {
                warnings.push("⚠️ TS: @ts-ignore encontrado".into());
            }
        }
        _ => {}
    }

    // 5. Detectar Secretos y API Keys (Usando Regex pre-compiladas)
    if RE_SEC_API_KEY_GOOGLE.is_match(content) {
        warnings.push("🛡️ SEGURIDAD: Google API Key detectada".into());
    }
    if RE_SEC_API_KEY_OPENAI.is_match(content) {
        warnings.push("🛡️ SEGURIDAD: OpenAI API Key detectada".into());
    }
    if RE_SEC_PASSWORD.is_match(content) {
        warnings.push("🛡️ SEGURIDAD: Password Hardcoded detectado".into());
    }

    warnings
}

/// Calcula score de seguridad (0-100)
fn calculate_security_score(warnings: &[String]) -> u8 {
    let mut score = 100i32;
    for w in warnings {
        if w.contains("SEGURIDAD") || w.contains("API Key") || w.contains("Password") {
            score -= 25; // Penalización máxima por secretos expuestos
        } else if w.contains("unsafe") {
            score -= 15; // Penalización media por unsafe
        } else {
            score -= 5; // Penalización leve
        }
    }
    score.clamp(0, 100) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_pattern() {
        let content = "fn foo() {}\nfn bar() {}";
        assert_eq!(count_regex(content, &RE_FN), 2);
    }

    #[test]
    fn test_estimate_complexity() {
        let content = "fn foo() { if true { } if false { } }";
        let complexity = estimate_complexity(content, 1.0);
        assert!(complexity > 2.0 && complexity < 10.0);
    }
}
