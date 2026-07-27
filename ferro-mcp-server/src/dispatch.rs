use ferro_projections::ServiceDef;

pub use ferro_rs::projection_read::{DispatchResult, ProjectionReadError};

/// Delegates to [`ferro_rs::projection_read::dispatch`], mapping
/// [`ProjectionReadError`] to [`crate::Error`] 1:1 at the framing boundary.
///
/// The full implementation — filter allowlisting, offset pagination (MAX_LIMIT=100),
/// soft-delete predicate, tenant predicate injection — lives in `ferro_rs::projection_read`.
/// This wrapper exists only to preserve the MCP crate's `crate::Result<DispatchResult>`
/// return type so every existing call site compiles unchanged.
pub async fn dispatch(
    service: &ServiceDef,
    filters: serde_json::Value,
    limit: u64,
    offset: u64,
    db: &sea_orm::DatabaseConnection,
    tenant_id: Option<i64>,
) -> crate::Result<DispatchResult> {
    ferro_rs::projection_read::dispatch(service, filters, limit, offset, db, tenant_id)
        .await
        .map_err(|e| match e {
            ProjectionReadError::InvalidFilter(m) => crate::Error::InvalidFilter(m),
            ProjectionReadError::Database(m) => crate::Error::Database(m),
        })
}
