//! Multi-tenant middleware support for Ferro framework.
//!
//! Provides task-local tenant context, resolver and lookup trait contracts,
//! and a default cached database lookup implementation.
//!
//! # Overview
//!
//! - [`TenantContext`] — holds id, slug, name, and optional plan fields
//! - [`current_tenant()`] — reads the current tenant from task-local storage
//! - [`TenantResolver`] — trait for pluggable tenant resolution strategies
//! - [`TenantLookup`] / [`DbTenantLookup`] — trait + cached implementation for DB queries
//! - [`TenantFailureMode`] — controls behavior when no tenant is resolved

pub mod context;
pub mod lookup;
pub mod middleware;
pub mod resolver;
pub mod scope;

pub use context::current_tenant;
pub use lookup::{DbTenantLookup, TenantLookup};
pub use middleware::TenantMiddleware;
pub use resolver::{
    HeaderResolver, JwtClaimResolver, PathResolver, SubdomainResolver, TenantResolver,
};
pub use scope::TenantScope;

use crate::error::FrameworkError;
use crate::http::{FromRequest, Request};
use async_trait::async_trait;

/// Core data for the resolved tenant.
///
/// Populated by [`TenantResolver`] and stored in task-local scope during a request.
/// The `plan` field is nullable — tenants may not have a billing plan assigned
/// until Stripe integration is complete (Phase 96).
#[derive(Debug, Clone, serde::Serialize)]
pub struct TenantContext {
    /// Unique numeric tenant ID (primary key).
    pub id: i64,
    /// URL-safe slug used for subdomain or path-based routing.
    pub slug: String,
    /// Human-readable tenant name.
    pub name: String,
    /// Optional billing plan identifier.
    pub plan: Option<String>,
}

/// Extracts the current tenant from task-local context.
///
/// Returns `Ok(TenantContext)` when called from a handler behind
/// `TenantMiddleware`. Returns a 400 error if no tenant context exists.
///
/// # Example
///
/// ```rust,ignore
/// #[handler]
/// pub async fn dashboard(tenant: TenantContext) -> Response {
///     Ok(json!({"tenant": tenant.name}))
/// }
/// ```
#[async_trait]
impl FromRequest for TenantContext {
    async fn from_request(_req: Request) -> Result<Self, FrameworkError> {
        current_tenant().ok_or_else(|| {
            FrameworkError::domain(
                "No tenant context available. Ensure this route is behind TenantMiddleware.",
                400,
            )
        })
    }
}

/// Controls framework behavior when no tenant is resolved for a request.
#[derive(Debug, Clone)]
pub enum TenantFailureMode {
    /// Return 404 Not Found when the tenant cannot be resolved.
    NotFound,
    /// Return 403 Forbidden when the tenant cannot be resolved.
    Forbidden,
    /// Pass through — allow the request even without a resolved tenant.
    Allow,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant::context::{tenant_scope, with_tenant_scope};
    use hyper_util::rt::TokioIo;
    use tokio::sync::oneshot;

    fn make_tenant(id: i64, slug: &str) -> TenantContext {
        TenantContext {
            id,
            slug: slug.to_string(),
            name: format!("Tenant {slug}"),
            plan: None,
        }
    }

    /// Create a minimal Request via TCP loopback.
    ///
    /// hyper::body::Incoming has no default constructor, so we use a real
    /// TCP connection (matching the pattern in middleware tests).
    async fn make_request() -> Request {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (tx, rx) = oneshot::channel::<Request>();
        let tx_holder = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));

        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let io = TokioIo::new(stream);
                let tx_holder = tx_holder.clone();
                hyper::server::conn::http1::Builder::new()
                    .serve_connection(
                        io,
                        hyper::service::service_fn(move |req| {
                            let tx_holder = tx_holder.clone();
                            async move {
                                if let Some(tx) = tx_holder.lock().unwrap().take() {
                                    let _ = tx.send(Request::new(req));
                                }
                                Ok::<_, hyper::Error>(hyper::Response::new(
                                    http_body_util::Empty::<bytes::Bytes>::new(),
                                ))
                            }
                        }),
                    )
                    .await
                    .ok();
            }
        });

        let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let io = TokioIo::new(stream);
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await.unwrap();
        tokio::spawn(async move { conn.await.ok() });

        let req = hyper::Request::builder()
            .uri("/test")
            .body(http_body_util::Empty::<bytes::Bytes>::new())
            .unwrap();
        let _ = sender.send_request(req).await;
        rx.await.unwrap()
    }

    /// Test 4: TenantContext FromRequest returns Ok(ctx) when current_tenant() is Some.
    #[tokio::test]
    async fn from_request_returns_ok_when_tenant_context_is_set() {
        let ctx = tenant_scope();
        {
            let mut guard = ctx.write().await;
            *guard = Some(make_tenant(99, "acme"));
        }

        let result = with_tenant_scope(ctx, async {
            let req = make_request().await;
            TenantContext::from_request(req).await
        })
        .await;

        assert!(result.is_ok(), "Expected Ok(TenantContext), got: {result:?}");
        let tenant = result.unwrap();
        assert_eq!(tenant.id, 99);
        assert_eq!(tenant.slug, "acme");
    }

    /// Test 5: TenantContext FromRequest returns Err(FrameworkError) with status 400 when no tenant context.
    #[tokio::test]
    async fn from_request_returns_400_error_when_no_tenant_context() {
        // Call from_request without any TenantMiddleware scope
        let req = make_request().await;
        let result = TenantContext::from_request(req).await;

        assert!(result.is_err(), "Expected Err when no tenant context");
        let err = result.unwrap_err();
        assert_eq!(
            err.status_code(),
            400,
            "Expected 400 status code, got: {}",
            err.status_code()
        );
    }
}
