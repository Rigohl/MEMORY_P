// [memory_p] ULTRA-SPEED PARALLEL EDIT
//! workspace.rs - Núcleo de Análisis y Edición Masiva
//! 100% Rust puro, paralelizable y optimizado para alto rendimiento.
//!
//! Funcionalidades:
//! - Análisis de archivos con detección de vulnerabilidades
//! - Edición y normalización de código
//! - Reparación automática de issues comunes
//! - Procesamiento paralelo con Rayon

use crate::analyzer::CodeAnalyzer;
use crate::error::{MemoryPError, Result};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
// use std::sync::Mutex; // REMOVED as system is now lock-free

// El bloqueo global fue removido para permitir escalado masivo 1M+ ops.

/// Analiza un archivo con el escáner "Nuclear God Mode" y métricas detalladas

pub fn analyze_file(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    let mut findings = Vec::new();

    // --- METRICAS DETALLADAS ---
    match CodeAnalyzer::analyze_file(path) {
        Ok(analysis) => {
            findings.push(format!("📊 FILE: {}", analysis.file_path));
            findings.push(format!(
                "📏 LOC: {} (Code: {}, Blank: {}, Comments: {})",
                analysis.lines_of_code,
                analysis.lines_with_code,
                analysis.blank_lines,
                analysis.comment_lines
            ));
            findings.push(format!(
                "📈 Functions: {}, Structs: {}, Imports: {}, Complexity: {:.1}",
                analysis.functions,
                analysis.structs,
                analysis.imports,
                analysis.complexity_estimate
            ));
            for warning in analysis.warnings {
                findings.push(format!("⚠️ {}", warning));
            }
        }
        Err(e) => {
            return Err(MemoryPError::AnalysisError(format!(
                "Error analizando {}: {}",
                path.display(),
                e
            )))
        }
    }

    // --- SEGURIDAD Y VULNERABILIDADES ---
    if content.contains("unsafe") {
        findings.push("☢️ UNSAFE".to_string());
    }
    if content.contains(".unwrap()") {
        findings.push("💥 POTENCIAL PANIC (.unwrap)".to_string());
    }
    if content.contains("std::process::Command") {
        findings.push("🐚 SHELL EXECUTION".to_string());
    }

    // --- RENDIMIENTO ---
    if content.contains("Mutex<") {
        findings.push("🔒 MUTEX LOCK (Contención)".to_string());
    }
    if content.contains("static mut") {
        findings.push("🦠 STATIC MUT (Globar State)".to_string());
    }
    if content.contains(".clone()") && content.len() > 10000 {
        findings.push("🧬 HEAVY CLONING".to_string());
    }

    // --- NEXT-GEN RUNTIMES (HVM, BEND, WASM) ---
    if content.contains("bend") || content.contains("HVM") {
        findings.push("🌀 NEXT-GEN RUNTIME (Bend/HVM)".to_string());
    }
    if content.contains("wasm-bindgen") || content.contains("js-sys") {
        findings.push("🕸️ WASM MODULE".to_string());
    }

    // --- CALIDAD DE CÓDIGO ---
    if content.contains("DONE_BY_ULTRA_ENGINE") || content.contains("FIXME") {
        findings.push("📝 PENDIENTE (DONE_BY_ULTRA_ENGINE/FIXME)".to_string());
    }
    if content.lines().count() > 500 {
        findings.push("📐 ARCHIVO GRANDE (>500 líneas)".to_string());
    }

    if findings.is_empty() {
        Ok(format!("{}: ✅ OK", path.display()))
    } else {
        Ok(format!("{}: [{}]", path.display(), findings.join(", ")))
    }
}

/// Procesa múltiples archivos en paralelo para cualquier operación con bloqueo de seguridad
pub fn process_parallel<F>(paths: &[PathBuf], op: F) -> Result<Vec<Result<String>>>
where
    F: Fn(&Path) -> Result<String> + Sync + Send,
{
    // ⚡ SIN LOCKS: Cada hilo procesa su archivo de forma aislada.
    let results: Vec<Result<String>> = paths.par_iter().map(|p| op(p)).collect();

    if results.is_empty() && !paths.is_empty() {
        return Err(MemoryPError::ParallelError(
            "El procesamiento paralelo retornó resultados vacíos inesperadamente".to_string(),
        ));
    }

    Ok(results)
}

/// Edita un archivo: Normalización estructural (Indentation, Tabs, EOL)

pub fn edit_file(path: &Path) -> Result<String> {
    // ⚡ MMAP READ (Zero-copy I/O)
    let file = fs::File::open(path)?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    let content = String::from_utf8_lossy(&mmap);

    let mut changed = false;
    let mut modified = String::with_capacity(mmap.len());

    for line in content.lines() {
        let trimmed = line.trim_end();
        let indented = trimmed.replace('\t', "    ");
        if indented != line {
            changed = true;
        }
        modified.push_str(&indented);
        modified.push('\n');
    }

    if changed {
        fs::write(path, &modified)?;
        Ok(format!(
            "{}: ✨ NORMALIZADO (Zero-copy Buffering)",
            path.display()
        ))
    } else {
        Ok(format!("{}: ❄️ YA ESTÁ ÓPTIMO", path.display()))
    }
}

/// Reparación inteligente optimizada (Import cleanup and EOL normalization)
pub fn smart_repair(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    let mut seen_imports = std::collections::HashSet::new();
    let mut modified = String::with_capacity(content.len());
    let mut changes = Vec::with_capacity(3);

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("use ") {
            if seen_imports.insert(trimmed.to_string()) {
                modified.push_str(line.trim_end());
                modified.push('\n');
            } else if !changes.contains(&"Duplicate import removed") {
                changes.push("Duplicate import removed");
            }
        } else {
            modified.push_str(line.trim_end());
            modified.push('\n');
        }
    }

    if modified != content {
        fs::write(path, &modified)?;
        Ok(format!(
            "{}: 🛠️ SMART REPAIR [ {} ]",
            path.display(),
            if changes.is_empty() {
                "Formatting".to_string()
            } else {
                changes.join(", ")
            }
        ))
    } else {
        Ok(format!("{}: ✨ CÓDIGO YA ÓPTIMO", path.display()))
    }
}

/// Repara un archivo: Fixes automáticos de "God Mode"
pub fn repair_file(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    let mut new_lines = Vec::new();
    let mut empty_count = 0;
    let mut changed = false;

    for line in content.lines() {
        if line.trim().is_empty() {
            empty_count += 1;
        } else {
            empty_count = 0;
        }

        if empty_count <= 2 {
            new_lines.push(line);
        } else {
            changed = true;
        }
    }

    if changed {
        let result = new_lines.join("\n") + "\n";
        fs::write(path, result)?;
        Ok(format!(
            "{}: 🛠️ REPARADO (Espacios redundantes eliminados)",
            path.display()
        ))
    } else {
        Ok(format!("{}: ✨ NO REQUIRIÓ REPARACIÓN", path.display()))
    }
}
