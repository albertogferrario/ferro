//! Middleware system for Ferro framework
//!
//! This module provides Laravel 12.x-style middleware support with:
//! - Global middleware (runs on all routes)
//! - Route group middleware (shared for a group of routes)
//! - Per-route middleware (applied to individual routes)
//! - Rate limiting middleware for API protection
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::{async_trait, Middleware, Next, Request, Response, HttpResponse};
//!
//! pub struct AuthMiddleware;
//!
//! #[async_trait]
//! impl Middleware for AuthMiddleware {
//!     async fn handle(&self, request: Request, next: Next) -> Response {
//!         if request.header("Authorization").is_none() {
//!             return Err(HttpResponse::text("Unauthorized").status(401));
//!         }
//!         next(request).await
//!     }
//! }
//! ```
//!
//! # Rate Limiting
//!
//! ```rust,ignore
//! use ferro::middleware::{RateLimiter, Limit, Throttle};
//!
//! // Register named limiter
//! RateLimiter::define("api", |req| Limit::per_minute(60));
//!
//! // Apply to routes
//! get!("/api/users", handler).middleware(Throttle::named("api"))
//!
//! // Inline limit
//! get!("/health", handler).middleware(Throttle::per_minute(120))
//! ```

mod chain;
mod cors;
mod metrics;
mod pre_route;
mod rate_limit;
mod registry;
mod security_headers;

pub use cors::Cors;
pub use metrics::MetricsMiddleware;
pub use security_headers::SecurityHeaders;

pub use chain::MiddlewareChain;
pub use pre_route::{
    get_pre_route_middleware, register_pre_route_middleware, rewrite_request_path,
    PreRouteMiddleware, PreRouteResult,
};
pub use rate_limit::{Limit, LimiterResponse, RateLimiter, Throttle};
pub use registry::get_global_middleware_info;
pub use registry::register_global_middleware;
pub use registry::MiddlewareRegistry;

use crate::http::{Request, Response};
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Type alias for the boxed future returned by middleware
pub type MiddlewareFuture = Pin<Box<dyn Future<Output = Response> + Send>>;

/// Type alias for the next handler in the middleware chain
///
/// Call `next(request).await` to pass control to the next middleware or the route handler.
pub type Next = Arc<dyn Fn(Request) -> MiddlewareFuture + Send + Sync>;

/// Type alias for boxed middleware handlers (internal use)
pub type BoxedMiddleware = Arc<dyn Fn(Request, Next) -> MiddlewareFuture + Send + Sync>;

/// Trait for implementing middleware
///
/// Middleware can inspect/modify requests, short-circuit responses, or pass control
/// to the next middleware in the chain by calling `next(request).await`.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::{async_trait, Middleware, Next, Request, Response, HttpResponse};
///
/// pub struct LoggingMiddleware;
///
/// #[async_trait]
/// impl Middleware for LoggingMiddleware {
///     async fn handle(&self, request: Request, next: Next) -> Response {
///         println!("--> {} {}", request.method(), request.path());
///         let response = next(request).await;
///         println!("<-- complete");
///         response
///     }
/// }
/// ```
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Handle the request
    ///
    /// - Call `next(request).await` to pass control to the next middleware
    /// - Return `Err(HttpResponse)` to short-circuit and respond immediately
    /// - Modify the response after calling `next()` for post-processing
    async fn handle(&self, request: Request, next: Next) -> Response;
}

/// Convert a Middleware trait object into a BoxedMiddleware
pub fn into_boxed<M: Middleware + 'static>(middleware: M) -> BoxedMiddleware {
    let middleware = Arc::new(middleware);
    Arc::new(move |req, next| {
        let mw = middleware.clone();
        Box::pin(async move { mw.handle(req, next).await })
    })
}
