---
plan: 157-01
phase: 157
status: complete
started: 2026-05-14T03:00:00Z
completed: 2026-05-14T13:43:46Z
self_check: PASSED
---

## Summary

Created the `ferro-migration` workspace crate providing backend-portable backfill helpers for use inside SeaORM `MigrationTrait::up` implementations. The crate eliminates the entire class of bug that caused the 2026-05-13 gestiscilo-it production incident by ensuring a single call like `backfill_random_hex(manager, "bookings", "checkin_token", 16).await?` works identically on SQLite and Postgres.

## What Was Built

- **`ferro-migration/src/lib.rs`** — public API re-exporting error and backfill modules
- **`ferro-migration/src/error.rs`** — `MigrationError` enum with `thiserror` derive
- **`ferro-migration/src/backfill.rs`** — `backfill_random_hex` and `backfill_ulid` helpers using backend-dispatched SQL (`hex(randomblob(N))` for SQLite, `encode(gen_random_bytes(N), 'hex')` for Postgres)
- **`ferro-migration/Cargo.toml`** — workspace crate with `sea-orm` and `hex` dependencies
- **`ferro-migration/README.md`** — usage guide with one-liner example
- **`Cargo.toml`** — added `ferro-migration` to workspace members
- **`.github/workflows/publish.yml`** — added `ferro-migration` to WAVE1A_CRATES
- **`CLAUDE.md`** — added `ferro-migration` row to Workspace Structure table

## Key Files

- `ferro-migration/src/backfill.rs` — killer feature: backend-dispatched backfill with unit tests
- `ferro-migration/Cargo.toml` — crate manifest
- `.github/workflows/publish.yml` — CI publish wave

## Commits

- `e3a6f5b9` — feat(157-01): scaffold ferro-migration crate with error type and lib skeleton
- `49ff26c5` — feat(157-01): implement backend-dispatched backfill helpers with unit tests
- `bf8b4c75` — chore(157-01): wire ferro-migration into CI publish Wave 1a and document in CLAUDE.md

## Deviations

None. All tasks completed as planned.

## Self-Check

- [x] `ferro-migration` crate compiles with `cargo build`
- [x] `backfill_random_hex` and `backfill_ulid` implemented with backend dispatch
- [x] Unit tests for both SQLite and Postgres paths present
- [x] Wired into workspace Cargo.toml
- [x] Added to CI publish workflow
- [x] CLAUDE.md updated
