//! Bridge between ferro-queue's TenantScopeProvider and the framework's tenant infrastructure.

use crate::tenant::context::{tenant_scope, with_tenant_scope};
use crate::tenant::TenantLookup;
use async_trait::async_trait;
use ferro_queue::{Error as QueueError, TenantScopeProvider};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Framework implementation of ferro-queue's TenantScopeProvider.
///
/// Uses TenantLookup::find_by_id() to restore full TenantContext from the
/// tenant_id stored in the job payload, then wraps job execution in a
/// task-local tenant scope so current_tenant() works inside job handlers.
pub struct FrameworkTenantScopeProvider {
    lookup: Arc<dyn TenantLookup>,
}

impl FrameworkTenantScopeProvider {
    /// Create a new provider backed by the given TenantLookup.
    pub fn new(lookup: Arc<dyn TenantLookup>) -> Self {
        Self { lookup }
    }
}

#[async_trait]
impl TenantScopeProvider for FrameworkTenantScopeProvider {
    async fn with_scope(
        &self,
        tenant_id: i64,
        f: Pin<Box<dyn Future<Output = Result<(), QueueError>> + Send>>,
    ) -> Result<(), QueueError> {
        let tenant = self
            .lookup
            .find_by_id(tenant_id)
            .await
            .ok_or_else(|| QueueError::tenant_not_found(tenant_id))?;

        let scope = tenant_scope();
        {
            let mut guard = scope.write().await;
            *guard = Some(tenant);
        }
        with_tenant_scope(scope, f).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant::context::current_tenant;
    use crate::tenant::TenantContext;

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

    struct MockLookup {
        tenant: Option<TenantContext>,
    }

    impl MockLookup {
        fn returning(tenant: TenantContext) -> Self {
            Self {
                tenant: Some(tenant),
            }
        }

        fn not_found() -> Self {
            Self { tenant: None }
        }
    }

    #[async_trait]
    impl TenantLookup for MockLookup {
        async fn find_by_slug(&self, _slug: &str) -> Option<TenantContext> {
            self.tenant.clone()
        }

        async fn find_by_id(&self, _id: i64) -> Option<TenantContext> {
            self.tenant.clone()
        }
    }

    /// FrameworkTenantScopeProvider::with_scope(1, job_future) calls find_by_id(1)
    /// and runs the job future inside a tenant scope.
    #[tokio::test]
    async fn with_scope_restores_tenant_context() {
        let tenant = make_tenant(1, "acme");
        let lookup = Arc::new(MockLookup::returning(tenant.clone()));
        let provider = FrameworkTenantScopeProvider::new(lookup);

        let result = provider.with_scope(1, Box::pin(async { Ok(()) })).await;

        assert!(result.is_ok(), "Expected Ok(()), got: {result:?}");
    }

    /// FrameworkTenantScopeProvider::with_scope(999, job_future) returns TenantNotFound
    /// when find_by_id returns None.
    #[tokio::test]
    async fn with_scope_returns_tenant_not_found_for_unknown_id() {
        let lookup = Arc::new(MockLookup::not_found());
        let provider = FrameworkTenantScopeProvider::new(lookup);

        let result = provider.with_scope(999, Box::pin(async { Ok(()) })).await;

        assert!(result.is_err(), "Expected Err for unknown tenant id");
        assert!(
            matches!(result, Err(QueueError::TenantNotFound { tenant_id: 999 })),
            "Expected TenantNotFound(999), got: {result:?}"
        );
    }

    /// Job future running inside with_scope can call current_tenant() and get the correct TenantContext.
    #[tokio::test]
    async fn current_tenant_accessible_inside_scope() {
        let tenant = make_tenant(42, "beta");
        let lookup = Arc::new(MockLookup::returning(tenant.clone()));
        let provider = FrameworkTenantScopeProvider::new(lookup);

        let observed_tenant = Arc::new(tokio::sync::Mutex::new(None::<TenantContext>));
        let observed_clone = observed_tenant.clone();

        let result = provider
            .with_scope(
                42,
                Box::pin(async move {
                    *observed_clone.lock().await = current_tenant();
                    Ok(())
                }),
            )
            .await;

        assert!(result.is_ok());
        let observed = observed_tenant.lock().await;
        assert!(
            observed.is_some(),
            "current_tenant() must return Some inside scope"
        );
        let t = observed.as_ref().unwrap();
        assert_eq!(t.id, 42);
        assert_eq!(t.slug, "beta");
    }
}
