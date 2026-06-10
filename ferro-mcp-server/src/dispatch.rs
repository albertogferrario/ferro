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

/// Executes the projection's parameterized read path with filter-key allowlisting and
/// offset-based pagination.
///
/// Security: filter KEYS are validated against `service.fields` (allowlist) before any SQL
/// assembly; unknown keys return `Err` and are never interpolated. Filter VALUES are bound
/// via `Statement::from_sql_and_values`, never string-interpolated. Table name is derived
/// from `service.name` (developer-controlled), not from the call payload.
///
/// No tenant or ownership filter is applied here — Phase 200 owns that seam.
pub async fn dispatch(
    service: &ServiceDef,
    filters: serde_json::Value,
    limit: u64,
    offset: u64,
    db: &sea_orm::DatabaseConnection,
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
