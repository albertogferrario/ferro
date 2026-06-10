use crate::schema::is_filter_field;
use ferro_projections::{FieldMeaning, ServiceDef};
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use serde::Serialize;

/// Hard upper bound on rows returned by a dispatch, enforced regardless of the
/// requested `limit`. Mirrors the `maximum` advertised in the tool input schema
/// so an oversized `limit` passed directly to `dispatch` cannot bypass the cap
/// (and cannot wrap negative on the `as i64` cast).
const MAX_LIMIT: u64 = 100;

/// Hard upper bound on `offset`, guarding the `u64 -> i64` cast. Without it,
/// a caller passing a `u64` above `i64::MAX` (e.g. `u64::MAX`) would wrap to a
/// negative offset on the `as i64` cast (`u64::MAX as i64 == -1`), producing an
/// incorrect/invalid SQL OFFSET. Mirrors the `MAX_LIMIT` clamp rationale (WR-01).
const MAX_OFFSET: u64 = i64::MAX as u64;

/// Result of a dispatch read over a projection's source table.
#[derive(Debug, Serialize)]
pub struct DispatchResult {
    pub rows: Vec<serde_json::Value>,
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
}

/// Build a parameter placeholder for the given backend and 1-based index.
fn placeholder(backend: DatabaseBackend, index: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${index}"),
        _ => "?".to_string(),
    }
}

/// Convert a serde_json::Value to a sea_orm::Value appropriate for the value's type.
fn json_to_sea_value(val: &serde_json::Value) -> sea_orm::Value {
    match val {
        serde_json::Value::Null => sea_orm::Value::String(None),
        serde_json::Value::Bool(b) => sea_orm::Value::Bool(Some(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                sea_orm::Value::BigInt(Some(i))
            } else {
                sea_orm::Value::Double(n.as_f64())
            }
        }
        serde_json::Value::String(s) => sea_orm::Value::String(Some(Box::new(s.clone()))),
        other => sea_orm::Value::String(Some(Box::new(other.to_string()))),
    }
}

/// Extract query result rows into JSON objects.
fn rows_to_json(rows: Vec<sea_orm::QueryResult>) -> Vec<serde_json::Value> {
    if rows.is_empty() {
        return Vec::new();
    }

    let columns: Vec<String> = rows
        .first()
        .map(|r| r.column_names().iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();

    rows.iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for col in &columns {
                let val = row
                    .try_get_by::<String, _>(col.as_str())
                    .map(serde_json::Value::String)
                    .or_else(|_| {
                        row.try_get_by::<i64, _>(col.as_str())
                            .map(|v| serde_json::Value::Number(v.into()))
                    })
                    .or_else(|_| {
                        row.try_get_by::<f64, _>(col.as_str()).map(|v| {
                            serde_json::Number::from_f64(v)
                                .map(serde_json::Value::Number)
                                .unwrap_or(serde_json::Value::Null)
                        })
                    })
                    .or_else(|_| {
                        row.try_get_by::<bool, _>(col.as_str())
                            .map(serde_json::Value::Bool)
                    })
                    .unwrap_or(serde_json::Value::Null);
                obj.insert(col.clone(), val);
            }
            serde_json::Value::Object(obj)
        })
        .collect()
}

/// Executes the projection's parameterized read path with filter-key allowlisting,
/// offset-based pagination, and optional tenant predicate injection.
///
/// Security: filter KEYS are validated against `service.fields` (allowlist) before any SQL
/// assembly; unknown keys return `Err` and are never interpolated. Filter VALUES are bound
/// via `Statement::from_sql_and_values`, never string-interpolated. Table name is derived
/// from `service.name` (developer-controlled), not from the call payload.
///
/// Tenant scoping (SC-1): when `service.tenant_column` is `Some(col)`, the tenant predicate
/// `AND "{col}" = ?` is injected as a bound parameter using `tenant_id`. The tenant value is
/// NEVER sourced from the call payload — it is always the `tenant_id` function parameter
/// passed by the caller (the app handler reads `current_tenant().map(|t| t.id)`).
///
/// Fail-closed (D-06): if `tenant_column` is `Some` but `tenant_id` is `None`, dispatch
/// returns `Err(InvalidFilter)` immediately — it never falls back to an unscoped SELECT.
pub async fn dispatch(
    service: &ServiceDef,
    filters: serde_json::Value,
    limit: u64,
    offset: u64,
    db: &sea_orm::DatabaseConnection,
    tenant_id: Option<i64>,
) -> crate::Result<DispatchResult> {
    let backend = db.get_database_backend();
    // Clamp the requested limit to MAX_LIMIT regardless of caller. The schema
    // advertises `maximum: 100`, but a caller invoking `dispatch` directly could
    // pass an arbitrary `u64`; without this clamp `u64::MAX as i64` wraps negative.
    let limit = limit.min(MAX_LIMIT);
    let offset = offset.min(MAX_OFFSET);
    // TODO: ServiceDef.table field for irregular plurals / custom table names
    let table = format!("{}s", service.name.to_lowercase());

    let mut where_clauses: Vec<String> = Vec::new();
    let mut values: Vec<sea_orm::Value> = Vec::new();
    let mut idx = 1usize;

    if let Some(obj) = filters.as_object() {
        for (key, val) in obj {
            // ALLOWLIST: the filter key must name a field that is FILTER-ELIGIBLE
            // (the exact predicate that gates the input schema), not merely a known
            // field. This prevents an agent from filtering on a Sensitive,
            // write-only, list, or Json/Binary field that the schema deliberately
            // excludes — which would otherwise leak the column via `SELECT *` or
            // enable an oracle attack. Unknown keys are never interpolated.
            match service.fields.iter().find(|f| &f.name == key) {
                Some(field) if is_filter_field(field) => {}
                _ => {
                    return Err(crate::Error::InvalidFilter(format!(
                        "unknown or non-filterable filter field: {key}"
                    )));
                }
            }
            where_clauses.push(format!("\"{}\" = {}", key, placeholder(backend, idx)));
            values.push(json_to_sea_value(val));
            idx += 1;
        }
    }

    // Tenant predicate — injected AFTER user filters, BEFORE count/data queries.
    // Never sourced from the call payload; always from current_tenant() passed by caller.
    if let Some(ref col) = service.tenant_column {
        match tenant_id {
            Some(tid) => {
                where_clauses.push(format!("\"{}\" = {}", col, placeholder(backend, idx)));
                values.push(sea_orm::Value::BigInt(Some(tid)));
                idx += 1;
            }
            None => {
                // Fail-closed (D-06): tenant-scoped projection + no tenant context → deny.
                return Err(crate::Error::InvalidFilter(
                    "tenant context required but not present".to_string(),
                ));
            }
        }
    }

    let where_str = if where_clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_clauses.join(" AND "))
    };

    // COUNT query — reuses the same WHERE clause
    let count_sql = format!("SELECT COUNT(*) as cnt FROM \"{table}\"{where_str}");
    let count_stmt = Statement::from_sql_and_values(backend, &count_sql, values.clone());
    let count_row = db
        .query_one(count_stmt)
        .await
        .map_err(|e| crate::Error::Database(format!("Count query failed: {e}")))?;
    let total: u64 = count_row
        .and_then(|r| r.try_get_by::<i64, _>("cnt").ok())
        .unwrap_or(0) as u64;

    // Deterministic ordering for stable offset pagination. Without ORDER BY,
    // offset-based pages can overlap or skip rows under concurrent writes. The
    // sort column is chosen from the projection's own fields (the Identifier
    // field, else the first field) — never from the call payload — so it cannot
    // be an injection vector.
    let order_col = service
        .fields
        .iter()
        .find(|f| matches!(f.meaning, FieldMeaning::Identifier))
        .or_else(|| service.fields.first())
        .map(|f| f.name.clone());
    let order_str = match &order_col {
        Some(col) => format!(" ORDER BY \"{col}\""),
        None => String::new(),
    };

    // DATA query with LIMIT/OFFSET bound as parameters
    let limit_str = format!(
        " LIMIT {} OFFSET {}",
        placeholder(backend, idx),
        placeholder(backend, idx + 1)
    );
    values.push(sea_orm::Value::BigInt(Some(limit as i64)));
    values.push(sea_orm::Value::BigInt(Some(offset as i64)));

    let data_sql = format!("SELECT * FROM \"{table}\"{where_str}{order_str}{limit_str}");
    let data_stmt = Statement::from_sql_and_values(backend, &data_sql, values);
    let rows = db
        .query_all(data_stmt)
        .await
        .map_err(|e| crate::Error::Database(format!("List query failed: {e}")))?;

    Ok(DispatchResult {
        rows: rows_to_json(rows),
        total,
        limit,
        offset,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::{DataType, FieldMeaning, ServiceDef};
    use sea_orm::{ConnectionTrait, Database, Statement};

    /// Creates an in-memory SQLite database seeded with an `orders` table
    /// containing rows for two tenants.
    async fn setup_orders_db() -> sea_orm::DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("sqlite connect");

        // Create table
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE IF NOT EXISTS orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                customer_name TEXT NOT NULL,
                total REAL NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                tenant_id INTEGER NOT NULL
            )"
            .to_string(),
        ))
        .await
        .expect("create table");

        // Seed rows: 2 rows for tenant 1, 2 rows for tenant 2
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "INSERT INTO orders (customer_name, total, status, tenant_id) VALUES
                ('Alice', 100.0, 'pending', 1),
                ('Bob',   200.0, 'shipped', 1),
                ('Carol', 150.0, 'pending', 2),
                ('Dave',  250.0, 'shipped', 2)"
                .to_string(),
        ))
        .await
        .expect("seed rows");

        db
    }

    fn order_service_with_tenant() -> ServiceDef {
        ServiceDef::new("order")
            .mcp_exposed(true)
            .tenant_column("tenant_id")
            .mcp_ability("view-orders")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("customer_name", DataType::String, FieldMeaning::EntityName)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("created_at", DataType::String, FieldMeaning::CreatedAt)
            .field("tenant_id", DataType::Integer, FieldMeaning::ForeignKey)
    }

    fn order_service_no_tenant() -> ServiceDef {
        ServiceDef::new("order")
            .mcp_exposed(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("customer_name", DataType::String, FieldMeaning::EntityName)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("created_at", DataType::String, FieldMeaning::CreatedAt)
            .field("tenant_id", DataType::Integer, FieldMeaning::ForeignKey)
    }

    /// SC-1: tenant A token returns only tenant A rows (not tenant B's).
    #[tokio::test]
    async fn tenant_scoping() {
        let db = setup_orders_db().await;
        let service = order_service_with_tenant();
        let result = dispatch(&service, serde_json::json!({}), 10, 0, &db, Some(1))
            .await
            .expect("dispatch ok");

        assert_eq!(result.rows.len(), 2, "tenant 1 has exactly 2 rows");
        for row in &result.rows {
            let tid = row["tenant_id"].as_i64().expect("tenant_id present");
            assert_eq!(tid, 1, "all rows belong to tenant 1");
        }
    }

    /// SC-1 cross-tenant isolation: tenant B sees only tenant B rows.
    #[tokio::test]
    async fn tenant_isolation() {
        let db = setup_orders_db().await;
        let service = order_service_with_tenant();
        let result = dispatch(&service, serde_json::json!({}), 10, 0, &db, Some(2))
            .await
            .expect("dispatch ok");

        assert_eq!(result.rows.len(), 2, "tenant 2 has exactly 2 rows");
        for row in &result.rows {
            let tid = row["tenant_id"].as_i64().expect("tenant_id present");
            assert_eq!(tid, 2, "all rows belong to tenant 2");
        }
    }

    /// D-06 fail-closed: tenant_column=Some but no tenant context → Err, never rows.
    #[tokio::test]
    async fn tenant_fail_closed() {
        let db = setup_orders_db().await;
        let service = order_service_with_tenant();
        let result = dispatch(&service, serde_json::json!({}), 10, 0, &db, None).await;

        assert!(
            result.is_err(),
            "must return Err when tenant_column=Some and tenant_id=None"
        );
        match result.unwrap_err() {
            crate::Error::InvalidFilter(msg) => {
                assert!(
                    msg.contains("tenant context required but not present"),
                    "error message: {msg}"
                );
            }
            other => panic!("expected InvalidFilter, got: {other:?}"),
        }
    }

    /// Explicit non-tenant projection: tenant_column=None → unscoped, all rows returned.
    #[tokio::test]
    async fn non_tenant_unscoped() {
        let db = setup_orders_db().await;
        let service = order_service_no_tenant();
        let result = dispatch(&service, serde_json::json!({}), 10, 0, &db, None)
            .await
            .expect("dispatch ok for non-tenant projection");

        assert_eq!(
            result.rows.len(),
            4,
            "non-tenant projection returns all 4 rows"
        );
    }
}
