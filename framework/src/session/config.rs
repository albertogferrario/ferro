//! Session configuration

use std::time::Duration;

/// Session configuration with dual timeout support (idle + absolute).
///
/// Idle timeout expires sessions after inactivity. Absolute timeout expires sessions
/// after a fixed period since creation, regardless of activity. Both are enforced
/// server-side per OWASP session management guidelines.
#[derive(Clone, Debug)]
pub struct SessionConfig {
    /// Idle timeout: session expires after this duration of inactivity
    pub idle_lifetime: Duration,
    /// Absolute timeout: session expires this duration after creation, regardless of activity
    pub absolute_lifetime: Duration,
    /// Cookie name for the session ID
    pub cookie_name: String,
    /// Cookie path
    pub cookie_path: String,
    /// Whether to set Secure flag on cookie (HTTPS only)
    pub cookie_secure: bool,
    /// Whether to set HttpOnly flag on cookie
    pub cookie_http_only: bool,
    /// SameSite attribute for the cookie
    pub cookie_same_site: String,
    /// Database table name for sessions
    pub table_name: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            idle_lifetime: Duration::from_secs(120 * 60), // 2 hours (120 minutes)
            absolute_lifetime: Duration::from_secs(43200 * 60), // 30 days (43200 minutes)
            cookie_name: "ferro_session".to_string(),
            cookie_path: "/".to_string(),
            cookie_secure: true,
            cookie_http_only: true,
            cookie_same_site: "Lax".to_string(),
            table_name: "sessions".to_string(),
        }
    }
}

impl SessionConfig {
    /// Create a new session config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Load session configuration from environment variables
    ///
    /// Environment variables:
    /// - `SESSION_LIFETIME`: Idle timeout in minutes (default: 120)
    /// - `SESSION_ABSOLUTE_LIFETIME`: Absolute timeout in minutes (default: 43200 / 30 days)
    /// - `SESSION_COOKIE`: Cookie name (default: ferro_session)
    /// - `SESSION_SECURE`: Set Secure flag (default: true)
    /// - `SESSION_PATH`: Cookie path (default: /)
    /// - `SESSION_SAME_SITE`: SameSite attribute (default: Lax)
    pub fn from_env() -> Self {
        let idle_lifetime_minutes: u64 = crate::env_optional("SESSION_LIFETIME")
            .and_then(|s: String| s.parse().ok())
            .unwrap_or(120);

        let absolute_lifetime_minutes: u64 = crate::env_optional("SESSION_ABSOLUTE_LIFETIME")
            .and_then(|s: String| s.parse().ok())
            .unwrap_or(43200);

        let cookie_secure = crate::env_optional("SESSION_SECURE")
            .map(|s: String| s.to_lowercase() == "true" || s == "1")
            .unwrap_or(true);

        Self {
            idle_lifetime: Duration::from_secs(idle_lifetime_minutes * 60),
            absolute_lifetime: Duration::from_secs(absolute_lifetime_minutes * 60),
            cookie_name: crate::env_optional("SESSION_COOKIE")
                .unwrap_or_else(|| "ferro_session".to_string()),
            cookie_path: crate::env_optional("SESSION_PATH").unwrap_or_else(|| "/".to_string()),
            cookie_secure,
            cookie_http_only: true, // Always true for security
            cookie_same_site: crate::env_optional("SESSION_SAME_SITE")
                .unwrap_or_else(|| "Lax".to_string()),
            table_name: "sessions".to_string(),
        }
    }

    /// Set the idle timeout duration
    pub fn idle_lifetime(mut self, duration: Duration) -> Self {
        self.idle_lifetime = duration;
        self
    }

    /// Set the absolute timeout duration
    pub fn absolute_lifetime(mut self, duration: Duration) -> Self {
        self.absolute_lifetime = duration;
        self
    }

    /// Set the cookie name
    pub fn cookie_name(mut self, name: impl Into<String>) -> Self {
        self.cookie_name = name.into();
        self
    }

    /// Set whether the cookie should be secure (HTTPS only)
    pub fn secure(mut self, secure: bool) -> Self {
        self.cookie_secure = secure;
        self
    }
}
