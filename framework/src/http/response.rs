use super::cookie::Cookie;
use bytes::Bytes;
use http_body_util::Full;

/// HTTP Response builder providing Laravel-like response creation
pub struct HttpResponse {
    status: u16,
    body: String,
    headers: Vec<(String, String)>,
}

/// Response type alias - allows using `?` operator for early returns
pub type Response = Result<HttpResponse, HttpResponse>;

impl HttpResponse {
    pub fn new() -> Self {
        Self {
            status: 200,
            body: String::new(),
            headers: Vec::new(),
        }
    }

    /// Create a response with a string body
    pub fn text(body: impl Into<String>) -> Self {
        Self {
            status: 200,
            body: body.into(),
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
        }
    }

    /// Create a JSON response from a serde_json::Value
    pub fn json(body: serde_json::Value) -> Self {
        Self {
            status: 200,
            body: body.to_string(),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
        }
    }

    /// Set the response body
    pub fn set_body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    /// Set the HTTP status code
    pub fn status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    /// Get the current HTTP status code
    pub fn status_code(&self) -> u16 {
        self.status
    }

    /// Get the response body as a string slice.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Add a header to the response
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Add a Set-Cookie header to the response
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::{Cookie, HttpResponse};
    ///
    /// let response = HttpResponse::text("OK")
    ///     .cookie(Cookie::new("session", "abc123"))
    ///     .cookie(Cookie::new("user_id", "42"));
    /// ```
    pub fn cookie(self, cookie: Cookie) -> Self {
        let header_value = cookie.to_header_value();
        self.header("Set-Cookie", header_value)
    }

    /// Wrap this response in Ok() for use as Response type
    pub fn ok(self) -> Response {
        Ok(self)
    }

    /// Convert to hyper response
    pub fn into_hyper(self) -> hyper::Response<Full<Bytes>> {
        let mut builder = hyper::Response::builder().status(self.status);

        for (name, value) in self.headers {
            builder = builder.header(name, value);
        }

        builder.body(Full::new(Bytes::from(self.body))).unwrap()
    }
}

impl Default for HttpResponse {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for Response to enable method chaining on macros
pub trait ResponseExt {
    fn status(self, code: u16) -> Self;
    fn header(self, name: impl Into<String>, value: impl Into<String>) -> Self;
}

impl ResponseExt for Response {
    fn status(self, code: u16) -> Self {
        self.map(|r| r.status(code))
    }

    fn header(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.map(|r| r.header(name, value))
    }
}

/// HTTP Redirect response builder
pub struct Redirect {
    location: String,
    query_params: Vec<(String, String)>,
    status: u16,
}

impl Redirect {
    /// Create a redirect to a specific URL/path
    pub fn to(path: impl Into<String>) -> Self {
        Self {
            location: path.into(),
            query_params: Vec::new(),
            status: 302,
        }
    }

    /// Create a redirect to a named route
    pub fn route(name: &str) -> RedirectRouteBuilder {
        RedirectRouteBuilder {
            name: name.to_string(),
            params: std::collections::HashMap::new(),
            query_params: Vec::new(),
            status: 302,
        }
    }

    /// Add a query parameter
    pub fn query(mut self, key: &str, value: impl Into<String>) -> Self {
        self.query_params.push((key.to_string(), value.into()));
        self
    }

    /// Set status to 301 (Moved Permanently)
    pub fn permanent(mut self) -> Self {
        self.status = 301;
        self
    }

    fn build_url(&self) -> String {
        if self.query_params.is_empty() {
            self.location.clone()
        } else {
            let query = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&");
            format!("{}?{}", self.location, query)
        }
    }
}

/// Auto-convert Redirect to Response
impl From<Redirect> for Response {
    fn from(redirect: Redirect) -> Response {
        Ok(HttpResponse::new()
            .status(redirect.status)
            .header("Location", redirect.build_url()))
    }
}

/// Builder for redirects to named routes with parameters
pub struct RedirectRouteBuilder {
    name: String,
    params: std::collections::HashMap<String, String>,
    query_params: Vec<(String, String)>,
    status: u16,
}

impl RedirectRouteBuilder {
    /// Add a route parameter value
    pub fn with(mut self, key: &str, value: impl Into<String>) -> Self {
        self.params.insert(key.to_string(), value.into());
        self
    }

    /// Add a query parameter
    pub fn query(mut self, key: &str, value: impl Into<String>) -> Self {
        self.query_params.push((key.to_string(), value.into()));
        self
    }

    /// Set status to 301 (Moved Permanently)
    pub fn permanent(mut self) -> Self {
        self.status = 301;
        self
    }

    fn build_url(&self) -> Option<String> {
        use crate::routing::route_with_params;

        let mut url = route_with_params(&self.name, &self.params)?;
        if !self.query_params.is_empty() {
            let query = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&");
            url = format!("{url}?{query}");
        }
        Some(url)
    }
}

/// Auto-convert RedirectRouteBuilder to Response
impl From<RedirectRouteBuilder> for Response {
    fn from(redirect: RedirectRouteBuilder) -> Response {
        let url = redirect.build_url().ok_or_else(|| {
            HttpResponse::text(format!("Route '{}' not found", redirect.name)).status(500)
        })?;
        Ok(HttpResponse::new()
            .status(redirect.status)
            .header("Location", url))
    }
}

/// Auto-convert FrameworkError to HttpResponse
///
/// This enables using the `?` operator in controller handlers to propagate
/// framework errors as appropriate HTTP responses.
///
/// When a hint is available (via `FrameworkError::hint()`), the JSON response
/// includes a `"hint"` field with actionable guidance for the developer.
impl From<crate::error::FrameworkError> for HttpResponse {
    fn from(err: crate::error::FrameworkError) -> HttpResponse {
        let status = err.status_code();
        let hint = err.hint();
        let mut body = match &err {
            crate::error::FrameworkError::ParamError { param_name } => {
                serde_json::json!({
                    "message": format!("Missing required parameter: {}", param_name)
                })
            }
            crate::error::FrameworkError::ValidationError { field, message } => {
                serde_json::json!({
                    "message": "Validation failed",
                    "field": field,
                    "error": message
                })
            }
            crate::error::FrameworkError::Validation(errors) => {
                // Laravel/Inertia-compatible validation error format
                errors.to_json()
            }
            crate::error::FrameworkError::Unauthorized => {
                serde_json::json!({
                    "message": "This action is unauthorized."
                })
            }
            _ => {
                serde_json::json!({
                    "message": err.to_string()
                })
            }
        };
        if let Some(hint_text) = hint {
            if let Some(obj) = body.as_object_mut() {
                obj.insert("hint".to_string(), serde_json::Value::String(hint_text));
            }
        }
        HttpResponse::json(body).status(status)
    }
}

/// Auto-convert AppError to HttpResponse
///
/// This enables using the `?` operator in controller handlers with AppError.
impl From<crate::error::AppError> for HttpResponse {
    fn from(err: crate::error::AppError) -> HttpResponse {
        // Convert AppError -> FrameworkError -> HttpResponse
        let framework_err: crate::error::FrameworkError = err.into();
        framework_err.into()
    }
}

/// Inertia-aware HTTP Redirect response builder.
///
/// Unlike standard `Redirect`, this respects the Inertia protocol:
/// - For Inertia XHR requests from POST/PUT/PATCH/DELETE, uses 303 status
/// - Includes X-Inertia header in responses to Inertia requests
/// - Falls back to standard 302 for non-Inertia requests
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::{InertiaRedirect, Request, Response};
///
/// pub async fn store(req: Request) -> Response {
///     // ... create record ...
///     InertiaRedirect::to(&req, "/items").into()
/// }
/// ```
pub struct InertiaRedirect<'a> {
    request: &'a crate::http::Request,
    location: String,
    query_params: Vec<(String, String)>,
}

impl<'a> InertiaRedirect<'a> {
    /// Create a redirect that respects Inertia protocol.
    pub fn to(request: &'a crate::http::Request, path: impl Into<String>) -> Self {
        Self {
            request,
            location: path.into(),
            query_params: Vec::new(),
        }
    }

    /// Add a query parameter.
    pub fn query(mut self, key: &str, value: impl Into<String>) -> Self {
        self.query_params.push((key.to_string(), value.into()));
        self
    }

    fn build_url(&self) -> String {
        if self.query_params.is_empty() {
            self.location.clone()
        } else {
            let query = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&");
            format!("{}?{}", self.location, query)
        }
    }

    fn is_post_like_method(&self) -> bool {
        matches!(
            self.request.method().as_str(),
            "POST" | "PUT" | "PATCH" | "DELETE"
        )
    }
}

impl From<InertiaRedirect<'_>> for Response {
    fn from(redirect: InertiaRedirect<'_>) -> Response {
        let url = redirect.build_url();
        let is_inertia = redirect.request.is_inertia();
        let is_post_like = redirect.is_post_like_method();

        if is_inertia {
            // Use 303 for POST-like methods to force GET on redirect
            let status = if is_post_like { 303 } else { 302 };
            Ok(HttpResponse::new()
                .status(status)
                .header("X-Inertia", "true")
                .header("Location", url))
        } else {
            // Standard redirect for non-Inertia requests
            Ok(HttpResponse::new().status(302).header("Location", url))
        }
    }
}
