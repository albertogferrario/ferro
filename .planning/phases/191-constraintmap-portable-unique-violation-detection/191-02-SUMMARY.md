---
phase: 191-constraintmap-portable-unique-violation-detection
plan: "02"
subsystem: framework/validation
tags: [constraint-map, unique-violation, sea-orm, sqlite, integration-tests, toctou]
dependency_graph:
  requires:
    - phase: 191-constraintmap-portable-unique-violation-detection/01
      provides: "ConstraintMap::try_map, MapConstraintExt — the public surface the tests consume"
  provides:
    - "SQLite integration test suite: SC1/SC2/SC3/SC4 automated coverage"
    - "constraint_map_fixture.rs: in-memory SQLite + UNIQUE INDEX helper"
    - "191-VERIFICATION.md: Postgres manual gate (constraint()-based) + SC evidence table"
  affects:
    - framework/tests/
    - 191-constraintmap-portable-unique-violation-detection/191-VERIFICATION.md
tech-stack:
  added: []
  patterns:
    - mod-include-sibling-fixture (mod constraint_map_fixture; in integration test file)
    - real-db-err-to-try_map (feed actual driver DbErr to public API instead of constructing manually)
    - toctou-simulation (seed winning insert + losing insert → capture DbErr → assert field error)
    - postgres-manual-gate-doc (mirrors Phase 190 pattern for CI-unavailable Postgres path)
key-files:
  created:
    - framework/tests/constraint_map_fixture.rs
    - framework/tests/constraint_map_integration.rs
    - .planning/phases/191-constraintmap-portable-unique-violation-detection/191-VERIFICATION.md
  modified:
    - framework/src/validation/constraint_map.rs (import ordering fix for cargo fmt)
key-decisions:
  - "Four integration tests cover all SCs: TOCTOU simulation (SC4), SQLite message-parse identity (SC3), non-UNIQUE passthrough (SC2), unregistered UNIQUE passthrough (SC2)"
  - "sqlite_identity_match_via_message uses a deliberately-wrong Postgres constraint name to prove the match came from the SQLite message-parse path only"
  - "Postgres manual gate documented in 191-VERIFICATION.md without an automated test (D-12) — omitting .sqlite() from the gate sample proves only constraint() can match"
  - "Import ordering fix (sea_orm::error::SqlErr before use sea_orm::{..}) bundled with Task 3 as a Rule 1 auto-fix"
requirements-completed: [VALID-04, VALID-05]
duration: 565s
completed: 2026-06-09
---

# Phase 191 Plan 02: Integration Tests + Postgres Manual Gate Summary

**SQLite integration suite proves the full ConstraintMap defensive-layer contract: TOCTOU simulation, message-parse identity, and passthrough, all against real in-memory SQLite driver errors.**

## Performance

- **Duration:** ~565s
- **Started:** 2026-06-09T16:17:24Z
- **Completed:** 2026-06-09T16:26:49Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Created `constraint_map_fixture.rs` with `init_constraint_db()` + `exec_sql()` helpers: in-memory SQLite singleton with a NON-PK UNIQUE INDEX on `cw.slug` so duplicate INSERTs raise real `SQLITE_CONSTRAINT_UNIQUE` (2067) driver errors.
- Created `constraint_map_integration.rs` with 4 `#[serial]` integration tests covering SC1/SC2/SC3-SQLite/SC4 — all green against the Wave 1 `ConstraintMap::try_map` implementation.
- Created `191-VERIFICATION.md` documenting the SC evidence table and the Postgres manual gate (structured `constraint()` path, no message parse), mirroring the Phase 190 gate format.
- Full quality gate green: `cargo fmt --all -- --check`, `cargo clippy --all --all-targets -- -D warnings`, `cargo test --all-features`.

## Task Commits

1. **Task 1: UNIQUE-indexed in-memory SQLite fixture** - `1d58277d` (feat)
2. **Task 2: SQLite integration tests** - `7cf4c092` (test)
3. **Task 3: Postgres manual gate doc + full quality gate** - `7f0c4024` (docs)

## Files Created/Modified

- `framework/tests/constraint_map_fixture.rs` — in-memory SQLite scratch table (`cw`) with NON-PK UNIQUE INDEX on `slug`; exposes `init_constraint_db()` and `exec_sql()`
- `framework/tests/constraint_map_integration.rs` — 4 integration tests: `toctou_simulation_maps_to_field_error`, `sqlite_identity_match_via_message`, `non_unique_error_passes_through_unchanged`, `unregistered_unique_passes_through`
- `.planning/phases/191-constraintmap-portable-unique-violation-detection/191-VERIFICATION.md` — SC evidence table + Postgres manual gate + sign-off block
- `framework/src/validation/constraint_map.rs` — import ordering fix (`sea_orm::error::SqlErr` before `use sea_orm::{…}`)

## Decisions Made

- `sqlite_identity_match_via_message` uses `"never_matches_pg_name"` as the Postgres constraint name to ensure the match can only come from the SQLite `.sqlite("cw.slug")` message-parse path. This isolates and proves SC3 SQLite independently.
- Postgres manual gate in 191-VERIFICATION.md intentionally omits `.sqlite(...)` from the sample `ConstraintMap` so the match can only come from `e.constraint()` — proving the Postgres identity path distinctly.
- Kept `cw` / `cw_slug_unique` as test-fixture identifiers (arbitrary scratch names in `tests/`), outside the SC5 project-agnostic audit scope which covers only `framework/src/validation/constraint_map.rs`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed import ordering in constraint_map.rs to pass cargo fmt**

- **Found during:** Task 3 (full quality gate — `cargo fmt --all -- --check`)
- **Issue:** `use sea_orm::{DbErr, RuntimeErr, SqlxError};` appeared before `use sea_orm::error::SqlErr;` — rustfmt requires `sea_orm::error::SqlErr` first (alphabetical within the same crate's module hierarchy).
- **Fix:** Swapped the two use lines so `use sea_orm::error::SqlErr;` precedes `use sea_orm::{DbErr, RuntimeErr, SqlxError};`.
- **Files modified:** `framework/src/validation/constraint_map.rs`
- **Verification:** `cargo fmt --all -- --check` exits 0.
- **Committed in:** `7f0c4024` (Task 3 commit)

---

**Total deviations:** 1 auto-fixed (1 Rule 1 — formatting bug caught by full gate)
**Impact on plan:** Necessary formatting fix; no behavioral change. Plan executed as written otherwise.

## Issues Encountered

None beyond the fmt ordering fix above.

## Known Stubs

None. All four integration tests wire real driver errors to real public API calls. No mock data, no placeholder returns, no TODOs.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes introduced. Test files feed DB driver errors to a pure-Rust type-matching function. No new threat surface.

## Next Phase Readiness

- Phase 191 is complete: `ConstraintMap` + `MapConstraintExt` (Plan 01) and the full integration test suite + Postgres gate (Plan 02) are all committed and green.
- REQUIREMENTS.md VALID-04 and VALID-05 are satisfied.
- Phase 192 (ferro-mcp `action_handler` template + validation docs showing the two layers together — VALID-06) is unblocked.

---

*Phase: 191-constraintmap-portable-unique-violation-detection*
*Completed: 2026-06-09*
