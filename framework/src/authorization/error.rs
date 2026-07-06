//! Authorization error types.

use crate::http::{HttpResponse, Response};
use std::fmt;

/// Error returned when authorization fails.
#[derive(Debug, Clone)]
pub struct AuthorizationError {
    /// The ability that was being checked.
    pub ability: String,
    /// Optional message explaining the denial.
    pub message: Option<String>,
    /// HTTP status code to return.
    pub status: u16,
}

impl AuthorizationError {
    /// Create a new authorization error.
    pub fn new(ability: impl Into<String>) -> Self {
        Self {
            ability: ability.into(),
            message: None,
            status: 403,
        }
    }

    /// Create an authorization error with a message.
    pub fn with_message(ability: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            ability: ability.into(),
            message: Some(message.into()),
            status: 403,
        }
    }

    /// Create an authorization error that appears as 404.
    pub fn not_found(ability: impl Into<String>) -> Self {
        Self {
            ability: ability.into(),
            message: None,
            status: 404,
        }
    }

    /// Set a custom HTTP status code.
    pub fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    /// Get the error message or a default.
    pub fn message_or_default(&self) -> String {
        self.message
            .clone()
            .unwrap_or_else(|| "This action is unauthorized.".to_string())
    }
}

impl fmt::Display for AuthorizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(msg) => write!(f, "Authorization failed for '{}': {}", self.ability, msg),
            None => write!(f, "Authorization failed for '{}'", self.ability),
        }
    }
}

impl std::error::Error for AuthorizationError {}

impl From<AuthorizationError> for HttpResponse {
    fn from(err: AuthorizationError) -> HttpResponse {
        let message = err.message_or_default();
        let body = serde_json::json!({
            "message": message
        });
        HttpResponse::json(body).status(err.status)
    }
}

impl From<AuthorizationError> for Response {
    fn from(err: AuthorizationError) -> Response {
        Err(HttpResponse::from(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_error() {
        let error = AuthorizationError::new("update");
        assert_eq!(error.ability, "update");
        assert_eq!(error.status, 403);
        assert!(error.message.is_none());
    }

    #[test]
    fn test_error_with_message() {
        let error = AuthorizationError::with_message("delete", "You do not own this resource");
        assert_eq!(error.ability, "delete");
        assert_eq!(
            error.message,
            Some("You do not own this resource".to_string())
        );
    }

    #[test]
    fn test_not_found_error() {
        let error = AuthorizationError::not_found("view");
        assert_eq!(error.status, 404);
    }

    #[test]
    fn test_display() {
        let error = AuthorizationError::with_message("update", "Forbidden");
        assert_eq!(
            error.to_string(),
            "Authorization failed for 'update': Forbidden"
        );
    }

    #[test]
    fn test_message_or_default() {
        let error = AuthorizationError::new("test");
        assert_eq!(error.message_or_default(), "This action is unauthorized.");

        let error = AuthorizationError::with_message("test", "Custom message");
        assert_eq!(error.message_or_default(), "Custom message");
    }
}
