# Phase 113: Pattern Coherence - Research

**Researched:** 2026-03-27
**Domain:** Documentation pattern standardization + Rust workspace code sharing
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- All code examples use explicit imports from crate root: `use ferro::{Request, Response, Router};`
- No glob imports (`use ferro::*`) — replace all 37 occurrences across 6 files
- No sub-module paths (`use ferro::validation::Validator`) — crate root only, per Phase 110 rule
- Imports always visible in examples (no `# use` hidden lines)
- components.md has 28 glob imports to convert — per-component import lists at Claude's discretion
- Every handler function example in docs gets `#[handler]` — no exceptions
- Non-handler functions (routes, services, policies) — Claude audits and fixes inconsistencies at discretion
- Return type style (`Response` vs full type) at Claude's discretion
- Replace `.unwrap()` with `?` where appropriate — Claude's discretion on edge cases
- 32 occurrences across 10 files to audit
- Pragmatic approach: fix what needs fixing, leave infallible operations if clearly safe
- Move COMPONENT_CATALOG to `ferro-json-ui` as `pub const COMPONENT_CATALOG: &str`
- Both ferro-cli and ferro-mcp already depend on ferro-json-ui — no new dependencies needed
- Remove duplicate definitions from `ferro-cli/src/ai.rs` and `ferro-mcp/src/tools/json_ui_generate.rs`
- Exact location in ferro-json-ui at Claude's discretion (lib.rs vs catalog.rs module)
- Record design decision resolution in PROJECT.md (updates the "Revisit" marker)

### Claude's Discretion

- Per-component import lists in components.md (exact imports per example)
- Non-handler macro consistency (services, policies, etc.)
- Return type presentation style
- unwrap() edge case judgment (infallible operations, test contexts)
- COMPONENT_CATALOG module location in ferro-json-ui

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| COH-01 | Import style standardized across all code examples in docs | 37 glob imports in 6 files, 59 sub-module path imports — all verified against ferro crate root exports |
| COH-02 | Handler macro patterns audited — all examples use `#[handler]` | Handler functions without `#[handler]` identified in storage.md and caching.md |
| COH-03 | Error propagation examples verified to use `?` not `unwrap()` | 32 `.unwrap()` occurrences across 10 files, categorized by fixability |
| COH-04 | COMPONENT_CATALOG duplication resolved (design decision + implementation) | Both locations confirmed, dependency gap identified: ferro-cli needs ferro-json-ui added to Cargo.toml |
</phase_requirements>

## Summary

Phase 113 is a documentation cleanup and code deduplication phase with no new framework features. All changes are mechanical: find-and-replace import styles, add missing `#[handler]` attributes, replace `.unwrap()` with `?`, and move a shared constant to its proper home.

The documentation lives in `docs/src/` (43 Markdown files in mdBook format). Code examples are embedded in fenced code blocks. There are two types of import violations: glob imports (`use ferro::*`) in 6 files totaling 37 occurrences, and sub-module path imports (`use ferro::validation::Validator`) in roughly 16 files totaling 59 occurrences. Both violate the Phase 110 rule: all ferro imports use explicit crate-root exports only.

The COMPONENT_CATALOG deduplication is the only Rust code change. The constant is currently copy-pasted between `ferro-cli/src/ai.rs` and `ferro-mcp/src/tools/json_ui_generate.rs`. The natural home is `ferro-json-ui` (the crate that defines JSON-UI types). ferro-mcp already depends on ferro-json-ui directly. ferro-cli does NOT have ferro-json-ui as a direct dependency — it must be added to `ferro-cli/Cargo.toml` before the constant can be referenced.

**Primary recommendation:** Two sequential tasks — (1) docs pattern fixes (import style + handler macro + unwrap), (2) COMPONENT_CATALOG refactor with Cargo.toml update + PROJECT.md update.

## Standard Stack

### What's exported at ferro crate root (verified against framework/src/lib.rs)

All of these are available as `use ferro::{...}` — no sub-module paths needed:

| Category | Available at ferro::{} |
|----------|----------------------|
| Auth | `Auth, AuthMiddleware, AuthUser, Authenticatable, GuestMiddleware, OptionalUser, UserProvider` |
| Session | `invalidate_all_for_user, session, session_mut, DatabaseSessionDriver, SessionConfig, SessionData, SessionMiddleware, SessionStore` |
| Validation | `Validator, validate, Rule, ValidationError, Validatable` + all rule functions: `required, email, min, max, string, confirmed, between, regex, in_array, not_in, alpha, alpha_dash, alpha_num, boolean, date, integer, nullable, numeric, same, url, different, required_if, accepted` |
| Container | `App, Container` |
| Middleware | `Limit, LimiterResponse, Middleware, MiddlewareFuture, MiddlewareRegistry, Next, RateLimiter, SecurityHeaders, Throttle, register_global_middleware` |
| Routing | `Router, get, post, put, patch, delete, route, routes, group` (and others) |
| Database | `Database, DatabaseConfig, DB, Model, ModelMut, RouteBinding, AutoRouteBinding, DbConnection, Seeder, DatabaseSeeder, SeederRegistry` |
| Testing | `Factory, FactoryBuilder, Fake, Sequence, TestClient, TestContainer, TestContainerGuard, TestDatabase, TestRequestBuilder, TestResponse` |
| Scheduling | `Schedule, Task, TaskBuilder, CronExpression, DayOfWeek, TaskEntry, TaskResult` |
| Hashing | `hash, needs_rehash, verify` |
| Macros | `handler, injectable, service, redirect, request, domain_error, ferro_test, test, describe` |

**NOT at crate root (sub-module only or not re-exported):**
- `ferro::models::*` — project-specific models, not a framework re-export (these examples use the project's own `crate::models::` path, and docs show them as `ferro::models::user` which is illustrative of user code pattern)
- `FactoryTraits`, `DatabaseFactory`, `Expect` — NOT in the crate root `pub use testing::{}` list; only `Factory, FactoryBuilder, Fake, Sequence, TestClient, TestContainer, TestContainerGuard, TestDatabase` are re-exported
- `ferro::prelude::*` — exists but deprecated (migration-guide.md correctly shows it as old pattern)

**The `rules!` macro:** `#[macro_export]` in `framework/src/validation/mod.rs`. As a macro_export it is available as `ferro::rules!` without any `use` statement. The docs pattern `use ferro::validation::{Validator, rules}` is WRONG — `rules` is a macro, not a path item. Correct usage: `use ferro::{Validator, required, email, ...}` and then `rules![required(), email()]` (macro needs no import when in scope via `#[macro_export]`).

## Architecture Patterns

### Import Style Decision Tree

```
Q: Is this a ferro:: type/function?
  → YES: use ferro::{TypeA, TypeB}; (crate root, never sub-module path)
  → NO: use crate::models::... (project code stays in crate::)

Q: Is this the rules! macro?
  → It's #[macro_export] — available as rules![] without any use statement
  → Remove "use ferro::validation::{..., rules}" and just call rules![]

Q: Is this a glob import?
  → Replace use ferro::*; with explicit list of actually-used types
```

### Correct Import Pattern (from app/src/controllers/auth_controller.rs)
```rust
// Source: ferro/app/src/controllers/auth_controller.rs (real production code)
use ferro::{
    confirmed, email, handler, hash, json_response, min, required, verify, Auth, HttpResponse,
    Request, Resource, Response, ResponseExt, Validator,
};
```

### Handler Pattern
```rust
// Source: CLAUDE.md framework docs
#[handler]
pub async fn show(req: Request, user: User) -> Response {
    Ok(json!({"user": user}))
}
```

Every route handler in docs MUST have `#[handler]`. Infrastructure functions (`register()`, trait method impls) do NOT need it.

### Error Propagation Pattern
```rust
// Use ? for fallible operations
let user = User::find_by_email(email).await?;

// NOT: .unwrap() on Option in handler context
let key_info = req.get::<ApiKeyInfo>().unwrap();  // BAD
let key_info = req.get::<ApiKeyInfo>().ok_or_else(|| HttpResponse::unauthorized())?;  // GOOD
```

### COMPONENT_CATALOG Move Pattern
```rust
// In ferro-json-ui (new location — lib.rs or catalog.rs)
/// Concise reference of all JSON-UI components for AI generation prompts.
pub const COMPONENT_CATALOG: &str = r#"..."#;

// In ferro-cli/src/ai.rs (after move)
use ferro_json_ui::COMPONENT_CATALOG;
// Use COMPONENT_CATALOG directly in build_view_context

// In ferro-mcp/src/tools/json_ui_generate.rs (after move)
use ferro_json_ui::COMPONENT_CATALOG;
// Remove local const, use COMPONENT_CATALOG.to_string()
```

### Anti-Patterns to Avoid

- **`use ferro::validation::{...}`** — never use sub-module paths; all validation items are at `ferro::{}`
- **`use ferro::container::App`** — `App` is at `ferro::App`
- **`use ferro::session::*`** — session items are at `ferro::{}` crate root
- **`use ferro::middleware::*`** — middleware items are at `ferro::{}` crate root
- **`use ferro::routing::*`** — routing items are at `ferro::{}` crate root
- **`use ferro::models::user`** — this is project-generated code, not a framework import; docs should use `crate::models::user` or a type alias
- **`use ferro::prelude::*`** — deprecated; only acceptable in migration-guide.md as the OLD pattern

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| COMPONENT_CATALOG sharing | New crate, build script, codegen | `pub const` in ferro-json-ui | Both consumers already dep on ferro-json-ui; single source of truth |
| Import discovery | Manual grep of types | Check framework/src/lib.rs `pub use` lines | Ground truth for what's at crate root |

## Common Pitfalls

### Pitfall 1: Testing Unwrap in Test Context
**What goes wrong:** Replacing `.unwrap()` with `?` in `#[tokio::test]` functions breaks compilation (test functions return `()` not `Result`).
**Why it happens:** Tests use `#[tokio::test]` which returns `()`, not the handler's `Response = Result<...>`.
**How to avoid:** In test context (functions with `#[tokio::test]`), `.unwrap()` on async results is idiomatic. Fix by using `expect("...")` for clarity but keep unwrap if returning `()`. Alternatively use `#[ferro_test]` which supports `? ` in tests.
**Warning signs:** `async fn test_something()` without return type — these are `()` returning tests.

### Pitfall 2: Sub-Module Imports That Look Explicit
**What goes wrong:** `use ferro::session::SessionConfig` looks explicit but violates the rule.
**Why it happens:** `SessionConfig` IS available at `ferro::SessionConfig` (it's in `pub use session::{..., SessionConfig, ...}`), so the sub-module import is unnecessary.
**How to avoid:** Always check `framework/src/lib.rs` pub use lines. If the type appears there, use crate root path.
**Warning signs:** Any `use ferro::X::Y` where X is a module name (validation, session, auth, middleware, routing, database, testing, container, scheduling).

### Pitfall 3: ferro::models is Project Code
**What goes wrong:** Treating `use ferro::models::user` as a framework import to fix.
**Why it happens:** The docs show database examples using `ferro::models::user` as a demo of "your project's models" — not a framework import. There is no `ferro::models` re-export.
**How to avoid:** Leave `ferro::models::user` patterns in docs as-is OR convert to `crate::models::user` for clarity. The framework does not own this path.
**Warning signs:** `ferro::models::` appearing in docs — this is scaffolded project code.

### Pitfall 4: Missing Cargo.toml Dependency for COMPONENT_CATALOG
**What goes wrong:** Moving COMPONENT_CATALOG to ferro-json-ui and importing it in ferro-cli fails to compile.
**Why it happens:** ferro-cli currently depends on ferro-mcp (which depends on ferro-json-ui transitively) but does NOT list ferro-json-ui as a direct dependency in its Cargo.toml. Transitive deps are not accessible without direct declaration.
**How to avoid:** Add `ferro-json-ui = { path = "../ferro-json-ui", version = "0.1" }` to ferro-cli/Cargo.toml before writing the import.
**Warning signs:** Compile error "use of undeclared crate or module `ferro_json_ui`" in ferro-cli.

### Pitfall 5: The `rules!` Macro Import
**What goes wrong:** Removing `use ferro::validation::{Validator, rules}` breaks the `rules![]` macro calls.
**Why it happens:** `rules!` is a `#[macro_export]` macro — it's available as `ferro::rules!` or just `rules!` anywhere in the crate without a `use` statement. The validator `.rules()` METHOD is different (takes a Vec). The macro import `use ...::{rules}` was never valid.
**How to avoid:** When converting `use ferro::validation::{Validator, rules}` to `use ferro::{Validator}`, verify that `rules![]` calls remain working (they will — macro_export makes it available without use).

### Pitfall 6: `FactoryTraits`, `DatabaseFactory`, `Expect` not at crate root
**What goes wrong:** Docs use `use ferro::testing::{FactoryTraits, DatabaseFactory, Expect}` — converting to `use ferro::{FactoryTraits}` fails.
**Why it happens:** These three are in the testing module but NOT listed in the `pub use testing::{...}` at crate root in framework/src/lib.rs.
**Resolution:** Keep `use ferro::testing::{FactoryTraits, DatabaseFactory, Expect}` as-is — these cannot be fixed to crate-root without a framework change. Document as known limitation in the plan, or file for Phase 114+.

## Code Examples

### Glob Import Replacement Pattern
```rust
// BEFORE (6 files, 37 occurrences)
use ferro::*;

#[handler]
pub async fn index(req: Request) -> Response { ... }

// AFTER — determine actual used types from the code block
use ferro::{handler, Request, Response};

#[handler]
pub async fn index(req: Request) -> Response { ... }
```

### Sub-module Import Replacement Pattern
```rust
// BEFORE (16 files, 59 occurrences) — examples
use ferro::session::SessionConfig;
use ferro::auth::{UserProvider, Authenticatable};
use ferro::validation::{Validator, rules};
use ferro::container::App;
use ferro::middleware::{RateLimiter, Limit};

// AFTER — all available at crate root per framework/src/lib.rs
use ferro::{SessionConfig, UserProvider, Authenticatable, Validator, App, RateLimiter, Limit};
// rules! macro needs no use statement
```

### Handler Macro Addition
```rust
// BEFORE (storage.md, caching.md)
async fn upload_file(
    request: Request,
    storage: Arc<Storage>,
) -> Response { ... }

// AFTER — route handlers returning Response need #[handler]
#[handler]
async fn upload_file(
    request: Request,
    storage: Arc<Storage>,
) -> Response { ... }

// NON-HANDLERS — do NOT add #[handler] to these
pub async fn register() { ... }            // bootstrap/registration function
async fn cache_user_session(...) -> Result<(), Error> { ... }  // utility, not a route
```

### Unwrap Replacement Patterns
```rust
// CASE 1: Option in handler — use ok_or_else
// BEFORE
let key_info = req.get::<ApiKeyInfo>().unwrap();
// AFTER
let key_info = req.get::<ApiKeyInfo>().ok_or_else(|| HttpResponse::bad_request())?;

// CASE 2: JSON field access in handler — already validated, use ? chain
// BEFORE
let email = data["email"].as_str().unwrap();
// AFTER
let email = data["email"].as_str().ok_or_else(|| HttpResponse::bad_request())?;

// CASE 3: serde_json::to_value — infallible for well-typed schemas, keep or use expect
// BEFORE
let schema_value = serde_json::to_value(&schema).unwrap();
// AFTER (use expect for documentation clarity)
let schema_value = serde_json::to_value(&schema).expect("schema serialization is infallible");

// CASE 4: Test context (#[tokio::test]) — keep unwrap or use expect
// BEFORE
let user = user::Entity::insert_one(new_user).await.unwrap();
// AFTER — tests return (), ? doesn't work; use expect for better diagnostics
let user = user::Entity::insert_one(new_user).await.expect("test user insert failed");

// CASE 5: Theme::from_path in docs — returns Result, use ?
// BEFORE
.default_theme(Theme::from_path("./themes/myapp").unwrap())
// AFTER — but this is setup code in main-like context, expect is reasonable
.default_theme(Theme::from_path("./themes/myapp").expect("theme directory not found"))

// CASE 6: App::get::<T>() in broadcaster examples — should be ? if in handler
// BEFORE (inside #[handler])
let broadcaster = App::get::<Broadcaster>().unwrap();
// AFTER
let broadcaster = App::get::<Broadcaster>().ok_or_else(|| HttpResponse::internal_server_error())?;
// OR — simpler in non-handler context: keep expect("broadcaster not registered")
```

### COMPONENT_CATALOG Move
```rust
// Step 1: Add to ferro-cli/Cargo.toml
// ferro-json-ui = { path = "../ferro-json-ui", version = "0.1" }

// Step 2: Add to ferro-json-ui/src/lib.rs (or new catalog.rs)
/// Concise reference of all JSON-UI components for AI generation prompts.
pub const COMPONENT_CATALOG: &str = r#"..."#;

// Step 3: Update ferro-cli/src/ai.rs
use ferro_json_ui::COMPONENT_CATALOG;
// Remove local const COMPONENT_CATALOG

// Step 4: Update ferro-mcp/src/tools/json_ui_generate.rs
use ferro_json_ui::COMPONENT_CATALOG;
// Remove local const COMPONENT_CATALOG
```

## State of the Art

| Old Pattern | Current Pattern | Notes |
|-------------|-----------------|-------|
| `use ferro::*` | `use ferro::{specific, types}` | Phase 110 established rule |
| `use ferro::validation::Validator` | `use ferro::{Validator}` | Sub-module paths deprecated |
| `use ferro::prelude::*` | Removed | migration-guide.md only |
| `.unwrap()` in doc examples | `.expect(...)` or `?` | Context-dependent |
| Duplicate COMPONENT_CATALOG | Single source in ferro-json-ui | Phase 113 resolves |

## Open Questions

1. **`ferro::models::user` pattern in docs**
   - What we know: There is no `ferro::models` re-export in framework/src/lib.rs
   - What's unclear: These are illustrative of project code. They should probably be `crate::models::user` but changing them may confuse readers who see them as "framework examples"
   - Recommendation: Convert to `crate::models::user` where clear it's project code. Keep in scope for COH-01 since it's a sub-module path violation.

2. **`FactoryTraits`, `DatabaseFactory`, `Expect` not at crate root**
   - What we know: These are in `ferro::testing` sub-module but not in the crate root `pub use testing::{...}` list
   - What's unclear: Whether to fix the framework exports (framework change) or leave the sub-module imports
   - Recommendation: Leave `use ferro::testing::{FactoryTraits, DatabaseFactory, Expect}` intact — framework export change is out of scope for Phase 113 (documentation audit only). Note in plan as known exception.

## Validation Architecture

> nyquist_validation is not set to false in .planning/config.json — section included.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (Rust built-in) |
| Config file | Cargo.toml workspace |
| Quick run command | `cargo test --all-features 2>&1 \| tail -20` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| COH-01 | No `use ferro::*` in docs | manual-only: grep verification | `grep -rn "use ferro::\*" docs/src/` | N/A |
| COH-02 | All doc handlers have `#[handler]` | manual-only: grep verification | `grep -B2 "^pub async fn\|^async fn" docs/src/ -r \| grep -v "#\[handler\]"` | N/A |
| COH-03 | No `.unwrap()` in doc examples | manual-only: grep verification | `grep -rn "\.unwrap()" docs/src/` | N/A |
| COH-04 | COMPONENT_CATALOG in ferro-json-ui | compilation test | `cargo build --all-features` | ✅ |

**Note:** COH-01, COH-02, COH-03 are documentation changes with no runnable code tests. Verification is grep-based. COH-04 involves Rust code changes that must compile.

### Sampling Rate
- **Per task commit:** `cargo build --all-features` (confirms COMPONENT_CATALOG compiles; docs changes need no build)
- **Per wave merge:** Full lint + test suite
- **Phase gate:** `grep -rn "use ferro::\*" docs/src/` returns zero results

### Wave 0 Gaps

None — existing infrastructure covers all phase requirements. No new test files needed.

## Sources

### Primary (HIGH confidence)

- `framework/src/lib.rs` — authoritative list of what is exported at `ferro::{}` crate root (verified directly)
- `ferro-cli/Cargo.toml` — confirmed: no ferro-json-ui direct dep (verified directly)
- `ferro-mcp/Cargo.toml` — confirmed: has ferro-json-ui direct dep at line 24 (verified directly)
- `ferro-cli/src/ai.rs` — COMPONENT_CATALOG location #1, confirmed identical to MCP version
- `ferro-mcp/src/tools/json_ui_generate.rs` — COMPONENT_CATALOG location #2
- `ferro-json-ui/src/lib.rs` — target crate for COMPONENT_CATALOG move; no existing COMPONENT_CATALOG export
- `framework/src/validation/mod.rs` — `rules!` is `#[macro_export]`, available without import
- `app/src/controllers/auth_controller.rs` — working example of correct ferro:: import style

### Secondary (MEDIUM confidence)

- CONTEXT.md — user decisions (locked)
- STATE.md — COMPONENT_CATALOG "Revisit" marker confirmed at PROJECT.md line 233

## Metadata

**Confidence breakdown:**
- Glob import count: HIGH — verified by grep, all 37 occurrences confirmed
- Sub-module import count: HIGH — verified by grep, 59 occurrences in 16 files confirmed
- What's at crate root: HIGH — verified against framework/src/lib.rs directly
- Handler gaps: HIGH — storage.md and caching.md handler functions without `#[handler]` confirmed
- Unwrap count: HIGH — 32 occurrences across 10 files confirmed by grep
- COMPONENT_CATALOG dependency gap: HIGH — ferro-cli/Cargo.toml verified, no ferro-json-ui entry
- Exceptions (FactoryTraits etc.): HIGH — verified these are not in crate root pub use list

**Research date:** 2026-03-27
**Valid until:** This is based on framework source code; valid until framework/src/lib.rs changes (stable)
