//! Ferro RealWorld/Conduit backend — benchmark harness

// Wave 1 scaffolds the JWT module, both middlewares, and the health handler.
// Later waves (Plans 02-06) wire them into routes/controllers. Allow until then.
#![allow(dead_code)]

use clap::{Parser, Subcommand};
use ferro::{Config, Server};
use sea_orm_migration::prelude::*;
use std::path::Path;

mod bootstrap;
mod config;
mod controllers;
mod dto;
mod jwt;
mod middleware;
mod migrations;
mod models;
mod routes;

use migrations::Migrator;

#[derive(Parser)]
#[command(name = "ferro-conduit")]
#[command(about = "Ferro RealWorld/Conduit benchmark backend")]
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
        #[arg(default_value = "1")]
        steps: u32,
    },
    /// Drop all tables and re-run all migrations
    #[command(name = "db:fresh")]
    DbFresh,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    Config::init(Path::new("."));
    config::register_all();

    match cli.command {
        None | Some(Commands::Serve { no_migrate: false }) => {
            run_migrations_silent().await;
            run_server().await;
        }
        Some(Commands::Serve { no_migrate: true }) => {
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
    }
}

async fn run_server() {
    bootstrap::register().await;
    let router = routes::register();
    if let Err(e) = Server::from_config(router).run().await {
        eprintln!("Failed to start server: {e}");
        std::process::exit(1);
    }
}

async fn get_database_connection() -> sea_orm::DatabaseConnection {
    use std::env;
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    sea_orm::Database::connect(&database_url)
        .await
        .expect("Failed to connect to database")
}

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
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");
    println!("Migrations completed successfully!");
}

async fn show_migration_status() {
    println!("Migration status:");
    let db = get_database_connection().await;
    Migrator::status(&db)
        .await
        .expect("Failed to get migration status");
}

async fn rollback_migrations(steps: u32) {
    println!("Rolling back {steps} migration(s)...");
    let db = get_database_connection().await;
    Migrator::down(&db, Some(steps))
        .await
        .expect("Failed to rollback migrations");
    println!("Rollback completed successfully!");
}

async fn fresh_migrations() {
    println!("Dropping all tables and re-running migrations...");
    let db = get_database_connection().await;
    Migrator::fresh(&db)
        .await
        .expect("Failed to refresh database");
    println!("Database refreshed successfully!");
}
