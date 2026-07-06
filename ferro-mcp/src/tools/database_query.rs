//! Database query tool - execute read-only SQL queries

use crate::error::{McpError, Result};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
}

pub async fn execute(project_root: &Path, query: &str) -> Result<QueryResult> {
    // Validate query is read-only
    let query_upper = query.trim().to_uppercase();
    if !query_upper.starts_with("SELECT")
        && !query_upper.starts_with("SHOW")
        && !query_upper.starts_with("DESCRIBE")
        && !query_upper.starts_with("EXPLAIN")
    {
        return Err(McpError::ToolError(
            "Only SELECT, SHOW, DESCRIBE, and EXPLAIN queries are allowed".to_string(),
        ));
    }

    // Get database URL from .env
    let database_url = get_database_url(project_root)?;

    // Connect to database
    let db: DatabaseConnection = Database::connect(&database_url)
        .await
        .map_err(|e| McpError::DatabaseError(format!("Failed to connect: {e}")))?;

    // Execute query
    let result = db
        .query_all(Statement::from_string(
            db.get_database_backend(),
            query.to_string(),
        ))
        .await
        .map_err(|e| McpError::DatabaseError(format!("Query failed: {e}")))?;

    if result.is_empty() {
        return Ok(QueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
            row_count: 0,
        });
    }

    // Extract column names from first row
    let columns: Vec<String> = result
        .first()
        .map(|row| row.column_names().iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();

    // Extract row values
    let rows: Vec<Vec<serde_json::Value>> = result
        .iter()
        .map(|row| {
            columns
                .iter()
                .map(|col| {
                    row.try_get_by::<String, _>(col.as_str())
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
                        .unwrap_or(serde_json::Value::Null)
                })
                .collect()
        })
        .collect();

    let row_count = rows.len();

    Ok(QueryResult {
        columns,
        rows,
        row_count,
    })
}

fn get_database_url(project_root: &Path) -> Result<String> {
    dotenvy::from_path(project_root.join(".env")).ok();

    std::env::var("DATABASE_URL")
        .map_err(|_| McpError::ConfigError("DATABASE_URL not set in .env".to_string()))
}
