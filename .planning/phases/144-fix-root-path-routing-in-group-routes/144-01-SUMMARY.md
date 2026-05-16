---
phase: 144-fix-root-path-routing-in-group-routes
plan: 01
subsystem: routing
tags: [routing, group, path-combination, unit-test]

requires: []
provides:
  - "pub(crate) fn combine_group_path(prefix, route_path) -> (canonical, Option<alternate>) in framework/src/routing/path.rs"
  - "mod path; declaration in framework/src/routing/mod.rs"
affects:
  - 144-02
  - 144-03

tech-stack:
  added: []
  patterns:
    - "Sibling private module pattern: pub(crate) helper in its own file, declared via mod in mod.rs, consumed via use super::path::combine_group_path in sibling modules"

key-files:
  created:
    - framework/src/routing/path.rs
  modified:
    - framework/src/routing/mod.rs

key-decisions:
  - "Helper declared pub(crate) — not re-exported via pub use. Consumers in Plans 02 and 03 use super::path::combine_group_path directly."
  - "added #[allow(dead_code)] to suppress the dead-code lint while Plans 02 and 03 (the consumers) are not yet wired — attribute will be removed once consumers land."

patterns-established:
  - "combine_group_path is the single source of truth for group-prefix + route-path combination. Both macros.rs (GroupDef) and group.rs (GroupBuilder) must import it rather than implement their own logic."

requirements-completed: [D-02, D-03, D-04, D-09]

duration: ~15min
completed: 2026-04-21
---

# Phase 144 Plan 01: Create combine_group_path helper module Summary

**Pure-string path-combination helper centralizing group-prefix + route-path semantics for both group implementations, with 8-row D-09 matrix test.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-04-21T20:45:00Z
- **Completed:** 2026-04-21T21:00:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Created `framework/src/routing/path.rs` with `pub(crate) fn combine_group_path` — the single source of truth for how a group prefix combines with a nested route path
- Declared `mod path;` in `framework/src/routing/mod.rs` alongside the existing sibling module declarations
- All 8 rows of the D-09 matrix pass in `combine_group_path_matrix` (verified via `cargo test` and standalone `rustc` verification against the exact function)

## Helper Signature Shipped

```rust
pub(crate) fn combine_group_path(prefix: &str, route_path: &str) -> (String, Option<String>)
```

## 8-Row D-09 Matrix (all passing)

| Row | prefix | route_path | canonical | alternate |
|-----|--------|-----------|-----------|-----------|
| 1 | `""` | `"/"` | `"/"` | None |
| 2 | `"/"` | `"/"` | `"/"` | None (root-in-root, D-02) |
| 3 | `"/"` | `"/x"` | `"/x"` | None (regression) |
| 4 | `"/api"` | `"/"` | `"/api"` | Some(`"/api/"`) (D-01 core) |
| 5 | `"/api"` | `"/x"` | `"/api/x"` | None (D-04 regression) |
| 6 | `"/api/"` | `"/x"` | `"/api/x"` | None (D-03 trailing-slash strip) |
| 7 | `"/api/"` | `"/"` | `"/api"` | Some(`"/api/"`) (D-03 + D-01) |
| 8 | `"/s/{slug}"` | `"/"` | `"/s/{slug}"` | Some(`"/s/{slug}/"`) (gestiscilo reproducer) |

## Task Commits

1. **Tasks 1 & 2: Create path.rs + add mod path; in mod.rs** - `66c109b0` (feat)

**Plan metadata:** (committed below)

## Files Created/Modified

- `framework/src/routing/path.rs` - New module: `combine_group_path` helper + `#[cfg(test)] mod tests` with 8-row matrix
- `framework/src/routing/mod.rs` - Added `mod path;` declaration in alphabetical order with sibling modules

## Decisions Made

- Added `#[allow(dead_code)]` to `combine_group_path` to suppress the dead-code lint cleanly. The function is `pub(crate)` but has no callers yet — Plans 02 and 03 wire the callers. The attribute will be removed once those plans land.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added #[allow(dead_code)] to suppress dead-code warning**
- **Found during:** Task 1 verification (clippy run)
- **Issue:** `pub(crate) fn combine_group_path` triggers `-D dead_code` warning because Plans 02 and 03 (the callers) are not yet present. Clippy acceptance criterion required zero warnings.
- **Fix:** Added `#[allow(dead_code)]` with a comment explaining the forward-declaration intent. The attribute is intentionally temporary.
- **Files modified:** `framework/src/routing/path.rs`
- **Verification:** `cargo clippy -p ferro-rs --lib --features json-ui -j 1 -- -D warnings` exits 0
- **Committed in:** `66c109b0` (task commit)

---

**Total deviations:** 1 auto-fixed (Rule 2 - correctness / lint compliance)
**Impact on plan:** The allow attribute is a necessary forward-declaration marker. It is the standard pattern in this codebase for helpers that ship before their consumers.

## Issues Encountered

- **Disk at 100% capacity** during verification. The worktree target directory contained 331MB of duplicate rlib artifacts from prior failed builds (multiple compilation attempts across feature sets). Resolved by identifying and removing older duplicate rlibs (same crate, different hash suffix) to free ~331MB. Clippy then ran in metadata-only mode (`cargo clippy` emits no `.o` files) without disk exhaustion. Test run (`cargo test`) needed a further sweep of `.o` intermediates between compile attempts, after which it succeeded.

## Next Phase Readiness

- `combine_group_path` is accessible to Plans 02 and 03 via `use super::path::combine_group_path;`
- Plans 02 and 03 must remove `#[allow(dead_code)]` once they import the function
- No blockers

---
*Phase: 144-fix-root-path-routing-in-group-routes*
*Completed: 2026-04-21*

## Self-Check: PASSED

- FOUND: `framework/src/routing/path.rs`
- FOUND: `framework/src/routing/mod.rs`
- FOUND: `.planning/phases/144-fix-root-path-routing-in-group-routes/144-01-SUMMARY.md`
- FOUND: commit `66c109b0`
