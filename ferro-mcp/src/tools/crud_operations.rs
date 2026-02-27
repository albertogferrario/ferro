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

struct ModelMeta {
    table_name: String,
    primary_key: String,
    fields: Vec<FieldMeta>,
}

struct FieldMeta {
    name: String,
    column_name: String,
    field_type: String,
    is_nullable: bool,
    is_primary_key: bool,
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

fn find_model<'a>(models: &'a [ModelDetails], name: &str) -> Result<&'a ModelDetails> {
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
fn json_to_sea_value(val: &serde_json::Value, field_type: &str) -> sea_orm::Value {
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
fn placeholder(backend: DatabaseBackend, index: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${index}"),
        _ => "?".to_string(),
    }
}

/// Extract query result rows into JSON objects.
fn rows_to_json(
    rows: &[sea_orm::QueryResult],
) -> Vec<serde_json::Value> {
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
fn validate_column(meta: &ModelMeta, name: &str) -> Result<String> {
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
fn find_field<'a>(meta: &'a ModelMeta, name: &str) -> Option<&'a FieldMeta> {
    meta.fields
        .iter()
        .find(|f| f.name == name || f.column_name == name)
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
    for field in &meta.fields {
        if field.is_primary_key || field.is_nullable {
            continue;
        }
        // Heuristic: created_at/updated_at often have defaults
        if field.name == "created_at" || field.name == "updated_at" {
            continue;
        }
        if !obj.contains_key(&field.name) {
            return Err(McpError::ToolError(format!(
                "Missing required field '{}' for model '{}'",
                field.name, model
            )));
        }
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

    let page = page.unwrap_or(1).max(1);
    let per_page = per_page.unwrap_or(25).min(100);
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
            message: format!("No {} record found with {} = {}", model, meta.primary_key, id),
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
            message: format!("No {} record found with {} = {}", model, meta.primary_key, id),
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
