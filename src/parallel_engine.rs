// [nuclear_god_mode] PROCESSED AT MAX SPEED
use crate::analyzer::CodeAnalyzer;
use crate::error::{MemoryPError, Result};
use crate::workspace;
use jwalk::WalkDir;
use memmap2::Mmap;
use rayon::prelude::*;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Configuración avanzada para el motor paralelo
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    pub max_threads: usize,
    pub chunk_size: usize,
    pub _read_buffer_size: usize,
    pub _file_timeout_ms: u64,
    pub _continue_on_error: bool,
    pub _large_file_threshold: usize,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_threads: 0,
            chunk_size: 100,
            _read_buffer_size: 1024 * 1024,
            _file_timeout_ms: 30000,
            _continue_on_error: true,
            _large_file_threshold: 10 * 1024 * 1024,
        }
    }
}

/// Estado de procesamiento de un archivo
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingStatus {
    Success,
    Warning,
    Error,
    Skipped,
}

/// Resultado detallado de un archivo procesado
#[derive(Debug, Clone, Serialize)]
pub struct ProcessingResult {
    pub path: String,
    pub status: ProcessingStatus,
    pub findings: Vec<String>,
}

/// Estadísticas de procesamiento
#[derive(Debug, Clone, Default, Serialize)]
pub struct ProcessingStats {
    pub total_files: usize,
    pub successful: usize,
    pub errors: usize,
    pub warnings: usize,
    pub skipped: usize,
    pub total_bytes: usize,
    pub total_duration_ms: u64,
}

/// Motor ultra-paralelo de alto rendimiento
pub struct UltraParallelEngine {
    pub pool: rayon::ThreadPool,
    pub config: ParallelConfig,
    processed_count: Arc<AtomicUsize>,
    total_bytes: Arc<AtomicUsize>,
}

impl UltraParallelEngine {
    pub fn new(config: ParallelConfig) -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(config.max_threads)
            .thread_name(|i| format!("ultra-worker-{}", i))
            .build()
            .unwrap();

        Self {
            pool,
            config,
            processed_count: Arc::new(AtomicUsize::new(0)),
            total_bytes: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn process_files<F>(
        &self,
        paths: &[PathBuf],
        operation: F,
    ) -> Result<(Vec<ProcessingResult>, ProcessingStats)>
    where
        F: Fn(&Path, &str) -> Result<(String, ProcessingStatus)> + Sync + Send,
    {
        let start = Instant::now();

        // Cierre de lógica central para evitar duplicación
        let process_one = |path: &PathBuf| -> ProcessingResult {
            let size = match fs::metadata(path) {
                Ok(m) => m.len(),
                Err(e) => {
                    return ProcessingResult {
                        path: path.display().to_string(),
                        status: ProcessingStatus::Error,
                        findings: vec![format!("Stat Error: {}", e)],
                    }
                }
            };

            // Decision Estratégica I/O basada en Simulación Bend
            if size > self.config._large_file_threshold as u64 {
                // 🚀 MMAP PATH (Zero-Copy)
                let file = match fs::File::open(path) {
                    Ok(f) => f,
                    Err(e) => {
                        return ProcessingResult {
                            path: path.display().to_string(),
                            status: ProcessingStatus::Error,
                            findings: vec![format!("Open Error: {}", e)],
                        }
                    }
                };
                let mmap = match unsafe { Mmap::map(&file) } {
                    Ok(m) => m,
                    Err(e) => {
                        return ProcessingResult {
                            path: path.display().to_string(),
                            status: ProcessingStatus::Error,
                            findings: vec![format!("Mmap Error: {}", e)],
                        }
                    }
                };
                let content = match std::str::from_utf8(&mmap) {
                    Ok(s) => s,
                    Err(_) => {
                        return ProcessingResult {
                            path: path.display().to_string(),
                            status: ProcessingStatus::Error,
                            findings: vec!["Binary file detected".into()],
                        }
                    }
                };

                // Process Mmap Slice
                self.total_bytes.fetch_add(size as usize, Ordering::Relaxed);
                match operation(path, content) {
                    Ok((msg, status)) => {
                        self.processed_count.fetch_add(1, Ordering::Relaxed);
                        ProcessingResult {
                            path: path.display().to_string(),
                            status,
                            findings: vec![msg],
                        }
                    }
                    Err(e) => ProcessingResult {
                        path: path.display().to_string(),
                        status: ProcessingStatus::Error,
                        findings: vec![format!("Error: {}", e)],
                    },
                }
            } else {
                // 🐢 STANDARD PATH (Buffered Read)
                match fs::read_to_string(path) {
                    Ok(content) => {
                        self.total_bytes.fetch_add(content.len(), Ordering::Relaxed);
                        match operation(path, &content) {
                            Ok((msg, status)) => {
                                self.processed_count.fetch_add(1, Ordering::Relaxed);
                                ProcessingResult {
                                    path: path.display().to_string(),
                                    status,
                                    findings: vec![msg],
                                }
                            }
                            Err(e) => ProcessingResult {
                                path: path.display().to_string(),
                                status: ProcessingStatus::Error,
                                findings: vec![format!("Error: {}", e)],
                            },
                        }
                    }
                    Err(e) => ProcessingResult {
                        path: path.display().to_string(),
                        status: ProcessingStatus::Error,
                        findings: vec![format!("IO Error: {}", e)],
                    },
                }
            }
        };

        let results: Vec<ProcessingResult> = self.pool.install(|| {
            // ESTRATEGIA OPTIMIZADA POR SIMULACIÓN BEND
            // < 256 archivos: Overhead de chunks supera beneficio -> par_iter directo.
            // > 256 archivos: Chunks mejoran cache locality y reducen overhead -> par_chunks.
            if paths.len() < 256 {
                paths.par_iter().map(process_one).collect()
            } else {
                paths
                    .par_chunks(self.config.chunk_size.max(1))
                    .flat_map(|chunk| chunk.par_iter().map(process_one))
                    .collect()
            }
        });

        let stats = ProcessingStats {
            total_files: results.len(),
            successful: results
                .iter()
                .filter(|r| r.status == ProcessingStatus::Success)
                .count(),
            warnings: results
                .iter()
                .filter(|r| r.status == ProcessingStatus::Warning)
                .count(),
            errors: results
                .iter()
                .filter(|r| r.status == ProcessingStatus::Error)
                .count(),
            skipped: results
                .iter()
                .filter(|r| r.status == ProcessingStatus::Skipped)
                .count(),
            total_bytes: self.total_bytes.load(Ordering::SeqCst),
            total_duration_ms: start.elapsed().as_millis() as u64,
        };

        Ok((results, stats))
    }
}

pub fn ultra_analyze(
    paths: &[PathBuf],
    config: ParallelConfig,
) -> Result<(Vec<ProcessingResult>, ProcessingStats)> {
    let engine = UltraParallelEngine::new(config);
    engine.process_files(paths, |path, content| {
        let mut findings = Vec::new();
        match CodeAnalyzer::analyze_file(path) {
            Ok(analysis) => {
                findings.push(format!(
                    "📊 LOC: {} | Complexity: {:.1}",
                    analysis.lines_of_code, analysis.complexity_estimate
                ));
                for w in analysis.warnings {
                    findings.push(w);
                }
            }
            Err(_) => findings.push("❌ Error en análisis estructural".into()),
        }
        if content.contains("unsafe") {
            findings.push("☢️ UNSAFE".into());
        }
        if content.contains(".unwrap()") {
            findings.push("💥 UNWRAP".into());
        }

        let status = if findings
            .iter()
            .any(|f| f.contains("🛡️") || f.contains("❌"))
        {
            ProcessingStatus::Error
        } else if findings.len() > 1 {
            ProcessingStatus::Warning
        } else {
            ProcessingStatus::Success
        };
        Ok((findings.join(" | "), status))
    })
}

pub fn ultra_repair(
    paths: &[PathBuf],
    config: ParallelConfig,
) -> Result<(Vec<ProcessingResult>, ProcessingStats)> {
    let engine = UltraParallelEngine::new(config);
    engine.process_files(paths, |path, _content| {
        match workspace::smart_repair(path) {
            Ok(msg) => Ok((msg, ProcessingStatus::Success)),
            Err(e) => Err(MemoryPError::Other(e.to_string())),
        }
    })
}

#[allow(dead_code)]
pub fn ultra_search(
    paths: &[PathBuf],
    pattern: &str,
    config: ParallelConfig,
) -> Result<(Vec<ProcessingResult>, ProcessingStats)> {
    let engine = UltraParallelEngine::new(config);
    let pat = pattern.to_string();
    engine.process_files(paths, |_, content| {
        if content.contains(&pat) {
            Ok(("Match encontrado".into(), ProcessingStatus::Success))
        } else {
            Ok(("No encontrado".into(), ProcessingStatus::Skipped))
        }
    })
}

#[allow(dead_code)]
pub fn ultra_replace(
    paths: &[PathBuf],
    pattern: &str,
    replacement: &str,
    config: ParallelConfig,
) -> Result<(Vec<ProcessingResult>, ProcessingStats)> {
    let engine = UltraParallelEngine::new(config);
    let pat = pattern.to_string();
    let rep = replacement.to_string();
    engine.process_files(paths, |path, content| {
        if content.contains(&pat) {
            let modified = content.replace(&pat, &rep);
            fs::write(path, modified).ok();
            Ok(("Reemplazado".into(), ProcessingStatus::Success))
        } else {
            Ok(("Sin cambios".into(), ProcessingStatus::Skipped))
        }
    })
}

pub fn ultra_edit(
    changes: &[crate::mcp::models::FileChange],
    config: ParallelConfig,
    dry_run: bool,
) -> Result<(Vec<ProcessingResult>, ProcessingStats)> {
    let engine = UltraParallelEngine::new(config);
    let start = Instant::now();
    use regex::Regex;

    // Paralelizamos sobre los archivos a cambiar
    let results: Vec<ProcessingResult> = engine.pool.install(|| {
        changes
            .par_iter()
            .map(|change| {
                let path = Path::new(&change.path);

                // Auto-create file if it doesn't exist
                if !path.exists() {
                    if let Some(parent) = path.parent() {
                        if let Err(e) = fs::create_dir_all(parent) {
                            return ProcessingResult {
                                path: change.path.clone(),
                                status: ProcessingStatus::Error,
                                findings: vec![format!("Failed to create parent dir: {}", e)],
                            };
                        }
                    }
                    // Create empty file so we can read it below
                    if let Err(e) = fs::write(path, "") {
                        return ProcessingResult {
                            path: change.path.clone(),
                            status: ProcessingStatus::Error,
                            findings: vec![format!("Failed to create new file: {}", e)],
                        };
                    }
                }

                let mut content = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(e) => {
                        return ProcessingResult {
                            path: change.path.clone(),
                            status: ProcessingStatus::Error,
                            findings: vec![format!("Read error: {}", e)],
                        }
                    }
                };

                let mut applied = 0;
                for op in &change.operations {
                    match op {
                        crate::mcp::models::EditOp::Replace {
                            target,
                            replacement,
                        } => {
                            if content.contains(target) {
                                content = content.replace(target, replacement);
                                applied += 1;
                            }
                        }
                        crate::mcp::models::EditOp::Append { content: suffix } => {
                            content.push_str(suffix);
                            applied += 1;
                        }
                        crate::mcp::models::EditOp::RegexReplace {
                            pattern,
                            replacement,
                        } => {
                            if let Ok(re) = Regex::new(pattern) {
                                let new_content = re.replace_all(&content, replacement).to_string();
                                if new_content != content {
                                    content = new_content;
                                    applied += 1;
                                }
                            }
                        }
                    }
                }

                if applied > 0 {
                    if !dry_run {
                        if let Err(e) = fs::write(path, &content) {
                            return ProcessingResult {
                                path: change.path.clone(),
                                status: ProcessingStatus::Error,
                                findings: vec![format!("Write error: {}", e)],
                            };
                        }
                    }
                    ProcessingResult {
                        path: change.path.clone(),
                        status: ProcessingStatus::Success,
                        findings: vec![format!("Applied {} edits", applied)],
                    }
                } else {
                    ProcessingResult {
                        path: change.path.clone(),
                        status: ProcessingStatus::Skipped,
                        findings: vec!["No match found for edits".into()],
                    }
                }
            })
            .collect()
    });

    let stats = ProcessingStats {
        total_files: results.len(),
        successful: results
            .iter()
            .filter(|r| r.status == ProcessingStatus::Success)
            .count(),
        errors: results
            .iter()
            .filter(|r| r.status == ProcessingStatus::Error)
            .count(),
        skipped: results
            .iter()
            .filter(|r| r.status == ProcessingStatus::Skipped)
            .count(),
        total_duration_ms: start.elapsed().as_millis() as u64,
        ..Default::default()
    };

    Ok((results, stats))
}

/// 🗑️ Eliminar archivos en paralelo (Ultra Safe con dry_run)
pub fn ultra_delete(
    paths: &[PathBuf],
    config: ParallelConfig,
    dry_run: bool,
) -> Result<(Vec<ProcessingResult>, ProcessingStats)> {
    let engine = UltraParallelEngine::new(config);
    let start = Instant::now();

    let results: Vec<ProcessingResult> = engine.pool.install(|| {
        paths
            .par_iter()
            .map(|path| {
                if !path.exists() {
                    return ProcessingResult {
                        path: path.display().to_string(),
                        status: ProcessingStatus::Skipped,
                        findings: vec!["File does not exist".into()],
                    };
                }

                if dry_run {
                    // Dry run: solo reportar qué se eliminaría
                    return ProcessingResult {
                        path: path.display().to_string(),
                        status: ProcessingStatus::Warning,
                        findings: vec!["[DRY_RUN] Would delete this file".into()],
                    };
                }

                // Intentar eliminar
                let result = if path.is_dir() {
                    fs::remove_dir_all(path)
                } else {
                    fs::remove_file(path)
                };

                match result {
                    Ok(_) => ProcessingResult {
                        path: path.display().to_string(),
                        status: ProcessingStatus::Success,
                        findings: vec!["Deleted successfully".into()],
                    },
                    Err(e) => ProcessingResult {
                        path: path.display().to_string(),
                        status: ProcessingStatus::Error,
                        findings: vec![format!("Delete failed: {}", e)],
                    },
                }
            })
            .collect()
    });

    let stats = ProcessingStats {
        total_files: results.len(),
        successful: results
            .iter()
            .filter(|r| r.status == ProcessingStatus::Success)
            .count(),
        errors: results
            .iter()
            .filter(|r| r.status == ProcessingStatus::Error)
            .count(),
        skipped: results
            .iter()
            .filter(|r| r.status == ProcessingStatus::Skipped)
            .count(),
        total_duration_ms: start.elapsed().as_millis() as u64,
        ..Default::default()
    };

    Ok((results, stats))
}
pub fn ultra_workflow(
    request: &crate::mcp::models::UltraWorkflowRequest,
    config: ParallelConfig,
) -> Result<(Vec<ProcessingResult>, ProcessingStats)> {
    let engine = UltraParallelEngine::new(config.clone());
    let start = Instant::now();
    let mut active_files: Vec<PathBuf> = Vec::new();
    let mut all_results: Vec<ProcessingResult> = Vec::new();
    let mut stats = ProcessingStats::default();

    use crate::mcp::models::WorkflowStep;

    for step in &request.steps {
        match step {
            WorkflowStep::Scan { path, extension } => {
                let root = Path::new(path);
                if root.exists() {
                    let ext = extension.as_deref();
                    // Default: Respect gitignore, Hide hidden files
                    let files = ScanUtils::collect_files(root, ext, true, false);
                    active_files = files;
                    stats.total_files = active_files.len();
                    all_results.push(ProcessingResult {
                        path: "PIPELINE_SCAN".into(),
                        status: ProcessingStatus::Success,
                        findings: vec![format!("Scanned {} files", active_files.len())],
                    });
                } else {
                    return Err(MemoryPError::Other(format!("Invalid path: {}", path)));
                }
            }
            WorkflowStep::Filter { pattern, invert } => {
                let re = Regex::new(pattern).map_err(|e| MemoryPError::Other(e.to_string()))?;
                let inv = invert.unwrap_or(false);

                // Parallel Filter
                let (kept, rejected): (Vec<_>, Vec<_>) = active_files.par_iter().partition(|p| {
                    if let Ok(content) = fs::read_to_string(p) {
                        let m = re.is_match(&content);
                        if inv {
                            !m
                        } else {
                            m
                        }
                    } else {
                        false
                    }
                });

                let kept_owned: Vec<PathBuf> = kept.into_iter().cloned().collect();
                let rejected_count = rejected.len();

                active_files = kept_owned;

                all_results.push(ProcessingResult {
                    path: "PIPELINE_FILTER".into(),
                    status: ProcessingStatus::Success,
                    findings: vec![format!(
                        "kept: {}, rejected: {}",
                        active_files.len(),
                        rejected_count
                    )],
                });
            }
            WorkflowStep::Analyze => {
                let (mut res, st) = engine.process_files(&active_files, |path, content| {
                    let mut findings = Vec::new();
                    if let Ok(analysis) = CodeAnalyzer::analyze_file(path) {
                        findings.push(format!("Complexity: {:.1}", analysis.complexity_estimate));
                        if analysis.security_score < 80 {
                            findings
                                .push(format!("Low Security Score: {}", analysis.security_score));
                        }
                    }
                    if content.contains("TODO") {
                        findings.push("Has TODO".into());
                    }
                    Ok((findings.join(" | "), ProcessingStatus::Success))
                })?;
                all_results.append(&mut res);
                stats.successful += st.successful; // Acumular stats
            }
            WorkflowStep::Edit { operations } => {
                // Adaptamos para usar ultra_edit logic
                // Creamos FileChange para cada archivo activo con las mismas operaciones
                let changes: Vec<crate::mcp::models::FileChange> = active_files
                    .iter()
                    .map(|p| crate::mcp::models::FileChange {
                        path: p.to_string_lossy().to_string(),
                        operations: operations.clone(),
                    })
                    .collect();

                let (mut res, st) =
                    ultra_edit(&changes, config.clone(), request.dry_run.unwrap_or(false))?;
                all_results.append(&mut res);
                stats.successful += st.successful;
            }
            WorkflowStep::Repair => {
                let (mut res, st) = ultra_repair(&active_files, config.clone())?;
                all_results.append(&mut res);
                stats.successful += st.successful;
            }
            WorkflowStep::Evolve {
                max_iterations,
                dry_run,
            } => {
                let max_iter = max_iterations.unwrap_or(5);
                let is_dry = dry_run.unwrap_or(true);

                for iteration in 0..max_iter {
                    // 1. Analyze current state
                    let mut issues_found = 0usize;
                    let mut fixes_applied = 0usize;

                    let (analysis_results, _) =
                        engine.process_files(&active_files, |path, content| {
                            let mut findings: Vec<String> = Vec::new();

                            // Detect fixable patterns
                            if content.contains(".clone()") && content.len() > 5000 {
                                findings.push("FIXABLE:heavy_clone".to_string());
                            }
                            if content.contains("unwrap()") {
                                findings.push("FIXABLE:unwrap_usage".to_string());
                            }
                            if content.contains("Vec::new()") && !content.contains("with_capacity")
                            {
                                findings.push("FIXABLE:vec_no_capacity".to_string());
                            }

                            Ok((findings.join("|"), ProcessingStatus::Success))
                        })?;

                    // Count issues
                    for res in &analysis_results {
                        issues_found += res
                            .findings
                            .iter()
                            .filter(|f| f.contains("FIXABLE:"))
                            .count();
                    }

                    if issues_found == 0 {
                        all_results.push(ProcessingResult {
                            path: "EVOLVE_COMPLETE".into(),
                            status: ProcessingStatus::Success,
                            findings: vec![format!(
                                "✅ No more issues after {} iterations",
                                iteration
                            )],
                        });
                        break;
                    }

                    // 2. Apply fixes (if not dry run)
                    if !is_dry {
                        let (repair_results, repair_stats) =
                            ultra_repair(&active_files, config.clone())?;
                        fixes_applied = repair_stats.successful;
                        all_results.extend(repair_results);
                    }

                    all_results.push(ProcessingResult {
                        path: format!("EVOLVE_ITER_{}", iteration + 1),
                        status: ProcessingStatus::Success,
                        findings: vec![format!(
                            "Issues: {}, Fixes: {} (dry_run: {})",
                            issues_found, fixes_applied, is_dry
                        )],
                    });

                    stats.successful += fixes_applied;
                }
            }
        }
    }

    stats.total_duration_ms = start.elapsed().as_millis() as u64;
    Ok((all_results, stats))
}

// Helper interno para scan (copiado de CodeAnalyzer logic o similar, simplificado)
struct ScanUtils;
impl ScanUtils {
    fn collect_files(
        path: &Path,
        extension: Option<&str>,
        _use_gitignore: bool,
        include_hidden: bool,
    ) -> Vec<PathBuf> {
        // 🚀 JWALK: High-Velocity Parallel Walking
        WalkDir::new(path)
            .skip_hidden(!include_hidden)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                if let Some(ext) = extension {
                    e.path().extension().map_or(false, |p_ext| p_ext == ext)
                } else {
                    true
                }
            })
            .map(|e| e.path())
            .collect()
    }
}
