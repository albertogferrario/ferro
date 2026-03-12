use thiserror::Error;

/// Errors that can occur during theme loading.
#[derive(Debug, Error)]
pub enum ThemeError {
    /// Filesystem read failure.
    #[error("failed to read theme file: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parse failure when reading theme.json.
    #[error("failed to parse theme.json: {0}")]
    Json(#[from] serde_json::Error),

    /// Theme directory not found at the given path.
    #[error("theme not found: {0}")]
    NotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_error_io_wraps_std_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = ThemeError::Io(io_err);
        assert!(matches!(err, ThemeError::Io(_)));
        assert!(err.to_string().contains("failed to read theme file"));
    }

    #[test]
    fn theme_error_json_wraps_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("{{not json").unwrap_err();
        let err = ThemeError::Json(json_err);
        assert!(matches!(err, ThemeError::Json(_)));
        assert!(err.to_string().contains("failed to parse theme.json"));
    }

    #[test]
    fn theme_error_not_found_contains_path_string() {
        let err = ThemeError::NotFound("/some/path".to_string());
        assert!(matches!(err, ThemeError::NotFound(_)));
        let msg = err.to_string();
        assert!(msg.contains("theme not found"));
        assert!(msg.contains("/some/path"));
    }
}
