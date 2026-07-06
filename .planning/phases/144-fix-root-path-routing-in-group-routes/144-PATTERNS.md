# Phase 144: Fix root path routing in group routes — Pattern Map

**Mapped:** 2026-04-21
**Files analyzed:** 9 (2 created, 7 modified)
**Analogs found:** 9 / 9 — all inside `framework/src/routing/` or same-role siblings

## File Classification

| File | Change | Role | Data Flow | Closest Analog | Match |
|---|---|---|---|---|---|
| `framework/src/routing/path.rs` | NEW | private helper module (utility) | pure string transform (input → `(canonical, Option<alternate>)`) | `framework/src/routing/macros.rs` lines 24–76 (`validate_route_path` + `convert_route_params` — same crate, same sibling-module pattern, `pub(crate)`/const helpers + inline `#[cfg(test)] mod tests`) | exact (role + data flow) |
| `framework/src/routing/mod.rs` | MODIFIED | crate layout | module declaration | existing `mod group; mod macros; mod router;` lines 1–3 | exact |
| `framework/src/routing/macros.rs` (register_with_inherited) | MODIFIED | macro-based group registration | prefix + route_path → matchit insert + `RouteInfo` + middleware | existing lines 644–730 (the buggy site itself) | self-analog — reshape in place |
| `framework/src/routing/group.rs` (GroupBuilder::finalize) | MODIFIED | builder-based group registration | same data flow as macros.rs | `framework/src/routing/macros.rs::register_with_inherited` (sister implementation, must stay in lockstep — D-11) | exact (role + data flow) |
| `framework/src/routing/router.rs` (new `insert_*_alias`) | MODIFIED | matchit insertion primitives | path → matchit leaf only (NO `register_route`, NO `ROUTE_REGISTRY`) | `insert_get` / `insert_post` / `insert_put` / `insert_patch` / `insert_delete` lines 229–267 (copy shape, strip `register_route` call) | exact |
| `framework/tests/routing_group_trailing_slash.rs` | NEW | integration test | registry-state introspection + `match_route` dispatch | `framework/tests/api_resource_derive.rs` (same `framework/tests/` directory, `extern crate ferro_rs as ferro;` convention, `#[tokio::test]` shape) | role-match (different subject, same harness shape) |
| `/Cargo.toml` | MODIFIED | workspace config | version bump | existing `version = "0.2.12"` at line 27 | exact |
| `/CHANGELOG.md` | MODIFIED | release notes | append entry | existing `ferro-stripe` section header + `### [0.4.0] — 2026-04-20` subheader at lines 6–8 | exact (format analog) |
| `docs/src/the-basics/routing.md` | MODIFIED | user docs prose | append subsection | existing "Route Groups" section lines 47–66 | exact |
| `docs/src/the-basics/middleware.md` | MODIFIED (optional) | user docs prose | one-line note | existing group middleware example lines 179–191 | exact |

---

## Pattern Assignments

### 1. `framework/src/routing/path.rs` (NEW — helper module, pure transform)

**Analog:** `framework/src/routing/macros.rs` lines 24–76 (the sibling utility-helper convention)

**Module-level doc + function shape pattern** (`macros.rs` lines 24–76):

```rust
//! Route definition macros and helpers for Laravel-like routing syntax
//! [ … ]

/// Const function to validate route paths start with '/'
///
/// This provides compile-time validation that all route paths begin with '/'.
/// [ … ]
pub const fn validate_route_path(path: &'static str) -> &'static str { … }

/// Convert Express-style `:param` route parameters to matchit-style `{param}`
/// [ … ]
fn convert_route_params(path: &str) -> String { … }
```

**Conventions to replicate:**
- File-level `//!` doc with one-paragraph purpose.
- `///` doc comment on every `pub(crate)` / `pub` item (scientific, minimalistic — CLAUDE.md: "no marketing language").
- Use early-return style: CLAUDE.md requires flat code. The four-branch decision tree (`stripped.is_empty()` × `route_path == "/"`) maps to four explicit early returns.
- `pub(crate)` visibility — helper is NOT re-exported through `mod.rs` pub-use (see §2).
- Inline `#[cfg(test)] mod tests { use super::*; … }` at the bottom of the same file, following `convert_route_params` test at macros.rs lines 1182–1211.

**Proposed signature** (from RESEARCH.md lines 195–218, re-stated for the executor):

```rust
/// Resolve a group prefix + nested route path into a canonical registration
/// path plus an optional alternate that differs only by a trailing slash.
pub(crate) fn combine_group_path(prefix: &str, route_path: &str) -> (String, Option<String>) {
    let stripped = prefix.strip_suffix('/').unwrap_or(prefix);
    // … four branches, see RESEARCH §Pattern 1
}
```

**Test layout** (mirror `macros.rs` lines 1178–1211):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combine_group_path_matrix() {
        let cases: &[(&str, &str, &str, Option<&str>)] = &[
            ("",        "/",     "/",    None),
            ("/",       "/",     "/",    None),
            ("/",       "/x",    "/x",   None),
            ("/api",    "/",     "/api", Some("/api/")),
            ("/api",    "/x",    "/api/x", None),
            ("/api/",   "/x",    "/api/x", None),
            ("/api/",   "/",     "/api", Some("/api/")),
            ("/s/{slug}", "/",   "/s/{slug}", Some("/s/{slug}/")),
        ];
        for (prefix, route, want_canon, want_alt) in cases {
            let (canon, alt) = combine_group_path(prefix, route);
            assert_eq!(&canon, want_canon, "prefix={prefix:?} route={route:?}");
            assert_eq!(alt.as_deref(), *want_alt, "prefix={prefix:?} route={route:?}");
        }
    }
}
```

---

### 2. `framework/src/routing/mod.rs` (MODIFIED — add `mod path;`)

**Analog:** existing lines 1–3 of the same file:

```rust
mod group;
mod macros;
mod router;
```

**Change:** add `mod path;` alongside the three existing `mod` declarations. Do NOT add anything to the `pub use` blocks at lines 5–30 — the helper is `pub(crate)` and not user-visible (per CLAUDE.md: "no versioned names, descriptive names" and D-07/D-08: introspection surface is unchanged).

---

### 3. `framework/src/routing/router.rs` (MODIFIED — add `insert_*_alias` methods)

**Analog:** same file, existing `insert_get` / `insert_post` / `insert_put` / `insert_patch` / `insert_delete` lines 229–267.

**Current pattern to copy** (lines 229–235):

```rust
/// Insert a GET route with a pre-boxed handler (internal use for groups)
pub(crate) fn insert_get(&mut self, path: &str, handler: Arc<BoxedHandler>) {
    self.get_routes
        .insert(path, (handler, path.to_string()))
        .ok();
    register_route("GET", path);
}
```

**New alias method shape** (proposed — executor picks exact arg name):

```rust
/// Insert a GET route alias pointing at the same handler as a previously
/// registered canonical route. Skips `register_route` so `RouteInfo` and
/// `get_registered_routes()` stay canonical (D-07). The stored matchit value
/// carries the CANONICAL pattern string so middleware lookup in `server.rs`
/// (keyed by `route_pattern`) resolves to the canonical `add_middleware` key
/// regardless of which variant matched.
pub(crate) fn insert_get_alias(
    &mut self,
    alias_path: &str,
    handler: Arc<BoxedHandler>,
    canonical_path: &str,
) {
    self.get_routes
        .insert(alias_path, (handler, canonical_path.to_string()))
        .ok();
    // Intentionally NO register_route call.
}
```

**Five methods total** — one per HTTP verb, mirroring lines 229, 238, 246, 254, 262. Keep `pub(crate)` visibility (matches siblings — A3 in RESEARCH.md).

**Critical invariant (from RESEARCH.md Pitfall 1):** the THIRD tuple element (`canonical_path.to_string()`) is what `server.rs:260` uses as the middleware-lookup key. Store `canonical_path`, not `alias_path`. This is what enables Strategy A (one `add_middleware` call covers both leaves).

**Optional** (per RESEARCH.md A4 / Anti-Patterns): add `debug_assert!(self.get_routes.insert(…).is_ok(), …)` rather than the bare `.ok()` — the alias insert should never conflict in well-formed user code. Keep `.ok()` on the canonical paths unchanged.

---

### 4. `framework/src/routing/macros.rs` — `GroupDef::register_with_inherited` (MODIFIED, lines 614–731)

**Analog:** the function itself is the self-analog — reshape in place.

**Three surgical edits:**

**a) Trailing-slash-strip on `full_prefix` at line 622–626** (Pitfall 3):

Current:
```rust
let full_prefix = if parent_prefix.is_empty() {
    self.prefix.to_string()
} else {
    format!("{}{}", parent_prefix, self.prefix)
};
```

Change to: strip one trailing `/` from `parent_prefix` before concatenation, so `group!("/a/", { group!("/b", { get!("/", h) }) })` accumulates to `/a/b` not `/a//b`.

**b) Replace the buggy half-working `/` special case at lines 650–663** with a call to `combine_group_path`:

Current (the exact bug):
```rust
let full_path = if converted_route_path == "/" {
    if full_prefix.is_empty() {
        "/".to_string()
    } else {
        full_prefix.clone()   // BUG: collapses /prefix/ variant
    }
} else if full_prefix == "/" {
    converted_route_path.to_string()
} else {
    format!("{full_prefix}{converted_route_path}")
};
let full_path: &'static str = Box::leak(full_path.into_boxed_str());
```

New shape:
```rust
let (canonical, alternate) = combine_group_path(&full_prefix, &converted_route_path);
let canonical_path: &'static str = Box::leak(canonical.into_boxed_str());
let alternate_path: Option<&'static str> =
    alternate.map(|s| Box::leak(s.into_boxed_str()) as &'static str);
```

**CLAUDE.md constraint:** "Delete old code completely — no deprecation." The half-working if/else block is replaced, not commented out or preserved as fallback.

**`Box::leak` conventions** (from macros.rs line 665, same file, existing pattern — `matchit` requires `&'static str` in the surrounding code paths because `register_route_name(name, full_path)` stores `&str`): apply `Box::leak` to BOTH canonical and alternate. Memory leak is bounded — routes are registered once at startup (noted in RESEARCH.md line 362).

**c) Dispatch to the match-arm block at lines 668–684** — emit canonical `insert_*` first, then `insert_*_alias` for the alternate (if present). Order matters per RESEARCH.md Anti-Patterns §4 (`update_route_name` uses `.rev().find(|r| r.path == path)` — canonical must be the most recently registered entry with that path):

```rust
match route.method {
    HttpMethod::Get => {
        router.insert_get(canonical_path, route.handler.clone());
        if let Some(alt) = alternate_path {
            router.insert_get_alias(alt, route.handler, canonical_path);
        }
    }
    HttpMethod::Post => { /* same shape */ }
    // Put / Patch / Delete mirrored
}
```

Note: `route.handler` becomes `Arc<BoxedHandler>` so `.clone()` is cheap and the alias path gets the same handler Arc.

**d) `register_route_name` at line 688** stays unchanged — called exactly once with `canonical_path` (D-08, Pitfall 5).

**e) MCP metadata block at lines 691–710** — unchanged, uses `canonical_path`.

**f) Middleware loop at lines 713–718** — unchanged if adopting Strategy A (canonical-only). RESEARCH.md §Pattern 2 documents this is the recommended strategy. If the planner locks Strategy B instead, mirror the loop for `alternate_path`. **Lock Strategy A unless CONTEXT overrides** (CONTEXT.md has no override — D-07 favors A).

**g) Nested group recursion at lines 720–728** — after (a) applies, `full_prefix` is already trailing-slash-normalized, so no further change needed at the recursion site.

---

### 5. `framework/src/routing/group.rs` — `GroupBuilder::finalize` (MODIFIED, lines 62–91)

**Analog:** `framework/src/routing/macros.rs::register_with_inherited` post-fix — the two implementations must produce equivalent registered routes for equivalent inputs (D-05, D-11).

**Current code** (lines 62–91, the worse-drifted site):

```rust
fn finalize(mut self) -> Router {
    for route in self.group_routes {
        let full_path = format!("{}{}", self.prefix, route.path);

        match route.method {
            GroupMethod::Get => { self.outer_router.insert_get(&full_path, route.handler); }
            // …
        }
        for mw in &self.middleware {
            self.outer_router.add_middleware(&full_path, mw.clone());
        }
    }
    self.outer_router
}
```

**New shape:** same control flow as the macros.rs fix — call `combine_group_path(&self.prefix, &route.path)`, emit canonical `insert_*` then optional `insert_*_alias`, add middleware under canonical path only (Strategy A). `group.rs` uses `String` paths (not `&'static str`) — no `Box::leak` needed here: `insert_get`/`insert_post`/etc take `&str`, and the `canonical` / `alt` Strings live long enough in local scope for the calls.

**Import:** add `use super::path::combine_group_path;` at the top of group.rs (line 3 region — after `use super::{BoxedHandler, RouteBuilder, Router};`).

**Tests:** group.rs currently has NO `#[cfg(test)] mod tests`. Add one at the bottom of the file mirroring the macros.rs matrix (D-11). Build a `Router` via `Router::new().group("/prefix", |r| r.get("/", handler)).middleware(…).into()` and assert `router.match_route(&hyper::Method::GET, "/prefix")` and `router.match_route(&hyper::Method::GET, "/prefix/")` both yield `Some((_, _, "/prefix"))`.

---

### 6. `framework/tests/routing_group_trailing_slash.rs` (NEW — integration test)

**Analog:** `framework/tests/api_resource_derive.rs` lines 1–9 (same directory, same workspace test harness conventions).

**Header pattern to copy** (api_resource_derive.rs lines 1–9):

```rust
//! Integration tests for the ApiResource derive macro.
//!
//! Tests field selection, rename, skip, and From<Model> generation.

extern crate ferro_rs as ferro;

use ferro_rs::{ApiResource, Request, Resource};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::oneshot;
```

**Conventions:**
- `//!` module doc naming the phase / the covered decision IDs (D-07, D-10).
- `extern crate ferro_rs as ferro;` — follow the precedent even though it's not strictly required for 2021 edition; the sister file uses it.
- `use ferro_rs::{…}` for the items under test (`Router`, `group`, `get`, `routes`, `get_registered_routes`, `RouteInfo`).
- `#[tokio::test]` for any async assertion; `#[test]` is fine for synchronous `match_route` checks.

**Test-harness hazard** (called out in RESEARCH.md line 454): `REGISTERED_ROUTES` is a process-global `OnceLock<RwLock<Vec<RouteInfo>>>`. Parallel integration tests see each other's writes. Two mitigations, pick one:

- **Option A:** `#[serial_test::serial]` on each test (already in `[dev-dependencies]` — framework/Cargo.toml line 77).
- **Option B:** snapshot `get_registered_routes().len()` before building the router, assert the DELTA equals the expected logical count. Works without `serial_test`.

Option A is the existing framework convention for registry-touching tests — prefer it.

**Dispatch-assertion helper** (RESEARCH.md line 455):

```rust
fn dispatch<'a>(router: &'a Router, method: hyper::Method, path: &str)
    -> Option<(std::collections::HashMap<String, String>, String)>
{
    router.match_route(&method, path).map(|(_, params, pattern)| (params, pattern))
}
```

Put it at the top of the file (private to the integration-test crate).

**Required test cases** (from RESEARCH.md Wave 0 Gaps + D-09/D-10):

1. `group!("/s/{slug}", { get!("/", h) })` — dispatch `/s/foo` → `Some(slug=foo, pattern="/s/{slug}")`; dispatch `/s/foo/` → `Some(slug=foo, pattern="/s/{slug}")`. Same `pattern` both times (middleware-lookup invariant).
2. `get_registered_routes()` delta is exactly 1 for the above group (D-07, D-10).
3. Middleware fires on both variants (the test builds a group with a counter-incrementing middleware and asserts the counter advances for both `/s/foo` and `/s/foo/`).
4. Gestiscilo reproducer: `group!("/s/{slug}", { get!("/", root), get!("/index.html", idx), get!("/{*path}", asset) })` — all four URL shapes resolve to the expected handler.
5. Regression: top-level `get!("/", home)` still produces exactly one `RouteInfo`.

---

### 7. `/Cargo.toml` (MODIFIED — version bump)

**Analog:** existing line 27: `version = "0.2.12"`.

**Change:** `version = "0.2.13"`. That's the entire edit — the workspace uses `version.workspace = true` in every member Cargo.toml (confirmed framework/Cargo.toml line 3), so the single-line change propagates. CLAUDE.md `.github/workflows/publish.yml` already covers patch bumps on existing crates (MEMORY: `project_ferro_publish_token_scoping.md`).

**Cargo.lock** — `cargo build` regenerates automatically, do not hand-edit.

---

### 8. `/CHANGELOG.md` (MODIFIED — new entry)

**Analog:** existing `## ferro-stripe` section (lines 6–121) with `### [0.4.0] — 2026-04-20` subheader.

**Format to replicate:**

```markdown
## <crate>

### [<version>] — <YYYY-MM-DD>

<one-paragraph motivation sentence>

#### Added | #### Removed (breaking) | #### Changed (breaking) | #### Unchanged | #### Migration guide
```

**This phase adds a new top-level `## framework` section** at the top of the file (above `## ferro-stripe`) — the crate's published name is `ferro-rs` per framework/Cargo.toml line 2 (`name = "ferro-rs"`). Use `## ferro-rs` as the section heading to match crates.io naming and to stay consistent with "crate-first" organization noted in RESEARCH.md §Release Checklist.

**Required content** (per D-13 + CLAUDE.md "Repository documents must read as neutral"):

```markdown
## ferro-rs

### [0.2.13] — 2026-04-21

Bug fix: `get!("/", …)` registered inside `group!("/prefix", { … })` is now
reachable at both `/prefix` and `/prefix/`. Previously only
`/prefix/index.html`-style non-root paths matched; the trailing-slash variant
of the root-in-group case returned 404. Discovered via a field application
(`/s/{slug}/` routing).

#### Fixed

- Group path combination in both `GroupDef::register_with_inherited`
  (macro-based `group!`) and `GroupBuilder::finalize` (builder-based
  `Router::group`) now registers a leaf `get!("/", …)` under both `/prefix`
  and `/prefix/`. Trailing slash on the group prefix is also correctly
  stripped, so `group!("/api/", { get!("/x", …) })` produces `/api/x`, not
  `/api//x`.

#### Unchanged

- Top-level (non-grouped) `get!("/", …)` behavior.
- Route introspection: `get_registered_routes()` and `ferro-mcp list_routes`
  still show one entry per logical handler — the canonical path without
  trailing slash.
- Named-route resolution: `route("foo", &[])` returns the canonical path.
- Middleware attached to grouped routes fires for both trailing-slash
  variants.
```

**Voice discipline (CLAUDE.md "neutral" rule):** frame as "bug fix discovered via a field application." Do NOT name the specific downstream project (`gestiscilo-it`) in the repository-committed CHANGELOG — that belongs in local memory, not a public artifact. The D-13 decision says "names the gestiscilo-it field test as the source," but CLAUDE.md `## Repository documents must read as neutral` overrides toward the neutral phrasing ("field application"). Planner should confirm.

---

### 9. `docs/src/the-basics/routing.md` (MODIFIED)

**Analog:** existing "Route Groups" section lines 47–66.

**Insertion point:** immediately after the closing triple-backtick of the code example at line 66, before "## Named Routes" at line 68.

**Prose conventions to follow** (observed in lines 47–66):
- Sentence-case prose, no emojis.
- Code fences in ` ```rust ` (not `rust,ignore`).
- Short declarative sentences; no marketing voice.

**Required text** (scientific/minimalistic per CLAUDE.md):

```markdown
### Root routes inside a group

A `"/"` route inside a non-root group matches both the bare prefix and the
prefix with trailing slash:

\`\`\`rust
Router::new()
    .group("/s/{slug}", |r| {
        r.get("/", show_item)         // matches /s/foo AND /s/foo/
         .get("/edit", edit_item)     // matches /s/foo/edit
    })
\`\`\`

Other paths concatenate normally. A trailing slash on the group prefix is
stripped before concatenation: `group("/api/", …)` with child `/x` produces
`/api/x`.
```

### 10. `docs/src/the-basics/middleware.md` (MODIFIED — optional)

**Analog:** existing group-middleware example lines 179–191.

**Insertion:** one sentence after line 191, before the "## Middleware Execution Order" heading at line 194:

```markdown
Middleware attached to a group applies uniformly to root-path routes inside
the group, covering both `/prefix` and `/prefix/` variants.
```

Only add if Strategy A lands (which it should per CONTEXT.md D-07). If Strategy B were chosen, the sentence becomes load-bearing; under A it's documentation of an invariant.

### 11. Rustdoc in `framework/src/routing/macros.rs` — "Path Combination" section (MODIFIED, lines 598–603)

**Analog:** the doc block itself at lines 598–603 (currently describes the buggy half-working rule).

**Change:** replace with three rules matching `combine_group_path`:
- Non-root prefix + `/` leaf → registered under both `/prefix` and `/prefix/`.
- Trailing slash on prefix is stripped before concatenation.
- Root prefix `/` + root leaf `/` stays `/`.

Keep the tone consistent with lines 24–38 (`validate_route_path`'s "no marketing language" style).

---

## Shared Patterns

### S1. `pub(crate)` boundary for routing internals

**Source:** `framework/src/routing/router.rs` lines 205, 213, 218, 230, 238, 246, 254, 262. Every Router method that mutates internal maps is `pub(crate)`.

**Apply to:** all new items introduced by this phase — `combine_group_path`, `insert_get_alias`, `insert_post_alias`, `insert_put_alias`, `insert_patch_alias`, `insert_delete_alias`. Never `pub` — user code must not reach these.

### S2. `Box::leak(s.into_boxed_str())` for `&'static str` route paths

**Source:** `framework/src/routing/macros.rs` line 665.

**Apply to:** the new alternate path in `register_with_inherited`. Same-site pattern, bounded leak (startup-time only). Do NOT apply in `group.rs` — `GroupBuilder::finalize` uses `String` paths throughout and does not need static lifetimes (`insert_get` takes `&str`).

### S3. `.ok()`-swallowed matchit insert

**Source:** `framework/src/routing/router.rs` lines 232, 240, 248, 256, 264 — uniformly uses `.insert(path, …).ok()`.

**Apply to:** the new alias methods. Stay consistent with the pattern. Per RESEARCH.md A4, optionally add `debug_assert!` on conflict for alias path only — the expected case is `Ok(())` since `/prefix` and `/prefix/` are distinct matchit leaves (A1, verified against matchit 0.8.6 source).

### S4. Registry `.rev().find(|r| r.path == path)` invariant

**Source:** `framework/src/routing/router.rs` lines 59, 70, 86 — `update_route_name`, `update_route_middleware`, `update_route_mcp` all walk `REGISTERED_ROUTES` in reverse to find the most recently registered entry with a given path.

**Apply to:** the canonical-first / alias-second call order in `register_with_inherited` (macros.rs) and `finalize` (group.rs). If alias were inserted first, `.rev().find("/prefix")` would still find canonical (which has no alias in `REGISTERED_ROUTES` — alias skips `register_route`), so the invariant holds either way, but canonical-first is clearer and matches the mental model documented in Anti-Patterns §4.

### S5. Inline `#[cfg(test)] mod tests { use super::*; … }` at end of module

**Source:** `framework/src/routing/macros.rs` lines 1178–1180.

**Apply to:** `path.rs` (required), `group.rs` (required — currently has none, this phase adds the mirrored matrix per D-11), and extensions to the existing `macros.rs` mod tests block.

### S6. Integration test harness conventions

**Source:** `framework/tests/api_resource_derive.rs` lines 1–9. `extern crate ferro_rs as ferro;` + `#[tokio::test]` + `use ferro_rs::…`. `framework/tests/` is the standard location for tests that exercise the public API end-to-end.

**Apply to:** `framework/tests/routing_group_trailing_slash.rs`.

### S7. Workspace-inherited version

**Source:** `framework/Cargo.toml` line 3 (`version.workspace = true`). Every published workspace member uses this.

**Apply to:** Cargo.toml version bump — change `/Cargo.toml` line 27 only. No per-crate edits needed.

### S8. Pre-commit gate

**Source:** CLAUDE.md `Testing & Linting`.

**Apply to:** every commit in this phase. Command:
```
cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features
```
`-D warnings` is enforced by CI (MEMORY: `feedback_ci_clippy_command_match.md` — the exact CI command matters; do not substitute a lighter local variant).

---

## No Analog Found

None. Every file in this phase has a direct or role-match analog inside the same crate.

---

## Metadata

**Analog search scope:** `framework/src/routing/`, `framework/tests/`, `docs/src/the-basics/`, `/CHANGELOG.md`, `/Cargo.toml`, `framework/Cargo.toml`.
**Files scanned:** 9 source/test files + 2 docs + 2 config + 1 changelog.
**Pattern extraction date:** 2026-04-21.
