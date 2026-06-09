//! La Mia Libreria — entry point.
//!
//! A personal book library built on the Ferro framework. Search any book that
//! exists (Open Library), keep the ones you want in your own collection, and
//! download public-domain titles (Project Gutenberg) to read offline.
//!
//! Usage:
//!   libreria              run the web server (migrates first)
//!   libreria serve        same as above
//!   libreria db:migrate   run pending migrations and exit

use clap::{Parser, Subcommand};
use ferro::{Config, DatabaseConfig, Server};
use sea_orm_migration::prelude::*;
use std::path::Path;

mod bootstrap;
mod catalog;
mod controllers;
mod migrations;
mod models;
#[allow(dead_code)]
mod projections;
mod routes;

use migrations::Migrator;

#[derive(Parser)]
#[command(name = "libreria")]
#[command(about = "La Mia Libreria — personal book collection on Ferro")]
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
    /// Run pending database migrations and exit
    #[command(name = "db:migrate")]
    DbMigrate,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Load .env and initialize framework configuration.
    Config::init(Path::new("."));
    // Register the database config Ferro's DB facade reads from.
    Config::register(DatabaseConfig::from_env());

    match cli.command {
        None | Some(Commands::Serve { no_migrate: false }) => {
            run_migrations().await;
            run_server().await;
        }
        Some(Commands::Serve { no_migrate: true }) => run_server().await,
        Some(Commands::DbMigrate) => run_migrations().await,
    }
}

async fn run_server() {
    // Initializes the database pool and any services.
    bootstrap::register().await;

    let router = routes::register();

    Server::from_config(router).run().await.unwrap_or_else(|e| {
        eprintln!("Error: server failed to start: {e}");
        eprintln!("  Check SERVER_HOST / SERVER_PORT in .env and that the port is free.");
        std::process::exit(1);
    });
}

/// Run migrations against a standalone connection.
///
/// Uses its own connection (separate from the runtime pool created in
/// `bootstrap::register`) so the default boot path never initializes the
/// framework DB twice.
async fn run_migrations() {
    let db = connect_database().await;
    if let Err(e) = Migrator::up(&db, None).await {
        eprintln!("Error: migration failed: {e}");
        std::process::exit(1);
    }
}

/// Open a database connection, creating the SQLite file if necessary.
async fn connect_database() -> sea_orm::DatabaseConnection {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://./database.db".to_string());

    // For SQLite, make sure the file exists so the first connection succeeds.
    let database_url = if database_url.starts_with("sqlite://") {
        let path = database_url
            .trim_start_matches("sqlite://")
            .trim_start_matches("./");
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
        database_url
    };

    sea_orm::Database::connect(&database_url)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Error: failed to connect to database: {e}");
            eprintln!("  Set DATABASE_URL in .env (default: sqlite://./database.db).");
            std::process::exit(1);
        })
}
