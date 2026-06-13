//! The tenant-scoped lookup contract the CRUD route macros call.

use async_trait::async_trait;

use crate::error::FrameworkError;

/// A model that can be looked up scoped to a tenant.
///
/// `#[resource_get]` / `#[resource_post]` emit a call to
/// `<Self as TenantScoped>::find_for_tenant(id, tenant.id)`. Because every
/// generated lookup passes the resolved `tenant_id`, a generated handler
/// cannot read a row owned by another tenant — implementations MUST include
/// `tenant_id` in the query predicate.
///
/// Implement with `#[async_trait]` (re-exported as `ferro::async_trait`):
///
/// ```ignore
/// #[ferro::async_trait]
/// impl TenantScoped for Customer {
///     type Id = i64;
///     async fn find_for_tenant(id: i64, tenant_id: i64)
///         -> Result<Option<Self>, ferro::FrameworkError> {
///         // SELECT * FROM customers WHERE id = ? AND tenant_id = ?
///     }
/// }
/// ```
#[async_trait]
pub trait TenantScoped: Sized + Send + Sync {
    /// The type of the resource's primary key (parsed from the path param).
    type Id: std::str::FromStr + Send;

    /// Look up one record owned by `tenant_id`.
    ///
    /// Returns `Ok(None)` when not found (triggers the macro's 404 /
    /// redirect-on-miss arm). Returns `Err` on infrastructure failures.
    ///
    /// Implementations MUST include `AND tenant_id = ?` (or equivalent) in
    /// the query predicate — this is the load-bearing security property that
    /// prevents cross-tenant reads through the generated handler path.
    async fn find_for_tenant(id: Self::Id, tenant_id: i64) -> Result<Option<Self>, FrameworkError>;
}
