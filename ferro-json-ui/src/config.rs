//! Configuration for JSON-UI rendering.
//!
//! Controls rendering behavior such as Tailwind CDN inclusion,
//! custom head content injection, and default body CSS classes.

/// Configuration for JSON-UI HTML rendering.
///
/// # Example
///
/// ```rust
/// use ferro_json_ui::JsonUiConfig;
///
/// let config = JsonUiConfig::new()
///     .tailwind_cdn(false)
///     .body_class("bg-surface text-text");
/// ```
#[derive(Debug, Clone, schemars::JsonSchema)]
pub struct JsonUiConfig {
    /// Include Tailwind CDN link in rendered HTML (dev convenience).
    pub tailwind_cdn: bool,
    /// Custom content to inject into the `<head>` element.
    pub custom_head: Option<String>,
    /// Default CSS classes for the `<body>` element.
    pub body_class: String,
}

impl Default for JsonUiConfig {
    fn default() -> Self {
        Self {
            tailwind_cdn: true,
            custom_head: None,
            body_class: "dark bg-background text-text font-sans".to_string(),
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
