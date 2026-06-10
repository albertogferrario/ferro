//! OAuth configuration, sourced from environment variables.
//!
//! `OAuthConfig::from_env()` fails closed: returns `Err` when `MCP_TOKEN_SECRET`
//! is unset or shorter than 32 bytes (T-13/T-14). `sanitized_app_url()` is
//! secret-free for discovery endpoints that must work pre-auth (Plan 02).

/// Strip ASCII control characters (including CR and LF) from an env-sourced value.
///
/// Env-sourced URLs and names flow into HTTP headers and JSON discovery docs.
/// A `\r` or `\n` would allow header injection (CR-01 analog). Sanitize at
/// the trust boundary where the env var enters.
fn sanitize_identity(raw: String) -> String {
    raw.chars().filter(|c| !c.is_ascii_control()).collect()
}

/// Secret-free APP_URL read for the public discovery endpoints.
///
/// Discovery is pre-auth and must work even when `MCP_TOKEN_SECRET` is unset,
/// so this does NOT go through `from_env()` (which fails closed on the secret).
/// Used by `discovery.rs` (Plan 02 fills the handler body).
#[allow(dead_code)]
pub(crate) fn sanitized_app_url() -> String {
    sanitize_identity(std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost".to_string()))
}

/// OAuth server configuration, read from environment variables.
///
/// Never hardcodes application identity (CLAUDE.md project-agnostic rule).
/// `APP_NAME` and `APP_URL` follow the same convention as `InertiaConfig` and
/// `McpServerConfig`. `MCP_TOKEN_SECRET` is a crate-local env var mirroring
/// the `STRIPE_SECRET_KEY` pattern in `ferro-stripe`.
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// Application name, sourced from `APP_NAME` env var.
    pub app_name: String,
    /// Application URL, sourced from `APP_URL` env var.
    pub app_url: String,
    /// HS256 signing key bytes, sourced from `MCP_TOKEN_SECRET` env var.
    pub token_secret: Vec<u8>,
}

/// Errors returned by `OAuthConfig::from_env()` when it fails closed.
#[derive(Debug, thiserror::Error)]
pub enum OAuthConfigError {
    /// `MCP_TOKEN_SECRET` env var is not set.
    #[error("MCP_TOKEN_SECRET env var not set")]
    MissingSecret,
    /// `MCP_TOKEN_SECRET` is shorter than 32 bytes (256-bit floor for HS256).
    #[error("MCP_TOKEN_SECRET must be at least 32 bytes")]
    SecretTooShort,
}

impl OAuthConfig {
    /// Build from environment variables. Fails closed on missing or short secret.
    ///
    /// Returns `Err(MissingSecret)` when `MCP_TOKEN_SECRET` is unset.
    /// Returns `Err(SecretTooShort)` when `MCP_TOKEN_SECRET` is shorter than 32 bytes.
    pub fn from_env() -> Result<Self, OAuthConfigError> {
        let app_name = sanitize_identity(
            std::env::var("APP_NAME").unwrap_or_else(|_| "Ferro".to_string()),
        );
        let app_url = sanitize_identity(
            std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost".to_string()),
        );
        let secret_str = std::env::var("MCP_TOKEN_SECRET")
            .map_err(|_| OAuthConfigError::MissingSecret)?;
        if secret_str.len() < 32 {
            return Err(OAuthConfigError::SecretTooShort);
        }
        Ok(Self {
            app_name,
            app_url,
            token_secret: secret_str.into_bytes(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_secret_returns_err() {
        std::env::remove_var("MCP_TOKEN_SECRET");
        let result = OAuthConfig::from_env();
        assert!(
            matches!(result, Err(OAuthConfigError::MissingSecret)),
            "expected MissingSecret, got {result:?}"
        );
    }

    #[test]
    fn short_secret_returns_err() {
        std::env::remove_var("MCP_TOKEN_SECRET");
        std::env::set_var("MCP_TOKEN_SECRET", "tooshort");
        let result = OAuthConfig::from_env();
        std::env::remove_var("MCP_TOKEN_SECRET");
        assert!(
            matches!(result, Err(OAuthConfigError::SecretTooShort)),
            "expected SecretTooShort, got {result:?}"
        );
    }

    #[test]
    fn valid_secret_returns_ok_with_bytes() {
        std::env::remove_var("MCP_TOKEN_SECRET");
        let secret = "a_valid_secret_that_is_at_least_32_bytes_long";
        std::env::set_var("MCP_TOKEN_SECRET", secret);
        let result = OAuthConfig::from_env();
        std::env::remove_var("MCP_TOKEN_SECRET");
        let config = result.expect("should succeed with valid secret");
        assert_eq!(config.token_secret, secret.as_bytes());
    }

    #[test]
    fn sanitize_strips_crlf_and_control_chars() {
        // CR-01 analog: CRLF in env-sourced identity must not survive into headers/JSON.
        let injected = "https://app.example\r\nX-Injected: evil".to_string();
        let cleaned = sanitize_identity(injected);
        assert!(!cleaned.contains('\r'));
        assert!(!cleaned.contains('\n'));
        assert_eq!(cleaned, "https://app.exampleX-Injected: evil");
    }

    #[test]
    fn sanitized_app_url_works_without_secret() {
        std::env::remove_var("MCP_TOKEN_SECRET");
        std::env::set_var("APP_URL", "https://test.example.com");
        let url = sanitized_app_url();
        std::env::remove_var("APP_URL");
        assert_eq!(url, "https://test.example.com");
    }
}
