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
    /// Application name used in the HTML title tag
    pub app_name: String,
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
    /// Path to Vite's manifest.json for resolving hashed asset filenames
    pub manifest_path: String,
    /// Optional page title. When `Some`, overrides `app_name` in `<title>`.
    pub title: Option<String>,
    /// Raw HTML injected into `<head>` before `</head>` (meta, favicon, font tags).
    /// SECURITY: developer-controlled config only — never populate from request data.
    /// Ignored when `html_template` is set (the custom template owns `<head>`).
    pub head_extras: Option<String>,
    /// id attribute of the mount node. Defaults to `"app"`.
    pub mount_id: String,
}

impl InertiaConfig {
    /// Build configuration from environment variables.
    ///
    /// Reads `APP_NAME`, `VITE_DEV_SERVER`, `VITE_ENTRY_POINT`, `INERTIA_VERSION`,
    /// and `APP_ENV`. Mirrors the framework `from_env()` convention.
    pub fn from_env() -> Self {
        let vite_dev_server = std::env::var("VITE_DEV_SERVER")
            .unwrap_or_else(|_| "http://localhost:5173".to_string());

        let is_dev = !matches!(
            std::env::var("APP_ENV").ok().as_deref(),
            Some("production") | Some("staging")
        );

        let app_name = std::env::var("APP_NAME").unwrap_or_else(|_| "Ferro".to_string());

        Self {
            app_name,
            vite_dev_server,
            entry_point: std::env::var("VITE_ENTRY_POINT")
                .unwrap_or_else(|_| "src/main.tsx".to_string()),
            version: std::env::var("INERTIA_VERSION").unwrap_or_else(|_| "1.0".to_string()),
            development: is_dev,
            html_template: None,
            manifest_path: "public/.vite/manifest.json".to_string(),
            title: None,
            head_extras: None,
            mount_id: "app".to_string(),
        }
    }

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

    /// Set the application name used in the HTML title tag.
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = name.into();
        self
    }

    /// Set the path to Vite's manifest.json.
    pub fn manifest_path(mut self, path: impl Into<String>) -> Self {
        self.manifest_path = path.into();
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

    /// Override the `<title>` tag. When `None`, falls back to `app_name`.
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }

    /// Raw HTML injected into `<head>` before `</head>`.
    /// Ignored when `html_template` is set.
    pub fn head_extras(mut self, h: impl Into<String>) -> Self {
        self.head_extras = Some(h.into());
        self
    }

    /// Set the mount node id (default `"app"`).
    pub fn mount_id(mut self, id: impl Into<String>) -> Self {
        self.mount_id = id.into();
        self
    }
}

impl Default for InertiaConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_reads_defaults() {
        // CI does not set VITE_ENTRY_POINT / INERTIA_VERSION, so defaults apply.
        let c = InertiaConfig::from_env();
        assert_eq!(c.entry_point, "src/main.tsx");
        assert_eq!(c.version, "1.0");
        assert_eq!(c.mount_id, "app");
    }

    #[test]
    fn new_fields_default_to_none() {
        let c = InertiaConfig::from_env();
        assert!(c.title.is_none());
        assert!(c.head_extras.is_none());
    }

    #[test]
    fn builders_set_new_fields() {
        let c = InertiaConfig::new()
            .title("My App")
            .head_extras(r#"<link rel="icon" href="/favicon.ico">"#)
            .mount_id("root");
        assert_eq!(c.title.as_deref(), Some("My App"));
        assert_eq!(
            c.head_extras.as_deref(),
            Some(r#"<link rel="icon" href="/favicon.ico">"#)
        );
        assert_eq!(c.mount_id, "root");
    }

    #[test]
    fn default_equals_from_env_shape() {
        let a = InertiaConfig::default();
        let b = InertiaConfig::from_env();
        assert_eq!(a.entry_point, b.entry_point);
        assert_eq!(a.version, b.version);
        assert_eq!(a.mount_id, b.mount_id);
    }
}
