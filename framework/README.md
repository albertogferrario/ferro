# Ferro

A batteries-included web framework for Rust.

Ferro brings a Laravel-style developer experience to Rust: expressive routing, dependency injection, an ORM layer, background jobs, events, notifications, real-time broadcasting, server-driven UIs, and first-class agent tooling via an in-process MCP introspection server. It is designed for agent-assisted development — every subsystem exposes typed introspection so coding assistants can reason about your application without guessing.

## What's included

- **Routing and middleware** — macro-based route definitions, typed extractors, middleware pipeline
- **Inertia.js integration** — React/TypeScript SPAs with automatic type generation and compile-time component validation
- **JSON-UI** — server-rendered, server-driven UIs with 30+ built-in components and a plugin system
- **Events and listeners** — typed event dispatcher with async listeners
- **Background jobs** — queue workers backed by `ferro-queue` with retries and scheduling
- **Multi-channel notifications** — mail, database, broadcast and custom channels via `ferro-notifications`
- **WebSocket broadcasting** — real-time channels via `ferro-broadcast`
- **File storage abstraction** — local and S3 drivers via `ferro-storage`
- **Caching with tags** — in-memory and Redis backends via `ferro-cache`
- **Localization** — translation files per locale via `ferro-lang`
- **Semantic theming** — fixed token vocabulary and intent templates via `ferro-theme`
- **Service projections** — typed model-to-UI pipelines via `ferro-projections`
- **Stripe billing** — subscription and payment primitives via `ferro-stripe`
- **AI classification and confirmations** — structured LLM output and human-in-the-loop flows via `ferro-ai`
- **WhatsApp messaging** — Business Cloud API integration via `ferro-whatsapp`
- **MCP introspection server** — 80+ tools for agent-assisted development via `ferro-mcp`

## Quick start

```bash
cargo install ferro-cli
ferro new myapp
cd myapp
ferro serve
```

Add the framework to an existing project:

```toml
[dependencies]
ferro = { package = "ferro-rs", version = "0.2" }
tokio = { version = "1", features = ["full"] }
```

## Documentation

- API reference: <https://docs.rs/ferro-rs>
- User guide: <https://docs.ferro-rs.dev/>
- Repository: <https://github.com/albertogferrario/ferro>

## Status

v0.2.0 — pre-1.0. Breaking changes are allowed between minor versions until 1.0.

## License

MIT
