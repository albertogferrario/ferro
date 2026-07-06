# Ferro Framework

Ferro is a Rust web framework optimized for AI-assisted authoring. It is built for developers whose primary authoring tool is a coding agent (Claude Code, Cursor, and similar), and exposes every subsystem through an in-process MCP (Model Context Protocol) server. An agent connected to your project reads routes, models, handlers, validations, and generation context as tool calls — not by parsing source.

The defining feature is **service projections**: declare a service and intent, get a working UI. The `ferro-projections` crate (shipped in v9.0) maps typed model pipelines to rendered views, so an agent can scaffold an end-to-end CRUD or workflow surface from a single declaration. At v1.0 the output is visual (HTML via JSON-UI, shipped in v10.0); the underlying model is designed to support additional rendering modalities over time.

There is no bundled agent UI. `ferro-mcp` is the introspection layer your agent talks to.

## What's included

- **Service projections** — typed model→UI pipelines via `ferro-projections`
- **JSON-UI** — server-rendered, server-driven UIs with 30+ components and a plugin system
- **MCP introspection** — a full suite of tools via `ferro-mcp` for agent-assisted development
- **Routing and middleware** — macro-based routes, typed extractors, middleware pipeline
- **Database** — SeaORM-based models, migrations, and query builder
- **Validation** — declarative rule sets with structured errors
- **Authentication** — session-based auth with guards and policies
- **Inertia.js** — full-stack React/TypeScript with compile-time component validation
- **Events, queues, notifications, broadcasting** — async workflows out of the box
- **Storage and caching** — pluggable backends (local/S3, in-memory/Redis)
- **Localization, theming, Stripe, AI, WhatsApp** — first-party crates for common needs
- **Deploy and CI scaffolding** — `ferro do:init`, `ferro ci:init`, `ferro doctor`

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

**Agent-first authoring** — Ferro is designed for the case where a human directs an AI agent and the agent writes the code. Every subsystem exposes typed introspection so the agent can reason about the application without guessing.

**Projection over hand-assembly** — UIs are derived from typed services and intents, not hand-wired component trees. The framework prefers declarations the agent can generate and refine.

**Convention over configuration** — Predictable file layout and naming so introspection produces useful answers and generated code lands in the right place.

**Type safety** — Routes, Inertia component paths, JSON-UI views, and database queries are validated at compile time. Mistakes surface as build errors the agent can read and fix.

**Async-first** — Built on Tokio.

## Status

Ferro is pre-1.0. Breaking changes are allowed between minor versions until 1.0.

## Getting Started

To start building, see the [Installation](getting-started/installation.md) guide. To wire an AI agent to your project via `ferro-mcp`, see [Working with Agents](getting-started/working-with-agents.md).
