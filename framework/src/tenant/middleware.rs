//! TenantMiddleware for Ferro framework.
//!
//! Resolves the current tenant from a request using a configurable chain of
//! [`TenantResolver`] strategies, stores the result in task-local context, and
//! either continues the request or returns a 404/403 error based on
//! [`TenantFailureMode`].
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::tenant::{TenantMiddleware, TenantFailureMode, SubdomainResolver};
//!
//! let middleware = TenantMiddleware::new()
//!     .resolver(SubdomainResolver::new(2, lookup.clone()))
//!     .on_failure(TenantFailureMode::NotFound);
//! ```

use crate::http::{HttpResponse, Response};
use crate::middleware::{Middleware, Next};
use crate::tenant::context::{tenant_scope, with_tenant_scope};
use crate::tenant::{TenantContext, TenantFailureMode};
use crate::Request;
use async_trait::async_trait;
use serde_json::json;

use super::resolver::TenantResolver;

/// Middleware that resolves the current tenant and stores it in task-local context.
///
/// Resolvers are tried in order; the first `Some` result wins. If no resolver
/// matches, the failure mode determines the response.
pub struct TenantMiddleware {
    resolvers: Vec<Box<dyn TenantResolver>>,
    on_failure: TenantFailureMode,
}

impl TenantMiddleware {
    /// Create a new `TenantMiddleware` with no resolvers and `NotFound` failure mode.
    pub fn new() -> Self {
        Self {
            resolvers: Vec::new(),
            on_failure: TenantFailureMode::NotFound,
        }
    }

    /// Add a resolver to the chain (consuming builder).
    ///
    /// Resolvers are tried in the order they were added. The first `Some` result wins.
    pub fn resolver(mut self, resolver: impl TenantResolver + 'static) -> Self {
        self.resolvers.push(Box::new(resolver));
        self
    }

    /// Set the failure mode when no resolver matches (consuming builder).
    pub fn on_failure(mut self, mode: TenantFailureMode) -> Self {
        self.on_failure = mode;
        self
    }
}

impl Default for TenantMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for TenantMiddleware {
    async fn handle(&self, request: Request, next: Next) -> Response {
        // Try resolvers in order, first Some wins.
        let mut resolved: Option<TenantContext> = None;
        for resolver in &self.resolvers {
            if let Some(ctx) = resolver.resolve(&request).await {
                resolved = Some(ctx);
                break;
            }
        }

        match resolved {
            Some(ctx) => {
                // Store tenant in task-local context for the downstream chain.
                let scope = tenant_scope();
                {
                    let mut guard = scope.write().await;
                    *guard = Some(ctx);
                }
                with_tenant_scope(scope, next(request)).await
            }
            None => match self.on_failure {
                TenantFailureMode::NotFound => {
                    Err(HttpResponse::json(json!({"error": "Tenant not found"})).status(404))
                }
                TenantFailureMode::Forbidden => {
                    Err(HttpResponse::json(json!({"error": "Access denied"})).status(403))
                }
                TenantFailureMode::Allow => {
                    // No tenant — continue with None in context.
                    let scope = tenant_scope();
                    with_tenant_scope(scope, next(request)).await
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::HttpResponse;
    use crate::tenant::context::current_tenant;
    use crate::tenant::{TenantContext, TenantFailureMode};
    use async_trait::async_trait;
    use hyper_util::rt::TokioIo;
    use std::sync::Arc;
    use std::sync::Mutex;
    use tokio::sync::oneshot;

    fn make_tenant(slug: &str) -> TenantContext {
        TenantContext {
            id: 1,
            slug: slug.to_string(),
            name: "Test Corp".to_string(),
            plan: None,
        }
    }

    /// Create a test Request via TCP loopback with optional headers.
    async fn make_request_with_header(header_name: &str, header_value: &str) -> Request {
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
                        Ok::<_, hyper::Error>(hyper::Response::new(http_body_util::Empty::<
                            bytes::Bytes,
                        >::new(
                        )))
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
            .header(header_name, header_value)
            .body(http_body_util::Empty::<bytes::Bytes>::new())
            .unwrap();

        let _ = sender.send_request(req).await;
        rx.await.unwrap()
    }

    async fn make_request() -> Request {
        make_request_with_header("x-test", "1").await
    }

    /// Mock resolver that always returns a fixed TenantContext.
    struct AlwaysResolver {
        tenant: TenantContext,
    }

    #[async_trait]
    impl TenantResolver for AlwaysResolver {
        async fn resolve(&self, _req: &Request) -> Option<TenantContext> {
            Some(self.tenant.clone())
        }
    }

    /// Mock resolver that always returns None.
    struct NeverResolver;

    #[async_trait]
    impl TenantResolver for NeverResolver {
        async fn resolve(&self, _req: &Request) -> Option<TenantContext> {
            None
        }
    }

    fn ok_next() -> Next {
        Arc::new(|_req| {
            Box::pin(async { Ok(HttpResponse::text("ok")) }) as crate::middleware::MiddlewareFuture
        })
    }

    /// Next that captures current_tenant() and returns it as JSON.
    fn tenant_capture_next() -> Next {
        Arc::new(|_req| {
            Box::pin(async move {
                let tenant = current_tenant();
                match tenant {
                    Some(t) => Ok(HttpResponse::json(serde_json::json!({"slug": t.slug}))),
                    None => Ok(HttpResponse::json(serde_json::json!({"slug": null}))),
                }
            }) as crate::middleware::MiddlewareFuture
        })
    }

    // Test 1: new() creates instance with empty resolver vec and NotFound default
    #[test]
    fn new_creates_empty_instance_with_not_found_default() {
        let mw = TenantMiddleware::new();
        assert!(mw.resolvers.is_empty());
        assert!(matches!(mw.on_failure, TenantFailureMode::NotFound));
    }

    // Test 2: .resolver(r) adds a resolver to the chain (consuming builder)
    #[test]
    fn resolver_adds_to_chain() {
        let mw = TenantMiddleware::new().resolver(NeverResolver);
        assert_eq!(mw.resolvers.len(), 1);
    }

    // Test 3: .on_failure(mode) sets the failure mode (consuming builder)
    #[test]
    fn on_failure_sets_mode() {
        let mw = TenantMiddleware::new().on_failure(TenantFailureMode::Allow);
        assert!(matches!(mw.on_failure, TenantFailureMode::Allow));
    }

    // Test 4: Middleware resolves tenant from first matching resolver and stores in task-local
    #[tokio::test]
    async fn resolves_tenant_and_stores_in_task_local() {
        let mw = TenantMiddleware::new()
            .resolver(AlwaysResolver {
                tenant: make_tenant("acme"),
            })
            .on_failure(TenantFailureMode::NotFound);

        let req = make_request().await;
        let next = tenant_capture_next();
        let resp = mw.handle(req, next).await.unwrap();

        let json: serde_json::Value = serde_json::from_str(resp.body()).unwrap();
        assert_eq!(json["slug"], "acme");
    }

    // Test 5: Middleware tries resolvers in order, first Some wins
    #[tokio::test]
    async fn tries_resolvers_in_order_first_some_wins() {
        let mw = TenantMiddleware::new()
            .resolver(NeverResolver)
            .resolver(AlwaysResolver {
                tenant: make_tenant("beta"),
            })
            .resolver(AlwaysResolver {
                tenant: make_tenant("gamma"),
            });

        let req = make_request().await;
        let next = tenant_capture_next();
        let response = mw.handle(req, next).await.unwrap();

        let json: serde_json::Value = serde_json::from_str(response.body()).unwrap();
        assert_eq!(json["slug"], "beta");
    }

    // Test 6: When no resolver matches and on_failure=NotFound, returns 404 JSON
    #[tokio::test]
    async fn no_match_not_found_returns_404() {
        let mw = TenantMiddleware::new()
            .resolver(NeverResolver)
            .on_failure(TenantFailureMode::NotFound);

        let req = make_request().await;
        let err = mw.handle(req, ok_next()).await.unwrap_err();

        assert_eq!(err.status_code(), 404);
        let json: serde_json::Value = serde_json::from_str(err.body()).unwrap();
        assert_eq!(json["error"], "Tenant not found");
    }

    // Test 7: When no resolver matches and on_failure=Forbidden, returns 403 JSON
    #[tokio::test]
    async fn no_match_forbidden_returns_403() {
        let mw = TenantMiddleware::new()
            .resolver(NeverResolver)
            .on_failure(TenantFailureMode::Forbidden);

        let req = make_request().await;
        let err = mw.handle(req, ok_next()).await.unwrap_err();

        assert_eq!(err.status_code(), 403);
        let json: serde_json::Value = serde_json::from_str(err.body()).unwrap();
        assert_eq!(json["error"], "Access denied");
    }

    // Test 8: When no resolver matches and on_failure=Allow, request continues with current_tenant()=None
    #[tokio::test]
    async fn no_match_allow_continues_with_none() {
        let mw = TenantMiddleware::new()
            .resolver(NeverResolver)
            .on_failure(TenantFailureMode::Allow);

        let req = make_request().await;
        let next = tenant_capture_next();
        let response = mw.handle(req, next).await.unwrap();

        let json: serde_json::Value = serde_json::from_str(response.body()).unwrap();
        assert!(json["slug"].is_null());
    }

    // Test 9: current_tenant() returns the resolved TenantContext during downstream handler execution
    #[tokio::test]
    async fn current_tenant_available_in_downstream_handler() {
        let mw = TenantMiddleware::new().resolver(AlwaysResolver {
            tenant: make_tenant("downstream-test"),
        });

        let req = make_request().await;
        let next = tenant_capture_next();
        let response = mw.handle(req, next).await.unwrap();

        let json: serde_json::Value = serde_json::from_str(response.body()).unwrap();
        assert_eq!(json["slug"], "downstream-test");
    }
}
