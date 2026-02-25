//! Relation map tool - show FK relationships between tables

use crate::error::{McpError, Result};
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct RelationMapInfo {
    pub relations: Vec<Relation>,
    pub summary: RelationSummary,
}

#[derive(Debug, Serialize)]
pub struct Relation {
    pub from_table: String,
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
    pub relation_type: String,
    pub constraint_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RelationSummary {
    pub total_relations: usize,
    pub tables_with_fks: Vec<String>,
    pub referenced_tables: Vec<String>,
}

pub async fn execute(project_root: &Path) -> Result<RelationMapInfo> {
    let database_url = get_database_url(project_root)?;

    let db: DatabaseConnection = Database::connect(&database_url)
        .await
        .map_err(|e| McpError::DatabaseError(format!("Failed to connect: {e}")))?;

    let relations = match db.get_database_backend() {
        DatabaseBackend::Sqlite => get_sqlite_relations(&db).await?,
        DatabaseBackend::Postgres => get_postgres_relations(&db).await?,
        DatabaseBackend::MySql => get_mysql_relations(&db).await?,
    };

    // Build summary
    let mut tables_with_fks: Vec<String> = relations.iter().map(|r| r.from_table.clone()).collect();
    tables_with_fks.sort();
    tables_with_fks.dedup();

    let mut referenced_tables: Vec<String> = relations.iter().map(|r| r.to_table.clone()).collect();
    referenced_tables.sort();
    referenced_tables.dedup();

    let summary = RelationSummary {
        total_relations: relations.len(),
        tables_with_fks,
        referenced_tables,
    };

    Ok(RelationMapInfo { relations, summary })
}

async fn get_sqlite_relations(db: &DatabaseConnection) -> Result<Vec<Relation>> {
    let mut relations = Vec::new();

    // Get all tables first
    let table_query =
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'";
    let table_rows = db
        .query_all(Statement::from_string(
            DatabaseBackend::Sqlite,
            table_query.to_string(),
        ))
        .await
        .map_err(|e| McpError::DatabaseError(format!("Failed to get tables: {e}")))?;

    for row in table_rows {
        let table_name: String = row
            .try_get_by("name")
            .map_err(|e| McpError::DatabaseError(format!("Failed to get table name: {e}")))?;

        // Get foreign keys for this table
        let fk_query = format!("PRAGMA foreign_key_list('{table_name}')");
        let fk_rows = db
            .query_all(Statement::from_string(DatabaseBackend::Sqlite, fk_query))
            .await
            .unwrap_or_default();

        for fk in fk_rows {
            let to_table: String = fk.try_get_by("table").unwrap_or_default();
            let from_column: String = fk.try_get_by("from").unwrap_or_default();
            let to_column: String = fk.try_get_by("to").unwrap_or_default();

            if !to_table.is_empty() && !from_column.is_empty() {
                let relation_type = infer_relation_type(&from_column);
                relations.push(Relation {
                    from_table: table_name.clone(),
                    from_column,
                    to_table,
                    to_column: if to_column.is_empty() {
                        "id".to_string()
                    } else {
                        to_column
                    },
                    relation_type,
                    constraint_name: None,
                });
            }
        }

        // Also infer relations from column naming conventions (_id suffix)
        let column_query = format!("PRAGMA table_info('{table_name}')");
        let column_rows = db
            .query_all(Statement::from_string(
                DatabaseBackend::Sqlite,
                column_query,
            ))
            .await
            .unwrap_or_default();

        for col in column_rows {
            let col_name: String = col.try_get_by("name").unwrap_or_default();

            // Check for _id suffix pattern (e.g., user_id -> users)
            if col_name.ends_with("_id") && col_name != "id" {
                let potential_table = format!("{}s", col_name.trim_end_matches("_id"));

                // Check if this relation already exists from FK constraints
                let already_exists = relations
                    .iter()
                    .any(|r| r.from_table == table_name && r.from_column == col_name);

                if !already_exists {
                    // Check if the inferred table actually exists
                    let check_query = format!(
                        "SELECT name FROM sqlite_master WHERE type='table' AND name='{potential_table}'"
                    );
                    if let Ok(rows) = db
                        .query_all(Statement::from_string(DatabaseBackend::Sqlite, check_query))
                        .await
                    {
                        if !rows.is_empty() {
                            relations.push(Relation {
                                from_table: table_name.clone(),
                                from_column: col_name.clone(),
                                to_table: potential_table,
                                to_column: "id".to_string(),
                                relation_type: "inferred_belongs_to".to_string(),
                                constraint_name: None,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(relations)
}

async fn get_postgres_relations(db: &DatabaseConnection) -> Result<Vec<Relation>> {
    let query = r#"
        SELECT
            tc.table_name AS from_table,
            kcu.column_name AS from_column,
            ccu.table_name AS to_table,
            ccu.column_name AS to_column,
            tc.constraint_name
        FROM information_schema.table_constraints AS tc
        JOIN information_schema.key_column_usage AS kcu
            ON tc.constraint_name = kcu.constraint_name
            AND tc.table_schema = kcu.table_schema
        JOIN information_schema.constraint_column_usage AS ccu
            ON ccu.constraint_name = tc.constraint_name
            AND ccu.table_schema = tc.table_schema
        WHERE tc.constraint_type = 'FOREIGN KEY'
            AND tc.table_schema = 'public'
        ORDER BY tc.table_name, kcu.column_name
    "#;

    let rows = db
        .query_all(Statement::from_string(
            DatabaseBackend::Postgres,
            query.to_string(),
        ))
        .await
        .map_err(|e| McpError::DatabaseError(format!("Failed to get relations: {e}")))?;

    let relations = rows
        .iter()
        .filter_map(|row| {
            let from_table: String = row.try_get_by("from_table").ok()?;
            let from_column: String = row.try_get_by("from_column").ok()?;
            let to_table: String = row.try_get_by("to_table").ok()?;
            let to_column: String = row.try_get_by("to_column").ok()?;
            let constraint_name: Option<String> = row.try_get_by("constraint_name").ok();

            Some(Relation {
                from_table,
                from_column: from_column.clone(),
                to_table,
                to_column,
                relation_type: infer_relation_type(&from_column),
                constraint_name,
            })
        })
        .collect();

    Ok(relations)
}

async fn get_mysql_relations(db: &DatabaseConnection) -> Result<Vec<Relation>> {
    // Get database name
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

    let query = format!(
        r#"
        SELECT
            TABLE_NAME AS from_table,
            COLUMN_NAME AS from_column,
            REFERENCED_TABLE_NAME AS to_table,
            REFERENCED_COLUMN_NAME AS to_column,
            CONSTRAINT_NAME AS constraint_name
        FROM information_schema.KEY_COLUMN_USAGE
        WHERE TABLE_SCHEMA = '{db_name}'
            AND REFERENCED_TABLE_NAME IS NOT NULL
        ORDER BY TABLE_NAME, COLUMN_NAME
        "#
    );

    let rows = db
        .query_all(Statement::from_string(DatabaseBackend::MySql, query))
        .await
        .map_err(|e| McpError::DatabaseError(format!("Failed to get relations: {e}")))?;

    let relations = rows
        .iter()
        .filter_map(|row| {
            let from_table: String = row
                .try_get_by("from_table")
                .or_else(|_| row.try_get_by("FROM_TABLE"))
                .ok()?;
            let from_column: String = row
                .try_get_by("from_column")
                .or_else(|_| row.try_get_by("FROM_COLUMN"))
                .ok()?;
            let to_table: String = row
                .try_get_by("to_table")
                .or_else(|_| row.try_get_by("TO_TABLE"))
                .ok()?;
            let to_column: String = row
                .try_get_by("to_column")
                .or_else(|_| row.try_get_by("TO_COLUMN"))
                .ok()?;
            let constraint_name: Option<String> = row
                .try_get_by("constraint_name")
                .or_else(|_| row.try_get_by("CONSTRAINT_NAME"))
                .ok();

            Some(Relation {
                from_table,
                from_column: from_column.clone(),
                to_table,
                to_column,
                relation_type: infer_relation_type(&from_column),
                constraint_name,
            })
        })
        .collect();

    Ok(relations)
}

fn infer_relation_type(column_name: &str) -> String {
    if column_name.ends_with("_id") {
        "belongs_to".to_string()
    } else if column_name == "id" {
        "has_many".to_string()
    } else {
        "unknown".to_string()
    }
}

fn get_database_url(project_root: &Path) -> Result<String> {
    dotenvy::from_path(project_root.join(".env")).ok();

    std::env::var("DATABASE_URL")
        .map_err(|_| McpError::ConfigError("DATABASE_URL not set in .env".to_string()))
}
