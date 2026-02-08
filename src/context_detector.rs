//! context_detector.rs - Detector Dinámico de Contextos
//!
//! Sistema que detecta automáticamente contextos relevantes
//! antes de realizar acciones, sin intervención externa

use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, info, warn};

use crate::error::{MemoryPError as Error, Result};

/// Tipo de contexto detectado
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContextType {
    /// Contexto de workspace/directorio
    Workspace,
    /// Contexto de archivo
    File,
    /// Contexto de configuración
    Configuration,
    /// Contexto de dependencias
    Dependencies,
    /// Contexto de Git
    Git,
    /// Contexto de sistema
    System,
}

/// Información de contexto detectada
#[derive(Debug, Clone)]
pub struct Context {
    /// Tipo de contexto
    pub context_type: ContextType,
    /// Descripción del contexto
    pub description: String,
    /// Datos relevantes del contexto
    pub data: HashMap<String, String>,
    /// Nivel de relevancia (0-100)
    pub relevance: u8,
}

/// Detector de contextos
pub struct ContextDetector {
    /// Caché de contextos detectados
    context_cache: HashMap<String, Vec<Context>>,
}

impl ContextDetector {
    /// Crea un nuevo detector de contextos
    pub fn new() -> Self {
        info!("🔍 Inicializando Detector de Contextos...");
        Self {
            context_cache: HashMap::new(),
        }
    }

    /// Detecta contextos relevantes automáticamente
    pub async fn detect_contexts(&self) -> Result<Vec<Context>> {
        debug!("🔎 Detectando contextos dinámicamente...");

        let mut contexts = Vec::new();

        // Detectar contexto de workspace
        if let Ok(workspace_ctx) = self.detect_workspace_context().await {
            contexts.push(workspace_ctx);
        }

        // Detectar contexto de configuración
        if let Ok(config_ctx) = self.detect_config_context().await {
            contexts.push(config_ctx);
        }

        // Detectar contexto de Git
        if let Ok(git_ctx) = self.detect_git_context().await {
            contexts.push(git_ctx);
        }

        // Detectar contexto de dependencias
        if let Ok(deps_ctx) = self.detect_dependencies_context().await {
            contexts.push(deps_ctx);
        }

        // Detectar contexto del sistema
        if let Ok(sys_ctx) = self.detect_system_context().await {
            contexts.push(sys_ctx);
        }

        info!("✅ Detectados {} contextos", contexts.len());
        Ok(contexts)
    }

    /// Detecta contexto del workspace actual
    async fn detect_workspace_context(&self) -> Result<Context> {
        debug!("📁 Detectando contexto de workspace...");

        let current_dir = std::env::current_dir()
            .map_err(|e| Error::Other(format!("No se pudo obtener directorio actual: {}", e)))?;

        let mut data = HashMap::new();
        data.insert(
            "path".to_string(),
            current_dir.to_string_lossy().to_string(),
        );

        // Detectar tipo de proyecto
        let project_type = if current_dir.join("Cargo.toml").exists() {
            "Rust"
        } else if current_dir.join("package.json").exists() {
            "Node.js"
        } else if current_dir.join("pyproject.toml").exists()
            || current_dir.join("setup.py").exists()
        {
            "Python"
        } else if current_dir.join("go.mod").exists() {
            "Go"
        } else {
            "Unknown"
        };

        data.insert("project_type".to_string(), project_type.to_string());

        Ok(Context {
            context_type: ContextType::Workspace,
            description: format!("Workspace: {} ({})", current_dir.display(), project_type),
            data,
            relevance: 90,
        })
    }

    /// Detecta contexto de configuración
    async fn detect_config_context(&self) -> Result<Context> {
        debug!("⚙️  Detectando contexto de configuración...");

        let mut data = HashMap::new();
        let config_files = vec![
            "Cargo.toml",
            "package.json",
            "pyproject.toml",
            ".env",
            "config.toml",
            "config.yaml",
        ];

        let mut found_configs = Vec::new();
        for config_file in config_files {
            let path = PathBuf::from(config_file);
            if path.exists() {
                found_configs.push(config_file);
            }
        }

        data.insert("config_files".to_string(), found_configs.join(", "));
        data.insert("count".to_string(), found_configs.len().to_string());

        Ok(Context {
            context_type: ContextType::Configuration,
            description: format!(
                "Archivos de configuración encontrados: {}",
                found_configs.len()
            ),
            data,
            relevance: 80,
        })
    }

    /// Detecta contexto de Git
    async fn detect_git_context(&self) -> Result<Context> {
        debug!("🔀 Detectando contexto de Git...");

        let mut data = HashMap::new();

        // Verificar si es un repositorio Git
        let git_dir = PathBuf::from(".git");
        if !git_dir.exists() {
            return Err(Error::Other("No es un repositorio Git".into()));
        }

        data.insert("is_git_repo".to_string(), "true".to_string());

        // Detectar branch actual (de forma simple)
        if let Ok(head_content) = fs::read_to_string(".git/HEAD").await {
            if let Some(branch) = head_content.strip_prefix("ref: refs/heads/") {
                data.insert("current_branch".to_string(), branch.trim().to_string());
            }
        }

        Ok(Context {
            context_type: ContextType::Git,
            description: "Repositorio Git detectado".to_string(),
            data,
            relevance: 85,
        })
    }

    /// Detecta contexto de dependencias
    async fn detect_dependencies_context(&self) -> Result<Context> {
        debug!("📦 Detectando contexto de dependencias...");

        let mut data = HashMap::new();
        let mut dep_count = 0;

        // Rust dependencies
        if let Ok(cargo_content) = fs::read_to_string("Cargo.toml").await {
            let deps: Vec<&str> = cargo_content
                .lines()
                .filter(|line| line.contains("=") && !line.starts_with("#"))
                .collect();
            dep_count += deps.len();
            data.insert("rust_dependencies".to_string(), deps.len().to_string());
        }

        // Node.js dependencies
        if let Ok(package_content) = fs::read_to_string("package.json").await {
            if package_content.contains("dependencies") {
                data.insert("nodejs_dependencies".to_string(), "present".to_string());
                dep_count += 1;
            }
        }

        data.insert("total_dependency_files".to_string(), dep_count.to_string());

        Ok(Context {
            context_type: ContextType::Dependencies,
            description: format!("Dependencias detectadas en {} archivos", dep_count),
            data,
            relevance: 75,
        })
    }

    /// Detecta contexto del sistema
    async fn detect_system_context(&self) -> Result<Context> {
        debug!("💻 Detectando contexto del sistema...");

        let mut data = HashMap::new();

        // Información del sistema operativo
        data.insert("os".to_string(), std::env::consts::OS.to_string());
        data.insert("arch".to_string(), std::env::consts::ARCH.to_string());

        // Número de CPUs
        let cpu_count = num_cpus::get();
        data.insert("cpu_count".to_string(), cpu_count.to_string());

        Ok(Context {
            context_type: ContextType::System,
            description: format!(
                "Sistema: {} ({}), {} CPUs",
                std::env::consts::OS,
                std::env::consts::ARCH,
                cpu_count
            ),
            data,
            relevance: 70,
        })
    }

    /// Detecta contextos específicos para una operación
    pub async fn detect_contexts_for_operation(&self, operation: &str) -> Result<Vec<Context>> {
        debug!("🎯 Detectando contextos para operación: {}", operation);

        let all_contexts = self.detect_contexts().await?;

        // Filtrar contextos relevantes según la operación
        let relevant_contexts: Vec<Context> = all_contexts
            .into_iter()
            .filter(|ctx| {
                match operation {
                    op if op.contains("build") => {
                        ctx.context_type == ContextType::Dependencies
                            || ctx.context_type == ContextType::Configuration
                    }
                    op if op.contains("git") => ctx.context_type == ContextType::Git,
                    op if op.contains("analyze") => {
                        ctx.context_type == ContextType::Workspace
                            || ctx.context_type == ContextType::File
                    }
                    _ => true, // Por defecto, incluir todos
                }
            })
            .collect();

        info!(
            "✅ {} contextos relevantes para '{}'",
            relevant_contexts.len(),
            operation
        );
        Ok(relevant_contexts)
    }

    /// Valida si un contexto es seguro para operar
    pub fn validate_context_safety(&self, context: &Context) -> Result<bool> {
        debug!("🔒 Validando seguridad del contexto...");

        // Verificar que no estamos en directorios del sistema
        if let Some(path) = context.data.get("path") {
            let unsafe_paths = vec!["/", "/bin", "/sbin", "/usr", "/etc", "/sys"];
            for unsafe_path in unsafe_paths {
                if path.starts_with(unsafe_path) && !path.contains("home") {
                    warn!("⚠️  Contexto potencialmente inseguro: {}", path);
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
}

impl Default for ContextDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_detector_creation() {
        let detector = ContextDetector::new();
        assert_eq!(detector.context_cache.len(), 0);
    }

    #[tokio::test]
    async fn test_detect_workspace_context() {
        let detector = ContextDetector::new();
        let result = detector.detect_workspace_context().await;
        assert!(result.is_ok());

        let context = result.unwrap();
        assert_eq!(context.context_type, ContextType::Workspace);
        assert!(context.data.contains_key("path"));
    }

    #[tokio::test]
    async fn test_detect_system_context() {
        let detector = ContextDetector::new();
        let result = detector.detect_system_context().await;
        assert!(result.is_ok());

        let context = result.unwrap();
        assert_eq!(context.context_type, ContextType::System);
        assert!(context.data.contains_key("cpu_count"));
    }

    #[tokio::test]
    async fn test_detect_contexts() {
        let detector = ContextDetector::new();
        let contexts = detector.detect_contexts().await.unwrap();
        assert!(!contexts.is_empty());
    }

    #[tokio::test]
    async fn test_validate_context_safety() {
        let detector = ContextDetector::new();

        let safe_context = Context {
            context_type: ContextType::Workspace,
            description: "Test".to_string(),
            data: {
                let mut map = HashMap::new();
                map.insert("path".to_string(), "/home/user/project".to_string());
                map
            },
            relevance: 80,
        };

        assert!(detector.validate_context_safety(&safe_context).unwrap());
    }
}
