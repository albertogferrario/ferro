# Ferro Framework

Ferro is an agent-first Rust web framework with Laravel-inspired conventions. It exposes its entire structure through MCP (Model Context Protocol), giving AI agents the same understanding of your application that developers have — routes, models, handlers, validations, and generation hints available as tool calls.

Ferro brings the developer experience of Laravel to Rust, providing familiar patterns and conventions while leveraging Rust's safety and performance. It is batteries-included: routing, database, authentication, queues, and 50+ other features work out of the box with sensible defaults.

## Features

- **MCP Introspection** - 57 built-in tools for AI agent integration
- **Routing** - Expressive route definitions with middleware support
- **Database** - SeaORM integration with migrations and models
- **Validation** - Laravel-style validation with declarative rules
- **Authentication** - Session-based auth with guards
- **Inertia.js** - Full-stack React/TypeScript with compile-time validation
- **Service Projections** - Automatic UI generation from model definitions
- **Events** - Event dispatcher with sync/async listeners
- **Queues** - Background job processing with Redis
- **Notifications** - Multi-channel notifications (mail, database, slack)
- **Broadcasting** - WebSocket channels with authorization
- **Storage** - File storage abstraction (local, S3)
- **Caching** - Cache with tags support
- **Testing** - Test utilities and factories

## Quick Example

```rust
use ferro::{handler, Request, Response, Router, AuthMiddleware, Inertia};

#[handler]
pub async fn index(req: Request) -> Response {
    let users = User::find().all(&db).await?;

    Inertia::render(&req, "Users/Index", UsersProps { users })
}

pub fn routes() -> Router {
    Router::new()
        .get("/users", index)
        .middleware(AuthMiddleware)
}
```

> **Agents can generate this automatically.** Connect `ferro-mcp` to your AI agent and use `code_templates` to scaffold handlers, `list_routes` to explore your API, and `get_handler` to read implementation details of any existing handler.

## Philosophy

**Agent-first** - Ferro exposes its entire structure via MCP so agents understand the application the same way developers do. Routes, models, handlers, services, projections, and generation hints are all available as tool calls. AI agents are first-class participants in the development workflow, not an afterthought.

**Convention over configuration** - Sensible defaults that work out of the box. Follow the conventions and the framework gets out of your way.

**Developer experience** - Clear error messages, helpful CLI, and comprehensive documentation. The same clarity that makes Ferro readable for developers makes it navigable for agents.

**Type safety** - Compile-time validation of routes, components, and queries. Inertia component paths, route parameters, and database queries are checked at compile time.

**Performance** - Async-first design built on Tokio.

## Getting Started

Ready to start building? Head to the [Installation](getting-started/installation.md) guide.

Want to connect an AI agent to your project? See [Working with Agents](getting-started/working-with-agents.md).
