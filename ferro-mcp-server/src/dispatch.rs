use ferro_projections::ServiceDef;
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use serde::Serialize;

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
    // TODO: ServiceDef.table field for irregular plurals / custom table names
    let table = format!("{}s", service.name.to_lowercase());

    let mut where_clauses: Vec<String> = Vec::new();
    let mut values: Vec<sea_orm::Value> = Vec::new();
    let mut idx = 1usize;

    if let Some(obj) = filters.as_object() {
        for (key, val) in obj {
            // ALLOWLIST: filter key must be a known field name — never interpolate unknown keys
            if !service.fields.iter().any(|f| &f.name == key) {
                return Err(crate::Error::Database(format!(
                    "unknown filter field: {key}"
                )));
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

    // DATA query with LIMIT/OFFSET bound as parameters
    let limit_str = format!(
        " LIMIT {} OFFSET {}",
        placeholder(backend, idx),
        placeholder(backend, idx + 1)
    );
    values.push(sea_orm::Value::BigInt(Some(limit as i64)));
    values.push(sea_orm::Value::BigInt(Some(offset as i64)));

    let data_sql = format!("SELECT * FROM \"{table}\"{where_str}{limit_str}");
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
