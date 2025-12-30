//! error.rs - Manejo centralizado de errores con thiserror
//! Define tipos de error custom para MEMORY_P

use std::path::PathBuf;
use thiserror::Error;

/// Error personalizado para MEMORY_P
#[derive(Error, Debug)]
pub enum MemoryPError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Invalid directory: {0}")]
    InvalidDirectory(String),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Error JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Argumentos inválidos: {0}")]
    InvalidParams(String),

    #[error("Unsupported template: {0}")]
    Unsupported(String),

    #[error("Parallel processing error: {0}")]
    ParallelError(String),

    #[allow(dead_code)]
    #[error("Workspace lock error: {0}")]
    LockError(String),

    #[allow(dead_code)]
    #[error("Error de análisis: {0}")]
    AnalysisError(String),

    #[error("Error: {0}")]
    Other(String),
}

/// Alias para Result<T, MemoryPError>
pub type Result<T> = std::result::Result<T, MemoryPError>;

impl From<String> for MemoryPError {
    fn from(s: String) -> Self {
        MemoryPError::Other(s)
    }
}

impl From<&str> for MemoryPError {
    fn from(s: &str) -> Self {
        MemoryPError::Other(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = MemoryPError::FileNotFound(PathBuf::from("test.rs"));
        assert!(err.to_string().contains("test.rs"));
    }

    #[test]
    fn test_error_conversion_from_string() {
        let err: MemoryPError = "test error".into();
        assert!(matches!(err, MemoryPError::Other(_)));
    }
}
