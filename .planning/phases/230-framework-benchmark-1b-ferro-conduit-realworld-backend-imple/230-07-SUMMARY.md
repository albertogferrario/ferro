---
phase: 230-framework-benchmark-1b-ferro-conduit
plan: 07
subsystem: benchmark/conduit
tags: [conduit, realworld, laravel, vendored, newman, perf, static-compression, jwt-carveout, like-for-like]
requires:
  - "230-06: Ferro Conduit feature-complete (full single-app Newman 422/422)"
  - "229: ferro-bench-toolbox (oha 1.9.0 + tokei 12.1.2), perf/static harness, laravel-micro serving recipes"
provides:
  - "Vendored + pinned Laravel RealWorld backend (f1amy @ c14fb83)"
  - "Dual-Newman like-for-like conformance (Ferro 422/422; Laravel 340/382)"
  - "Static compression with hand-rolled JWT carved out on BOTH sides"
  - "Perf: Ferro vs Laravel php-fpm + Octane on shared Postgres"
  - "benchmark/results/conduit/RESULTS.md (honest framing, four VALIDATION boxes signed)"
affects:
  - "Phase 230 COMPLETE — closes framework-benchmark-1b"
tech-stack:
  added:
    - "f1amy/laravel-realworld-example-app (vendored, MIT, Laravel 9)"
    - "Laravel Octane 1.x + RoadRunner 2.12.3 (conduit variant)"
  patterns:
    - "Static carveout (count_static.run_with_carveout): framework_provided = total - hand_rolled"
    - "Shared Postgres server, one logical DB per app (isolated schema, same host resources)"
    - "Globals-seeding harness for the vintage Conduit collection (register user + dragons article)"
key-files:
  created:
    - benchmark/apps/laravel-conduit/ (vendored tree, 177 files)
    - benchmark/apps/laravel-conduit/PINNED_COMMIT.md
    - benchmark/apps/laravel-conduit/Dockerfile
    - benchmark/apps/laravel-conduit/Dockerfile.octane
    - benchmark/apps/conduit-db-init/01-create-conduit-dbs.sql
    - benchmark/conduit-compose.yaml
    - benchmark/contracts/conduit/seed_and_run.sh
    - benchmark/contracts/conduit/perf_seed.sh
    - benchmark/results/conduit/RESULTS.md
    - benchmark/results/conduit/meta.json
    - benchmark/results/conduit/{newman,static,perf}-*.json
  modified:
    - benchmark/harness/static/count_static.py
    - benchmark/harness/static/test_count_static.py
    - benchmark/harness/perf/run_perf.py
decisions:
  - "Vendored f1amy (modern Laravel 9, maintained) over gothinkster (archived Laravel 5.x) — both use Laravel default validation messages, so gothinkster would not pass the frozen collection's strict error strings either, and it needs legacy-PHP Docker work"
  - "Shared Postgres SERVER with one logical DB per app — both own the same canonical Conduit schema, so a single DB collides; isolated schema is the standard like-for-like setup"
  - "Did NOT modify vendored Laravel app logic to force 422/422; reported the 42 strict-error-contract divergences honestly per D-10/T-230-22"
  - "Carved out the hand-rolled JWT on BOTH sides (Ferro 89, Laravel 353) — both apps hand-roll JWT in app code (a symmetry, not the anticipated asymmetry)"
metrics:
  duration: ~75m
  completed: 2026-06-15
  tasks: 3
  files: 18
  commits: 5
---

# Phase 230 Plan 07: Laravel Parity + Cross-Impl Harness — Conduit Benchmark Summary

The Conduit (RealWorld) like-for-like comparison is complete: a community Laravel
backend is vendored and pinned, the full Conduit Newman collection ran against both
backends, static compression counts the hand-rolled JWT separately on both sides, and
the perf workload ran Ferro vs Laravel (php-fpm + Octane) on a shared Postgres server.
`benchmark/results/conduit/RESULTS.md` reports it all in neutral voice with honest
caveats and the four VALIDATION sign-off boxes checked. **This closes Phase 230.**

## Pinned Laravel backend

- **f1amy/laravel-realworld-example-app @ `c14fb8370b71a42a3a74b8ea936a1f96b2af9d69`**
  (MIT, Laravel 9, PHP 8.2), `.git` removed — vendored, not authored.
- Evaluated vs gothinkster (archived Laravel 5.x): both use Laravel's stock validation
  messages, so neither reproduces the frozen collection's `"can't be blank"` strings;
  f1amy is modern and needs no legacy-PHP image, so it was chosen. Rationale + evidence
  in `benchmark/apps/laravel-conduit/PINNED_COMMIT.md`.

## Headline numbers

**Dual Newman (full Conduit collection, identical globals seeding):**

| Backend | Requests | Assertions | Failed |
|---------|---------:|-----------:|-------:|
| Ferro Conduit | 75/75 | **422/422** | **0** |
| Laravel Conduit (f1amy) | 75/75 | 340/382 | 42 |

Ferro is fully green. The vendored Laravel app passes the functional contract; the 42
failing assertions are strict error-contract divergences (Laravel's default validation
wording / error-envelope shape / 200-vs-204 delete / `articlesCount` page-vs-total) —
documented honestly, vendored logic unmodified.

**Static compression (JWT carved out — D-10 honesty):**

| | Ferro | Laravel |
|---|---:|---:|
| Total app code | 1,957 | 4,595 |
| Hand-rolled JWT (carveout) | 89 | 353 |
| **Framework-provided app code** | **1,868** | **4,242** |
| Files | 37 | 142 |

Both apps hand-roll JWT in application code (Ferro is session-based; f1amy does not use
`tymon/jwt-auth`) — a symmetry, both carveouts shown. Ferro expresses the same backend
in ~2.3x less framework-provided application code.

**Perf (oha, 30s, c=256, shared Postgres):**

| Endpoint | Ferro | Laravel fpm | Octane |
|----------|------:|------------:|-------:|
| /api/tags | **9,616** | 1,755 | 2,163 |
| /api/articles?limit=20 | 273 | **2,552** | 2,096 |
| /api/profiles/celeb | **11,156** | 2,671 | 1,792 |

Honest mixed result: Ferro wins tags + profiles ~5-6x, but **loses the article list ~9x**
because the Ferro Conduit's handler does per-article N+1 relation queries where f1amy
eager-loads. Reported as a concrete action item, not hidden.

## Tasks → commits

1. **Task 1: Vendor + pin Laravel; Dockerize (php-fpm + octane); Conduit compose** — `730c0881` (feat)
2. **Task 2: Harness — static JWT carveout + perf Conduit preset** — `20e84710` (feat)
3. **Config-only Laravel boot fixes (PHP 8.2 + carbon bump + isolated DBs)** — `4a48e949` (fix)
4. **Task 3: Dual Newman + static + perf; RESULTS.md** — `4910cd3d` (feat)

## Deviations from Plan

### Auto-fixed Issues (Rule 3 — blocking, config-only)

**1. [Rule 3 - Blocking] Shared `conduit` DB collided (both apps own `users`)**
- **Found during:** stack bring-up (checkpoint:human-verify).
- **Issue:** A single shared `conduit` database made the second app's `users` migration
  fail (`relation "users" already exists`).
- **Fix:** Shared Postgres *server* with one logical DB per app (`conduit_ferro`,
  `conduit_laravel`) via `apps/conduit-db-init/`. Still like-for-like (same engine, host).
- **Commit:** `4a48e949`.

**2. [Rule 3 - Blocking] Laravel 500 on every request (Carbon vs modern PHP)**
- **Issue:** Vendored `nesbot/carbon` 2.58.0 raises `Carbon::setLastErrors(... bool given)`
  on PHP 8.2/8.3 (`DateTime::getLastErrors()` returns `false`).
- **Fix:** PHP 8.2 image + `composer update nesbot/carbon` → 2.73.0 (transitive bump, no
  app-logic change). Documented in PINNED_COMMIT.md as config-only.
- **Commit:** `4a48e949`.

**3. [Rule 3 - Blocking] Octane "RoadRunner not installed"**
- **Issue:** Octane 1.x's roadrunner runtime check requires the `spiral/roadrunner` v2
  meta-package by name (only `roadrunner-cli`/`-worker` were present); octane config also
  unpublished.
- **Fix:** `composer require "spiral/roadrunner:^2.0"` in Dockerfile.octane; entrypoint
  publishes octane config + symlinks `/app/rr` onto PATH.
- **Commit:** `4910cd3d`.

### Deliberate honesty deviation (not auto-fixed)

**"Both green" not achieved for the vendored Laravel app.** Reaching 422/422 would
require editing the vendored app's logic to match the frozen collection's exact error
strings/status codes. The benchmark forbids modifying vendored logic, and the honesty
rule (D-10 / T-230-22: "unfair/overstated comparison → mitigate") prefers reporting the
gap. The 42-assertion divergence is fully characterized in RESULTS.md §1. This is the
correct disposition, not a failure to complete the work.

## Known Stubs

None. All result artifacts are real, produced from live runs against both backends.

## Threat Flags

None beyond the plan's threat model. Benchmark secrets (`JWT_SECRET`/`APP_KEY`/DB creds)
are throwaway local-compose values, documented not-production-grade (T-230-21 accepted).

## Verification

- Harness pytest: static 3/3 (incl. new carveout invariant), perf 2/2 — green.
- Newman: Ferro 422/422 (0 failed); Laravel 340/382 (42 failed, characterized).
- Static carveout JSON contains `hand_rolled` + `framework_provided_code_lines` on both.
- Perf JSON present for ferro + laravel-fpm + laravel-octane (all success_rate 1.0).
- RESULTS.md contains "not framework-provided" + four checked VALIDATION boxes.
- Containers torn down (`down -v`); no vendor/target/node_modules/.env tracked in git.

## Self-Check: PASSED

- Created files present (verified below).
- Commits present: `730c0881`, `20e84710`, `4a48e949`, `4910cd3d`.
