//! Configuration for Inertia.js integration.

/// Configuration for Inertia.js responses.
///
/// # Example
///
/// ```rust
/// use ferro_inertia::InertiaConfig;
///
/// // Development configuration (default)
/// let config = InertiaConfig::default();
///
/// // Production configuration
/// let config = InertiaConfig::new()
///     .version("1.0.0")
///     .production();
///
/// // Custom Vite dev server
/// let config = InertiaConfig::new()
///     .vite_dev_server("http://localhost:3000")
///     .entry_point("src/app.tsx");
/// ```
#[derive(Debug, Clone)]
pub struct InertiaConfig {
    /// Vite dev server URL (e.g., "http://localhost:5173")
    pub vite_dev_server: String,
    /// Entry point for the frontend (e.g., "src/main.tsx")
    pub entry_point: String,
    /// Asset version for cache busting
    pub version: String,
    /// Whether we're in development mode (use Vite dev server)
    pub development: bool,
    /// Custom HTML template (if None, uses default)
    pub html_template: Option<String>,
}

impl Default for InertiaConfig {
    fn default() -> Self {
        let vite_dev_server = std::env::var("VITE_DEV_SERVER")
            .unwrap_or_else(|_| "http://localhost:5173".to_string());

        let is_dev = !matches!(
            std::env::var("APP_ENV").ok().as_deref(),
            Some("production") | Some("staging")
        );

        Self {
            vite_dev_server,
            entry_point: "src/main.tsx".to_string(),
            version: "1.0".to_string(),
            development: is_dev,
            html_template: None,
        }
    }
}

impl InertiaConfig {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the Vite dev server URL.
    pub fn vite_dev_server(mut self, url: impl Into<String>) -> Self {
        self.vite_dev_server = url.into();
        self
    }

    /// Set the frontend entry point.
    pub fn entry_point(mut self, entry: impl Into<String>) -> Self {
        self.entry_point = entry.into();
        self
    }

    /// Set the asset version for cache busting.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Enable production mode (disables Vite dev server integration).
    pub fn production(mut self) -> Self {
        self.development = false;
        self
    }

    /// Enable development mode (enables Vite dev server integration).
    pub fn development(mut self) -> Self {
        self.development = true;
        self
    }

    /// Set a custom HTML template.
    ///
    /// The template should contain the following placeholders:
    /// - `{page}` - The escaped JSON page data
    /// - `{csrf}` - The CSRF token (optional)
    ///
    /// # Example
    ///
    /// ```rust
    /// use ferro_inertia::InertiaConfig;
    ///
    /// let template = r#"
    /// <!DOCTYPE html>
    /// <html>
    /// <head><title>My App</title></head>
    /// <body>
    ///     <div id="app" data-page="{page}"></div>
    ///     <script src="/app.js"></script>
    /// </body>
    /// </html>
    /// "#;
    ///
    /// let config = InertiaConfig::new()
    ///     .html_template(template);
    /// ```
    pub fn html_template(mut self, template: impl Into<String>) -> Self {
        self.html_template = Some(template.into());
        self
    }
}
