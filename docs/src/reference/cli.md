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

Start the development server for both backend and frontend. Auto-reload is opt-in via `--watch`; without it, the `r` key triggers rebuilds on demand.

```bash
# Start both backend and frontend. No file watching; press r to rebuild on demand.
ferro serve

# Enable file-watch auto-reload with a 500ms trailing-edge debounce.
ferro serve --watch

# Custom ports.
ferro serve --port 8080 --frontend-port 5173

# Backend only (no frontend dev server).
ferro serve --backend-only

# Frontend only (no Rust compilation).
ferro serve --frontend-only

# Skip TypeScript type generation.
ferro serve --skip-types
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--port` | `8080` | Backend server port |
| `--frontend-port` | `5173` | Frontend dev server port |
| `--backend-only` | `false` | Run only the backend |
| `--frontend-only` | `false` | Run only the frontend |
| `--skip-types` | `false` | Skip TypeScript type generation |
| `--watch` | `false` | Enable file-watch auto-reload (500ms debounce) |

**Key bindings (when stdin is a TTY):**

| Key | Action |
|-----|--------|
| `r` | Rebuild the backend and regenerate types. If a build is in flight, it is cancelled and restarted. |
| `q` or Ctrl-C | Graceful shutdown. |

When stdin is not a TTY (for example, when `ferro serve` is piped or run under a process supervisor), the `r` key is unavailable and the banner says so; Ctrl-C still works.

**What it does:**

1. Starts the Rust backend via an in-process supervisor that owns the `cargo run` child directly. Auto-reload is opt-in via `--watch`; without it, the supervisor waits for the `r` key to trigger a rebuild.
2. Starts the Vite frontend dev server.
3. With `--watch`, watches `*.rs` files under `src/` with a 500ms trailing-edge debounce; a burst of saves produces one rebuild after the burst settles.
4. On every rebuild (manual or file-triggered), regenerates TypeScript types from Rust `InertiaProps` structs.
5. Auto-resolves port conflicts: if the frontend port is in use, the next available port is selected and propagated to the backend via `VITE_DEV_SERVER`.

### `ferro generate-types`

Generate TypeScript type definitions from Rust `InertiaProps` structs.

```bash
ferro generate-types
```

This scans your Rust code for structs deriving `InertiaProps` and generates corresponding TypeScript interfaces in `frontend/src/types/`.

> **Includes route generation:** `generate-types` also runs route generation internally, producing TypeScript route helpers alongside the type definitions.

### `ferro clean`

Remove build artifacts from the project.

```bash
# Remove all build artifacts (runs cargo clean)
ferro clean

# Remove only artifacts older than N days (requires cargo-sweep)
ferro clean --sweep 7
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--sweep <days>` | — | Remove only artifacts older than N days. Requires `cargo install cargo-sweep`. |

**What it does:**

1. Without `--sweep`: runs `cargo clean`, removing the entire `target/` directory.
2. With `--sweep <days>`: invokes `cargo sweep --maxage <days>` to remove only stale artifacts, preserving recently compiled dependencies.

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
    json_response!({ "message": "index" })
}

#[handler]
pub async fn show(req: Request) -> Response {
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
        tracing::info!("executing resource action");
        Ok(())
    }
}
```

### `ferro make:auth`

Scaffold a complete authentication system with migration, controller, and setup instructions.

```bash
# Generate auth scaffolding
ferro make:auth

# Force overwrite existing files
ferro make:auth --force
```

**Options:**

| Option | Description |
|--------|-------------|
| `--force`, `-f` | Overwrite existing auth controller and migration |

**Generated Files:**

- `src/migrations/m{timestamp}_add_auth_fields_to_users.rs` -- ALTER TABLE migration adding `password`, `remember_token`, and `email_verified_at` fields to the existing users table
- `src/controllers/auth_controller.rs` -- Controller with `register`, `login`, and `logout` handlers

**What it does:**

1. Generates an ALTER TABLE migration (assumes `users` table already exists)
2. Creates an auth controller with register/login/logout handlers
3. Registers the controller module in `src/controllers/mod.rs`
4. Prints setup instructions for the auth provider and route registration

The command uses an ALTER TABLE approach because most projects already have a users table with basic fields. The migration adds only the authentication-specific columns.

**See also:** [Authentication guide](../authentication.md) for the complete auth setup walkthrough.

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
        tracing::info!("handling event");
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
        tracing::info!(id = self.image_id, "processing item");
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

#[derive(DeriveIden)]
enum Item {
    Table,
    Id,
    Name,
    CreatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Item::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Item::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Item::Name).string().not_null())
                    .col(ColumnDef::new(Item::CreatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Item::Table).to_owned())
            .await
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

**Generated file:** `src/views/user_index.json`

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "User Index",
  "layout": "dashboard",
  "root": "root",
  "elements": {
    "root": {
      "type": "Card",
      "props": {
        "title": "User Index",
        "description": "Edit src/views/user_index.json to customize this view."
      },
      "children": ["heading"]
    },
    "heading": {
      "type": "Text",
      "props": { "content": "User Index", "element": "h1" }
    }
  }
}
```

**Generated handler usage:**

```rust
#[handler]
pub async fn user_index(req: Request) -> Response {
    let data = serde_json::json!({});
    JsonUi::render_file("views/user_index.json", data)
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
use ferro::{Task, Schedule};
use async_trait::async_trait;

pub struct CleanupExpiredSessions;

#[async_trait]
impl Task for CleanupExpiredSessions {
    fn schedule(&self) -> Schedule {
        Schedule::daily()
    }

    async fn handle(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("running scheduled task");
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
use ferro::Seeder;
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
use ferro::Factory;
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

### `ferro make:resource`

Generate an API resource for transforming models into structured JSON responses.

```bash
# Basic resource (generates struct with placeholder fields)
ferro make:resource UserResource

# Auto-append "Resource" suffix if not present
ferro make:resource User

# With model path for automatic From<Model> implementation
ferro make:resource UserResource --model entities::users::Model
```

**Options:**

| Option | Description |
|--------|-------------|
| `--model`, `-m` | Model path for auto-generating `From<Model>` implementation |

**Generated file:** `src/resources/user_resource.rs`

```rust
use ferro::{ApiResource, Resource, ResourceMap, Request};

#[derive(ApiResource)]
pub struct UserResource {
    pub id: i32,
    // Add fields from your model here
    // #[resource(rename = "display_name")]
    // pub name: String,
    // #[resource(skip)]
    // pub password_hash: String,
}
```

**Field attributes:**

| Attribute | Description |
|-----------|-------------|
| `#[resource(rename = "name")]` | Rename field in JSON output |
| `#[resource(skip)]` | Exclude field from JSON output |

The `#[derive(ApiResource)]` macro generates the `Resource` trait implementation, which provides `to_resource()` and `to_json()` methods. When `--model` is specified, a `From<Model>` implementation is generated to map model fields to resource fields.

**After generation**, add `pub mod user_resource;` to `src/resources/mod.rs`.

**See also:** [API Resources guide](../api-resources.md) for the complete resource system documentation.

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

### `ferro make:api`

Generate REST API endpoints for a set of models.

```bash
# Generate API for specific models
ferro make:api User Post

# Generate API for all detected models
ferro make:api --all

# Skip confirmation prompt and exclude specific fields
ferro make:api User --yes --exclude password_hash,secret_token

# Include all fields, disabling auto-exclusion of sensitive field names
ferro make:api User --include-all
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `models` | — | One or more model names (positional) |
| `--all` | `false` | Generate API for all detected models |
| `--yes`, `-y` | `false` | Skip confirmation prompt |
| `--exclude` | — | Comma-delimited list of field names to exclude from generated endpoints |
| `--include-all` | `false` | Disable auto-exclusion of sensitive field names |

**What it does:**

1. Detects models in `src/models/` (or uses the provided list)
2. Generates controller, resource, and route entries for each model
3. Automatically excludes fields matching sensitive patterns: `password`, `secret`, `token`, `hash`, `key`, `salt`, `private`, `credentials`
4. `--exclude` accepts comma-delimited values (e.g. `password_hash,secret_token`)
5. `--include-all` disables the auto-exclusion list entirely

### `ferro make:api-key`

Generate an API key with SHA-256 hashing.

```bash
# Generate a live API key
ferro make:api-key "Production Bot"

# Generate a test key
ferro make:api-key "CI Testing" --env test
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `name` | — | Display name for the key (positional, required) |
| `--env` | `live` | Key environment: `live` or `test` |

**What it does:**

1. Generates a cryptographically random API key prefixed with `fe_<env>_`
2. Hashes the key with SHA-256 (the hash is what gets stored)
3. Prints the raw key once — it is not stored in plaintext and cannot be recovered
4. Outputs a ready-to-run SQL INSERT statement and a Rust snippet for verifying keys in handlers

### `ferro make:lang`

Create translation files for a locale.

```bash
ferro make:lang fr
ferro make:lang pt-br
```

**Options:**

| Option | Description |
|--------|-------------|
| `name` | BCP 47 locale tag (positional, required; e.g. `en`, `fr`, `pt-br`, `zh-hans`) |

**Generated files:**

- `lang/<locale>/validation.json` — Validation error message translations
- `lang/<locale>/app.json` — Application-level string translations

### `ferro make:policy`

Create an authorization policy for a model.

```bash
ferro make:policy Post
ferro make:policy PostPolicy --model Post
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `name` | — | Policy name (positional, required) |
| `--model`, `-m` | Name without `Policy` suffix | The model this policy guards |

**Generated file:** `src/policies/<name>_policy.rs`

```rust
pub struct PostPolicy;

impl PostPolicy {
    pub fn view(&self, user: &User, post: &Post) -> bool {
        true
    }

    pub fn create(&self, user: &User) -> bool {
        true
    }

    pub fn update(&self, user: &User, post: &Post) -> bool {
        user.id == post.user_id
    }

    pub fn delete(&self, user: &User, post: &Post) -> bool {
        user.id == post.user_id
    }
}
```

### `ferro make:projection`

Create a service projection definition.

```bash
ferro make:projection user
ferro make:projection order --from-model
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `name` | — | Projection name (positional, required) |
| `--from-model` | `false` | Populate fields from the matching SeaORM model in `src/models/` |

**Generated file:** `src/projections/<name>.rs`

### `ferro make:stripe`

Scaffold Stripe billing integration.

```bash
# Basic Stripe integration
ferro make:stripe

# Include Stripe Connect scaffolding
ferro make:stripe --connect
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--connect` | `false` | Include Stripe Connect webhook and connect account ID field |

**Generated files:**

- `src/stripe/mod.rs` — Stripe module root
- `src/stripe/webhook.rs` — Stripe webhook handler
- `src/stripe/listeners.rs` — Stripe event listeners
- `src/stripe/connect_webhook.rs` — Connect webhook handler (with `--connect`)
- `src/migrations/m<timestamp>_create_tenant_billing_table.rs` — Billing table migration (with `--connect`)

### `ferro make:theme`

Create a JSON-UI theme with Tailwind v4 semantic tokens.

```bash
ferro make:theme ocean
ferro make:theme corporate
```

**Options:**

| Option | Description |
|--------|-------------|
| `name` | Theme name (positional, required) |

**Generated files:**

- `themes/<name>/tokens.css` — Tailwind v4 `@theme` block with 23 semantic token slots
- `themes/<name>/theme.json` — Empty JSON object for intent template overrides

The `tokens.css` file uses the Tailwind v4 `@theme` directive and defines semantic color, typography, and shape tokens. The `theme.json` file is a placeholder for overriding how structural intents (Browse, Focus, Collect, etc.) render for this theme.

### `ferro make:whatsapp`

Scaffold WhatsApp Cloud API integration.

```bash
ferro make:whatsapp
```

No flags. Generates a complete WhatsApp webhook and listener scaffold.

**Generated files:**

- `src/whatsapp/mod.rs` — WhatsApp module root
- `src/whatsapp/webhook.rs` — WhatsApp webhook handler
- `src/whatsapp/listeners.rs` — WhatsApp message listeners

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

### `ferro db:seed`

Run database seeders to populate the database with test data.

```bash
# Run all seeders
ferro db:seed

# Run a specific seeder
ferro db:seed --class UserSeeder
```

**Options:**

| Option | Description |
|--------|-------------|
| `--class` | Run only a specific seeder by name |

**How it works:**

The command delegates to `cargo run -- db:seed` in your project, which executes the registered seeders. Seeders are Rust structs implementing the `Seeder` trait, located in `src/seeders/`.

If no seeders directory exists, the command will prompt you to create one with `ferro make:seeder <name>`.

**See also:** [`ferro make:seeder`](#ferro-makeseeder) to generate new seeder files.

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

## Deployment Commands

### `ferro do:init`

Initialize DigitalOcean App Platform deployment. Generates `.do/app.yaml` only
— CI and Dockerfile scaffolding are handled by separate commands (`ci:init`,
`docker:init`).

```bash
ferro do:init           # write .do/app.yaml if missing
ferro do:init --force   # regenerate an existing spec
```

The GitHub repo is auto-detected from `git remote get-url origin`; the region
is hardcoded to `fra1` in the template (edit the file afterwards to change
it). The command reads `.env.production` to enumerate env keys for a
commented-out scaffold; if `.env.production` is missing the command fails
hard.

**Generated file:**
- `.do/app.yaml` — App Platform spec with sanitized app name, auto-detected
  `github.repo`, `region: fra1`, a `services:` web entry, a `workers:` entry
  per non-test `[[bin]]`, and a commented-out `envs:` scaffold. No
  `databases:` block is emitted.

**Options:**
- `--force`, `-f` — Overwrite an existing `.do/app.yaml`.

See [`do:init`](../cli/do-init.md) for the full page.

## Docker Commands

### `ferro docker:init`

Generate a production-ready Dockerfile and `.dockerignore`. The Docker build
consumes the project `Cargo.toml` directly — no dual manifest is written.
Developers working against an unpublished ferro checkout maintain an
uncommitted `[patch.crates-io]` block in their project `Cargo.toml`.

```bash
ferro docker:init          # write Dockerfile + .dockerignore
ferro docker:init --force  # overwrite existing files
```

**Options:**
- `--force`, `-f` — Overwrite existing `Dockerfile` / `.dockerignore`.

**Generated files:**
- `Dockerfile` — multi-stage build, base image defaults to
  `rust:slim-bookworm`.
- `.dockerignore`

**`[package.metadata.ferro.deploy]` in `Cargo.toml`:**

`docker:init` reads optional deploy metadata from the project's `Cargo.toml`:

```toml
[package.metadata.ferro.deploy]
runtime_apt = ["libpq5", "ca-certificates"]
copy_dirs = ["migrations", "templates"]
```

- `runtime_apt` — additional apt packages installed into the runtime stage.
- `copy_dirs` — extra directories copied from the builder into the runtime
  image.

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

## Validation & Diagnostics

### `ferro api:check`

Verify that the local API server is running and MCP-compatible.

```bash
# Check local API server for MCP integration
ferro api:check

# Custom URL and API key
ferro api:check --url http://localhost:8080 --api-key fe_live_xxx

# Custom OpenAPI spec path
ferro api:check --spec-path /api/docs/openapi.json
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--url` | `http://localhost:8080` | Base URL of the running API server |
| `--api-key` | — | API key for testing authenticated endpoints |
| `--spec-path` | `/api/openapi.json` | Path to the OpenAPI spec endpoint |

**What it does:**

1. Checks server reachability by sending a GET request to `--url`
2. Fetches the OpenAPI spec from `<url><spec-path>`
3. Validates the spec structure (version, paths, info object)
4. Tests API key authentication if `--api-key` is provided
5. Prints `ferro-api-mcp` configuration for connecting AI tools to this server

### `ferro projection:check`

Validate service projection definitions.

> **Requires the `projections` feature.** Build with `cargo build --features projections` or add `projections` to the `default` features in `Cargo.toml` before running this command.

```bash
# Validate all projections
ferro projection:check

# Validate a single projection by function name
ferro projection:check --name user_service
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--name` | — | Check only the named projection function |

**What it does:**

1. Discovers all projection definitions in `src/projections/`
2. Validates that each projection's field types, intent hints, and visibility rules are well-formed
3. Reports structural errors and type mismatches with file and line references

### `ferro validate:contracts`

Validate Inertia TypeScript contracts against Rust handler props.

```bash
# Validate all contracts
ferro validate:contracts

# Filter by route prefix
ferro validate:contracts --filter /users

# JSON output for CI integration
ferro validate:contracts --json
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--filter`, `-f` | — | Only validate routes matching this prefix |
| `--json` | `false` | Output results as JSON (useful for CI) |

**What it does:**

1. Compares Rust `InertiaProps` structs with the generated TypeScript interfaces in `frontend/src/types/`
2. Reports field name mismatches, type incompatibilities, and missing optional fields
3. With `--json`, outputs a machine-readable result for use in CI pipelines

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
| `clean` | Remove build artifacts |
| `make:action` | Create an action class |
| `make:api` | Generate REST API endpoints for models |
| `make:api-key` | Generate an API key |
| `make:auth` | Scaffold authentication system |
| `make:controller` | Create a controller |
| `make:error` | Create a custom error |
| `make:event` | Create an event |
| `make:factory` | Create a model factory |
| `make:inertia` | Create an Inertia page |
| `make:job` | Create a background job |
| `make:json-view` | Create a JSON-UI view (AI-powered) |
| `make:lang` | Create translation files for a locale |
| `make:listener` | Create a listener |
| `make:middleware` | Create middleware |
| `make:migration` | Create a migration |
| `make:notification` | Create a notification |
| `make:policy` | Create an authorization policy |
| `make:projection` | Create a service projection |
| `make:resource` | Create an API resource |
| `make:scaffold` | Create complete CRUD scaffold |
| `make:seeder` | Create a database seeder |
| `make:stripe` | Scaffold Stripe billing integration |
| `make:task` | Create a scheduled task |
| `make:theme` | Create a JSON-UI theme |
| `make:whatsapp` | Scaffold WhatsApp integration |
| `db:migrate` | Run migrations |
| `db:rollback` | Rollback migrations |
| `db:status` | Show migration status |
| `db:fresh` | Fresh migrate (drop all) |
| `db:seed` | Run database seeders |
| `db:sync` | Sync database schema |
| `db:query` | Execute raw SQL query |
| `do:init` | Generate DigitalOcean App Platform spec (`.do/app.yaml`) |
| `ci:init` | Generate GitHub Actions CI workflow (`.github/workflows/ci.yml`) |
| `doctor` | Run project health diagnostics (eleven checks) |
| `docker:init` | Generate Dockerfile and .dockerignore |
| `docker:compose` | Manage Docker Compose |
| `schedule:run` | Run due scheduled tasks |
| `schedule:work` | Start scheduler worker |
| `schedule:list` | List scheduled tasks |
| `storage:link` | Create storage symlink |
| `api:check` | Verify API server and MCP integration |
| `projection:check` | Validate service projection definitions |
| `validate:contracts` | Validate Inertia TypeScript contracts |
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
