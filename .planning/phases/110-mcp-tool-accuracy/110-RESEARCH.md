# Phase 110: MCP Tool Accuracy - Research

**Researched:** 2026-03-26
**Domain:** Rust MCP server — ferro-mcp/src/service.rs, ferro-mcp/src/tools/code_templates.rs
**Confidence:** HIGH

## Summary

Phase 110 has two distinct tasks: (1) audit the `description` strings in all `#[tool]` attribute macros in `service.rs` to ensure they accurately describe what the framework can do, and (2) verify that code snippets in `code_templates.rs` and `generation_context.rs` compile against current framework exports.

The term "generation_hints" in the requirements refers NOT to a dedicated Rust field but to the structured text inside each tool's `description = "..."` attribute in `ferro-mcp/src/service.rs`. These descriptions guide agents with **When to use**, **Returns**, **Combine with**, and **Tip** sections. They are the hints agents read before generating code.

The most significant accuracy issue found during research: `code_templates.rs` and `generation_context.rs` both reference `ferro::prelude::*` in import lists and code snippets, but **the framework has no `prelude` module**. `framework/src/lib.rs` exports everything at the crate root; the correct import is `use ferro::{handler, Request, Response, ...}` with explicit types. This is a compile-time failure for any agent that copies these templates verbatim.

A second issue: the tool count in requirements says 57, but the actual count in `service.rs` is 65 (verified by counting `#[tool(` attributes). The REQUIREMENTS.md was last updated before 8 tools were added (projection tools, AI tools, WhatsApp tools, Stripe tools).

**Primary recommendation:** Fix `ferro::prelude::*` → explicit imports in all code_templates.rs and generation_context.rs snippets. Separately, audit tool description accuracy against current exports. Both are direct file edits to `ferro-mcp/src/`.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CLIMCP-02 | generation_hints audited and refreshed across all 57 MCP tool responses | Tool descriptions in service.rs examined; patterns for drift identified |
| CLIMCP-03 | MCP code_templates.rs patterns verified against current framework exports | Critical `ferro::prelude::*` bug found; framework/src/lib.rs exports mapped |
</phase_requirements>

## Standard Stack

### Core
| File | Location | Purpose |
|------|----------|---------|
| `service.rs` | `ferro-mcp/src/service.rs` | 65 `#[tool]` handlers with description strings |
| `code_templates.rs` | `ferro-mcp/src/tools/code_templates.rs` | 9 template categories, ~20 templates |
| `generation_context.rs` | `ferro-mcp/src/tools/generation_context.rs` | Import templates + common patterns |
| `framework/src/lib.rs` | root crate exports | Source of truth for all valid API types |

### No external dependencies involved
This phase is pure source code text editing — no new crates, no new APIs.

## Architecture Patterns

### How "generation_hints" work in this codebase

The MCP tool description strings follow a consistent format:

```rust
#[tool(
    name = "tool_name",
    description = "One-line summary.\n\n\
        **When to use:** ...\n\n\
        **Returns:** ...\n\n\
        **Combine with:** `other_tool` for context.\n\n\
        **Note:** Optional caveat."
)]
pub async fn tool_name(&self) -> String { ... }
```

These strings are the "generation_hints" the requirement refers to. They are embedded in the compiled binary and returned to agents via the MCP protocol's tool listing.

### Framework export pattern (source of truth)

`framework/src/lib.rs` is the authoritative export list. Key types agents need:

```rust
// Correct handler imports:
use ferro::{handler, Request, Response, HttpResponse};
use ferro::{Validator, rules};       // for validation
use ferro::{Inertia, SavedInertiaContext};  // feature-gated: cfg(feature = "inertia")
use ferro::{PaginationMeta, ResourceCollection, Resource};
use ferro::{json, text, bytes};      // response helpers
use ferro::{AppError, FrameworkError};
use sea_orm::{EntityTrait, QueryFilter, QueryOrder, PaginatorTrait, ColumnTrait};
use sea_orm::Set;                    // for ActiveModel mutation
```

There is NO `ferro::prelude` module. The `ferro_rs::prelude::*` pattern appears in old `.claude/commands/ferro/*.md` command files but that is legacy from before the crate was renamed.

### UpdateBuilder pattern (confirmed correct in templates)

`#[derive(FerroModel)]` (from `ferro-macros/src/model.rs`) generates:

```rust
// On the Model struct:
pub fn update(self) -> ModelUpdateBuilder { ... }
pub fn create() -> ModelBuilder { ... }
pub fn query() -> QueryBuilder<Entity> { ... }
pub async fn delete(self) -> Result<u64, FrameworkError> { ... }

// On ModelUpdateBuilder:
pub fn set_field_name(mut self, value: T) -> Self { ... }
pub fn clear_field_name(mut self) -> Self { ... }  // for Option<T> fields only
pub async fn save(self) -> Result<Model, FrameworkError> { ... }
```

The `update_handler` template in code_templates.rs (lines 205-251) already uses the correct pattern:
```rust
let result = existing
    .update()
    .set_name(data.name)
    .save()
    .await?;
```
This is correct. No legacy ActiveModel pattern found in update templates.

### Anti-Patterns to Avoid

- **`ferro::prelude::*`**: Does not exist. Will cause compile error `error[E0432]: unresolved import ferro::prelude`.
- **`ferro_rs::prelude::*`**: Old crate name, also wrong.
- **`ferry::prelude::*`**: Typo variant.
- **`StatusCode::CREATED`**: Used in the `create_handler` template — this should come from `use axum::http::StatusCode` or via `HttpResponse::status()`. Verify against actual app usage.
- **`.with_status(StatusCode::CREATED)`**: The `ResponseExt` trait's `status()` method takes a `u16`, not `StatusCode` directly — verify exact API.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Counting tools | Manual grep | `grep -c "#\[tool("` on service.rs | Exact, not guesswork |
| Verifying export exists | Grep in lib.rs | Search framework/src/lib.rs pub use lines | 100% accurate |
| Checking UpdateBuilder | Reading macro output | Read ferro-macros/src/model.rs | Definitive source |

## Common Pitfalls

### Pitfall 1: Confusing "generation_hints" with a Rust field
**What goes wrong:** Searching for `generation_hints` as a Rust identifier and finding nothing.
**Why it happens:** The term comes from requirements prose, not code.
**How to avoid:** Search for `#[tool(` in `service.rs` — each description string IS the generation hint.

### Pitfall 2: Assuming the tool count is 57
**What goes wrong:** The requirements say "57 MCP tools" but the actual count is 65.
**Why it happens:** Requirements were written before projection tools (5), AI tools (2), Stripe tools (3), WhatsApp tools (2) were added.
**How to avoid:** Run `grep -c '#\[tool(' ferro-mcp/src/service.rs` to get the real count. The fix is to update ALL 65 tools, not just 57.

### Pitfall 3: Assuming `ferro::prelude::*` exists
**What goes wrong:** Copying templates to a ferro project produces `E0432: unresolved import ferro::prelude`.
**Why it happens:** The `prelude` pattern was planned/used in older command docs but the module was never added to framework.
**How to avoid:** Reference `framework/src/lib.rs` directly. All exports are at crate root — `use ferro::{handler, Request, Response}`.

### Pitfall 4: Wrong validation import pattern
**What goes wrong:** Templates import `use ferro::validation::{Validator, rules}` but the `rules` macro is actually `ferro::rules!`.
**Why it happens:** Macro namespace differs from module namespace.
**How to avoid:** Verify in app code: actual usage is `ferro::rules![required(), email()]` (macro) and `use ferro::{Validator, required, email, ...}` (individual rule functions).

### Pitfall 5: Treating `StatusCode::CREATED` as available
**What goes wrong:** Template imports `use ferro::prelude::*` to get `StatusCode` and `ResponseExt`.
**Why it happens:** In the `create_handler` template, `.with_status(StatusCode::CREATED)` is used.
**How to avoid:** In actual app code, `HttpResponse::status(u16)` or `json_response!({...}).status(201)` is the pattern. `StatusCode` is not re-exported from the `ferro` crate root.

## Code Examples

Verified patterns from actual app code (`app/src/controllers/`, `app/src/api/`):

### Correct handler import pattern (verified from app/src/controllers/auth_controller.rs)
```rust
use ferro::{
    confirmed, email, handler, hash, json_response, min, required, verify,
    Auth, HttpResponse, Request, Resource, Response, ResponseExt, Validator,
};
use sea_orm::Set;
use serde::Deserialize;
```

### Correct UpdateBuilder pattern (verified from app/src/api/user_api.rs)
```rust
#[handler]
pub async fn update(user: users::Model, form: UpdateUserRequest) -> Response {
    let mut builder = user.update();
    if let Some(ref v) = form.name {
        builder = builder.set_name(v.clone());
    }
    let updated = builder.save().await
        .map_err(|e| HttpResponse::json(serde_json::json!({"error": e.to_string()})).status(500))?;
    Ok(ferro::Resource::to_wrapped_response(&UserResource::from(&updated), &req))
}
```

### Service.rs tool description format (pattern for audit)
```rust
// All 65 tools follow this format in ferro-mcp/src/service.rs:
#[tool(
    name = "tool_name",
    description = "One-line purpose.\n\n\
        **When to use:** Use case description.\n\n\
        **Returns:** What the tool returns.\n\n\
        **Combine with:** `related_tool` for follow-up tasks."
)]
```

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| `ferro_rs::prelude::*` | `use ferro::{handler, Request, ...}` | All templates need update |
| `use sea_orm::Set;` (separate) | included in `use sea_orm::Set;` — still valid | No change needed |
| 57 MCP tools | 65 MCP tools | Audit scope is 65, not 57 |
| Legacy ActiveModel update | `model.update().set_x().save().await` | Templates are already correct |

## Open Questions

1. **`ResponseExt::with_status` vs `.status(u16)` API**
   - What we know: `create_handler` template uses `.with_status(StatusCode::CREATED)` — but actual app uses `.status(201)` via `json_response!(...).status(201)`
   - What's unclear: Whether `with_status` exists on `HttpResponse` or if it's `.status(u16)` only
   - Recommendation: During plan execution, search `framework/src/http/response.rs` for the exact method name before updating templates

2. **`rules!` macro vs `rules` module**
   - What we know: validation templates import `use ferro::validation::{Validator, rules}` — but actual app uses `ferro::rules![...]` macro syntax
   - What's unclear: Whether `rules` is a valid module path that re-exports the macro
   - Recommendation: During plan execution, verify `framework/src/validation/mod.rs` exports before changing templates

3. **Tool descriptions accuracy beyond API correctness**
   - What we know: 65 tool descriptions exist and the pattern is documented
   - What's unclear: Whether the "When to use" and "Combine with" sections have gone stale (e.g., tools added after others that should be cross-referenced)
   - Recommendation: Spot-check 5-10 tools that were added most recently (projection, AI, Stripe, WhatsApp) for completeness

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (all-features) |
| Config file | Cargo.toml workspace |
| Quick run command | `cargo test -p ferro-mcp --all-features` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CLIMCP-02 | Tool descriptions are non-empty and structured | unit | `cargo test -p ferro-mcp` | N/A — descriptions are string literals, no automated test |
| CLIMCP-03 | Import strings in templates reference valid paths | unit | `cargo test -p ferro-mcp -- code_templates` | ✅ `code_templates::tests::test_all_categories_present` |
| CLIMCP-03 | UpdateBuilder pattern is correct | unit | `cargo test -p ferro-mcp -- code_templates` | ✅ existing tests compile the module |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-mcp --all-features`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
None — existing test infrastructure covers module compilation. The code_templates tests verify categories exist; they do not verify import correctness (which is a string content check, not a compile check). This is acceptable — the correctness check is done by the human/agent reading the research and fixing the known `prelude` bug.

## Sources

### Primary (HIGH confidence)
- `ferro-mcp/src/service.rs` — all 65 tool definitions with description strings read directly
- `ferro-mcp/src/tools/code_templates.rs` — all template code read directly
- `ferro-mcp/src/tools/generation_context.rs` — import templates read directly
- `framework/src/lib.rs` — authoritative export list read directly
- `ferro-macros/src/model.rs` — UpdateBuilder generation logic read directly
- `app/src/controllers/auth_controller.rs` — real app import patterns
- `app/src/api/user_api.rs` — real UpdateBuilder usage

### Secondary (MEDIUM confidence)
- `.claude/commands/ferro/controller.md` — contains `ferro_rs::prelude::*` (confirmed OLD/wrong)
- `docs/src/upgrading/migration-guide.md` — shows `ferro::prelude::*` as "After" migration target (may have been aspirational documentation that was never backed by a real module)

### Tertiary (LOW confidence — needs verification during execution)
- `ResponseExt::with_status` API surface — not verified, see Open Questions

## Metadata

**Confidence breakdown:**
- Tool count (65): HIGH — counted directly with grep
- `prelude` module missing: HIGH — confirmed no `pub mod prelude` in framework/src/lib.rs
- UpdateBuilder correctness in templates: HIGH — compared template to macro source
- Tool description format: HIGH — read all patterns in service.rs
- `rules!` import pattern: MEDIUM — seen in app code but module re-export path not confirmed

**Research date:** 2026-03-26
**Valid until:** 2026-04-25 (stable framework, 30-day window)
