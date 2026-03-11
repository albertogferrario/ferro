//! RequiresPlan middleware for plan-based route access control.
//!
//! Blocks requests when the current tenant's subscription plan does not satisfy
//! the required plan tier. Uses [`ferro_stripe::plan_satisfies`] for tier comparison
//! so higher tiers always satisfy lower requirements (enterprise > pro > free).
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::RequiresPlan;
//!
//! // Only allow pro and above
//! let middleware = RequiresPlan::new("pro");
//! ```

use crate::http::{HttpResponse, Response};
use crate::middleware::{Middleware, Next};
use crate::tenant::context::current_tenant;
use crate::Request;
use async_trait::async_trait;
use serde_json::json;

/// Middleware that gates route access by subscription plan tier.
///
/// Reads the current tenant from task-local context and checks whether
/// the tenant's subscription satisfies the required plan using the plan
/// hierarchy (enterprise > pro > free). Returns a 403 JSON response
/// when the plan requirement is not met.
pub struct RequiresPlan {
    required_plan: &'static str,
}

impl RequiresPlan {
    /// Create a new `RequiresPlan` middleware requiring the given plan tier.
    ///
    /// `plan` must be one of: `"free"`, `"pro"`, `"enterprise"`, or a custom string.
    pub fn new(plan: &'static str) -> Self {
        Self {
            required_plan: plan,
        }
    }
}

#[async_trait]
impl Middleware for RequiresPlan {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let tenant = match current_tenant() {
            Some(t) => t,
            None => {
                return Err(HttpResponse::json(json!({
                    "error": "No tenant context available",
                    "required_plan": self.required_plan,
                }))
                .status(400));
            }
        };

        let subscription = match &tenant.subscription {
            Some(s) => s.clone(),
            None => {
                return Err(HttpResponse::json(json!({
                    "error": "No active subscription",
                    "required_plan": self.required_plan,
                }))
                .status(403));
            }
        };

        if !subscription.subscribed() {
            return Err(HttpResponse::json(json!({
                "error": "Subscription is not active",
                "required_plan": self.required_plan,
            }))
            .status(403));
        }

        if !ferro_stripe::plan_satisfies(&subscription.plan, self.required_plan) {
            return Err(HttpResponse::json(json!({
                "error": "Plan does not meet requirement",
                "required_plan": self.required_plan,
            }))
            .status(403));
        }

        next(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::HttpResponse;
    use crate::tenant::context::{tenant_scope, with_tenant_scope};
    use crate::tenant::TenantContext;
    use ferro_stripe::{SubscriptionInfo, SubscriptionStatus};
    use hyper_util::rt::TokioIo;
    use std::sync::Arc;
    use std::sync::Mutex;
    use tokio::sync::oneshot;

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
            .body(http_body_util::Empty::<bytes::Bytes>::new())
            .unwrap();

        let _ = sender.send_request(req).await;
        rx.await.unwrap()
    }

    fn ok_next() -> Next {
        Arc::new(|_req| {
            Box::pin(async { Ok(HttpResponse::text("ok")) }) as crate::middleware::MiddlewareFuture
        })
    }

    async fn run_middleware_with_tenant(
        tenant: TenantContext,
        required_plan: &'static str,
    ) -> Response {
        let scope = tenant_scope();
        {
            let mut guard = scope.write().await;
            *guard = Some(tenant);
        }

        let mw = RequiresPlan::new(required_plan);
        let req = make_request().await;
        let next = ok_next();

        with_tenant_scope(scope, async move { mw.handle(req, next).await }).await
    }

    async fn run_middleware_no_tenant(required_plan: &'static str) -> Response {
        let mw = RequiresPlan::new(required_plan);
        let req = make_request().await;
        let next = ok_next();
        mw.handle(req, next).await
    }

    #[tokio::test]
    async fn pro_tenant_passes_requires_pro() {
        let tenant = make_tenant_with_subscription("pro", SubscriptionStatus::Active);
        let result = run_middleware_with_tenant(tenant, "pro").await;
        assert!(result.is_ok(), "pro tenant should pass RequiresPlan(pro)");
    }

    #[tokio::test]
    async fn enterprise_tenant_passes_requires_pro() {
        let tenant = make_tenant_with_subscription("enterprise", SubscriptionStatus::Active);
        let result = run_middleware_with_tenant(tenant, "pro").await;
        assert!(
            result.is_ok(),
            "enterprise tenant should satisfy RequiresPlan(pro)"
        );
    }

    #[tokio::test]
    async fn free_tenant_blocked_by_requires_pro() {
        let tenant = make_tenant_with_subscription("free", SubscriptionStatus::Active);
        let result = run_middleware_with_tenant(tenant, "pro").await;
        let err = result.unwrap_err();
        assert_eq!(err.status_code(), 403);
        let json: serde_json::Value = serde_json::from_str(err.body()).unwrap();
        assert_eq!(json["required_plan"], "pro");
    }

    #[tokio::test]
    async fn tenant_without_subscription_blocked_by_requires_pro() {
        let tenant = make_tenant_no_subscription();
        let result = run_middleware_with_tenant(tenant, "pro").await;
        let err = result.unwrap_err();
        assert_eq!(err.status_code(), 403);
        let json: serde_json::Value = serde_json::from_str(err.body()).unwrap();
        assert_eq!(json["required_plan"], "pro");
    }

    #[tokio::test]
    async fn canceled_subscription_blocked_by_requires_pro() {
        let tenant = make_tenant_with_subscription("pro", SubscriptionStatus::Canceled);
        let result = run_middleware_with_tenant(tenant, "pro").await;
        let err = result.unwrap_err();
        assert_eq!(err.status_code(), 403);
        let json: serde_json::Value = serde_json::from_str(err.body()).unwrap();
        assert_eq!(json["required_plan"], "pro");
    }

    #[tokio::test]
    async fn free_tenant_passes_requires_free() {
        let tenant = make_tenant_with_subscription("free", SubscriptionStatus::Active);
        let result = run_middleware_with_tenant(tenant, "free").await;
        assert!(result.is_ok(), "free tenant should pass RequiresPlan(free)");
    }

    #[tokio::test]
    async fn no_tenant_context_returns_400() {
        let result = run_middleware_no_tenant("pro").await;
        let err = result.unwrap_err();
        assert_eq!(err.status_code(), 400);
    }
}
