# CLI Reference

Ferro provides a powerful CLI tool for project scaffolding, code generation, database management, and development workflow automation.

## Installation

```bash
cargo install ferro-cli
```

Or build from source:

```bash
git clone https://github.com/albertogferrario/ferro
cd ferro/ferro-cli
cargo install --path .
```

## Project Commands

### `ferro new`

Create a new Ferro project with the complete directory structure.

```bash
# Interactive mode (prompts for project name)
ferro new

# Direct creation
ferro new my-app

# Skip git initialization
ferro new my-app --no-git

# Non-interactive mode (uses defaults)
ferro new my-app --no-interaction
```

**Options:**

| Option | Description |
|--------|-------------|
| `--no-interaction` | Skip prompts, use defaults |
| `--no-git` | Don't initialize git repository |

**Generated Structure:**

```
my-app/
├── src/
│   ├── main.rs
│   ├── bootstrap.rs
│   ├── routes.rs
│   ├── controllers/
│   │   └── mod.rs
│   ├── middleware/
│   │   ├── mod.rs
│   │   └── cors.rs
│   ├── models/
│   │   └── mod.rs
│   ├── migrations/
│   │   └── mod.rs
│   ├── events/
│   │   └── mod.rs
│   ├── listeners/
│   │   └── mod.rs
│   ├── jobs/
│   │   └── mod.rs
│   ├── notifications/
│   │   └── mod.rs
│   ├── tasks/
│   │   └── mod.rs
│   ├── seeders/
│   │   └── mod.rs
│   └── factories/
│       └── mod.rs
├── frontend/
│   ├── src/
│   │   ├── pages/
│   │   │   └── Home.tsx
│   │   ├── layouts/
│   │   │   └── Layout.tsx
│   │   ├── types/
│   │   │   └── inertia.d.ts
│   │   ├── app.tsx
│   │   └── main.tsx
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   └── tailwind.config.js
├── Cargo.toml
├── .env
├── .env.example
└── .gitignore
```

## Development Commands

### `ferro serve`

Start the development server with hot reloading for both backend and frontend.

```bash
# Start both backend and frontend
ferro serve

# Custom ports
ferro serve --port 8080 --frontend-port 5173

# Backend only (no frontend dev server)
ferro serve --backend-only

# Frontend only (no Rust compilation)
ferro serve --frontend-only

# Skip TypeScript type generation
ferro serve --skip-types
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--port` | `3000` | Backend server port |
| `--frontend-port` | `5173` | Frontend dev server port |
| `--backend-only` | `false` | Run only the backend |
| `--frontend-only` | `false` | Run only the frontend |
| `--skip-types` | `false` | Don't regenerate TypeScript types |

**What it does:**

1. Starts the Rust backend with `cargo watch` for hot reloading
2. Starts the Vite frontend dev server
3. Watches Rust files to regenerate TypeScript types automatically
4. Proxies frontend requests to the backend

### `ferro generate-types`

Generate TypeScript type definitions from Rust `InertiaProps` structs.

```bash
ferro generate-types
```

This scans your Rust code for structs deriving `InertiaProps` and generates corresponding TypeScript interfaces in `frontend/src/types/`.

## Code Generators

All generators follow the pattern `ferro make:<type> <name> [options]`.

### `ferro make:controller`

Generate a controller with handler methods.

```bash
# Basic controller
ferro make:controller UserController

# Resource controller with CRUD methods
ferro make:controller PostController --resource

# API controller (JSON responses)
ferro make:controller Api/ProductController --api
```

**Options:**

| Option | Description |
|--------|-------------|
| `--resource` | Generate index, show, create, store, edit, update, destroy methods |
| `--api` | Generate API-style controller (JSON responses) |

**Generated file:** `src/controllers/user_controller.rs`

```rust
use ferro::{handler, Request, Response, json_response};

#[handler]
pub async fn index(req: Request) -> Response {
    // TODO: Implement
    json_response!({ "message": "index" })
}

#[handler]
pub async fn show(req: Request) -> Response {
    // TODO: Implement
    json_response!({ "message": "show" })
}

// ... additional methods for --resource
```

### `ferro make:middleware`

Generate middleware for request/response processing.

```bash
ferro make:middleware Auth
ferro make:middleware RateLimit
```

**Generated file:** `src/middleware/auth.rs`

```rust
use ferro::{Middleware, Request, Response, Next};
use async_trait::async_trait;

pub struct Auth;

#[async_trait]
impl Middleware for Auth {
    async fn handle(&self, request: Request, next: Next) -> Response {
        // TODO: Implement middleware logic
        next.run(request).await
    }
}
```

### `ferro make:action`

Generate a single-action class for complex business logic.

```bash
ferro make:action CreateOrder
ferro make:action ProcessPayment
```

**Generated file:** `src/actions/create_order.rs`

```rust
use ferro::FrameworkError;

pub struct CreateOrder;

impl CreateOrder {
    pub async fn execute(&self) -> Result<(), FrameworkError> {
        // TODO: Implement action
        Ok(())
    }
}
```

### `ferro make:event`

Generate an event struct for the event dispatcher.

```bash
ferro make:event UserRegistered
ferro make:event OrderPlaced
```

**Generated file:** `src/events/user_registered.rs`

```rust
use ferro_events::Event;

#[derive(Debug, Clone, Event)]
pub struct UserRegistered {
    pub user_id: i64,
}
```

### `ferro make:listener`

Generate a listener that responds to events.

```bash
ferro make:listener SendWelcomeEmail
ferro make:listener NotifyAdmins
```

**Generated file:** `src/listeners/send_welcome_email.rs`

```rust
use ferro_events::{Listener, Event};
use async_trait::async_trait;

pub struct SendWelcomeEmail;

#[async_trait]
impl<E: Event + Send + Sync> Listener<E> for SendWelcomeEmail {
    async fn handle(&self, event: &E) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement listener
        Ok(())
    }
}
```

### `ferro make:job`

Generate a background job for queue processing.

```bash
ferro make:job ProcessImage
ferro make:job SendEmail
```

**Generated file:** `src/jobs/process_image.rs`

```rust
use ferro_queue::{Job, JobContext};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessImage {
    pub image_id: i64,
}

#[async_trait]
impl Job for ProcessImage {
    async fn handle(&self, ctx: &JobContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement job
        Ok(())
    }
}
```

### `ferro make:notification`

Generate a multi-channel notification.

```bash
ferro make:notification OrderShipped
ferro make:notification InvoiceGenerated
```

**Generated file:** `src/notifications/order_shipped.rs`

```rust
use ferro_notifications::{Notification, Notifiable, Channel};

pub struct OrderShipped {
    pub order_id: i64,
}

impl Notification for OrderShipped {
    fn via(&self) -> Vec<Channel> {
        vec![Channel::Mail, Channel::Database]
    }
}
```

### `ferro make:migration`

Generate a database migration file.

```bash
ferro make:migration create_posts_table
ferro make:migration add_status_to_orders
```

**Generated file:** `src/migrations/m20240115_143052_create_posts_table.rs`

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // TODO: Implement migration
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // TODO: Implement rollback
        Ok(())
    }
}
```

### `ferro make:inertia`

Generate an Inertia.js page component with TypeScript types.

```bash
ferro make:inertia Dashboard
ferro make:inertia Users/Profile
```

**Generated files:**
- `frontend/src/pages/Dashboard.tsx`
- `src/controllers/` (props struct)

### `ferro make:json-view`

Generate a JSON-UI view file with AI-powered component generation.

```bash
# Generate a view (AI-powered if ANTHROPIC_API_KEY is set)
ferro make:json-view UserIndex

# Generate with a description for AI context
ferro make:json-view UserEdit --description "Edit form for user profile"

# Specify a layout
ferro make:json-view Login --layout auth

# Skip AI generation, use static template
ferro make:json-view Dashboard --no-ai
```

**Options:**

| Option | Description |
|--------|-------------|
| `--description`, `-d` | Description of the desired UI for AI generation |
| `--layout`, `-l` | Layout to use (default: `app`) |
| `--no-ai` | Skip AI generation, use static template |

**How it works:**

1. Scans your models and routes for project context
2. Sends context to Anthropic API (Claude Sonnet) for intelligent view generation
3. Falls back to a static template if no API key is configured or AI generation fails
4. Generates a view file in `src/views/` and updates `mod.rs`

**Requirements:**

- Set `ANTHROPIC_API_KEY` in your environment for AI-powered generation
- Without the key, a static template with common components is generated
- Model override via `FERRO_AI_MODEL` environment variable

**Generated file:** `src/views/user_index.rs`

```rust
use ferro::{
    Action, Component, ComponentNode, JsonUiView, TableColumn, TableProps,
    TextElement, TextProps,
};

pub fn view() -> JsonUiView {
    JsonUiView::new()
        .title("User Index")
        .layout("app")
        .component(ComponentNode {
            key: "heading".to_string(),
            component: Component::Text(TextProps {
                content: "User Index".to_string(),
                element: TextElement::H1,
            }),
            action: None,
            visibility: None,
        })
        // ... additional components based on AI context or static template
}
```

### `ferro make:task`

Generate a scheduled task.

```bash
ferro make:task CleanupExpiredSessions
ferro make:task SendDailyReport
```

**Generated file:** `src/tasks/cleanup_expired_sessions.rs`

```rust
use ferro::scheduling::{Task, Schedule};
use async_trait::async_trait;

pub struct CleanupExpiredSessions;

#[async_trait]
impl Task for CleanupExpiredSessions {
    fn schedule(&self) -> Schedule {
        Schedule::daily()
    }

    async fn handle(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement task
        Ok(())
    }
}
```

### `ferro make:seeder`

Generate a database seeder.

```bash
ferro make:seeder UserSeeder
ferro make:seeder ProductSeeder
```

**Generated file:** `src/seeders/user_seeder.rs`

```rust
use ferro::database::Seeder;
use async_trait::async_trait;

pub struct UserSeeder;

#[async_trait]
impl Seeder for UserSeeder {
    async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Seed data
        Ok(())
    }
}
```

### `ferro make:factory`

Generate a model factory for testing.

```bash
ferro make:factory UserFactory
ferro make:factory PostFactory
```

**Generated file:** `src/factories/user_factory.rs`

```rust
use ferro::testing::Factory;
use fake::{Fake, Faker};

pub struct UserFactory;

impl Factory for UserFactory {
    type Model = user::Model;

    fn definition(&self) -> Self::Model {
        // TODO: Define factory
        todo!()
    }
}
```

### `ferro make:error`

Generate a custom error type.

```bash
ferro make:error PaymentFailed
ferro make:error ValidationError
```

### `ferro make:scaffold`

Generate a complete CRUD scaffold: model, migration, controller, and Inertia pages.

```bash
# Basic scaffold
ferro make:scaffold Post

# With field definitions
ferro make:scaffold Post title:string content:text published:bool

# Complex example
ferro make:scaffold Product name:string description:text price:float stock:integer
```

**Field Types:**

| Type | Rust Type | Database Type |
|------|-----------|---------------|
| `string` | `String` | `VARCHAR(255)` |
| `text` | `String` | `TEXT` |
| `integer` | `i32` | `INTEGER` |
| `bigint` | `i64` | `BIGINT` |
| `float` | `f64` | `DOUBLE` |
| `bool` | `bool` | `BOOLEAN` |
| `datetime` | `DateTime` | `TIMESTAMP` |
| `date` | `Date` | `DATE` |
| `uuid` | `Uuid` | `UUID` |

**Generated Files:**

```
src/
├── models/post.rs           # SeaORM entity
├── migrations/m*_create_posts_table.rs
├── controllers/post_controller.rs
frontend/src/pages/
├── posts/
│   ├── Index.tsx
│   ├── Show.tsx
│   ├── Create.tsx
│   └── Edit.tsx
```

## Database Commands

### `ferro db:migrate`

Run all pending migrations.

```bash
ferro db:migrate
```

### `ferro db:rollback`

Rollback the last batch of migrations.

```bash
ferro db:rollback
```

### `ferro db:status`

Show the status of all migrations.

```bash
ferro db:status
```

Output:

```
+------+------------------------------------------------+-------+
| Ran? | Migration                                       | Batch |
+------+------------------------------------------------+-------+
| Yes  | m20240101_000001_create_users_table            | 1     |
| Yes  | m20240101_000002_create_posts_table            | 1     |
| No   | m20240115_143052_add_status_to_posts           |       |
+------+------------------------------------------------+-------+
```

### `ferro db:fresh`

Drop all tables and re-run all migrations.

```bash
ferro db:fresh
```

**Warning:** This is destructive and will delete all data.

### `ferro db:sync`

Synchronize the database schema and generate entity files.

```bash
# Sync entities from existing database
ferro db:sync

# Run migrations first, then sync
ferro db:sync --migrate
```

**Options:**

| Option | Description |
|--------|-------------|
| `--migrate` | Run pending migrations before syncing |

This command:
1. Discovers the database schema (tables, columns, types)
2. Generates SeaORM entity files in `src/models/entities/`
3. Creates user-friendly model wrappers with the Ferro Model API

### `ferro db:query`

Execute a raw SQL query against the database.

```bash
# Simple SELECT query
ferro db:query "SELECT * FROM users LIMIT 5"

# Query with conditions
ferro db:query "SELECT id, name, email FROM users WHERE active = true"

# Count query
ferro db:query "SELECT COUNT(*) FROM posts"
```

**Features:**

- Reads `DATABASE_URL` from `.env` file
- Supports SQLite and PostgreSQL databases
- Displays results in a formatted table
- Handles NULL values gracefully
- Shows row count after results

**Example Output:**

```
+-----+-------+-------------------+
| 1   | Alice | alice@example.com |
| 2   | Bob   | bob@example.com   |
+-----+-------+-------------------+

→ 2 row(s)
```

**Use Cases:**

- Quick data inspection during development
- Debugging database state
- Verifying migration results
- Ad-hoc queries without external tools

## Docker Commands

### `ferro docker:init`

Initialize Docker configuration files.

```bash
ferro docker:init
```

**Generated files:**
- `Dockerfile`
- `docker-compose.yml`
- `.dockerignore`

### `ferro docker:compose`

Manage Docker Compose services.

```bash
# Start services
ferro docker:compose up

# Stop services
ferro docker:compose down

# Rebuild and start
ferro docker:compose up --build
```

## Scheduling Commands

### `ferro schedule:run`

Run scheduled tasks that are due.

```bash
ferro schedule:run
```

This executes all tasks whose schedule indicates they should run now. Typically called by a system cron job every minute:

```cron
* * * * * cd /path/to/project && ferro schedule:run >> /dev/null 2>&1
```

### `ferro schedule:work`

Start the scheduler worker for continuous task execution.

```bash
ferro schedule:work
```

This runs in the foreground and checks for due tasks every minute. Useful for development or container deployments.

### `ferro schedule:list`

Display all registered scheduled tasks.

```bash
ferro schedule:list
```

Output:

```
+---------------------------+-------------+-------------------+
| Task                      | Schedule    | Next Run          |
+---------------------------+-------------+-------------------+
| CleanupExpiredSessions    | Daily 00:00 | 2024-01-16 00:00  |
| SendDailyReport           | Daily 09:00 | 2024-01-15 09:00  |
| PruneOldNotifications     | Weekly Mon  | 2024-01-22 00:00  |
+---------------------------+-------------+-------------------+
```

## Storage Commands

### `ferro storage:link`

Create a symbolic link from `public/storage` to `storage/app/public`.

```bash
ferro storage:link
```

This allows publicly accessible files stored in `storage/app/public` to be served via the web server.

## AI-Assisted Development

### `ferro mcp`

Start the Model Context Protocol (MCP) server for AI-assisted development.

```bash
ferro mcp
```

The MCP server provides introspection tools that help AI assistants understand your Ferro application structure, including routes, models, controllers, and configuration.

### `ferro boost:install`

Install AI development boost features.

```bash
ferro boost:install
```

This sets up configuration for enhanced AI-assisted development workflows.

### `ferro claude:install`

Install Ferro Claude Code skills to enable `/ferro:*` slash commands in Claude Code.

```bash
# Install all skills
ferro claude:install

# Force overwrite existing skills
ferro claude:install --force

# List available skills without installing
ferro claude:install --list
```

**Options:**

| Option | Description |
|--------|-------------|
| `--force`, `-f` | Overwrite existing skill files |
| `--list`, `-l` | List available skills without installing |

**Installed Location:** `~/.claude/commands/ferro/`

**Available Skills:**

| Command | Description |
|---------|-------------|
| `/ferro:help` | Show all available Ferro commands |
| `/ferro:info` | Display project information |
| `/ferro:routes` | List all registered routes |
| `/ferro:route:explain` | Explain a specific route in detail |
| `/ferro:model` | Generate a new model with migration |
| `/ferro:models` | List all models with fields |
| `/ferro:controller` | Generate a new controller |
| `/ferro:middleware` | Generate new middleware |
| `/ferro:db` | Database operations (migrate, rollback, seed) |
| `/ferro:test` | Run tests with coverage options |
| `/ferro:serve` | Start the development server |
| `/ferro:new` | Create a new Ferro project |
| `/ferro:tinker` | Interactive database REPL |
| `/ferro:diagnose` | Diagnose errors using MCP introspection |

Skills leverage ferro-mcp for intelligent code generation and project introspection.

## Command Summary

| Command | Description |
|---------|-------------|
| `new` | Create a new Ferro project |
| `serve` | Start development server |
| `generate-types` | Generate TypeScript types |
| `make:controller` | Create a controller |
| `make:middleware` | Create middleware |
| `make:action` | Create an action class |
| `make:event` | Create an event |
| `make:listener` | Create a listener |
| `make:job` | Create a background job |
| `make:notification` | Create a notification |
| `make:migration` | Create a migration |
| `make:inertia` | Create an Inertia page |
| `make:json-view` | Create a JSON-UI view (AI-powered) |
| `make:task` | Create a scheduled task |
| `make:seeder` | Create a database seeder |
| `make:factory` | Create a model factory |
| `make:error` | Create a custom error |
| `make:policy` | Create an authorization policy |
| `make:scaffold` | Create complete CRUD scaffold |
| `db:migrate` | Run migrations |
| `db:rollback` | Rollback migrations |
| `db:status` | Show migration status |
| `db:fresh` | Fresh migrate (drop all) |
| `db:sync` | Sync database schema |
| `db:query` | Execute raw SQL query |
| `docker:init` | Initialize Docker files |
| `docker:compose` | Manage Docker Compose |
| `schedule:run` | Run due scheduled tasks |
| `schedule:work` | Start scheduler worker |
| `schedule:list` | List scheduled tasks |
| `storage:link` | Create storage symlink |
| `mcp` | Start MCP server |
| `boost:install` | Install AI boost features |
| `claude:install` | Install Claude Code skills |

## Environment Variables

The CLI respects these environment variables:

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | Database connection string |
| `APP_ENV` | Application environment (development, production) |
| `RUST_LOG` | Logging level |
