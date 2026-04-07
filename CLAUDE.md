# Claude Instructions for Ferro Framework

## Vision Anchors (read `.planning/VISION.md` for the full thesis)

Ferro is **raw infrastructure** built as a generation target for AI-assisted authoring. The audience is **agent-assisted humans** — vibe coders and developers using Cursor, Claude Code, and similar tools. Non-developers come later via a separate Builder Brand tool that does not yet exist; ferro itself never targets non-developers directly.

**The killer feature is projection/intent** (already shipped in v9.0 `ferro-projections`, polished by v12.0). Every planning decision should ask: *does this serve projection/intent as the killer feature, or does it dilute the bet?* Polish elsewhere is welcome but never at the cost of the killer feature's clarity.

**Ferro-mcp is the v1.0 product surface**, not the Rust crate API. Agents talk to ferro through MCP tools. MCP tool descriptions, json_ui_catalog accuracy, code_templates accuracy, and generation_context quality are user-facing product, not infrastructure polish.

## Operational Principles

These principles override convenience and speed when they conflict:

1. **Substance-first investment ordering.** When prioritizing work, the order is: compressive (projection/intent) → operational (it just works) → conceptual (small core in mental model) → aesthetic (visual polish). Never invert. Aesthetic polish before compressive validation is vanity.

2. **Continuous coherence tax.** Every feature phase pays the conceptual coherence tax at write-time. Before adding code, ask: *does this fit the existing surface, or does the surface need to evolve to accommodate it?* If it doesn't fit, the phase scope expands to include the cross-cutting refactor. Coherence is enforced when code is written, not patched up periodically.

3. **No stop-loss on projection/intent.** If validation reveals problems, iterate with real cases and ultrathinking. Do not pivot away from the killer feature. The bet is committed.

4. **Co-dependent forcing function.** Gestiscilo's commercial roadmap drives ferro's evolution. Every gestiscilo feature is also a ferro feature opportunity. Mitigate overfitting through deliberate diversification (build canonical apps in domains gestiscilo doesn't cover) and synthetic catalog regression tests.

5. **Multimodal is the named weakness.** Visual at v1.0; audio and physical are v2.0+. The 7 intents may be subtly web-shaped — this is unprobed. When designing new abstractions, ask: *would this still hold if the rendering target were voice or haptic?* If the answer is "no, this assumes a screen," flag it.

6. **Maximum-quality stance has implicit guardrails.** Pick the hardest version of every problem you can deliver. Narrow only when delivery is structurally impossible (multimodal at v1.0 was the only such case). When in doubt: more, deeper, broader.

7. **Beauty is a design criterion, not decoration.** All four dimensions (aesthetic, conceptual, operational, compressive) are non-negotiable for v1.0. Honor them in priority order.

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
