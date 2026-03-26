# Ferro

**A Laravel-inspired web framework for Rust**

[![Crates.io](https://img.shields.io/crates/v/ferro.svg)](https://crates.io/crates/ferro)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Build web applications in Rust with the developer experience you love from Laravel and Rails. Ferro gives you expressive routing, powerful tooling, and batteries-included features—without sacrificing Rust's performance.

[Website](https://ferro-rs.dev/) | [Documentation](https://docs.ferro-rs.dev/)

## Quick Start

```bash
cargo install ferro-cli
ferro new myapp
cd myapp
ferro serve
```

Your app is now running at `http://localhost:8080`

## Example

If you've used Laravel or Rails, this will feel familiar:

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

## Why Ferro?

- **Familiar patterns** — Routes, controllers, middleware, service container
- **CLI generators** — `ferro make:controller`, `ferro make:model`, `ferro db:migrate`
- **Database built-in** — Migrations, ORM, query builder
- **Modern frontend** — First-class Inertia.js + React with automatic TypeScript types
- **Rust performance** — All the safety and speed, none of the ceremony

## JSON-UI

An alternative to Inertia for building UIs without a frontend build step. Define views as JSON, render to HTML with Tailwind on the server.

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

## License

MIT
