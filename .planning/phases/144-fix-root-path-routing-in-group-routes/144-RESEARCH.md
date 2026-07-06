# Phase 144: Fix root path routing in group routes - Research

**Researched:** 2026-04-21
**Domain:** HTTP routing (path combination inside route groups)
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Path Combination Semantics**
- **D-01:** `get!("/", handler)` inside `group!("/prefix", { ... })` MUST be reachable at both `/prefix` and `/prefix/`. Implementation: register both variants into `matchit` (insert the handler twice under the two paths). Additive, zero breaking change.
- **D-02:** `get!("/", handler)` inside `group!("/", { ... })` MUST resolve to a single `/` (not `//`). Preserve current correct behavior.
- **D-03:** Trailing slash in the group prefix (e.g. `group!("/prefix/", { get!("/x", ...) })`) MUST produce `/prefix/x`, not `/prefix//x`. Strip one trailing `/` from the prefix before concatenation.
- **D-04:** Non-root route paths inside a non-root group retain current behavior: `group!("/api", { get!("/users", ...) }) → /api/users`.

**Scope of Fix**
- **D-05:** Fix MUST apply to BOTH group implementations: `framework/src/routing/macros.rs` (`GroupDef::register_with_inherited`) and `framework/src/routing/group.rs` (`GroupBuilder::finalize`).
- **D-06:** Nested groups MUST follow the same rules recursively. The full accumulated prefix is what gets combined with the leaf route path.

**Route Introspection**
- **D-07:** When a handler is registered under both `/prefix` and `/prefix/`, `RouteInfo` MUST contain ONE entry at the canonical path without trailing slash (`/prefix`).
- **D-08:** Route naming via `.name("foo")` and `route!("foo", ...)` MUST register only the canonical (no trailing slash) path in `ROUTE_REGISTRY`. `route_url("foo", &[])` returns the canonical form.

**Tests**
- **D-09:** Table-driven tests covering the full matrix of prefix × route-path combinations (8 cases enumerated in CONTEXT.md).
- **D-10:** An integration test asserting `list_routes()` / `get_registered_routes()` returns exactly ONE entry per (method, canonical path) pair.
- **D-11:** Both `group.rs` and `macros.rs` paths MUST have equivalent tests.

**Version & Release**
- **D-12:** Patch release 0.2.12 → 0.2.13 across workspace.
- **D-13:** Changelog entry names the gestiscilo-it field test as the source.

### Claude's Discretion

- Exact helper function name for path combination (e.g. `combine_prefix_and_route`).
- Whether to extract the combination logic into a shared helper used by both `group.rs` and `macros.rs` (recommended — the two implementations should not drift) or keep two mirrored implementations.
- Ordering of `insert_get` / `insert_post` / etc calls when registering both variants.
- Whether to short-circuit `/prefix` == `/prefix/` registration when the prefix is already `/`.

### Deferred Ideas (OUT OF SCOPE)

- Canonical URL enforcement (301-redirect `/prefix/` → `/prefix` or vice versa) — opt-in middleware, future phase.
- Method routing mismatch (405 Method Not Allowed) for `/prefix` vs `/prefix/` — not a known issue today.
- Wildcard `/{*path}` interaction with group root-path handlers — already works per the gestiscilo field test.
- Reconciling the two group implementations into one — future refactor phase.

</user_constraints>

## Summary

The bug is local: two functions each build the matchit insertion key by concatenating prefix + route path, and both mishandle the `/` leaf-route case. All nine locked decisions (D-01 … D-13) can be satisfied by a single small helper that canonicalizes the (prefix, route_path) → (canonical_path, alternate_path_opt) pair, applied at exactly two call sites. The rest of the router (matchit insertion, middleware lookup keyed by registered pattern, `RouteInfo` registry, `ROUTE_REGISTRY` named-route map) is compatible with the fix as-is — no signatures change.

Two critical correctness traps are confirmed by reading the dispatch path:

1. **Middleware is keyed by the matchit pattern that matched the request**, not by the request URL. `server.rs` line 260 does `router.get_route_middleware(&route_pattern)`, where `route_pattern` is the second element of the matchit value tuple (`RouteValue = (Arc<BoxedHandler>, String)`). If we insert a handler under both `/prefix` and `/prefix/` but only call `add_middleware("/prefix", …)`, the `/prefix/` request matches but sees zero middleware. Fix: the path-combination helper must be integrated such that `add_middleware` is invoked for BOTH registered paths.

2. **`register_route()` is called unconditionally inside every `insert_*` method** on `Router`. A naive double-insert would produce two `RouteInfo` entries per handler, violating D-07 / D-10. Fix: bypass the duplicate `register_route` call for the alternate path. The cleanest way (given `insert_get`/etc are `pub(crate)`) is to add a new `pub(crate)` method pair — one that registers + introspects (canonical), one that only inserts into matchit (alternate). No public API change.

**Primary recommendation:** Introduce a private `fn combine_group_path(prefix: &str, route_path: &str) -> PathVariants` in `framework/src/routing/mod.rs` (new private utility module or inline in `router.rs`), returning `(canonical: String, alternate: Option<String>)`. Add `pub(crate) fn insert_get_alias(…)` etc on `Router` that only touches matchit (no `register_route` call, no `ROUTE_REGISTRY` touch). Both `macros.rs::GroupDef::register_with_inherited` and `group.rs::GroupBuilder::finalize` call the helper, then perform one canonical insert + zero or one alias insert, and add middleware for both paths in the canonical-plus-alternate loop.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Path combination (prefix + route) | Routing module (in-process) | — | Pure string computation, no external state |
| matchit insertion (URL → handler) | `Router` / matchit radix tree | — | matchit 0.8 owns URL-to-value matching |
| Route introspection (`RouteInfo` list) | `routing::router` registry | `ferro-mcp` static analyzer | Live registry feeds OpenAPI + debug endpoints; MCP `list_routes` reads source (separate path) |
| Middleware association (path → mw) | `Router::route_middleware` HashMap | `server.rs` dispatch | Keyed by matchit pattern at dispatch time |
| Named-route URL generation | `ROUTE_REGISTRY` static | — | Maps name → canonical path |
| Nested-group prefix accumulation | `GroupDef::register_with_inherited` | — | Recursive concatenation, same helper at every level |

**Tier correctness:** All work in this phase lives inside `framework/src/routing/`. No responsibility crosses a crate boundary. `ferro-mcp` is unaffected — its `list_routes` tool reads `routes.rs` source statically; the `get_registered_routes()` runtime list is consumed only by `framework::debug` endpoints and `framework::api::openapi`, both of which inherit the D-07 canonical-path guarantee automatically.

## Standard Stack

This is a bug fix inside the existing framework. No new dependencies are added. Relevant existing crates only:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `matchit` | 0.8.6 | Radix-tree URL router | Already the router. [VERIFIED: Cargo.lock line 2963] |

**Version verification:** `framework/Cargo.toml` line 29 declares `matchit = "0.8"`. `Cargo.lock` pins `matchit 0.8.6`. Published 2024-07-31 per crates.io. [VERIFIED: Cargo.lock]

### Alternatives Considered

None relevant. The locked decision D-01 ("register both variants") is a framework-level behavior that sits entirely above matchit. Swapping matchit is out of scope and not justified by this bug.

**Installation:** No new dependencies.

## Architecture Patterns

### System Architecture Diagram

```
                        Request arrives at server.rs:247
                                    │
                                    ▼
                      router.match_route(&method, &path)
                                    │
                                    ▼
                matchit radix tree (per method)  ─── stores two leaves:
                                    │                 /prefix     → (handler, "/prefix")
                                    │                 /prefix/    → (handler, "/prefix")  ← same handler Arc, same pattern string
                                    ▼
                 Returns (handler, params, route_pattern="/prefix")
                                    │
                                    ▼
                 server.rs:260 router.get_route_middleware(&route_pattern)
                                    │
                                    ▼
                 HashMap<String, Vec<BoxedMiddleware>>  ─── contains two keys after fix:
                                    │                       "/prefix"  → [mw...]
                                    │                       "/prefix/" → [mw...]  (same mw vec cloned)
                                    ▼
                         MiddlewareChain + handler executes


Registration pipeline (macros.rs path):

  group!("/s/{slug}", { get!("/", serve_root) })
                  │
                  ▼
  GroupDef::register_with_inherited(router, parent_prefix="", …)
                  │
                  ▼  for each item: combine_group_path(full_prefix, route_path)
                  ▼
  PathVariants { canonical: "/s/{slug}", alternate: Some("/s/{slug}/") }
                  │
                  ▼
  router.insert_get("/s/{slug}", handler)       ← calls register_route + matchit.insert
  router.insert_get_alias("/s/{slug}/", handler) ← calls matchit.insert ONLY
                  │
                  ▼
  register_route_name("serve_root", "/s/{slug}")   (canonical only, per D-08)
                  │
                  ▼
  For mw in group_middleware:
      router.add_middleware("/s/{slug}", mw.clone())
      router.add_middleware("/s/{slug}/", mw.clone())   ← both, per middleware dispatch contract
```

### Component Responsibilities

| File | Current responsibility | Phase 144 change |
|------|------------------------|------------------|
| `framework/src/routing/macros.rs` lines 614–731 | `GroupDef::register_with_inherited` — recursive prefix + middleware accumulator | Invoke path-combination helper; emit one or two `insert_*` calls; mirror middleware add loop |
| `framework/src/routing/group.rs` lines 62–91 | `GroupBuilder::finalize` — flat prefix + middleware application | Same change as macros.rs, using shared helper |
| `framework/src/routing/router.rs` lines 229–267 | `insert_get/post/put/patch/delete` — matchit insert + `register_route` | Add alias variants (`insert_get_alias`, etc.) that skip `register_route` |
| `framework/src/routing/mod.rs` or new `framework/src/routing/path.rs` | — | New home for `combine_group_path` helper + unit tests |

### Recommended Project Structure

No new files are required; a small private module for the helper is nice-to-have:

```
framework/src/routing/
├── group.rs            # fix GroupBuilder::finalize
├── macros.rs           # fix GroupDef::register_with_inherited
├── mod.rs              # (optionally) add: mod path;
├── path.rs             # NEW: pub(super) fn combine_group_path(...) + unit tests
└── router.rs           # add insert_{method}_alias pub(crate) methods
```

Alternatively, inline the helper at the bottom of `macros.rs` (or in `router.rs`) and `pub(super)` it so `group.rs` can import it. Either location is acceptable; a dedicated `path.rs` is slightly cleaner (single responsibility, fewer edits per file).

### Pattern 1: Path-combination helper

**What:** Normalize `(prefix, route_path)` into one canonical path plus an optional alternate trailing-slash variant.

**When to use:** At both call sites that register grouped routes. Do NOT use for top-level `get!("/foo", ...)` outside a group — that path is taken via `RouteDefBuilder::register` and doesn't need the double-insert (there's no prefix to collide with).

**Example (proposed signature):**

```rust
// Source: proposed new framework/src/routing/path.rs
/// Resolve a group prefix + nested route path into a canonical registration
/// path plus an optional alternate that differs only by a trailing slash.
///
/// # Rules
///
/// - The prefix MUST be empty or start with `/`. No validation is performed
///   (enforced upstream by `validate_route_path`).
/// - A single trailing `/` in the prefix is stripped before concatenation.
/// - If the route path is `/`, the canonical form is the stripped prefix (or
///   `/` if the prefix is empty / was just `/`), and the alternate form is
///   the canonical form with a `/` appended — UNLESS canonical already ends
///   in `/`, in which case no alternate is emitted (D-02).
/// - Otherwise, the canonical form is `stripped_prefix + route_path` and no
///   alternate is emitted (D-04).
///
/// Returns `(canonical, alternate)`; the caller inserts both.
pub(crate) fn combine_group_path(prefix: &str, route_path: &str) -> (String, Option<String>) {
    // Strip exactly one trailing slash from the prefix (D-03).
    let stripped = prefix.strip_suffix('/').unwrap_or(prefix);

    if route_path == "/" {
        // Leaf is root — canonical is the stripped prefix; alternate adds a slash.
        if stripped.is_empty() {
            // group!("/", { get!("/", ...) }) → single "/" (D-02)
            ("/".to_string(), None)
        } else {
            let canonical = stripped.to_string();
            let alternate = format!("{canonical}/");
            (canonical, Some(alternate))
        }
    } else {
        // Non-root leaf — simple concatenation on stripped prefix (D-04).
        if stripped.is_empty() {
            // group!("/", { get!("/foo", ...) }) → "/foo"
            (route_path.to_string(), None)
        } else {
            (format!("{stripped}{route_path}"), None)
        }
    }
}
```

Unit tests co-located with the helper cover all eight rows of the D-09 matrix at the string level.

### Pattern 2: Alias insertion methods on `Router`

```rust
// Source: proposed additions to framework/src/routing/router.rs
impl Router {
    /// Insert a GET route under an alias path pointing at the same handler.
    /// Does NOT call register_route — the canonical path is already tracked.
    pub(crate) fn insert_get_alias(&mut self, path: &str, handler: Arc<BoxedHandler>, canonical: &str) {
        self.get_routes
            .insert(path, (handler, canonical.to_string()))
            .ok();
    }
    // …repeat for insert_post_alias, insert_put_alias, insert_patch_alias, insert_delete_alias
}
```

Note the third argument `canonical`: the matchit *value* for the alias stores the **canonical** pattern string so that `router.match_route(...)`'s third return (route_pattern) stays stable regardless of which variant matched. This is the simplest way to make middleware lookup work (server.rs line 260 keys by `route_pattern`) — both the `/prefix` and `/prefix/` leaves return `route_pattern = "/prefix"`, and `add_middleware` is only called for the canonical path.

**Trade-off:** There are two viable middleware strategies — pick one and document the choice:

| Strategy | `add_middleware` calls | `route_pattern` value stored | Dispatch behavior |
|---|---|---|---|
| **A: Canonical-only middleware** (recommended) | once, with canonical path | both leaves store canonical | `/prefix/` request matches, returns pattern `/prefix`, middleware lookup finds entries |
| **B: Both-path middleware** | twice, canonical + alternate | each leaf stores its own path | `/prefix/` request matches, returns pattern `/prefix/`, middleware lookup finds separate entries |

Strategy A is cleaner — one `add_middleware` call, one `HashMap` entry, metrics/logs all attribute to the canonical pattern. Strategy B doubles the `route_middleware` HashMap entries and makes metrics grouping see two distinct labels for what is logically one route. Recommend A. Planner should confirm in PLAN.md.

### Anti-Patterns to Avoid

- **Silently using `.ok()` to swallow matchit insert errors.** The current code does this everywhere; Phase 144 should not add more silent insertions. For the alias insert, the expected result is `Ok(())` because `/prefix` and `/prefix/` are distinct matchit leaves (confirmed below). If it returns `Err`, something is wrong (e.g. a route was registered twice from user code) — propagate that through the existing `.ok()` pattern for now to avoid scope creep, but add a `debug_assert!` against `Conflict` so CI tests fail loudly. [ASSUMED: introducing debug_assert is within scope.]
- **Duplicating the path-combination logic between macros.rs and group.rs.** The two implementations already drifted once (macros.rs got a half-working `/` special case, group.rs got none). Sharing a helper is the fix that prevents this from recurring.
- **Calling `register_route_name` twice.** `ROUTE_REGISTRY` is a `HashMap<String, String>` keyed by name. Writing the same name with two paths just overwrites; the second write wins. This is silent drift — register canonical only (D-08).
- **Inserting the alias before the canonical.** `register_route` appends to `REGISTERED_ROUTES`. `update_route_name` / `update_route_mcp` / `update_route_middleware` all use `.iter_mut().rev().find(|r| r.path == path)` to find "the most recent route with this path." If the canonical is inserted first and then the alias (which skips `register_route`), the `.rev().find("/prefix")` call still finds the canonical entry correctly. **Order:** canonical first, alias second. This preserves D-07.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| URL-to-handler matching | Custom trie | Existing `matchit::Router` | Already battle-tested at 0.8.6 |
| Normalizing trailing-slash redirects | Custom middleware in this phase | Deferred per CONTEXT.md | D-01 explicitly chooses "register both" over redirects |
| Path canonicalization at request time | Pre-route middleware that strips trailing slash | — (deferred) | Opt-in, future phase |

**Key insight:** The only "new" logic in this phase is the six-line `combine_group_path` helper. Every other mechanism (matchit insertion, middleware dispatch, `RouteInfo` tracking, named-route registry) already works — the fix threads through existing code paths with one canonical insert and one alias insert per grouped route.

## Runtime State Inventory

This is a framework code fix, not a rename / refactor / migration. No stored data, live service config, OS-registered state, secrets, or build artifacts hold pre-fix routing strings. The runtime registries (`REGISTERED_ROUTES`, `ROUTE_REGISTRY`) are `OnceLock<RwLock<…>>` initialized fresh on every process start from the `routes! { }` macro expansion — no migration needed.

**Nothing found in any category** — verified by inspecting `framework/src/routing/router.rs` (lines 10–14, `static OnceLock`s are per-process, no persistence).

## Common Pitfalls

### Pitfall 1: Middleware keyed by wrong path
**What goes wrong:** Double-insert in matchit succeeds, `/prefix/` request matches, but no middleware runs because `add_middleware` was only called for `/prefix`.
**Why it happens:** `server.rs:260` looks up middleware via the matchit value tuple's second element (`route_pattern`). If the alias leaf stores its own path instead of the canonical, the middleware lookup fails.
**How to avoid:** In `insert_*_alias`, store the CANONICAL path as the second tuple element, not the alias path. Then one `add_middleware(canonical, …)` call covers both variants. (See Pattern 2 table, Strategy A.)
**Warning signs:** Test case: `group!("/s/{slug}", { get!("/", h) }).middleware(LogMw)` — assert the log middleware runs for both `/s/foo` and `/s/foo/`. Without this test, the bug ships silently.

### Pitfall 2: Duplicate `RouteInfo` entries
**What goes wrong:** Calling `insert_get` twice triggers `register_route` twice, producing two `RouteInfo` entries with identical method + path for the same handler. `get_registered_routes()` consumers (OpenAPI spec, debug endpoint, ferro-mcp surface) see inflated route counts.
**Why it happens:** `insert_get` / `insert_post` / etc unconditionally call `register_route(method, path)` (`router.rs:234`, 242, 250, 258, 266).
**How to avoid:** Add alias methods that skip `register_route`. D-10 integration test asserts `get_registered_routes().len()` equals the logical route count.
**Warning signs:** OpenAPI spec has duplicate operations; ferro-mcp `list_routes` output shows `/prefix` and `/prefix/` as separate routes when they should be one.

### Pitfall 3: Nested-group prefix accumulation with trailing slashes
**What goes wrong:** `group!("/a/", { group!("/b", { get!("/", h) }) })` — naive recursion produces `full_prefix = "/a/" + "/b" = "/a//b"` at level 2. The leaf then becomes `/a//b` — broken.
**Why it happens:** Current `register_with_inherited` at line 625 does `format!("{}{}", parent_prefix, self.prefix)` with no trailing-slash strip.
**How to avoid:** Apply the same strip-one-trailing-slash rule at the `full_prefix` computation step, not only at the leaf. Either inline at line 625, or extract a `combine_prefixes` helper symmetric to `combine_group_path`. Simplest: modify line 622–626 so the concat strips trailing `/` from `parent_prefix` before appending `self.prefix`.
**Warning signs:** Tests like `group!("/a/", { group!("/b/", { get!("/", h) }) })` expect `/a/b` + `/a/b/` — easy to miss without an explicit nested-trailing-slash test case.

### Pitfall 4: Root prefix double-registration
**What goes wrong:** `group!("/", { get!("/", h) })` — if treated naively by the new rule, canonical becomes `""` (stripped prefix) and alternate becomes `"/"` — then canonical is invalid. D-02 specifies single `/`.
**Why it happens:** The stripped-prefix + append-slash formula degenerates at the root case.
**How to avoid:** Explicit `if stripped.is_empty()` branch in `combine_group_path` returning `("/", None)` for the root-in-root case (as shown in Pattern 1 code).
**Warning signs:** Test case `group!("/", { get!("/", h) })` matches `/` exactly and produces exactly one `RouteInfo` entry.

### Pitfall 5: Named-route registration under alias
**What goes wrong:** Calling `register_route_name(name, alternate_path)` overwrites the canonical entry in `ROUTE_REGISTRY`, making `route("foo", &[])` return `/prefix/` instead of `/prefix`.
**Why it happens:** `ROUTE_REGISTRY` is a `HashMap` — last write wins.
**How to avoid:** Call `register_route_name` exactly once, with the canonical path. The helper returns `(canonical, alternate)`; the call site wires `canonical` into `register_route_name` and skips it for the alias.
**Warning signs:** `route_url("foo", &[])` test assertion must check canonical form.

### Pitfall 6: `RouteDefBuilder::register` path (non-grouped routes)
**What goes wrong:** Phase 144 could accidentally introduce double-insert at the non-group top level, e.g. `get!("/", home)` at the root of `routes! { }`.
**Why it happens:** If the fix is applied at the wrong layer (e.g. inside `insert_get`), every `/` route anywhere would double-register.
**How to avoid:** The fix lives in `GroupDef::register_with_inherited` and `GroupBuilder::finalize` ONLY. `RouteDefBuilder::register` (macros.rs line 162) is unchanged. Verify with a regression test that top-level `get!("/", h)` produces exactly one `RouteInfo` entry.
**Warning signs:** D-10 integration test catches this — count `RouteInfo` entries before and after the fix against the scaffold's `routes.rs`.

## Code Examples

Verified patterns from existing ferro code:

### Existing path combination (the broken site)
```rust
// Source: framework/src/routing/macros.rs lines 650–663 (current)
let full_path = if converted_route_path == "/" {
    if full_prefix.is_empty() {
        "/".to_string()
    } else {
        full_prefix.clone()   // BUG: collapses /prefix/ case to bare /prefix
    }
} else if full_prefix == "/" {
    converted_route_path.to_string()
} else {
    format!("{full_prefix}{converted_route_path}")
};
```

### Existing insert pattern (reuse as-is)
```rust
// Source: framework/src/routing/router.rs lines 229–235
pub(crate) fn insert_get(&mut self, path: &str, handler: Arc<BoxedHandler>) {
    self.get_routes
        .insert(path, (handler, path.to_string()))
        .ok();
    register_route("GET", path);
}
```

### Existing recursive nested-group pattern
```rust
// Source: framework/src/routing/macros.rs lines 720–728 (current — also affected by D-06)
GroupItem::NestedGroup(nested) => {
    nested.register_with_inherited(
        router,
        &full_prefix,             // ← must be trailing-slash-normalized before recursion
        &combined_middleware,
        &combined_mcp,
    );
}
```

### Existing `Box::leak` pattern for static path strings
```rust
// Source: framework/src/routing/macros.rs line 665
let full_path: &'static str = Box::leak(full_path.into_boxed_str());
// ← matchit.insert takes `impl Into<String>` in 0.8.6, but the codebase
//   uniformly leaks for a 'static str so insertion sites can also call
//   register_route_name(name, full_path) which stores &str. Follow the pattern.
//   Memory "leak" is bounded: routes are registered once at startup.
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Naive `format!("{}{}", prefix, path)` everywhere | Shared `combine_group_path` helper with three named rules (D-02, D-03, D-04) | Phase 144 (2026-04-21) | macros.rs and group.rs stop drifting; one testable function encodes all path-combination semantics |
| Half-working `/` special case in macros.rs only | Consistent `/` handling in both implementations, with both trailing-slash and non-trailing-slash variants registered | Phase 144 | `/s/{slug}/` reaches its handler — the gestiscilo field-test fix |
| `insert_get` always calls `register_route` | `insert_get` canonical path; `insert_get_alias` for aliases | Phase 144 | `get_registered_routes()` stays canonical; OpenAPI spec and debug endpoint see one route per logical handler |

**No deprecations.** This is a pure bug fix — existing public APIs (`group!`, `Router::group`, `GroupDef`, `get!`, `post!`, etc.) are unchanged.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | matchit 0.8.6 treats `/prefix` and `/prefix/` as distinct routes, no Conflict on insertion | Standard Stack | If wrong, alias insert would silently fail via `.ok()`; integration test would catch |
| A2 | The middleware `route_middleware` HashMap only needs one entry for canonical (Strategy A works) | Pattern 2 | If wrong, middleware wouldn't run on `/prefix/` variant; test case catches explicitly |
| A3 | Adding `insert_*_alias` as `pub(crate)` on `Router` is non-breaking (user code cannot reach them) | Pattern 2 | Low risk — `pub(crate)` is invisible outside the framework crate |
| A4 | `debug_assert!` on matchit insert is acceptable addition in scope | Anti-Patterns | Low risk — can be omitted if planner prefers minimal diff |

**All other claims verified** against file contents, `Cargo.lock`, or matchit upstream source.

## Open Questions

1. **Helper location: new `path.rs` module vs inline in `router.rs` vs inline in `macros.rs` with `pub(super)`?**
   - What we know: All three are valid; `pub(super)` requires both sibling modules to import it.
   - What's unclear: Project convention preference.
   - Recommendation: New `framework/src/routing/path.rs` — single responsibility, easy to test, easy to find. Planner picks.

2. **Should the fix add a `debug_assert!` for matchit Conflict, or stay with silent `.ok()`?**
   - What we know: Current codebase uses `.ok()` uniformly; the alias insert is the first place where we have a strong expectation of no conflict.
   - Recommendation: Add `debug_assert!` only in the alias path; keep `.ok()` for canonical. Bounds scope.

3. **Does `RouteDefBuilder::register` (non-grouped path) need to also apply the combine helper for `get!("/", …)` at the top level?**
   - What we know: Top-level `get!("/", h)` goes through `Router::get(path, h)` at line 270, inserts as `/` directly with no prefix handling — this already works correctly.
   - Recommendation: No change to that path; add a regression test confirming top-level root route is unaffected.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain (cargo, rustc) | Build and test | ✓ | project uses `rust-version = "1.88.0"` | — |
| matchit | Routing | ✓ | 0.8.6 (pinned in Cargo.lock) | — |
| `cargo fmt`, `cargo clippy`, `cargo test` | Pre-commit gates | ✓ (implied by project CI) | stable | — |

**Missing dependencies with no fallback:** None.
**Missing dependencies with fallback:** None.

This phase is a pure code change inside the framework crate. No external tools, databases, services, or runtimes beyond the existing Rust/cargo pipeline.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Built-in Rust `#[test]` + `cargo test --all-features` |
| Config file | None — tests live inline under `#[cfg(test)] mod tests { … }` in each module (existing convention, see `framework/src/routing/macros.rs` line 1178). Integration tests may be added under `framework/tests/` if needed (e.g. a new `routing_group_trailing_slash.rs`). |
| Quick run command | `cargo test -p ferro-rs --lib routing::` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-01 | `group!("/prefix", { get!("/", h) })` reaches handler at both `/prefix` and `/prefix/` | unit | `cargo test -p ferro-rs --lib routing::macros::tests::group_root_handler_matches_both_variants` | ❌ Wave 0 (new test) |
| D-02 | `group!("/", { get!("/", h) })` registers exactly one `/` route | unit | `cargo test -p ferro-rs --lib routing::macros::tests::root_prefix_root_handler_is_single_slash` | ❌ Wave 0 |
| D-03 | `group!("/api/", { get!("/x", h) })` produces `/api/x`, not `/api//x` | unit | `cargo test -p ferro-rs --lib routing::macros::tests::trailing_slash_prefix_is_stripped` | ❌ Wave 0 |
| D-04 | `group!("/api", { get!("/users", h) })` still produces `/api/users` (regression) | unit | `cargo test -p ferro-rs --lib routing::macros::tests::non_root_prefix_non_root_path_unchanged` | ❌ Wave 0 |
| D-05 | Both `macros.rs` and `group.rs` pass the same matrix | unit | `cargo test -p ferro-rs --lib routing::group::tests::<mirrored matrix>` | ❌ Wave 0 |
| D-06 | Nested `group!("/a", { group!("/b", { get!("/", h) }) })` matches `/a/b` and `/a/b/` | unit | `cargo test -p ferro-rs --lib routing::macros::tests::nested_group_root_matches_both_variants` | ❌ Wave 0 |
| D-07 | `get_registered_routes()` contains exactly one `RouteInfo` per logical handler after `group!("/prefix", { get!("/", h) })` | integration | `cargo test -p ferro-rs --test routing_group_trailing_slash -- no_duplicate_route_info` | ❌ Wave 0 (new integration file) |
| D-08 | Named-route lookup: `get!("/", h).name("home")` inside `group!("/api", ...)` → `route("home", &[])` returns `/api` | unit | `cargo test -p ferro-rs --lib routing::macros::tests::named_route_resolves_to_canonical` | ❌ Wave 0 |
| Gestiscilo reproducer | `group!("/s/{slug}", { get!("/", root), get!("/index.html", idx), get!("/{*path}", asset) })` — `/s/foo` → `root`, `/s/foo/` → `root`, `/s/foo/index.html` → `idx`, `/s/foo/bar.css` → `asset` with `slug=foo` | unit | `cargo test -p ferro-rs --lib routing::macros::tests::gestiscilo_reproducer` | ❌ Wave 0 |
| Regression: top-level `get!("/", h)` unchanged | unit | `cargo test -p ferro-rs --lib routing::tests::top_level_root_route_is_single_slash` | ❌ Wave 0 |
| helper: `combine_group_path` matrix (8 rows per D-09) | unit (table-driven) | `cargo test -p ferro-rs --lib routing::path::tests::combine_group_path_matrix` | ❌ Wave 0 (new helper module test) |

**Test strategy:** All tests are automated; no manual verification required. The gestiscilo reproducer can be expressed as a unit test against a synthetic `Router` built via `routes! { }`, dispatching requests via `router.match_route(&method, path)` and asserting `(handler_present, params, route_pattern)`. No HTTP roundtrip needed for correctness.

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-rs --lib routing::` (<30 s on a reasonable Mac)
- **Per wave merge:** `cargo test -p ferro-rs --all-features`
- **Phase gate:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`

### Wave 0 Gaps

- [ ] `framework/src/routing/path.rs` — NEW file with `combine_group_path` helper + inline `#[cfg(test)] mod tests` covering the 8-row matrix
- [ ] Extend `framework/src/routing/macros.rs` `#[cfg(test)] mod tests` (line 1178) with the D-01..D-04, D-06, D-08 cases and the gestiscilo reproducer — these require a test helper that builds a `Router` via the `routes!` macro or equivalent and asserts `match_route` outcomes
- [ ] Extend `framework/src/routing/group.rs` with a sibling `#[cfg(test)] mod tests` mirroring the macros.rs matrix (D-11). group.rs currently has no inline tests.
- [ ] `framework/tests/routing_group_trailing_slash.rs` — NEW integration test file asserting `get_registered_routes().len()` has exactly one entry per logical handler (D-10). This reads the process-global registry, so either (a) use `serial_test::serial` to guard against test ordering bleed, or (b) assert counts relative to a before/after delta rather than absolute values.
- [ ] Test helper utility — `fn dispatch(router: &Router, method: &str, path: &str) -> Option<(HashMap<String,String>, String)>` returning `(params, route_pattern)` for assertion clarity. Live either in a private `mod test_util` at the bottom of each test module, or in a shared `#[cfg(test)] pub(crate) mod test_util;` inside `routing/mod.rs`.
- [ ] `framework/tests/` already has two files (`api_resource_derive.rs`, `validation_derive.rs`) — follow their structure for the new integration file.

No framework install needed — `serial_test` is already a `[dev-dependencies]` entry in `framework/Cargo.toml` (line 76 region).

## Release Checklist

| Task | Files to Touch | Notes |
|------|---------------|-------|
| Workspace version bump 0.2.12 → 0.2.13 | `/Cargo.toml` line 27 (`version = "0.2.13"`) | Applies to every workspace crate via `version.workspace = true` |
| Regenerate lockfile | `/Cargo.lock` | `cargo build` refreshes automatically |
| Changelog entry | `/CHANGELOG.md` | Add section under a new `## framework` (or `## ferro-rs`) heading — current file is organized crate-first with `### [version] — date` subsections. Match the existing format from the `ferro-stripe` 0.4.0 entry. Content: name the gestiscilo-it field test (D-13), show the reproducer, show the canonical form users should prefer. |
| crates.io publish | (CI) | Via existing `.github/workflows/publish.yml` — the workspace's publish-update token covers version bumps on existing crates. No new crates added, so publish-new bootstrap is not needed (MEMORY: `project_ferro_publish_token_scoping.md`). |
| Docs update | See next section | — |

## Documentation Updates

Files that MUST be updated to reflect the new "`/` inside a group matches both with and without trailing slash" semantics:

| File | Lines | Change |
|------|-------|--------|
| `docs/src/the-basics/routing.md` | section "Route Groups" (around line 58) | Add a short subsection: "A `/` route inside a non-root group matches both `/prefix` and `/prefix/`. Other paths concatenate normally." Update any example that shows `group!("/s/{slug}", …)` patterns. |
| `docs/src/the-basics/middleware.md` | lines 179, 187 (inside `.group("/api", …)` and `.group("/admin", …)` examples) | No code change needed; optionally add a note that middleware applies uniformly to both trailing-slash variants. |
| `docs/src/features/authentication.md` | lines 382, 489, 497 (the `group!("/")` patterns) | Verify examples still compile; the root-prefix case (`group!("/")`) is unchanged by the fix. Add no new text unless the planner sees fit. |
| `docs/src/features/api.md` | lines 230, 235, 593, 671 (`group!("/api/v1")`, `group!("/api/v1/admin")`) | Unaffected — non-root prefix + non-root paths. No change. |
| `docs/src/features/rate-limiting.md` | lines 68–74 (`group!("/api", …)`, `group!("/auth", …)`) | Unaffected. No change. |
| `docs/src/features/stripe.md` | (grep showed 0 matches in this file currently) | No change needed. |
| `docs/src/features/api-mcp.md` | lines 183, 213 (`group!("/api/v1")`, `group!("/api/v1/internal")`) | Unaffected. No change. |
| rustdoc in `framework/src/routing/macros.rs` lines 14–22, 483–493, 596–606 | `group!` macro and `GroupDef` doc comments, and the "Path Combination" section at line 598–603 | Update the "Path Combination" section (lines 598–603) to reflect new rules: "If route path is `/` inside a non-root group, handler is registered under both `/prefix` and `/prefix/`. Trailing slash in the prefix is stripped. Root prefix `/` combined with route path `/` stays `/`." Keep scientific/minimalistic tone per CLAUDE.md. |

**ferro-mcp updates:** None required. `ferro-mcp::list_routes` parses `routes.rs` source and exposes what's written, not runtime state. The canonical-path invariant (D-07) makes `get_registered_routes()` identical before and after the fix for any pre-existing user code — no MCP surface change.

## Sources

### Primary (HIGH confidence)
- `framework/src/routing/macros.rs` — read in full, exact line references are file-verified
- `framework/src/routing/group.rs` — read in full
- `framework/src/routing/router.rs` — read in full (REGISTERED_ROUTES, ROUTE_REGISTRY, insert_* methods, match_route, get_route_middleware)
- `framework/src/routing/mod.rs` — pub surface, no changes needed
- `framework/src/server.rs` lines 220–300 — dispatch path confirmed: `route_pattern` keys middleware lookup
- `framework/src/middleware/mod.rs` — confirmed `pre_route_middleware` runs before `match_route`, not relevant to this fix's middleware semantics
- `ferro-cli/src/templates/files/backend/routes.rs.tpl` — confirmed default scaffold uses `group!("/", …)` pattern; fix must not regress
- `Cargo.toml`, `Cargo.lock` — workspace version 0.2.12, matchit 0.8.6 pinned
- `CHANGELOG.md` header — confirmed format (`### [version] — date` under `## crate` sections)
- `.planning/STATE.md` line 146 — phase 144 roadmap entry, gestiscilo source attribution
- `.planning/ROADMAP.md` lines 1268–1277 — phase 144 skeleton entry

### Secondary (MEDIUM confidence)
- matchit 0.8 docs (https://docs.rs/matchit/0.8.6/matchit/struct.Router.html) — confirmed `insert` signature, `InsertError::Conflict` semantics
- matchit source `src/tree.rs` (v0.8.6 tag on GitHub) — confirmed no trailing-slash normalization: "/foo" and "/foo/" are stored as distinct leaves
- `docs/src/the-basics/routing.md`, `middleware.md`, `features/*.md` — confirmed documentation call sites via grep

### Tertiary (LOW confidence)
- None. All claims in this research are either file-verified or cross-checked against upstream matchit source/docs.

## Project Constraints (from CLAUDE.md)

Actionable directives from `./CLAUDE.md` the planner MUST honor:

- **Every commit MUST pass:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`. `-D warnings` is enforced — any clippy warning blocks the merge.
- **No co-author attribution lines in commit messages.** No "Generated with Claude" trailers.
- **Prefer editing existing files over creating new ones.** The helper module (`routing/path.rs`) is the only new file recommended; planner may consolidate into `router.rs` or `macros.rs` if preferred.
- **Delete old code completely** — the half-working `/` special case at macros.rs:651–657 gets replaced, not left as fallback.
- **Update docs in `docs/src/` whenever framework surface changes.** The "`/` inside a group matches both" semantics is a user-visible change; `routing.md` MUST be updated.
- **Update `ferro-mcp` whenever introspection-affecting features change.** Not applicable here — `get_registered_routes()` stays canonical per D-07.
- **No versioned names** (no `register_v2`, `insert_get_new`). Use descriptive names (`combine_group_path`, `insert_get_alias`).
- **Early returns, flat code.** The helper's `if stripped.is_empty()` branches are all early returns.
- **Godoc-equivalent Rust doc comments on all exported/pub(crate) symbols.** The helper and alias methods need `///` doc comments.
- **Table tests for complex logic.** D-09 specifies table-driven tests; `combine_group_path_matrix` test uses a `&[(prefix, route_path, canonical, alternate)]` slice iterated with `for`.
- **Framework invariant restored:** "both APIs produce the same registered routes for equivalent definitions." Phase 144 tests in `group.rs` mirror those in `macros.rs` (D-11) — any future drift is caught.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — single existing dependency (matchit) verified against Cargo.lock and upstream source
- Architecture: HIGH — all call sites and data-flow paths read end-to-end in the source
- Pitfalls: HIGH — every trap is grounded in a specific line of existing code
- Test strategy: HIGH — follows existing `#[cfg(test)] mod tests` convention in the same modules

**Research date:** 2026-04-21
**Valid until:** 2026-05-21 (30 days — routing subsystem is stable; only invalidated if another phase changes `Router` internals or `GroupDef`/`GroupBuilder` structure)
