use chrono::Local;
use console::style;
use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// Template generators
// ---------------------------------------------------------------------------

fn stripe_mod_template(connect: bool) -> String {
    let connect_mod = if connect {
        "\npub mod connect_webhook;"
    } else {
        ""
    };

    format!(
        r#"pub mod webhook;
pub mod listeners;{connect_mod}

use ferro::Stripe;

/// Initialize Stripe. Call from bootstrap.rs.
pub fn init() {{
    let config = ferro::StripeConfig::from_env()
        .expect("Stripe configuration missing. Set STRIPE_SECRET_KEY and STRIPE_WEBHOOK_SECRET.");
    Stripe::init(config);
}}
"#
    )
}

fn stripe_webhook_template() -> String {
    r#"use ferro::{handler, HttpResponse, Request, Response, Stripe};
use ferro::ProcessStripeWebhook;

#[handler]
pub async fn stripe_webhook(req: Request) -> Response {
    let sig = req
        .header("stripe-signature")
        .ok_or_else(|| HttpResponse::text("Missing stripe-signature").status(400))?;
    let body = req
        .body_string()
        .await
        .map_err(|_| HttpResponse::text("Failed to read body").status(400))?;

    // Verify signature inline per locked decision.
    ferro::verify_webhook(&body, &sig, &Stripe::config().webhook_secret)
        .map_err(|_| HttpResponse::text("Invalid signature").status(400))?;

    // Dispatch to ferro-queue for async processing per locked decision.
    ferro::dispatch_job(ProcessStripeWebhook::platform(&body))
        .map_err(|e| HttpResponse::text(format!("Queue error: {e}")).status(500))?;

    // Ack 200 immediately.
    Ok(HttpResponse::json(serde_json::json!({"received": true})))
}
"#
    .to_string()
}

fn stripe_connect_webhook_template() -> String {
    r#"use ferro::{handler, HttpResponse, Request, Response, Stripe};
use ferro::ProcessStripeWebhook;

#[handler]
pub async fn stripe_connect_webhook(req: Request) -> Response {
    let sig = req
        .header("stripe-signature")
        .ok_or_else(|| HttpResponse::text("Missing stripe-signature").status(400))?;
    let body = req
        .body_string()
        .await
        .map_err(|_| HttpResponse::text("Failed to read body").status(400))?;

    // Verify signature using the Connect webhook secret.
    ferro::verify_webhook(&body, &sig, &Stripe::config().connect_webhook_secret
        .as_deref()
        .unwrap_or_default())
        .map_err(|_| HttpResponse::text("Invalid signature").status(400))?;

    // Dispatch to ferro-queue for async processing.
    ferro::dispatch_job(ProcessStripeWebhook::connect(&body))
        .map_err(|e| HttpResponse::text(format!("Queue error: {e}")).status(500))?;

    Ok(HttpResponse::json(serde_json::json!({"received": true})))
}
"#
    .to_string()
}

fn stripe_listeners_template() -> String {
    r#"use ferro::{async_trait, EventError, Listener};
use ferro::{StripeCheckoutCompleted, StripeSubscriptionDeleted, StripeSubscriptionUpdated};

pub struct SyncSubscriptionPlan;

#[async_trait]
impl Listener<StripeSubscriptionUpdated> for SyncSubscriptionPlan {
    async fn handle(&self, event: &StripeSubscriptionUpdated) -> Result<(), EventError> {
        // TODO: Update tenant_billing table with new subscription state.
        // TODO: Invalidate tenant cache: lookup.invalidate(&slug, tenant_id)
        println!("Subscription updated: {}", event.subscription_id);
        Ok(())
    }
}

pub struct HandleSubscriptionDeleted;

#[async_trait]
impl Listener<StripeSubscriptionDeleted> for HandleSubscriptionDeleted {
    async fn handle(&self, event: &StripeSubscriptionDeleted) -> Result<(), EventError> {
        // TODO: Mark tenant_billing as cancelled.
        println!("Subscription deleted: {}", event.subscription_id);
        Ok(())
    }
}

pub struct HandleCheckoutCompleted;

#[async_trait]
impl Listener<StripeCheckoutCompleted> for HandleCheckoutCompleted {
    async fn handle(&self, event: &StripeCheckoutCompleted) -> Result<(), EventError> {
        // TODO: Provision access for the new subscriber.
        println!("Checkout completed: {}", event.session_id);
        Ok(())
    }
}
"#
    .to_string()
}

fn stripe_migration_template(timestamp: &str) -> String {
    format!(
        r#"use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {{
    fn name(&self) -> &str {{
        "m{timestamp}_create_tenant_billing_table"
    }}
}}

#[async_trait::async_trait]
impl MigrationTrait for Migration {{
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
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
                    trial_ends_at TIMESTAMP,
                    current_period_end TIMESTAMP,
                    cancel_at_period_end BOOLEAN NOT NULL DEFAULT 0,
                    stripe_connect_account_id TEXT,
                    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
                );
                CREATE INDEX idx_tenant_billing_tenant_id ON tenant_billing(tenant_id);",
            )
            .await?;
        Ok(())
    }}

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_tenant_billing_tenant_id;
                DROP TABLE IF EXISTS tenant_billing;",
            )
            .await?;
        Ok(())
    }}
}}
"#
    )
}

// ---------------------------------------------------------------------------
// File generation helpers
// ---------------------------------------------------------------------------

/// Write a file only if it does not already exist.
/// Returns true if the file was created, false if skipped.
fn write_if_not_exists(path: &Path, content: &str, label: &str) -> bool {
    if path.exists() {
        println!(
            "{} {} already exists, skipping",
            style("Skip:").yellow().bold(),
            label
        );
        return false;
    }
    if let Err(e) = fs::write(path, content) {
        eprintln!(
            "{} Failed to write {}: {}",
            style("Error:").red().bold(),
            label,
            e
        );
        return false;
    }
    println!("{} {}", style("Created:").green().bold(), label);
    true
}

fn ensure_dir(path: &Path) -> bool {
    if path.exists() {
        return true;
    }
    if let Err(e) = fs::create_dir_all(path) {
        eprintln!(
            "{} Failed to create directory {}: {}",
            style("Error:").red().bold(),
            path.display(),
            e
        );
        return false;
    }
    println!(
        "{} Created directory {}",
        style("Created:").green().bold(),
        path.display()
    );
    true
}

fn find_migrations_dir() -> Option<&'static Path> {
    if Path::new("src/migrations").exists() {
        Some(Path::new("src/migrations"))
    } else if Path::new("src/database/migrations").exists() {
        Some(Path::new("src/database/migrations"))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Execute `ferro make:stripe [--connect]`.
///
/// Generates the Stripe integration scaffold into the current Ferro project.
pub fn execute(connect: bool) {
    println!("Scaffolding Stripe integration...\n");

    let stripe_dir = Path::new("src/stripe");

    if !ensure_dir(stripe_dir) {
        std::process::exit(1);
    }

    // mod.rs
    write_if_not_exists(
        &stripe_dir.join("mod.rs"),
        &stripe_mod_template(connect),
        "src/stripe/mod.rs",
    );

    // webhook.rs
    write_if_not_exists(
        &stripe_dir.join("webhook.rs"),
        &stripe_webhook_template(),
        "src/stripe/webhook.rs",
    );

    // listeners.rs
    write_if_not_exists(
        &stripe_dir.join("listeners.rs"),
        &stripe_listeners_template(),
        "src/stripe/listeners.rs",
    );

    // connect_webhook.rs (only with --connect)
    if connect {
        write_if_not_exists(
            &stripe_dir.join("connect_webhook.rs"),
            &stripe_connect_webhook_template(),
            "src/stripe/connect_webhook.rs",
        );
    }

    // Migration
    generate_migration(connect);

    // Print env hints
    println!("\n{}", style("Add to your .env file:").bold());
    println!("  STRIPE_SECRET_KEY=sk_test_xxx");
    println!("  STRIPE_WEBHOOK_SECRET=whsec_xxx");
    if connect {
        println!("  STRIPE_CONNECT_WEBHOOK_SECRET=whsec_xxx");
        println!("  STRIPE_APPLICATION_FEE_PERCENT=10");
    }

    // Print next steps
    print_next_steps(connect);
}

fn generate_migration(connect: bool) {
    let migrations_dir = match find_migrations_dir() {
        Some(dir) => dir,
        None => {
            println!(
                "{} No migrations directory found — skipping migration generation.",
                style("Note:").yellow().bold()
            );
            println!(
                "{}",
                style("  Create src/migrations/ and re-run make:stripe to generate the migration.")
                    .dim()
            );
            return;
        }
    };

    // Check if billing migration already exists
    if let Ok(entries) = fs::read_dir(migrations_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains("tenant_billing") {
                println!(
                    "{} Billing migration already exists: {}",
                    style("Skip:").yellow().bold(),
                    name
                );
                return;
            }
        }
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let migration_name = format!("m{timestamp}_create_tenant_billing_table");
    let file_path = migrations_dir.join(format!("{migration_name}.rs"));
    let content = stripe_migration_template(&timestamp);

    write_if_not_exists(&file_path, &content, &format!("{}", file_path.display()));

    // Register in mod.rs
    register_migration(migrations_dir, &migration_name, connect);
}

fn register_migration(migrations_dir: &Path, migration_name: &str, _connect: bool) {
    let mod_path = migrations_dir.join("mod.rs");

    if !mod_path.exists() {
        return;
    }

    let content = match fs::read_to_string(&mod_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mod_decl = format!("mod {migration_name};");
    if content.contains(&mod_decl) {
        return;
    }

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Find last mod declaration
    let mut last_mod_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if (line.trim().starts_with("mod ") || line.trim().starts_with("pub mod m"))
            && !line.contains("mod tests")
        {
            last_mod_idx = Some(i);
        }
    }

    let insert_idx = match last_mod_idx {
        Some(idx) => idx + 1,
        None => {
            let mut idx = 0;
            for (i, line) in lines.iter().enumerate() {
                if line.contains("sea_orm_migration") || line.is_empty() {
                    idx = i + 1;
                } else if line.starts_with("mod ") || line.starts_with("pub struct") {
                    break;
                }
            }
            idx
        }
    };
    lines.insert(insert_idx, mod_decl);

    // Add to migrations() vec
    let box_new_line = format!("            Box::new({migration_name}::Migration),");
    let mut insert_vec_idx = None;

    for (i, line) in lines.iter().enumerate() {
        if line.contains("vec![]") {
            lines[i] = line.replace("vec![]", &format!("vec![\n{box_new_line}\n        ]"));
            let _ = fs::write(&mod_path, lines.join("\n"));
            return;
        }
        if line.contains("vec![") && !line.contains("vec![]") {
            for (j, inner_line) in lines.iter().enumerate().skip(i + 1) {
                if inner_line.trim() == "]" || inner_line.trim().starts_with(']') {
                    insert_vec_idx = Some(j);
                    break;
                }
            }
            break;
        }
    }

    if let Some(idx) = insert_vec_idx {
        lines.insert(idx, box_new_line);
    }

    let _ = fs::write(&mod_path, lines.join("\n"));
}

fn print_next_steps(connect: bool) {
    println!("\n{}", style("Next steps:").bold());
    println!(
        "\n  {} Call Stripe::init() from your bootstrap.rs:",
        style("1.").dim()
    );
    println!("     {}", style("crate::stripe::init();").cyan());

    println!(
        "\n  {} Register webhook routes in src/routes.rs:",
        style("2.").dim()
    );
    println!(
        "     {}",
        style("use crate::stripe::webhook::stripe_webhook;").cyan()
    );
    println!(
        "     {}",
        style("post!(\"/stripe/webhook\", stripe_webhook)").cyan()
    );
    if connect {
        println!(
            "     {}",
            style("use crate::stripe::connect_webhook::stripe_connect_webhook;").cyan()
        );
        println!(
            "     {}",
            style("post!(\"/stripe/connect/webhook\", stripe_connect_webhook)").cyan()
        );
    }

    println!("\n  {} Run the migration:", style("3.").dim());
    println!("     {}", style("ferro db:migrate").cyan());
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Generate the scaffold files in a temp directory for testing.
#[cfg(test)]
pub fn generate_in_dir(base_dir: &Path, connect: bool) {
    let stripe_dir = base_dir.join("src/stripe");
    fs::create_dir_all(&stripe_dir).unwrap();

    fs::write(stripe_dir.join("mod.rs"), stripe_mod_template(connect)).unwrap();
    fs::write(stripe_dir.join("webhook.rs"), stripe_webhook_template()).unwrap();
    fs::write(stripe_dir.join("listeners.rs"), stripe_listeners_template()).unwrap();

    if connect {
        fs::write(
            stripe_dir.join("connect_webhook.rs"),
            stripe_connect_webhook_template(),
        )
        .unwrap();
    }

    let migrations_dir = base_dir.join("src/migrations");
    fs::create_dir_all(&migrations_dir).unwrap();
    let timestamp = "20260101_000000";
    let migration_name = format!("m{timestamp}_create_tenant_billing_table");
    fs::write(
        migrations_dir.join(format!("{migration_name}.rs")),
        stripe_migration_template(timestamp),
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn read_file(path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
    }

    // --- mod.rs template tests ---

    #[test]
    fn test_mod_template_without_connect() {
        let tmpl = stripe_mod_template(false);
        assert!(tmpl.contains("pub mod webhook;"));
        assert!(tmpl.contains("pub mod listeners;"));
        assert!(!tmpl.contains("pub mod connect_webhook;"));
        assert!(tmpl.contains("use ferro::Stripe;"));
        assert!(tmpl.contains("pub fn init()"));
        assert!(tmpl.contains("ferro::StripeConfig::from_env()"));
        assert!(tmpl.contains("Stripe::init(config);"));
    }

    #[test]
    fn test_mod_template_with_connect() {
        let tmpl = stripe_mod_template(true);
        assert!(tmpl.contains("pub mod webhook;"));
        assert!(tmpl.contains("pub mod listeners;"));
        assert!(tmpl.contains("pub mod connect_webhook;"));
    }

    // --- webhook.rs template tests ---

    #[test]
    fn test_webhook_template_uses_dispatch_job() {
        let tmpl = stripe_webhook_template();
        // Must use dispatch_job (not dispatch_event) per locked decision
        assert!(tmpl.contains("ferro::dispatch_job(ProcessStripeWebhook::platform("));
        assert!(!tmpl.contains("dispatch_event"));
        assert!(tmpl.contains("ferro::verify_webhook("));
        assert!(tmpl.contains("stripe-signature"));
        assert!(tmpl.contains(r#"{"received": true}"#));
    }

    #[test]
    fn test_webhook_template_uses_ferro_imports() {
        let tmpl = stripe_webhook_template();
        assert!(tmpl.contains("use ferro::{"));
        assert!(tmpl.contains("use ferro::ProcessStripeWebhook;"));
    }

    // --- connect_webhook template tests ---

    #[test]
    fn test_connect_webhook_template() {
        let tmpl = stripe_connect_webhook_template();
        assert!(tmpl.contains("stripe_connect_webhook"));
        assert!(tmpl.contains("ProcessStripeWebhook::connect("));
        assert!(tmpl.contains("ferro::dispatch_job("));
        assert!(tmpl.contains("connect_webhook_secret"));
    }

    // --- listeners.rs template tests ---

    #[test]
    fn test_listeners_template() {
        let tmpl = stripe_listeners_template();
        assert!(tmpl.contains("StripeSubscriptionUpdated"));
        assert!(tmpl.contains("StripeSubscriptionDeleted"));
        assert!(tmpl.contains("StripeCheckoutCompleted"));
        assert!(tmpl.contains("impl Listener<StripeSubscriptionUpdated> for SyncSubscriptionPlan"));
        assert!(tmpl.contains("async fn handle("));
        assert!(tmpl.contains("use ferro::{async_trait, EventError, Listener};"));
    }

    // --- migration template tests ---

    #[test]
    fn test_migration_sql_schema() {
        let tmpl = stripe_migration_template("20260101_000000");
        assert!(tmpl.contains("CREATE TABLE tenant_billing"));
        assert!(tmpl.contains("tenant_id INTEGER NOT NULL UNIQUE"));
        assert!(tmpl.contains("stripe_customer_id TEXT NOT NULL"));
        assert!(tmpl.contains("stripe_subscription_id TEXT"));
        assert!(tmpl.contains("plan TEXT NOT NULL DEFAULT 'free'"));
        assert!(tmpl.contains("subscription_status TEXT NOT NULL DEFAULT 'active'"));
        assert!(tmpl.contains("trial_ends_at TIMESTAMP"));
        assert!(tmpl.contains("current_period_end TIMESTAMP"));
        assert!(tmpl.contains("cancel_at_period_end BOOLEAN NOT NULL DEFAULT 0"));
        assert!(tmpl.contains("stripe_connect_account_id TEXT"));
        assert!(tmpl.contains("created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP"));
        assert!(tmpl.contains("updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP"));
        assert!(
            tmpl.contains("CREATE INDEX idx_tenant_billing_tenant_id ON tenant_billing(tenant_id)")
        );
        // Must have a down migration
        assert!(tmpl.contains("DROP TABLE IF EXISTS tenant_billing"));
    }

    #[test]
    fn test_migration_uses_timestamp() {
        let ts = "20260315_120000";
        let tmpl = stripe_migration_template(ts);
        assert!(tmpl.contains(&format!("m{ts}_create_tenant_billing_table")));
    }

    // --- file generation tests ---

    #[test]
    fn test_generates_required_files_without_connect() {
        let tmp = TempDir::new().unwrap();
        generate_in_dir(tmp.path(), false);

        let stripe_dir = tmp.path().join("src/stripe");
        assert!(
            stripe_dir.exists(),
            "src/stripe directory should be created"
        );
        assert!(
            stripe_dir.join("mod.rs").exists(),
            "mod.rs should be created"
        );
        assert!(
            stripe_dir.join("webhook.rs").exists(),
            "webhook.rs should be created"
        );
        assert!(
            stripe_dir.join("listeners.rs").exists(),
            "listeners.rs should be created"
        );
        assert!(
            !stripe_dir.join("connect_webhook.rs").exists(),
            "connect_webhook.rs should NOT be created without --connect"
        );
    }

    #[test]
    fn test_generates_connect_webhook_with_connect_flag() {
        let tmp = TempDir::new().unwrap();
        generate_in_dir(tmp.path(), true);

        let stripe_dir = tmp.path().join("src/stripe");
        assert!(
            stripe_dir.join("connect_webhook.rs").exists(),
            "connect_webhook.rs should be created with --connect"
        );

        // Verify connect webhook content
        let content = read_file(&stripe_dir.join("connect_webhook.rs"));
        assert!(content.contains("stripe_connect_webhook"));
        assert!(content.contains("ProcessStripeWebhook::connect("));
    }

    #[test]
    fn test_does_not_overwrite_existing_files() {
        let tmp = TempDir::new().unwrap();
        let stripe_dir = tmp.path().join("src/stripe");
        fs::create_dir_all(&stripe_dir).unwrap();

        // Write an existing mod.rs with custom content
        let existing_content = "// Custom user content that should not be overwritten\n";
        fs::write(stripe_dir.join("mod.rs"), existing_content).unwrap();

        // Generate — should skip mod.rs
        generate_in_dir(tmp.path(), false);

        // mod.rs should still have original content
        let content = read_file(&stripe_dir.join("mod.rs"));
        // Since generate_in_dir writes unconditionally for tests (it's a test helper),
        // we test the write_if_not_exists logic directly:
        let out_path = tmp.path().join("test_file.txt");
        write_if_not_exists(&out_path, "new content", "test_file.txt");
        assert_eq!(fs::read_to_string(&out_path).unwrap(), "new content");

        // Writing again should not change the content
        write_if_not_exists(&out_path, "overwritten content", "test_file.txt");
        assert_eq!(
            fs::read_to_string(&out_path).unwrap(),
            "new content",
            "write_if_not_exists must not overwrite existing files"
        );
        drop(content); // use the variable
    }

    #[test]
    fn test_migration_created() {
        let tmp = TempDir::new().unwrap();
        generate_in_dir(tmp.path(), false);

        let migrations_dir = tmp.path().join("src/migrations");
        assert!(migrations_dir.exists());

        // Verify at least one migration file exists with tenant_billing content
        let entries: Vec<_> = fs::read_dir(&migrations_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(
            !entries.is_empty(),
            "At least one migration file should be created"
        );

        let has_billing = entries.iter().any(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.contains("tenant_billing")
        });
        assert!(has_billing, "A tenant_billing migration should be created");
    }

    #[test]
    fn test_generated_webhook_uses_queue_not_events() {
        let tmp = TempDir::new().unwrap();
        generate_in_dir(tmp.path(), false);

        let webhook_path = tmp.path().join("src/stripe/webhook.rs");
        let content = read_file(&webhook_path);

        // Must use dispatch_job, not dispatch_event
        assert!(
            content.contains("dispatch_job"),
            "webhook.rs must use dispatch_job (not dispatch_event)"
        );
        assert!(
            !content.contains("dispatch_event"),
            "webhook.rs must NOT use dispatch_event"
        );
    }
}
