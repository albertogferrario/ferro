---
phase: 229-framework-benchmark-harness-foundation-1a-build-the-reproduc
plan: 05
subsystem: benchmark
tags: [benchmark, docker, conformance, load-test, oha, tokei, laravel, ferro, results]

# Dependency graph
requires:
  - phase: 229-01
    provides: parse_oha interface + count_static + build_tables contracts
  - phase: 229-02
    provides: harness Python units (parse_perf, count_static, build_tables)
  - phase: 229-03
    provides: ferro-bench-toolbox image (oha 1.9.0 + tokei 12.1.2) + run_perf.py
  - phase: 229-04
    provides: ferro-micro + laravel-micro app images
provides:
  - benchmark/contracts/conformance/test_conformance.py (4-endpoint acceptance gate)
  - benchmark/compose.yaml (postgres:16.4 + both apps, shared DB, healthcheck)
  - benchmark/results/2026-06-15/ (meta.json + perf + static + internal + public)
  - benchmark/README.md "Running the benchmark" section (D-11 reproducibility)
affects:
  - public git (committed raw JSON results + hardware metadata — no secrets)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "compose.yaml: service_healthy depends_on gate for both apps against shared postgres"
    - "laravel artisan serve: config:cache at container startup bakes Docker env vars before worker fork"
    - "build_tables.py: skip non-perf/static files (meta.json) when globbing results dir"

key-files:
  created:
    - benchmark/contracts/conformance/test_conformance.py
    - benchmark/compose.yaml
    - benchmark/results/2026-06-15/meta.json
    - benchmark/results/2026-06-15/perf-ferro.json
    - benchmark/results/2026-06-15/perf-laravel.json
    - benchmark/results/2026-06-15/static-ferro.json
    - benchmark/results/2026-06-15/static-laravel.json
    - benchmark/results/2026-06-15/internal.md
    - benchmark/results/2026-06-15/public.md
  modified:
    - benchmark/apps/ferro-micro/src/controllers/bench.rs
    - benchmark/apps/ferro-micro/src/seeders/world_seeder.rs
    - benchmark/apps/laravel-micro/Dockerfile
    - benchmark/apps/laravel-micro/app/Models/World.php
    - benchmark/apps/laravel-micro/database/migrations/2026_06_15_021350_create_world_table.php
    - benchmark/apps/laravel-micro/database/seeders/WorldSeeder.php
    - benchmark/apps/laravel-micro/routes/web.php
    - benchmark/harness/report/build_tables.py
    - benchmark/README.md

key-decisions:
  - "Host port remapping: ferro→3001, laravel→8001 (host ports 3000/8000 occupied by Next.js dev server and Docker backend)"
  - "laravel artisan serve config:cache at startup: PHP built-in server worker forks inherit process env correctly only after config:cache bakes DB_HOST=db from Docker env into bootstrap/cache/config.php; artisan tinker saw correct env but HTTP workers did not"
  - "Shared DB column name is random_number (snake_case from ferro migration); laravel model/seeder/routes updated to use random_number column, JSON output still maps to randomNumber per contract"
  - "build_tables.py: filter to perf-*/static-* prefixes only; meta.json caused KeyError when loaded as a phantom framework entry"
  - "Laravel world migration: hasTable guard prevents duplicate-table error on shared DB (ferro already created it)"

requirements-completed: []

# Metrics
duration: 25min
completed: 2026-06-15
---

# Phase 229 Plan 05: Conformance + Compose + First Results Run Summary

**Conformance test + compose orchestration + first committed results run: both apps pass 4/4 contract assertions; ferro-micro 148,852 rps on /json vs laravel 98 rps; 4.1x fewer LoC.**

## Performance

- **Duration:** 25 min
- **Started:** 2026-06-15T02:18:58Z
- **Completed:** 2026-06-15T02:43:58Z (approx)
- **Tasks:** 2 (Task 9: conformance + compose; Task 10: results run + report + README)
- **Files created:** 9
- **Files modified:** 9

## Accomplishments

- `benchmark/contracts/conformance/test_conformance.py` asserts all four micro-endpoints against `BASE_URL`; ferro passes 4/4 and laravel passes 4/4 (acceptance gate cleared)
- `benchmark/compose.yaml` orchestrates postgres:16.4 + ferro-micro + laravel-micro on a shared DB with `service_healthy` healthcheck gate
- First committed results run under `results/2026-06-15/`:
  - meta.json: Apple M1 Pro, 8 physical cores, 16 GB RAM, Darwin 23.6.0; oha 1.9.0 / tokei 12.1.2 / postgres 16.4
  - perf-ferro.json: /json 148,852 rps, /db 16,571 rps, /queries 658 rps, /updates 333 rps (all success_rate=1.0)
  - perf-laravel.json: /json 98 rps, /db 74 rps, /queries 66 rps, /updates 49 rps (artisan serve, no Octane)
  - static-ferro.json: 344 LoC, 14 files, 1158 source tokens
  - static-laravel.json: 1427 LoC, 44 files, 8874 source tokens
  - internal.md: full tables; public.md: headline subset + honesty caveat (D-10)
- README "Running the benchmark" section (D-11): 8-step reproducible workflow

## Task Commits

1. **Task 9: Conformance test + compose** — `12ad5a74`
2. **Task 10: First results run + report + README** — `85c18fb7`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ferro-micro: ThreadRng (!Send) held across .await**
- **Found during:** Task 9 (docker compose build ferro-micro — compile error E0277)
- **Issue:** `rand::thread_rng()` was called in async handlers (`db_handler`, `queries`, `updates`) and the `ThreadRng` temporary was alive across `.await` points, violating the `Send` bound required by tokio. Also `WorldSeeder` lacked `#[derive(Default)]` required by `SeederRegistry::add::<T>()`
- **Fix:** Replaced all `rand::thread_rng().gen_range()` calls with a `rand_id()` helper that uses `rand::random::<u16>()` (no persistent rng state held across awaits); added `#[derive(Default)]` to `WorldSeeder`. In the seeder, rng is scoped to a block that ends before the `.exec(db).await` call
- **Files modified:** `bench.rs`, `world_seeder.rs`
- **Commit:** `12ad5a74`

**2. [Rule 1 - Bug] Host port conflicts (3000 used by Next.js, 8000 by Docker backend)**
- **Found during:** Task 9 (`docker compose up` — "address already in use" on port 3000)
- **Issue:** Host ports 3000 and 8000 were already bound (Next.js dev server on 3000, Docker backend service on 8000)
- **Fix:** Remapped host ports in compose.yaml: ferro → `3001:3000`, laravel → `8001:8000`. Conformance tests and README updated to use the remapped ports
- **Files modified:** `compose.yaml`
- **Commit:** `12ad5a74`

**3. [Rule 1 - Bug] laravel-micro: column name mismatch (randomNumber vs random_number)**
- **Found during:** Task 9 (laravel seeder failed with "column randomNumber of relation world does not exist")
- **Issue:** Laravel model/seeder used `randomNumber` (camelCase) but ferro's migration created the column as `random_number` (snake_case). Both apps share one DB, so ferro's schema wins
- **Fix:** Updated `World.php` fillable to `random_number`; `WorldSeeder.php` insert key to `random_number`; `routes/web.php` attribute access to `$w->random_number` while keeping JSON output key as `randomNumber` per contract
- **Files modified:** `World.php`, `WorldSeeder.php`, `routes/web.php`
- **Commit:** `12ad5a74`

**4. [Rule 3 - Blocking] laravel-micro world migration fails on shared DB (table already exists)**
- **Found during:** Task 9 (`php artisan migrate` — "relation world already exists")
- **Issue:** Ferro's migration already ran and created the `world` table before laravel's migration ran
- **Fix:** Added `if (!Schema::hasTable('world'))` guard in the laravel migration's `up()` method
- **Files modified:** `2026_06_15_021350_create_world_table.php`
- **Commit:** `12ad5a74`

**5. [Rule 3 - Blocking] laravel artisan serve workers use sqlite despite pgsql Docker env**
- **Found during:** Task 9 (conformance test — HTTP 500 on /db; logs showed "Connection: sqlite")
- **Issue:** `artisan serve` spawns PHP built-in server worker processes. When the `.env` file set `DB_CONNECTION=sqlite`, the dotenv `immutable` repository loaded `.env` before Docker env could be applied in worker processes — child workers inherited sqlite. Artisan CLI processes worked correctly (they ran after the env was fully available) but HTTP workers did not
- **Fix:** Two-part: (1) updated `.env` to `DB_CONNECTION=pgsql` and removed `DB_HOST` lines so dotenv has nothing to override; (2) added `php artisan config:cache` to the Dockerfile CMD before `artisan serve` — this bakes all Docker env vars (including `DB_HOST=db`) into `bootstrap/cache/config.php` at container startup, so forked worker processes read the cached config directly
- **Files modified:** `Dockerfile`, `.env` (gitignored — change documented here)
- **Commit:** `12ad5a74`

**6. [Rule 1 - Bug] build_tables.py: meta.json caused KeyError on "perf" key**
- **Found during:** Task 10 (`python3 build_tables.py results/2026-06-15/` — KeyError: 'perf')
- **Issue:** `load_results` globbed `*.json` and loaded `meta.json` (no `-` in name) as framework `"meta"` with key `"static"`. `render_markdown` iterated over all keys including `"meta"` which had no `"perf"` sub-key
- **Fix:** Added `startswith(("perf-", "static-"))` filter at the top of the loop; only files matching those prefixes are loaded as result data
- **Files modified:** `build_tables.py`
- **Commit:** `85c18fb7`

---

**Total deviations:** 6 auto-fixed (4 bugs, 2 blocking issues)
**Impact on plan:** All fixes were correctness requirements. No scope creep. The conformance gate was not weakened — all fixes made the apps correct.

## Known Stubs

None. All four endpoints return real data from Postgres.

## Threat Flags

None beyond what the plan's threat model covered. Results directory contains no secrets or PII. `.env` credentials are gitignored (laravel app default). The `compose.yaml` DB password (`bench`) is a throwaway benchmark credential, not a production secret.

## Self-Check: PASSED

All created files verified present on disk. Both task commits verified in git log.

| Check | Result |
|-------|--------|
| benchmark/contracts/conformance/test_conformance.py | FOUND |
| benchmark/compose.yaml | FOUND |
| benchmark/results/2026-06-15/meta.json | FOUND |
| benchmark/results/2026-06-15/perf-ferro.json | FOUND |
| benchmark/results/2026-06-15/perf-laravel.json | FOUND |
| benchmark/results/2026-06-15/static-ferro.json | FOUND |
| benchmark/results/2026-06-15/static-laravel.json | FOUND |
| benchmark/results/2026-06-15/internal.md | FOUND |
| benchmark/results/2026-06-15/public.md | FOUND |
| benchmark/README.md | FOUND |
| commit 12ad5a74 (Task 9) | FOUND |
| commit 85c18fb7 (Task 10) | FOUND |
