//! List migrations tool - show migration status

use crate::error::{McpError, Result};
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement};
use serde::Serialize;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct MigrationsInfo {
    pub migrations: Vec<MigrationInfo>,
}

#[derive(Debug, Serialize)]
pub struct MigrationInfo {
    pub name: String,
    pub status: String,
    pub applied_at: Option<String>,
}

pub async fn execute(project_root: &Path) -> Result<MigrationsInfo> {
    // Find all migration files
    let migrations_dir = project_root.join("src/migrations");

    if !migrations_dir.exists() {
        return Err(McpError::FileNotFound("src/migrations".to_string()));
    }

    let mut defined_migrations = scan_migration_files(&migrations_dir);
    defined_migrations.sort();

    // Try to get applied migrations from database
    let applied_migrations = get_applied_migrations(project_root).await;

    // Build migration info
    let migrations: Vec<MigrationInfo> = defined_migrations
        .iter()
        .map(|name| {
            let applied = applied_migrations.iter().find(|(n, _)| n == name);
            MigrationInfo {
                name: name.clone(),
                status: if applied.is_some() {
                    "applied".to_string()
                } else {
                    "pending".to_string()
                },
                applied_at: applied.and_then(|(_, at)| at.clone()),
            }
        })
        .collect();

    Ok(MigrationsInfo { migrations })
}

fn scan_migration_files(migrations_dir: &Path) -> Vec<String> {
    let mut migrations = Vec::new();

    for entry in WalkDir::new(migrations_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .file_name()
                .map(|f| f.to_string_lossy().starts_with('m'))
                .unwrap_or(false)
                && e.path().extension().map(|ext| ext == "rs").unwrap_or(false)
        })
    {
        if let Some(name) = entry.path().file_stem() {
            let name_str = name.to_string_lossy().to_string();
            if name_str != "mod" {
                migrations.push(name_str);
            }
        }
    }

    // Also check mod.rs for migration registration
    let mod_file = migrations_dir.join("mod.rs");
    if mod_file.exists() {
        if let Ok(content) = fs::read_to_string(&mod_file) {
            // Look for migration module declarations
            for line in content.lines() {
                if line.trim().starts_with("mod m") {
                    if let Some(name) = line
                        .trim()
                        .trim_start_matches("mod ")
                        .trim_end_matches(';')
                        .split_whitespace()
                        .next()
                    {
                        if !migrations.contains(&name.to_string()) {
                            migrations.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    migrations
}

async fn get_applied_migrations(project_root: &Path) -> Vec<(String, Option<String>)> {
    let database_url = match get_database_url(project_root) {
        Ok(url) => url,
        Err(_) => return Vec::new(),
    };

    let db: DatabaseConnection = match Database::connect(&database_url).await {
        Ok(conn) => conn,
        Err(_) => return Vec::new(),
    };

    // Query seaql_migrations table
    let query = match db.get_database_backend() {
        DatabaseBackend::Sqlite => {
            "SELECT version, applied_at FROM seaql_migrations ORDER BY version"
        }
        DatabaseBackend::Postgres | DatabaseBackend::MySql => {
            "SELECT version, applied_at FROM seaql_migrations ORDER BY version"
        }
    };

    let result = db
        .query_all(Statement::from_string(
            db.get_database_backend(),
            query.to_string(),
        ))
        .await;

    match result {
        Ok(rows) => rows
            .iter()
            .filter_map(|row| {
                let version: String = row.try_get_by("version").ok()?;
                let applied_at: Option<String> = row.try_get_by("applied_at").ok();
                Some((version, applied_at))
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

fn get_database_url(project_root: &Path) -> Result<String> {
    dotenvy::from_path(project_root.join(".env")).ok();

    std::env::var("DATABASE_URL")
        .map_err(|_| McpError::ConfigError("DATABASE_URL not set in .env".to_string()))
}
