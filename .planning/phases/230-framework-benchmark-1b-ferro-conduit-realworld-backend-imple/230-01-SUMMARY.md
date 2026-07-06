---
phase: 230-framework-benchmark-1b-ferro-conduit
plan: 01
subsystem: benchmark/ferro-conduit
tags: [benchmark, conduit, jwt, middleware, scaffold]
requires: []
provides:
  - ferro-conduit standalone app skeleton (release-buildable, Docker-buildable)
  - hand-rolled JWT module (src/jwt.rs) — isolated for Plan 07 static count
  - JwtAuthMiddleware + OptionalJwtMiddleware + UserId extension type
  - health endpoint ({"status":"ok"})
affects:
  - root Cargo.toml [workspace] exclude (defensive guard)
tech-stack:
  added: [jsonwebtoken@9, slug@0.1]
  patterns:
    - "Standalone [[bin]] package outside the root ferro workspace (ferro-micro template)"
    - "Custom Middleware via request extension map (UserId), read by req.get::<UserId>() — never AuthUser<T> (session-bound)"
    - "Hand-rolled HS256 JWT isolated in one module, labeled non-framework-provided"
key-files:
  created:
    - benchmark/apps/ferro-conduit/Cargo.toml
    - benchmark/apps/ferro-conduit/.env.example
    - benchmark/apps/ferro-conduit/Dockerfile
    - benchmark/apps/ferro-conduit/src/main.rs
    - benchmark/apps/ferro-conduit/src/bootstrap.rs
    - benchmark/apps/ferro-conduit/src/config/mod.rs
    - benchmark/apps/ferro-conduit/src/controllers/mod.rs
    - benchmark/apps/ferro-conduit/src/controllers/health.rs
    - benchmark/apps/ferro-conduit/src/jwt.rs
    - benchmark/apps/ferro-conduit/src/middleware/mod.rs
    - benchmark/apps/ferro-conduit/src/middleware/jwt_auth.rs
    - benchmark/apps/ferro-conduit/src/middleware/optional_jwt.rs
    - benchmark/apps/ferro-conduit/src/migrations/mod.rs
  modified:
    - Cargo.toml
decisions:
  - "Dropped the db:seed subcommand + seeders/models modules from the ferro-micro template — not needed in Wave 1; the plan's module set excludes them"
  - "Expiry test mints at -120s TTL, not -1s: jsonwebtoken's default Validation carries a 60s leeway so a -1s token still validates"
  - "Crate-level #![allow(dead_code)] in main.rs: Wave 1 scaffolds JWT/middlewares/health that later waves (Plans 02-06) wire into routes; required for clippy -D warnings to pass"
  - "docker build deferred to Plan 06 per the plan; Dockerfile reviewed against ferro-micro's proven multi-stage pattern (identical, binary path changed)"
metrics:
  duration: ~6m
  completed: 2026-06-15
---

# Phase 230 Plan 01: Ferro Conduit Scaffold + JWT Auth Foundation Summary

Standalone `ferro-conduit` benchmark app scaffolded outside the root workspace, with a hand-rolled HS256 JWT module (isolated in `src/jwt.rs`) and two JWT middlewares routing a verified `UserId` into the request extension map — the foundation every later Conduit wave builds on.

## What Was Built

- **Task 1 — App skeleton** (`c50b8f08`): standalone `[[bin]]` package mirroring `ferro-micro` (clap subcommands, `Migrator` wiring with an empty migration vec for Plan 02, `Cors::permissive()` + `DB::init()` in bootstrap, Dockerfile, `.env.example`, `/health` route returning `{"status":"ok"}`). Root `Cargo.toml` gained a `[workspace] exclude` listing both `ferro-conduit` and `ferro-micro` as a defensive guard. `cargo build --release` succeeds; the `target/release/ferro-conduit` binary (15 MB) is produced.
- **Task 2 — JWT module** (`dec7a069`): `JwtClaims` + `mint_token` / `mint_token_with_ttl` / `decode_token` (HS256, `jsonwebtoken` v9), plus `jwt_secret()`. Module doc labels it HAND-ROLLED / not-framework-provided. 3 unit tests: round-trip, expiry-rejected, bad-signature-rejected.
- **Task 3 — JWT middlewares** (`e53f81f5`): `JwtAuthMiddleware` (Conduit 401 envelope on missing/invalid), `OptionalJwtMiddleware` (inserts `UserId` when present, never rejects), a shared `extract_user_id` helper (unit-tested: no-header→None, valid→Some(UserId(7)), bad→None), and the `UserId(pub i64)` extension type.

## Verification Evidence

- `cargo build --release` (standalone, no root-workspace inheritance): succeeded in 1m38s.
- `cargo test`: 6 passed (jwt:: 3 + middleware:: 3), 0 failed.
- `cargo clippy --release -- -D warnings`: clean.
- No workspace table in `benchmark/apps/ferro-conduit/Cargo.toml` (confirmed; the one literal `[workspace]` occurrence is inside a comment, reworded to avoid grep false-positives).

## JWT Isolation Note (for Plan 07 static count)

The ONE hand-rolled, non-framework-provided capability is `benchmark/apps/ferro-conduit/src/jwt.rs` (~95 lines incl. tests; ~60 lines non-test). The middleware header-parse/decode glue lives in `src/middleware/{mod.rs,jwt_auth.rs,optional_jwt.rs}` and depends on `jwt.rs`. Plan 07's static-compression report should carve these out as "not framework-provided" per D-10 honesty.

## Docker

`docker build` was DEFERRED to Plan 06 (per the plan's verification note). Docker is available on this host, but a second heavy build was avoided to keep CPU work sequential. The Dockerfile is a structural copy of `ferro-micro`'s proven multi-stage build with the binary path changed to `ferro-conduit`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Expiry test was non-deterministic at -1s TTL**
- **Found during:** Task 2 (first test run)
- **Issue:** `jsonwebtoken`'s default `Validation` applies a 60s `leeway`, so a token expired 1 second ago still validated and the expiry test failed.
- **Fix:** Mint the expiry-test token at -120s TTL (beyond the default leeway). Documented in the test comment.
- **Files modified:** src/jwt.rs
- **Commit:** dec7a069

**2. [Rule 3 - Blocking] clippy -D warnings failed on scaffold (dead_code + uninlined_format_args)**
- **Found during:** Task 3 (clippy gate)
- **Issue:** The JWT functions, both middlewares, and the health handler are unused until later waves wire them into routes — `dead_code` blocked the `-D warnings` gate. A pre-existing `println!("...{}", steps)` in main.rs also tripped `uninlined_format_args`.
- **Fix:** Added a crate-level `#![allow(dead_code)]` (scoped with a comment naming Plans 02-06 as consumers) and inlined the format arg.
- **Files modified:** src/main.rs
- **Commit:** e53f81f5

### Scope Adjustment

- **Dropped `db:seed` + `mod seeders` / `mod models`** from the ferro-micro template. The plan's module set (`bootstrap, config, controllers, jwt, middleware, migrations, routes`) excludes them and Wave 1 has no models/seeders yet. Later waves add models; a seeder path can return if needed.

## Self-Check: PASSED

- All 13 created files exist on disk; root Cargo.toml modified.
- Commits c50b8f08, dec7a069, e53f81f5 present in `git log`.
- 6 tests pass; clippy clean; release binary built.
