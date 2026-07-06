//! Database schema tool - get table structure information

use crate::error::{McpError, Result};
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct SchemaInfo {
    pub tables: Vec<TableInfo>,
}

#[derive(Debug, Serialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Serialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub primary_key: bool,
    pub default_value: Option<String>,
}

pub async fn execute(project_root: &Path, table_filter: Option<&str>) -> Result<SchemaInfo> {
    // Get database URL from .env
    let database_url = get_database_url(project_root)?;

    // Connect to database
    let db: DatabaseConnection = Database::connect(&database_url)
        .await
        .map_err(|e| McpError::DatabaseError(format!("Failed to connect: {e}")))?;

    let tables = match db.get_database_backend() {
        DatabaseBackend::Sqlite => get_sqlite_schema(&db, table_filter).await?,
        DatabaseBackend::Postgres => get_postgres_schema(&db, table_filter).await?,
        DatabaseBackend::MySql => get_mysql_schema(&db, table_filter).await?,
    };

    Ok(SchemaInfo { tables })
}

async fn get_sqlite_schema(
    db: &DatabaseConnection,
    table_filter: Option<&str>,
) -> Result<Vec<TableInfo>> {
    let mut tables = Vec::new();

    // Get all tables
    let table_query = if let Some(filter) = table_filter {
        format!(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='{filter}' AND name NOT LIKE 'sqlite_%'"
        )
    } else {
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'".to_string()
    };

    let table_rows = db
        .query_all(Statement::from_string(DatabaseBackend::Sqlite, table_query))
        .await
        .map_err(|e| McpError::DatabaseError(format!("Failed to get tables: {e}")))?;

    for row in table_rows {
        let table_name: String = row
            .try_get_by("name")
            .map_err(|e| McpError::DatabaseError(format!("Failed to get table name: {e}")))?;

        // Get columns for this table
        let column_query = format!("PRAGMA table_info('{table_name}')");
        let column_rows = db
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                column_query,
            ))
            .await
            .map_err(|e| McpError::DatabaseError(format!("Failed to get columns: {e}")))?;

        let columns: Vec<ColumnInfo> = column_rows
            .iter()
            .filter_map(|col| {
                let name: String = col.try_get_by("name").ok()?;
                let data_type: String = col.try_get_by("type").ok().unwrap_or_default();
                let notnull: i32 = col.try_get_by("notnull").ok().unwrap_or(0);
                let pk: i32 = col.try_get_by("pk").ok().unwrap_or(0);
                let dflt_value: Option<String> = col.try_get_by("dflt_value").ok();

                Some(ColumnInfo {
                    name,
                    data_type,
                    nullable: notnull == 0,
                    primary_key: pk == 1,
                    default_value: dflt_value,
                })
            })
            .collect();

        tables.push(TableInfo {
            name: table_name,
            columns,
        });
    }

    Ok(tables)
}

async fn get_postgres_schema(
    db: &DatabaseConnection,
    table_filter: Option<&str>,
) -> Result<Vec<TableInfo>> {
    let mut tables = Vec::new();

    // Get all tables from information_schema
    let table_query = if let Some(filter) = table_filter {
        format!(
            "SELECT table_name FROM information_schema.tables
             WHERE table_schema = 'public' AND table_type = 'BASE TABLE' AND table_name = '{filter}'"
        )
    } else {
        "SELECT table_name FROM information_schema.tables
         WHERE table_schema = 'public' AND table_type = 'BASE TABLE'"
            .to_string()
    };

    let table_rows = db
        .query_all(Statement::from_string(
            DatabaseBackend::Postgres,
            table_query,
        ))
        .await
        .map_err(|e| McpError::DatabaseError(format!("Failed to get tables: {e}")))?;

    for row in table_rows {
        let table_name: String = row
            .try_get_by("table_name")
            .map_err(|e| McpError::DatabaseError(format!("Failed to get table name: {e}")))?;

        // Get columns for this table
        let column_query = format!(
            "SELECT column_name, data_type, is_nullable, column_default
             FROM information_schema.columns
             WHERE table_schema = 'public' AND table_name = '{table_name}'
             ORDER BY ordinal_position"
        );

        let column_rows = db
            .query_all(Statement::from_string(
                DatabaseBackend::Postgres,
                column_query,
            ))
            .await
            .map_err(|e| McpError::DatabaseError(format!("Failed to get columns: {e}")))?;

        // Get primary key columns
        let pk_query = format!(
            "SELECT a.attname
             FROM pg_index i
             JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
             WHERE i.indrelid = '{table_name}'::regclass AND i.indisprimary"
        );

        let pk_rows = db
            .query_all(Statement::from_string(DatabaseBackend::Postgres, pk_query))
            .await
            .unwrap_or_default();

        let pk_columns: Vec<String> = pk_rows
            .iter()
            .filter_map(|row| row.try_get_by::<String, _>("attname").ok())
            .collect();

        let columns: Vec<ColumnInfo> = column_rows
            .iter()
            .filter_map(|col| {
                let name: String = col.try_get_by("column_name").ok()?;
                let data_type: String = col.try_get_by("data_type").ok()?;
                let is_nullable: String = col.try_get_by("is_nullable").ok().unwrap_or_default();
                let default_value: Option<String> = col.try_get_by("column_default").ok();

                Some(ColumnInfo {
                    name: name.clone(),
                    data_type,
                    nullable: is_nullable == "YES",
                    primary_key: pk_columns.contains(&name),
                    default_value,
                })
            })
            .collect();

        tables.push(TableInfo {
            name: table_name,
            columns,
        });
    }

    Ok(tables)
}

async fn get_mysql_schema(
    db: &DatabaseConnection,
    table_filter: Option<&str>,
) -> Result<Vec<TableInfo>> {
    let mut tables = Vec::new();

    // Get database name from connection
    let db_name_result = db
        .query_one(Statement::from_string(
            DatabaseBackend::MySql,
            "SELECT DATABASE()".to_string(),
        ))
        .await
        .map_err(|e| McpError::DatabaseError(format!("Failed to get database name: {e}")))?;

    let db_name: String = db_name_result
        .and_then(|row| row.try_get_by_index::<String>(0).ok())
        .unwrap_or_default();

    // Get all tables
    let table_query = if let Some(filter) = table_filter {
        format!(
            "SELECT table_name FROM information_schema.tables
             WHERE table_schema = '{db_name}' AND table_type = 'BASE TABLE' AND table_name = '{filter}'"
        )
    } else {
        format!(
            "SELECT table_name FROM information_schema.tables
             WHERE table_schema = '{db_name}' AND table_type = 'BASE TABLE'"
        )
    };

    let table_rows = db
        .query_all(Statement::from_string(DatabaseBackend::MySql, table_query))
        .await
        .map_err(|e| McpError::DatabaseError(format!("Failed to get tables: {e}")))?;

    for row in table_rows {
        let table_name: String = row
            .try_get_by("table_name")
            .or_else(|_| row.try_get_by("TABLE_NAME"))
            .map_err(|e| McpError::DatabaseError(format!("Failed to get table name: {e}")))?;

        // Get columns for this table
        let column_query = format!(
            "SELECT column_name, data_type, is_nullable, column_default, column_key
             FROM information_schema.columns
             WHERE table_schema = '{db_name}' AND table_name = '{table_name}'
             ORDER BY ordinal_position"
        );

        let column_rows = db
            .query_all(Statement::from_string(DatabaseBackend::MySql, column_query))
            .await
            .map_err(|e| McpError::DatabaseError(format!("Failed to get columns: {e}")))?;

        let columns: Vec<ColumnInfo> = column_rows
            .iter()
            .filter_map(|col| {
                let name: String = col
                    .try_get_by("column_name")
                    .or_else(|_| col.try_get_by("COLUMN_NAME"))
                    .ok()?;
                let data_type: String = col
                    .try_get_by("data_type")
                    .or_else(|_| col.try_get_by("DATA_TYPE"))
                    .ok()?;
                let is_nullable: String = col
                    .try_get_by("is_nullable")
                    .or_else(|_| col.try_get_by("IS_NULLABLE"))
                    .ok()
                    .unwrap_or_default();
                let column_key: String = col
                    .try_get_by("column_key")
                    .or_else(|_| col.try_get_by("COLUMN_KEY"))
                    .ok()
                    .unwrap_or_default();
                let default_value: Option<String> = col
                    .try_get_by("column_default")
                    .or_else(|_| col.try_get_by("COLUMN_DEFAULT"))
                    .ok();

                Some(ColumnInfo {
                    name,
                    data_type,
                    nullable: is_nullable == "YES",
                    primary_key: column_key == "PRI",
                    default_value,
                })
            })
            .collect();

        tables.push(TableInfo {
            name: table_name,
            columns,
        });
    }

    Ok(tables)
}

fn get_database_url(project_root: &Path) -> Result<String> {
    dotenvy::from_path(project_root.join(".env")).ok();

    std::env::var("DATABASE_URL")
        .map_err(|_| McpError::ConfigError("DATABASE_URL not set in .env".to_string()))
}
