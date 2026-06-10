use ferro_projections::ServiceDef;
use sea_orm::DatabaseConnection;
use serde::Serialize;

/// Result of a dispatch read over a projection's source table.
#[derive(Debug, Serialize)]
pub struct DispatchResult {
    pub rows: Vec<serde_json::Value>,
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
}

/// Executes the projection's read path. Implemented in plan 03.
pub async fn dispatch(
    _service: &ServiceDef,
    _filters: serde_json::Value,
    limit: u64,
    offset: u64,
    _db: &DatabaseConnection,
) -> crate::Result<DispatchResult> {
    Ok(DispatchResult { rows: Vec::new(), total: 0, limit, offset })
}
