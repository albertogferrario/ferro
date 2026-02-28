//! Metrics collection middleware
//!
//! Records request timing and error rates for performance monitoring.
//! Should be registered as the first global middleware to capture full request duration.

use crate::http::Request;
use crate::http::Response;
use crate::metrics;
use crate::middleware::{Middleware, Next};
use async_trait::async_trait;
use std::time::Instant;

/// Middleware that collects request metrics
///
/// Records:
/// - Request count per route
/// - Response time (min, max, avg)
/// - Error count (4xx and 5xx responses)
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::middleware::MetricsMiddleware;
///
/// Server::from_config(router)
///     .middleware(MetricsMiddleware)  // Add as first middleware
///     .run()
///     .await;
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct MetricsMiddleware;

impl MetricsMiddleware {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Middleware for MetricsMiddleware {
    async fn handle(&self, request: Request, next: Next) -> Response {
        // Skip if metrics collection is disabled
        if !metrics::is_enabled() {
            return next(request).await;
        }

        // Skip internal debug endpoints
        let path = request.path();
        if path.starts_with("/_ferro/") {
            return next(request).await;
        }

        let start = Instant::now();
        let method = request.method().to_string();

        // Get route pattern (with placeholders like {id}) instead of actual path
        // This groups metrics by route pattern, not individual URLs
        let route_pattern = request
            .route_pattern()
            .unwrap_or_else(|| "UNMATCHED".to_string());

        // Execute the rest of the middleware chain and handler
        let response = next(request).await;

        let duration = start.elapsed();

        // Determine if this is an error response
        let is_error = match &response {
            Ok(resp) => resp.status_code() >= 400,
            Err(resp) => resp.status_code() >= 400,
        };

        // Record the metrics
        metrics::record_request(&route_pattern, &method, duration, is_error);

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_middleware_new() {
        let middleware = MetricsMiddleware::new();
        assert!(format!("{middleware:?}").contains("MetricsMiddleware"));
    }

    #[test]
    fn test_metrics_middleware_default() {
        let middleware = MetricsMiddleware;
        assert!(format!("{middleware:?}").contains("MetricsMiddleware"));
    }

    #[test]
    fn test_metrics_middleware_clone() {
        let middleware = MetricsMiddleware::new();
        let cloned = middleware;
        // Both should exist and be the same type
        assert!(format!("{cloned:?}").contains("MetricsMiddleware"));
    }

    #[test]
    fn test_metrics_middleware_copy() {
        let middleware = MetricsMiddleware::new();
        let copied: MetricsMiddleware = middleware; // Copy semantics
        let _original = middleware; // Original still usable
        assert!(format!("{copied:?}").contains("MetricsMiddleware"));
    }

    // Note: Full middleware behavior (request handling, timing, error detection)
    // requires integration testing with actual Request/Response types.
    // The core metrics recording logic is tested in the metrics module.
}
