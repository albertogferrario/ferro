//! CORS middleware for Ferro framework
//!
//! Adds Cross-Origin Resource Sharing headers to responses and handles
//! OPTIONS preflight requests. Suitable for public API endpoints consumed
//! by browser clients on different origins (e.g. custom-domain frontends).
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro::middleware::Cors;
//!
//! // Permissive: allow all origins
//! group!("/api/v1").middleware(Cors::permissive()).routes(|r| {
//!     r.get("/products", products::index);
//! });
//!
//! // Restrictive: allow specific origins only
//! group!("/api/v1").middleware(
//!     Cors::new()
//!         .allow_origins(vec!["https://example.com", "https://app.example.com"])
//!         .allow_methods(vec!["GET", "POST"])
//!         .allow_headers(vec!["Content-Type", "Authorization"]),
//! ).routes(|r| {
//!     r.get("/products", products::index);
//! });
//! ```

use crate::http::{HttpResponse, Request, Response};
use crate::middleware::{Middleware, Next};
use async_trait::async_trait;

/// CORS middleware
///
/// Appends CORS response headers and short-circuits OPTIONS preflight requests
/// with a 204 No Content response. Use [`Cors::permissive()`] for open APIs or
/// [`Cors::new()`] + builder methods for origin-restricted endpoints.
pub struct Cors {
    origins: Origins,
    methods: Vec<String>,
    headers: Vec<String>,
    max_age: u32,
}

enum Origins {
    Any,
    List(Vec<String>),
}

impl Cors {
    /// Create a new `Cors` with permissive defaults
    ///
    /// Allows all origins (`*`), GET/POST/OPTIONS methods, and Content-Type/Accept headers.
    pub fn permissive() -> Self {
        Self {
            origins: Origins::Any,
            methods: vec!["GET".into(), "POST".into(), "OPTIONS".into()],
            headers: vec!["Content-Type".into(), "Accept".into()],
            max_age: 86400,
        }
    }

    /// Create a new `Cors` builder with no allowed origins
    ///
    /// Call [`allow_origins`](Self::allow_origins) to configure allowed origins.
    pub fn new() -> Self {
        Self {
            origins: Origins::List(Vec::new()),
            methods: vec!["GET".into(), "POST".into(), "OPTIONS".into()],
            headers: vec!["Content-Type".into(), "Accept".into()],
            max_age: 86400,
        }
    }

    /// Set the list of allowed origins
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Cors::new().allow_origins(vec!["https://example.com", "https://app.example.com"])
    /// ```
    pub fn allow_origins<I, S>(mut self, origins: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.origins = Origins::List(origins.into_iter().map(Into::into).collect());
        self
    }

    /// Set the allowed HTTP methods
    pub fn allow_methods<I, S>(mut self, methods: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.methods = methods.into_iter().map(Into::into).collect();
        self
    }

    /// Set the allowed request headers
    pub fn allow_headers<I, S>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.headers = headers.into_iter().map(Into::into).collect();
        self
    }

    /// Set the preflight cache duration in seconds (default: 86400)
    pub fn max_age(mut self, seconds: u32) -> Self {
        self.max_age = seconds;
        self
    }

    /// Determine the `Access-Control-Allow-Origin` value for a given request origin
    fn allowed_origin(&self, request_origin: Option<&str>) -> Option<String> {
        match &self.origins {
            Origins::Any => Some("*".into()),
            Origins::List(list) => {
                let origin = request_origin?;
                if list.iter().any(|o| o == origin) {
                    Some(origin.to_string())
                } else {
                    None
                }
            }
        }
    }

    /// Apply CORS headers to a response
    fn apply(&self, response: HttpResponse, origin: &str) -> HttpResponse {
        response
            .header("Access-Control-Allow-Origin", origin)
            .header("Access-Control-Allow-Methods", self.methods.join(", "))
            .header("Access-Control-Allow-Headers", self.headers.join(", "))
            .header("Access-Control-Max-Age", self.max_age.to_string())
    }
}

impl Default for Cors {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for Cors {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let request_origin = request.header("Origin").map(|s| s.to_string());
        let origin = self.allowed_origin(request_origin.as_deref());

        // Preflight: respond immediately without reaching the handler
        if request.method() == "OPTIONS" {
            let response = HttpResponse::new().status(204);
            return match origin {
                Some(ref o) => Ok(self.apply(response, o)),
                None => Ok(response),
            };
        }

        let response = next(request).await;

        match origin {
            Some(ref o) => match response {
                Ok(r) => Ok(self.apply(r, o)),
                Err(r) => Err(self.apply(r, o)),
            },
            None => response,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissive_allows_any_origin() {
        let cors = Cors::permissive();
        assert!(matches!(cors.origins, Origins::Any));
        assert_eq!(cors.allowed_origin(Some("https://example.com")), Some("*".into()));
        assert_eq!(cors.allowed_origin(None), Some("*".into()));
    }

    #[test]
    fn test_allow_origins_list() {
        let cors = Cors::new().allow_origins(vec!["https://a.com", "https://b.com"]);
        assert_eq!(cors.allowed_origin(Some("https://a.com")), Some("https://a.com".into()));
        assert_eq!(cors.allowed_origin(Some("https://b.com")), Some("https://b.com".into()));
        assert_eq!(cors.allowed_origin(Some("https://c.com")), None);
        assert_eq!(cors.allowed_origin(None), None);
    }

    #[test]
    fn test_builder_methods() {
        let cors = Cors::new()
            .allow_origins(vec!["https://x.com"])
            .allow_methods(vec!["GET", "POST", "PUT"])
            .allow_headers(vec!["Authorization", "Content-Type"])
            .max_age(3600);

        assert_eq!(cors.methods, vec!["GET", "POST", "PUT"]);
        assert_eq!(cors.headers, vec!["Authorization", "Content-Type"]);
        assert_eq!(cors.max_age, 3600);
    }

    #[test]
    fn test_default_is_restrictive() {
        let cors = Cors::default();
        // No origins configured — should not allow anything
        assert_eq!(cors.allowed_origin(Some("https://any.com")), None);
    }
}
