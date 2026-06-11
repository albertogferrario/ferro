//! Global tenant lookup instance shared between bootstrap and route configuration.
//!
//! `TENANT_LOOKUP` is initialized once in `bootstrap::register()` and read by
//! `routes::register()` when constructing `JwtClaimResolver` + `TenantMiddleware`.

use crate::models::tenants::Tenant;
use ferro::{DbTenantLookup, TenantContext, TenantLookup};
use std::sync::{Arc, OnceLock};

static TENANT_LOOKUP: OnceLock<Arc<dyn TenantLookup>> = OnceLock::new();

/// Initialize the global tenant lookup. Called once from `bootstrap::register()`.
///
/// Idempotent: subsequent calls are ignored (OnceLock semantics).
pub fn init(lookup: Arc<dyn TenantLookup>) {
    let _ = TENANT_LOOKUP.set(lookup);
}

/// Retrieve the global tenant lookup.
///
/// # Panics
/// Panics if called before `init()`. The framework boot sequence calls
/// `bootstrap::register()` before `routes::register()`, so this is always
/// safe in the normal server-start path.
pub fn get() -> Arc<dyn TenantLookup> {
    TENANT_LOOKUP
        .get()
        .expect("tenant_lookup not initialized — call bootstrap::register() first")
        .clone()
}

/// Build the application `DbTenantLookup` backed by the Tenant model.
pub fn build() -> Arc<dyn TenantLookup> {
    Arc::new(DbTenantLookup::new(
        |slug| {
            Box::pin(async move {
                Tenant::find_by_slug(&slug)
                    .await
                    .ok()
                    .flatten()
                    .map(|t| TenantContext::new(t.id, t.slug, t.name, None))
            })
        },
        |id| {
            Box::pin(async move {
                Tenant::find_by_id(id)
                    .await
                    .ok()
                    .flatten()
                    .map(|t| TenantContext::new(t.id, t.slug, t.name, None))
            })
        },
    ))
}
