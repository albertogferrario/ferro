//! Task-local tenant context for Ferro framework.
//!
//! Provides task-local storage for the current tenant, allowing handlers
//! to call [`current_tenant()`] without passing tenant data explicitly.
//!
//! The `TenantMiddleware` (Plan 02) sets the tenant per-request. Handlers
//! read via [`current_tenant()`].

use crate::tenant::TenantContext;
use std::sync::Arc;
use tokio::sync::RwLock;

tokio::task_local! {
    pub(crate) static TENANT_CONTEXT: Arc<RwLock<Option<TenantContext>>>;
}

/// Get the current tenant context.
///
/// Returns a clone of the current tenant if the request is within a
/// `TenantMiddleware` scope. Returns `None` if called outside middleware scope
/// or if no tenant was resolved for the request.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::tenant::current_tenant;
///
/// if let Some(tenant) = current_tenant() {
///     println!("tenant: {}", tenant.slug);
/// }
/// ```
pub fn current_tenant() -> Option<TenantContext> {
    TENANT_CONTEXT
        .try_with(|ctx| ctx.try_read().ok().and_then(|guard| guard.clone()))
        .ok()
        .flatten()
}

/// Create a tenant context for use with `TENANT_CONTEXT.scope()`.
///
/// Returns `Arc<RwLock<Option<TenantContext>>>` initialized to `None`.
/// The middleware controls the scope lifetime.
// Used by TenantMiddleware (Plan 02).
#[allow(dead_code)]
pub(crate) fn tenant_scope() -> Arc<RwLock<Option<TenantContext>>> {
    Arc::new(RwLock::new(None))
}

/// Run an async block within a tenant context scope.
///
/// Used by `TenantMiddleware` to make [`current_tenant()`] available
/// during request processing.
// Used by TenantMiddleware (Plan 02).
#[allow(dead_code)]
pub(crate) async fn with_tenant_scope<F, R>(ctx: Arc<RwLock<Option<TenantContext>>>, f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    TENANT_CONTEXT.scope(ctx, f).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant::{TenantContext, TenantFailureMode};

    fn make_tenant() -> TenantContext {
        TenantContext {
            id: 1,
            slug: "acme".to_string(),
            name: "ACME Corp".to_string(),
            plan: Some("pro".to_string()),
            #[cfg(feature = "stripe")]
            subscription: None,
        }
    }

    #[test]
    fn tenant_context_constructs_and_clones() {
        let tc = make_tenant();
        let cloned = tc.clone();
        assert_eq!(tc.id, cloned.id);
        assert_eq!(tc.slug, cloned.slug);
        assert_eq!(tc.name, cloned.name);
        assert_eq!(tc.plan, cloned.plan);
    }

    #[test]
    fn tenant_context_serializes_to_json() {
        let tc = TenantContext {
            id: 42,
            slug: "beta-corp".to_string(),
            name: "Beta Corp".to_string(),
            plan: None,
            #[cfg(feature = "stripe")]
            subscription: None,
        };
        let json = serde_json::to_value(&tc).unwrap();
        assert_eq!(json["id"], 42);
        assert_eq!(json["slug"], "beta-corp");
        assert_eq!(json["name"], "Beta Corp");
        assert!(json["plan"].is_null());
    }

    #[test]
    fn current_tenant_returns_none_outside_scope() {
        let result = current_tenant();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn current_tenant_returns_some_within_scope() {
        let ctx = tenant_scope();
        {
            let mut guard = ctx.write().await;
            *guard = Some(make_tenant());
        }
        let result = with_tenant_scope(ctx, async { current_tenant() }).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().slug, "acme");
    }

    #[test]
    fn tenant_scope_creates_arc_rwlock_initialized_to_none() {
        let scope = tenant_scope();
        let guard = scope.try_read().unwrap();
        assert!(guard.is_none());
    }

    #[tokio::test]
    async fn with_tenant_scope_returns_none_outside_and_some_inside() {
        let ctx = tenant_scope();
        {
            let mut guard = ctx.write().await;
            *guard = Some(make_tenant());
        }

        // Inside scope: Some
        let inside = with_tenant_scope(ctx, async { current_tenant() }).await;
        assert!(inside.is_some());

        // Outside scope: None
        let outside = current_tenant();
        assert!(outside.is_none());
    }

    #[test]
    fn tenant_failure_mode_variants_exist() {
        let _not_found = TenantFailureMode::NotFound;
        let _forbidden = TenantFailureMode::Forbidden;
        let _allow = TenantFailureMode::Allow;
    }
}
