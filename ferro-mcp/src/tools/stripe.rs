//! MCP introspection tools for Stripe integration.
//!
//! Provides three tools:
//! - `stripe_config_status` — reports env var presence and scaffold existence
//! - `stripe_webhook_events` — lists listener implementations discovered from source
//! - `stripe_subscription_info` — reports tenant_billing table schema from migrations

use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// stripe_config_status
// ---------------------------------------------------------------------------

/// Status of Stripe configuration in the current project.
#[derive(Debug, Serialize)]
pub struct StripeConfigStatus {
    /// True when all required keys are present.
    pub configured: bool,
    /// Names of env vars that are set (values masked).
    pub keys_present: Vec<String>,
    /// Names of env vars that are missing.
    pub keys_missing: Vec<String>,
    /// True when src/stripe/ directory exists.
    pub scaffold_exists: bool,
    /// List of scaffold files found in src/stripe/.
    pub scaffold_files: Vec<String>,
}

/// Report Stripe configuration status for the project.
///
/// Reads env vars from the environment (and .env if present) and checks
/// whether the scaffold directory exists.
pub fn stripe_config_status(project_root: &Path) -> StripeConfigStatus {
    // Load .env if present so values are available without a running server
    let env_path = project_root.join(".env");
    if env_path.exists() {
        let _ = dotenvy::from_path(&env_path);
    }

    let required_keys = [
        "STRIPE_SECRET_KEY",
        "STRIPE_WEBHOOK_SECRET",
        "STRIPE_PUBLISHABLE_KEY",
    ];
    let optional_keys = [
        "STRIPE_CONNECT_WEBHOOK_SECRET",
        "STRIPE_APPLICATION_FEE_PERCENT",
    ];

    let mut keys_present = Vec::new();
    let mut keys_missing = Vec::new();

    for key in &required_keys {
        if std::env::var(key).is_ok() {
            keys_present.push(key.to_string());
        } else {
            keys_missing.push(key.to_string());
        }
    }

    for key in &optional_keys {
        if std::env::var(key).is_ok() {
            keys_present.push(key.to_string());
        }
        // Optional keys are not added to missing
    }

    let scaffold_dir = project_root.join("src/stripe");
    let scaffold_exists = scaffold_dir.is_dir();

    let scaffold_files = if scaffold_exists {
        match fs::read_dir(&scaffold_dir) {
            Ok(entries) => {
                let mut files: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
                    .map(|e| format!("src/stripe/{}", e.file_name().to_string_lossy()))
                    .collect();
                files.sort();
                files
            }
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };

    let configured = keys_missing.is_empty();

    StripeConfigStatus {
        configured,
        keys_present,
        keys_missing,
        scaffold_exists,
        scaffold_files,
    }
}

// ---------------------------------------------------------------------------
// stripe_webhook_events
// ---------------------------------------------------------------------------

/// A discovered event listener in the Stripe listeners file.
#[derive(Debug, Serialize)]
pub struct WebhookEventInfo {
    /// The Ferro event type (e.g., "StripeSubscriptionUpdated").
    pub event_type: String,
    /// The listener struct name (e.g., "SyncSubscriptionPlan").
    pub listener: String,
    /// Relative file path where the listener is defined.
    pub file: String,
}

/// List of discovered Stripe webhook event listeners.
#[derive(Debug, Serialize)]
pub struct StripeWebhookEvents {
    pub events: Vec<WebhookEventInfo>,
}

/// Scan src/stripe/listeners.rs for Listener impl blocks.
pub fn stripe_webhook_events(project_root: &Path) -> StripeWebhookEvents {
    let listeners_path = project_root.join("src/stripe/listeners.rs");

    if !listeners_path.exists() {
        return StripeWebhookEvents { events: Vec::new() };
    }

    let content = match fs::read_to_string(&listeners_path) {
        Ok(c) => c,
        Err(_) => return StripeWebhookEvents { events: Vec::new() },
    };

    // Match: impl Listener<EventType> for StructName
    let re = Regex::new(r"impl\s+Listener<(\w+)>\s+for\s+(\w+)").unwrap();

    let events: Vec<WebhookEventInfo> = re
        .captures_iter(&content)
        .map(|cap| WebhookEventInfo {
            event_type: cap[1].to_string(),
            listener: cap[2].to_string(),
            file: "src/stripe/listeners.rs".to_string(),
        })
        .collect();

    StripeWebhookEvents { events }
}

// ---------------------------------------------------------------------------
// stripe_subscription_info
// ---------------------------------------------------------------------------

/// Column description extracted from SQL migration.
#[derive(Debug, Serialize)]
pub struct ColumnInfo {
    pub name: String,
    pub sql_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
}

/// Index description extracted from SQL migration.
#[derive(Debug, Serialize)]
pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
}

/// Schema information for the tenant_billing table.
#[derive(Debug, Serialize)]
pub struct StripeSubscriptionInfo {
    /// True when a migration creating tenant_billing was found.
    pub table_exists: bool,
    /// Relative path to the migration file (if found).
    pub migration_file: Option<String>,
    /// Column definitions parsed from the CREATE TABLE statement.
    pub columns: Vec<ColumnInfo>,
    /// Index definitions parsed from the migration.
    pub indexes: Vec<IndexInfo>,
}

/// Read the tenant_billing schema from migration files.
pub fn stripe_subscription_info(project_root: &Path) -> StripeSubscriptionInfo {
    let migration_dirs = [
        project_root.join("src/migrations"),
        project_root.join("src/database/migrations"),
    ];

    for dir in &migration_dirs {
        if !dir.is_dir() {
            continue;
        }

        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "rs") {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.contains("tenant_billing") {
                continue;
            }

            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let relative = path
                .strip_prefix(project_root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            let (columns, indexes) = parse_create_table(&content);

            return StripeSubscriptionInfo {
                table_exists: true,
                migration_file: Some(relative),
                columns,
                indexes,
            };
        }
    }

    StripeSubscriptionInfo {
        table_exists: false,
        migration_file: None,
        columns: Vec::new(),
        indexes: Vec::new(),
    }
}

/// Parse columns and indexes from a CREATE TABLE SQL block embedded in Rust source.
fn parse_create_table(source: &str) -> (Vec<ColumnInfo>, Vec<IndexInfo>) {
    // Extract the SQL string from execute_unprepared(...)
    let sql_re = Regex::new(r#"execute_unprepared\s*\(\s*"([^"]+)""#).unwrap();
    let sql = match sql_re.captures(source) {
        Some(cap) => cap[1].replace("\\n", "\n").replace("\\\"", "\""),
        None => return (Vec::new(), Vec::new()),
    };

    let mut columns = Vec::new();
    let mut indexes = Vec::new();

    // Parse column definitions from CREATE TABLE block
    let table_re = Regex::new(r"CREATE TABLE\s+\w+\s*\(([^;]+)\)").unwrap();
    if let Some(cap) = table_re.captures(&sql) {
        let cols_block = &cap[1];
        for line in cols_block.lines() {
            let line = line.trim().trim_end_matches(',').trim();
            if line.is_empty()
                || line.starts_with("PRIMARY KEY")
                || line.starts_with("UNIQUE")
                || line.starts_with("FOREIGN KEY")
                || line.starts_with("CHECK")
            {
                continue;
            }

            if let Some(col) = parse_column_line(line) {
                columns.push(col);
            }
        }
    }

    // Parse CREATE INDEX statements
    let index_re =
        Regex::new(r"CREATE(?:\s+UNIQUE)?\s+INDEX\s+(\w+)\s+ON\s+\w+\s*\(([^)]+)\)").unwrap();
    for cap in index_re.captures_iter(&sql) {
        let name = cap[1].to_string();
        let cols: Vec<String> = cap[2].split(',').map(|c| c.trim().to_string()).collect();
        indexes.push(IndexInfo {
            name,
            columns: cols,
        });
    }

    (columns, indexes)
}

/// Parse a single column definition line into a ColumnInfo.
fn parse_column_line(line: &str) -> Option<ColumnInfo> {
    let parts: Vec<&str> = line.splitn(3, char::is_whitespace).collect();
    if parts.len() < 2 {
        return None;
    }

    let name = parts[0].to_string();
    let sql_type = parts[1].to_string();

    let nullable = !line.to_uppercase().contains("NOT NULL");
    let default_value = extract_default(line);

    Some(ColumnInfo {
        name,
        sql_type,
        nullable,
        default_value,
    })
}

/// Extract the DEFAULT value from a column definition.
fn extract_default(line: &str) -> Option<String> {
    let upper = line.to_uppercase();
    let idx = upper.find("DEFAULT ")?;
    let after = &line[idx + 8..];
    // Take up to next keyword or end
    let end = after.find([',', ')', '\n']).unwrap_or(after.len());
    Some(after[..end].trim().to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // --- stripe_config_status tests ---

    #[test]
    fn test_config_status_scaffold_exists() {
        let tmp = TempDir::new().unwrap();
        let stripe_dir = tmp.path().join("src/stripe");
        fs::create_dir_all(&stripe_dir).unwrap();
        fs::write(stripe_dir.join("mod.rs"), "// mod").unwrap();
        fs::write(stripe_dir.join("webhook.rs"), "// webhook").unwrap();

        let status = stripe_config_status(tmp.path());

        assert!(status.scaffold_exists);
        assert_eq!(status.scaffold_files.len(), 2);
        assert!(status.scaffold_files.iter().any(|f| f.ends_with("mod.rs")));
        assert!(status
            .scaffold_files
            .iter()
            .any(|f| f.ends_with("webhook.rs")));
    }

    #[test]
    fn test_config_status_scaffold_missing() {
        let tmp = TempDir::new().unwrap();
        let status = stripe_config_status(tmp.path());

        assert!(!status.scaffold_exists);
        assert!(status.scaffold_files.is_empty());
    }

    #[test]
    fn test_config_status_serializes() {
        let status = StripeConfigStatus {
            configured: false,
            keys_present: vec!["STRIPE_SECRET_KEY".to_string()],
            keys_missing: vec!["STRIPE_WEBHOOK_SECRET".to_string()],
            scaffold_exists: false,
            scaffold_files: Vec::new(),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("STRIPE_SECRET_KEY"));
        assert!(json.contains("STRIPE_WEBHOOK_SECRET"));
        assert!(json.contains("\"configured\":false"));
    }

    #[test]
    fn test_config_status_keys_missing_when_not_configured() {
        let tmp = TempDir::new().unwrap();
        // Env vars almost certainly won't be set in a temp dir context
        let status = stripe_config_status(tmp.path());

        // keys_missing should include the required keys that are absent
        // We can't guarantee env state, but we can check the structure
        let all_keys: Vec<&str> = status
            .keys_present
            .iter()
            .chain(status.keys_missing.iter())
            .map(|s| s.as_str())
            .collect();

        assert!(all_keys.contains(&"STRIPE_SECRET_KEY"));
        assert!(all_keys.contains(&"STRIPE_WEBHOOK_SECRET"));
    }

    // --- stripe_webhook_events tests ---

    #[test]
    fn test_webhook_events_not_found_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let result = stripe_webhook_events(tmp.path());
        assert!(result.events.is_empty());
    }

    #[test]
    fn test_webhook_events_parses_listeners() {
        let tmp = TempDir::new().unwrap();
        let stripe_dir = tmp.path().join("src/stripe");
        fs::create_dir_all(&stripe_dir).unwrap();

        let content = r#"
use ferro::{async_trait, EventError, Listener};
use ferro::{StripeSubscriptionUpdated, StripeSubscriptionDeleted};

pub struct SyncSubscriptionPlan;

#[async_trait]
impl Listener<StripeSubscriptionUpdated> for SyncSubscriptionPlan {
    async fn handle(&self, event: &StripeSubscriptionUpdated) -> Result<(), EventError> {
        Ok(())
    }
}

pub struct HandleSubscriptionDeleted;

#[async_trait]
impl Listener<StripeSubscriptionDeleted> for HandleSubscriptionDeleted {
    async fn handle(&self, event: &StripeSubscriptionDeleted) -> Result<(), EventError> {
        Ok(())
    }
}
"#;
        fs::write(stripe_dir.join("listeners.rs"), content).unwrap();

        let result = stripe_webhook_events(tmp.path());
        assert_eq!(result.events.len(), 2);

        let event_types: Vec<&str> = result
            .events
            .iter()
            .map(|e| e.event_type.as_str())
            .collect();
        assert!(event_types.contains(&"StripeSubscriptionUpdated"));
        assert!(event_types.contains(&"StripeSubscriptionDeleted"));

        let listeners: Vec<&str> = result.events.iter().map(|e| e.listener.as_str()).collect();
        assert!(listeners.contains(&"SyncSubscriptionPlan"));
        assert!(listeners.contains(&"HandleSubscriptionDeleted"));

        for event in &result.events {
            assert_eq!(event.file, "src/stripe/listeners.rs");
        }
    }

    #[test]
    fn test_webhook_events_serializes() {
        let info = WebhookEventInfo {
            event_type: "StripeSubscriptionUpdated".to_string(),
            listener: "SyncSubscriptionPlan".to_string(),
            file: "src/stripe/listeners.rs".to_string(),
        };
        let events = StripeWebhookEvents { events: vec![info] };
        let json = serde_json::to_string(&events).unwrap();
        assert!(json.contains("StripeSubscriptionUpdated"));
        assert!(json.contains("SyncSubscriptionPlan"));
    }

    // --- stripe_subscription_info tests ---

    #[test]
    fn test_subscription_info_no_migration() {
        let tmp = TempDir::new().unwrap();
        let result = stripe_subscription_info(tmp.path());
        assert!(!result.table_exists);
        assert!(result.migration_file.is_none());
        assert!(result.columns.is_empty());
        assert!(result.indexes.is_empty());
    }

    #[test]
    fn test_subscription_info_parses_migration() {
        let tmp = TempDir::new().unwrap();
        let migrations_dir = tmp.path().join("src/migrations");
        fs::create_dir_all(&migrations_dir).unwrap();

        let migration_content = r#"
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260101_000000_create_tenant_billing_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE tenant_billing (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    tenant_id INTEGER NOT NULL UNIQUE,
                    stripe_customer_id TEXT NOT NULL,
                    stripe_subscription_id TEXT,
                    plan TEXT NOT NULL DEFAULT 'free',
                    subscription_status TEXT NOT NULL DEFAULT 'active',
                    cancel_at_period_end BOOLEAN NOT NULL DEFAULT 0,
                    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
                );
                CREATE INDEX idx_tenant_billing_tenant_id ON tenant_billing(tenant_id);",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS tenant_billing;")
            .await?;
        Ok(())
    }
}
"#;
        fs::write(
            migrations_dir.join("m20260101_000000_create_tenant_billing_table.rs"),
            migration_content,
        )
        .unwrap();

        let result = stripe_subscription_info(tmp.path());
        assert!(result.table_exists, "table_exists should be true");
        assert!(result.migration_file.is_some());
        assert!(result
            .migration_file
            .as_deref()
            .unwrap()
            .contains("tenant_billing"));

        // Should have parsed columns
        assert!(
            !result.columns.is_empty(),
            "Should have parsed column definitions"
        );

        let col_names: Vec<&str> = result.columns.iter().map(|c| c.name.as_str()).collect();
        assert!(col_names.contains(&"tenant_id"), "Should have tenant_id");
        assert!(
            col_names.contains(&"stripe_customer_id"),
            "Should have stripe_customer_id"
        );
        assert!(col_names.contains(&"plan"), "Should have plan");

        // Should have parsed indexes
        assert!(!result.indexes.is_empty(), "Should have parsed indexes");
        assert_eq!(
            result.indexes[0].name, "idx_tenant_billing_tenant_id",
            "Index name should match"
        );
        assert!(result.indexes[0].columns.contains(&"tenant_id".to_string()));
    }

    #[test]
    fn test_subscription_info_serializes() {
        let info = StripeSubscriptionInfo {
            table_exists: true,
            migration_file: Some("src/migrations/m20260101_tenant_billing.rs".to_string()),
            columns: vec![ColumnInfo {
                name: "tenant_id".to_string(),
                sql_type: "INTEGER".to_string(),
                nullable: false,
                default_value: None,
            }],
            indexes: vec![IndexInfo {
                name: "idx_tenant_billing_tenant_id".to_string(),
                columns: vec!["tenant_id".to_string()],
            }],
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("table_exists"));
        assert!(json.contains("tenant_id"));
        assert!(json.contains("idx_tenant_billing_tenant_id"));
    }
}
