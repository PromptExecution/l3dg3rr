use thiserror::Error;

#[derive(Debug, Error)]
pub enum XeroError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("OAuth2 error: {0}")]
    Auth(String),

    #[error("No active Xero tenant. Run xero auth flow first.")]
    NoTenant,

    #[error("Token expired and refresh failed: {0}")]
    TokenExpired(String),

    #[error("Xero API error {status}: {message}")]
    ApiError { status: u16, message: String },

    #[error("Not authenticated. Call get_auth_url() and exchange_code() first.")]
    NotAuthenticated,
}

pub type XeroResult<T> = Result<T, XeroError>;
