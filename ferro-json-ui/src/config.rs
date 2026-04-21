//! Configuration for JSON-UI rendering.
//!
//! Controls rendering behavior such as Tailwind CDN inclusion,
//! custom head content injection, default body CSS classes, and
//! stylesheet URLs emitted as `<link>` tags in the HTML `<head>`.

/// Configuration for JSON-UI HTML rendering.
///
/// # Example
///
/// ```rust
/// use ferro_json_ui::JsonUiConfig;
///
/// // Default: framework-served base CSS, no Tailwind CDN.
/// let default_config = JsonUiConfig::new();
/// assert_eq!(default_config.stylesheet_urls, vec!["/_ferro/ferro-base.css"]);
/// assert!(!default_config.tailwind_cdn);
///
/// // Opt into CDN for dev, and override stylesheet list:
/// let dev_config = JsonUiConfig::new()
///     .tailwind_cdn(true)
///     .stylesheet_urls(vec!["/custom.css".to_string()]);
/// ```
#[derive(Debug, Clone, schemars::JsonSchema)]
pub struct JsonUiConfig {
    /// Include Tailwind CDN link in rendered HTML (dev convenience).
    ///
    /// Default: `false`. Set to `true` to load the Tailwind v4 browser runtime
    /// from cdn.jsdelivr.net. The runtime is a development convenience per
    /// Tailwind's docs — it does not work reliably on Safari/WebKit. Production
    /// apps rely on the pre-built base CSS served via `stylesheet_urls`.
    pub tailwind_cdn: bool,

    /// Stylesheet URLs emitted as `<link rel="stylesheet" href="...">` in `<head>`,
    /// in order.
    ///
    /// Default: `vec!["/_ferro/ferro-base.css".to_string()]` — the framework-served
    /// pre-built base CSS. Apps override this list to inject additional stylesheets
    /// (e.g., app-level theme token files) or to drop the default entirely.
    pub stylesheet_urls: Vec<String>,

    /// Custom content to inject into the `<head>` element.
    pub custom_head: Option<String>,
    /// Default CSS classes for the `<body>` element.
    pub body_class: String,
}

impl Default for JsonUiConfig {
    fn default() -> Self {
        Self {
            tailwind_cdn: false,
            stylesheet_urls: vec!["/_ferro/ferro-base.css".to_string()],
            custom_head: None,
            body_class: "bg-background text-text font-sans".to_string(),
        }
    }
}

impl JsonUiConfig {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable Tailwind CDN inclusion.
    pub fn tailwind_cdn(mut self, enabled: bool) -> Self {
        self.tailwind_cdn = enabled;
        self
    }

    /// Replace the stylesheet URL list.
    ///
    /// Each URL emits a `<link rel="stylesheet" href="...">` in `<head>`, in order.
    /// Default: `["/_ferro/ferro-base.css"]`.
    ///
    /// Pass an empty `Vec` to disable the framework-served base CSS.
    pub fn stylesheet_urls(mut self, urls: Vec<String>) -> Self {
        self.stylesheet_urls = urls;
        self
    }

    /// Set custom content to inject into the `<head>` element.
    pub fn custom_head(mut self, head: impl Into<String>) -> Self {
        self.custom_head = Some(head.into());
        self
    }

    /// Set the default CSS classes for the `<body>` element.
    pub fn body_class(mut self, class: impl Into<String>) -> Self {
        self.body_class = class.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_tailwind_cdn_false_and_default_stylesheet_urls() {
        let c = JsonUiConfig::default();
        assert!(!c.tailwind_cdn, "tailwind_cdn default must be false");
        assert_eq!(
            c.stylesheet_urls,
            vec!["/_ferro/ferro-base.css".to_string()],
            "default stylesheet_urls must be exactly [\"/_ferro/ferro-base.css\"]"
        );
    }

    #[test]
    fn stylesheet_urls_builder_replaces_entire_list() {
        let c =
            JsonUiConfig::new().stylesheet_urls(vec!["/a.css".to_string(), "/b.css".to_string()]);
        assert_eq!(c.stylesheet_urls, vec!["/a.css", "/b.css"]);
        assert!(
            !c.stylesheet_urls
                .contains(&"/_ferro/ferro-base.css".to_string()),
            "builder must replace the default, not append"
        );
    }

    #[test]
    fn stylesheet_urls_builder_accepts_empty_vec() {
        let c = JsonUiConfig::new().stylesheet_urls(vec![]);
        assert!(c.stylesheet_urls.is_empty());
    }

    #[test]
    fn json_schema_derives_with_new_field() {
        // Proves schemars::JsonSchema derive still compiles after adding Vec<String>.
        let _schema = schemars::schema_for!(JsonUiConfig);
    }
}
