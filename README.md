# Ferro

**A Rust web framework optimized for AI-assisted authoring**

[![Crates.io](https://img.shields.io/crates/v/ferro-rs.svg)](https://crates.io/crates/ferro-rs)
[![Docs.rs](https://img.shields.io/docsrs/ferro-rs)](https://docs.rs/ferro-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Ferro is the substrate for developers who build alongside AI coding agents (Claude Code, Cursor, and similar). Every subsystem is introspectable through an in-process MCP server, so an agent connected to your project reads routes, models, handlers, and generation context as tool calls instead of guessing from source.

The defining feature is **service projections**: declare a service and intent, get a working UI. `ferro-projections` maps typed model pipelines to rendered views, letting an agent scaffold an end-to-end surface from a single declaration. At v1.0 the output is visual (HTML via JSON-UI); the underlying model is media-independent.

[API reference](https://docs.rs/ferro-rs) · [User guide](https://docs.ferro-rs.dev/) · [Repository](https://github.com/albertogferrario/ferro)

## Quick Start

```bash
cargo install ferro-cli
ferro new myapp
cd myapp
ferro serve
```

Your app is now running at `http://localhost:8080`. Point your AI agent at `ferro-mcp` via its MCP configuration and the agent can introspect and extend the project directly. There is no bundled agent UI — `ferro-mcp` is the introspection layer your agent talks to.

## Example

```rust
use ferro::{get, post, routes, json_response, Request, Response};

routes! {
    get("/", index),
    get("/users/{id}", show),
    post("/users", store),
}

async fn index(_req: Request) -> Response {
    json_response!({ "message": "Welcome to Ferro!" })
}

async fn show(req: Request) -> Response {
    let id = req.param("id")?;
    json_response!({ "user": { "id": id } })
}

async fn store(_req: Request) -> Response {
    // Your logic here
    json_response!({ "created": true })
}
```

## What's included

- **Routing and middleware** — macro-based routes, typed extractors, middleware pipeline
- **Database** — SeaORM-based models, migrations, query builder
- **Inertia.js integration** — React/TypeScript SPAs with automatic type generation
- **JSON-UI** — server-rendered, server-driven UIs with 30+ components
- **Events and listeners** — typed event dispatcher with async listeners
- **Background jobs** — queue workers with retries and scheduling (`ferro-queue`)
- **Multi-channel notifications** — mail, database, broadcast, custom channels (`ferro-notifications`)
- **WebSocket broadcasting** — real-time channels (`ferro-broadcast`)
- **File storage** — local and S3 drivers (`ferro-storage`)
- **Caching with tags** — in-memory and Redis backends (`ferro-cache`)
- **Localization** — per-locale translations (`ferro-lang`)
- **Semantic theming** — token vocabulary and intent templates (`ferro-theme`)
- **Service projections** — typed model-to-UI pipelines (`ferro-projections`)
- **Stripe billing** — subscriptions and payments (`ferro-stripe`)
- **AI classification + confirmations** — structured LLM output and human-in-the-loop (`ferro-ai`)
- **WhatsApp messaging** — Business Cloud API (`ferro-whatsapp`)
- **MCP introspection server** — 80+ agent tools (`ferro-mcp`)
- **CLI generators** — `ferro make:controller`, `ferro make:model`, `ferro db:migrate`
- **Structured audit log** — append-only before/after history with replay (`ferro-audit`)

## JSON-UI

An alternative to Inertia for building UIs without a frontend build step. Define views as JSON, render to HTML with Tailwind on the server. Shipped in v10.0 with 30+ components and a plugin system.

```json
{
  "layout": "app",
  "components": [
    {
      "type": "Table",
      "props": {
        "columns": ["name", "email"],
        "dataPath": "/data/users"
      },
      "actions": [
        { "name": "edit", "handler": "users.edit" },
        { "name": "delete", "handler": "users.destroy", "confirm": true }
      ]
    }
  ]
}
```

- Server-side rendering (no JS bundle required)
- Predefined components: Table, Form, Card, Button, Input, Alert, Modal
- Actions map directly to Ferro handlers
- Coexists with Inertia (use JSON-UI for CRUD, Inertia for custom UIs)

## End-to-End Type Safety

Ferro provides automatic TypeScript type generation from your Rust structs. Define your props once in Rust, and use them with full type safety in React.

**Define props in Rust:**

```rust
use ferro::{InertiaProps, inertia_response, Request, Response};

#[derive(InertiaProps)]
pub struct User {
    pub name: String,
    pub email: String,
}

#[derive(InertiaProps)]
pub struct HomeProps {
    pub title: String,
    pub user: User,
}

pub async fn index(_req: Request) -> Response {
    inertia_response!("Home", HomeProps {
        title: "Welcome!".to_string(),
        user: User {
            name: "John".to_string(),
            email: "john@example.com".to_string(),
        },
    })
}
```

**Run type generation:**

```bash
ferro generate-types
```

**TypeScript types are auto-generated:**

```typescript
// frontend/src/types/inertia-props.ts (auto-generated)
export interface HomeProps {
  title: string;
  user: User;
}

export interface User {
  name: string;
  email: string;
}
```

**Use in your React components with full autocomplete:**

```tsx
import { HomeProps } from "../types/inertia-props";

export default function Home({ title, user }: HomeProps) {
  return (
    <div>
      <h1>{title}</h1>
      <p>Welcome, {user.name}!</p>
      <p>Email: {user.email}</p>
    </div>
  );
}
```

Change a field in Rust, regenerate types, and TypeScript will catch any mismatches at compile time.

## Documentation

Ready to build something? Check out the [full documentation](https://docs.ferro-rs.dev/) to get started.

## Status

v0.2.0 — pre-1.0. Breaking changes are allowed between minor versions until 1.0. Current milestone work targets v12.0 spec-driven rendering.

## License

MIT
