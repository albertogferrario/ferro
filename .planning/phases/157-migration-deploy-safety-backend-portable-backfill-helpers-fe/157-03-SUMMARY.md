---
plan: 157-03
phase: 157
status: complete
started: 2026-05-14T03:00:00Z
completed: 2026-05-14T13:43:46Z
self_check: PASSED
---

## Summary

Added the `migrate_gate` doctor check that errors (not warns) when a project has migrations but no PRE_DEPLOY migrate job configured in `.do/app.yaml`. After this plan, `ferro doctor --deploy` catches the gestiscilo-it failure scenario at pre-push time rather than at deploy time.

## What Was Built

- **`ferro-cli/src/doctor/checks/migrate_gate.rs`** — `MigrateGateCheck` implementing `DoctorCheck` for `CheckCategory::Deploy`; `check_impl` line-scans `app.yaml` for a PRE_DEPLOY migrate job; skips when no `migrations/` dir or no `.do/app.yaml`; 9 unit tests
- **`ferro-cli/src/doctor/checks/mod.rs`** — added `pub mod migrate_gate` and `pub use migrate_gate::MigrateGateCheck`
- **`ferro-cli/src/doctor/registry.rs`** — imported `MigrateGateCheck`, inserted `Box::new(MigrateGateCheck)` after `DockerTemplateDriftCheck` in `default_checks()`, renamed registry tests to `default_checks_returns_twelve_in_declared_order` (len 12) and `deploy_category_filter_returns_three` (3 deploy checks)
- **`ferro-cli/src/doctor/check.rs`** — updated `deploy_names` in the `check_categories_are_consistent` test to include `migrate_gate`

## Key Files

- `ferro-cli/src/doctor/checks/migrate_gate.rs` — full check implementation
- `ferro-cli/src/doctor/registry.rs` — registration and updated tests

## Commits

- `9cabd5d9` — feat(157-03): add migrate_gate doctor check with 9 unit tests
- `a5529368` — feat(157-03): register MigrateGateCheck in default_checks, update registry tests to 12/3

## Deviations

None. All tasks completed as planned.

## Self-Check

- [x] `MigrateGateCheck` returns `Error` when migrations dir exists + app.yaml exists + no PRE_DEPLOY job
- [x] Returns `Ok` (skipped) when no migrations dir
- [x] Returns `Ok` (skipped) when no `.do/app.yaml`
- [x] `default_checks()` returns 12 entries
- [x] Deploy filter returns 3 entries including `migrate_gate`
- [x] Registry tests renamed and updated
