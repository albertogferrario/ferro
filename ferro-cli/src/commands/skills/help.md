---
name: ferro:help
description: Show available Ferro commands and usage guide
---

<objective>
Display the complete Ferro command reference.

Output ONLY the reference content below. Do NOT add project-specific analysis or commentary.
</objective>

<reference>
# Ferro Command Reference

**Ferro** is a Laravel-inspired Rust web framework. These commands help you build Ferro applications efficiently.

## Prerequisites

Ensure `ferro-mcp` is running and connected. The MCP server provides introspection tools that power these commands.

## Quick Start

1. `/ferro:new` - Create new Ferro project
2. `/ferro:model <name>` - Generate model with migration
3. `/ferro:controller <name>` - Generate controller
4. `/ferro:routes` - List all routes
5. `/ferro:serve` - Start development server

## Commands

### Project Management

**`/ferro:new [name]`**
Create a new Ferro project.

- Scaffolds complete project structure
- Sets up database configuration
- Creates initial routes and models
- Initializes git repository

Usage: `/ferro:new my-app`

**`/ferro:info`**
Show project information using ferro-mcp introspection.

- Displays installed crates and versions
- Shows configured services
- Lists database status

Usage: `/ferro:info`

### Code Generation

**`/ferro:model <name> [fields...]`**
Generate a new model with migration.

- Creates model file in `src/models/`
- Creates migration in `migrations/`
- Supports field definitions: `name:type`

Usage: `/ferro:model Post title:string body:text user_id:integer`

**`/ferro:controller <name> [--resource]`**
Generate a controller.

- Creates controller in `src/controllers/`
- With `--resource`: generates CRUD handlers

Usage: `/ferro:controller PostController --resource`

**`/ferro:middleware <name>`**
Generate middleware.

- Creates middleware in `src/middleware/`
- Includes before/after hooks template

Usage: `/ferro:middleware RateLimit`

**`/ferro:migration <name>`**
Generate a database migration.

- Creates timestamped migration file
- Includes up/down methods

Usage: `/ferro:migration create_comments_table`

**`/ferro:job <name>`**
Generate a background job.

- Creates job in `src/jobs/`
- Implements Queueable trait

Usage: `/ferro:job SendWelcomeEmail`

**`/ferro:event <name>`**
Generate an event and listener.

- Creates event in `src/events/`
- Creates listener template

Usage: `/ferro:event UserRegistered`

**`/ferro:notification <name>`**
Generate a notification.

- Creates notification in `src/notifications/`
- Supports mail, database, broadcast channels

Usage: `/ferro:notification InvoicePaid`

### Database

**`/ferro:db:migrate`**
Run pending migrations.

Usage: `/ferro:db:migrate`

**`/ferro:db:rollback [steps]`**
Rollback migrations.

Usage: `/ferro:db:rollback 1`

**`/ferro:db:seed`**
Run database seeders.

Usage: `/ferro:db:seed`

**`/ferro:db:fresh`**
Drop all tables and re-run migrations.

Usage: `/ferro:db:fresh --seed`

**`/ferro:db:schema`**
Show database schema using ferro-mcp.

Usage: `/ferro:db:schema`

**`/ferro:tinker`**
Interactive database REPL.

- Execute queries interactively
- Test model methods

Usage: `/ferro:tinker`

### Routes

**`/ferro:routes`**
List all registered routes.

- Shows method, path, handler, middleware
- Uses ferro-mcp list_routes tool

Usage: `/ferro:routes`

**`/ferro:route:explain <path>`**
Explain a specific route in detail.

- Shows handler code
- Lists middleware chain
- Shows request/response types

Usage: `/ferro:route:explain /api/users`

### Testing

**`/ferro:test [filter]`**
Run tests.

- Runs cargo test with all features
- Optional filter for specific tests

Usage: `/ferro:test`
Usage: `/ferro:test user`

**`/ferro:test:coverage`**
Run tests with coverage report.

Usage: `/ferro:test:coverage`

### Development

**`/ferro:serve`**
Start development server.

- Runs with hot reload
- Shows request logs

Usage: `/ferro:serve`

**`/ferro:build`**
Build for production.

- Optimized release build
- Runs clippy and tests first

Usage: `/ferro:build`

### Debugging

**`/ferro:diagnose [error]`**
Diagnose errors using ferro-mcp.

- Analyzes last error
- Suggests fixes
- Shows related code

Usage: `/ferro:diagnose`
Usage: `/ferro:diagnose "connection refused"`

**`/ferro:logs [lines]`**
Show recent application logs.

Usage: `/ferro:logs 50`

### Introspection (via ferro-mcp)

**`/ferro:models`**
List all models with their fields.

**`/ferro:services`**
List registered services.

**`/ferro:middleware:list`**
List all middleware.

**`/ferro:events:list`**
List all events and listeners.

**`/ferro:jobs:list`**
List all jobs and their status.

## Files & Structure

```
my-ferro-app/
├── src/
│   ├── main.rs              # Application entry
│   ├── routes.rs            # Route definitions
│   ├── controllers/         # HTTP handlers
│   ├── models/              # Database models
│   │   └── entities/        # Auto-generated SeaORM entities
│   ├── middleware/          # Request/response middleware
│   ├── jobs/                # Background jobs
│   ├── events/              # Event definitions
│   ├── notifications/       # Notification classes
│   └── config/              # Configuration files
├── migrations/              # Database migrations
├── tests/                   # Test files
├── .env                     # Environment variables
└── Cargo.toml
```

## Common Workflows

**Creating a new resource:**
```
/ferro:model Post title:string body:text
/ferro:controller PostController --resource
# Edit routes.rs to add resource routes
/ferro:db:migrate
/ferro:test
```

**Debugging an issue:**
```
/ferro:diagnose
/ferro:logs 100
/ferro:route:explain /api/posts
```

**Checking project state:**
```
/ferro:info
/ferro:routes
/ferro:db:schema
```

## Getting Help

- Run `/ferro:help` for this reference
- Check ferro-mcp tools with `list_routes`, `database_schema`, etc.
- Read the Ferro documentation in `docs/`
</reference>
