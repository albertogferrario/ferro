---
plan: 157-04
phase: 157
status: complete
started: 2026-05-14T03:00:00Z
completed: 2026-05-14T13:43:46Z
self_check: PASSED
---

## Summary

Fixed the silent-failure anti-pattern in `run_migrations_silent` across all three sites: framework source, sample app, and new-project template. A migration failure at server boot now aborts the process with `std::process::exit(1)`, eliminating the "server starts with stale schema" failure mode that caused the 2026-05-13 gestiscilo-it production incident.

## What Was Built

- **`framework/src/app.rs`** — replaced `eprintln!("Warning: Migration failed")` + continue with `eprintln!("Migration failed: {e}") + process::exit(1)`; added doc comment clarifying abort-on-failure intent (matching `run_seeders` precedent at line 329)
- **`app/src/main.rs`** — same abort pattern in sample app's `run_migrations_silent` free function; removed "Warning:" prefix; switched to `{e}` formatting
- **`ferro-cli/src/templates/files/backend/main.rs.tpl`** — same fix in new-project template; uses `{e}` formatting per workspace standard

## Key Files

- `framework/src/app.rs` — framework `run_migrations_silent` with abort
- `app/src/main.rs` — sample app abort
- `ferro-cli/src/templates/files/backend/main.rs.tpl` — template abort

## Commits

- `b5777e20` — fix(157-04): run_migrations_silent aborts on failure with process::exit(1)
- `59043b8c` — fix(157-04): abort on migration failure in sample app and new-project template

## Deviations

None. All three sites patched with the same `eprintln!("Migration failed: {e}") + std::process::exit(1)` pattern.

## Self-Check

- [x] `framework/src/app.rs` uses `process::exit(1)` on migration failure
- [x] `app/src/main.rs` uses `process::exit(1)` on migration failure
- [x] Template uses `process::exit(1)` on migration failure
- [x] No remaining "Warning: Migration failed" strings across all three trees
- [x] All uses `{e}` formatting (not `{}`)
- [x] Doc comment updated to clarify abort-on-failure behavior
