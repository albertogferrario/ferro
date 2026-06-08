use super::body::FerroBody;
use super::cookie::Cookie;
use bytes::Bytes;
use http_body_util::Full;

/// HTTP Response builder providing Laravel-like response creation
#[derive(Debug)]
pub struct HttpResponse {
    status: u16,
    body: Bytes,
    headers: Vec<(String, String)>,
}

/// Response type alias - allows using `?` operator for early returns
pub type Response = Result<HttpResponse, HttpResponse>;

impl HttpResponse {
    /// Create an empty 200 OK response.
    pub fn new() -> Self {
        Self {
            status: 200,
            body: Bytes::new(),
            headers: Vec::new(),
        }
    }

    /// Create a response with a string body
    pub fn text(body: impl Into<String>) -> Self {
        let s: String = body.into();
        Self {
            status: 200,
            body: Bytes::from(s),
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
        }
    }

    /// Create a JSON response from a serde_json::Value
    pub fn json(body: serde_json::Value) -> Self {
        Self {
            status: 200,
            body: Bytes::from(body.to_string()),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
        }
    }

    /// Create a response with raw binary data.
    ///
    /// No default Content-Type is set; the caller must add one via `.header()`.
    pub fn bytes(body: impl Into<Bytes>) -> Self {
        Self {
            status: 200,
            body: body.into(),
            headers: vec![],
        }
    }

    /// Create a file download response with Content-Disposition header.
    ///
    /// Auto-detects Content-Type from the filename extension using `mime_guess`.
    /// Falls back to `application/octet-stream` for unknown extensions.
    /// The filename is sanitized against header injection (control characters
    /// and quote marks are stripped).
    pub fn download(body: impl Into<Bytes>, filename: &str) -> Self {
        let safe_name: String = filename
            .chars()
            .filter(|c| !c.is_control() && *c != '"' && *c != '\\')
            .collect();

        let content_type = mime_guess::from_path(&safe_name)
            .first()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        Self {
            status: 200,
            body: body.into(),
            headers: vec![
                ("Content-Type".to_string(), content_type),
                (
                    "Content-Disposition".to_string(),
                    format!("attachment; filename=\"{safe_name}\""),
                ),
            ],
        }
    }

    /// Set the response body
    pub fn set_body(mut self, body: impl Into<String>) -> Self {
        let s: String = body.into();
        self.body = Bytes::from(s);
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
    ///
    /// Returns an empty string for non-UTF-8 bodies (e.g. binary data).
    /// Use `body_bytes()` to access raw binary data.
    pub fn body(&self) -> &str {
        std::str::from_utf8(&self.body).unwrap_or("")
    }

    /// Get the response body as raw bytes.
    pub fn body_bytes(&self) -> &Bytes {
        &self.body
    }

    /// Set a response header, replacing any existing header with the same name.
    ///
    /// The name match is case-insensitive (ASCII). Use [`append_header`](Self::append_header)
    /// for legitimately multi-value headers such as `Set-Cookie`, `Vary`, or `Link`.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let name = name.into();
        self.headers.retain(|(n, _)| !n.eq_ignore_ascii_case(&name));
        self.headers.push((name, value.into()));
        self
    }

    /// Append a response header without removing any existing entry with the same name.
    ///
    /// Intended for headers that legitimately carry multiple values on separate lines,
    /// such as `Set-Cookie` (RFC 6265 §4.1), `Vary`, and `Link`. For single-value
    /// headers like `Content-Type` or `Location`, use [`header`](Self::header) instead.
    pub fn append_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Get the response headers as a borrowed slice.
    ///
    /// Returns all header entries in insertion order. Multi-value headers
    /// (e.g. `Set-Cookie`) appear as multiple entries with the same name.
    pub fn headers(&self) -> &[(String, String)] {
        &self.headers
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
        self.append_header("Set-Cookie", header_value)
    }

    /// Wrap this response in Ok() for use as Response type
    pub fn ok(self) -> Response {
        Ok(self)
    }

    /// Convert to hyper response.
    ///
    /// Returns `hyper::Response<FerroBody>` — the buffered body is wrapped as
    /// `FerroBody::Full`. For streaming SSE responses, use `HttpResponse::sse()` instead,
    /// which returns the response with a `FerroBody::Stream` body directly.
    pub fn into_hyper(self) -> hyper::Response<FerroBody> {
        let mut builder = hyper::Response::builder().status(self.status);

        for (name, value) in self.headers {
            builder = builder.header(name, value);
        }

        builder.body(FerroBody::Full(Full::new(self.body))).unwrap()
    }

    /// Create an SSE streaming response with the correct headers, returning a channel
    /// sender and the ready-to-send hyper response.
    ///
    /// Sets `Content-Type: text/event-stream`, `Cache-Control: no-cache`,
    /// `Connection: keep-alive`, and `X-Accel-Buffering: no` (disables nginx proxy
    /// buffering). The response body is `FerroBody::Stream` — structurally guaranteed
    /// to never be whole-body buffered (D-06).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[handler]
    /// pub async fn stream(req: Request) -> Response {
    ///     let (tx, response) = HttpResponse::sse_channel(16);
    ///     tokio::spawn(async move {
    ///         tx.send(SseEvent::data("hello")).await.ok();
    ///         // tx dropped → stream ends
    ///     });
    ///     Ok(response.into())
    /// }
    /// ```
    pub fn sse_channel(
        buffer: usize,
    ) -> (
        tokio::sync::mpsc::Sender<super::sse::SseEvent>,
        hyper::Response<FerroBody>,
    ) {
        let (tx, stream) = super::sse::SseStream::channel(buffer);
        let response = hyper::Response::builder()
            .status(200)
            .header("Content-Type", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .header("X-Accel-Buffering", "no")
            .body(FerroBody::Stream(stream))
            .unwrap();
        (tx, response)
    }

    /// Create an SSE streaming response from an existing [`SseStream`](super::sse::SseStream).
    ///
    /// Sets the same four required SSE headers as [`sse_channel`](Self::sse_channel).
    /// Use this when you already have an `SseStream` (e.g. created via
    /// [`SseStream::channel`](super::sse::SseStream::channel)).
    pub fn sse(stream: super::sse::SseStream) -> hyper::Response<FerroBody> {
        hyper::Response::builder()
            .status(200)
            .header("Content-Type", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .header("X-Accel-Buffering", "no")
            .body(FerroBody::Stream(stream))
            .unwrap()
    }
}

impl Default for HttpResponse {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for Response to enable method chaining on macros
pub trait ResponseExt {
    /// Set the HTTP status code.
    fn status(self, code: u16) -> Self;
    /// Set a response header, replacing any existing header with the same name (case-insensitive).
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

    /// Create a redirect that returns the user to the page that triggered
    /// the current request, derived from the `Referer` header.
    ///
    /// Preserves query string and fragment from the source page so tab
    /// selection (`?tab=note`), pagination cursors, and scroll-restoration
    /// keys (`scroll_preserve.rs`) survive form POSTs.
    ///
    /// Falls back to `fallback` when the Referer is absent or points
    /// off-origin. Same-origin is enforced by requiring the Referer's host
    /// to match the request's `Host` header (or the Referer to be already
    /// a relative path) — protects against open-redirect via spoofed Referer.
    pub fn back(req: &crate::http::Request, fallback: impl Into<String>) -> Self {
        let location = same_origin_path_from_referer(req).unwrap_or_else(|| fallback.into());
        Self {
            location,
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

/// Extracts a same-origin `/path?query#fragment` string from the request's
/// `Referer` header, returning `None` when the header is absent, malformed,
/// or points off-origin.
///
/// Same-origin rule: when the Referer is an absolute URL (`scheme://host/...`)
/// the host must equal the request's `Host` header. When the Referer is
/// already a relative path it is accepted as-is. Scheme-relative URLs
/// (`//evil.com/x`) are rejected.
fn same_origin_path_from_referer(req: &crate::http::Request) -> Option<String> {
    let referer = req.header("referer")?;
    // Scheme-relative URLs (//host/...) — reject; they bypass scheme check.
    if referer.starts_with("//") {
        return None;
    }
    // Already-relative path — accept as-is.
    if referer.starts_with('/') {
        return Some(referer.to_string());
    }
    // Absolute URL — strip `scheme://host` prefix and verify host matches.
    let rest = referer
        .strip_prefix("http://")
        .or_else(|| referer.strip_prefix("https://"))?;
    let (referer_host, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let request_host = req.header("host")?;
    if referer_host == request_host {
        Some(path.to_string())
    } else {
        None
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

/// Auto-convert ferro_projections::Error to HttpResponse
///
/// This enables using the `?` operator in controller handlers with projection errors.
#[cfg(feature = "projections")]
impl From<ferro_projections::Error> for HttpResponse {
    fn from(err: ferro_projections::Error) -> HttpResponse {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_constructor() {
        let resp = HttpResponse::bytes(vec![0xFF, 0xFE, 0x00]);
        assert_eq!(resp.body_bytes().as_ref(), &[0xFF, 0xFE, 0x00]);
        assert_eq!(resp.status_code(), 200);
        assert!(
            resp.headers.is_empty(),
            "bytes() should set no default headers"
        );
    }

    #[test]
    fn test_bytes_from_vec_u8() {
        let resp = HttpResponse::bytes(vec![1, 2, 3]);
        assert_eq!(resp.body_bytes().len(), 3);
    }

    #[test]
    fn test_bytes_with_content_type() {
        let resp = HttpResponse::bytes(b"PNG data".to_vec()).header("Content-Type", "image/png");
        let ct = resp
            .headers
            .iter()
            .find(|(k, _)| k == "Content-Type")
            .map(|(_, v)| v.as_str());
        assert_eq!(ct, Some("image/png"));
    }

    #[test]
    fn test_download_constructor() {
        let resp = HttpResponse::download(b"pdf content".to_vec(), "report.pdf");
        let ct = resp
            .headers
            .iter()
            .find(|(k, _)| k == "Content-Type")
            .map(|(_, v)| v.as_str());
        assert_eq!(ct, Some("application/pdf"));

        let cd = resp
            .headers
            .iter()
            .find(|(k, _)| k == "Content-Disposition")
            .map(|(_, v)| v.as_str());
        assert_eq!(cd, Some("attachment; filename=\"report.pdf\""));
    }

    #[test]
    fn test_download_unknown_extension() {
        let resp = HttpResponse::download(b"data".to_vec(), "file.zzqx");
        let ct = resp
            .headers
            .iter()
            .find(|(k, _)| k == "Content-Type")
            .map(|(_, v)| v.as_str());
        assert_eq!(ct, Some("application/octet-stream"));
    }

    #[test]
    fn test_download_filename_sanitization() {
        let resp = HttpResponse::download(b"data".to_vec(), "evil\"file\nname.pdf");
        let cd = resp
            .headers
            .iter()
            .find(|(k, _)| k == "Content-Disposition")
            .map(|(_, v)| v.as_str())
            .unwrap();
        assert!(
            !cd.contains('"') || cd.matches('"').count() == 2,
            "filename should be properly quoted"
        );
        assert!(!cd.contains('\n'), "filename should not contain newlines");
    }

    #[test]
    fn test_text_still_works() {
        let resp = HttpResponse::text("hello");
        assert_eq!(resp.body(), "hello");
        assert_eq!(resp.body_bytes().as_ref(), b"hello");
    }

    #[test]
    fn test_json_still_works() {
        let resp = HttpResponse::json(serde_json::json!({"ok": true}));
        let body = resp.body();
        assert!(!body.is_empty(), "json body should not be empty");
        let parsed: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(parsed["ok"], true);
        assert!(!resp.body_bytes().is_empty());
    }

    #[test]
    fn test_body_returns_empty_for_binary() {
        let resp = HttpResponse::bytes(vec![0xFF, 0xFE]);
        assert_eq!(resp.body(), "");
    }

    #[test]
    fn test_into_hyper_preserves_binary() {
        use http_body_util::BodyExt;

        let data = vec![0xFF, 0x00, 0xFE];
        let resp = HttpResponse::bytes(data.clone());
        let hyper_resp = resp.into_hyper();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let collected =
            rt.block_on(async { hyper_resp.into_body().collect().await.unwrap().to_bytes() });
        assert_eq!(collected.as_ref(), &data);
    }

    #[test]
    fn test_header_replaces_existing() {
        let resp = HttpResponse::text("x").header("Content-Type", "text/html");
        let ct: Vec<_> = resp
            .headers()
            .iter()
            .filter(|(k, _)| k.eq_ignore_ascii_case("Content-Type"))
            .collect();
        assert_eq!(ct.len(), 1, "expected exactly one Content-Type entry");
        assert_eq!(ct[0].1, "text/html");
    }

    #[test]
    fn test_multi_cookie_preserved() {
        let resp = HttpResponse::new()
            .cookie(Cookie::new("a", "1"))
            .cookie(Cookie::new("b", "2"));
        let cookies: Vec<_> = resp
            .headers()
            .iter()
            .filter(|(k, _)| k == "Set-Cookie")
            .collect();
        assert_eq!(
            cookies.len(),
            2,
            "both Set-Cookie entries must be preserved"
        );
    }

    #[test]
    fn test_header_case_insensitive_replace() {
        let resp = HttpResponse::new()
            .append_header("content-type", "text/plain")
            .header("Content-Type", "text/html");
        let ct: Vec<_> = resp
            .headers()
            .iter()
            .filter(|(k, _)| k.eq_ignore_ascii_case("Content-Type"))
            .collect();
        assert_eq!(ct.len(), 1, "lowercase prior entry must be replaced");
        assert_eq!(ct[0].1, "text/html");
    }

    #[test]
    fn test_append_header_does_not_replace() {
        let resp = HttpResponse::new()
            .append_header("X-Tag", "a")
            .append_header("X-Tag", "b");
        let count = resp.headers().iter().filter(|(k, _)| k == "X-Tag").count();
        assert_eq!(count, 2, "append_header must not strip existing entries");
    }

    #[test]
    fn test_headers_accessor() {
        let resp = HttpResponse::text("x");
        assert!(
            !resp.headers().is_empty(),
            "headers() accessor should return the prepopulated Content-Type"
        );
    }
}
