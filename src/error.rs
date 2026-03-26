use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryPError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization Error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid Directory: {0}")]
    InvalidDirectory(String),

    #[error("Unsupported Operation: {0}")]
    Unsupported(String),

    #[error("Other Error: {0}")]
    Other(String),

    #[error("FFI Error: {0}")]
    Ffi(#[from] crate::ffi::error::FfiError),

    #[error("Join Error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("Anyhow Error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("Invalid Params: {0}")]
    InvalidParams(String),

    #[error("Analysis Error: {0}")]
    AnalysisError(String),

    #[error("Parallel Execution Error: {0}")]
    ParallelError(String),

    #[error("Shared Memory Error: {0}")]
    SharedMemoryError(String),

    #[error("Database Error: {0}")]
    Db(String),

    #[error("SQL Error: {0}")]
    Database(String),

    #[error("Not Found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, MemoryPError>;

// Conversions from String types
impl From<String> for MemoryPError {
    fn from(err: String) -> Self {
        MemoryPError::Other(err)
    }
}

impl From<&str> for MemoryPError {
    fn from(err: &str) -> Self {
        MemoryPError::Other(err.to_string())
    }
}
