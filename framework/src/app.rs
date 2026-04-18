//! Application builder for Ferro framework
//!
//! Provides a fluent builder API to configure and run a Ferro application.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::Application;
//!
//! #[tokio::main]
//! async fn main() {
//!     Application::new()
//!         .config(config::register_all)
//!         .bootstrap(bootstrap::register)
//!         .routes(routes::register)
//!         .migrations::<migrations::Migrator>()
//!         .run()
//!         .await;
//! }
//! ```

use crate::seeder::SeederRegistry;
use crate::{Config, Router, Server};
use clap::{Parser, Subcommand};
use sea_orm_migration::prelude::*;
use std::env;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;

/// Type alias for async bootstrap function
type BootstrapFn = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

/// CLI structure for Ferro applications
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
    /// Run database seeders
    #[command(name = "db:seed")]
    DbSeed {
        /// Run only a specific seeder
        #[arg(long)]
        class: Option<String>,
    },
    /// Export the JSON-UI v2 spec schema (full spec or a single component's Props)
    #[cfg(feature = "json-ui")]
    #[command(name = "json-ui:schema")]
    JsonUiSchema {
        /// Write to file instead of stdout
        #[arg(long, short = 'o')]
        output: Option<String>,

        /// Pretty-print JSON output (default behavior — flag accepted for explicitness)
        #[arg(long)]
        pretty: bool,

        /// Export only the Props schema for a single component (e.g., "Card")
        #[arg(long)]
        component: Option<String>,
    },
}

/// Application builder for Ferro framework
///
/// Use this to configure and run your Ferro application with a fluent API.
pub struct Application<M = NoMigrator>
where
    M: MigratorTrait,
{
    config_fn: Option<Box<dyn FnOnce()>>,
    bootstrap_fn: Option<BootstrapFn>,
    routes_fn: Option<Box<dyn FnOnce() -> Router + Send>>,
    seeders_fn: Option<Box<dyn FnOnce() -> SeederRegistry + Send>>,
    _migrator: std::marker::PhantomData<M>,
}

/// Placeholder type for when no migrator is configured
pub struct NoMigrator;

impl MigratorTrait for NoMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![]
    }
}

impl Application<NoMigrator> {
    /// Create a new application builder
    pub fn new() -> Self {
        Application {
            config_fn: None,
            bootstrap_fn: None,
            routes_fn: None,
            seeders_fn: None,
            _migrator: std::marker::PhantomData,
        }
    }
}

impl Default for Application<NoMigrator> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M> Application<M>
where
    M: MigratorTrait,
{
    /// Register a configuration function
    ///
    /// This function is called early during startup to register
    /// application configuration.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// App::new()
    ///     .config(config::register_all)
    /// ```
    pub fn config<F>(mut self, f: F) -> Self
    where
        F: FnOnce() + 'static,
    {
        self.config_fn = Some(Box::new(f));
        self
    }

    /// Register a bootstrap function
    ///
    /// This async function is called to register services, middleware,
    /// and other application components.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// App::new()
    ///     .bootstrap(bootstrap::register)
    /// ```
    pub fn bootstrap<F, Fut>(mut self, f: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.bootstrap_fn = Some(Box::new(move || Box::pin(f())));
        self
    }

    /// Register a routes function
    ///
    /// This function returns the application's router configuration.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// App::new()
    ///     .routes(routes::register)
    /// ```
    pub fn routes<F>(mut self, f: F) -> Self
    where
        F: FnOnce() -> Router + Send + 'static,
    {
        self.routes_fn = Some(Box::new(f));
        self
    }

    /// Configure the migrator type for database migrations
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Application::new()
    ///     .migrations::<migrations::Migrator>()
    /// ```
    pub fn migrations<NewM>(self) -> Application<NewM>
    where
        NewM: MigratorTrait,
    {
        Application {
            config_fn: self.config_fn,
            bootstrap_fn: self.bootstrap_fn,
            routes_fn: self.routes_fn,
            seeders_fn: self.seeders_fn,
            _migrator: std::marker::PhantomData,
        }
    }

    /// Register a seeders function
    ///
    /// This function returns the application's seeder registry for database seeding.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Application::new()
    ///     .seeders(seeders::register)
    /// ```
    pub fn seeders<F>(mut self, f: F) -> Self
    where
        F: FnOnce() -> SeederRegistry + Send + 'static,
    {
        self.seeders_fn = Some(Box::new(f));
        self
    }

    /// Run the application
    ///
    /// This parses CLI arguments and executes the appropriate command:
    /// - `serve` (default): Run the web server
    /// - `db:migrate`: Run pending migrations
    /// - `db:status`: Show migration status
    /// - `db:rollback`: Rollback migrations
    /// - `db:fresh`: Drop and re-run all migrations
    /// - `schedule:*`: Scheduler commands
    pub async fn run(self) {
        let cli = Cli::parse();

        // Initialize framework configuration (loads .env files)
        Config::init(Path::new("."));

        // Destructure self to avoid partial move issues
        let Application {
            config_fn,
            bootstrap_fn,
            routes_fn,
            seeders_fn,
            _migrator,
        } = self;

        // Run user's config registration
        if let Some(config_fn) = config_fn {
            config_fn();
        }

        // Initialize translator (after config so user can override LangConfig)
        crate::lang::init::init();

        match cli.command {
            None | Some(Commands::Serve { no_migrate: false }) => {
                // Default: run server with auto-migrate
                Self::run_migrations_silent::<M>().await;
                Self::run_server_internal(bootstrap_fn, routes_fn).await;
            }
            Some(Commands::Serve { no_migrate: true }) => {
                // Run server without migrations
                Self::run_server_internal(bootstrap_fn, routes_fn).await;
            }
            Some(Commands::DbMigrate) => {
                Self::run_migrations::<M>().await;
            }
            Some(Commands::DbStatus) => {
                Self::show_migration_status::<M>().await;
            }
            Some(Commands::DbRollback { steps }) => {
                Self::rollback_migrations::<M>(steps).await;
            }
            Some(Commands::DbFresh) => {
                Self::fresh_migrations::<M>().await;
            }
            Some(Commands::ScheduleWork) => {
                Self::run_scheduler_daemon_internal(bootstrap_fn).await;
            }
            Some(Commands::ScheduleRun) => {
                Self::run_scheduled_tasks_internal(bootstrap_fn).await;
            }
            Some(Commands::ScheduleList) => {
                Self::list_scheduled_tasks().await;
            }
            Some(Commands::DbSeed { class }) => {
                Self::run_seeders(seeders_fn, class).await;
            }
            #[cfg(feature = "json-ui")]
            Some(Commands::JsonUiSchema {
                output,
                pretty,
                component,
            }) => {
                Self::run_json_ui_schema(output, pretty, component).await;
            }
        }
    }

    #[cfg(feature = "json-ui")]
    async fn run_json_ui_schema(output: Option<String>, pretty: bool, component: Option<String>) {
        // Build a local Catalog so BuildFailed surfaces as non-zero exit
        // (NOT a panic via global_catalog's `expect`). RESEARCH §8 L-1 pattern.
        let catalog = match ferro_json_ui::Catalog::build() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error building catalog: {e}");
                std::process::exit(1);
            }
        };

        let value: &serde_json::Value = match &component {
            Some(name) => match catalog.component_schema(name) {
                Some(v) => v,
                None => {
                    eprintln!("error: unknown component '{name}'");
                    std::process::exit(1);
                }
            },
            None => catalog.json_schema(),
        };

        // CONTEXT D-21: default output is pretty-printed. The --pretty flag
        // stays as an explicit opt-in for back-compat with tooling that passes
        // it; compact is NOT reachable via any flag in Phase 117.
        let _ = pretty;
        let serialized = serde_json::to_string_pretty(value).expect("schema serializes");

        match output {
            Some(path) => {
                if let Err(e) = std::fs::write(&path, serialized) {
                    eprintln!("error writing to {path}: {e}");
                    std::process::exit(1);
                }
            }
            None => println!("{serialized}"),
        }
    }

    async fn run_seeders(
        seeders_fn: Option<Box<dyn FnOnce() -> SeederRegistry + Send>>,
        class: Option<String>,
    ) {
        // Initialize database connection for seeders
        let config = crate::database::DatabaseConfig::from_env();
        if let Err(e) = crate::database::DB::init_with(config).await {
            eprintln!("Failed to connect to database: {e}");
            std::process::exit(1);
        }

        let registry = match seeders_fn {
            Some(f) => f(),
            None => {
                eprintln!("No seeders registered.");
                eprintln!("Register seeders with .seeders(seeders::register) in main.rs");
                return;
            }
        };

        let result = match class {
            Some(name) => registry.run_one(&name).await,
            None => registry.run_all().await,
        };

        if let Err(e) = result {
            eprintln!("Seeding failed: {e}");
            std::process::exit(1);
        }
    }

    async fn run_server_internal(
        bootstrap_fn: Option<BootstrapFn>,
        routes_fn: Option<Box<dyn FnOnce() -> Router + Send>>,
    ) {
        // Run bootstrap
        if let Some(bootstrap_fn) = bootstrap_fn {
            bootstrap_fn().await;
        }

        // Get router
        let router = if let Some(routes_fn) = routes_fn {
            routes_fn()
        } else {
            Router::new()
        };

        // Create server with configuration from environment
        Server::from_config(router)
            .run()
            .await
            .expect("Failed to start server");
    }

    async fn get_database_connection() -> sea_orm::DatabaseConnection {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

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
            database_url
        };

        sea_orm::Database::connect(&database_url)
            .await
            .expect("Failed to connect to database")
    }

    async fn run_migrations_silent<Migrator: MigratorTrait>() {
        let db = Self::get_database_connection().await;
        if let Err(e) = Migrator::up(&db, None).await {
            eprintln!("Warning: Migration failed: {e}");
        }
    }

    async fn run_migrations<Migrator: MigratorTrait>() {
        println!("Running migrations...");
        let db = Self::get_database_connection().await;
        Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");
        println!("Migrations completed successfully!");
    }

    async fn show_migration_status<Migrator: MigratorTrait>() {
        println!("Migration status:");
        let db = Self::get_database_connection().await;
        Migrator::status(&db)
            .await
            .expect("Failed to get migration status");
    }

    async fn rollback_migrations<Migrator: MigratorTrait>(steps: u32) {
        println!("Rolling back {steps} migration(s)...");
        let db = Self::get_database_connection().await;
        Migrator::down(&db, Some(steps))
            .await
            .expect("Failed to rollback migrations");
        println!("Rollback completed successfully!");
    }

    async fn fresh_migrations<Migrator: MigratorTrait>() {
        println!("WARNING: Dropping all tables and re-running migrations...");
        let db = Self::get_database_connection().await;
        Migrator::fresh(&db)
            .await
            .expect("Failed to refresh database");
        println!("Database refreshed successfully!");
    }

    async fn run_scheduler_daemon_internal(bootstrap_fn: Option<BootstrapFn>) {
        // Run bootstrap for scheduler context
        if let Some(bootstrap_fn) = bootstrap_fn {
            bootstrap_fn().await;
        }

        println!("==============================================");
        println!("  Ferro Scheduler Daemon");
        println!("==============================================");
        println!();
        println!("  Note: Create tasks with `ferro make:task <name>`");
        println!("  Press Ctrl+C to stop");
        println!();
        println!("==============================================");

        eprintln!("Scheduler daemon is not yet configured.");
        eprintln!("Create a scheduled task with: ferro make:task <name>");
        eprintln!("Then register it in src/schedule.rs");
    }

    async fn run_scheduled_tasks_internal(bootstrap_fn: Option<BootstrapFn>) {
        // Run bootstrap for scheduler context
        if let Some(bootstrap_fn) = bootstrap_fn {
            bootstrap_fn().await;
        }

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
}
