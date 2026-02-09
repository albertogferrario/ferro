# AI Agent Instructions

Instructions for AI agents working with the Ferro framework codebase.

## Quick Start: Use ferro-mcp

**Before exploring the codebase manually, use the ferro-mcp MCP server tools.**

The ferro-mcp crate provides introspection tools (like Laravel Boost) that give you instant access to:

| Tool | Use When |
|------|----------|
| `application_info` | Starting work - get framework version, environment, installed crates, models |
| `list_routes` | Understanding available endpoints |
| `list_middleware` | Debugging request flow |
| `list_migrations` | Working with database schema |
| `list_events` | Understanding event-driven features |
| `list_jobs` | Working with background processing |
| `database_schema` | Writing queries or migrations |
| `database_query` | Testing queries directly |
| `last_error` | Debugging failures |
| `read_logs` | Investigating runtime behavior |
| `get_config` | Understanding configuration |
| `search_docs` | Finding relevant documentation |
| `tinker` | Testing code snippets interactively |

**Recommended workflow:**
1. Run `application_info` to understand the project state
2. Use `list_routes` to find relevant handlers
3. Use `database_schema` before writing queries
4. Use `last_error` when debugging issues

## Project Overview

Ferro is a Laravel-inspired web framework for Rust. It provides familiar patterns for developers coming from Laravel/PHP while leveraging Rust's safety and performance.

## Workspace Structure

| Crate | Purpose |
|-------|---------|
| `framework` | Core web framework (routing, HTTP, database, validation, middleware) |
| `ferro-cli` | CLI tool for project scaffolding and code generation |
| `ferro-events` | Event dispatcher with sync/async listeners |
| `ferro-queue` | Background job processing with Redis backend |
| `ferro-notifications` | Multi-channel notifications (mail, database, slack) |
| `ferro-broadcast` | WebSocket broadcasting with channel authorization |
| `ferro-storage` | File storage abstraction (local, S3, memory drivers) |
| `ferro-cache` | Caching with tags support |
| `ferro-macros` | Procedural macros (#[handler], #[service], etc.) |
| `ferro-inertia` | Inertia.js adapter for full-stack React/TypeScript |
| `ferro-mcp` | MCP server for AI-assisted development |
| `app` | Sample reference application |

## Architecture Patterns

### Request Flow

```
Request → Global Middleware → Route Middleware → Handler → Response
```

### Handler Signature

```rust
#[handler]
pub async fn show(req: Request, user: User) -> Response {
    Ok(json!({"user": user}))
}
```

- Handlers return `Response` which is `Result<HttpResponse, HttpResponse>`
- Use `#[handler]` macro for automatic dependency injection
- Parameters are extracted from request (path, query, body, services)

### Dependency Injection

```rust
// Define service trait
#[service(ConcreteUserService)]
pub trait UserService: Send + Sync {
    async fn find(&self, id: i64) -> Option<User>;
}

// Implement service
#[injectable]
pub struct ConcreteUserService { /* ... */ }

impl UserService for ConcreteUserService { /* ... */ }
```

Services are automatically resolved from the Container.

### Middleware

```rust
// Global middleware (bootstrap.rs)
global_middleware!(LoggingMiddleware);
global_middleware!(SessionMiddleware::new(config));

// Route middleware
route("/admin", admin_handler).middleware(AuthMiddleware);
```

### Database (SeaORM)

```rust
// Models implement Entity trait
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub email: String,
}

// Queries
let users = User::find().all(&db).await?;
let user = User::find_by_id(1).one(&db).await?;
```

### Validation

```rust
use ferro::{Validator, rules, required, email, min};

let result = Validator::new(&data)
    .rules("email", rules![required(), email()])
    .rules("password", rules![required(), min(8)])
    .validate();
```

### Inertia.js Responses

```rust
#[handler]
pub async fn index(req: Request) -> Response {
    Inertia::render(&req, "Users/Index", UsersIndexProps {
        users: User::all().await?,
    })
}
```

Component paths are validated at compile-time against `frontend/src/pages/`.

**Form handling pattern**: When you need to read the request body AND render Inertia responses (e.g., for validation errors), save the context first since `req.input()` consumes the request:

```rust
pub async fn login(req: Request) -> Response {
    let ctx = SavedInertiaContext::from(&req);  // Save before consuming
    let form: LoginRequest = req.input().await?;  // Consumes req

    if let Err(errors) = form.validate() {
        return Inertia::render_ctx(&ctx, "auth/Login", LoginProps { errors });
    }
    // ...
}
```

## Code Conventions

### File Organization

- `src/actions/` - Business logic handlers
- `src/models/` - Database entities
- `src/middleware/` - Custom middleware
- `src/services/` - Service implementations
- `src/requests/` - Form request validation
- `src/events/` - Event definitions
- `src/listeners/` - Event listeners
- `src/jobs/` - Background jobs
- `src/notifications/` - Notification classes

### Naming Conventions

- Handlers: `snake_case` functions with `#[handler]`
- Models: `PascalCase` structs
- Services: `PascalCase` traits with `Service` suffix
- Middleware: `PascalCase` structs with `Middleware` suffix

### Error Handling

```rust
// Use thiserror for domain errors
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("User not found")]
    NotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
}

// Return HttpResponse for HTTP errors
Err(HttpResponse::not_found("User not found"))
```

## CLI Commands

```bash
# Create new project
ferro new my-app

# Generate code
ferro make:controller UserController
ferro make:model User
ferro make:middleware AuthMiddleware
ferro make:event UserRegistered
ferro make:listener SendWelcomeEmail
ferro make:job ProcessPayment
ferro make:notification OrderShipped

# Database
ferro db:migrate
ferro db:rollback
ferro db:fresh

# Development
ferro serve              # Start dev server
ferro generate-types     # Generate TypeScript types
```

## Testing

```rust
#[ferro_test]
async fn test_user_creation() {
    let client = TestClient::new().await;

    let response = client
        .post("/api/users")
        .json(&json!({"name": "John", "email": "john@example.com"}))
        .send()
        .await;

    expect!(response.status()).to_equal(201);
}
```

## Common Tasks

### Adding a New Route

1. Create handler in `src/actions/`
2. Add route in `src/routes.rs`
3. Add middleware if needed

### Adding a New Model

1. Run `ferro make:model ModelName`
2. Create migration
3. Run `ferro db:migrate`

### Adding Background Jobs

1. Run `ferro make:job JobName`
2. Implement `Runnable` trait
3. Dispatch with `JobName::dispatch(data).await`

### Adding Events

1. Run `ferro make:event EventName`
2. Run `ferro make:listener ListenerName`
3. Register listener in `src/providers/event_service_provider.rs`
4. Dispatch with `EventName { data }.dispatch().await`

## Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Application entry point |
| `src/routes.rs` | Route definitions |
| `src/bootstrap.rs` | Global middleware registration |
| `framework/src/lib.rs` | Framework public API |
| `Cargo.toml` | Workspace dependencies |

## Development Tips

1. **Run tests**: `cargo test --all-features`
2. **Check formatting**: `cargo fmt --check`
3. **Run linter**: `cargo clippy`
4. **Build docs**: `cargo doc --no-deps --open`

## MCP Server Setup

To enable ferro-mcp tools in your AI agent, add to your MCP configuration:

```json
{
  "mcpServers": {
    "ferro": {
      "command": "cargo",
      "args": ["run", "--package", "ferro-mcp"],
      "cwd": "/path/to/ferro/project"
    }
  }
}
```

The MCP server will then expose all introspection tools to your AI agent automatically.
