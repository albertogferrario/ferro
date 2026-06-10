use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("render error: {0}")]
    Render(String),
    /// A caller-supplied filter key is unknown or not filter-eligible. This is a
    /// client parameter problem (maps to JSON-RPC `-32602` Invalid params),
    /// distinct from an internal `Database` failure (`-32603` Internal error).
    #[error("invalid filter: {0}")]
    InvalidFilter(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
