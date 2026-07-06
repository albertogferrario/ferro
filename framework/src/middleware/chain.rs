//! Middleware chain execution engine

use super::{BoxedMiddleware, MiddlewareFuture, Next};
use crate::http::{Request, Response};
use crate::routing::BoxedHandler;
use std::sync::Arc;

/// Builds and executes the middleware chain
///
/// The chain is built from the outside-in:
/// 1. Global middleware (first to run)
/// 2. Route group middleware
/// 3. Route-level middleware
/// 4. The actual route handler (innermost)
pub struct MiddlewareChain {
    middleware: Vec<BoxedMiddleware>,
}

impl MiddlewareChain {
    /// Create a new empty middleware chain
    pub fn new() -> Self {
        Self {
            middleware: Vec::new(),
        }
    }

    /// Add middleware to the chain
    ///
    /// Middleware are executed in the order they are added.
    pub fn push(&mut self, middleware: BoxedMiddleware) {
        self.middleware.push(middleware);
    }

    /// Add multiple middleware to the chain
    pub fn extend(&mut self, middleware: impl IntoIterator<Item = BoxedMiddleware>) {
        self.middleware.extend(middleware);
    }

    /// Execute the middleware chain with the given request and final handler
    ///
    /// The chain is executed from outside-in:
    /// - First middleware added runs first
    /// - Each middleware can call `next(request)` to continue the chain
    /// - The final handler is called at the end of the chain
    pub async fn execute(self, request: Request, handler: Arc<BoxedHandler>) -> Response {
        if self.middleware.is_empty() {
            // No middleware - call handler directly
            return handler(request).await;
        }

        // Build the chain from inside-out
        // Start with the actual handler as the innermost "next"
        let handler_clone = handler.clone();
        let mut next: Next = Arc::new(move |req| handler_clone(req));

        // Wrap each middleware around the next, from last to first
        // This creates the correct execution order: first middleware runs first
        for middleware in self.middleware.into_iter().rev() {
            let current_next = next;
            let mw = middleware;
            next = Arc::new(move |req| {
                let n = current_next.clone();
                let m = mw.clone();
                Box::pin(async move { m(req, n).await }) as MiddlewareFuture
            });
        }

        // Execute the outermost middleware (which was the first added)
        next(request).await
    }
}

impl Default for MiddlewareChain {
    fn default() -> Self {
        Self::new()
    }
}
