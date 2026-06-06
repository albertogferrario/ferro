use crate::config::env::{env, Environment};

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Application name
    pub name: String,
    /// Current environment
    pub environment: Environment,
    /// Debug mode enabled
    pub debug: bool,
    /// Application URL
    pub url: String,
    /// Cumulative-byte threshold per `key` for `crate::http::Request::inline_budget`.
    /// Default 102_400 (100 KiB). Override via env `INLINE_BUDGET_BYTES`.
    pub inline_budget_threshold_bytes: usize,
}

impl AppConfig {
    /// Build config from environment variables
    pub fn from_env() -> Self {
        Self {
            name: env("APP_NAME", "Ferro Application".to_string()),
            environment: Environment::detect(),
            debug: env("APP_DEBUG", true),
            url: env("APP_URL", "http://localhost:8080".to_string()),
            inline_budget_threshold_bytes: env(
                "INLINE_BUDGET_BYTES",
                crate::telemetry::inline_budget::DEFAULT_INLINE_BUDGET_THRESHOLD_BYTES,
            ),
        }
    }

    /// Create a builder for customizing config
    pub fn builder() -> AppConfigBuilder {
        AppConfigBuilder::default()
    }

    /// Check if debug mode is enabled
    pub fn is_debug(&self) -> bool {
        self.debug
    }

    /// Check if running in production
    pub fn is_production(&self) -> bool {
        self.environment.is_production()
    }

    /// Check if running in development
    pub fn is_development(&self) -> bool {
        self.environment.is_development()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Builder for AppConfig
#[derive(Default)]
pub struct AppConfigBuilder {
    name: Option<String>,
    environment: Option<Environment>,
    debug: Option<bool>,
    url: Option<String>,
    inline_budget_threshold_bytes: Option<usize>,
}

impl AppConfigBuilder {
    /// Set the application name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the environment
    pub fn environment(mut self, env: Environment) -> Self {
        self.environment = Some(env);
        self
    }

    /// Set debug mode
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = Some(debug);
        self
    }

    /// Set the application URL
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Override the inline-budget threshold (default 102_400 bytes / 100 KiB).
    pub fn inline_budget_threshold_bytes(mut self, bytes: usize) -> Self {
        self.inline_budget_threshold_bytes = Some(bytes);
        self
    }

    /// Build the AppConfig
    pub fn build(self) -> AppConfig {
        let default = AppConfig::from_env();
        AppConfig {
            name: self.name.unwrap_or(default.name),
            environment: self.environment.unwrap_or(default.environment),
            debug: self.debug.unwrap_or(default.debug),
            url: self.url.unwrap_or(default.url),
            inline_budget_threshold_bytes: self
                .inline_budget_threshold_bytes
                .unwrap_or(default.inline_budget_threshold_bytes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn inline_budget_threshold_default() {
        std::env::remove_var("INLINE_BUDGET_BYTES");
        let cfg = AppConfig::from_env();
        assert_eq!(cfg.inline_budget_threshold_bytes, 102_400);
    }

    #[test]
    #[serial]
    fn inline_budget_threshold_env_override() {
        std::env::set_var("INLINE_BUDGET_BYTES", "50000");
        let cfg = AppConfig::from_env();
        assert_eq!(cfg.inline_budget_threshold_bytes, 50_000);
        std::env::remove_var("INLINE_BUDGET_BYTES");
    }

    #[test]
    #[serial]
    fn inline_budget_threshold_builder_override() {
        let cfg = AppConfigBuilder::default()
            .inline_budget_threshold_bytes(200_000)
            .build();
        assert_eq!(cfg.inline_budget_threshold_bytes, 200_000);
    }
}
