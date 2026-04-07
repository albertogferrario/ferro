use ferro_cli::commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ferro")]
#[command(version)]
#[command(about = "A CLI for scaffolding Ferro web applications", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Ferro project
    New {
        /// The name of the project to create
        name: Option<String>,

        /// Skip all prompts and use defaults
        #[arg(long)]
        no_interaction: bool,

        /// Skip git initialization
        #[arg(long)]
        no_git: bool,
    },
    /// Start the development servers (backend + frontend)
    Serve {
        /// Backend port (default: 8080)
        #[arg(long, short = 'p', default_value = "8080")]
        port: u16,

        /// Frontend port (default: 5173)
        #[arg(long, default_value = "5173")]
        frontend_port: u16,

        /// Only start backend server
        #[arg(long)]
        backend_only: bool,

        /// Only start frontend server
        #[arg(long)]
        frontend_only: bool,

        /// Skip TypeScript type generation
        #[arg(long)]
        skip_types: bool,
    },
    /// Generate TypeScript types from Rust InertiaProps structs
    GenerateTypes {
        /// Output file path (default: frontend/src/types/inertia-props.ts)
        #[arg(long, short = 'o')]
        output: Option<String>,

        /// Watch for changes and regenerate
        #[arg(long, short = 'w')]
        watch: bool,
    },
    /// Generate type-safe TypeScript route helpers (or stable JSON with --json)
    #[command(name = "generate-routes")]
    GenerateRoutes {
        /// Output file path (TS mode only, default: frontend/src/types/routes.ts)
        #[arg(long, short = 'o')]
        output: Option<String>,

        /// Emit stable JSON schema to stdout instead of writing TypeScript (D-10..D-12)
        #[arg(long)]
        json: bool,
    },
    /// Generate a new middleware
    #[command(name = "make:middleware")]
    MakeMiddleware {
        /// Name of the middleware (e.g., Auth, RateLimit)
        name: String,
    },
    /// Generate a new controller
    #[command(name = "make:controller")]
    MakeController {
        /// Name of the controller (e.g., users, user_profile)
        name: String,
    },
    /// Generate a new action
    #[command(name = "make:action")]
    MakeAction {
        /// Name of the action (e.g., AddTodo, CreateUser)
        name: String,
    },
    /// Generate an API key for authentication
    #[command(name = "make:api-key")]
    MakeApiKey {
        /// Name of the key (e.g., "Production Bot")
        name: String,
        /// Environment: "live" or "test"
        #[arg(long, default_value = "live")]
        env: String,
    },
    /// Scaffold a complete REST API layer for existing models
    #[command(name = "make:api")]
    MakeApi {
        /// Model names to generate API for (e.g., User Post)
        models: Vec<String>,
        /// Generate API for all detected models
        #[arg(long)]
        all: bool,
        /// Skip confirmation prompts
        #[arg(long, short = 'y')]
        yes: bool,
        /// Fields to exclude from API resources (comma-separated, e.g., password_hash,secret_token)
        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,
        /// Include all fields in API resources (disable auto-exclusion of sensitive fields)
        #[arg(long)]
        include_all: bool,
    },
    /// Scaffold a complete authentication system (migration, controller, routes)
    #[command(name = "make:auth")]
    MakeAuth {
        /// Overwrite existing files
        #[arg(long, short = 'f')]
        force: bool,
    },
    /// Scaffold Stripe integration (webhook routes, event listeners, migration, env config)
    #[command(name = "make:stripe")]
    MakeStripe {
        /// Include Stripe Connect scaffolding (connect webhook, connect account ID field)
        #[arg(long)]
        connect: bool,
    },
    /// Scaffold WhatsApp Business integration (webhook routes, event listeners, env config)
    #[command(name = "make:whatsapp")]
    MakeWhatsapp,
    /// Generate a new domain error
    #[command(name = "make:error")]
    MakeError {
        /// Name of the error (e.g., UserNotFound, InvalidInput)
        name: String,
    },
    /// Generate a new Inertia page
    #[command(name = "make:inertia")]
    MakeInertia {
        /// Name of the page (e.g., About, UserProfile)
        name: String,
    },
    /// Generate a new JSON-UI view (AI-powered)
    #[command(name = "make:json-view")]
    MakeJsonView {
        /// Name of the view (e.g., UserIndex, Dashboard)
        name: String,
        /// Description of the desired UI for AI generation
        #[arg(long, short = 'd')]
        description: Option<String>,
        /// Skip AI generation, use static template
        #[arg(long)]
        no_ai: bool,
        /// Layout to use (default: app)
        #[arg(long, short = 'l')]
        layout: Option<String>,
    },
    /// Generate a new domain event
    #[command(name = "make:event")]
    MakeEvent {
        /// Name of the event (e.g., UserRegistered, OrderPlaced)
        name: String,
    },
    /// Generate a new test factory
    #[command(name = "make:factory")]
    MakeFactory {
        /// Name of the factory (e.g., User, Post)
        name: String,
    },
    /// Generate a new event listener
    #[command(name = "make:listener")]
    MakeListener {
        /// Name of the listener (e.g., SendWelcomeEmail, NotifyAdmin)
        name: String,
        /// Event type to listen to (optional)
        #[arg(long, short = 'e')]
        event: Option<String>,
    },
    /// Generate translation files for a new locale
    #[command(name = "make:lang")]
    MakeLang {
        /// Locale code (e.g., en, fr, de, pt-br)
        name: String,
    },
    /// Scaffold a new theme (tokens.css + theme.json)
    #[command(name = "make:theme")]
    MakeTheme {
        /// Theme name (e.g., ocean, corporate, midnight)
        name: String,
    },
    /// Generate a new background job
    #[command(name = "make:job")]
    MakeJob {
        /// Name of the job (e.g., ProcessPayment, SendEmail)
        name: String,
    },
    /// Generate a new notification
    #[command(name = "make:notification")]
    MakeNotification {
        /// Name of the notification (e.g., OrderShipped, WelcomeUser)
        name: String,
    },
    /// Generate a new database migration
    #[command(name = "make:migration")]
    MakeMigration {
        /// Name of the migration (e.g., create_users_table, add_email_to_users)
        name: String,
    },
    /// Generate a feature module skeleton (controller/model/views/routes)
    #[command(name = "make:module")]
    MakeModule {
        /// Name of the module (e.g., orders, user_profile)
        name: String,
        /// Also drop a migration stub under migration/src/
        #[arg(long)]
        with_migration: bool,
        /// Skip views/ subtree (headless module)
        #[arg(long)]
        no_views: bool,
        /// Overwrite existing files
        #[arg(long, short = 'f')]
        force: bool,
    },
    /// Generate a new authorization policy
    #[command(name = "make:policy")]
    MakePolicy {
        /// Name of the policy (e.g., Post, PostPolicy)
        name: String,
        /// Model name (defaults to name without "Policy" suffix)
        #[arg(long, short = 'm')]
        model: Option<String>,
    },
    /// Generate a service projection definition (ServiceDef)
    #[command(name = "make:projection")]
    MakeProjection {
        /// Name of the service (e.g., user, order, product)
        name: String,
        /// Populate fields from a matching SeaORM model in src/models/
        #[arg(long)]
        from_model: bool,
    },
    /// Validate service projections for structural issues
    #[cfg(feature = "projections")]
    #[command(name = "projection:check")]
    ProjectionCheck {
        /// Check a single projection by function name
        #[arg(long)]
        name: Option<String>,
    },
    /// Generate a new API resource
    #[command(name = "make:resource")]
    MakeResource {
        /// Name of the resource (e.g., UserResource, User)
        name: String,
        /// Model path for `From<Model>` generation (e.g., `entities::users::Model`)
        #[arg(long, short = 'm')]
        model: Option<String>,
    },
    /// Generate a new scheduled task
    #[command(name = "make:task")]
    MakeTask {
        /// Name of the task (e.g., CleanupLogs, SendReminders)
        name: String,
    },
    /// Generate a new database seeder
    #[command(name = "make:seeder")]
    MakeSeeder {
        /// Name of the seeder (e.g., Users, Products)
        name: String,
    },
    /// Generate a complete scaffold (model, migration, controller, views)
    #[command(name = "make:scaffold")]
    MakeScaffold {
        /// Name of the resource (e.g., Post, User)
        name: String,
        /// Fields in format field:type (e.g., title:string body:text published:bool)
        #[arg(trailing_var_arg = true)]
        fields: Vec<String>,
        /// Generate test file with CRUD test stubs
        #[arg(long)]
        with_tests: bool,
        /// Generate factory with field definitions
        #[arg(long)]
        with_factory: bool,
        /// Automatically register routes in src/routes.rs
        #[arg(long)]
        auto_routes: bool,
        /// Skip confirmation prompt for auto-routes (for CI/automation)
        #[arg(long, short = 'y')]
        yes: bool,
        /// Generate API-only scaffold (JSON responses, no Inertia pages)
        #[arg(long)]
        api: bool,
        /// Disable smart defaults detection (use explicit flags only)
        #[arg(long)]
        no_smart_defaults: bool,
        /// Suppress smart defaults summary output
        #[arg(long, short = 'q')]
        quiet: bool,
    },
    /// Run all pending database migrations
    #[command(name = "db:migrate")]
    DbMigrate,
    /// Rollback the last database migration(s)
    #[command(name = "db:rollback")]
    DbRollback {
        /// Number of migrations to rollback
        #[arg(long, default_value = "1")]
        step: u32,
    },
    /// Show the status of all migrations
    #[command(name = "db:status")]
    DbStatus,
    /// Drop all tables and re-run all migrations
    #[command(name = "db:fresh")]
    DbFresh,
    /// Run database seeders
    #[command(name = "db:seed")]
    DbSeed {
        /// Run only a specific seeder
        #[arg(long)]
        class: Option<String>,
    },
    /// Sync database schema to entity files (runs migrations + generates entities)
    #[command(name = "db:sync")]
    DbSync {
        /// Skip running migrations before sync
        #[arg(long)]
        skip_migrations: bool,
        /// Regenerate model files (overwrites existing custom models with new Eloquent-like API)
        #[arg(long)]
        regenerate_models: bool,
    },
    /// Execute a raw SQL query against the database
    #[command(name = "db:query")]
    DbQuery {
        /// SQL query to execute
        query: String,
    },
    /// Generate a production-ready Dockerfile, .dockerignore, and Cargo.docker.toml
    #[command(name = "docker:init")]
    DockerInit {
        /// Overwrite existing Dockerfile and .dockerignore
        #[arg(long)]
        force: bool,
        /// One-shot ferro version override (does not mutate Cargo.toml)
        #[arg(long)]
        ferro_version: Option<String>,
    },
    /// Generate DigitalOcean App Platform deployment spec (stub — Phase 122.2 Plan 07)
    #[command(name = "do:init")]
    DoInit {
        /// Overwrite existing .do/app.yaml
        #[arg(long)]
        force: bool,
    },
    /// Generate docker-compose.yml for local development
    #[command(name = "docker:compose")]
    DockerCompose {
        /// Include Mailpit email testing service
        #[arg(long)]
        with_mailpit: bool,
        /// Include MinIO S3-compatible storage service
        #[arg(long)]
        with_minio: bool,
    },
    /// Run all due scheduled tasks once (typically called by cron every minute)
    #[command(name = "schedule:run")]
    ScheduleRun,
    /// Start the scheduler daemon (runs continuously, checks every minute)
    #[command(name = "schedule:work")]
    ScheduleWork,
    /// List all registered scheduled tasks
    #[command(name = "schedule:list")]
    ScheduleList,
    /// Create a symbolic link from public/storage to storage/app/public
    #[command(name = "storage:link")]
    StorageLink {
        /// Create a relative symlink
        #[arg(long)]
        relative: bool,
    },
    /// Start the MCP server for AI-assisted development
    Mcp {
        /// Working directory for the project to introspect
        #[arg(long)]
        cwd: Option<String>,
    },
    /// Install AI development boost (MCP config + guidelines)
    #[command(name = "boost:install")]
    BoostInstall {
        /// Target editor: cursor, vscode, claude (auto-detected if omitted)
        #[arg(long)]
        editor: Option<String>,
    },
    /// Install Ferro Claude Code skills to ~/.claude/commands/ferro/
    #[command(name = "claude:install")]
    ClaudeInstall {
        /// Overwrite existing skill files
        #[arg(long, short = 'f')]
        force: bool,
        /// List available skills without installing
        #[arg(long, short = 'l')]
        list: bool,
    },
    /// Clean build artifacts
    Clean {
        /// Remove only artifacts older than N days (requires cargo-sweep)
        #[arg(long)]
        sweep: Option<u32>,
    },
    /// Validate Inertia frontend/backend prop contracts
    #[command(name = "validate:contracts")]
    ValidateContracts {
        /// Filter by route or component name
        #[arg(long, short = 'f')]
        filter: Option<String>,

        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },
    /// Generate a GitHub Actions CI workflow at .github/workflows/ci.yml
    #[command(name = "ci:init")]
    CiInit {
        /// Overwrite an existing workflow file
        #[arg(long)]
        force: bool,
    },
    /// Run health checks against the current Ferro project (D-01..D-09)
    Doctor {
        /// Emit machine-readable JSON instead of colored human output
        #[arg(long)]
        json: bool,
    },
    /// Check local API readiness for MCP integration
    #[command(name = "api:check")]
    ApiCheck {
        /// Base URL of the running API server
        #[arg(long, default_value = "http://localhost:8080")]
        url: String,
        /// API key to test authentication
        #[arg(long)]
        api_key: Option<String>,
        /// Path to the OpenAPI spec endpoint
        #[arg(long, default_value = "/api/openapi.json")]
        spec_path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            name,
            no_interaction,
            no_git,
        } => {
            commands::new::run(name, no_interaction, no_git);
        }
        Commands::Serve {
            port,
            frontend_port,
            backend_only,
            frontend_only,
            skip_types,
        } => {
            commands::serve::run(port, frontend_port, backend_only, frontend_only, skip_types);
        }
        Commands::GenerateTypes { output, watch } => {
            commands::generate_types::run(output, watch);
        }
        Commands::GenerateRoutes { output, json } => {
            if json {
                commands::generate_routes::run_json();
            } else {
                commands::generate_routes::run(output);
            }
        }
        Commands::MakeMiddleware { name } => {
            commands::make_middleware::run(name);
        }
        Commands::MakeController { name } => {
            commands::make_controller::run(name);
        }
        Commands::MakeAction { name } => {
            commands::make_action::run(name);
        }
        Commands::MakeApiKey { name, env } => {
            commands::make_api_key::run(name, env);
        }
        Commands::MakeApi {
            models,
            all,
            yes,
            exclude,
            include_all,
        } => {
            commands::make_api::run(models, all, yes, exclude, include_all);
        }
        Commands::MakeAuth { force } => {
            commands::make_auth::run(force);
        }
        Commands::MakeStripe { connect } => {
            commands::make_stripe::execute(connect);
        }
        Commands::MakeWhatsapp => {
            let project_root =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            commands::make_whatsapp::execute(&project_root);
        }
        Commands::MakeError { name } => {
            commands::make_error::run(name);
        }
        Commands::MakeInertia { name } => {
            commands::make_inertia::run(name);
        }
        Commands::MakeJsonView {
            name,
            description,
            no_ai,
            layout,
        } => {
            commands::make_json_view::run(name, description, no_ai, layout);
        }
        Commands::MakeEvent { name } => {
            commands::make_event::run(name);
        }
        Commands::MakeFactory { name } => {
            commands::make_factory::run(name);
        }
        Commands::MakeListener { name, event } => {
            commands::make_listener::run(name, event);
        }
        Commands::MakeLang { name } => {
            commands::make_lang::run(name);
        }
        Commands::MakeTheme { name } => {
            commands::make_theme::run(&name);
        }
        Commands::MakeJob { name } => {
            commands::make_job::run(name);
        }
        Commands::MakeNotification { name } => {
            commands::make_notification::run(name);
        }
        Commands::MakeMigration { name } => {
            commands::make_migration::run(name);
        }
        Commands::MakeModule {
            name,
            with_migration,
            no_views,
            force,
        } => {
            commands::make_module::run(name, with_migration, no_views, force);
        }
        Commands::MakePolicy { name, model } => {
            commands::make_policy::run(name, model);
        }
        Commands::MakeProjection { name, from_model } => {
            commands::make_projection::execute(&name, from_model);
        }
        #[cfg(feature = "projections")]
        Commands::ProjectionCheck { name } => {
            commands::projection_check::execute(name.as_deref());
        }
        Commands::MakeResource { name, model } => {
            commands::make_resource::run(name, model);
        }
        Commands::MakeTask { name } => {
            commands::make_task::run(name);
        }
        Commands::MakeSeeder { name } => {
            commands::make_seeder::run(name);
        }
        Commands::MakeScaffold {
            name,
            fields,
            with_tests,
            with_factory,
            auto_routes,
            yes,
            api,
            no_smart_defaults,
            quiet,
        } => {
            commands::make_scaffold::run(
                name,
                fields,
                with_tests,
                with_factory,
                auto_routes,
                yes,
                api,
                no_smart_defaults,
                quiet,
            );
        }
        Commands::DbMigrate => {
            commands::db_migrate::run();
        }
        Commands::DbRollback { step } => {
            commands::db_rollback::run(step);
        }
        Commands::DbStatus => {
            commands::db_status::run();
        }
        Commands::DbFresh => {
            commands::db_fresh::run();
        }
        Commands::DbSeed { class } => {
            commands::db_seed::run(class);
        }
        Commands::DbSync {
            skip_migrations,
            regenerate_models,
        } => {
            commands::db_sync::run(skip_migrations, regenerate_models);
        }
        Commands::DbQuery { query } => {
            commands::db_query::run(query);
        }
        Commands::DockerInit {
            force,
            ferro_version,
        } => {
            commands::docker_init::run_with(force, ferro_version);
        }
        Commands::DoInit { force } => {
            commands::do_init::run(force);
        }
        Commands::DockerCompose {
            with_mailpit,
            with_minio,
        } => {
            commands::docker_compose::run(with_mailpit, with_minio);
        }
        Commands::ScheduleRun => {
            commands::schedule_run::run();
        }
        Commands::ScheduleWork => {
            commands::schedule_work::run();
        }
        Commands::ScheduleList => {
            commands::schedule_list::run();
        }
        Commands::StorageLink { relative } => {
            commands::storage_link::run(relative);
        }
        Commands::Mcp { cwd } => {
            commands::mcp::run(cwd);
        }
        Commands::BoostInstall { editor } => {
            commands::boost_install::run(editor);
        }
        Commands::ClaudeInstall { force, list } => {
            commands::claude_install::run(force, list);
        }
        Commands::Clean { sweep } => {
            commands::clean::run(sweep);
        }
        Commands::ValidateContracts { filter, json } => {
            commands::validate_contracts::run(filter, json);
        }
        Commands::CiInit { force } => {
            commands::ci_init::run(force);
        }
        Commands::Doctor { json } => {
            commands::doctor::run(json);
        }
        Commands::ApiCheck {
            url,
            api_key,
            spec_path,
        } => {
            commands::api_check::run(url, api_key, spec_path);
        }
    }
}
