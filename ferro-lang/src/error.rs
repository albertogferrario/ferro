use thiserror::Error;

/// Errors that can occur during translation loading and lookup.
#[derive(Debug, Error)]
pub enum LangError {
    /// File reading failure.
    #[error("failed to read translation file: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON parse failure.
    #[error("failed to parse translation JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    /// No translation files found in the given path.
    #[error("no translations loaded from the given path")]
    NoTranslationsLoaded,
}
