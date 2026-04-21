//! Pre-route middleware — runs before path extraction and route matching.
//!
//! Use to rewrite request paths or short-circuit responses before routing.
//! Registered via the `pre_route_middleware!` macro in `bootstrap.rs`.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::{async_trait, PreRouteMiddleware, PreRouteResult};
//!
//! pub struct HostMiddleware;
//!
//! #[async_trait]
//! impl PreRouteMiddleware for HostMiddleware {
//!     async fn handle(
//!         &self,
//!         req: hyper::Request<hyper::body::Incoming>,
//!     ) -> PreRouteResult {
//!         // rewrite path or short-circuit
//!         Ok(req)
//!     }
//! }
//! ```

use async_trait::async_trait;
use bytes::Bytes;
use http_body_util::Full;
use std::sync::{Arc, OnceLock, RwLock};

/// Pre-route middleware result: `Ok` continues with the (possibly rewritten) request,
/// `Err` short-circuits and sends the response immediately.
pub type PreRouteResult = Result<
    hyper::Request<hyper::body::Incoming>,
    hyper::Response<Full<Bytes>>,
>;

/// Trait for middleware that runs before path extraction and route matching.
///
/// Implement this when you need to rewrite the request URI before Ferro routes it —
/// for example, custom domain → canonical path translation.
#[async_trait]
pub trait PreRouteMiddleware: Send + Sync {
    async fn handle(&self, req: hyper::Request<hyper::body::Incoming>) -> PreRouteResult;
}

type BoxedPreRoute = Arc<dyn PreRouteMiddleware>;

static PRE_ROUTE_REGISTRY: OnceLock<RwLock<Vec<BoxedPreRoute>>> = OnceLock::new();

/// Register a pre-route middleware. Called by the `pre_route_middleware!` macro.
pub fn register_pre_route_middleware<M: PreRouteMiddleware + 'static>(middleware: M) {
    let registry = PRE_ROUTE_REGISTRY.get_or_init(|| RwLock::new(Vec::new()));
    if let Ok(mut vec) = registry.write() {
        vec.push(Arc::new(middleware));
    }
}

/// Get all registered pre-route middleware.
pub fn get_pre_route_middleware() -> Vec<BoxedPreRoute> {
    PRE_ROUTE_REGISTRY
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|vec| vec.clone())
        .unwrap_or_default()
}

/// Rewrite the URI path of a hyper request, preserving the query string.
pub fn rewrite_request_path(
    req: hyper::Request<hyper::body::Incoming>,
    new_path: &str,
) -> hyper::Request<hyper::body::Incoming> {
    let (mut parts, body) = req.into_parts();
    let query = parts
        .uri
        .query()
        .map(|q| format!("?{q}"))
        .unwrap_or_default();
    let new_uri_str = format!("{new_path}{query}");
    if let Ok(new_uri) = new_uri_str.parse::<hyper::Uri>() {
        parts.uri = new_uri;
    }
    hyper::Request::from_parts(parts, body)
}
