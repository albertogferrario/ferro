//! Request trait for framework-agnostic Inertia header extraction.

/// Trait for extracting Inertia-specific data from HTTP requests.
///
/// Implement this trait for your framework's request type to enable
/// Inertia.js integration.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_inertia::InertiaRequest;
///
/// impl InertiaRequest for axum::extract::Request {
///     fn inertia_header(&self, name: &str) -> Option<&str> {
///         self.headers()
///             .get(name)
///             .and_then(|v| v.to_str().ok())
///     }
///
///     fn path(&self) -> &str {
///         self.uri().path()
///     }
/// }
/// ```
pub trait InertiaRequest {
    /// Get a header value by name.
    ///
    /// Used to extract Inertia-specific headers like `X-Inertia`,
    /// `X-Inertia-Version`, and `X-Inertia-Partial-Data`.
    fn inertia_header(&self, name: &str) -> Option<&str>;

    /// Get the request path (URL path component).
    fn path(&self) -> &str;

    /// Check if this is an Inertia XHR request.
    ///
    /// Returns `true` if the `X-Inertia` header is present and set to "true".
    fn is_inertia(&self) -> bool {
        self.inertia_header("X-Inertia")
            .map(|v| v == "true")
            .unwrap_or(false)
    }

    /// Get the Inertia asset version from the request.
    ///
    /// Returns the value of the `X-Inertia-Version` header if present.
    fn inertia_version(&self) -> Option<&str> {
        self.inertia_header("X-Inertia-Version")
    }

    /// Get the partial reload data keys.
    ///
    /// Returns a list of prop keys requested for partial reload via
    /// the `X-Inertia-Partial-Data` header.
    fn inertia_partial_data(&self) -> Option<Vec<&str>> {
        self.inertia_header("X-Inertia-Partial-Data")
            .map(|v| v.split(',').map(str::trim).collect())
    }

    /// Get the component name for partial reload.
    ///
    /// Returns the value of the `X-Inertia-Partial-Component` header.
    fn inertia_partial_component(&self) -> Option<&str> {
        self.inertia_header("X-Inertia-Partial-Component")
    }

    /// Check if the request accepts JSON responses.
    ///
    /// Returns `true` if the `Accept` header contains `application/json`.
    fn accepts_json(&self) -> bool {
        self.inertia_header("Accept")
            .map(|v| v.contains("application/json"))
            .unwrap_or(false)
    }
}
