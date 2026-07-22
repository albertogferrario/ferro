//! Request logging middleware.

use ferro::{async_trait, Middleware, Next, Request, Response};

/// Logs each incoming request method + path.
pub struct LoggingMiddleware;

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let method = request.method().to_string();
        let path = request.path().to_string();
        tracing::info!(%method, %path, "--> request");
        next(request).await
    }
}
