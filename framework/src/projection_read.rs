use ferro_projections::{DataType, FieldDef, FieldMeaning, ServiceDef};
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

/// Error type for projection read operations.
///
/// Maps 1:1 to the variants in `ferro-mcp-server::Error` that `dispatch`
/// originally produced, so the thin wrapper in `ferro-mcp-server` can map
/// these back without information loss.
#[derive(Debug, thiserror::Error)]
pub enum ProjectionReadError {
    /// A filter key or value is invalid (unknown field, disallowed op, bad type,
    /// or missing tenant context on a tenant-scoped projection).
    #[error("invalid filter: {0}")]
    InvalidFilter(String),
    /// A database query failed.
    #[error("database error: {0}")]
    Database(String),
}

/// `Result` alias for projection read operations.
pub type ProjectionReadResult<T> = Result<T, ProjectionReadError>;

/// Result of a dispatch read over a projection's source table.
#[derive(Debug, Serialize)]
pub struct DispatchResult {
    /// The rows returned by the query, each as a JSON object.
    pub rows: Vec<serde_json::Value>,
    /// Total row count matching the filter (before pagination).
    pub total: u64,
    /// The effective limit used in this query (clamped to `MAX_LIMIT`).
    pub limit: u64,
    /// The effective offset used in this query.
    pub offset: u64,
}

/// Returns `true` if this field should appear as an equality filter in the
/// projection read path's input schema.
///
/// Gate order (load-bearing — do not reorder):
/// 1. Must be readable — excludes write-only (e.g., passwords) regardless of meaning.
/// 2. Must not be a list — equality filters on list columns are not useful.
/// 3. Must not carry `Sensitive` meaning — guards fields that ARE readable but still private.
/// 4. `DataType` must not be `Json` or `Binary` — equality filters are not useful there.
/// 5. Meaning must be in the conservative allowlist: Identifier, ForeignKey, Status,
///    Category, Boolean, Custom(_). All other meanings (EntityName, Email, Money, …) are
///    intentionally excluded.
pub fn is_filter_field(field: &FieldDef) -> bool {
    if !field.readable {
        return false;
    } // gate 1
    if field.is_list {
        return false;
    } // gate 2
    if matches!(field.meaning, FieldMeaning::Sensitive) {
        return false;
    } // gate 3
      // gate 4: equality filter on JSON/Binary columns is not useful
    if matches!(field.data_type, DataType::Json | DataType::Binary) {
        return false;
    }
    // gate 5: conservative meaning allowlist
    matches!(
        field.meaning,
        FieldMeaning::Identifier
            | FieldMeaning::ForeignKey
            | FieldMeaning::Status
            | FieldMeaning::Category
            | FieldMeaning::Boolean
            | FieldMeaning::Custom(_)
    )
}

/// Returns `true` if this field should receive `__gt/__gte/__lt/__lte` range params.
///
/// Gate order:
/// 1. Must be readable.
/// 2. Must not be a list.
/// 3. Must not carry `Sensitive` meaning.
/// 4. DataType must not be `Json` or `Binary`.
/// 5. DataType must be ordered/comparable: Integer, Float, DateTime, or Date.
///
/// Gate 5 is DataType-based (Integer/Float/DateTime/Date), NOT meaning-based, so
/// Money/Quantity/Percentage fields — excluded by `is_filter_field`'s meaning gate —
/// still get range params.
pub fn is_range_filter_field(field: &FieldDef) -> bool {
    if !field.readable {
        return false;
    } // gate 1
    if field.is_list {
        return false;
    } // gate 2
    if matches!(field.meaning, FieldMeaning::Sensitive) {
        return false;
    } // gate 3
    if matches!(field.data_type, DataType::Json | DataType::Binary) {
        return false;
    } // gate 4
    matches!(
        field.data_type,
        DataType::Integer | DataType::Float | DataType::DateTime | DataType::Date
    )
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

/// Split `"field__op"` on the LAST `__` separator.
///
/// Returns `Some(("field", "op"))` or `None` if no `__` is present.
/// Uses `rfind` (not `find`) so field names that themselves contain `__` split correctly.
fn split_op_key(key: &str) -> Option<(&str, &str)> {
    let pos = key.rfind("__")?;
    Some((&key[..pos], &key[pos + 2..]))
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
    mut filters: serde_json::Value,
    limit: u64,
    offset: u64,
    db: &sea_orm::DatabaseConnection,
    tenant_id: Option<i64>,
) -> ProjectionReadResult<DispatchResult> {
    let backend = db.get_database_backend();
    // Clamp the requested limit to MAX_LIMIT regardless of caller. The schema
    // advertises `maximum: 100`, but a caller invoking `dispatch` directly could
    // pass an arbitrary `u64`; without this clamp `u64::MAX as i64` wraps negative.
    let limit = limit.min(MAX_LIMIT);
    let offset = offset.min(MAX_OFFSET);
    let table = service.resolved_table();

    // Extract `sort` BEFORE the filter loop (Pitfall 4 — must not appear as a filter key).
    // `filters` is `mut` so we can remove the key in-place without cloning the whole object.
    let sort_param: Option<String> = if let Some(obj) = filters.as_object_mut() {
        obj.remove("sort")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    } else {
        None
    };

    // Parse sort into (column, direction). A field is sortable if it is eligible for
    // equality (is_filter_field) OR range (is_range_filter_field) filtering — the latter
    // admits ordered numeric meanings (Money/Quantity/Percentage) that the equality
    // allowlist excludes but which the schema advertises range ops on, so a field that
    // can be range-filtered can also be sorted.
    let parsed_sort: Option<(String, &'static str)> = match sort_param.as_deref() {
        None => None,
        Some(s) => {
            let (col, dir) = if let Some(bare) = s.strip_prefix('-') {
                (bare, "DESC")
            } else {
                (s, "ASC")
            };
            match service.fields.iter().find(|f| f.name == col) {
                Some(f) if is_filter_field(f) || is_range_filter_field(f) => {
                    Some((col.to_string(), dir))
                }
                _ => {
                    return Err(ProjectionReadError::InvalidFilter(format!(
                        "unknown or non-sortable field: {col}"
                    )));
                }
            }
        }
    };

    let mut where_clauses: Vec<String> = Vec::new();
    let mut values: Vec<sea_orm::Value> = Vec::new();
    let mut idx = 1usize;

    if let Some(obj) = filters.as_object() {
        for (key, val) in obj {
            if let Some((base, op)) = split_op_key(key) {
                // Op path — validate the op suffix against the allowlist, then the base
                // field against the appropriate field allowlist. All values bound.
                let op_sql = match op {
                    "gt" => ">",
                    "gte" => ">=",
                    "lt" => "<",
                    "lte" => "<=",
                    "ne" => "!=",
                    "in" => "IN",
                    _ => {
                        return Err(ProjectionReadError::InvalidFilter(format!(
                            "unknown op suffix '{op}' in filter key '{key}'"
                        )));
                    }
                };
                // Validate base field against the appropriate allowlist (D-10/D-12).
                // gt/gte/lt/lte → is_range_filter_field; ne/in → is_filter_field.
                let _base_field = match service.fields.iter().find(|f| f.name == base) {
                    Some(f)
                        if matches!(op, "gt" | "gte" | "lt" | "lte")
                            && is_range_filter_field(f) =>
                    {
                        f
                    }
                    Some(f) if matches!(op, "ne" | "in") && is_filter_field(f) => f,
                    _ => {
                        return Err(ProjectionReadError::InvalidFilter(format!(
                            "unknown or non-filterable filter field: {key}"
                        )));
                    }
                };

                if op == "in" {
                    let arr = val.as_array().ok_or_else(|| {
                        ProjectionReadError::InvalidFilter(format!(
                            "'__in' value for '{base}' must be an array"
                        ))
                    })?;
                    if arr.is_empty() {
                        return Err(ProjectionReadError::InvalidFilter(format!(
                            "'__in' array for '{base}' must not be empty"
                        )));
                    }
                    // Build parameterized IN placeholders. `idx` advances by `arr.len()` in
                    // one step (after collecting) so the index sequence stays correct for
                    // subsequent clauses. Each element is bound separately below.
                    let placeholders: Vec<String> = (0..arr.len())
                        .map(|i| placeholder(backend, idx + i))
                        .collect();
                    idx += arr.len();
                    where_clauses.push(format!("\"{}\" IN ({})", base, placeholders.join(", ")));
                    for item in arr {
                        values.push(json_to_sea_value(item));
                    }
                } else {
                    where_clauses.push(format!(
                        "\"{}\" {} {}",
                        base,
                        op_sql,
                        placeholder(backend, idx)
                    ));
                    values.push(json_to_sea_value(val));
                    idx += 1;
                }
            } else {
                // Equality path — byte-for-byte identical to the pre-extension loop.
                // ALLOWLIST: the filter key must name a field that is FILTER-ELIGIBLE
                // (the exact predicate that gates the input schema), not merely a known
                // field. This prevents an agent from filtering on a Sensitive,
                // write-only, list, or Json/Binary field that the schema deliberately
                // excludes — which would otherwise leak the column via `SELECT *` or
                // enable an oracle attack. Unknown keys are never interpolated.
                match service.fields.iter().find(|f| &f.name == key) {
                    Some(field) if is_filter_field(field) => {}
                    _ => {
                        return Err(ProjectionReadError::InvalidFilter(format!(
                            "unknown or non-filterable filter field: {key}"
                        )));
                    }
                }
                where_clauses.push(format!("\"{}\" = {}", key, placeholder(backend, idx)));
                values.push(json_to_sea_value(val));
                idx += 1;
            }
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
                return Err(ProjectionReadError::InvalidFilter(
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
        .map_err(|e| ProjectionReadError::Database(format!("Count query failed: {e}")))?;
    let total: u64 = count_row
        .and_then(|r| r.try_get_by::<i64, _>("cnt").ok())
        .unwrap_or(0) as u64;

    // Deterministic ordering for stable offset pagination. Without ORDER BY,
    // offset-based pages can overlap or skip rows under concurrent writes. The
    // tiebreaker column is chosen from the projection's own fields (the Identifier
    // field, else the first field) — never from the call payload — so it cannot
    // be an injection vector. A user-supplied `sort` (parsed above and validated
    // against the is_filter_field allowlist) is placed BEFORE the tiebreaker.
    let order_col = service
        .fields
        .iter()
        .find(|f| matches!(f.meaning, FieldMeaning::Identifier))
        .or_else(|| service.fields.first())
        .map(|f| f.name.clone());
    let order_str = match (&parsed_sort, &order_col) {
        (Some((col, dir)), Some(tiebreaker)) if col != tiebreaker => {
            format!(" ORDER BY \"{col}\" {dir}, \"{tiebreaker}\"")
        }
        (Some((col, dir)), _) => format!(" ORDER BY \"{col}\" {dir}"),
        (None, Some(tiebreaker)) => format!(" ORDER BY \"{tiebreaker}\""),
        (None, None) => String::new(),
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
        .map_err(|e| ProjectionReadError::Database(format!("List query failed: {e}")))?;

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
            ProjectionReadError::InvalidFilter(msg) => {
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

    /// split_op_key splits on the LAST `__` (rfind).
    #[test]
    fn test_split_op_key_basic() {
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
            ProjectionReadError::InvalidFilter(msg) => {
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
            ProjectionReadError::InvalidFilter(_) => {}
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
            ProjectionReadError::InvalidFilter(msg) => {
                assert!(
                    msg.contains("non-sortable") || msg.contains("non-filterable"),
                    "msg: {msg}"
                );
            }
            other => panic!("expected InvalidFilter, got: {other:?}"),
        }
    }

    /// `total__gt` and `total__lte` return the correct subset of rows.
    ///
    /// Seed: Alice=100, Bob=200, Carol=150, Dave=250.
    /// `total__gt 150` → Bob(200) + Dave(250) = 2 rows.
    /// `total__lte 150` → Alice(100) + Carol(150) = 2 rows.
    #[tokio::test]
    async fn range_filter_returns_correct_rows() {
        let db = setup_orders_db().await;
        let service = order_service_no_tenant();

        // gt: strictly greater than 150 → Bob(200) + Dave(250)
        let result = dispatch(
            &service,
            serde_json::json!({"total__gt": 150.0}),
            10,
            0,
            &db,
            None,
        )
        .await
        .expect("total__gt dispatch ok");
        assert_eq!(result.rows.len(), 2, "total__gt 150: Bob + Dave");

        // lte: less-than-or-equal 150 → Alice(100) + Carol(150)
        let result = dispatch(
            &service,
            serde_json::json!({"total__lte": 150.0}),
            10,
            0,
            &db,
            None,
        )
        .await
        .expect("total__lte dispatch ok");
        assert_eq!(result.rows.len(), 2, "total__lte 150: Alice + Carol");
    }

    /// `status__in` returns exactly the rows whose status is in the array.
    ///
    /// Seed: Alice=pending, Bob=shipped, Carol=pending, Dave=shipped.
    #[tokio::test]
    async fn in_filter_returns_correct_rows() {
        let db = setup_orders_db().await;
        let service = order_service_no_tenant();

        // in ["pending"] → Alice + Carol
        let result = dispatch(
            &service,
            serde_json::json!({"status__in": ["pending"]}),
            10,
            0,
            &db,
            None,
        )
        .await
        .expect("status__in dispatch ok");
        assert_eq!(result.rows.len(), 2, "status__in [pending]: Alice + Carol");
        for row in &result.rows {
            assert_eq!(
                row["status"],
                serde_json::Value::String("pending".to_string())
            );
        }

        // empty array → Err(InvalidFilter)
        let err_result = dispatch(
            &service,
            serde_json::json!({"status__in": []}),
            10,
            0,
            &db,
            None,
        )
        .await;
        assert!(err_result.is_err(), "empty __in must be an error");
        match err_result.unwrap_err() {
            ProjectionReadError::InvalidFilter(_) => {}
            other => panic!("expected InvalidFilter, got: {other:?}"),
        }
    }

    /// `sort=id` (ASC) and `sort=-id` (DESC) order rows correctly.
    ///
    /// Uses `id` (Identifier meaning → passes `is_filter_field`) so sort validation passes.
    #[tokio::test]
    async fn sort_orders_rows() {
        let db = setup_orders_db().await;
        let service = order_service_no_tenant();

        // sort=id → ascending by id: 1,2,3,4
        let asc = dispatch(
            &service,
            serde_json::json!({"sort": "id"}),
            10,
            0,
            &db,
            None,
        )
        .await
        .expect("sort=id dispatch ok");
        assert_eq!(asc.rows.len(), 4);
        let ids_asc: Vec<i64> = asc.rows.iter().map(|r| r["id"].as_i64().unwrap()).collect();
        assert_eq!(ids_asc, vec![1, 2, 3, 4], "asc by id");

        // sort=-id → descending: 4,3,2,1
        let desc = dispatch(
            &service,
            serde_json::json!({"sort": "-id"}),
            10,
            0,
            &db,
            None,
        )
        .await
        .expect("sort=-id dispatch ok");
        assert_eq!(desc.rows.len(), 4);
        let ids_desc: Vec<i64> = desc
            .rows
            .iter()
            .map(|r| r["id"].as_i64().unwrap())
            .collect();
        assert_eq!(ids_desc, vec![4, 3, 2, 1], "desc by id");
    }

    /// Equality filter `{"status": "pending"}` returns the same rows as before
    /// the `__op`/`sort` extension (back-compat).
    #[tokio::test]
    async fn equality_filter_backcompat() {
        let db = setup_orders_db().await;
        let service = order_service_no_tenant();

        let result = dispatch(
            &service,
            serde_json::json!({"status": "pending"}),
            10,
            0,
            &db,
            None,
        )
        .await
        .expect("equality filter dispatch ok");

        assert_eq!(result.rows.len(), 2, "equality filter: Alice + Carol");
        for row in &result.rows {
            assert_eq!(
                row["status"],
                serde_json::Value::String("pending".to_string()),
                "all rows must have status=pending"
            );
        }
    }

    /// SC#3 / T-239-02: a soft-deleted row is excluded from dispatch results by construction.
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
