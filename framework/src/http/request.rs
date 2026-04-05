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
}

impl Request {
    /// Create a new request from a raw hyper request.
    pub fn new(inner: hyper::Request<hyper::body::Incoming>) -> Self {
        Self {
            inner,
            params: HashMap::new(),
            extensions: HashMap::new(),
            route_pattern: None,
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
