//! JSON-UI integration for the Ferro framework.
//!
//! This module bridges the `ferro-json-ui` crate with the framework's
//! HTTP response types, providing `JsonUi::render()` as the primary
//! entry point for JSON-UI handlers.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::{JsonUi, JsonUiView, ComponentNode, Component, CardProps, Response};
//!
//! pub async fn index() -> Response {
//!     let view = JsonUiView::new()
//!         .title("Dashboard")
//!         .component(ComponentNode {
//!             key: "welcome".to_string(),
//!             component: Component::Card(CardProps {
//!                 title: "Welcome".to_string(),
//!                 description: None,
//!                 children: vec![],
//!             }),
//!             action: None,
//!             visibility: None,
//!         });
//!
//!     JsonUi::render(&view, &serde_json::json!({}))
//! }
//! ```

use crate::http::{HttpResponse, Response};
use ferro_json_ui::{JsonUiConfig, JsonUiView};

/// Stateless JSON-UI renderer.
///
/// Provides methods for rendering JSON-UI views as HTML or JSON responses.
/// Follows the same pattern as `Inertia` -- a unit struct with static methods.
pub struct JsonUi;

impl JsonUi {
    /// Render a JSON-UI view as an HTML response.
    ///
    /// Returns the view as a full HTML page with an embedded JSON representation.
    /// The actual component-to-HTML rendering is implemented in Phase 28 (HTML Renderer);
    /// this method produces a scaffold with the view JSON for development and testing.
    pub fn render(view: &JsonUiView, data: &serde_json::Value) -> Response {
        Self::render_with_config(view, data, &JsonUiConfig::new())
    }

    /// Render with custom configuration.
    pub fn render_with_config(
        view: &JsonUiView,
        data: &serde_json::Value,
        config: &JsonUiConfig,
    ) -> Response {
        let view_json = serde_json::to_string(view)
            .map_err(|e| HttpResponse::text(format!("JSON-UI serialization error: {e}")).status(500))?;
        let data_json = serde_json::to_string(data)
            .map_err(|e| HttpResponse::text(format!("JSON-UI data serialization error: {e}")).status(500))?;

        let title = view.title.as_deref().unwrap_or("Ferro");

        let mut head = String::new();
        if config.tailwind_cdn {
            head.push_str(r#"<script src="https://cdn.tailwindcss.com"></script>"#);
        }
        if let Some(custom) = &config.custom_head {
            head.push_str(custom);
        }

        let view_pretty = serde_json::to_string_pretty(view).unwrap_or_default();

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    {head}
</head>
<body class="{body_class}">
    <div id="ferro-json-ui"
         data-view="{view_escaped}"
         data-props="{data_escaped}">
        <!-- JSON-UI placeholder: component rendering implemented in Phase 28 -->
        <pre style="padding: 1rem; font-size: 0.75rem; color: #666;">{view_pretty}</pre>
    </div>
</body>
</html>"#,
            title = html_escape(title),
            head = head,
            body_class = html_escape(&config.body_class),
            view_escaped = html_escape_attr(&view_json),
            data_escaped = html_escape_attr(&data_json),
            view_pretty = html_escape(&view_pretty),
        );

        Ok(HttpResponse::text(html)
            .status(200)
            .header("Content-Type", "text/html; charset=utf-8"))
    }

    /// Return the view as JSON (for API consumers or debugging).
    pub fn render_json(view: &JsonUiView, data: &serde_json::Value) -> Response {
        let payload = serde_json::json!({
            "view": view,
            "data": data,
        });
        Ok(HttpResponse::json(payload))
    }
}

/// Escape characters that are meaningful in HTML text content.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Escape characters for use inside HTML attribute values.
fn html_escape_attr(s: &str) -> String {
    html_escape(s)
}
