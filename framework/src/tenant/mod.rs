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
#[cfg(feature = "stripe")]
pub mod requires_plan;
pub mod resolver;
pub mod scope;
#[cfg(feature = "stripe")]
pub mod subscription;
pub mod worker;

pub use context::current_tenant;
pub use lookup::{DbTenantLookup, TenantLookup};
pub use middleware::TenantMiddleware;
#[cfg(feature = "stripe")]
pub use requires_plan::RequiresPlan;
pub use resolver::{
    HeaderResolver, JwtClaimResolver, PathResolver, SubdomainResolver, TenantResolver,
};
pub use scope::TenantScope;
pub use worker::FrameworkTenantScopeProvider;

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
    /// Optional billing plan identifier (legacy — use subscription.plan when stripe feature is enabled).
    pub plan: Option<String>,
    /// Full subscription state (available when stripe feature is enabled).
    #[cfg(feature = "stripe")]
    pub subscription: Option<subscription::SubscriptionInfo>,
}

#[cfg(feature = "stripe")]
impl TenantContext {
    /// Returns true when the tenant's subscription is in a trial period.
    ///
    /// Returns false if there is no active subscription.
    pub fn on_trial(&self) -> bool {
        self.subscription.as_ref().is_some_and(|s| s.on_trial())
    }

    /// Returns true when the tenant has an active or trialing subscription.
    ///
    /// Returns false if there is no subscription or status is not active/trialing.
    pub fn subscribed(&self) -> bool {
        self.subscription.as_ref().is_some_and(|s| s.subscribed())
    }

    /// Returns true when the subscription is scheduled to cancel but the billing period is still active.
    ///
    /// Returns false if there is no subscription.
    pub fn on_grace_period(&self) -> bool {
        self.subscription
            .as_ref()
            .is_some_and(|s| s.on_grace_period())
    }

    /// Returns the current plan identifier from the subscription, falling back to the legacy plan field.
    ///
    /// Returns `None` if neither the subscription nor the legacy plan is set.
    pub fn current_plan(&self) -> Option<&str> {
        self.subscription
            .as_ref()
            .map(|s| s.plan.as_str())
            .or(self.plan.as_deref())
    }
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
pub enum TenantFailureMode {
    /// Return 404 Not Found when the tenant cannot be resolved.
    NotFound,
    /// Return 403 Forbidden when the tenant cannot be resolved.
    Forbidden,
    /// Pass through — allow the request even without a resolved tenant.
    Allow,
    /// Return a custom response when the tenant cannot be resolved.
    Custom(Box<dyn Fn() -> crate::http::Response + Send + Sync>),
}

impl std::fmt::Debug for TenantFailureMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "NotFound"),
            Self::Forbidden => write!(f, "Forbidden"),
            Self::Allow => write!(f, "Allow"),
            Self::Custom(_) => write!(f, "Custom(...)"),
        }
    }
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
            #[cfg(feature = "stripe")]
            subscription: None,
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

        assert!(
            result.is_ok(),
            "Expected Ok(TenantContext), got: {result:?}"
        );
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

    #[cfg(feature = "stripe")]
    mod stripe_tests {
        use super::*;
        use crate::tenant::subscription::{SubscriptionInfo, SubscriptionStatus};

        fn make_subscription(plan: &str, status: SubscriptionStatus) -> SubscriptionInfo {
            SubscriptionInfo {
                stripe_subscription_id: "sub_test".to_string(),
                plan: plan.to_string(),
                status,
                trial_ends_at: None,
                cancel_at_period_end: false,
                current_period_end: chrono::Utc::now(),
                stripe_connect_account_id: None,
            }
        }

        fn make_tenant_with_subscription(plan: &str, status: SubscriptionStatus) -> TenantContext {
            TenantContext {
                id: 1,
                slug: "acme".to_string(),
                name: "ACME Corp".to_string(),
                plan: Some(plan.to_string()),
                subscription: Some(make_subscription(plan, status)),
            }
        }

        fn make_tenant_no_subscription() -> TenantContext {
            TenantContext {
                id: 1,
                slug: "acme".to_string(),
                name: "ACME Corp".to_string(),
                plan: None,
                subscription: None,
            }
        }

        #[test]
        fn tenant_with_none_subscription_serializes_with_null_subscription() {
            let tenant = make_tenant_no_subscription();
            let json = serde_json::to_value(&tenant).unwrap();
            assert!(json["subscription"].is_null());
        }

        #[test]
        fn tenant_with_some_subscription_serializes_with_full_subscription_object() {
            let tenant = make_tenant_with_subscription("pro", SubscriptionStatus::Active);
            let json = serde_json::to_value(&tenant).unwrap();
            assert!(!json["subscription"].is_null());
            assert_eq!(json["subscription"]["plan"], "pro");
            assert_eq!(json["subscription"]["status"], "active");
        }

        #[test]
        fn on_trial_returns_true_when_subscription_is_trialing() {
            let tenant = make_tenant_with_subscription("pro", SubscriptionStatus::Trialing);
            assert!(tenant.on_trial());
        }

        #[test]
        fn on_trial_returns_false_when_subscription_is_active() {
            let tenant = make_tenant_with_subscription("pro", SubscriptionStatus::Active);
            assert!(!tenant.on_trial());
        }

        #[test]
        fn on_trial_returns_false_when_no_subscription() {
            let tenant = make_tenant_no_subscription();
            assert!(!tenant.on_trial());
        }

        #[test]
        fn subscribed_returns_true_when_active() {
            let tenant = make_tenant_with_subscription("pro", SubscriptionStatus::Active);
            assert!(tenant.subscribed());
        }

        #[test]
        fn subscribed_returns_false_when_canceled() {
            let tenant = make_tenant_with_subscription("pro", SubscriptionStatus::Canceled);
            assert!(!tenant.subscribed());
        }

        #[test]
        fn subscribed_returns_false_when_no_subscription() {
            let tenant = make_tenant_no_subscription();
            assert!(!tenant.subscribed());
        }

        #[test]
        fn on_grace_period_returns_false_when_no_subscription() {
            let tenant = make_tenant_no_subscription();
            assert!(!tenant.on_grace_period());
        }

        #[test]
        fn on_grace_period_returns_true_when_cancel_at_period_end_and_active() {
            let mut tenant = make_tenant_with_subscription("pro", SubscriptionStatus::Active);
            if let Some(ref mut sub) = tenant.subscription {
                sub.cancel_at_period_end = true;
            }
            assert!(tenant.on_grace_period());
        }

        #[test]
        fn current_plan_returns_subscription_plan_when_present() {
            let tenant = make_tenant_with_subscription("enterprise", SubscriptionStatus::Active);
            assert_eq!(tenant.current_plan(), Some("enterprise"));
        }

        #[test]
        fn current_plan_falls_back_to_legacy_plan_when_no_subscription() {
            let tenant = TenantContext {
                id: 1,
                slug: "acme".to_string(),
                name: "ACME Corp".to_string(),
                plan: Some("pro".to_string()),
                subscription: None,
            };
            assert_eq!(tenant.current_plan(), Some("pro"));
        }

        #[test]
        fn current_plan_returns_none_when_neither_is_set() {
            let tenant = make_tenant_no_subscription();
            assert_eq!(tenant.current_plan(), None);
        }
    }
}
