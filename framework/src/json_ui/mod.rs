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
        let view_json = serde_json::to_string(view).map_err(|e| {
            HttpResponse::text(format!("JSON-UI serialization error: {e}")).status(500)
        })?;
        let data_json = serde_json::to_string(data).map_err(|e| {
            HttpResponse::text(format!("JSON-UI data serialization error: {e}")).status(500)
        })?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_json_ui::{CardProps, Component, ComponentNode};

    /// Extract the Ok variant from a Response without requiring Debug on HttpResponse.
    fn ok_response(result: Response) -> HttpResponse {
        match result {
            Ok(r) => r,
            Err(_) => panic!("expected Ok response, got Err"),
        }
    }

    fn response_body(response: HttpResponse) -> String {
        let hyper = response.into_hyper();
        let body_bytes = hyper.into_body();
        format!("{:?}", body_bytes)
    }

    fn sample_view() -> JsonUiView {
        JsonUiView::new()
            .title("Test Page")
            .component(ComponentNode {
                key: "card".to_string(),
                component: Component::Card(CardProps {
                    title: "Hello".to_string(),
                    description: Some("A test card".to_string()),
                    children: vec![],
                }),
                action: None,
                visibility: None,
            })
    }

    /// Check that a hyper response contains a Content-Type header with the given value.
    /// Handles the case where multiple Content-Type headers exist (HttpResponse::text()
    /// sets text/plain, then .header() adds the correct one).
    fn has_content_type(
        hyper: &hyper::Response<http_body_util::Full<bytes::Bytes>>,
        expected: &str,
    ) -> bool {
        hyper
            .headers()
            .get_all("Content-Type")
            .iter()
            .any(|v| v.to_str().map(|s| s == expected).unwrap_or(false))
    }

    #[test]
    fn render_produces_valid_html() {
        let view = sample_view();
        let data = serde_json::json!({});
        let result = JsonUi::render(&view, &data);

        assert!(result.is_ok());
        let response = ok_response(result);
        assert_eq!(response.status_code(), 200);

        let hyper = response.into_hyper();
        assert!(has_content_type(&hyper, "text/html; charset=utf-8"));

        let body = format!("{:?}", hyper.into_body());
        assert!(body.contains("<!DOCTYPE html>"));
        assert!(body.contains("Test Page"));
        assert!(body.contains("data-view="));
        assert!(body.contains("data-props="));
    }

    #[test]
    fn render_json_returns_json() {
        let view = sample_view();
        let data = serde_json::json!({"users": [1, 2, 3]});
        let result = JsonUi::render_json(&view, &data);

        assert!(result.is_ok());
        let response = ok_response(result);
        assert_eq!(response.status_code(), 200);

        let hyper = response.into_hyper();
        assert!(has_content_type(&hyper, "application/json"));

        let body = format!("{:?}", hyper.into_body());
        assert!(body.contains("view"));
        assert!(body.contains("data"));
    }

    #[test]
    fn config_tailwind_disabled() {
        let view = sample_view();
        let data = serde_json::json!({});
        let config = JsonUiConfig::new().tailwind_cdn(false);
        let result = JsonUi::render_with_config(&view, &data, &config);

        let body = response_body(ok_response(result));
        assert!(!body.contains("cdn.tailwindcss.com"));
    }

    #[test]
    fn config_custom_head() {
        let view = sample_view();
        let data = serde_json::json!({});
        let config =
            JsonUiConfig::new().custom_head(r#"<link rel="stylesheet" href="/custom.css">"#);
        let result = JsonUi::render_with_config(&view, &data, &config);

        let body = response_body(ok_response(result));
        assert!(body.contains("/custom.css"));
    }

    #[test]
    fn config_body_class() {
        let view = sample_view();
        let data = serde_json::json!({});
        let config = JsonUiConfig::new().body_class("dark bg-black");
        let result = JsonUi::render_with_config(&view, &data, &config);

        let body = response_body(ok_response(result));
        assert!(body.contains("dark bg-black"));
    }

    #[test]
    fn html_escaping_prevents_xss_in_title() {
        let view = JsonUiView::new().title(r#"<script>alert("xss")</script>"#);
        let data = serde_json::json!({});
        let result = JsonUi::render(&view, &data);

        let body = response_body(ok_response(result));
        // The raw script tag must not appear unescaped
        assert!(!body.contains("<script>alert"));
        assert!(body.contains("&lt;script&gt;"));
    }

    #[test]
    fn html_escaping_in_data_attributes() {
        let view = sample_view();
        let data = serde_json::json!({"key": "<img src=x onerror=alert(1)>"});
        let result = JsonUi::render(&view, &data);

        let body = response_body(ok_response(result));
        // Angle brackets must be escaped in attribute values
        assert!(!body.contains("<img src=x"));
        assert!(body.contains("&lt;img"));
    }

    #[test]
    fn empty_view_renders_valid_html() {
        let view = JsonUiView::new();
        let data = serde_json::json!({});
        let result = JsonUi::render(&view, &data);

        assert!(result.is_ok());
        let response = ok_response(result);
        assert_eq!(response.status_code(), 200);

        let body = response_body(response);
        assert!(body.contains("<!DOCTYPE html>"));
        // Default title when none set
        assert!(body.contains("Ferro"));
    }

    #[test]
    fn html_escape_fn_handles_all_special_chars() {
        let input = r#"Hello & "World" <foo> 'bar'"#;
        let escaped = html_escape(input);
        assert_eq!(
            escaped,
            "Hello &amp; &quot;World&quot; &lt;foo&gt; &#x27;bar&#x27;"
        );
    }
}
