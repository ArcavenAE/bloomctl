use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BloomctlError {
    #[error("authentication: {0}")]
    Auth(String),

    #[error("operation '{0}' not found in spec")]
    UnknownOperation(String),

    #[error("missing required parameter '{0}' for operation '{1}'")]
    MissingParam(String, String),

    #[error("invalid parameter '{0}': {1}")]
    InvalidParam(String, String),

    #[error(
        "write-guard: operation '{operation}' is {method} (mutating) and bloomctl is \
         read-only by default against the live tenant. To proceed deliberately, pass \
         --allow-write, set BLOOMCTL_ALLOW_WRITE=1, or persist \
         `bloomctl config set allow_writes true`."
    )]
    WriteGuard { operation: String, method: String },

    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },

    #[error("network: {0}")]
    Network(String),

    #[error("spec: {0}")]
    Spec(String),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, BloomctlError>;

impl From<reqwest::Error> for BloomctlError {
    fn from(e: reqwest::Error) -> Self {
        BloomctlError::Network(e.to_string())
    }
}
