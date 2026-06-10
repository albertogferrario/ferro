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
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            app_name: std::env::var("APP_NAME").unwrap_or_else(|_| "Ferro".to_string()),
            app_url: std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost".to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
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
