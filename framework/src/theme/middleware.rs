//! ThemeMiddleware for Ferro framework.
//!
//! Resolves the active theme from a request using a configurable chain of
//! [`ThemeResolver`] strategies, stores the result in task-local context,
//! and continues the request. Always succeeds — falls back to the configured
//! default theme when no resolver matches.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::theme::{ThemeMiddleware, TenantThemeResolver, DefaultResolver, Theme};
//!
//! let middleware = ThemeMiddleware::new()
//!     .resolver(TenantThemeResolver::new("./themes"))
//!     .default_theme(Theme::default_theme());
//! ```

use crate::http::Response;
use crate::middleware::{Middleware, Next};
use crate::Request;
use async_trait::async_trait;
use ferro_theme::Theme;
use std::sync::Arc;

use super::context::{theme_scope, with_theme_scope};
use super::resolver::ThemeResolver;

/// Middleware that resolves the active theme and stores it in task-local context.
///
/// Resolvers are tried in order; the first `Some` result wins. If no resolver
/// matches, the configured default theme is used. A theme is always available
/// downstream — there is no failure mode.
pub struct ThemeMiddleware {
    resolvers: Vec<Box<dyn ThemeResolver>>,
    default: Arc<Theme>,
}

impl ThemeMiddleware {
    /// Create a new `ThemeMiddleware` with no resolvers and the built-in default theme.
    pub fn new() -> Self {
        Self {
            resolvers: Vec::new(),
            default: Arc::new(Theme::default_theme()),
        }
    }

    /// Add a resolver to the chain (consuming builder).
    ///
    /// Resolvers are tried in the order they were added. The first `Some` result wins.
    pub fn resolver(mut self, resolver: impl ThemeResolver + 'static) -> Self {
        self.resolvers.push(Box::new(resolver));
        self
    }

    /// Set a custom default theme (consuming builder).
    ///
    /// Used when no resolver matches. Defaults to `Theme::default_theme()`.
    pub fn default_theme(mut self, theme: Theme) -> Self {
        self.default = Arc::new(theme);
        self
    }
}

impl Default for ThemeMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for ThemeMiddleware {
    async fn handle(&self, request: Request, next: Next) -> Response {
        // Try resolvers in order; first Some wins.
        let mut resolved: Option<Arc<Theme>> = None;
        for resolver in &self.resolvers {
            if let Some(theme) = resolver.resolve(&request).await {
                resolved = Some(theme);
                break;
            }
        }

        // Fall back to configured default when no resolver matches.
        let theme = resolved.unwrap_or_else(|| Arc::clone(&self.default));

        let scope = theme_scope();
        {
            let mut guard = scope.write().await;
            *guard = Some(theme);
        }
        with_theme_scope(scope, next(request)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::HttpResponse;
    use crate::theme::context::current_theme;
    use async_trait::async_trait;
    use bytes::Bytes;
    use http_body_util::Empty;
    use hyper_util::rt::TokioIo;
    use std::sync::Mutex;
    use tokio::sync::oneshot;

    async fn make_request() -> Request {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = oneshot::channel();
        let tx_holder = Arc::new(Mutex::new(Some(tx)));

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let io = TokioIo::new(stream);
            let tx_holder = tx_holder.clone();
            let service =
                hyper::service::service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
                    let tx_holder = tx_holder.clone();
                    async move {
                        if let Some(tx) = tx_holder.lock().unwrap().take() {
                            let _ = tx.send(Request::new(req));
                        }
                        Ok::<_, hyper::Error>(hyper::Response::new(Empty::<Bytes>::new()))
                    }
                });
            hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service)
                .await
                .ok();
        });

        let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let io = TokioIo::new(stream);
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await.unwrap();
        tokio::spawn(async move {
            conn.await.ok();
        });

        let req = hyper::Request::builder()
            .uri("/test")
            .header("x-test", "1")
            .body(Empty::<Bytes>::new())
            .unwrap();

        let _ = sender.send_request(req).await;
        rx.await.unwrap()
    }

    /// Mock resolver that always returns a theme.
    struct AlwaysThemeResolver {
        css_marker: String,
    }

    #[async_trait]
    impl ThemeResolver for AlwaysThemeResolver {
        async fn resolve(&self, _req: &Request) -> Option<Arc<Theme>> {
            let mut theme = Theme::default_theme();
            theme.css = format!("/* {} */", self.css_marker);
            Some(Arc::new(theme))
        }
    }

    /// Mock resolver that always returns None.
    struct NeverThemeResolver;

    #[async_trait]
    impl ThemeResolver for NeverThemeResolver {
        async fn resolve(&self, _req: &Request) -> Option<Arc<Theme>> {
            None
        }
    }

    fn ok_next() -> Next {
        Arc::new(|_req| {
            Box::pin(async { Ok(HttpResponse::text("ok")) }) as crate::middleware::MiddlewareFuture
        })
    }

    /// Next that captures current_theme() CSS and returns as text body.
    fn theme_capture_next() -> Next {
        Arc::new(|_req| {
            Box::pin(async move {
                let body = match current_theme() {
                    Some(t) => t.css.clone(),
                    None => "no-theme".to_string(),
                };
                Ok(HttpResponse::text(body))
            }) as crate::middleware::MiddlewareFuture
        })
    }

    // Test: ThemeMiddleware::new() creates instance with empty resolvers and default theme
    #[test]
    fn new_creates_empty_instance_with_default_theme() {
        let mw = ThemeMiddleware::new();
        assert!(mw.resolvers.is_empty());
        // default theme contains --color-primary from the embedded CSS
        assert!(mw.default.css.contains("--color-primary"));
    }

    // Test: .resolver(r) adds resolver to chain (consuming builder)
    #[test]
    fn resolver_adds_to_chain() {
        let mw = ThemeMiddleware::new().resolver(NeverThemeResolver);
        assert_eq!(mw.resolvers.len(), 1);
    }

    // Test: Middleware resolves theme from first matching resolver and stores in task-local
    #[tokio::test]
    async fn resolves_theme_from_first_matching_resolver() {
        let mw = ThemeMiddleware::new().resolver(AlwaysThemeResolver {
            css_marker: "first-resolver".to_string(),
        });

        let req = make_request().await;
        let next = theme_capture_next();
        let resp = mw.handle(req, next).await.unwrap();
        assert!(resp.body().contains("first-resolver"));
    }

    // Test: Middleware tries resolvers in order, first Some wins (skip later resolvers)
    #[tokio::test]
    async fn tries_resolvers_in_order_first_some_wins() {
        let mw = ThemeMiddleware::new()
            .resolver(NeverThemeResolver)
            .resolver(AlwaysThemeResolver {
                css_marker: "second-resolver".to_string(),
            })
            .resolver(AlwaysThemeResolver {
                css_marker: "third-resolver".to_string(),
            });

        let req = make_request().await;
        let next = theme_capture_next();
        let resp = mw.handle(req, next).await.unwrap();
        assert!(
            resp.body().contains("second-resolver"),
            "second resolver should win"
        );
        assert!(
            !resp.body().contains("third-resolver"),
            "third resolver should not run"
        );
    }

    // Test: When no resolver matches, uses default theme
    #[tokio::test]
    async fn uses_default_theme_when_no_resolver_matches() {
        let mw = ThemeMiddleware::new().resolver(NeverThemeResolver);

        let req = make_request().await;
        let next = theme_capture_next();
        let resp = mw.handle(req, next).await.unwrap();
        // Default theme contains --color-primary
        assert!(
            resp.body().contains("--color-primary"),
            "should use default theme CSS when no resolver matches"
        );
    }

    // Test: current_theme() available in downstream handler (via theme_capture_next)
    #[tokio::test]
    async fn current_theme_available_in_downstream_handler() {
        let mw = ThemeMiddleware::new().resolver(AlwaysThemeResolver {
            css_marker: "downstream-check".to_string(),
        });

        let req = make_request().await;
        let next = theme_capture_next();
        let resp = mw.handle(req, next).await.unwrap();
        assert!(resp.body().contains("downstream-check"));
    }

    // Test: With no resolvers, middleware uses default theme
    #[tokio::test]
    async fn no_resolvers_uses_default() {
        let mw = ThemeMiddleware::new(); // no resolvers
        let req = make_request().await;
        let next = theme_capture_next();
        let resp = mw.handle(req, next).await.unwrap();
        assert!(
            resp.body().contains("--color-primary"),
            "no resolvers: should fall back to default theme"
        );
    }

    // Test: .default_theme() sets custom default
    #[tokio::test]
    async fn default_theme_sets_custom_default() {
        let mut custom = Theme::default_theme();
        custom.css = "/* custom-default */".to_string();
        let mw = ThemeMiddleware::new()
            .resolver(NeverThemeResolver)
            .default_theme(custom);

        let req = make_request().await;
        let next = theme_capture_next();
        let resp = mw.handle(req, next).await.unwrap();
        assert!(resp.body().contains("custom-default"));
    }

    // Test: Middleware always continues the request (no failure mode)
    #[tokio::test]
    async fn middleware_always_continues_request() {
        let mw = ThemeMiddleware::new().resolver(NeverThemeResolver);
        let req = make_request().await;
        let result = mw.handle(req, ok_next()).await;
        assert!(result.is_ok());
    }
}
