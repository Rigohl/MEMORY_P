//! simulation_engine.rs - Motor de Simulaciones Bend con soporte GPU/CPU
//! Integra simulaciones paralelas para optimización de parámetros

use crate::error::{MemoryPError, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Resultado de una simulación Bend
#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub name: String,
    pub output: String,
    pub success: bool,
    pub duration_ms: u64,
    pub mode: SimulationMode,
}

/// Modo de ejecución de la simulación
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SimulationMode {
    CpuC,        // bend run-c (compilado a C)
    GpuCuda,     // bend run-cu (CUDA GPU)
    Interpreted, // bend run (interpretado)
}

impl std::fmt::Display for SimulationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimulationMode::CpuC => write!(f, "CPU-C"),
            SimulationMode::GpuCuda => write!(f, "GPU-CUDA"),
            SimulationMode::Interpreted => write!(f, "Interpreted"),
        }
    }
}

/// Ejecuta una simulación Bend via WSL
pub fn run_bend_simulation(
    name: &str,
    logic: &str,
    _params: &serde_json::Value,
    use_gpu: bool,
) -> Result<String> {
    let filename = format!("{}.bend", name);

    // Escribir el archivo .bend
    fs::write(&filename, logic).map_err(|e| MemoryPError::Other(format!("Write failed: {}", e)))?;

    let mode = if use_gpu {
        SimulationMode::GpuCuda
    } else {
        SimulationMode::CpuC
    };

    let result = execute_bend_via_wsl(&filename, mode)?;

    // Cleanup
    let _ = fs::remove_file(&filename);

    Ok(result)
}

/// Ejecuta un archivo Bend existente
pub fn run_bend_file(path: &Path, mode: SimulationMode) -> Result<SimulationResult> {
    let start = std::time::Instant::now();
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let result = execute_bend_via_wsl(path.to_str().unwrap_or(""), mode);

    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(output) => Ok(SimulationResult {
            name,
            output,
            success: true,
            duration_ms,
            mode,
        }),
        Err(e) => Ok(SimulationResult {
            name,
            output: format!("Error: {}", e),
            success: false,
            duration_ms,
            mode,
        }),
    }
}

/// Escanea directorio por archivos .bend y los categoriza
pub fn scan_bend_simulations(dir: &Path) -> Result<Vec<BendSimulation>> {
    if !dir.exists() || !dir.is_dir() {
        return Err(MemoryPError::InvalidDirectory(dir.display().to_string()));
    }

    let mut simulations = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "bend") {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let content = fs::read_to_string(&path).unwrap_or_default();
            let category = categorize_simulation(&name, &content);

            simulations.push(BendSimulation {
                name,
                path: path.clone(),
                category,
                lines: content.lines().count(),
            });
        }
    }

    Ok(simulations)
}

/// Información de una simulación Bend
#[derive(Debug, Clone)]
pub struct BendSimulation {
    pub name: String,
    pub path: std::path::PathBuf,
    pub category: SimulationCategory,
    pub lines: usize,
}

/// Categorías de simulaciones
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SimulationCategory {
    Optimization,
    Benchmark,
    Stress,
    Comparison,
    Other,
}

impl std::fmt::Display for SimulationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimulationCategory::Optimization => write!(f, "optimization"),
            SimulationCategory::Benchmark => write!(f, "benchmark"),
            SimulationCategory::Stress => write!(f, "stress"),
            SimulationCategory::Comparison => write!(f, "comparison"),
            SimulationCategory::Other => write!(f, "other"),
        }
    }
}

fn categorize_simulation(name: &str, content: &str) -> SimulationCategory {
    let name_lower = name.to_lowercase();

    if name_lower.contains("opt") || content.contains("optimize") {
        SimulationCategory::Optimization
    } else if name_lower.contains("bench") || name_lower.contains("perf") {
        SimulationCategory::Benchmark
    } else if name_lower.contains("stress") || name_lower.contains("massive") {
        SimulationCategory::Stress
    } else if name_lower.contains("compare") || name_lower.contains("war") {
        SimulationCategory::Comparison
    } else {
        SimulationCategory::Other
    }
}

fn execute_bend_via_wsl(filename: &str, mode: SimulationMode) -> Result<String> {
    let bend_executable = "bend";
    let mode_arg = match mode {
        SimulationMode::GpuCuda => "run-cu",
        SimulationMode::CpuC => "run-c",
        SimulationMode::Interpreted => "run",
    };

    let cmd_str = format!("{} {} ./{}", bend_executable, mode_arg, filename);

    tracing::info!("🌀 Executing BEND [{}]: {}", mode, cmd_str);

    let output = Command::new("wsl")
        .arg("bash")
        .arg("-l")
        .arg("-c")
        .arg(&cmd_str)
        .output()
        .map_err(|e| MemoryPError::Other(format!("Failed to spawn WSL: {}", e)))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if stderr.contains("command not found") {
            return Err(MemoryPError::Other(
                "Bend not found in WSL. Install with 'cargo install bend-lang' inside WSL.".into(),
            ));
        }

        Err(MemoryPError::Other(format!(
            "Bend Error:\nSTDOUT: {}\nSTDERR: {}",
            String::from_utf8_lossy(&output.stdout),
            stderr
        )))
    }
}

/// Ejecuta múltiples simulaciones en paralelo
pub fn run_batch_simulations(
    simulations: &[BendSimulation],
    mode: SimulationMode,
) -> Vec<SimulationResult> {
    use rayon::prelude::*;

    simulations
        .par_iter()
        .map(|sim| {
            run_bend_file(&sim.path, mode).unwrap_or_else(|e| SimulationResult {
                name: sim.name.clone(),
                output: format!("Error: {}", e),
                success: false,
                duration_ms: 0,
                mode,
            })
        })
        .collect()
}

/// Genera reporte de simulaciones disponibles
pub fn generate_simulation_report(dir: &Path) -> Result<SimulationReport> {
    let simulations = scan_bend_simulations(dir)?;

    let optimization_count = simulations
        .iter()
        .filter(|s| s.category == SimulationCategory::Optimization)
        .count();
    let benchmark_count = simulations
        .iter()
        .filter(|s| s.category == SimulationCategory::Benchmark)
        .count();
    let stress_count = simulations
        .iter()
        .filter(|s| s.category == SimulationCategory::Stress)
        .count();

    Ok(SimulationReport {
        total: simulations.len(),
        optimization_count,
        benchmark_count,
        stress_count,
        simulations,
    })
}

#[derive(Debug)]
pub struct SimulationReport {
    pub total: usize,
    pub optimization_count: usize,
    pub benchmark_count: usize,
    pub stress_count: usize,
    pub simulations: Vec<BendSimulation>,
}
