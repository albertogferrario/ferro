---
phase: 144-fix-root-path-routing-in-group-routes
plan: 02
subsystem: routing
tags: [routing, macros, alias, matchit, group]

requires:
  - 144-01

provides:
  - "pub(crate) insert_get_alias + insert_post_alias + insert_put_alias + insert_patch_alias + insert_delete_alias on Router (framework/src/routing/router.rs)"
  - "GroupDef::register_with_inherited using combine_group_path + canonical/alias emit + trailing-slash-normalized parent prefix (framework/src/routing/macros.rs)"

affects:
  - 144-03
  - 144-04

tech-stack:
  added: []
  patterns:
    - "Strategy A: alias matchit leaf stores canonical pattern string — middleware lookup at server.rs:260 resolves under canonical key regardless of which URL variant matched"
    - "Canonical-first, alias-second call order: insert_get(canonical, handler.clone()) then insert_get_alias(alt, handler, canonical)"
    - "Path-specific registry assertions instead of global delta counts to avoid cross-test contamination via REGISTERED_ROUTES static"

key-files:
  created: []
  modified:
    - framework/src/routing/router.rs
    - framework/src/routing/macros.rs
    - framework/src/routing/path.rs

key-decisions:
  - "Replaced serial delta-count assertions with path-specific get_registered_routes() filter checks — avoids REGISTERED_ROUTES global contamination across parallel tests without requiring --test-threads=1"
  - "Unique path prefixes in tests (e.g. /api-d01, /api-d04) prevent collision with other concurrent test registrations into the shared global registry"
  - "Did not touch GroupItem::NestedGroup recursion site (lines 720-728) per Edit F — full_prefix is already trailing-slash-normalized after Edit B so the recursion accumulates correctly"

requirements-completed: [D-01, D-02, D-03, D-04, D-06, D-07, D-08]

duration: ~30min
completed: 2026-04-21
---

# Phase 144 Plan 02: Apply combine_group_path + alias inserts in macro-based group routing Summary

**Five `pub(crate)` alias methods on Router plus full GroupDef::register_with_inherited reshape using the Plan 01 helper — fixes `get!("/", h)` inside a group reaching the handler at both `/prefix` and `/prefix/`.**

## Performance

- **Duration:** ~30 min
- **Started:** 2026-04-21T21:00:00Z
- **Completed:** 2026-04-21T21:31:21Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

### Task 1: insert_{method}_alias methods on Router (router.rs)

Five new `pub(crate)` methods added immediately after `insert_delete`:

```rust
pub(crate) fn insert_get_alias(&mut self, alias_path: &str, handler: Arc<BoxedHandler>, canonical_path: &str)
pub(crate) fn insert_post_alias(...)
pub(crate) fn insert_put_alias(...)
pub(crate) fn insert_patch_alias(...)
pub(crate) fn insert_delete_alias(...)
```

Each stores `(handler, canonical_path.to_string())` in the matchit tree — **no `register_route` call** (D-07 invariant) and **no `ROUTE_REGISTRY` touch** (D-08 invariant). The canonical pattern in the value tuple is the Strategy A mechanism: `server.rs:260` keys middleware lookup by `matchit_value.1`, so a request matched by `/prefix/` resolves middleware under `/prefix`.

### Task 2: GroupDef::register_with_inherited reshape (macros.rs)

Five surgical edits applied:

**Edit A** — Added `use super::path::combine_group_path;` import.

**Edit B** — Parent-prefix trailing-slash strip before concatenation:
```rust
let stripped_parent = parent_prefix.strip_suffix('/').unwrap_or(parent_prefix);
format!("{}{}", stripped_parent, self.prefix)
```
Prevents `/a//b` accumulation in nested groups where outer prefix has trailing slash (D-06, Pitfall 3).

**Edit C** — Replaced the buggy `full_path` branch with `combine_group_path` call:
```rust
let (canonical, alternate) = combine_group_path(&full_prefix, &converted_route_path);
let canonical_path: &'static str = Box::leak(canonical.into_boxed_str());
let alternate_path: Option<&'static str> = alternate.map(|s| Box::leak(s.into_boxed_str()) as &'static str);
```

**Edit D** — Canonical-first + alias-second insert across all five HTTP verbs:
```rust
router.insert_get(canonical_path, route.handler.clone());
if let Some(alt) = alternate_path {
    router.insert_get_alias(alt, route.handler, canonical_path);
}
```

**Edit E** — All downstream uses of `full_path` replaced with `canonical_path`: `register_route_name`, `update_route_mcp`, `add_middleware` (both inherited and route-specific loops).

Also removed `#[allow(dead_code)]` from `combine_group_path` in `path.rs` — the function now has a caller.

### Task 2: 7 new tests (Edit G)

| Test | D-XX | Behavior verified |
|------|------|-------------------|
| `group_root_handler_matches_both_variants` | D-01, D-07 | `/api-d01` and `/api-d01/` both match; alternate carries canonical pattern; exactly 1 RouteInfo |
| `root_prefix_root_handler_is_single_slash` | D-02 | `group!("/", { get!("/", h) })` → `/` matches; `//` does not |
| `trailing_slash_prefix_is_stripped` | D-03 | `group!("/api/", ...)` → `/api/x` matches; `/api//x` does not |
| `non_root_prefix_non_root_path_unchanged` | D-04, D-07 | `/api-d04/users` matches; `/api-d04/users/` does not; exactly 1 RouteInfo |
| `nested_group_root_matches_both_variants` | D-06, Pitfall 3 | Both `/a/b` and `/a/b/` match for clean and trailing-slash outer prefixes |
| `named_route_resolves_to_canonical` | D-08 | `route("home_canonical_test", &[])` returns `"/api"`, not `"/api/"` |
| `top_level_root_route_is_single_slash` | regression | Top-level `Router::new().get("/", h)` matches `/`; `//` does not |

## Final Test Count

```
cargo test -p ferro-rs --lib --features json-ui routing:: -- --nocapture
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 409 filtered out
```

Breakdown: 8 pre-existing routing tests + 7 new tests from this plan + 1 `combine_group_path_matrix` from Plan 01 = 16 total.

## Task Commits

1. **Task 1: insert_{method}_alias on Router** — `db530f63`
2. **Task 2: combine_group_path + alias inserts + 7 tests** — `847ef183`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test delta assertions replaced with path-specific registry checks**
- **Found during:** Task 2 test run
- **Issue:** `REGISTERED_ROUTES` is a global static shared across all test threads. Serial tests (via `#[serial_test::serial]`) are only serialized relative to other `#[serial]` tests — non-serial tests still run concurrently, causing `after - before` deltas to be 3 or 5 instead of 1.
- **Fix:** Replaced delta assertions with `get_registered_routes().iter().filter(|r| r.path == "/specific-path").count()` checks on unique path prefixes (e.g. `/api-d01`, `/api-d04`) that cannot be registered by other tests. Also dropped the `#[serial_test::serial]` attribute since it is no longer needed.
- **Files modified:** `framework/src/routing/macros.rs`
- **Commit:** `847ef183`

---

**Total deviations:** 1 auto-fixed (Rule 1 — test correctness)
**Impact on plan:** All 7 required tests pass. The fix makes tests robust under parallel execution without requiring `--test-threads=1`.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. All changes are in the routing registration path (compile-time / startup, not request-time). The T-144-12 middleware-bypass mitigation (Strategy A) is implemented as specified: alias leaves store canonical pattern, verifiable via `grep -c "canonical_path.to_string()" framework/src/routing/router.rs` → 5.

## Known Stubs

None. This plan adds no UI components or data-fetching code.

## Self-Check: PASSED

- FOUND: `framework/src/routing/router.rs` — 5 alias methods present
- FOUND: `framework/src/routing/macros.rs` — combine_group_path import + call + 7 tests present
- FOUND: `framework/src/routing/path.rs` — #[allow(dead_code)] removed
- FOUND: commit `db530f63` (Task 1)
- FOUND: commit `847ef183` (Task 2)
- VERIFIED: `cargo test -p ferro-rs --lib --features json-ui routing::` → 16 passed, 0 failed
- VERIFIED: `cargo clippy -p ferro-rs --all-targets --features json-ui -- -D warnings` → 0 warnings
