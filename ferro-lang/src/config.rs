/// Type-safe localization configuration.
///
/// Reads from environment variables with sensible defaults.
/// Follows the same pattern as `AppConfig` and `ServerConfig`.
///
/// | Variable | Default | Description |
/// |----------|---------|-------------|
/// | `APP_LOCALE` | `"en"` | Default locale |
/// | `APP_FALLBACK_LOCALE` | `"en"` | Fallback when key missing in requested locale |
/// | `LANG_PATH` | `"lang"` | Directory containing translation files |
#[derive(Debug, Clone)]
pub struct LangConfig {
    /// Default locale identifier (e.g. `"en"`, `"es"`).
    pub locale: String,
    /// Fallback locale used when a key is missing in the requested locale.
    pub fallback_locale: String,
    /// Path to the directory containing `{locale}/*.json` translation files.
    pub path: String,
}

impl LangConfig {
    /// Build config from environment variables with defaults.
    pub fn from_env() -> Self {
        Self {
            locale: std::env::var("APP_LOCALE").unwrap_or_else(|_| "en".to_string()),
            fallback_locale: std::env::var("APP_FALLBACK_LOCALE")
                .unwrap_or_else(|_| "en".to_string()),
            path: std::env::var("LANG_PATH").unwrap_or_else(|_| "lang".to_string()),
        }
    }

    /// Create a builder for customizing config.
    pub fn builder() -> LangConfigBuilder {
        LangConfigBuilder::default()
    }
}

impl Default for LangConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Builder for `LangConfig`.
///
/// Unset fields fall back to environment variables (via `from_env()`).
#[derive(Default)]
pub struct LangConfigBuilder {
    locale: Option<String>,
    fallback_locale: Option<String>,
    path: Option<String>,
}

impl LangConfigBuilder {
    /// Set the default locale.
    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = Some(locale.into());
        self
    }

    /// Set the fallback locale.
    pub fn fallback_locale(mut self, fallback: impl Into<String>) -> Self {
        self.fallback_locale = Some(fallback.into());
        self
    }

    /// Set the translation files directory path.
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Build the `LangConfig`, filling unset fields from environment.
    pub fn build(self) -> LangConfig {
        let default = LangConfig::from_env();
        LangConfig {
            locale: self.locale.unwrap_or(default.locale),
            fallback_locale: self.fallback_locale.unwrap_or(default.fallback_locale),
            path: self.path.unwrap_or(default.path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_returns_defaults() {
        // Clear any env vars that might interfere
        std::env::remove_var("APP_LOCALE");
        std::env::remove_var("APP_FALLBACK_LOCALE");
        std::env::remove_var("LANG_PATH");

        let config = LangConfig::from_env();
        assert_eq!(config.locale, "en");
        assert_eq!(config.fallback_locale, "en");
        assert_eq!(config.path, "lang");
    }

    #[test]
    fn from_env_reads_env_vars() {
        std::env::set_var("APP_LOCALE", "es");
        std::env::set_var("APP_FALLBACK_LOCALE", "fr");
        std::env::set_var("LANG_PATH", "resources/lang");

        let config = LangConfig::from_env();
        assert_eq!(config.locale, "es");
        assert_eq!(config.fallback_locale, "fr");
        assert_eq!(config.path, "resources/lang");

        // Clean up
        std::env::remove_var("APP_LOCALE");
        std::env::remove_var("APP_FALLBACK_LOCALE");
        std::env::remove_var("LANG_PATH");
    }

    #[test]
    fn builder_overrides_fields() {
        std::env::remove_var("APP_LOCALE");
        std::env::remove_var("APP_FALLBACK_LOCALE");
        std::env::remove_var("LANG_PATH");

        let config = LangConfig::builder()
            .locale("pt-br")
            .fallback_locale("en")
            .path("translations")
            .build();

        assert_eq!(config.locale, "pt-br");
        assert_eq!(config.fallback_locale, "en");
        assert_eq!(config.path, "translations");
    }

    #[test]
    fn builder_fills_unset_from_env() {
        std::env::remove_var("APP_LOCALE");
        std::env::remove_var("APP_FALLBACK_LOCALE");
        std::env::remove_var("LANG_PATH");

        let config = LangConfig::builder().locale("de").build();

        assert_eq!(config.locale, "de");
        assert_eq!(config.fallback_locale, "en"); // from default
        assert_eq!(config.path, "lang"); // from default
    }

    #[test]
    fn default_delegates_to_from_env() {
        std::env::remove_var("APP_LOCALE");
        std::env::remove_var("APP_FALLBACK_LOCALE");
        std::env::remove_var("LANG_PATH");

        let config = LangConfig::default();
        assert_eq!(config.locale, "en");
        assert_eq!(config.fallback_locale, "en");
        assert_eq!(config.path, "lang");
    }
}
