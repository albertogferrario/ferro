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

use std::collections::HashMap;

use crate::http::{HttpResponse, Response};
use ferro_json_ui::{render_to_html, resolve_actions, resolve_errors, JsonUiConfig, JsonUiView};

/// Stateless JSON-UI renderer.
///
/// Provides methods for rendering JSON-UI views as HTML or JSON responses.
/// Follows the same pattern as `Inertia` -- a unit struct with static methods.
pub struct JsonUi;

impl JsonUi {
    /// Clone the view and resolve all action handler names to URLs.
    fn resolve(view: &JsonUiView) -> JsonUiView {
        let mut resolved = view.clone();
        resolve_actions(&mut resolved, |handler| crate::routing::route(handler, &[]));
        resolved
    }

    /// Render a JSON-UI view as an HTML response.
    ///
    /// Returns the view as a full HTML page with rendered component HTML and Tailwind classes.
    /// All action handler references are resolved to URLs before rendering.
    /// The view JSON and data are embedded as `data-view` and `data-props` attributes
    /// on the wrapper div for potential JS hydration.
    pub fn render(view: &JsonUiView, data: &serde_json::Value) -> Response {
        Self::render_with_config(view, data, &JsonUiConfig::new())
    }

    /// Render with custom configuration.
    pub fn render_with_config(
        view: &JsonUiView,
        data: &serde_json::Value,
        config: &JsonUiConfig,
    ) -> Response {
        let view = Self::resolve(view);
        let view_json = serde_json::to_string(&view).map_err(|e| {
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

        let rendered = render_to_html(&view, data);

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
        {rendered}
    </div>
</body>
</html>"#,
            title = html_escape(title),
            head = head,
            body_class = html_escape(&config.body_class),
            view_escaped = html_escape_attr(&view_json),
            data_escaped = html_escape_attr(&data_json),
            rendered = rendered,
        );

        Ok(HttpResponse::text(html)
            .status(200)
            .header("Content-Type", "text/html; charset=utf-8"))
    }

    /// Return the view as JSON (for API consumers or debugging).
    ///
    /// All action handler references are resolved to URLs before output.
    /// If `data` is non-null, it takes precedence over the view's embedded data.
    /// If `data` is null, falls back to the view's `.data` field.
    pub fn render_json(view: &JsonUiView, data: &serde_json::Value) -> Response {
        let view = Self::resolve(view);
        let effective_data = if data.is_null() { &view.data } else { data };
        let payload = serde_json::json!({
            "view": view,
            "data": effective_data,
        });
        Ok(HttpResponse::json(payload))
    }

    /// Clone the view, resolve actions, and populate validation errors on form fields.
    fn resolve_with_errors(view: &JsonUiView, errors: &HashMap<String, Vec<String>>) -> JsonUiView {
        let mut resolved = view.clone();
        resolve_actions(&mut resolved, |handler| crate::routing::route(handler, &[]));
        resolve_errors(&mut resolved, errors);
        resolved.errors = Some(errors.clone());
        resolved
    }

    /// Render a JSON-UI view as HTML with validation errors populated on form fields.
    ///
    /// Same as `render()` but also populates error messages on matching form field
    /// components (Input, Select, Checkbox, Switch) and sets `view.errors`.
    pub fn render_with_errors(
        view: &JsonUiView,
        data: &serde_json::Value,
        errors: &HashMap<String, Vec<String>>,
    ) -> Response {
        Self::render_with_errors_config(view, data, errors, &JsonUiConfig::new())
    }

    /// Render with errors and custom configuration.
    fn render_with_errors_config(
        view: &JsonUiView,
        data: &serde_json::Value,
        errors: &HashMap<String, Vec<String>>,
        config: &JsonUiConfig,
    ) -> Response {
        let view = Self::resolve_with_errors(view, errors);
        let view_json = serde_json::to_string(&view).map_err(|e| {
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

        let rendered = render_to_html(&view, data);

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
        {rendered}
    </div>
</body>
</html>"#,
            title = html_escape(title),
            head = head,
            body_class = html_escape(&config.body_class),
            view_escaped = html_escape_attr(&view_json),
            data_escaped = html_escape_attr(&data_json),
            rendered = rendered,
        );

        Ok(HttpResponse::text(html)
            .status(200)
            .header("Content-Type", "text/html; charset=utf-8"))
    }

    /// Return the view as JSON with validation errors populated on form fields.
    ///
    /// Same as `render_json()` but also populates error messages on matching
    /// form field components and sets `view.errors`.
    pub fn render_json_with_errors(
        view: &JsonUiView,
        data: &serde_json::Value,
        errors: &HashMap<String, Vec<String>>,
    ) -> Response {
        let view = Self::resolve_with_errors(view, errors);
        let effective_data = if data.is_null() { &view.data } else { data };
        let payload = serde_json::json!({
            "view": view,
            "data": effective_data,
        });
        Ok(HttpResponse::json(payload))
    }

    /// Render a JSON-UI view as HTML, accepting a framework `ValidationError` directly.
    ///
    /// Extracts the error map via `.all()` and delegates to `render_with_errors()`.
    /// This is the primary convenience method for handlers.
    pub fn render_validation_error(
        view: &JsonUiView,
        data: &serde_json::Value,
        validation_error: &crate::validation::ValidationError,
    ) -> Response {
        Self::render_with_errors(view, data, validation_error.all())
    }

    /// Return JSON with validation errors from a framework `ValidationError`.
    ///
    /// JSON variant of `render_validation_error()`.
    pub fn render_json_validation_error(
        view: &JsonUiView,
        data: &serde_json::Value,
        validation_error: &crate::validation::ValidationError,
    ) -> Response {
        Self::render_json_with_errors(view, data, validation_error.all())
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
    use ferro_json_ui::{
        Action, ButtonProps, ButtonVariant, CardProps, Component, ComponentNode, HttpMethod, Size,
    };

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
                    footer: vec![],
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

    #[test]
    fn render_json_uses_explicit_data_over_embedded() {
        let view = sample_view().data(serde_json::json!({"embedded": true}));
        let explicit_data = serde_json::json!({"explicit": true});
        let result = JsonUi::render_json(&view, &explicit_data);

        let response = ok_response(result);
        let hyper = response.into_hyper();
        let body = format!("{:?}", hyper.into_body());

        // Explicit data should be used, not the embedded one
        assert!(body.contains("explicit"));
        // The view's embedded data is in the "view" key (part of the serialized view)
        assert!(body.contains("embedded"));
    }

    #[test]
    fn render_json_falls_back_to_embedded_data() {
        let view = sample_view().data(serde_json::json!({"embedded": true}));
        let null_data = serde_json::Value::Null;
        let result = JsonUi::render_json(&view, &null_data);

        let response = ok_response(result);
        let hyper = response.into_hyper();
        let body = format!("{:?}", hyper.into_body());

        // Should use the view's embedded data
        assert!(body.contains("embedded"));
    }

    #[test]
    fn render_resolves_action_urls() {
        // Register a test route name -> path mapping.
        crate::routing::register_route_name("users.index", "/users");

        let view = JsonUiView::new().title("Users").component(ComponentNode {
            key: "btn".to_string(),
            component: Component::Button(ButtonProps {
                label: "List Users".to_string(),
                variant: ButtonVariant::Default,
                size: Size::Default,
                disabled: None,
                icon: None,
                icon_position: None,
            }),
            action: Some(Action {
                handler: "users.index".to_string(),
                url: None,
                method: HttpMethod::Get,
                confirm: None,
                on_success: None,
                on_error: None,
            }),
            visibility: None,
        });

        // render_json should resolve action URLs.
        let result = JsonUi::render_json(&view, &serde_json::json!({}));
        let body = response_body(ok_response(result));
        assert!(
            body.contains("/users"),
            "render_json output should contain the resolved URL"
        );

        // render (HTML) should also resolve action URLs.
        let result = JsonUi::render(&view, &serde_json::json!({}));
        let body = response_body(ok_response(result));
        assert!(
            body.contains("/users"),
            "render output should contain the resolved URL"
        );

        // Original view must not be mutated (clone semantics).
        assert_eq!(
            view.components[0].action.as_ref().unwrap().url,
            None,
            "original view should not be mutated"
        );
    }

    #[test]
    fn render_without_actions_still_works() {
        // Verify views with no actions render without issues (no regression).
        let view = sample_view();
        let data = serde_json::json!({"items": [1, 2]});

        let result = JsonUi::render(&view, &data);
        assert!(result.is_ok());

        let result = JsonUi::render_json(&view, &data);
        assert!(result.is_ok());
    }

    // -----------------------------------------------------------------------
    // render_with_errors tests
    // -----------------------------------------------------------------------

    use ferro_json_ui::{FormProps, InputProps, InputType};
    use std::collections::HashMap;

    fn form_view_with_inputs() -> JsonUiView {
        JsonUiView::new()
            .title("Create User")
            .component(ComponentNode {
                key: "form".to_string(),
                component: Component::Form(FormProps {
                    action: Action {
                        handler: "users.store".to_string(),
                        url: None,
                        method: HttpMethod::Post,
                        confirm: None,
                        on_success: None,
                        on_error: None,
                    },
                    fields: vec![
                        ComponentNode {
                            key: "name-input".to_string(),
                            component: Component::Input(InputProps {
                                field: "name".to_string(),
                                label: "Name".to_string(),
                                input_type: InputType::Text,
                                placeholder: None,
                                required: None,
                                disabled: None,
                                error: None,
                                description: None,
                                default_value: None,
                                data_path: None,
                            }),
                            action: None,
                            visibility: None,
                        },
                        ComponentNode {
                            key: "email-input".to_string(),
                            component: Component::Input(InputProps {
                                field: "email".to_string(),
                                label: "Email".to_string(),
                                input_type: InputType::Email,
                                placeholder: None,
                                required: None,
                                disabled: None,
                                error: None,
                                description: None,
                                default_value: None,
                                data_path: None,
                            }),
                            action: None,
                            visibility: None,
                        },
                    ],
                    method: None,
                }),
                action: None,
                visibility: None,
            })
    }

    fn make_errors(pairs: &[(&str, &[&str])]) -> HashMap<String, Vec<String>> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
            .collect()
    }

    #[test]
    fn render_with_errors_populates_form_fields() {
        let view = form_view_with_inputs();
        let errors = make_errors(&[
            ("name", &["Name is required"]),
            ("email", &["Email is invalid"]),
        ]);
        let data = serde_json::json!({});
        let result = JsonUi::render_with_errors(&view, &data, &errors);

        assert!(result.is_ok());
        let body = response_body(ok_response(result));

        // The HTML data-view attribute should contain the error messages.
        assert!(
            body.contains("Name is required"),
            "body should contain 'Name is required'"
        );
        assert!(
            body.contains("Email is invalid"),
            "body should contain 'Email is invalid'"
        );
    }

    #[test]
    fn render_json_with_errors_includes_errors_in_response() {
        let view = form_view_with_inputs();
        let errors = make_errors(&[("name", &["Name is required"])]);
        let data = serde_json::json!({});
        let result = JsonUi::render_json_with_errors(&view, &data, &errors);

        assert!(result.is_ok());
        let body = response_body(ok_response(result));

        // Field-level error on name input.
        assert!(
            body.contains("Name is required"),
            "body should contain field-level error"
        );
        // view.errors map should be present with field entries.
        assert!(
            body.contains("name"),
            "body should contain the error field name"
        );
    }

    #[test]
    fn render_with_errors_empty_map_produces_no_errors() {
        let view = form_view_with_inputs();
        let errors: HashMap<String, Vec<String>> = HashMap::new();
        let data = serde_json::json!({});

        let with_errors = JsonUi::render_with_errors(&view, &data, &errors);
        let without_errors = JsonUi::render(&view, &data);

        assert!(with_errors.is_ok());
        assert!(without_errors.is_ok());

        let body_with = response_body(ok_response(with_errors));
        // With empty errors, form field errors should remain null.
        // The view.errors field will be Some({}) but field errors are None.
        assert!(
            !body_with.contains("Name is required"),
            "empty errors should not produce field-level messages"
        );
    }

    #[test]
    fn render_validation_error_accepts_framework_type() {
        let view = form_view_with_inputs();
        let mut ve = crate::validation::ValidationError::new();
        ve.add("name", "Name is required");
        ve.add("email", "Email must be valid");

        let data = serde_json::json!({});
        let result = JsonUi::render_validation_error(&view, &data, &ve);

        assert!(result.is_ok());
        let body = response_body(ok_response(result));
        assert!(
            body.contains("Name is required"),
            "should contain name error"
        );
        assert!(
            body.contains("Email must be valid"),
            "should contain email error"
        );
    }

    #[test]
    fn render_with_errors_preserves_action_resolution() {
        crate::routing::register_route_name("users.store", "/users");

        let view = form_view_with_inputs();
        let errors = make_errors(&[("name", &["Name is required"])]);
        let data = serde_json::json!({});

        // render_json_with_errors should have both action URL resolved and errors populated.
        let result = JsonUi::render_json_with_errors(&view, &data, &errors);
        assert!(result.is_ok());
        let body = response_body(ok_response(result));

        assert!(body.contains("/users"), "action URL should be resolved");
        assert!(
            body.contains("Name is required"),
            "field errors should be populated"
        );
    }
}
