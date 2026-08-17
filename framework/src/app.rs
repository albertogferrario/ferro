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
        let database_url = match std::env::var("DATABASE_URL") {
            Ok(u) => u,
            Err(_) => {
                eprintln!("DATABASE_URL must be set");
                std::process::exit(1);
            }
        };
        let db = match sea_orm::Database::connect(&database_url).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to connect to database: {e}");
                std::process::exit(1);
            }
        };

        let registry = match seeders_fn {
            Some(f) => f(),
            None => {
                eprintln!("No seeders registered.");
                eprintln!("Register seeders with .seeders(seeders::register) in main.rs");
                return;
            }
        };

        let result = match class {
            Some(name) => registry.run_one(&name, &db).await,
            None => registry.run_all(&db).await,
        };

        if let Err(e) = result {
            eprintln!("Seeding failed: {e}");
            std::process::exit(1);
        }
    }

    /// Shared boot step executed by both `serve` and `run_worker`.
    ///
    /// Runs bootstrap, initialises the queue DB connection, wires the WR-01
    /// broadcast transport when configured, and registers the offload result hooks.
    /// When `no_worker` is `false` (serve path) it also spawns an in-process
    /// `WorkerLoop` over all registered queues (D-05).
    ///
    /// # WR-01 transport attach
    ///
    /// After `bootstrap_fn` runs (so the app-registered `Broadcaster` is
    /// available), the framework checks whether `BroadcastConfig.transport_redis_url`
    /// is set:
    ///
    /// - `redis-transport` feature ON + URL set → `RedisTransport::connect`; on
    ///   success the transport-attached `Broadcaster` replaces the singleton and
    ///   the broadcaster-aware hook is registered; on connect failure a `warn!` is
    ///   emitted and the in-process hub is used.
    /// - `redis-transport` feature OFF + URL set → one `warn!`, in-process hub.
    /// - No URL → no change.
    ///
    /// A framework-default in-process `Broadcaster` is installed when bootstrap
    /// registered none, so the broadcaster-aware result hook is always the registration path.
    #[doc(hidden)]
    pub async fn run_common_boot(bootstrap_fn: Option<BootstrapFn>, no_worker: bool) {
        // Step 1: bootstrap — registers the Broadcaster singleton and any other app services.
        if let Some(bootstrap_fn) = bootstrap_fn {
            bootstrap_fn().await;
        }
        // Initialise the queue DB connection. Guard with is_initialized() so a consumer
        // bootstrap that already called Queue::init() does not double-init.
        if !ferro_queue::Queue::is_initialized() {
            let conn = Self::get_database_connection().await;
            let _ = ferro_queue::Queue::init(conn).await;
        }

        // Step 2: ensure a Broadcaster (D-01). Worker-only boots that registered none get a
        // framework-default in-process hub, so App::get::<Broadcaster>() is always Some by the
        // time the offload hook registers. The hub publishes to nobody in that case, which is
        // harmless — the broadcast path is best-effort and swallows zero-subscriber sends, and
        // the snapshot remains the authoritative record (247 D-02 / Pitfall 5).
        if crate::App::get::<ferro_broadcast::Broadcaster>().is_none() {
            crate::App::singleton(ferro_broadcast::Broadcaster::new());
        }

        // Step 3: attach the shared broadcast transport when configured (WR-01: after bootstrap so
        // the Broadcaster is present, before hook registration so the hook sees the transport-
        // attached instance; WR-03: before any WebSocket client connects — run_common_boot returns
        // before Server::from_config().run()). App::get is now guaranteed Some (Step 2).
        #[cfg(feature = "redis-transport")]
        {
            let bc = crate::App::get::<ferro_broadcast::Broadcaster>()
                .expect("Broadcaster ensured in Step 2");
            if let Some(ref url) = bc.config().transport_redis_url {
                match ferro_broadcast::transport::redis::RedisTransport::connect(url).await {
                    Ok(t) => {
                        let bc2 = bc.with_transport(std::sync::Arc::new(t)).await;
                        crate::App::singleton(bc2);
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "BROADCAST_REDIS_URL set but Redis connect failed — in-process hub only"
                        );
                    }
                }
            }
        }
        // D-07: feature-off + URL-set → one warn, then proceed with the in-process hub.
        #[cfg(not(feature = "redis-transport"))]
        {
            let bc = crate::App::get::<ferro_broadcast::Broadcaster>()
                .expect("Broadcaster ensured in Step 2");
            if bc.config().transport_redis_url.is_some() {
                tracing::warn!(
                    "BROADCAST_REDIS_URL is set but the `redis-transport` feature is disabled \
                     — falling back to in-process hub"
                );
            }
        }

        // Step 4: register the offload result hook ONCE. The Broadcaster is guaranteed present
        // (Step 2) and transport-attached if configured (Step 3). ferro-queue keeps the hook in a
        // OnceLock — re-registration is silently ignored, and this is the single production site.
        let bc = crate::App::get::<ferro_broadcast::Broadcaster>()
            .expect("Broadcaster ensured in Step 2");
        crate::offload::register_offload_hooks_with_broadcaster(std::sync::Arc::new(bc));

        // Step 5: serve spawns an in-process worker over all registered queues ONCE (D-05).
        // run_worker passes no_worker = true and starts its own WorkerLoop, so this is skipped there.
        if !no_worker && ferro_queue::Queue::has_registered_jobs() {
            Self::spawn_in_process_worker();
        }
    }

    /// Spawn an in-process `WorkerLoop` over all registered queues (D-05).
    ///
    /// The loop runs on a detached `tokio::spawn`; serve continues to the HTTP
    /// accept loop. Any `WorkerLoop` error is printed but does not terminate the
    /// process.
    fn spawn_in_process_worker() {
        // Warn when jobs are registered (so a WorkerLoop is started) but the
        // queue is in sync mode (WR-04). In sync mode every `dispatch()`
        // runs inline in the request path — `.delay()`/`.on_queue()` are
        // ignored — while the WorkerLoop polls an empty queue. This is the
        // default when QUEUE_CONNECTION is unset, which is a foot-gun in
        // production.
        if ferro_queue::QueueConfig::is_sync_mode() {
            eprintln!(
                "WARNING: queue jobs are registered but QUEUE_CONNECTION is sync \
                 (or unset, which defaults to sync). dispatch() will run jobs inline \
                 in the request path and ignore delay/on_queue. Set QUEUE_CONNECTION \
                 to a non-sync value (e.g. 'db') to enable background processing."
            );
        }
        let all_queues = ferro_queue::Queue::registered_queue_names();
        let config = ferro_queue::WorkerConfig::new(all_queues);
        let worker = ferro_queue::WorkerLoop::from_registry(config);
        tokio::spawn(async move {
            if let Err(e) = worker.run().await {
                eprintln!("WorkerLoop exited with error: {e}");
            }
        });
    }

    /// Run the background worker process.
    ///
    /// Executes the shared boot step (`run_common_boot`) with `no_worker = true`
    /// so the serve in-process worker is not spawned. Then runs a `WorkerLoop`
    /// directly (blocking until SIGTERM / Ctrl-C).
    ///
    /// `queues` controls which queues this worker consumes:
    /// - Empty → all registered queues (D-03).
    /// - Non-empty → exactly the named queues.
    pub async fn run_worker(bootstrap_fn: Option<BootstrapFn>, queues: Vec<String>) {
        Self::run_common_boot(bootstrap_fn, /*no_worker=*/ true).await;
        let effective_queues = if queues.is_empty() {
            ferro_queue::Queue::registered_queue_names() // D-03: all registered
        } else {
            queues
        };

        // WR-02: warn when a requested queue has no registered job handlers.
        // A typo like `--queue reprots` otherwise boots a healthy-looking worker
        // that claims and reaps nothing — an idle loop indistinguishable from a
        // busy one. This is a warning, not a hard error: a queue may legitimately
        // have zero local handlers if its jobs are enqueued by another service.
        let known = ferro_queue::Queue::registered_queue_names();
        for q in &effective_queues {
            if !known.contains(q) {
                tracing::warn!(
                    queue = %q,
                    "worker started for a queue with no registered job handlers — it will idle"
                );
            }
        }

        let config = ferro_queue::WorkerConfig::new(effective_queues);
        let worker = ferro_queue::WorkerLoop::from_registry(config);
        if let Err(e) = worker.run().await {
            eprintln!("Worker exited with error: {e}");
            std::process::exit(1);
        }
    }

    async fn run_server_internal(
        bootstrap_fn: Option<BootstrapFn>,
        routes_fn: Option<Box<dyn FnOnce() -> Router + Send>>,
    ) {
        Self::run_common_boot(bootstrap_fn, /*no_worker=*/ false).await;

        // Get router
        let router = if let Some(routes_fn) = routes_fn {
            routes_fn()
        } else {
            Router::new()
        };

        // Create server with configuration from environment
        if let Err(e) = Server::from_config(router).run().await {
            eprintln!("Failed to start server: {e}");
            std::process::exit(1);
        }
    }

    async fn get_database_connection() -> sea_orm::DatabaseConnection {
        // WR-01: both the `serve` (in-process) and `worker` boot paths route
        // through here via `run_common_boot`. A bare `.expect(...)` panic on a
        // misconfigured DATABASE_URL — a common first-run failure mode for a
        // deployable worker — is an operational regression relative to the
        // consumer `main.rs`, which prints actionable remediation and exits(1).
        // Mirror that behaviour so both paths surface the same guidance.
        let database_url = match env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(e) => {
                eprintln!("Error: DATABASE_URL not set");
                eprintln!("  Cause: {e}");
                eprintln!();
                eprintln!("How to fix:");
                eprintln!("  1. Add DATABASE_URL to .env file");
                eprintln!("  2. Example: DATABASE_URL=sqlite://./database.db");
                eprintln!("  3. Example: DATABASE_URL=postgres://user:pass@localhost/db");
                std::process::exit(1);
            }
        };

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

        match sea_orm::Database::connect(&database_url).await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("Error: Failed to connect to database at {database_url}");
                eprintln!("  Cause: {e}");
                eprintln!();
                eprintln!("How to fix:");
                eprintln!("  1. Check that the database server is running");
                eprintln!("  2. Verify the DATABASE_URL is correct");
                eprintln!("  3. Ensure database credentials are valid");
                std::process::exit(1);
            }
        }
    }

    /// Run migrations during server boot without success logging.
    ///
    /// "Silent" refers only to the success path (no progress logs that would
    /// interleave with server startup). On failure this method writes to stderr
    /// and aborts the process to prevent the server from accepting traffic with
    /// a stale schema.
    async fn run_migrations_silent<Migrator: MigratorTrait>() {
        let db = Self::get_database_connection().await;
        if let Err(e) = Migrator::up(&db, None).await {
            eprintln!("Migration failed: {e}");
            std::process::exit(1);
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

// ---------------------------------------------------------------------------
// Module-level entry points (facade re-exports)
// ---------------------------------------------------------------------------

/// Run the background worker process.
///
/// Convenience free function over [`Application::run_worker`]. Executes the
/// shared boot step and then runs a `WorkerLoop` over `queues` (or all
/// registered queues when `queues` is empty).
///
/// Re-exported at the `ferro` facade level as `ferro::run_worker`.
pub async fn run_worker(bootstrap_fn: Option<BootstrapFn>, queues: Vec<String>) {
    Application::<NoMigrator>::run_worker(bootstrap_fn, queues).await;
}

/// Shared boot step for both `serve` and `run_worker`.
///
/// Exposed `pub` so integration tests can drive the boot step directly.
/// Not part of the stable public API — prefer [`Application`] for production use.
#[doc(hidden)]
pub async fn run_common_boot(bootstrap_fn: Option<BootstrapFn>, no_worker: bool) {
    Application::<NoMigrator>::run_common_boot(bootstrap_fn, no_worker).await;
}
