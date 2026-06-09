/// Errors that can occur in ferro-ai operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Configuration error (missing env var or invalid value).
    #[error("ai config error: {0}")]
    Config(String),

    /// HTTP provider error with optional status code for retry logic.
    ///
    /// The `message` field carries only provider response text.
    /// It MUST NOT contain the API key or auth header.
    #[error("ai provider error ({status:?}): {message}")]
    Provider {
        /// HTTP status code if the failure carried one. Network errors have `None`.
        status: Option<u16>,
        /// Provider-supplied error message. Must not contain the API key or auth header.
        message: String,
    },

    /// This provider does not implement the requested capability.
    ///
    /// Returned by capability methods a provider lacks (e.g. `AnthropicClient::embed()`).
    /// Never panics — callers check for this variant to implement fallback behavior.
    #[error("capability not supported by this provider")]
    Unsupported,

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

    /// Schema normalization failed (malformed schemars output or unexpected structure).
    #[error("schema normalization error: {0}")]
    SchemaError(String),

    /// Tool dispatch loop exceeded the configured `max_iterations` without finishing.
    #[error("tool dispatch exceeded max_iterations ({0})")]
    ToolIterationLimit(u32),

    /// A tool name referenced in a provider response is not registered.
    ///
    /// **Not currently constructed by `ToolRegistry::dispatch`.** The dispatch loop
    /// intentionally surfaces unknown tool names to the LLM as a `ToolError` message
    /// (model-recoverable, per D-13/SC#6) rather than aborting the loop. This variant
    /// is reserved for future direct-dispatch helpers (e.g. a `dispatch_single` that
    /// calls one tool by name and must distinguish "not registered" from handler errors).
    /// Callers pattern-matching on `Error` should not expect this variant to be
    /// reachable from `dispatch` in the current implementation.
    #[error("tool not found: {0}")]
    ToolNotFound(String),

    /// Request timed out after all retries were exhausted.
    #[error("classification request timed out after retries")]
    Timeout,

    /// Confirmation store operation failed.
    #[error("confirmation store error: {0}")]
    StoreError(String),

    /// sqlx database error from the `pgvector` store.
    ///
    /// Only reachable when the `pgvector` feature is enabled and
    /// `PgVectorStore::store` or `PgVectorStore::nearest` is called.
    /// The message is `sqlx::Error::to_string()` — it does not contain
    /// embedding data (`f32` arrays are not included in sqlx error messages).
    #[error("pgvector store error: {0}")]
    Sqlx(String),
}

impl Error {
    /// Returns `true` for errors that should trigger a retry.
    ///
    /// Permanent HTTP errors (400, 401, 403, 404, 422) are not retried.
    /// Transient errors (429, 500, 503, 529) and network errors (`status: None`) are retried.
    /// All non-`Provider` variants (including `Unsupported` and `Timeout`) return `false`.
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Provider {
                status: Some(s), ..
            } => !matches!(s, 400 | 401 | 403 | 404 | 422),
            Error::Provider { status: None, .. } => true, // network error → retry
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_is_retryable() {
        assert!(!Error::Provider {
            status: Some(400),
            message: "".into()
        }
        .is_retryable());
        assert!(!Error::Provider {
            status: Some(401),
            message: "".into()
        }
        .is_retryable());
        assert!(!Error::Provider {
            status: Some(403),
            message: "".into()
        }
        .is_retryable());
        assert!(!Error::Provider {
            status: Some(404),
            message: "".into()
        }
        .is_retryable());
        assert!(!Error::Provider {
            status: Some(422),
            message: "".into()
        }
        .is_retryable());
        assert!(Error::Provider {
            status: Some(429),
            message: "".into()
        }
        .is_retryable());
        assert!(Error::Provider {
            status: Some(500),
            message: "".into()
        }
        .is_retryable());
        assert!(Error::Provider {
            status: Some(503),
            message: "".into()
        }
        .is_retryable());
        assert!(Error::Provider {
            status: Some(529),
            message: "".into()
        }
        .is_retryable());
        assert!(Error::Provider {
            status: None,
            message: "".into()
        }
        .is_retryable());
        assert!(!Error::Unsupported.is_retryable());
        assert!(!Error::Timeout.is_retryable());
    }
}
