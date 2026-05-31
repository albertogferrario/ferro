//! JSON-UI integration for the Ferro framework.
//!
//! This module bridges the `ferro-json-ui` crate with the framework's
//! HTTP response types, providing `JsonUi::render()` as the primary
//! entry point for JSON-UI handlers.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::{JsonUi, Spec, Element, Response};
//!
//! pub async fn index() -> Response {
//!     let spec = Spec::builder()
//!         .title("Dashboard")
//!         .element("welcome", Element::new("Card").prop("title", "Welcome"))
//!         .build()
//!         .expect("spec is valid");
//!
//!     JsonUi::render(&spec, &serde_json::json!({}))
//! }
//! ```

use std::collections::HashMap;

use crate::http::{HttpResponse, Response};
use ferro_json_ui::{
    expand_directives, global_catalog, render_layout, render_spec_to_html_with_plugins,
    resolve_actions, resolve_errors, resolve_expressions, JsonUiConfig, LayoutContext, Spec,
};

/// Stateless JSON-UI renderer.
///
/// Provides methods for rendering JSON-UI specs as HTML or JSON responses.
/// Follows the same pattern as `Inertia` -- a unit struct with static methods.
pub struct JsonUi;

impl JsonUi {
    /// Clone the spec, expand `$each` / `$if` directives, resolve action
    /// handler names to URLs, and resolve `$data` / `$template` expression
    /// nodes in element props.
    ///
    /// Pipeline (Phase 163 ordering, locked by 163-03-PLAN):
    /// 1. `expand_directives` — materializes `$each` templates into N clones
    ///    and removes `$if`-falsy elements. Must run FIRST so all downstream
    ///    passes operate on the post-expansion element set.
    /// 2. `resolve_actions` — handler names → URLs on the expanded set.
    /// 3. `resolve_expressions` — `$data` / `$template` markers in props.
    fn resolve(spec: &Spec) -> Spec {
        let mut resolved = spec.clone();
        expand_directives(&mut resolved);
        // D-16: validate AFTER expand_directives so $if-removed elements skip
        // enum validation. Errors are logged at error level; render continues
        // (hard surface available via resolve_with_errors on error paths).
        if let Err(errs) = global_catalog().validate(&resolved) {
            for e in &errs {
                tracing::error!(
                    target: "ferro_json_ui::catalog",
                    error = %e,
                    "render-time catalog validation failed (resolve clean-path)"
                );
            }
        }
        resolve_actions(&mut resolved, |handler| crate::routing::route(handler, &[]));
        resolve_expressions(&mut resolved);
        resolved
    }

    /// Render a JSON-UI spec as an HTML response.
    ///
    /// Returns the spec as a full HTML page with rendered element HTML and Tailwind classes.
    /// All action handler references are resolved to URLs before rendering.
    /// The spec JSON and data are embedded as `data-view` and `data-props` attributes
    /// on the wrapper div for potential JS hydration.
    pub fn render(spec: &Spec, data: &serde_json::Value) -> Response {
        Self::render_with_config(spec, data, &JsonUiConfig::new())
    }

    /// Render with custom configuration.
    pub fn render_with_config(
        spec: &Spec,
        data: &serde_json::Value,
        config: &JsonUiConfig,
    ) -> Response {
        let resolved = Self::resolve(spec);
        Self::build_response(&resolved, data, config)
    }

    /// Build an HTML response from a resolved spec using the layout system.
    ///
    /// Shared implementation for both `render_with_config` and `render_with_errors_config`.
    /// Serializes spec/data, builds head content, renders elements, then dispatches
    /// to the layout registry for the final HTML shell.
    fn build_response(spec: &Spec, data: &serde_json::Value, config: &JsonUiConfig) -> Response {
        let spec_json = serde_json::to_string(spec).map_err(|e| {
            HttpResponse::text(format!("JSON-UI serialization error: {e}")).status(500)
        })?;
        let data_json = serde_json::to_string(data).map_err(|e| {
            HttpResponse::text(format!("JSON-UI data serialization error: {e}")).status(500)
        })?;

        let title_owned: String = match &spec.title {
            None => "Ferro".to_string(),
            Some(ferro_json_ui::TitleBinding::Literal(s)) => s.clone(),
            Some(ferro_json_ui::TitleBinding::Binding(r)) => {
                // Resolve at render time against spec.data using JSON Pointer syntax.
                // Missing path or non-string value falls back to the default.
                spec.data
                    .pointer(&r.data)
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| "Ferro".to_string())
            }
        };
        let title: &str = &title_owned;

        let mut head = String::new();
        // Inter Variable via Bunny Fonts — loaded unconditionally so font renders
        // regardless of whether the Tailwind CDN is active.
        head.push_str(
            "<link rel=\"preconnect\" href=\"https://fonts.bunny.net\">\
             <link href=\"https://fonts.bunny.net/css?family=inter:300,400,500,600,700&display=swap\" rel=\"stylesheet\">",
        );
        // Emit one <link> per configured stylesheet URL, in order.
        // Defaults to the framework-served pre-built base CSS at /_ferro/ferro-base.css.
        // URL values are HTML-escaped before emission into the href attribute — even
        // though the config API is app-controlled (trusted), defense-in-depth prevents
        // attribute-context injection if an app forwards an untrusted value through
        // `.stylesheet_urls(...)`. Same discipline the existing layout code uses for
        // data attributes.
        for url in &config.stylesheet_urls {
            head.push_str(&format!(
                r#"<link rel="stylesheet" href="{}">"#,
                html_escape(url)
            ));
        }
        if config.tailwind_cdn {
            head.push_str(
                r#"<script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4"></script>"#,
            );
        }
        if let Some(custom) = &config.custom_head {
            head.push_str(custom);
        }
        // Inject active theme CSS as a plain <style>. theme.css must contain
        // standard CSS (:root { ... }) — not Tailwind's @theme syntax, which
        // only works under the CDN browser runtime.
        #[cfg(feature = "theme")]
        {
            if let Some(theme) = crate::theme::context::current_theme() {
                head.push_str(&format!("<style>{}</style>", theme.css));
            }
        }

        let result = render_spec_to_html_with_plugins(spec, data);

        // Append plugin CSS assets to the head string.
        let full_head = if result.css_head.is_empty() {
            head
        } else {
            format!("{}{}", head, result.css_head)
        };

        let ctx = LayoutContext {
            title,
            content: &result.html,
            head: &full_head,
            body_class: &config.body_class,
            view_json: &spec_json,
            data_json: &data_json,
            scripts: &result.scripts,
        };

        let layout_name = spec.layout.as_deref();
        let html = render_layout(layout_name, &ctx);

        Ok(HttpResponse::text(html)
            .status(200)
            .header("Content-Type", "text/html; charset=utf-8"))
    }

    /// Load a v2 spec file, merge handler data, and render to HTML.
    ///
    /// Uses the process-level spec cache (`ferro_json_ui::load_cached`).
    /// In development, reloads on mtime change. In production, cached for the
    /// process lifetime.
    pub fn render_file(
        path: impl AsRef<std::path::Path>,
        handler_data: serde_json::Value,
    ) -> Response {
        Self::render_file_with_config(path, handler_data, &JsonUiConfig::new())
    }

    /// Load a v2 spec file and render with custom configuration.
    pub fn render_file_with_config(
        path: impl AsRef<std::path::Path>,
        handler_data: serde_json::Value,
        config: &JsonUiConfig,
    ) -> Response {
        let reload = !crate::Config::is_production();
        let arc_spec = ferro_json_ui::load_cached(path.as_ref(), reload)
            .map_err(|e| HttpResponse::text(format!("Failed to load spec: {e}")).status(500))?;
        let spec = (*arc_spec).clone().merge_data(handler_data);
        let data = spec.data.clone();
        let resolved = Self::resolve(&spec);
        Self::build_response(&resolved, &data, config)
    }

    /// Return the spec as JSON (for API consumers or debugging).
    ///
    /// All action handler references are resolved to URLs before output.
    /// If `data` is non-null, it takes precedence over the spec's embedded data.
    /// If `data` is null, falls back to the spec's `.data` field.
    pub fn render_json(spec: &Spec, data: &serde_json::Value) -> Response {
        let spec = Self::resolve(spec);
        let effective_data = if data.is_null() { &spec.data } else { data };
        let payload = serde_json::json!({
            "spec": spec,
            "data": effective_data,
        });
        Ok(HttpResponse::json(payload))
    }

    /// Clone the spec, expand `$each` / `$if` directives, resolve actions,
    /// resolve `$data` / `$template` expressions, then populate validation
    /// errors on form fields. Same Phase 163 pipeline ordering as
    /// `JsonUi::resolve` — `expand_directives` runs FIRST.
    fn resolve_with_errors(spec: &Spec, errors: &HashMap<String, Vec<String>>) -> Spec {
        let mut resolved = spec.clone();
        expand_directives(&mut resolved);
        // D-16: validate AFTER expand_directives so $if-removed elements skip
        // enum validation. Catalog errors are logged at error level; this path
        // is used for form re-renders where shape errors surface as diagnostics.
        if let Err(errs) = global_catalog().validate(&resolved) {
            for e in &errs {
                tracing::error!(
                    target: "ferro_json_ui::catalog",
                    error = %e,
                    "render-time catalog validation failed (resolve_with_errors)"
                );
            }
        }
        resolve_actions(&mut resolved, |handler| crate::routing::route(handler, &[]));
        resolve_expressions(&mut resolved);
        resolve_errors(&mut resolved, errors);
        resolved
    }

    /// Render a JSON-UI spec as HTML with validation errors populated on form fields.
    ///
    /// Same as `render()` but also populates error messages on matching form field
    /// elements (Input, Select, Checkbox, Switch) via their `field` or `name` prop.
    pub fn render_with_errors(
        spec: &Spec,
        data: &serde_json::Value,
        errors: &HashMap<String, Vec<String>>,
    ) -> Response {
        Self::render_with_errors_config(spec, data, errors, &JsonUiConfig::new())
    }

    /// Render with errors and custom configuration.
    fn render_with_errors_config(
        spec: &Spec,
        data: &serde_json::Value,
        errors: &HashMap<String, Vec<String>>,
        config: &JsonUiConfig,
    ) -> Response {
        let resolved = Self::resolve_with_errors(spec, errors);
        Self::build_response(&resolved, data, config)
    }

    /// Return the spec as JSON with validation errors populated on form fields.
    ///
    /// Same as `render_json()` but also populates error messages on matching
    /// form field elements.
    pub fn render_json_with_errors(
        spec: &Spec,
        data: &serde_json::Value,
        errors: &HashMap<String, Vec<String>>,
    ) -> Response {
        let spec = Self::resolve_with_errors(spec, errors);
        let effective_data = if data.is_null() { &spec.data } else { data };
        let payload = serde_json::json!({
            "spec": spec,
            "data": effective_data,
        });
        Ok(HttpResponse::json(payload))
    }

    /// Render a JSON-UI spec as HTML, accepting a framework `ValidationError` directly.
    ///
    /// Extracts the error map via `.all()` and delegates to `render_with_errors()`.
    /// This is the primary convenience method for handlers.
    pub fn render_validation_error(
        spec: &Spec,
        data: &serde_json::Value,
        validation_error: &crate::validation::ValidationError,
    ) -> Response {
        Self::render_with_errors(spec, data, validation_error.all())
    }

    /// Return JSON with validation errors from a framework `ValidationError`.
    ///
    /// JSON variant of `render_validation_error()`.
    pub fn render_json_validation_error(
        spec: &Spec,
        data: &serde_json::Value,
        validation_error: &crate::validation::ValidationError,
    ) -> Response {
        Self::render_json_with_errors(spec, data, validation_error.all())
    }
}

/// Escape characters that are meaningful in HTML text/attribute context.
///
/// Used by head-assembly code (e.g., stylesheet URL href emission) to
/// prevent attribute-context injection via URL values. Covers the five
/// characters called out by the OWASP HTML entity encoding guidance:
/// `&`, `<`, `>`, `"`, `'`.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_json_ui::{Action, Element, HttpMethod, Spec};

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
        format!("{body_bytes:?}")
    }

    /// Extract the raw HTML body string from an HttpResponse.
    ///
    /// Unlike `response_body`, this returns the actual HTML content rather than
    /// a Debug-formatted representation. Use for assertions on HTML tag content.
    fn html_body(response: HttpResponse) -> String {
        response.body().to_string()
    }

    /// A two-element spec: a root Card with a single Text child. Enough
    /// surface to exercise title/layout plumbing without any plugin assets.
    fn sample_spec() -> Spec {
        Spec::builder()
            .title("Test Page")
            .element(
                "card",
                Element::new("Card")
                    .prop("title", "Hello")
                    .prop("description", "A test card"),
            )
            .build()
            .expect("sample_spec is valid")
    }

    /// Check that a hyper response contains a Content-Type header with the given value.
    /// After the phase 143.1 header replace-semantics fix, only one Content-Type
    /// entry is emitted per response; `get_all` now yields a single-element iterator.
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
        let spec = sample_spec();
        let data = serde_json::json!({});
        let result = JsonUi::render(&spec, &data);

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
        let spec = sample_spec();
        let data = serde_json::json!({"users": [1, 2, 3]});
        let result = JsonUi::render_json(&spec, &data);

        assert!(result.is_ok());
        let response = ok_response(result);
        assert_eq!(response.status_code(), 200);

        let hyper = response.into_hyper();
        assert!(has_content_type(&hyper, "application/json"));

        let body = format!("{:?}", hyper.into_body());
        assert!(body.contains("spec"));
        assert!(body.contains("data"));
    }

    #[test]
    fn config_tailwind_disabled() {
        let spec = sample_spec();
        let data = serde_json::json!({});
        let config = JsonUiConfig::new().tailwind_cdn(false);
        let result = JsonUi::render_with_config(&spec, &data, &config);

        let body = response_body(ok_response(result));
        assert!(!body.contains("@tailwindcss/browser"));
    }

    #[test]
    fn config_custom_head() {
        let spec = sample_spec();
        let data = serde_json::json!({});
        let config =
            JsonUiConfig::new().custom_head(r#"<link rel="stylesheet" href="/custom.css">"#);
        let result = JsonUi::render_with_config(&spec, &data, &config);

        let body = response_body(ok_response(result));
        assert!(body.contains("/custom.css"));
    }

    #[test]
    fn config_body_class() {
        let spec = sample_spec();
        let data = serde_json::json!({});
        let config = JsonUiConfig::new().body_class("dark bg-black");
        let result = JsonUi::render_with_config(&spec, &data, &config);

        let body = response_body(ok_response(result));
        assert!(body.contains("dark bg-black"));
    }

    #[test]
    fn default_config_emits_ferro_base_css_link_and_no_cdn_script() {
        let view = sample_spec();
        let data = serde_json::json!({});
        let result = JsonUi::render(&view, &data);
        let body = html_body(ok_response(result));

        assert!(
            body.contains(r#"<link rel="stylesheet" href="/_ferro/ferro-base.css?v="#),
            "default config must emit <link> to /_ferro/ferro-base.css; body was: {body}"
        );
        assert!(
            !body.contains("@tailwindcss/browser"),
            "default config must NOT emit the Tailwind CDN script"
        );
    }

    #[test]
    fn tailwind_cdn_opt_in_coexists_with_default_stylesheet_urls() {
        // D-14: both <link> and <script> appear; no mutual-exclusion logic.
        let view = sample_spec();
        let data = serde_json::json!({});
        let config = JsonUiConfig::new().tailwind_cdn(true);
        let result = JsonUi::render_with_config(&view, &data, &config);
        let body = html_body(ok_response(result));

        assert!(
            body.contains(r#"<link rel="stylesheet" href="/_ferro/ferro-base.css?v="#),
            "<link> must be present even when CDN is opted in"
        );
        assert!(
            body.contains("@tailwindcss/browser"),
            "CDN script must be present when tailwind_cdn(true) is set"
        );

        // Ordering: <link> before <script>.
        let link_pos = body
            .find(r#"<link rel="stylesheet" href="/_ferro/ferro-base.css?v="#)
            .expect("link must exist");
        let cdn_pos = body.find("@tailwindcss/browser").expect("cdn must exist");
        assert!(
            link_pos < cdn_pos,
            "<link> tags must precede the CDN <script>"
        );
    }

    #[test]
    fn stylesheet_urls_emitted_in_order_and_replaces_default() {
        let view = sample_spec();
        let data = serde_json::json!({});
        let config =
            JsonUiConfig::new().stylesheet_urls(vec!["/a.css".to_string(), "/b.css".to_string()]);
        let result = JsonUi::render_with_config(&view, &data, &config);
        let body = html_body(ok_response(result));

        assert!(body.contains(r#"<link rel="stylesheet" href="/a.css">"#));
        assert!(body.contains(r#"<link rel="stylesheet" href="/b.css">"#));
        assert!(
            !body.contains(r#"href="/_ferro/ferro-base.css""#),
            "custom stylesheet_urls must replace the default, not append"
        );
        let a_pos = body.find("/a.css").unwrap();
        let b_pos = body.find("/b.css").unwrap();
        assert!(a_pos < b_pos, "stylesheet_urls order must be preserved");
    }

    #[test]
    fn empty_stylesheet_urls_emits_no_ferro_base_link() {
        let view = sample_spec();
        let data = serde_json::json!({});
        let config = JsonUiConfig::new().stylesheet_urls(vec![]);
        let result = JsonUi::render_with_config(&view, &data, &config);
        let body = html_body(ok_response(result));

        assert!(
            !body.contains(r#"href="/_ferro/ferro-base.css""#),
            "empty stylesheet_urls must not emit the default link"
        );
        // Bunny Fonts is always present as a separate <link> — do NOT assert
        // absence of all <link> tags.
    }

    #[test]
    fn stylesheet_urls_are_html_escaped_in_href_attribute() {
        // URL with the five characters html_escape handles: &, <, >, ", '.
        // Locks in defense-in-depth against attribute-context injection if an
        // app forwards untrusted values into .stylesheet_urls(...).
        let view = sample_spec();
        let data = serde_json::json!({});
        let config =
            JsonUiConfig::new().stylesheet_urls(vec![r#"/s.css?a=1&b=2&q=<x>""#.to_string()]);
        let result = JsonUi::render_with_config(&view, &data, &config);
        let body = html_body(ok_response(result));

        // Escaped form must be present.
        assert!(
            body.contains(r#"href="/s.css?a=1&amp;b=2&amp;q=&lt;x&gt;&quot;">"#),
            "URL must be HTML-escaped in href; body was: {body}"
        );
        // Raw unescaped form must NOT appear inside the href attribute.
        assert!(
            !body.contains(r#"href="/s.css?a=1&b=2&q=<x>""#),
            "raw unescaped URL must not appear in href"
        );
    }

    #[test]
    fn bunny_fonts_link_in_head() {
        let spec = sample_spec();
        let data = serde_json::json!({});
        let result = JsonUi::render(&spec, &data);

        let body = response_body(ok_response(result));
        assert!(
            body.contains("fonts.bunny.net"),
            "head should contain Bunny Fonts link"
        );
        assert!(
            body.contains("family=inter"),
            "head should request Inter font family"
        );
    }

    #[test]
    fn html_escaping_prevents_xss_in_title() {
        let spec = Spec::builder()
            .title(r#"<script>alert("xss")</script>"#)
            .element("card", Element::new("Text").prop("content", "ignored"))
            .build()
            .expect("spec is valid");
        let data = serde_json::json!({});
        let result = JsonUi::render(&spec, &data);

        let body = response_body(ok_response(result));
        // The raw script tag must not appear unescaped
        assert!(!body.contains("<script>alert"));
        assert!(body.contains("&lt;script&gt;"));
    }

    #[test]
    fn html_escaping_in_data_attributes() {
        let spec = sample_spec();
        let data = serde_json::json!({"key": "<img src=x onerror=alert(1)>"});
        let result = JsonUi::render(&spec, &data);

        let body = response_body(ok_response(result));
        // Angle brackets must be escaped in attribute values
        assert!(!body.contains("<img src=x"));
        assert!(body.contains("&lt;img"));
    }

    #[test]
    fn empty_spec_renders_valid_html() {
        // A "minimal" spec is one element with no title — Spec::builder() requires
        // at least one element (there's no such thing as a zero-element spec under
        // v2's structural validation).
        let spec = Spec::builder()
            .element("root", Element::new("Text"))
            .build()
            .expect("minimal spec is valid");
        let data = serde_json::json!({});
        let result = JsonUi::render(&spec, &data);

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
        let spec = Spec::builder()
            .title("Test Page")
            .element(
                "card",
                Element::new("Card")
                    .prop("title", "Hello")
                    .prop("description", "A test card"),
            )
            .data(serde_json::json!({"embedded": true}))
            .build()
            .expect("spec is valid");
        let explicit_data = serde_json::json!({"explicit": true});
        let result = JsonUi::render_json(&spec, &explicit_data);

        let response = ok_response(result);
        let hyper = response.into_hyper();
        let body = format!("{:?}", hyper.into_body());

        // Explicit data should be used, not the embedded one
        assert!(body.contains("explicit"));
        // The spec's embedded data is serialized under the "spec" key as spec.data
        assert!(body.contains("embedded"));
    }

    #[test]
    fn render_json_falls_back_to_embedded_data() {
        let spec = Spec::builder()
            .title("Test Page")
            .element("card", Element::new("Card").prop("title", "Hello"))
            .data(serde_json::json!({"embedded": true}))
            .build()
            .expect("spec is valid");
        let null_data = serde_json::Value::Null;
        let result = JsonUi::render_json(&spec, &null_data);

        let response = ok_response(result);
        let hyper = response.into_hyper();
        let body = format!("{:?}", hyper.into_body());

        // Should use the spec's embedded data
        assert!(body.contains("embedded"));
    }

    #[test]
    fn render_resolves_action_urls() {
        // Register a test route name -> path mapping.
        crate::routing::register_route_name("users.index", "/users");

        let action = Action {
            handler: ferro_json_ui::action::ActionHandler::Literal("users.index".to_string()),
            url: None,
            method: HttpMethod::Get,
            confirm: None,
            on_success: None,
            on_error: None,
            target: None,
        };
        let spec = Spec::builder()
            .title("Users")
            .element(
                "btn",
                Element::new("Button")
                    .prop("label", "List Users")
                    .action(action),
            )
            .build()
            .expect("spec is valid");

        // render_json should resolve action URLs.
        let result = JsonUi::render_json(&spec, &serde_json::json!({}));
        let body = response_body(ok_response(result));
        assert!(
            body.contains("/users"),
            "render_json output should contain the resolved URL"
        );

        // render (HTML) should also resolve action URLs.
        let result = JsonUi::render(&spec, &serde_json::json!({}));
        let body = response_body(ok_response(result));
        assert!(
            body.contains("/users"),
            "render output should contain the resolved URL"
        );

        // Original spec must not be mutated (clone semantics).
        let original_action = spec.elements.get("btn").unwrap().action.as_ref().unwrap();
        assert_eq!(
            original_action.url, None,
            "original spec should not be mutated"
        );
    }

    #[test]
    fn render_without_actions_still_works() {
        // Verify specs with no actions render without issues (no regression).
        let spec = sample_spec();
        let data = serde_json::json!({"items": [1, 2]});

        let result = JsonUi::render(&spec, &data);
        assert!(result.is_ok());

        let result = JsonUi::render_json(&spec, &data);
        assert!(result.is_ok());
    }

    // -----------------------------------------------------------------------
    // render_file tests
    // -----------------------------------------------------------------------

    #[test]
    fn render_file_returns_error_for_missing_file() {
        let result = JsonUi::render_file(
            std::path::Path::new("/nonexistent/path/test.json"),
            serde_json::json!({}),
        );
        assert!(result.is_err(), "missing file should return Err response");
        let err = result.unwrap_err();
        assert_eq!(err.status_code(), 500);
    }

    // -----------------------------------------------------------------------
    // render_with_errors tests
    // -----------------------------------------------------------------------

    use std::collections::HashMap;

    /// A form spec with two Input children identified by field name. The
    /// v2 walker renders each Input with its `props.errors` array, which
    /// `resolve_errors` populates per-field before render. Error strings
    /// surface both in the rendered Input markup and (for JSON handlers)
    /// in the serialized spec under `data-view`.
    fn form_spec_with_inputs() -> Spec {
        let action = Action {
            handler: ferro_json_ui::action::ActionHandler::Literal("users.store".to_string()),
            url: None,
            method: HttpMethod::Post,
            confirm: None,
            on_success: None,
            on_error: None,
            target: None,
        };
        Spec::builder()
            .title("Create User")
            .element(
                "form",
                Element::new("Form")
                    .action(action)
                    .child("name-input")
                    .child("email-input"),
            )
            .element(
                "name-input",
                Element::new("Input")
                    .prop("field", "name")
                    .prop("label", "Name")
                    .prop("input_type", "text"),
            )
            .element(
                "email-input",
                Element::new("Input")
                    .prop("field", "email")
                    .prop("label", "Email")
                    .prop("input_type", "email"),
            )
            .build()
            .expect("form spec is valid")
    }

    fn make_errors(pairs: &[(&str, &[&str])]) -> HashMap<String, Vec<String>> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
            .collect()
    }

    #[test]
    fn render_with_errors_populates_form_fields() {
        let spec = form_spec_with_inputs();
        let errors = make_errors(&[
            ("name", &["Name is required"]),
            ("email", &["Email is invalid"]),
        ]);
        let data = serde_json::json!({});
        let result = JsonUi::render_with_errors(&spec, &data, &errors);

        assert!(result.is_ok());
        let body = html_body(ok_response(result));

        assert!(
            body.contains(r#"<p id="err-name" class="text-sm text-destructive">Name is required</p>"#),
            "error <p> must appear below name input; got: {body}"
        );
        assert!(
            body.contains(r#"<p id="err-email" class="text-sm text-destructive">Email is invalid</p>"#),
            "error <p> must appear below email input; got: {body}"
        );
        assert!(!body.contains("<!-- ferro-json-ui:"), "no diagnostic comments in happy path; got: {body}");
    }

    #[test]
    fn render_json_with_errors_includes_errors_in_response() {
        let spec = form_spec_with_inputs();
        let errors = make_errors(&[("name", &["Name is required"])]);
        let data = serde_json::json!({});
        let result = JsonUi::render_json_with_errors(&spec, &data, &errors);

        assert!(result.is_ok());
        let body = response_body(ok_response(result));

        // Field-level error on name input.
        assert!(
            body.contains("Name is required"),
            "body should contain field-level error"
        );
        // The "name" field key appears in the serialized spec.
        assert!(
            body.contains("name"),
            "body should contain the error field name"
        );
    }

    #[test]
    fn render_with_errors_empty_map_produces_no_errors() {
        let spec = form_spec_with_inputs();
        let errors: HashMap<String, Vec<String>> = HashMap::new();
        let data = serde_json::json!({});

        let with_errors = JsonUi::render_with_errors(&spec, &data, &errors);
        let without_errors = JsonUi::render(&spec, &data);

        assert!(with_errors.is_ok());
        assert!(without_errors.is_ok());

        let body_with = response_body(ok_response(with_errors));
        // With empty errors, no field error messages are populated.
        assert!(
            !body_with.contains("Name is required"),
            "empty errors should not produce field-level messages"
        );
    }

    #[test]
    fn render_validation_error_accepts_framework_type() {
        let spec = form_spec_with_inputs();
        let mut ve = crate::validation::ValidationError::new();
        ve.add("name", "Name is required");
        ve.add("email", "Email must be valid");

        let data = serde_json::json!({});
        let result = JsonUi::render_validation_error(&spec, &data, &ve);

        assert!(result.is_ok());
        let body = html_body(ok_response(result));
        assert!(
            body.contains(r#"<p id="err-name" class="text-sm text-destructive">Name is required</p>"#),
            "error <p> must appear below name input; got: {body}"
        );
        assert!(
            body.contains(r#"<p id="err-email" class="text-sm text-destructive">Email must be valid</p>"#),
            "error <p> must appear below email input; got: {body}"
        );
        assert!(!body.contains("<!-- ferro-json-ui:"), "no diagnostic comments in happy path; got: {body}");
    }

    // D-07a: manual $data binding path — runtime handler data → error <p>
    #[test]
    fn pipeline_data_binding_error_prop_renders_p_tag() {
        let spec = Spec::builder()
            .element(
                "email-input",
                Element::new("Input")
                    .prop("field", "email")
                    .prop("label", "Email")
                    .prop("error", serde_json::json!({"$data": "/email_error"})),
            )
            .build()
            .expect("spec is valid");

        let data = serde_json::json!({"email_error": "must be valid"});
        let result = JsonUi::render(&spec, &data);
        let body = html_body(ok_response(result));

        assert!(
            body.contains(r#"<p id="err-email" class="text-sm text-destructive">must be valid</p>"#),
            "error paragraph must appear below the input; got: {body}"
        );
        assert!(
            !body.contains("<!-- ferro-json-ui:"),
            "no diagnostic comments in happy path; got: {body}"
        );
    }

    // D-07b: blessed render_validation_error path → error <p>
    #[test]
    fn pipeline_render_validation_error_renders_p_tag() {
        let spec = Spec::builder()
            .element(
                "email-input",
                Element::new("Input")
                    .prop("field", "email")
                    .prop("label", "Email"),
            )
            .build()
            .expect("spec is valid");

        let mut ve = crate::validation::ValidationError::new();
        ve.add("email", "must be valid");

        let data = serde_json::json!({});
        let result = JsonUi::render_validation_error(&spec, &data, &ve);
        let body = html_body(ok_response(result));

        assert!(
            body.contains(r#"<p id="err-email" class="text-sm text-destructive">must be valid</p>"#),
            "error paragraph must appear below the input; got: {body}"
        );
        assert!(
            body.contains(r#"aria-invalid="true""#),
            "aria-invalid must be set on the input; got: {body}"
        );
        assert!(
            !body.contains("<!-- ferro-json-ui:"),
            "no diagnostic comments in happy path; got: {body}"
        );
    }

    #[test]
    fn render_with_errors_preserves_action_resolution() {
        crate::routing::register_route_name("users.store", "/users");

        let spec = form_spec_with_inputs();
        let errors = make_errors(&[("name", &["Name is required"])]);
        let data = serde_json::json!({});

        // render_json_with_errors should have both action URL resolved and errors populated.
        let result = JsonUi::render_json_with_errors(&spec, &data, &errors);
        assert!(result.is_ok());
        let body = response_body(ok_response(result));

        assert!(body.contains("/users"), "action URL should be resolved");
        assert!(
            body.contains("Name is required"),
            "field errors should be populated"
        );
    }

    // ------------------------------------------------------------------------
    // Expression resolution tests
    // ------------------------------------------------------------------------

    fn expression_spec_with_data_marker() -> Spec {
        Spec::builder()
            .data(serde_json::json!({"greeting": "Hello"}))
            .element(
                "root",
                Element::new("Text").prop("content", serde_json::json!({"$data": "/greeting"})),
            )
            .build()
            .expect("spec builder should succeed")
    }

    #[test]
    fn render_resolves_data_expression_before_html_emission() {
        let spec = expression_spec_with_data_marker();
        let result = JsonUi::render(&spec, &serde_json::json!({}));
        assert!(result.is_ok());
        let body = response_body(ok_response(result));
        assert!(
            body.contains("Hello"),
            "rendered HTML must contain resolved value, got: {body}"
        );
        assert!(
            !body.contains("$data"),
            "rendered HTML must NOT contain '$data' marker, got: {body}"
        );
    }

    #[test]
    fn render_json_returns_spec_with_no_expression_markers() {
        let spec = expression_spec_with_data_marker();
        let result = JsonUi::render_json(&spec, &serde_json::Value::Null);
        assert!(result.is_ok());
        let body = response_body(ok_response(result));
        assert!(
            body.contains("Hello"),
            "render_json must contain resolved string value, got: {body}"
        );
        assert!(
            !body.contains("$data"),
            "render_json must NOT contain '$data' marker, got: {body}"
        );
    }

    #[test]
    fn render_with_config_honors_expression_resolution() {
        let spec = Spec::builder()
            .data(serde_json::json!({"count": 42}))
            .element(
                "root",
                Element::new("Text").prop("content", serde_json::json!({"$data": "/count"})),
            )
            .build()
            .expect("spec builder should succeed");
        let result =
            JsonUi::render_with_config(&spec, &serde_json::json!({}), &JsonUiConfig::new());
        assert!(result.is_ok());
        let body = response_body(ok_response(result));
        assert!(
            body.contains("42"),
            "render_with_config must render resolved numeric value, got: {body}"
        );
        assert!(
            !body.contains("$data"),
            "render_with_config output must NOT contain '$data' marker"
        );
    }

    #[test]
    fn render_with_errors_resolves_expressions_then_applies_errors() {
        let spec = Spec::builder()
            .data(serde_json::json!({"field_label": "Email"}))
            .element(
                "form_field",
                Element::new("Input").prop("field", "email").prop(
                    "label",
                    serde_json::json!({"$template": "Errors for {/field_label}"}),
                ),
            )
            .build()
            .expect("spec builder should succeed");

        let mut errors: HashMap<String, Vec<String>> = HashMap::new();
        errors.insert("email".to_string(), vec!["is required".to_string()]);

        let result = JsonUi::render_with_errors(&spec, &serde_json::json!({}), &errors);
        assert!(result.is_ok());
        let body = response_body(ok_response(result));
        assert!(
            body.contains("Errors for Email"),
            "render_with_errors must render template against resolved spec.data, got: {body}"
        );
        assert!(
            body.contains("is required"),
            "render_with_errors must attach error message after template resolution, got: {body}"
        );
        assert!(
            !body.contains("{/field_label}"),
            "render_with_errors output must NOT contain unresolved template placeholder"
        );
        assert!(
            !body.contains("$template"),
            "render_with_errors output must NOT contain '$template' marker"
        );
    }

    #[test]
    fn render_json_with_errors_returns_resolved_spec_with_errors() {
        // Pins must_haves truth #5: render_json_with_errors inherits expression
        // resolution via resolve_with_errors. Returns JSON payload
        // {"spec": resolved_spec, "data": effective_data}.
        let spec = Spec::builder()
            .data(serde_json::json!({"user_name": "Alice"}))
            .element(
                "form_field",
                Element::new("Input").prop("field", "email").prop(
                    "label",
                    serde_json::json!({"$template": "Hello, {/user_name}"}),
                ),
            )
            .build()
            .expect("spec builder should succeed");

        let mut errors: HashMap<String, Vec<String>> = HashMap::new();
        errors.insert("email".to_string(), vec!["is invalid".to_string()]);

        let result = JsonUi::render_json_with_errors(&spec, &serde_json::Value::Null, &errors);
        assert!(result.is_ok());
        let body = response_body(ok_response(result));
        assert!(
            body.contains("Hello, Alice"),
            "render_json_with_errors must return resolved template output, got: {body}"
        );
        assert!(
            body.contains("is invalid"),
            "render_json_with_errors must attach error message to the email field, got: {body}"
        );
        assert!(
            !body.contains("$template"),
            "render_json_with_errors output must NOT contain '$template' marker, got: {body}"
        );
        assert!(
            !body.contains("{/user_name}"),
            "render_json_with_errors output must NOT contain unresolved template placeholder, got: {body}"
        );
    }

    // -----------------------------------------------------------------------
    // Layout integration tests
    // -----------------------------------------------------------------------

    use ferro_json_ui::{register_layout, Layout, LayoutContext};

    fn sample_spec_with_layout(layout: &str) -> Spec {
        Spec::builder()
            .title("Test Page")
            .layout(layout)
            .element(
                "card",
                Element::new("Card")
                    .prop("title", "Hello")
                    .prop("description", "A test card"),
            )
            .build()
            .expect("sample_spec_with_layout is valid")
    }

    #[test]
    fn render_uses_default_layout_when_none_set() {
        let spec = sample_spec(); // no .layout() call
        let data = serde_json::json!({});
        let result = JsonUi::render(&spec, &data);

        assert!(result.is_ok());
        let body = response_body(ok_response(result));

        // DefaultLayout produces valid HTML with the ferro-json-ui wrapper
        assert!(body.contains("<!DOCTYPE html>"));
        assert!(body.contains("data-view="));
        assert!(body.contains("data-props="));
        assert!(body.contains("ferro-json-ui"));
        // No nav or sidebar in default layout
        assert!(!body.contains("<nav"));
        assert!(!body.contains("<aside"));
    }

    #[test]
    fn render_uses_named_layout() {
        let spec = sample_spec_with_layout("app");
        let data = serde_json::json!({});
        let result = JsonUi::render(&spec, &data);

        assert!(result.is_ok());
        let body = response_body(ok_response(result));

        // AppLayout includes nav, sidebar, and main content area
        assert!(body.contains("<nav"));
        assert!(body.contains("<aside"));
        assert!(body.contains("<main"));
        assert!(body.contains("ferro-json-ui"));
    }

    #[test]
    fn render_uses_auth_layout() {
        let spec = sample_spec_with_layout("auth");
        let data = serde_json::json!({});
        let result = JsonUi::render(&spec, &data);

        assert!(result.is_ok());
        let body = response_body(ok_response(result));

        // AuthLayout centers content with max-width card
        assert!(body.contains("flex items-center justify-center"));
        assert!(body.contains("max-w-md"));
        assert!(body.contains("ferro-json-ui"));
        // No nav or sidebar
        assert!(!body.contains("<nav"));
        assert!(!body.contains("<aside"));
    }

    #[test]
    fn render_with_errors_uses_layout() {
        let mut spec = form_spec_with_inputs();
        spec.layout = Some("auth".to_string());
        let errors = make_errors(&[("name", &["Name is required"])]);
        let data = serde_json::json!({});
        let result = JsonUi::render_with_errors(&spec, &data, &errors);

        assert!(result.is_ok());
        let body = response_body(ok_response(result));

        // Auth layout structure present
        assert!(body.contains("flex items-center justify-center"));
        // Error content present
        assert!(body.contains("Name is required"));
    }

    #[test]
    fn render_custom_layout() {
        struct TestLayout;
        impl Layout for TestLayout {
            fn render(&self, ctx: &LayoutContext) -> String {
                format!("<custom-layout>{}</custom-layout>", ctx.content)
            }
        }

        register_layout("test-custom", TestLayout);

        let spec = sample_spec_with_layout("test-custom");
        let data = serde_json::json!({});
        let result = JsonUi::render(&spec, &data);

        assert!(result.is_ok());
        let body = response_body(ok_response(result));
        assert!(body.contains("<custom-layout>"));
        assert!(body.contains("</custom-layout>"));
    }

    #[test]
    fn render_unknown_layout_falls_back_to_default() {
        let spec = sample_spec_with_layout("nonexistent-layout-xyz");
        let data = serde_json::json!({});
        let result = JsonUi::render(&spec, &data);

        assert!(result.is_ok());
        let body = response_body(ok_response(result));

        // Falls back to default layout (valid HTML, no nav/sidebar)
        assert!(body.contains("<!DOCTYPE html>"));
        assert!(body.contains("ferro-json-ui"));
        assert!(!body.contains("<nav"));
        assert!(!body.contains("<aside"));
    }

    // -----------------------------------------------------------------------
    // Theme CSS injection tests (only compiled with theme feature)
    // -----------------------------------------------------------------------

    #[cfg(feature = "theme")]
    mod theme_tests {
        use super::*;
        use crate::theme::context::{theme_scope, with_theme_scope};
        use ferro_theme::Theme;
        use std::sync::Arc;

        fn ok_response_body(result: Response) -> String {
            let response = match result {
                Ok(r) => r,
                Err(_) => panic!("expected Ok response"),
            };
            response.body().to_string()
        }

        fn sample_spec() -> Spec {
            Spec::builder()
                .title("Theme Test")
                .element("card", Element::new("Card").prop("title", "Hello"))
                .build()
                .expect("theme sample_spec is valid")
        }

        // Test: When current_theme() returns Some, theme CSS is included in rendered HTML head as a style tag
        #[tokio::test]
        async fn theme_css_injected_into_head_when_theme_active() {
            let mut custom_theme = Theme::default_theme();
            custom_theme.css = ":root { --color-primary: red; }".to_string();
            let scope = theme_scope();
            {
                let mut guard = scope.write().await;
                *guard = Some(Arc::new(custom_theme));
            }

            let spec = sample_spec();
            let data = serde_json::json!({});
            let body = with_theme_scope(scope, async {
                ok_response_body(JsonUi::render(&spec, &data))
            })
            .await;

            assert!(
                body.contains("<style>") && body.contains("--color-primary: red"),
                "theme CSS should be injected as a plain <style> tag"
            );
            assert!(
                !body.contains(r#"<style type="text/tailwindcss">"#),
                "theme CSS must not use the Tailwind-CDN-only type=\"text/tailwindcss\" attribute"
            );
        }

        // Test: When current_theme() returns None (no ThemeMiddleware), head still works without theme CSS
        #[test]
        fn no_theme_css_injected_when_no_middleware() {
            // No theme scope — current_theme() returns None
            let spec = sample_spec();
            let data = serde_json::json!({});
            let result = JsonUi::render(&spec, &data);
            let body = ok_response_body(result);

            // Should still render valid HTML
            assert!(body.contains("<!DOCTYPE html>"));
            // Should not have theme-specific style injection (no scope active)
            // The page still renders — no crash on missing theme
        }

        // Test: Theme CSS is injected AFTER the Tailwind CDN script tag
        #[tokio::test]
        async fn theme_css_injected_after_tailwind_cdn() {
            let mut custom_theme = Theme::default_theme();
            custom_theme.css = ":root { --color-test: blue; }".to_string();
            let scope = theme_scope();
            {
                let mut guard = scope.write().await;
                *guard = Some(Arc::new(custom_theme));
            }

            let spec = sample_spec();
            let data = serde_json::json!({});
            let config = JsonUiConfig::new().tailwind_cdn(true);
            let body = with_theme_scope(scope, async {
                ok_response_body(JsonUi::render_with_config(&spec, &data, &config))
            })
            .await;

            // Find positions to verify ordering
            let cdn_pos = body.find("@tailwindcss/browser").unwrap_or(0);
            let style_pos = body.find("--color-test").unwrap_or(0);
            assert!(
                cdn_pos < style_pos,
                "theme CSS should come after Tailwind CDN script"
            );
        }

        // Test: Theme CSS style tag does not duplicate if custom_head already has content
        #[tokio::test]
        async fn theme_css_does_not_duplicate_custom_head_content() {
            let mut custom_theme = Theme::default_theme();
            custom_theme.css = ":root { --color-custom: purple; }".to_string();
            let scope = theme_scope();
            {
                let mut guard = scope.write().await;
                *guard = Some(Arc::new(custom_theme));
            }

            let spec = sample_spec();
            let data = serde_json::json!({});
            let config =
                JsonUiConfig::new().custom_head(r#"<link rel="stylesheet" href="/my.css">"#);
            let body = with_theme_scope(scope, async {
                ok_response_body(JsonUi::render_with_config(&spec, &data, &config))
            })
            .await;

            // Both custom head content AND theme CSS should be present
            assert!(body.contains("/my.css"), "custom_head should be present");
            assert!(
                body.contains("--color-custom"),
                "theme CSS should be injected"
            );
            // Theme CSS should appear exactly once
            let count = body.matches("--color-custom").count();
            assert_eq!(count, 1, "theme CSS should not be duplicated");
        }
    }

    // -----------------------------------------------------------------------
    // Plugin integration tests
    // -----------------------------------------------------------------------

    // Plugin integration: MapPlugin is auto-registered in the global plugin
    // registry. The v2 walker dispatches unknown type_names ("Map") through
    // the plugin registry and collects CSS/JS assets for head/body injection.
    #[test]
    fn test_plugin_component_renders_in_full_page() {
        // MapPlugin is auto-registered via the global registry OnceLock init.
        let spec = Spec::builder()
            .title("Map Page")
            .element(
                "map",
                Element::new("Map")
                    .prop("center", serde_json::json!([51.505, -0.09]))
                    .prop("zoom", 13)
                    .prop(
                        "markers",
                        serde_json::json!([
                            {"lat": 51.5, "lng": -0.09, "popup": "London"}
                        ]),
                    ),
            )
            .build()
            .expect("map spec is valid");
        let data = serde_json::json!({});
        let result = JsonUi::render(&spec, &data);

        assert!(result.is_ok(), "render should succeed");
        let body = response_body(ok_response(result));

        // Leaflet CSS in head area.
        assert!(
            body.contains("leaflet.css"),
            "should include Leaflet CSS link"
        );
        // Leaflet JS before body close.
        assert!(
            body.contains("leaflet.js"),
            "should include Leaflet JS script"
        );
        // Map container with data attribute.
        assert!(
            body.contains("data-ferro-map"),
            "should contain map data attribute"
        );
        // Init script with DOMContentLoaded.
        assert!(
            body.contains("DOMContentLoaded"),
            "should contain init script"
        );
    }

    /// Assets deduplicated — one `leaflet.css` link even when the spec
    /// references the Map plugin from multiple elements (CONTEXT D-18).
    #[test]
    fn test_plugin_assets_deduplicated_across_elements() {
        let spec = Spec::builder()
            .title("Two Maps")
            .element("root", Element::new("Grid").child("map-a").child("map-b"))
            .element(
                "map-a",
                Element::new("Map")
                    .prop("center", serde_json::json!([51.505, -0.09]))
                    .prop("zoom", 13),
            )
            .element(
                "map-b",
                Element::new("Map")
                    .prop("center", serde_json::json!([40.7128, -74.006]))
                    .prop("zoom", 12),
            )
            .build()
            .expect("two-map spec is valid");
        let data = serde_json::json!({});
        let result = JsonUi::render(&spec, &data);

        assert!(result.is_ok(), "render should succeed");
        let body = response_body(ok_response(result));

        let css_link_count = body.matches("leaflet.css").count();
        assert_eq!(
            css_link_count, 1,
            "expected exactly 1 leaflet.css occurrence, found {css_link_count}"
        );
    }

    // -----------------------------------------------------------------------
    // Title binding resolution tests (D-12)
    // -----------------------------------------------------------------------

    #[test]
    fn render_title_literal() {
        let spec = Spec::builder()
            .title("Hello")
            .element("root", Element::new("Text").prop("content", "body"))
            .build()
            .expect("spec is valid");
        let data = serde_json::json!({});
        let result = JsonUi::render(&spec, &data);
        let body = html_body(ok_response(result));
        assert!(
            body.contains("<title>Hello</title>"),
            "literal title must appear verbatim; got: {body}"
        );
    }

    #[test]
    fn render_title_binding_resolves() {
        let spec = Spec::builder()
            .title_binding("/page_title")
            .data(serde_json::json!({"page_title": "Dynamic"}))
            .element("root", Element::new("Text").prop("content", "body"))
            .build()
            .expect("spec is valid");
        let data = serde_json::json!({"page_title": "Dynamic"});
        let result = JsonUi::render(&spec, &data);
        let body = html_body(ok_response(result));
        assert!(
            body.contains("<title>Dynamic</title>"),
            "binding title must resolve to data value; got: {body}"
        );
    }

    #[test]
    fn render_title_binding_missing_path_falls_back() {
        let spec = Spec::builder()
            .title_binding("/missing")
            .element("root", Element::new("Text").prop("content", "body"))
            .build()
            .expect("spec is valid");
        let data = serde_json::json!({});
        let result = JsonUi::render(&spec, &data);
        let body = html_body(ok_response(result));
        assert!(
            body.contains("<title>Ferro</title>"),
            "missing binding path must fall back to 'Ferro'; got: {body}"
        );
    }
}
