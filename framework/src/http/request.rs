use super::body::{collect_body, parse_form, parse_json};
use super::cookie::parse_cookies;
use super::ParamError;
use crate::error::FrameworkError;
use bytes::Bytes;
use serde::de::DeserializeOwned;
use std::any::{Any, TypeId};
use std::collections::HashMap;

/// HTTP Request wrapper providing Laravel-like access to request data
pub struct Request {
    inner: hyper::Request<hyper::body::Incoming>,
    params: HashMap<String, String>,
    extensions: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    /// Route pattern for metrics (e.g., "/users/{id}" instead of "/users/123")
    route_pattern: Option<String>,
    /// Success-side overrides recorded by `req.flash(...)` / `req.redirect_to(...)`.
    /// Read by the `#[action]` macro runtime after the handler body returns.
    action_overrides: crate::http::action::ActionOverrides,
}

impl Request {
    /// Create a new request from a raw hyper request.
    pub fn new(inner: hyper::Request<hyper::body::Incoming>) -> Self {
        Self {
            inner,
            params: HashMap::new(),
            extensions: HashMap::new(),
            route_pattern: None,
            action_overrides: crate::http::action::ActionOverrides::default(),
        }
    }

    /// Attach route parameters extracted from the URL path.
    pub fn with_params(mut self, params: HashMap<String, String>) -> Self {
        self.params = params;
        self
    }

    /// Set the route pattern (e.g., "/users/{id}")
    pub fn with_route_pattern(mut self, pattern: String) -> Self {
        self.route_pattern = Some(pattern);
        self
    }

    /// Get the route pattern for metrics grouping
    pub fn route_pattern(&self) -> Option<String> {
        self.route_pattern.clone()
    }

    /// Insert a value into the request extensions (type-map pattern)
    ///
    /// This is async-safe unlike thread-local storage.
    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) {
        self.extensions.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Get a reference to a value from the request extensions
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.extensions
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Get a mutable reference to a value from the request extensions
    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.extensions
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Get the request method
    pub fn method(&self) -> &hyper::Method {
        self.inner.method()
    }

    /// Get the request path
    pub fn path(&self) -> &str {
        self.inner.uri().path()
    }

    /// Rewrite the request path (server-side only — the browser URL is unchanged).
    ///
    /// Replaces the URI path component while preserving the scheme, authority, and
    /// query string. Used by pre-route middleware (e.g. `HostMiddleware`) to map
    /// custom-domain requests onto internal slug-based routes before routing occurs.
    ///
    /// `new_path` must begin with `/`. Panics in debug mode if it does not.
    pub fn set_path(&mut self, new_path: &str) {
        debug_assert!(
            new_path.starts_with('/'),
            "set_path: path must begin with '/', got {new_path:?}"
        );
        let old_uri = self.inner.uri();
        // Preserve scheme, authority, and query string; replace path only.
        let mut parts = old_uri.clone().into_parts();
        let path_and_query = match old_uri.query() {
            Some(q) => format!("{new_path}?{q}"),
            None => new_path.to_string(),
        };
        parts.path_and_query = Some(
            path_and_query
                .parse()
                .unwrap_or_else(|_| new_path.parse().expect("invalid path")),
        );
        if let Ok(new_uri) = http::Uri::from_parts(parts) {
            *self.inner.uri_mut() = new_uri;
        }
    }

    /// Get a route parameter by name (e.g., /users/{id})
    /// Returns Err(ParamError) if the parameter is missing, enabling use of `?` operator
    pub fn param(&self, name: &str) -> Result<&str, ParamError> {
        self.params
            .get(name)
            .map(|s| s.as_str())
            .ok_or_else(|| ParamError {
                param_name: name.to_string(),
            })
    }

    /// Get a route parameter parsed as a specific type
    ///
    /// Combines `param()` with parsing, returning a typed value.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// pub async fn show(req: Request) -> Response {
    ///     let id: i32 = req.param_as("id")?;
    ///     // ...
    /// }
    /// ```
    pub fn param_as<T: std::str::FromStr>(&self, name: &str) -> Result<T, ParamError>
    where
        T::Err: std::fmt::Display,
    {
        let value = self.param(name)?;
        value.parse::<T>().map_err(|e| ParamError {
            param_name: format!("{name} (parse error: {e})"),
        })
    }

    /// Get all route parameters
    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    /// Get a query string parameter by name
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // URL: /users?page=2&limit=10
    /// let page = req.query("page"); // Some("2")
    /// let sort = req.query("sort"); // None
    /// ```
    pub fn query(&self, name: &str) -> Option<String> {
        self.inner.uri().query().and_then(|q| {
            form_urlencoded::parse(q.as_bytes())
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.into_owned())
        })
    }

    /// Get a query string parameter or a default value
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // URL: /users?page=2
    /// let page = req.query_or("page", "1"); // "2"
    /// let limit = req.query_or("limit", "10"); // "10"
    /// ```
    pub fn query_or(&self, name: &str, default: &str) -> String {
        self.query(name).unwrap_or_else(|| default.to_string())
    }

    /// Get a query string parameter parsed as a specific type
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // URL: /users?page=2&limit=10
    /// let page: Option<i32> = req.query_as("page"); // Some(2)
    /// ```
    pub fn query_as<T: std::str::FromStr>(&self, name: &str) -> Option<T> {
        self.query(name).and_then(|v| v.parse().ok())
    }

    /// Get a query string parameter parsed as a specific type, or a default
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // URL: /users?page=2
    /// let page: i32 = req.query_as_or("page", 1); // 2
    /// let limit: i32 = req.query_as_or("limit", 10); // 10
    /// ```
    pub fn query_as_or<T: std::str::FromStr>(&self, name: &str, default: T) -> T {
        self.query_as(name).unwrap_or(default)
    }

    // ── Phase 137: validation flash round-trip helpers ────────────────────────

    /// Read a previously-submitted form value from session flash.
    ///
    /// After a POST handler calls `ValidationError::with_old_input(&data).redirect_back(...)`,
    /// the GET handler retrieves the value with `req.old("field_name")` and passes it as
    /// `InputProps.default_value` to repopulate the form.
    ///
    /// Reads from `_flash.old._old_input.<field>` without clearing (read-only semantics).
    /// Flash aging (move new→old→deleted) is handled by the session middleware at request
    /// boundaries, so multiple reads in the same GET handler are safe.
    ///
    /// Returns `None` when no flash value exists, no session is active, or the key is absent.
    pub fn old(&self, field: &str) -> Option<String> {
        let key = format!("_flash.old._old_input.{field}");
        crate::session::session().and_then(|s| s.get::<String>(&key))
    }

    /// Read the first validation error message for a field from session flash.
    ///
    /// After a POST handler calls `errors.redirect_back(...)`, the GET handler calls
    /// `req.validation_error("field_name")` and passes the result as `InputProps.error`.
    ///
    /// Reads from `_flash.old._validation_errors` without clearing (read-only semantics).
    ///
    /// Returns `None` when no flash errors exist, no session is active, or the field has no error.
    pub fn validation_error(&self, field: &str) -> Option<String> {
        let errors: Option<std::collections::HashMap<String, Vec<String>>> =
            crate::session::session().and_then(|s| {
                s.get::<std::collections::HashMap<String, Vec<String>>>(
                    "_flash.old._validation_errors",
                )
            });
        errors.and_then(|map| map.get(field).and_then(|v| v.first()).cloned())
    }

    /// Returns `true` when any validation errors were flashed from a prior request.
    ///
    /// Useful for rendering a form-wide error summary banner.
    pub fn has_validation_errors(&self) -> bool {
        crate::session::session()
            .and_then(|s| {
                s.get::<std::collections::HashMap<String, Vec<String>>>(
                    "_flash.old._validation_errors",
                )
            })
            .map(|m| !m.is_empty())
            .unwrap_or(false)
    }

    /// Get the inner hyper request
    pub fn inner(&self) -> &hyper::Request<hyper::body::Incoming> {
        &self.inner
    }

    /// Get a header value by name
    pub fn header(&self, name: &str) -> Option<&str> {
        self.inner.headers().get(name).and_then(|v| v.to_str().ok())
    }

    /// Get the Content-Type header
    pub fn content_type(&self) -> Option<&str> {
        self.header("content-type")
    }

    /// Resolve the URL the current request was triggered from, falling
    /// back to `fallback` when the `Referer` is absent, malformed, or
    /// points off-origin.
    ///
    /// Use to capture a "back" target at handler entry before the request
    /// body is consumed (e.g. before [`body_bytes`](Self::body_bytes)). The
    /// returned `String` then feeds [`crate::http::Redirect::to`] to send
    /// the user back to where they came from, preserving query strings
    /// (e.g. `?tab=note`) and any other URL state.
    ///
    /// Same-origin rule mirrors [`crate::http::Redirect::back`]: absolute
    /// URLs must share the request's `Host`; scheme-relative URLs are
    /// rejected.
    pub fn back_or(&self, fallback: impl Into<String>) -> String {
        let referer = match self.header("referer") {
            Some(r) => r,
            None => return fallback.into(),
        };
        if referer.starts_with("//") {
            return fallback.into();
        }
        if referer.starts_with('/') {
            return referer.to_string();
        }
        let rest = match referer
            .strip_prefix("http://")
            .or_else(|| referer.strip_prefix("https://"))
        {
            Some(r) => r,
            None => return fallback.into(),
        };
        let (referer_host, path) = match rest.find('/') {
            Some(i) => (&rest[..i], &rest[i..]),
            None => (rest, "/"),
        };
        let request_host = match self.header("host") {
            Some(h) => h,
            None => return fallback.into(),
        };
        if referer_host == request_host {
            path.to_string()
        } else {
            fallback.into()
        }
    }

    /// Check if this is an Inertia XHR request
    pub fn is_inertia(&self) -> bool {
        self.header("X-Inertia")
            .map(|v| v == "true")
            .unwrap_or(false)
    }

    /// Get all cookies from the request
    ///
    /// Parses the Cookie header and returns a HashMap of cookie names to values.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let cookies = req.cookies();
    /// if let Some(session) = cookies.get("session") {
    ///     println!("Session: {}", session);
    /// }
    /// ```
    pub fn cookies(&self) -> HashMap<String, String> {
        self.header("Cookie").map(parse_cookies).unwrap_or_default()
    }

    /// Get a specific cookie value by name
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(session_id) = req.cookie("session") {
    ///     // Use session_id
    /// }
    /// ```
    pub fn cookie(&self, name: &str) -> Option<String> {
        self.cookies().get(name).cloned()
    }

    /// Get the Inertia version from request headers
    pub fn inertia_version(&self) -> Option<&str> {
        self.header("X-Inertia-Version")
    }

    /// Get partial component name for partial reloads
    pub fn inertia_partial_component(&self) -> Option<&str> {
        self.header("X-Inertia-Partial-Component")
    }

    /// Get partial data keys for partial reloads
    pub fn inertia_partial_data(&self) -> Option<Vec<&str>> {
        self.header("X-Inertia-Partial-Data")
            .map(|v| v.split(',').collect())
    }

    /// Consume the request and collect the body as bytes
    pub async fn body_bytes(self) -> Result<(RequestParts, Bytes), FrameworkError> {
        let content_type = self
            .inner
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let params = self.params;
        let bytes = collect_body(self.inner.into_body()).await?;

        Ok((
            RequestParts {
                params,
                content_type,
            },
            bytes,
        ))
    }

    /// Parse the request body as JSON
    ///
    /// Consumes the request since the body can only be read once.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[derive(Deserialize)]
    /// struct CreateUser { name: String, email: String }
    ///
    /// pub async fn store(req: Request) -> Response {
    ///     let data: CreateUser = req.json().await?;
    ///     // ...
    /// }
    /// ```
    pub async fn json<T: DeserializeOwned>(self) -> Result<T, FrameworkError> {
        let (_, bytes) = self.body_bytes().await?;
        parse_json(&bytes)
    }

    /// Parse the request body as form-urlencoded
    ///
    /// Consumes the request since the body can only be read once.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[derive(Deserialize)]
    /// struct LoginForm { username: String, password: String }
    ///
    /// pub async fn login(req: Request) -> Response {
    ///     let form: LoginForm = req.form().await?;
    ///     // ...
    /// }
    /// ```
    pub async fn form<T: DeserializeOwned>(self) -> Result<T, FrameworkError> {
        let (_, bytes) = self.body_bytes().await?;
        parse_form(&bytes)
    }

    /// Parse the request body as `multipart/form-data`.
    ///
    /// Consumes the request since the body can only be read once.
    /// The per-field byte cap is read from `UPLOAD_MAX_SIZE_MB` (default 10 MiB),
    /// and the per-request field cap from `UPLOAD_MAX_FIELDS` (default 100).
    ///
    /// # Errors
    ///
    /// Returns `FrameworkError::internal(...)` with the literal message
    /// `"Content-Type is not multipart/form-data or missing boundary"` when
    /// the request's `Content-Type` header is absent, malformed, or not a
    /// multipart value.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// pub async fn upload(req: Request) -> Response {
    ///     let form = req.multipart().await?;
    ///     let title = form.field("title").unwrap_or_default();
    ///     let file = form.file("attachment");
    ///     // ...
    /// }
    /// ```
    pub async fn multipart(self) -> Result<super::multipart::MultipartForm, FrameworkError> {
        let content_type = self
            .inner
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_default();
        let body = self.inner.into_body();
        super::multipart::parse_multipart_body(
            body,
            &content_type,
            super::multipart::max_file_bytes(),
            super::multipart::max_fields(),
        )
        .await
    }

    /// Parse the body as multipart/form-data and return the first file
    /// uploaded under `field`.
    ///
    /// Consumes the request since the body can only be read once. Returns
    /// `Ok(None)` when the multipart body parses successfully but contains
    /// no file with that field name.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// pub async fn upload_avatar(req: Request) -> Response {
    ///     let file = req.file("avatar").await?
    ///         .ok_or_else(|| FrameworkError::internal("missing avatar"))?;
    ///     // file.store(&disk, &path).await?;
    ///     Ok(json!({"size": file.size()}))
    /// }
    /// ```
    pub async fn file(
        self,
        field: &str,
    ) -> Result<Option<super::multipart::UploadedFile>, FrameworkError> {
        let mut form = self.multipart().await?;
        Ok(form.files_map.remove(field).and_then(|mut v| {
            if v.is_empty() {
                None
            } else {
                Some(v.swap_remove(0))
            }
        }))
    }

    /// Parse the request body based on Content-Type header
    ///
    /// - `application/json` -> JSON parsing
    /// - `application/x-www-form-urlencoded` -> Form parsing
    /// - Otherwise -> JSON parsing (default)
    ///
    /// Consumes the request since the body can only be read once.
    pub async fn input<T: DeserializeOwned>(self) -> Result<T, FrameworkError> {
        let (parts, bytes) = self.body_bytes().await?;

        match parts.content_type.as_deref() {
            Some(ct) if ct.starts_with("application/x-www-form-urlencoded") => parse_form(&bytes),
            _ => parse_json(&bytes),
        }
    }

    /// Consume the request and return its parts along with the inner hyper request body
    ///
    /// This is used internally by the handler macro for FormRequest extraction.
    pub fn into_parts(self) -> (RequestParts, hyper::body::Incoming) {
        let content_type = self
            .inner
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let params = self.params;
        let body = self.inner.into_body();

        (
            RequestParts {
                params,
                content_type,
            },
            body,
        )
    }
}

impl Request {
    /// Record a success-side flash key for the `#[action]` macro runtime to write
    /// to the session `_action` flash slot when the handler returns `Ok(())`.
    ///
    /// Has no observable effect outside an `#[action]`-decorated handler.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[action(redirect_to = "/dashboard/pagine")]
    /// pub async fn create(req: Request) -> ActionResult {
    ///     let new_id = Page::create(...).await?;
    ///     req.redirect_to(format!("/dashboard/pagine/{new_id}"));
    ///     req.flash("created");
    ///     Ok(())
    /// }
    /// ```
    pub fn flash(&mut self, key: impl Into<String>) {
        self.action_overrides.flash = Some(key.into());
    }

    /// Record a success-side redirect override for the `#[action]` macro runtime
    /// to apply when the handler returns `Ok(())`. The URL is validated as
    /// same-origin (must start with `/`) when applied — external URLs are
    /// silently rejected (T-180-02).
    ///
    /// Has no observable effect outside an `#[action]`-decorated handler.
    pub fn redirect_to(&mut self, url: impl Into<String>) {
        self.action_overrides.redirect_override = Some(url.into());
    }

    /// Internal — read by the `#[action]` macro runtime to apply recorded overrides.
    pub(crate) fn action_overrides(&self) -> &crate::http::action::ActionOverrides {
        &self.action_overrides
    }
}

/// Request parts after body has been separated
///
/// Contains metadata needed for body parsing without the body itself.
#[derive(Clone)]
pub struct RequestParts {
    /// Route parameters extracted from the URL path.
    pub params: HashMap<String, String>,
    /// Content-Type header value, if present.
    pub content_type: Option<String>,
}

#[cfg(test)]
mod tests {
    // Phase 137: unit tests for old() / validation_error() / has_validation_errors().
    //
    // The Request struct wraps hyper::body::Incoming which cannot be constructed
    // in unit tests.  We therefore test the underlying session-reading logic
    // directly (the same code path the methods delegate to) using
    // SESSION_CONTEXT.scope() to inject a session.
    //
    // Full end-to-end round-trips (POST → flash → GET → InputProps) live in the
    // gestiscilo integration test scaffold (validation_roundtrip_tests.rs).

    use crate::session::middleware::SESSION_CONTEXT;
    use crate::session::store::SessionData;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // ── No-session guard tests ────────────────────────────────────────────────

    #[tokio::test]
    async fn test_session_absent_old_returns_none() {
        // Outside any SESSION_CONTEXT scope, session() returns None.
        // old() delegates to session().and_then(...) so it must also return None.
        let val =
            crate::session::session().and_then(|s| s.get::<String>("_flash.old._old_input.email"));
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn test_session_absent_validation_error_returns_none() {
        let val = crate::session::session().and_then(|s| {
            s.get::<HashMap<String, Vec<String>>>("_flash.old._validation_errors")
                .and_then(|map| map.get("email").and_then(|v| v.first()).cloned())
        });
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn test_session_absent_has_validation_errors_false() {
        let val = crate::session::session()
            .and_then(|s| s.get::<HashMap<String, Vec<String>>>("_flash.old._validation_errors"))
            .map(|m| !m.is_empty())
            .unwrap_or(false);
        assert!(!val);
    }

    // ── Session-present tests (direct logic, mirrors Request method bodies) ───

    #[tokio::test]
    async fn test_old_reads_from_flash_old_key() {
        let mut session = SessionData::new("test-id".to_string(), "csrf".to_string());
        // Simulate age_flash_data() having moved the flash to _flash.old.*
        session.put(
            "_flash.old._old_input.email",
            "user@example.com".to_string(),
        );

        let ctx = Arc::new(RwLock::new(Some(session)));
        let val = SESSION_CONTEXT
            .scope(ctx, async {
                crate::session::session()
                    .and_then(|s| s.get::<String>("_flash.old._old_input.email"))
            })
            .await;

        assert_eq!(val, Some("user@example.com".to_string()));
    }

    #[tokio::test]
    async fn test_validation_error_reads_first_message_for_field() {
        let mut session = SessionData::new("test-id".to_string(), "csrf".to_string());
        let mut errors: HashMap<String, Vec<String>> = HashMap::new();
        errors.insert(
            "email".to_string(),
            vec!["Inserisci un indirizzo email valido".to_string()],
        );
        session.put("_flash.old._validation_errors", &errors);

        let ctx = Arc::new(RwLock::new(Some(session)));
        let (email_err, other_err) = SESSION_CONTEXT
            .scope(ctx, async {
                let email_err = crate::session::session().and_then(|s| {
                    s.get::<HashMap<String, Vec<String>>>("_flash.old._validation_errors")
                        .and_then(|map| map.get("email").and_then(|v| v.first()).cloned())
                });
                // Reading the same session twice must not clear the data.
                let other_err = crate::session::session().and_then(|s| {
                    s.get::<HashMap<String, Vec<String>>>("_flash.old._validation_errors")
                        .and_then(|map| map.get("name").and_then(|v| v.first()).cloned())
                });
                (email_err, other_err)
            })
            .await;

        assert_eq!(
            email_err,
            Some("Inserisci un indirizzo email valido".to_string())
        );
        assert_eq!(other_err, None);
    }

    #[tokio::test]
    async fn test_multiple_reads_do_not_clear_flash() {
        // Validates read-only semantics: calling session().get() twice returns
        // the same value (unlike get_flash which clears on read).
        let mut session = SessionData::new("test-id".to_string(), "csrf".to_string());
        session.put("_flash.old._old_input.name", "Mario".to_string());

        let ctx = Arc::new(RwLock::new(Some(session)));
        let (first, second) = SESSION_CONTEXT
            .scope(ctx, async {
                let a = crate::session::session()
                    .and_then(|s| s.get::<String>("_flash.old._old_input.name"));
                let b = crate::session::session()
                    .and_then(|s| s.get::<String>("_flash.old._old_input.name"));
                (a, b)
            })
            .await;

        assert_eq!(first, Some("Mario".to_string()));
        assert_eq!(second, Some("Mario".to_string()));
    }
}
