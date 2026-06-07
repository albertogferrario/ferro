//! Deployment configuration.

/// Deployment system configuration.
#[derive(Debug, Clone, Default)]
pub struct DeploymentConfig {
    /// Domain for preview subdomain URLs (`DEPLOYMENT_PREVIEW_DOMAIN` env var).
    pub preview_domain: Option<String>,
}

impl DeploymentConfig {
    /// Create configuration from environment variables.
    ///
    /// Reads:
    /// - `DEPLOYMENT_PREVIEW_DOMAIN`: domain for preview URLs (optional)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_deployments::DeploymentConfig;
    ///
    /// let config = DeploymentConfig::from_env();
    /// ```
    pub fn from_env() -> Self {
        Self {
            preview_domain: std::env::var("DEPLOYMENT_PREVIEW_DOMAIN").ok(),
        }
    }

    /// Set the preview domain.
    pub fn with_preview_domain(mut self, domain: impl Into<String>) -> Self {
        self.preview_domain = Some(domain.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    /// Guard that removes environment variables on drop, ensuring cleanup even on panic.
    struct EnvGuard {
        vars: Vec<String>,
    }

    impl EnvGuard {
        fn new() -> Self {
            Self { vars: Vec::new() }
        }

        fn also_set(&mut self, key: &str, value: &str) {
            std::env::set_var(key, value);
            self.vars.push(key.to_string());
        }

        fn also_remove(&mut self, key: &str) {
            std::env::remove_var(key);
            self.vars.push(key.to_string());
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for var in &self.vars {
                std::env::remove_var(var);
            }
        }
    }

    #[test]
    #[serial]
    fn preview_domain_from_env() {
        let mut guard = EnvGuard::new();
        guard.also_set("DEPLOYMENT_PREVIEW_DOMAIN", "preview.example.test");

        let config = DeploymentConfig::from_env();
        assert_eq!(config.preview_domain, Some("preview.example.test".to_string()));
    }

    #[test]
    #[serial]
    fn preview_domain_none_when_unset() {
        let mut guard = EnvGuard::new();
        guard.also_remove("DEPLOYMENT_PREVIEW_DOMAIN");

        let config = DeploymentConfig::from_env();
        assert_eq!(config.preview_domain, None);
    }
}
