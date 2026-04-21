//! Pre-route middleware registry.
//!
//! Pre-route middleware runs before route matching, allowing path rewrites that
//! affect which route handler is selected. Standard global middleware runs after
//! route matching (inside the handler chain) and cannot influence routing.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro::{pre_route_middleware, PreRouteMiddleware, Request, HttpResponse};
//! use async_trait::async_trait;
//!
//! pub struct HostMiddleware;
//!
//! #[async_trait]
//! impl PreRouteMiddleware for HostMiddleware {
//!     async fn rewrite(&self, request: Request) -> Result<Request, HttpResponse> {
//!         // Inspect Host header, call request.set_path() if needed, then:
//!         Ok(request)
//!         // Or to short-circuit:
//!         // Err(HttpResponse::new().status(404).set_body("Not found"))
//!     }
//! }
//!
//! // In bootstrap.rs:
//! pre_route_middleware!(HostMiddleware::new());
//! ```

use crate::http::{HttpResponse, Request};
use async_trait::async_trait;
use std::sync::RwLock;
use std::sync::{Arc, OnceLock};

/// Trait for middleware that runs before route matching.
///
/// Implement `rewrite` to inspect and/or rewrite the request path. The method
/// receives ownership of the request and must return either the (possibly modified)
/// request to continue routing, or an `HttpResponse` to short-circuit immediately.
///
/// Short-circuiting is appropriate for cases like unknown custom domains (return 404)
/// where the request should never reach route matching.
#[async_trait]
pub trait PreRouteMiddleware: Send + Sync {
    /// Inspect and optionally rewrite the request before route matching.
    ///
    /// - Return `Ok(request)` to continue (with or without path rewrite via `set_path`).
    /// - Return `Err(response)` to short-circuit — the response is sent immediately
    ///   and route matching is skipped.
    async fn rewrite(&self, request: Request) -> Result<Request, HttpResponse>;
}

/// Type alias for a boxed pre-route middleware.
pub type BoxedPreRouteMiddleware = Arc<dyn PreRouteMiddleware>;

/// Global pre-route middleware registry.
static PRE_ROUTE_MIDDLEWARE: OnceLock<RwLock<Vec<BoxedPreRouteMiddleware>>> = OnceLock::new();

/// Register a pre-route middleware that runs before route matching on every request.
///
/// Called by the `pre_route_middleware!` macro. Middleware runs in registration order.
pub fn register_pre_route_middleware<M: PreRouteMiddleware + 'static>(middleware: M) {
    let registry = PRE_ROUTE_MIDDLEWARE.get_or_init(|| RwLock::new(Vec::new()));
    if let Ok(mut vec) = registry.write() {
        vec.push(Arc::new(middleware));
    }
}

/// Get all registered pre-route middleware.
///
/// Used internally by `server.rs` before route matching.
pub fn get_pre_route_middleware() -> Vec<BoxedPreRouteMiddleware> {
    PRE_ROUTE_MIDDLEWARE
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|vec| vec.clone())
        .unwrap_or_default()
}
