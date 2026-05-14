//! Ferro Application Entry Point
//!
//! This is the unified entry point for the Ferro application.
//! It provides subcommands for running the web server, migrations, and scheduler.
//!
//! # Usage
//!
//! ```bash
//! ./app                    # Run web server with auto-migrate (default)
//! ./app serve              # Run web server with auto-migrate
//! ./app serve --no-migrate # Run web server without auto-migrate
//! ./app db:migrate         # Run pending migrations
//! ./app db:status          # Show migration status
//! ./app db:rollback        # Rollback last migration
//! ./app db:fresh           # Drop all tables and re-run migrations
//! ./app schedule:work      # Run scheduler daemon
//! ./app schedule:run       # Run due tasks once
//! ./app schedule:list      # List registered tasks
//! ```

use clap::{Parser, Subcommand};
use ferro::{Config, Server};
use sea_orm_migration::prelude::*;
use std::env;
use std::path::Path;

/// Print actionable error and exit
fn fail_with(context: &str, error: impl std::fmt::Display, how_to_fix: &[&str]) -> ! {
    eprintln!("Error: {context}");
    eprintln!("  Cause: {error}");
    eprintln!();
    if !how_to_fix.is_empty() {
        eprintln!("How to fix:");
        for (i, fix) in how_to_fix.iter().enumerate() {
            eprintln!("  {}. {}", i + 1, fix);
        }
        eprintln!();
    }
    std::process::exit(1)
}

mod actions;
mod api;
mod bootstrap;
mod config;
mod controllers;
mod middleware;
mod migrations;
mod models;
#[allow(dead_code)]
mod projections;
mod providers;
mod resources;
mod routes;

use migrations::Migrator;

#[derive(Parser)]
#[command(name = "app")]
#[command(about = "Ferro application server and utilities")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the web server (default command)
    Serve {
        /// Skip running migrations on startup
        #[arg(long)]
        no_migrate: bool,
    },
    /// Run pending database migrations
    #[command(name = "db:migrate")]
    DbMigrate,
    /// Show migration status
    #[command(name = "db:status")]
    DbStatus,
    /// Rollback the last migration(s)
    #[command(name = "db:rollback")]
    DbRollback {
        /// Number of migrations to rollback
        #[arg(default_value = "1")]
        steps: u32,
    },
    /// Drop all tables and re-run all migrations
    #[command(name = "db:fresh")]
    DbFresh,
    /// Run the scheduler daemon (checks every minute)
    #[command(name = "schedule:work")]
    ScheduleWork,
    /// Run all due scheduled tasks once
    #[command(name = "schedule:run")]
    ScheduleRun,
    /// List all registered scheduled tasks
    #[command(name = "schedule:list")]
    ScheduleList,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize framework configuration (loads .env files)
    Config::init(Path::new("."));

    // Register application configs
    config::register_all();

    match cli.command {
        None | Some(Commands::Serve { no_migrate: false }) => {
            // Default: run server with auto-migrate
            run_migrations_silent().await;
            run_server().await;
        }
        Some(Commands::Serve { no_migrate: true }) => {
            // Run server without migrations
            run_server().await;
        }
        Some(Commands::DbMigrate) => {
            run_migrations().await;
        }
        Some(Commands::DbStatus) => {
            show_migration_status().await;
        }
        Some(Commands::DbRollback { steps }) => {
            rollback_migrations(steps).await;
        }
        Some(Commands::DbFresh) => {
            fresh_migrations().await;
        }
        Some(Commands::ScheduleWork) => {
            run_scheduler_daemon().await;
        }
        Some(Commands::ScheduleRun) => {
            run_scheduled_tasks().await;
        }
        Some(Commands::ScheduleList) => {
            list_scheduled_tasks().await;
        }
    }
}

async fn run_server() {
    // Register services and global middleware
    bootstrap::register().await;

    let router = routes::register();

    Server::from_config(router).run().await.unwrap_or_else(|e| {
        fail_with(
            "Server failed to start",
            e,
            &[
                "Check SERVER_HOST and SERVER_PORT in .env",
                "Ensure the port is not already in use",
                "Verify network permissions allow binding to the address",
            ],
        )
    });
}

async fn get_database_connection() -> sea_orm::DatabaseConnection {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|e| {
        fail_with(
            "DATABASE_URL not set",
            e,
            &[
                "Add DATABASE_URL to .env file",
                "Example: DATABASE_URL=sqlite://./database.db",
                "Example: DATABASE_URL=postgres://user:pass@localhost/db",
            ],
        )
    });

    // For SQLite, ensure the database file can be created
    let database_url = if database_url.starts_with("sqlite://") {
        let path = database_url.trim_start_matches("sqlite://");
        let path = path.trim_start_matches("./");

        if let Some(parent) = Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).ok();
            }
        }

        if !Path::new(path).exists() {
            std::fs::File::create(path).ok();
        }

        format!("sqlite:{path}?mode=rwc")
    } else {
        database_url.clone()
    };

    let url_for_error = database_url.clone();
    sea_orm::Database::connect(&database_url)
        .await
        .unwrap_or_else(|e| {
            fail_with(
                &format!("Failed to connect to database at {url_for_error}"),
                e,
                &[
                    "Check that the database server is running",
                    "Verify the DATABASE_URL is correct",
                    "Ensure database credentials are valid",
                ],
            )
        })
}

/// Run migrations during server boot without success logging.
///
/// On failure this aborts the process via `std::process::exit(1)` to prevent
/// serving traffic with a stale schema. See Phase 157.
async fn run_migrations_silent() {
    let db = get_database_connection().await;
    if let Err(e) = Migrator::up(&db, None).await {
        eprintln!("Migration failed: {e}");
        std::process::exit(1);
    }
}

async fn run_migrations() {
    println!("Running migrations...");
    let db = get_database_connection().await;
    Migrator::up(&db, None).await.unwrap_or_else(|e| {
        fail_with(
            "Migration failed",
            e,
            &[
                "Run `./app db:status` to see pending migrations",
                "Check database permissions",
                "Verify migration files are valid",
            ],
        )
    });
    println!("Migrations completed successfully!");
}

async fn show_migration_status() {
    println!("Migration status:");
    let db = get_database_connection().await;
    Migrator::status(&db).await.unwrap_or_else(|e| {
        fail_with(
            "Could not get migration status",
            e,
            &[
                "Ensure database is accessible",
                "Check DATABASE_URL in .env",
            ],
        )
    });
}

async fn rollback_migrations(steps: u32) {
    println!("Rolling back {steps} migration(s)...");
    let db = get_database_connection().await;
    Migrator::down(&db, Some(steps)).await.unwrap_or_else(|e| {
        fail_with(
            "Rollback failed",
            e,
            &[
                "Check that there are migrations to rollback with `./app db:status`",
                "Verify database permissions",
            ],
        )
    });
    println!("Rollback completed successfully!");
}

async fn fresh_migrations() {
    println!("WARNING: Dropping all tables and re-running migrations...");
    let db = get_database_connection().await;
    Migrator::fresh(&db).await.unwrap_or_else(|e| {
        fail_with(
            "Database refresh failed",
            e,
            &[
                "Ensure database user has DROP TABLE permissions",
                "Check database connection is valid",
            ],
        )
    });
    println!("Database refreshed successfully!");
}

// Schedule functionality - these will work once the user creates tasks with `ferro make:task`
async fn run_scheduler_daemon() {
    // Bootstrap the application for scheduler context
    bootstrap::register().await;

    println!("==============================================");
    println!("  Ferro Scheduler Daemon");
    println!("==============================================");
    println!();
    println!("  Note: Create tasks with `ferro make:task <name>`");
    println!("  Press Ctrl+C to stop");
    println!();
    println!("==============================================");

    // For now, this is a placeholder that will be enhanced when schedule module exists
    // In a full implementation, this would load src/schedule.rs and run tasks
    eprintln!("Scheduler daemon is not yet configured.");
    eprintln!("Create a scheduled task with: ferro make:task <name>");
    eprintln!("Then register it in src/schedule.rs");
}

async fn run_scheduled_tasks() {
    // Bootstrap the application for scheduler context
    bootstrap::register().await;

    println!("Running scheduled tasks...");
    eprintln!("Scheduler is not yet configured.");
    eprintln!("Create a scheduled task with: ferro make:task <name>");
}

async fn list_scheduled_tasks() {
    println!("Registered scheduled tasks:");
    println!();
    eprintln!("No scheduled tasks registered.");
    eprintln!("Create a scheduled task with: ferro make:task <name>");
}
