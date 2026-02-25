//! Authorization response types.

use std::fmt;

/// The result of an authorization check.
///
/// This enum represents whether an action is allowed or denied,
/// with optional messages and HTTP status codes for denied responses.
#[derive(Debug, Clone)]
pub enum AuthResponse {
    /// The action is allowed.
    Allow,
    /// The action is denied.
    Deny {
        /// Optional message explaining why the action was denied.
        message: Option<String>,
        /// HTTP status code to return (default: 403).
        status: u16,
    },
}

impl AuthResponse {
    /// Create an allow response.
    pub fn allow() -> Self {
        Self::Allow
    }

    /// Create a deny response with a message.
    pub fn deny(message: impl Into<String>) -> Self {
        Self::Deny {
            message: Some(message.into()),
            status: 403,
        }
    }

    /// Create a deny response without a message.
    pub fn deny_silent() -> Self {
        Self::Deny {
            message: None,
            status: 403,
        }
    }

    /// Create a deny response that appears as a 404 Not Found.
    ///
    /// This is useful when you want to hide the existence of a resource
    /// from unauthorized users.
    pub fn deny_as_not_found() -> Self {
        Self::Deny {
            message: None,
            status: 404,
        }
    }

    /// Create a deny response with a custom status code.
    pub fn deny_with_status(message: impl Into<String>, status: u16) -> Self {
        Self::Deny {
            message: Some(message.into()),
            status,
        }
    }

    /// Check if the response allows the action.
    pub fn allowed(&self) -> bool {
        matches!(self, Self::Allow)
    }

    /// Check if the response denies the action.
    pub fn denied(&self) -> bool {
        matches!(self, Self::Deny { .. })
    }

    /// Get the denial message if present.
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Deny { message, .. } => message.as_deref(),
            Self::Allow => None,
        }
    }

    /// Get the HTTP status code for denied responses.
    pub fn status(&self) -> u16 {
        match self {
            Self::Allow => 200,
            Self::Deny { status, .. } => *status,
        }
    }
}

impl From<bool> for AuthResponse {
    fn from(allowed: bool) -> Self {
        if allowed {
            Self::Allow
        } else {
            Self::Deny {
                message: None,
                status: 403,
            }
        }
    }
}

impl fmt::Display for AuthResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allow => write!(f, "Allowed"),
            Self::Deny {
                message: Some(msg), ..
            } => write!(f, "Denied: {msg}"),
            Self::Deny { message: None, .. } => write!(f, "Denied"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allow() {
        let response = AuthResponse::allow();
        assert!(response.allowed());
        assert!(!response.denied());
        assert_eq!(response.status(), 200);
    }

    #[test]
    fn test_deny_with_message() {
        let response = AuthResponse::deny("Not authorized");
        assert!(!response.allowed());
        assert!(response.denied());
        assert_eq!(response.message(), Some("Not authorized"));
        assert_eq!(response.status(), 403);
    }

    #[test]
    fn test_deny_silent() {
        let response = AuthResponse::deny_silent();
        assert!(response.denied());
        assert_eq!(response.message(), None);
        assert_eq!(response.status(), 403);
    }

    #[test]
    fn test_deny_as_not_found() {
        let response = AuthResponse::deny_as_not_found();
        assert!(response.denied());
        assert_eq!(response.status(), 404);
    }

    #[test]
    fn test_from_bool_true() {
        let response: AuthResponse = true.into();
        assert!(response.allowed());
    }

    #[test]
    fn test_from_bool_false() {
        let response: AuthResponse = false.into();
        assert!(response.denied());
    }

    #[test]
    fn test_display() {
        assert_eq!(AuthResponse::allow().to_string(), "Allowed");
        assert_eq!(
            AuthResponse::deny("Forbidden").to_string(),
            "Denied: Forbidden"
        );
        assert_eq!(AuthResponse::deny_silent().to_string(), "Denied");
    }
}
