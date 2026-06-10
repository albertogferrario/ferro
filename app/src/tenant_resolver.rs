//! Application-specific tenant resolvers.
//!
//! `SessionUserTenantResolver` resolves the tenant from the session-authenticated user.
//! Used on the `/authorize` route where no JWT exists yet — the user is authenticated
//! via session cookie from the browser login, so `Auth::id()` is available.
//!
//! Resolving: Auth::id() → User::find_by_id(id) → user.tenant_id → Tenant::find_by_id(tid)

use crate::models::tenants::Tenant;
use crate::models::users::User;
use async_trait::async_trait;
use ferro::{Auth, Request, TenantContext, TenantResolver};

/// Resolves the current tenant from the session-authenticated user's `tenant_id` field.
///
/// Used on `/authorize` (browser login path) where the JWT does not yet exist.
/// The resolver reads the session-bound user ID via `Auth::id()`, loads the user,
/// and maps `user.tenant_id` to a `TenantContext`.
///
/// Returns `None` if the user is unauthenticated, has no `tenant_id`, or the
/// tenant cannot be found in the database — in all cases `TenantMiddleware`'s
/// configured failure mode determines the response.
pub struct SessionUserTenantResolver;

impl SessionUserTenantResolver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SessionUserTenantResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TenantResolver for SessionUserTenantResolver {
    async fn resolve(&self, _req: &Request) -> Option<TenantContext> {
        // Read the session-authenticated user ID.
        let user_id = Auth::id()?;

        // Load the concrete user from the database.
        let user = User::find_by_id(user_id).await.ok().flatten()?;

        // Extract the tenant FK.
        let tenant_id = user.tenant_id?;

        // Look up the tenant and map to TenantContext.
        Tenant::find_by_id(tenant_id)
            .await
            .ok()
            .flatten()
            .map(|t| TenantContext {
                id: t.id,
                slug: t.slug,
                name: t.name,
                plan: None,
            })
    }
}
