// ============================================================================
// AI Development Boost Templates
// ============================================================================

/// Ferro framework guidelines for AI assistants
pub fn ferro_guidelines_template() -> &'static str {
    r#"# Ferro Framework Guidelines

Ferro is a Rust web framework inspired by Laravel, providing a familiar developer experience with Rust's performance and safety.

## Project Structure

```
app/
├── src/
│   ├── main.rs           # Application entry point
│   ├── routes.rs         # Route definitions
│   ├── bootstrap.rs      # Application bootstrap
│   ├── controllers/      # Request handlers
│   ├── middleware/       # HTTP middleware
│   ├── models/           # Database models (SeaORM entities)
│   │   └── entities/     # Auto-generated entities (do not edit)
│   ├── actions/          # Business logic actions
│   ├── events/           # Domain events
│   ├── listeners/        # Event listeners
│   ├── jobs/             # Background jobs
│   ├── notifications/    # Multi-channel notifications
│   ├── tasks/            # Scheduled tasks
│   ├── config/           # Configuration modules
│   └── migrations/       # Database migrations
└── Cargo.toml
frontend/
├── src/
│   ├── main.tsx          # Frontend entry
│   └── pages/            # Inertia.js pages (React/TypeScript)
└── package.json
```

## Key Conventions

### Controllers
- Use the `#[handler]` macro for route handlers
- Return `Response` type using helper macros

```rust
use ferro::{handler, json_response, Request, Response};

#[handler]
pub async fn index(_req: Request) -> Response {
    json_response!({ "message": "Hello" })
}
```

### Middleware
- Implement the `Middleware` trait
- Use `#[async_trait]` for async methods

```rust
use ferro::{async_trait, Middleware, Next, Request, Response};

pub struct MyMiddleware;

#[async_trait]
impl Middleware for MyMiddleware {
    async fn handle(&self, request: Request, next: Next) -> Response {
        // Before request
        let response = next(request).await;
        // After request
        response
    }
}
```

### Models (SeaORM)
- Models use SeaORM with an Eloquent-like API
- Entity files in `models/entities/` are auto-generated
- Custom logic goes in `models/{table_name}.rs`

```rust
// Query builder pattern
let users = User::query()
    .filter(Column::Active.eq(true))
    .all()
    .await?;

// Fluent create
let user = User::create()
    .set_email("user@example.com")
    .set_name("John")
    .insert()
    .await?;

// Fluent update
let updated = user
    .set_name("Jane")
    .update()
    .await?;
```

### Inertia.js Integration
- Backend sends data via `inertia_response!` macro
- Frontend receives as props in React components
- TypeScript types auto-generated from Rust structs

```rust
// Backend
#[handler]
pub async fn show(req: Request) -> Response {
    inertia_response!("Users/Show", {
        "user": user,
        "posts": posts
    })
}
```

### Database Migrations
- Create with `ferro make:migration <name>`
- Run with `ferro migrate`
- Sync models with `ferro db:sync`

### Error Handling
- Use `#[domain_error]` macro for custom errors
- Errors automatically convert to appropriate HTTP responses

```rust
use ferro::domain_error;

#[domain_error(status = 404, message = "User not found")]
pub struct UserNotFound;
```

## CLI Commands

- `ferro new <name>` - Create new project
- `ferro serve` - Start dev servers
- `ferro make:controller <name>` - Generate controller
- `ferro make:middleware <name>` - Generate middleware
- `ferro make:migration <name>` - Generate migration
- `ferro make:event <name>` - Generate event
- `ferro make:job <name>` - Generate background job
- `ferro migrate` - Run migrations
- `ferro db:sync` - Sync DB schema to entities
- `ferro mcp` - Start MCP server for AI assistance

## Best Practices

1. **Use Actions for Business Logic**: Keep controllers thin, move logic to action classes
2. **Leverage the Type System**: Use Rust's types for validation and safety
3. **Auto-generate Types**: Run `ferro generate-types` to sync Rust structs to TypeScript
4. **Database Sync**: Use `ferro db:sync` after migrations to update entity files
5. **Middleware Order**: Register middleware in the correct order in routes.rs
"#
}

/// Cursor-specific rules file
pub fn cursor_rules_template() -> &'static str {
    r#"# Ferro Framework - Cursor Rules

You are working on a Ferro framework project. Ferro is a Rust web framework inspired by Laravel.

## Framework Knowledge

- Ferro uses Rust with async/await for the backend
- Frontend uses React + TypeScript with Inertia.js
- Database layer uses SeaORM with an Eloquent-like API
- The project follows Laravel conventions adapted for Rust

## Code Style

- Use `#[handler]` macro for route handlers
- Use `#[async_trait]` for middleware
- Use `#[domain_error]` for custom errors
- Follow Rust naming conventions (snake_case for functions, PascalCase for types)

## When Generating Code

1. Controllers go in `app/src/controllers/`
2. Middleware goes in `app/src/middleware/`
3. Models go in `app/src/models/`
4. React pages go in `frontend/src/pages/`

## Available MCP Tools

Use the Ferro MCP tools for introspection:
- `application_info` - Get app info, versions, crates
- `list_routes` - See all defined routes
- `db_schema` - Get database schema
- `db_query` - Run read-only SQL queries
- `list_migrations` - Check migration status
- `list_middleware` - See registered middleware
- `read_logs` - Read application logs
- `last_error` - Get recent errors
- `tinker` - Execute Rust code in app context
- `browser_logs` - Read frontend error logs

## Common Patterns

### Adding a new page
1. Create controller handler in `app/src/controllers/`
2. Add route in `app/src/routes.rs`
3. Create React page in `frontend/src/pages/`
4. Run `ferro generate-types` to sync types

### Adding a database table
1. `ferro make:migration create_table_name`
2. Edit migration file
3. `ferro migrate`
4. `ferro db:sync`
"#
}

/// CLAUDE.md template for Claude Code
pub fn claude_md_template() -> &'static str {
    r#"# Project Instructions

This is a Ferro framework project - a Rust web framework inspired by Laravel.

## Quick Reference

- **Backend**: Rust with async/await, SeaORM for database
- **Frontend**: React + TypeScript with Inertia.js
- **CLI**: Use `ferro` command for scaffolding

## MCP Tools Available

The Ferro MCP server provides these introspection tools:
- `application_info`, `list_routes`, `db_schema`, `db_query`
- `list_migrations`, `list_middleware`, `list_events`, `list_jobs`
- `read_logs`, `last_error`, `browser_logs`, `tinker`

## Development Workflow

1. Use `ferro serve` to start dev servers
2. Use `ferro make:*` commands for scaffolding
3. Use `ferro db:sync` after migrations to update models
4. Use `ferro generate-types` to sync TypeScript types

## Ferro Framework Guidelines

See `.ai/guidelines/ferro.md` for detailed framework conventions.
"#
}

/// Section to append to existing CLAUDE.md
pub fn claude_md_ferro_section() -> &'static str {
    r#"
---

# Ferro Framework

This is a Ferro framework project - a Rust web framework inspired by Laravel.

## MCP Tools Available

The Ferro MCP server provides introspection tools:
- `application_info`, `list_routes`, `db_schema`, `db_query`
- `list_migrations`, `list_middleware`, `list_events`, `list_jobs`
- `read_logs`, `last_error`, `browser_logs`, `tinker`

## Framework Conventions

See `.ai/guidelines/ferro.md` for detailed framework conventions.
"#
}

/// GitHub Copilot instructions
pub fn copilot_instructions_template() -> &'static str {
    r#"# GitHub Copilot Instructions

## Project Type
This is a Ferro framework project (Rust web framework inspired by Laravel).

## Key Files
- `app/src/routes.rs` - Route definitions
- `app/src/controllers/` - Request handlers
- `app/src/models/` - Database models (SeaORM)
- `frontend/src/pages/` - React/TypeScript pages

## Code Patterns

### Controller Handler
```rust
use ferro::{handler, json_response, Request, Response};

#[handler]
pub async fn index(_req: Request) -> Response {
    json_response!({ "data": value })
}
```

### SeaORM Query
```rust
let items = Model::query()
    .filter(Column::Field.eq(value))
    .all()
    .await?;
```

### Inertia Response
```rust
inertia_response!("PageName", { "prop": value })
```

## Conventions
- Controllers are async handlers with `#[handler]` macro
- Models use SeaORM with Eloquent-like query builder
- Frontend pages receive data as Inertia props
- TypeScript types are auto-generated from Rust structs
"#
}
