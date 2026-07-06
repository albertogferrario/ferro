---
phase: 157-migration-deploy-safety-backend-portable-backfill-helpers-fe
fixed_at: 2026-05-14T00:00:00Z
review_path: .planning/phases/157-migration-deploy-safety-backend-portable-backfill-helpers-fe/157-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 157: Code Review Fix Report

**Fixed at:** 2026-05-14T00:00:00Z
**Source review:** .planning/phases/157-migration-deploy-safety-backend-portable-backfill-helpers-fe/157-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 3
- Fixed: 3
- Skipped: 0

## Fixed Issues

### WR-01: Integer truncation produces wrong output for odd `hex_len` values

**Files modified:** `ferro-migration/src/backfill.rs`
**Commit:** 71fc848d
**Applied fix:** Added an early return in `sql_for_random_hex` that rejects odd `hex_len` values with `Error::UnsupportedBackend`. Updated the `backfill_random_hex` doc comment to document the even-only constraint. Added `random_hex_odd_hex_len_returns_error` test covering both SQLite and Postgres backends with `hex_len = 5`. All 8 unit tests pass.

---

### WR-02: Internal tenant identity and incident reference committed to a framework doc comment

**Files modified:** `framework/src/app.rs`
**Commit:** bd94964c
**Applied fix:** Replaced the doc comment on `run_migrations_silent` that contained `gestiscilo-it 2026-05-13 incident` with a neutral description of the invariant: "prevents the server from accepting traffic with a stale schema."

---

### WR-03: `.expect()` in server startup path panics without actionable context

**Files modified:** `framework/src/app.rs`, `ferro-cli/src/templates/files/backend/main.rs.tpl`
**Commit:** bd94964c
**Applied fix:** Replaced `.expect("Failed to start server")` in `Application::run_server_internal` and in the template's `run_server()` function with `if let Err(e) { eprintln!(...); std::process::exit(1); }`, matching the `run_migrations_silent` pattern already established in this phase.

---

_Fixed: 2026-05-14T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
