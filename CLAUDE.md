# Claude Instructions for Ferro Framework

## Vision Anchors (see `.planning/VISION.md` for the full design philosophy)

Ferro is a Rust web framework optimized for AI-assisted authoring. Its surface is shaped for an agent reading the project through `ferro-mcp`, the introspection layer, rather than for hand-typing.

The **core abstraction is projection / intent** (`ferro-projections`, shipped in v9.0; v12.0 refines its rendering target). The framework is built around this abstraction; new design decisions should keep it clear and central.

**Rendering architecture:** The `Renderer` trait uses associated types for output and context — it is modality-agnostic. Renderers live in their output crate (e.g., `JsonUiRenderer` in ferro-json-ui), not in ferro-projections. ferro-projections owns only the trait, `derive_intents()`, and `ServiceDef`. When adding rendering capabilities, add a `Renderer` implementation in the output crate, do not add dependencies to ferro-projections.

`ferro-mcp` is the introspection layer agents use to read routes, models, handlers, and generation context. MCP tool descriptions, `json_ui_catalog` accuracy, `code_templates` accuracy, and `generation_context` quality are part of the framework's surface and held to the same quality bar as the Rust API.

## Architecture Principles

1. **Substance-first investment ordering.** When prioritizing work, the order is: compressive (projection / intent) → operational (it just works) → conceptual (small core in mental model) → aesthetic (visual polish).

2. **Continuous conceptual coherence.** Every feature phase enforces conceptual coherence at write-time. Before adding code, ask whether it fits the existing surface or whether the surface needs to evolve to accommodate it. If it does not fit, the phase scope expands to include the cross-cutting refactor.

3. **Validation through real-world applications.** The projection / intent system is iterated against real applications and a synthetic catalog of canonical app classes.

4. **Multimodal is a v2.0+ direction.** Visual rendering at v1.0; additional modalities later. When designing new abstractions, prefer formulations that do not silently assume a screen.

5. **Beauty is a design criterion, not decoration.** All four dimensions (aesthetic, conceptual, operational, compressive) are required for v1.0, applied in the priority order above.

6. **Project-agnostic crates.** Crates under `ferro-*` are libraries shared across every ferro application; they must not hardcode any application identity (app name, brand strings, copy, URLs). When a crate needs app-level identity — e.g. an `organizationName` on a generated artifact, a "powered by" footer, a default sender name, a callback base URL — it reads framework conventions: `APP_NAME` and `APP_URL`, the same env vars `framework::config::AppConfig` consumes. The pattern: each `ferro-*` crate exposes its own config struct with `app_name` / `app_url` fields populated from those env vars in `from_env()`, mirroring `ferro-inertia::InertiaConfig::app_name`. Reviewers should reject hardcoded strings like `"gestiscilo"`, `"Ferro Application"`, `"https://example.com"`, or any specific tenant identifier inside a `ferro-*` crate. The sole exception is documentation examples explicitly framed as samples.

## Quick Start

**Use ferro-mcp first.** The MCP tools provide instant introspection:
- `application_info` - Project state, models, installed crates
- `list_routes` - All endpoints
- `db_schema` - Table structure
- `last_error` - Debug failures

## Workspace Structure

| Crate | Purpose | Key Files |
|-------|---------|-----------|
| `framework` | Core web framework | `src/lib.rs` (public API) |
| `ferro-cli` | CLI tool | `src/commands/` |
| `ferro-events` | Event dispatcher | `src/lib.rs` |
| `ferro-queue` | Background jobs | `src/lib.rs` |
| `ferro-notifications` | Multi-channel notifications | `src/lib.rs` |
| `ferro-broadcast` | WebSocket broadcasting | `src/lib.rs` |
| `ferro-storage` | File storage abstraction | `src/lib.rs` |
| `ferro-cache` | Caching with tags | `src/lib.rs` |
| `ferro-macros` | Proc macros | `src/lib.rs` |
| `ferro-mcp` | MCP introspection library, launched in-process by `ferro mcp` subcommand | `src/tools/` |
| `ferro-inertia` | Inertia.js adapter | `src/lib.rs` |
| `ferro-json-ui` | JSON-based server-driven UI schema and renderer | `src/lib.rs` |
| `ferro-lang` | Localization (per-locale translation files) | `src/lib.rs` |
| `ferro-api-mcp` | Standalone MCP server bridging OpenAPI specs to MCP tools | `src/lib.rs` |
| `ferro-projections` | Service projection definitions (typed model→UI pipeline) | `src/lib.rs` |
| `ferro-stripe` | Stripe payment and subscription integration | `src/lib.rs` |
| `ferro-theme` | Semantic theme tokens and intent template schema | `src/lib.rs` |
| `ferro-ai` | AI structured classification and confirmation primitives | `src/lib.rs` |
| `ferro-whatsapp` | WhatsApp Business Cloud API integration | `src/lib.rs` |
| `ferro-orm` | Atomic conditional updates and ORM primitives (`GuardedUpdate`) | `src/lib.rs` |
| `ferro-audit` | Append-only structured before/after audit log with replay | `src/lib.rs` |
| `ferro-reservation` | Generic hold/commit/release reservation kernel | `src/lib.rs` |
| `app` | Sample application | Reference implementation |

## Key Patterns

### Handler Functions
```rust
#[handler]
pub async fn show(req: Request, user: User) -> Response {
    Ok(json!({"user": user}))
}
```
- Return `Response` = `Result<HttpResponse, HttpResponse>`
- Parameters auto-extracted from request

### Services
```rust
#[service(ConcreteType)]
pub trait MyService: Send + Sync { ... }

#[injectable]
pub struct ConcreteType;
```

### Validation
```rust
Validator::new(&data)
    .rules("email", rules![required(), email()])
    .validate()
```

### Inertia
```rust
// Basic render
Inertia::render(&req, "Component", Props { ... })

// Form handlers: save context before consuming request
let ctx = SavedInertiaContext::from(&req);
let form = req.input().await?;  // Consumes req
Inertia::render_ctx(&ctx, "Component", Props { ... })  // Use saved ctx
```
Component paths validated at compile-time.

## Common Operations

### Adding/Updating Features
1. Implement in appropriate crate
2. Export from `lib.rs`
3. Add to `framework/src/lib.rs` re-exports if user-facing
4. **Update documentation** in `docs/src/` (required)
5. **Update ferro-mcp** if the feature affects introspection (new commands, routes, models, etc.)

### Testing & Linting (MUST run before every commit)
```bash
cargo fmt --all -- --check
cargo clippy --all --all-targets -- -D warnings
cargo test --all-features
```
`--all-targets` is required — it catches issues in test code that `--all` alone misses. CI enforces `-D warnings` so any warning is a build failure.

### Documentation
- User docs: `docs/src/`
- API docs: `cargo doc --no-deps`

## File Locations

| Need | Location |
|------|----------|
| Public API | `framework/src/lib.rs` |
| Route macros | `ferro-macros/src/routing.rs` |
| Handler macro | `ferro-macros/src/handler.rs` |
| Validation rules | `framework/src/validation/rules/` |
| HTTP types | `framework/src/http/` |
| Database | `framework/src/database/` |
| Middleware | `framework/src/middleware/` |
| CLI commands | `ferro-cli/src/commands/` |

## Notes

- Never add co-author attribution to commits
- **Run fmt + clippy + tests before every commit** — see Testing & Linting section for exact commands
- Prefer editing existing files over creating new ones
- Keep changes focused and minimal
- **Always update docs when framework changes** - `docs/src/` must reflect current features
- **Update ferro-mcp when needed** - New CLI commands, routes, models, or introspectable features require MCP tool updates
