//! CRUD operations tool — model-aware create, list, update, delete via parameterized SQL

use crate::error::{McpError, Result};
use crate::tools::list_models::{self, ModelDetails};
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement};
use serde::Serialize;
use std::path::Path;

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct CrudResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct CrudListResult {
    pub data: Vec<serde_json::Value>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}

// ---------------------------------------------------------------------------
// Model metadata
// ---------------------------------------------------------------------------

pub(crate) struct ModelMeta {
    pub(crate) table_name: String,
    pub(crate) primary_key: String,
    pub(crate) fields: Vec<FieldMeta>,
}

pub(crate) struct FieldMeta {
    pub(crate) name: String,
    pub(crate) column_name: String,
    pub(crate) field_type: String,
    pub(crate) is_nullable: bool,
    pub(crate) is_primary_key: bool,
}

fn get_model_metadata(project_root: &Path, model_name: &str) -> Result<ModelMeta> {
    let models = list_models::execute(project_root)?;
    let model = find_model(&models, model_name)?;

    let table_name = model
        .table
        .clone()
        .unwrap_or_else(|| model.name.to_lowercase() + "s");

    let primary_key = model
        .fields
        .iter()
        .find(|f| f.is_primary_key)
        .map(|f| f.name.clone())
        .unwrap_or_else(|| "id".to_string());

    let fields = model
        .fields
        .iter()
        .map(|f| FieldMeta {
            column_name: f.name.clone(),
            name: f.name.clone(),
            field_type: f.field_type.clone(),
            is_nullable: f.is_nullable,
            is_primary_key: f.is_primary_key,
        })
        .collect();

    Ok(ModelMeta {
        table_name,
        primary_key,
        fields,
    })
}

pub(crate) fn find_model<'a>(models: &'a [ModelDetails], name: &str) -> Result<&'a ModelDetails> {
    let lower = name.to_lowercase();
    models
        .iter()
        .find(|m| m.name.to_lowercase() == lower)
        .ok_or_else(|| {
            let available: Vec<&str> = models.iter().map(|m| m.name.as_str()).collect();
            McpError::NotFound(format!(
                "Model '{}' not found. Available: {}",
                name,
                available.join(", ")
            ))
        })
}

// ---------------------------------------------------------------------------
// Database helpers
// ---------------------------------------------------------------------------

fn get_database_url(project_root: &Path) -> Result<String> {
    dotenvy::from_path(project_root.join(".env")).ok();
    std::env::var("DATABASE_URL")
        .map_err(|_| McpError::ConfigError("DATABASE_URL not set in .env".to_string()))
}

async fn connect_db(project_root: &Path) -> Result<DatabaseConnection> {
    let url = get_database_url(project_root)?;
    Database::connect(&url)
        .await
        .map_err(|e| McpError::DatabaseError(format!("Failed to connect: {e}")))
}

/// Convert a serde_json::Value to a sea_orm::Value appropriate for the field type.
pub(crate) fn json_to_sea_value(val: &serde_json::Value, field_type: &str) -> sea_orm::Value {
    match val {
        serde_json::Value::Null => sea_orm::Value::String(None),
        serde_json::Value::Bool(b) => sea_orm::Value::Bool(Some(*b)),
        serde_json::Value::Number(n) => {
            if field_type.contains("i64") {
                sea_orm::Value::BigInt(n.as_i64())
            } else if field_type.contains("i32") || field_type.contains("i16") {
                sea_orm::Value::Int(n.as_i64().map(|v| v as i32))
            } else if field_type.contains("f64") || field_type.contains("f32") {
                sea_orm::Value::Double(n.as_f64())
            } else {
                // Default: try integer first, then float
                if let Some(i) = n.as_i64() {
                    sea_orm::Value::BigInt(Some(i))
                } else {
                    sea_orm::Value::Double(n.as_f64())
                }
            }
        }
        serde_json::Value::String(s) => sea_orm::Value::String(Some(Box::new(s.clone()))),
        other => sea_orm::Value::String(Some(Box::new(other.to_string()))),
    }
}

/// Build a parameter placeholder for the given backend and 1-based index.
pub(crate) fn placeholder(backend: DatabaseBackend, index: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${index}"),
        _ => "?".to_string(),
    }
}

/// Extract query result rows into JSON objects.
fn rows_to_json(rows: &[sea_orm::QueryResult]) -> Vec<serde_json::Value> {
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

/// Validate that a column name exists in model metadata. Returns the column name
/// if valid, or an error describing the mismatch.
pub(crate) fn validate_column(meta: &ModelMeta, name: &str) -> Result<String> {
    meta.fields
        .iter()
        .find(|f| f.name == name || f.column_name == name)
        .map(|f| f.column_name.clone())
        .ok_or_else(|| {
            let known: Vec<&str> = meta.fields.iter().map(|f| f.name.as_str()).collect();
            McpError::ToolError(format!(
                "Unknown field '{}'. Available: {}",
                name,
                known.join(", ")
            ))
        })
}

/// Find the FieldMeta for a given field name.
pub(crate) fn find_field<'a>(meta: &'a ModelMeta, name: &str) -> Option<&'a FieldMeta> {
    meta.fields
        .iter()
        .find(|f| f.name == name || f.column_name == name)
}

// ---------------------------------------------------------------------------
// Pagination helpers
// ---------------------------------------------------------------------------

/// Normalize page number: clamp to minimum of 1.
pub(crate) fn normalize_page(page: Option<u64>) -> u64 {
    page.unwrap_or(1).max(1)
}

/// Normalize per-page count: default 25, clamp to [1, 100].
pub(crate) fn normalize_per_page(per_page: Option<u64>) -> u64 {
    per_page.unwrap_or(25).clamp(1, 100)
}

// ---------------------------------------------------------------------------
// Required field validation
// ---------------------------------------------------------------------------

/// Validate that all required fields are present in the data object.
/// Returns the name of the first missing required field, or None if all present.
pub(crate) fn find_missing_required_field<'a>(
    fields: &'a [FieldMeta],
    data_keys: &[&str],
) -> Option<&'a str> {
    for field in fields {
        if field.is_primary_key || field.is_nullable {
            continue;
        }
        if field.name == "created_at" || field.name == "updated_at" {
            continue;
        }
        if !data_keys.contains(&field.name.as_str()) {
            return Some(&field.name);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// CRUD operations
// ---------------------------------------------------------------------------

pub async fn create(
    project_root: &Path,
    model: &str,
    data: &serde_json::Value,
) -> Result<CrudResult> {
    let meta = get_model_metadata(project_root, model)?;
    let db = connect_db(project_root).await?;
    let backend = db.get_database_backend();

    let obj = data
        .as_object()
        .ok_or_else(|| McpError::ToolError("data must be a JSON object".to_string()))?;

    // Validate required fields (non-nullable, non-PK fields that lack defaults)
    let data_keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
    if let Some(missing) = find_missing_required_field(&meta.fields, &data_keys) {
        return Err(McpError::ToolError(format!(
            "Missing required field '{missing}' for model '{model}'"
        )));
    }

    // Build column and placeholder lists from provided data
    let mut columns = Vec::new();
    let mut placeholders = Vec::new();
    let mut values: Vec<sea_orm::Value> = Vec::new();
    let mut idx = 1usize;

    for (key, val) in obj {
        let col = validate_column(&meta, key)?;
        let field_meta = find_field(&meta, key);
        let field_type = field_meta.map(|f| f.field_type.as_str()).unwrap_or("");

        columns.push(format!("\"{col}\""));
        placeholders.push(placeholder(backend, idx));
        values.push(json_to_sea_value(val, field_type));
        idx += 1;
    }

    let cols_str = columns.join(", ");
    let vals_str = placeholders.join(", ");

    let row = match backend {
        DatabaseBackend::Postgres => {
            let sql = format!(
                "INSERT INTO \"{}\" ({}) VALUES ({}) RETURNING *",
                meta.table_name, cols_str, vals_str
            );
            let stmt = Statement::from_sql_and_values(backend, &sql, values);
            db.query_all(stmt)
                .await
                .map_err(|e| McpError::DatabaseError(format!("Insert failed: {e}")))?
        }
        _ => {
            // SQLite / MySQL: INSERT then SELECT by last_insert_rowid
            let sql = format!(
                "INSERT INTO \"{}\" ({}) VALUES ({})",
                meta.table_name, cols_str, vals_str
            );
            let stmt = Statement::from_sql_and_values(backend, &sql, values);
            db.execute(stmt)
                .await
                .map_err(|e| McpError::DatabaseError(format!("Insert failed: {e}")))?;

            let select_sql = format!(
                "SELECT * FROM \"{}\" WHERE \"{}\" = last_insert_rowid()",
                meta.table_name, meta.primary_key
            );
            let select_stmt = Statement::from_string(backend, select_sql);
            db.query_all(select_stmt)
                .await
                .map_err(|e| McpError::DatabaseError(format!("Select after insert failed: {e}")))?
        }
    };

    let json_rows = rows_to_json(&row);
    let record = json_rows.into_iter().next();

    Ok(CrudResult {
        success: true,
        data: record,
        message: format!("{model} record created"),
    })
}

pub async fn list(
    project_root: &Path,
    model: &str,
    filters: Option<&serde_json::Value>,
    page: Option<u64>,
    per_page: Option<u64>,
) -> Result<CrudListResult> {
    let meta = get_model_metadata(project_root, model)?;
    let db = connect_db(project_root).await?;
    let backend = db.get_database_backend();

    let page = normalize_page(page);
    let per_page = normalize_per_page(per_page);
    let offset = (page - 1) * per_page;

    let mut where_clauses = Vec::new();
    let mut values: Vec<sea_orm::Value> = Vec::new();
    let mut idx = 1usize;

    if let Some(filters) = filters {
        if let Some(obj) = filters.as_object() {
            for (key, val) in obj {
                let col = validate_column(&meta, key)?;
                let field_meta = find_field(&meta, key);
                let field_type = field_meta.map(|f| f.field_type.as_str()).unwrap_or("");

                where_clauses.push(format!("\"{}\" = {}", col, placeholder(backend, idx)));
                values.push(json_to_sea_value(val, field_type));
                idx += 1;
            }
        }
    }

    let where_str = if where_clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_clauses.join(" AND "))
    };

    // Count query
    let count_sql = format!(
        "SELECT COUNT(*) as cnt FROM \"{}\"{}",
        meta.table_name, where_str
    );
    let count_stmt = Statement::from_sql_and_values(backend, &count_sql, values.clone());
    let count_row = db
        .query_one(count_stmt)
        .await
        .map_err(|e| McpError::DatabaseError(format!("Count query failed: {e}")))?;

    let total: u64 = count_row
        .and_then(|r| r.try_get_by::<i64, _>("cnt").ok())
        .unwrap_or(0) as u64;

    // Data query with pagination
    let limit_str = format!(
        " LIMIT {} OFFSET {}",
        placeholder(backend, idx),
        placeholder(backend, idx + 1)
    );
    values.push(sea_orm::Value::BigInt(Some(per_page as i64)));
    values.push(sea_orm::Value::BigInt(Some(offset as i64)));

    let data_sql = format!(
        "SELECT * FROM \"{}\"{}{}",
        meta.table_name, where_str, limit_str
    );
    let data_stmt = Statement::from_sql_and_values(backend, &data_sql, values);
    let rows = db
        .query_all(data_stmt)
        .await
        .map_err(|e| McpError::DatabaseError(format!("List query failed: {e}")))?;

    let data = rows_to_json(&rows);

    Ok(CrudListResult {
        data,
        total,
        page,
        per_page,
    })
}

pub async fn update(
    project_root: &Path,
    model: &str,
    id: &serde_json::Value,
    data: &serde_json::Value,
) -> Result<CrudResult> {
    let meta = get_model_metadata(project_root, model)?;
    let db = connect_db(project_root).await?;
    let backend = db.get_database_backend();

    let obj = data
        .as_object()
        .ok_or_else(|| McpError::ToolError("data must be a JSON object".to_string()))?;

    if obj.is_empty() {
        return Err(McpError::ToolError(
            "data must contain at least one field to update".to_string(),
        ));
    }

    let mut set_clauses = Vec::new();
    let mut values: Vec<sea_orm::Value> = Vec::new();
    let mut idx = 1usize;

    for (key, val) in obj {
        let col = validate_column(&meta, key)?;
        let field_meta = find_field(&meta, key);
        let field_type = field_meta.map(|f| f.field_type.as_str()).unwrap_or("");

        set_clauses.push(format!("\"{}\" = {}", col, placeholder(backend, idx)));
        values.push(json_to_sea_value(val, field_type));
        idx += 1;
    }

    // PK field type for the WHERE clause
    let pk_field = find_field(&meta, &meta.primary_key);
    let pk_type = pk_field.map(|f| f.field_type.as_str()).unwrap_or("i32");
    values.push(json_to_sea_value(id, pk_type));

    let set_str = set_clauses.join(", ");
    let pk_placeholder = placeholder(backend, idx);

    let row = match backend {
        DatabaseBackend::Postgres => {
            let sql = format!(
                "UPDATE \"{}\" SET {} WHERE \"{}\" = {} RETURNING *",
                meta.table_name, set_str, meta.primary_key, pk_placeholder
            );
            let stmt = Statement::from_sql_and_values(backend, &sql, values);
            db.query_all(stmt)
                .await
                .map_err(|e| McpError::DatabaseError(format!("Update failed: {e}")))?
        }
        _ => {
            let sql = format!(
                "UPDATE \"{}\" SET {} WHERE \"{}\" = {}",
                meta.table_name, set_str, meta.primary_key, pk_placeholder
            );
            let stmt = Statement::from_sql_and_values(backend, &sql, values);
            db.execute(stmt)
                .await
                .map_err(|e| McpError::DatabaseError(format!("Update failed: {e}")))?;

            // Re-select updated row
            let select_sql = format!(
                "SELECT * FROM \"{}\" WHERE \"{}\" = {}",
                meta.table_name,
                meta.primary_key,
                placeholder(backend, 1)
            );
            let select_values = vec![json_to_sea_value(id, pk_type)];
            let select_stmt = Statement::from_sql_and_values(backend, &select_sql, select_values);
            db.query_all(select_stmt)
                .await
                .map_err(|e| McpError::DatabaseError(format!("Select after update failed: {e}")))?
        }
    };

    let json_rows = rows_to_json(&row);
    let record = json_rows.into_iter().next();

    if record.is_none() {
        return Ok(CrudResult {
            success: false,
            data: None,
            message: format!(
                "No {} record found with {} = {}",
                model, meta.primary_key, id
            ),
        });
    }

    Ok(CrudResult {
        success: true,
        data: record,
        message: format!("{model} record updated"),
    })
}

pub async fn delete(
    project_root: &Path,
    model: &str,
    id: &serde_json::Value,
) -> Result<CrudResult> {
    let meta = get_model_metadata(project_root, model)?;
    let db = connect_db(project_root).await?;
    let backend = db.get_database_backend();

    let pk_field = find_field(&meta, &meta.primary_key);
    let pk_type = pk_field.map(|f| f.field_type.as_str()).unwrap_or("i32");

    let sql = format!(
        "DELETE FROM \"{}\" WHERE \"{}\" = {}",
        meta.table_name,
        meta.primary_key,
        placeholder(backend, 1)
    );
    let values = vec![json_to_sea_value(id, pk_type)];
    let stmt = Statement::from_sql_and_values(backend, &sql, values);

    let result = db
        .execute(stmt)
        .await
        .map_err(|e| McpError::DatabaseError(format!("Delete failed: {e}")))?;

    let affected = result.rows_affected();

    if affected == 0 {
        return Ok(CrudResult {
            success: false,
            data: None,
            message: format!(
                "No {} record found with {} = {}",
                model, meta.primary_key, id
            ),
        });
    }

    Ok(CrudResult {
        success: true,
        data: Some(serde_json::json!({
            "deleted": true,
            "id": id,
        })),
        message: format!("{model} record deleted"),
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::list_models::{FieldInfo, ModelDetails};
    use sea_orm::DatabaseBackend;
    use serde_json::json;

    /// Build a ModelMeta for testing with the given fields.
    fn test_meta(fields: Vec<FieldMeta>) -> ModelMeta {
        ModelMeta {
            table_name: "users".to_string(),
            primary_key: "id".to_string(),
            fields,
        }
    }

    fn field(name: &str, field_type: &str, nullable: bool, pk: bool) -> FieldMeta {
        FieldMeta {
            name: name.to_string(),
            column_name: name.to_string(),
            field_type: field_type.to_string(),
            is_nullable: nullable,
            is_primary_key: pk,
        }
    }

    fn test_models() -> Vec<ModelDetails> {
        vec![
            ModelDetails {
                name: "User".to_string(),
                table: Some("users".to_string()),
                path: "src/models/user.rs".to_string(),
                fields: vec![
                    FieldInfo {
                        name: "id".to_string(),
                        field_type: "i32".to_string(),
                        is_primary_key: true,
                        is_nullable: false,
                    },
                    FieldInfo {
                        name: "email".to_string(),
                        field_type: "String".to_string(),
                        is_primary_key: false,
                        is_nullable: false,
                    },
                ],
            },
            ModelDetails {
                name: "Post".to_string(),
                table: Some("posts".to_string()),
                path: "src/models/post.rs".to_string(),
                fields: vec![FieldInfo {
                    name: "id".to_string(),
                    field_type: "i64".to_string(),
                    is_primary_key: true,
                    is_nullable: false,
                }],
            },
        ]
    }

    // -----------------------------------------------------------------------
    // json_to_sea_value tests
    // -----------------------------------------------------------------------

    #[test]
    fn json_to_sea_value_string() {
        let val = json!("hello");
        let result = json_to_sea_value(&val, "String");
        assert_eq!(
            result,
            sea_orm::Value::String(Some(Box::new("hello".to_string())))
        );
    }

    #[test]
    fn json_to_sea_value_bool() {
        let val = json!(true);
        let result = json_to_sea_value(&val, "bool");
        assert_eq!(result, sea_orm::Value::Bool(Some(true)));
    }

    #[test]
    fn json_to_sea_value_null() {
        let val = json!(null);
        let result = json_to_sea_value(&val, "String");
        assert_eq!(result, sea_orm::Value::String(None));
    }

    #[test]
    fn json_to_sea_value_i64_field() {
        let val = json!(42);
        let result = json_to_sea_value(&val, "i64");
        assert_eq!(result, sea_orm::Value::BigInt(Some(42)));
    }

    #[test]
    fn json_to_sea_value_i32_field() {
        let val = json!(42);
        let result = json_to_sea_value(&val, "i32");
        assert_eq!(result, sea_orm::Value::Int(Some(42)));
    }

    #[test]
    fn json_to_sea_value_i16_field() {
        let val = json!(7);
        let result = json_to_sea_value(&val, "i16");
        assert_eq!(result, sea_orm::Value::Int(Some(7)));
    }

    #[test]
    fn json_to_sea_value_f64_field() {
        let val = json!(2.71);
        let result = json_to_sea_value(&val, "f64");
        assert_eq!(result, sea_orm::Value::Double(Some(2.71)));
    }

    #[test]
    fn json_to_sea_value_f32_field() {
        let val = json!(2.5);
        let result = json_to_sea_value(&val, "f32");
        assert_eq!(result, sea_orm::Value::Double(Some(2.5)));
    }

    #[test]
    fn json_to_sea_value_option_i64() {
        // Option<i64> contains "i64", so should match the i64 branch
        let val = json!(99);
        let result = json_to_sea_value(&val, "Option<i64>");
        assert_eq!(result, sea_orm::Value::BigInt(Some(99)));
    }

    #[test]
    fn json_to_sea_value_option_i32() {
        // Option<i32> contains "i32", should match the i32 branch
        let val = json!(10);
        let result = json_to_sea_value(&val, "Option<i32>");
        assert_eq!(result, sea_orm::Value::Int(Some(10)));
    }

    #[test]
    fn json_to_sea_value_option_null() {
        // Null with any Option type should produce String(None)
        let val = json!(null);
        let result = json_to_sea_value(&val, "Option<i64>");
        assert_eq!(result, sea_orm::Value::String(None));
    }

    #[test]
    fn json_to_sea_value_number_unknown_type_integer() {
        // Unknown field type with integer JSON defaults to BigInt
        let val = json!(100);
        let result = json_to_sea_value(&val, "");
        assert_eq!(result, sea_orm::Value::BigInt(Some(100)));
    }

    #[test]
    fn json_to_sea_value_number_unknown_type_float() {
        // Unknown field type with float JSON defaults to Double
        let val = json!(1.5);
        let result = json_to_sea_value(&val, "");
        assert_eq!(result, sea_orm::Value::Double(Some(1.5)));
    }

    #[test]
    fn json_to_sea_value_json_object_serializes_to_string() {
        let val = json!({"key": "value"});
        let result = json_to_sea_value(&val, "String");
        match result {
            sea_orm::Value::String(Some(s)) => {
                assert!(s.contains("key"));
            }
            other => panic!("Expected String, got {other:?}"),
        }
    }

    #[test]
    fn json_to_sea_value_json_array_serializes_to_string() {
        let val = json!([1, 2, 3]);
        let result = json_to_sea_value(&val, "String");
        match result {
            sea_orm::Value::String(Some(s)) => {
                assert!(s.contains("[1,2,3]"));
            }
            other => panic!("Expected String, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // placeholder tests
    // -----------------------------------------------------------------------

    #[test]
    fn placeholder_postgres_format() {
        assert_eq!(placeholder(DatabaseBackend::Postgres, 1), "$1");
        assert_eq!(placeholder(DatabaseBackend::Postgres, 2), "$2");
        assert_eq!(placeholder(DatabaseBackend::Postgres, 10), "$10");
    }

    #[test]
    fn placeholder_sqlite_format() {
        assert_eq!(placeholder(DatabaseBackend::Sqlite, 1), "?");
        assert_eq!(placeholder(DatabaseBackend::Sqlite, 5), "?");
    }

    #[test]
    fn placeholder_mysql_format() {
        assert_eq!(placeholder(DatabaseBackend::MySql, 1), "?");
        assert_eq!(placeholder(DatabaseBackend::MySql, 3), "?");
    }

    // -----------------------------------------------------------------------
    // find_model tests
    // -----------------------------------------------------------------------

    #[test]
    fn find_model_exact_match() {
        let models = test_models();
        let result = find_model(&models, "User");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "User");
    }

    #[test]
    fn find_model_case_insensitive() {
        let models = test_models();
        let result = find_model(&models, "user");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "User");
    }

    #[test]
    fn find_model_uppercase() {
        let models = test_models();
        let result = find_model(&models, "USER");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "User");
    }

    #[test]
    fn find_model_not_found_lists_available() {
        let models = test_models();
        let result = find_model(&models, "Comment");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Comment"));
        assert!(err_msg.contains("User"));
        assert!(err_msg.contains("Post"));
    }

    #[test]
    fn find_model_empty_list() {
        let models: Vec<ModelDetails> = vec![];
        let result = find_model(&models, "User");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("User"));
    }

    // -----------------------------------------------------------------------
    // validate_column tests
    // -----------------------------------------------------------------------

    #[test]
    fn validate_column_known_field() {
        let meta = test_meta(vec![
            field("id", "i32", false, true),
            field("email", "String", false, false),
        ]);
        let result = validate_column(&meta, "email");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "email");
    }

    #[test]
    fn validate_column_unknown_field_lists_available() {
        let meta = test_meta(vec![
            field("id", "i32", false, true),
            field("email", "String", false, false),
        ]);
        let result = validate_column(&meta, "unknown_col");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("unknown_col"));
        assert!(err_msg.contains("id"));
        assert!(err_msg.contains("email"));
    }

    // -----------------------------------------------------------------------
    // find_field tests
    // -----------------------------------------------------------------------

    #[test]
    fn find_field_by_name() {
        let meta = test_meta(vec![
            field("id", "i32", false, true),
            field("email", "String", false, false),
        ]);
        let result = find_field(&meta, "email");
        assert!(result.is_some());
        assert_eq!(result.unwrap().field_type, "String");
    }

    #[test]
    fn find_field_not_found() {
        let meta = test_meta(vec![field("id", "i32", false, true)]);
        let result = find_field(&meta, "nonexistent");
        assert!(result.is_none());
    }

    // -----------------------------------------------------------------------
    // normalize_page tests
    // -----------------------------------------------------------------------

    #[test]
    fn normalize_page_none_defaults_to_1() {
        assert_eq!(normalize_page(None), 1);
    }

    #[test]
    fn normalize_page_zero_clamped_to_1() {
        assert_eq!(normalize_page(Some(0)), 1);
    }

    #[test]
    fn normalize_page_valid_value_unchanged() {
        assert_eq!(normalize_page(Some(5)), 5);
    }

    #[test]
    fn normalize_page_large_value_unchanged() {
        assert_eq!(normalize_page(Some(1000)), 1000);
    }

    // -----------------------------------------------------------------------
    // normalize_per_page tests
    // -----------------------------------------------------------------------

    #[test]
    fn normalize_per_page_none_defaults_to_25() {
        assert_eq!(normalize_per_page(None), 25);
    }

    #[test]
    fn normalize_per_page_zero_clamped_to_1() {
        assert_eq!(normalize_per_page(Some(0)), 1);
    }

    #[test]
    fn normalize_per_page_over_100_capped() {
        assert_eq!(normalize_per_page(Some(200)), 100);
    }

    #[test]
    fn normalize_per_page_exactly_100() {
        assert_eq!(normalize_per_page(Some(100)), 100);
    }

    #[test]
    fn normalize_per_page_valid_value_unchanged() {
        assert_eq!(normalize_per_page(Some(50)), 50);
    }

    // -----------------------------------------------------------------------
    // find_missing_required_field tests
    // -----------------------------------------------------------------------

    #[test]
    fn required_field_missing_non_nullable() {
        let fields = vec![
            field("id", "i32", false, true),
            field("email", "String", false, false),
            field("name", "String", false, false),
        ];
        let data_keys = vec!["email"];
        let result = find_missing_required_field(&fields, &data_keys);
        assert_eq!(result, Some("name"));
    }

    #[test]
    fn required_field_nullable_can_be_omitted() {
        let fields = vec![
            field("id", "i32", false, true),
            field("email", "String", false, false),
            field("bio", "Option<String>", true, false),
        ];
        let data_keys = vec!["email"];
        let result = find_missing_required_field(&fields, &data_keys);
        assert!(result.is_none());
    }

    #[test]
    fn required_field_primary_key_not_required() {
        let fields = vec![
            field("id", "i32", false, true),
            field("email", "String", false, false),
        ];
        let data_keys = vec!["email"];
        let result = find_missing_required_field(&fields, &data_keys);
        assert!(result.is_none());
    }

    #[test]
    fn required_field_created_at_not_required() {
        let fields = vec![
            field("id", "i32", false, true),
            field("email", "String", false, false),
            field("created_at", "DateTime", false, false),
            field("updated_at", "DateTime", false, false),
        ];
        let data_keys = vec!["email"];
        let result = find_missing_required_field(&fields, &data_keys);
        assert!(result.is_none());
    }

    #[test]
    fn required_field_all_present() {
        let fields = vec![
            field("id", "i32", false, true),
            field("email", "String", false, false),
            field("name", "String", false, false),
        ];
        let data_keys = vec!["email", "name"];
        let result = find_missing_required_field(&fields, &data_keys);
        assert!(result.is_none());
    }

    // -----------------------------------------------------------------------
    // Type matching order: i32 vs i64 correctness
    // -----------------------------------------------------------------------

    #[test]
    fn type_matching_i32_does_not_match_i64_branch() {
        // "i32" does not contain "i64", so i32 should not go through BigInt path
        let val = json!(42);
        let i32_result = json_to_sea_value(&val, "i32");
        let i64_result = json_to_sea_value(&val, "i64");
        assert_eq!(i32_result, sea_orm::Value::Int(Some(42)));
        assert_eq!(i64_result, sea_orm::Value::BigInt(Some(42)));
    }

    #[test]
    fn type_matching_option_i32_does_not_match_i64() {
        // "Option<i32>" does not contain "i64"
        let val = json!(42);
        let result = json_to_sea_value(&val, "Option<i32>");
        assert_eq!(result, sea_orm::Value::Int(Some(42)));
    }
}
