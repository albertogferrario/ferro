/// Errors that can occur in ferro-ai operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Configuration error (missing env var or invalid value).
    #[error("ai config error: {0}")]
    Config(String),

    /// AI provider API call failed.
    #[error("ai provider error: {0}")]
    Provider(String),

    /// Classification returned confidence below the configured threshold.
    #[error("low confidence classification (confidence: {confidence:.2})")]
    LowConfidence {
        /// The best guess returned by the provider.
        best_guess: serde_json::Value,
        /// The confidence score (0.0–1.0).
        confidence: f64,
    },

    /// Failed to deserialize the provider response into the target type.
    #[error("deserialization error: {0}")]
    Deserialization(String),

    /// Request timed out after all retries were exhausted.
    #[error("classification request timed out after retries")]
    Timeout,

    /// Confirmation store operation failed.
    #[error("confirmation store error: {0}")]
    StoreError(String),
}
