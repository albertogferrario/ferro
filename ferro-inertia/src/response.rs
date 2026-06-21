//! Inertia response generation.

use crate::config::InertiaConfig;
use crate::manifest::resolve_assets;
use crate::request::InertiaRequest;
use crate::shared::InertiaShared;
use serde::Serialize;

/// Framework-agnostic HTTP response.
///
/// Convert this to your framework's response type.
#[derive(Debug, Clone)]
pub struct InertiaHttpResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers as (name, value) pairs
    pub headers: Vec<(String, String)>,
    /// Response body
    pub body: String,
    /// Content type
    pub content_type: &'static str,
}

impl InertiaHttpResponse {
    /// Create a JSON response with Inertia headers.
    pub fn json(body: impl Into<String>) -> Self {
        Self {
            status: 200,
            headers: vec![
                ("X-Inertia".to_string(), "true".to_string()),
                ("Vary".to_string(), "X-Inertia".to_string()),
            ],
            body: body.into(),
            content_type: "application/json",
        }
    }

    /// Create a raw JSON response without Inertia headers.
    ///
    /// Used for JSON fallback when a non-Inertia client requests JSON.
    pub fn raw_json(body: impl Into<String>) -> Self {
        Self {
            status: 200,
            headers: vec![],
            body: body.into(),
            content_type: "application/json",
        }
    }

    /// Create an HTML response.
    pub fn html(body: impl Into<String>) -> Self {
        Self {
            status: 200,
            headers: vec![("Vary".to_string(), "X-Inertia".to_string())],
            body: body.into(),
            content_type: "text/html; charset=utf-8",
        }
    }

    /// Create a 409 Conflict response for version mismatch.
    pub fn conflict(location: impl Into<String>) -> Self {
        Self {
            status: 409,
            headers: vec![("X-Inertia-Location".to_string(), location.into())],
            body: String::new(),
            content_type: "text/plain",
        }
    }

    /// Set the HTTP status code.
    pub fn status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    /// Add a header to the response.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Create a redirect response for Inertia requests.
    ///
    /// For POST/PUT/PATCH/DELETE requests, uses status 303 (See Other) to force
    /// the browser to follow the redirect with a GET request.
    ///
    /// For GET requests, uses standard 302.
    pub fn redirect(location: impl Into<String>, is_post_like: bool) -> Self {
        // POST/PUT/PATCH/DELETE -> 303 (See Other) forces GET on redirect
        // GET -> 302 (Found) standard redirect
        let status = if is_post_like { 303 } else { 302 };

        Self {
            status,
            headers: vec![
                ("X-Inertia".to_string(), "true".to_string()),
                ("Location".to_string(), location.into()),
            ],
            body: String::new(),
            content_type: "text/plain",
        }
    }
}

/// Main Inertia integration struct.
///
/// Provides methods for rendering Inertia responses in a framework-agnostic way.
pub struct Inertia;

impl Inertia {
    /// Render an Inertia response.
    ///
    /// This is the primary method for returning Inertia responses from handlers.
    /// It automatically:
    /// - Detects XHR vs initial page load
    /// - Filters props for partial reloads
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_inertia::Inertia;
    /// use serde_json::json;
    ///
    /// let response = Inertia::render(&req, "Home", json!({
    ///     "title": "Welcome",
    ///     "user": { "name": "John" }
    /// }));
    /// ```
    pub fn render<R, P>(req: &R, component: &str, props: P) -> InertiaHttpResponse
    where
        R: InertiaRequest,
        P: Serialize,
    {
        Self::render_internal(req, component, props, None, InertiaConfig::default(), false)
    }

    /// Render an Inertia response with JSON fallback for API clients.
    ///
    /// When enabled, requests with `Accept: application/json` header (but without
    /// `X-Inertia: true`) will receive raw props as JSON instead of HTML.
    ///
    /// This is useful for:
    /// - API testing with curl or Postman
    /// - Hybrid apps that sometimes need raw JSON
    /// - Debug tooling
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_inertia::Inertia;
    /// use serde_json::json;
    ///
    /// // curl -H "Accept: application/json" http://localhost:3000/posts/1
    /// // Returns raw JSON props instead of HTML
    /// let response = Inertia::render_with_json_fallback(&req, "Posts/Show", json!({
    ///     "post": { "id": 1, "title": "Hello" }
    /// }));
    /// ```
    pub fn render_with_json_fallback<R, P>(
        req: &R,
        component: &str,
        props: P,
    ) -> InertiaHttpResponse
    where
        R: InertiaRequest,
        P: Serialize,
    {
        Self::render_internal(req, component, props, None, InertiaConfig::default(), true)
    }

    /// Render an Inertia response with shared props.
    pub fn render_with_shared<R, P>(
        req: &R,
        component: &str,
        props: P,
        shared: &InertiaShared,
    ) -> InertiaHttpResponse
    where
        R: InertiaRequest,
        P: Serialize,
    {
        Self::render_internal(
            req,
            component,
            props,
            Some(shared),
            InertiaConfig::default(),
            false,
        )
    }

    /// Render an Inertia response with custom configuration.
    pub fn render_with_config<R, P>(
        req: &R,
        component: &str,
        props: P,
        config: InertiaConfig,
    ) -> InertiaHttpResponse
    where
        R: InertiaRequest,
        P: Serialize,
    {
        Self::render_internal(req, component, props, None, config, false)
    }

    /// Render an Inertia response with all options.
    pub fn render_with_options<R, P>(
        req: &R,
        component: &str,
        props: P,
        shared: Option<&InertiaShared>,
        config: InertiaConfig,
    ) -> InertiaHttpResponse
    where
        R: InertiaRequest,
        P: Serialize,
    {
        Self::render_internal(req, component, props, shared, config, false)
    }

    /// Render an Inertia response with all options and JSON fallback.
    pub fn render_with_options_and_json_fallback<R, P>(
        req: &R,
        component: &str,
        props: P,
        shared: Option<&InertiaShared>,
        config: InertiaConfig,
    ) -> InertiaHttpResponse
    where
        R: InertiaRequest,
        P: Serialize,
    {
        Self::render_internal(req, component, props, shared, config, true)
    }

    /// Internal render method with all options.
    fn render_internal<R, P>(
        req: &R,
        component: &str,
        props: P,
        shared: Option<&InertiaShared>,
        config: InertiaConfig,
        json_fallback: bool,
    ) -> InertiaHttpResponse
    where
        R: InertiaRequest,
        P: Serialize,
    {
        let url = req.path().to_string();
        let is_inertia = req.is_inertia();
        let partial_data = req.inertia_partial_data();
        let partial_component = req.inertia_partial_component();

        // Serialize props
        let mut props_value = match serde_json::to_value(&props) {
            Ok(v) => v,
            Err(e) => {
                return InertiaHttpResponse::html(format!("Failed to serialize props: {e}"))
                    .status(500);
            }
        };

        // Merge shared props
        if let Some(shared) = shared {
            shared.merge_into(&mut props_value);
        }

        // Filter props for partial reloads
        if is_inertia {
            if let Some(partial_keys) = partial_data {
                let should_filter = partial_component.map(|pc| pc == component).unwrap_or(false);

                if should_filter {
                    props_value = Self::filter_partial_props(props_value, &partial_keys);
                }
            }
        }

        // Check for JSON fallback before normal Inertia handling
        // If JSON fallback is enabled and request accepts JSON but is not an Inertia request,
        // return raw props as JSON
        if json_fallback && !is_inertia && req.accepts_json() {
            return InertiaHttpResponse::raw_json(
                serde_json::to_string(&props_value).unwrap_or_default(),
            );
        }

        let response = InertiaResponse::new(component, props_value, url).with_config(config);

        // Extract CSRF token from shared props for HTML response
        let csrf = shared.and_then(|s| s.csrf.as_deref());

        if is_inertia {
            response.to_json_response()
        } else {
            response.to_html_response(csrf)
        }
    }

    /// Check if a version conflict should trigger a full reload.
    ///
    /// Returns `Some(response)` with a 409 Conflict if versions don't match.
    pub fn check_version<R: InertiaRequest>(
        req: &R,
        current_version: &str,
        redirect_url: &str,
    ) -> Option<InertiaHttpResponse> {
        if !req.is_inertia() {
            return None;
        }

        if let Some(client_version) = req.inertia_version() {
            if client_version != current_version {
                return Some(InertiaHttpResponse::conflict(redirect_url));
            }
        }

        None
    }

    /// Filter props to only include those requested in partial reload.
    fn filter_partial_props(props: serde_json::Value, partial_keys: &[&str]) -> serde_json::Value {
        match props {
            serde_json::Value::Object(map) => {
                let filtered: serde_json::Map<String, serde_json::Value> = map
                    .into_iter()
                    .filter(|(k, _)| partial_keys.contains(&k.as_str()))
                    .collect();
                serde_json::Value::Object(filtered)
            }
            other => other,
        }
    }
}

/// Internal response builder.
pub struct InertiaResponse {
    component: String,
    props: serde_json::Value,
    url: String,
    config: InertiaConfig,
}

impl InertiaResponse {
    /// Create a new Inertia response.
    pub fn new(component: impl Into<String>, props: serde_json::Value, url: String) -> Self {
        Self {
            component: component.into(),
            props,
            url,
            config: InertiaConfig::default(),
        }
    }

    /// Set the configuration.
    pub fn with_config(mut self, config: InertiaConfig) -> Self {
        self.config = config;
        self
    }

    /// Build JSON response for XHR requests.
    pub fn to_json_response(&self) -> InertiaHttpResponse {
        let page = serde_json::json!({
            "component": self.component,
            "props": self.props,
            "url": self.url,
            "version": self.config.version,
        });

        InertiaHttpResponse::json(serde_json::to_string(&page).unwrap_or_default())
    }

    /// Build HTML response for initial page loads.
    pub fn to_html_response(&self, csrf_token: Option<&str>) -> InertiaHttpResponse {
        let page_data = serde_json::json!({
            "component": self.component,
            "props": self.props,
            "url": self.url,
            "version": self.config.version,
        });

        // Escape JSON for HTML attribute
        let page_json = serde_json::to_string(&page_data)
            .unwrap_or_default()
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;");

        let csrf = csrf_token.unwrap_or("");

        // Use custom template if provided
        if let Some(template) = &self.config.html_template {
            let html = template
                .replace("{page}", &page_json)
                .replace("{csrf}", csrf);
            return InertiaHttpResponse::html(html);
        }

        // Derive shared template values from config fields (title/head_extras/mount_id).
        // These are computed once here and used in both dev and prod branches below.
        let title_text = self
            .config
            .title
            .as_deref()
            .unwrap_or(&self.config.app_name);
        let head_extras = self.config.head_extras.as_deref().unwrap_or("");
        let mount_id = self.config.mount_id.as_str();

        // Default template
        let html = if self.config.development {
            format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="csrf-token" content="{}">
    <title>{}</title>
    <script type="module">
        import RefreshRuntime from '{}/@react-refresh'
        RefreshRuntime.injectIntoGlobalHook(window)
        window.$RefreshReg$ = () => {{}}
        window.$RefreshSig$ = () => (type) => type
        window.__vite_plugin_react_preamble_installed__ = true
    </script>
    <script type="module" src="{}/@vite/client"></script>
    <script type="module" src="{}/{}"></script>
    {}
</head>
<body>
    <div id="{}" data-page="{}"></div>
</body>
</html>"#,
                csrf,
                title_text,
                self.config.vite_dev_server,
                self.config.vite_dev_server,
                self.config.vite_dev_server,
                self.config.entry_point,
                head_extras,
                mount_id,
                page_json
            )
        } else {
            let assets = resolve_assets(&self.config.manifest_path, &self.config.entry_point);

            let css_tags: String = assets
                .css
                .iter()
                .map(|href| format!(r#"    <link rel="stylesheet" href="{href}">"#))
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="csrf-token" content="{csrf}">
    <title>{title_text}</title>
    <script type="module" src="{js_src}"></script>
{css_tags}
    {head_extras}
</head>
<body>
    <div id="{mount_id}" data-page="{page_json}"></div>
</body>
</html>"#,
                js_src = assets.js,
                title_text = title_text,
                head_extras = head_extras,
                mount_id = mount_id,
            )
        };

        InertiaHttpResponse::html(html)
    }
}

#[cfg(test)]
mod content_negotiation_tests {
    use super::*;
    use crate::config::InertiaConfig;

    struct MockReq {
        is_inertia: bool,
        path: &'static str,
    }

    impl crate::request::InertiaRequest for MockReq {
        fn inertia_header(&self, name: &str) -> Option<&str> {
            if name == "X-Inertia" && self.is_inertia {
                Some("true")
            } else {
                None
            }
        }
        fn path(&self) -> &str {
            self.path
        }
    }

    #[test]
    fn non_inertia_request_returns_html_document() {
        let req = MockReq {
            is_inertia: false,
            path: "/home",
        };
        let resp = Inertia::render_with_config(
            &req,
            "Home",
            serde_json::json!({"title": "Hi"}),
            InertiaConfig::new().development(),
        );
        assert_eq!(resp.content_type, "text/html; charset=utf-8");
        assert!(resp.body.contains("<!DOCTYPE html>"));
        assert!(resp.body.contains(r#"data-page=""#));
    }

    #[test]
    fn inertia_request_returns_json_contract() {
        let req = MockReq {
            is_inertia: true,
            path: "/home",
        };
        let resp = Inertia::render_with_config(
            &req,
            "Home",
            serde_json::json!({"title": "Hi"}),
            InertiaConfig::new().development(),
        );
        assert_eq!(resp.content_type, "application/json");
        let body: serde_json::Value = serde_json::from_str(&resp.body).unwrap();
        assert_eq!(body["component"], "Home");
    }

    #[test]
    fn html_data_page_equals_json_contract() {
        let props = serde_json::json!({"title": "Hi", "count": 42});
        let cfg = InertiaConfig::new().development().version("test-1");
        let html = Inertia::render_with_config(
            &MockReq {
                is_inertia: false,
                path: "/home",
            },
            "Home",
            props.clone(),
            cfg.clone(),
        );
        let json = Inertia::render_with_config(
            &MockReq {
                is_inertia: true,
                path: "/home",
            },
            "Home",
            props,
            cfg,
        );
        let start = html.body.find(r#"data-page=""#).unwrap() + 11;
        let end = html.body[start..].find('"').unwrap() + start;
        let page = html.body[start..end]
            .replace("&quot;", "\"")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&#x27;", "'");
        let html_page: serde_json::Value = serde_json::from_str(&page).unwrap();
        let json_page: serde_json::Value = serde_json::from_str(&json.body).unwrap();
        assert_eq!(html_page, json_page);
    }

    #[test]
    fn dev_mode_emits_vite_client_script() {
        let resp = Inertia::render_with_config(
            &MockReq {
                is_inertia: false,
                path: "/",
            },
            "App",
            serde_json::json!({}),
            InertiaConfig::new()
                .development()
                .vite_dev_server("http://localhost:5173"),
        );
        assert!(resp.body.contains("http://localhost:5173/@vite/client"));
    }

    #[test]
    fn title_override() {
        let resp = Inertia::render_with_config(
            &MockReq {
                is_inertia: false,
                path: "/",
            },
            "App",
            serde_json::json!({}),
            InertiaConfig::new()
                .development()
                .app_name("Fallback")
                .title("Explicit"),
        );
        assert!(resp.body.contains("<title>Explicit</title>"));
        assert!(!resp.body.contains("<title>Fallback</title>"));
    }

    #[test]
    fn head_extras_in_html() {
        let resp = Inertia::render_with_config(
            &MockReq {
                is_inertia: false,
                path: "/",
            },
            "App",
            serde_json::json!({}),
            InertiaConfig::new()
                .development()
                .head_extras(r#"<meta name="x" content="y">"#),
        );
        assert!(resp.body.contains(r#"<meta name="x" content="y">"#));
    }

    #[test]
    fn mount_id_applied() {
        let resp = Inertia::render_with_config(
            &MockReq {
                is_inertia: false,
                path: "/",
            },
            "App",
            serde_json::json!({}),
            InertiaConfig::new().development().mount_id("root"),
        );
        assert!(resp.body.contains(r#"id="root" data-page="#));
    }

    // SECURITY (T-238-03): prod build must never emit the @vite/client preamble.
    // Uses the prod branch — asserts ABSENCE of dev tags (manifest-cache bleed
    // is harmless for absence assertions).
    #[test]
    fn prod_mode_does_not_leak_dev_server() {
        let resp = Inertia::render_with_config(
            &MockReq {
                is_inertia: false,
                path: "/",
            },
            "App",
            serde_json::json!({}),
            InertiaConfig::new()
                .production()
                .vite_dev_server("http://localhost:5173"),
        );
        assert!(!resp.body.contains("/@vite/client"));
        assert!(!resp.body.contains("@react-refresh"));
    }
}
