use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum MemoryPError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
    #[error("Directory not found: {0}")]
    InvalidDirectory(String),
    #[error("File not found: {0:?}")]
    FileNotFound(PathBuf),
    #[error("Unsupported template: {0}")]
    Unsupported(String),
    #[error("Shared memory error: {0}")]
    SharedMemoryError(String),
    #[error("Analysis error: {0}")]
    AnalysisError(String),
    #[error("Parallel error: {0}")]
    ParallelError(String),
    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, MemoryPError>;
