//! App identity for the MCP server, sourced from framework env conventions.

/// Configuration for the MCP server, read from environment variables.
///
/// Mirrors `InertiaConfig::default()` — reads `APP_NAME` and `APP_URL`
/// from the environment. Never hardcodes application identity (CLAUDE.md
/// project-agnostic rule).
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Application name, sourced from `APP_NAME` env var.
    pub app_name: String,
    /// Application URL, sourced from `APP_URL` env var.
    pub app_url: String,
    /// Crate version, sourced from `CARGO_PKG_VERSION` at compile time.
    pub version: String,
    /// TTL for confirmation tokens in seconds.
    /// Range: 300–600 (5–10 min). Default: 300.
    /// Sourced from `CONFIRMATION_TTL_SECS` env var; clamped to 300–600 if out of range.
    pub confirmation_ttl_seconds: u64,
}

/// Strip ASCII control characters (including CR and LF) from an env-sourced value.
///
/// `app_url` and `app_name` flow into HTTP header values (e.g. `WWW-Authenticate`).
/// A `\r` or `\n` in the source would let an operator-injected or misconfigured
/// `APP_URL` split the response or inject headers (CR-01). Sanitizing at the trust
/// boundary — where the env var enters — neutralizes the sink regardless of how the
/// value is later used.
fn sanitize_identity(raw: String) -> String {
    raw.chars().filter(|c| !c.is_ascii_control()).collect()
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            app_name: sanitize_identity(
                std::env::var("APP_NAME").unwrap_or_else(|_| "Ferro".to_string()),
            ),
            app_url: sanitize_identity(
                std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost".to_string()),
            ),
            version: env!("CARGO_PKG_VERSION").to_string(),
            confirmation_ttl_seconds: std::env::var("CONFIRMATION_TTL_SECS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .map(|v| v.clamp(300, 600))
                .unwrap_or(300),
        }
    }
}

impl McpServerConfig {
    /// Build from environment variables. Alias of `default()`, mirrors the
    /// `from_env()` naming used by sibling `ferro-*` config structs.
    pub fn from_env() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::sanitize_identity;

    #[test]
    fn sanitize_strips_crlf_and_control_chars() {
        // CR-01: a CRLF in an env-sourced identity must not survive into a header value.
        let injected = "https://app.example\r\nX-Injected: evil".to_string();
        let cleaned = sanitize_identity(injected);
        assert!(!cleaned.contains('\r'));
        assert!(!cleaned.contains('\n'));
        assert_eq!(cleaned, "https://app.exampleX-Injected: evil");
    }

    #[test]
    fn sanitize_preserves_normal_url() {
        assert_eq!(
            sanitize_identity("https://app.example.com".to_string()),
            "https://app.example.com"
        );
    }
}
