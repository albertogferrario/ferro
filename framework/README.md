# Ferro

A batteries-included Rust web framework optimized for AI-assisted authoring.

Ferro is built for developers whose primary authoring tool is an AI coding agent (Claude Code, Cursor, and similar). Every subsystem exposes typed introspection through an in-process MCP server (`ferro-mcp`), so an agent connected to your project can read routes, models, handlers, validations, and generation context as tool calls instead of guessing from source.

The defining feature is **service projections**: describe a service and intent, get a working UI. The `ferro-projections` crate maps typed model pipelines to rendered views, so an agent can scaffold an end-to-end CRUD or workflow surface from a single declaration. At v1.0 the output is visual (HTML via JSON-UI); the underlying model is media-independent.

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

## Agent workflow

1. Install `ferro-cli` and create a project.
2. Point your AI agent at `ferro-mcp` via its MCP configuration.
3. The agent introspects the project (`application_info`, `list_routes`, `list_models`, `list_projections`) and generates code, projections, and views against the live application surface.

There is no bundled agent UI. `ferro-mcp` is the introspection layer your agent talks to.

## Status

v0.2.0 — pre-1.0. Breaking changes are allowed between minor versions until 1.0. The framework is developed in the open and is not feature-frozen.

## License

MIT
