use serde::Deserialize;
use std::fs;

/// Configuración principal de MEMORY_P
#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub parallelism: ParallelismConfig,
    #[serde(default)]
    pub advanced: AdvancedConfig,
    #[serde(default)]
    pub orchestrator: OrchestratorConfig,
}

/// Configuración de paralelismo (Rayon + Tokio)
#[derive(Deserialize, Debug, Clone)]
pub struct ParallelismConfig {
    pub threads: usize,
    pub batch_size: usize,
}

/// Configuración avanzada de I/O y rendimiento
#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct AdvancedConfig {
    /// Timeout para operaciones de archivo (ms)
    pub file_timeout_ms: u64,
    /// Umbral para usar MMAP en lugar de buffered read (bytes)
    pub large_file_threshold: usize,
    /// Activar modo Zero-copy con rkyv
    pub enable_zerocopy: bool,
    /// Activar caché de análisis con SCC
    pub enable_scc_cache: bool,
}

/// Configuración del orquestador Julia
#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct OrchestratorConfig {
    pub auto_analyze: bool,
    pub mcp_port: u16,
    pub bend_enabled: bool,
    pub report_format: String,
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            file_timeout_ms: 8000,
            large_file_threshold: 10 * 1024 * 1024, // 10MB
            enable_zerocopy: true,
            enable_scc_cache: true,
        }
    }
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            auto_analyze: true,
            mcp_port: 4040,
            bend_enabled: true,
            report_format: "json".to_string(),
        }
    }
}

impl AppConfig {
    /// Carga configuración desde memory_p.toml con soporte para todas las secciones
    pub fn load() -> Self {
        let path = "memory_p.toml";
        if let Ok(content) = fs::read_to_string(path) {
            match toml::from_str(&content) {
                Ok(cfg) => {
                    tracing::info!("⚡ Configuración cargada de {}: {:?}", path, cfg);
                    return cfg;
                }
                Err(e) => {
                    tracing::warn!("⚠️ Error al parsear {}: {}, usando defaults.", path, e);
                }
            }
        }

        // Default "Safe"
        Self {
            parallelism: ParallelismConfig {
                threads: num_cpus::get(),
                batch_size: 100,
            },
            advanced: AdvancedConfig::default(),
            orchestrator: OrchestratorConfig::default(),
        }
    }

    /// Retorna configuración optimizada para el motor paralelo
    pub fn to_parallel_config(&self) -> crate::parallel_engine::ParallelConfig {
        crate::parallel_engine::ParallelConfig {
            max_threads: self.parallelism.threads,
            chunk_size: self.parallelism.batch_size,
            _read_buffer_size: 64 * 1024,
            _file_timeout_ms: self.advanced.file_timeout_ms,
            _continue_on_error: true,
            _large_file_threshold: self.advanced.large_file_threshold,
        }
    }
}

/// Singleton global de configuración (Lock-free con lazy_static)
use lazy_static::lazy_static;
lazy_static! {
    pub static ref CONFIG: AppConfig = AppConfig::load();
}
