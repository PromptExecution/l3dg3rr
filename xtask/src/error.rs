use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpbError {
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("ZIP: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("binary not found: {path}")]
    BinaryNotFound { path: std::path::PathBuf },

    #[error("invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("publish failed: {0}")]
    PublishFailed(String),
}
