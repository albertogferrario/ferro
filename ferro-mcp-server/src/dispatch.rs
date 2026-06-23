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
    let table = service.resolved_table();

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

    // Soft-delete predicate — injected AFTER tenant predicate, BEFORE WHERE assembly.
    // Gated ONLY on the projection having explicitly declared a soft-delete column
    // (`.soft_delete_column(...)`), so tables without a deleted_at column — including any
    // projection that flips `.deletable(true)` without declaring the column — are unaffected.
    // IS NULL has no bound value.
    if service.soft_delete_column.is_some() {
        let col = service.resolved_soft_delete_column();
        where_clauses.push(format!("\"{col}\" IS NULL"));
        // No values.push() — IS NULL takes no bound parameter.
        // idx is NOT incremented: LIMIT/OFFSET placeholders keep correct indices on Postgres.
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

    // ---- Task 1 RED tests: split_op_key, __op filter loop, sort ----

    /// split_op_key splits on the LAST `__` (rfind).
    #[test]
    fn test_split_op_key_basic() {
        // This test verifies split_op_key exists and uses rfind:
        // expected behavior — field__op
        assert_eq!(split_op_key("total__gt"), Some(("total", "gt")));
        assert_eq!(split_op_key("status__in"), Some(("status", "in")));
        // no __ → None
        assert_eq!(split_op_key("total"), None);
        // field with embedded __ → split on LAST __
        assert_eq!(split_op_key("my__field__lte"), Some(("my__field", "lte")));
    }

    /// dispatch rejects an unknown op suffix with InvalidFilter.
    #[tokio::test]
    async fn test_unknown_op_suffix_returns_error() {
        let db = setup_orders_db().await;
        let service = order_service_no_tenant();
        let result = dispatch(
            &service,
            serde_json::json!({"total__badop": 100.0}),
            10,
            0,
            &db,
            None,
        )
        .await;
        assert!(result.is_err(), "unknown op suffix must be an error");
        match result.unwrap_err() {
            crate::Error::InvalidFilter(msg) => {
                assert!(msg.contains("unknown op suffix"), "msg: {msg}");
            }
            other => panic!("expected InvalidFilter, got: {other:?}"),
        }
    }

    /// dispatch rejects __in with an empty array.
    #[tokio::test]
    async fn test_empty_in_array_returns_error() {
        let db = setup_orders_db().await;
        let service = order_service_no_tenant();
        let result = dispatch(
            &service,
            serde_json::json!({"status__in": []}),
            10,
            0,
            &db,
            None,
        )
        .await;
        assert!(result.is_err(), "empty __in must be an error");
        match result.unwrap_err() {
            crate::Error::InvalidFilter(_) => {}
            other => panic!("expected InvalidFilter, got: {other:?}"),
        }
    }

    /// dispatch rejects sort on a non-filterable field.
    #[tokio::test]
    async fn test_unknown_sort_field_returns_error() {
        let db = setup_orders_db().await;
        let service = order_service_no_tenant();
        let result = dispatch(
            &service,
            serde_json::json!({"sort": "customer_name"}), // EntityName — not is_filter_field
            10,
            0,
            &db,
            None,
        )
        .await;
        assert!(result.is_err(), "non-sortable field must be an error");
        match result.unwrap_err() {
            crate::Error::InvalidFilter(msg) => {
                assert!(
                    msg.contains("non-sortable") || msg.contains("non-filterable"),
                    "msg: {msg}"
                );
            }
            other => panic!("expected InvalidFilter, got: {other:?}"),
        }
    }

    // ---- Task 1 RED tests end ----

    /// SC#3 / T-239-02: a soft-deleted row is excluded from dispatch results by construction.
    ///
    /// Uses a self-contained in-memory DB — does NOT modify setup_orders_db() — so the
    /// existing tenant tests remain coupling-free.
    #[tokio::test]
    async fn soft_delete_excluded() {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("sqlite connect");

        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE IF NOT EXISTS orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                customer_name TEXT NOT NULL,
                total REAL NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                tenant_id INTEGER NOT NULL,
                deleted_at TEXT NULL
            )"
            .to_string(),
        ))
        .await
        .expect("create table");

        // Seed: 1 active row (deleted_at NULL), 1 soft-deleted row (deleted_at non-NULL).
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "INSERT INTO orders (customer_name, total, status, tenant_id, deleted_at) VALUES
                ('Alice', 100.0, 'pending', 1, NULL),
                ('Bob',   200.0, 'shipped', 1, '2026-06-23 12:00:00')"
                .to_string(),
        ))
        .await
        .expect("seed rows");

        let service = ServiceDef::new("order")
            .mcp_exposed(true)
            .soft_delete_column("deleted_at")
            .tenant_column("tenant_id")
            .mcp_ability("view-orders")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("customer_name", DataType::String, FieldMeaning::EntityName)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("created_at", DataType::String, FieldMeaning::CreatedAt)
            .field("tenant_id", DataType::Integer, FieldMeaning::ForeignKey);

        let result = dispatch(&service, serde_json::json!({}), 10, 0, &db, Some(1))
            .await
            .expect("dispatch ok");

        assert_eq!(
            result.rows.len(),
            1,
            "soft-deleted row must be excluded; only 1 active row"
        );
        assert_eq!(
            result.rows[0]["customer_name"],
            serde_json::Value::String("Alice".to_string())
        );
        assert_eq!(
            result.total, 1,
            "total count must also exclude the soft-deleted row"
        );
    }
}
