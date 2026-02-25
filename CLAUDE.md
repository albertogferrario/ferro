# Claude Instructions for Ferro Framework

## Quick Start

**Use ferro-mcp first.** The MCP tools provide instant introspection:
- `application_info` - Project state, models, installed crates
- `list_routes` - All endpoints
- `database_schema` - Table structure
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
| `ferro-mcp` | MCP introspection server | `src/tools/` |
| `ferro-inertia` | Inertia.js adapter | `src/lib.rs` |
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
