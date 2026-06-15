---
phase: 229-framework-benchmark-harness-foundation-1a-build-the-reproduc
verified: 2026-06-15T10:45:00Z
status: passed
score: 10/10
overrides_applied: 0
re_verification: null
gaps: []
deferred: []
human_verification: []
---

# Phase 229: Framework Benchmark Harness Foundation (1A) — Verification Report

**Phase Goal:** Build the reproducible `benchmark/` harness (contracts + static-counter + perf-runner + reporting toolbox) and prove it end-to-end on four micro-endpoints (`/json`, `/db`, `/queries`, `/updates`) implemented in Ferro and a minimal Laravel 11 app, producing the first committed perf + static results with recorded hardware.
**Verified:** 2026-06-15T10:45:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `benchmark/` tree exists with harness, contracts, apps, results subdirs | VERIFIED | `benchmark/{README.md,.gitignore,contracts/,harness/,apps/,results/}` all present; `.gitkeep` keeps `results/` tracked before first run |
| 2 | `parse_oha` returns `{rps, p50_ms, p90_ms, p99_ms, success_rate}` and raises `ValueError` on `success_rate < 0.99` | VERIFIED | `benchmark/harness/perf/parse_perf.py` lines 4–18; test file asserts both behaviors; 6 total pytest tests pass |
| 3 | `count_static.run(app_dir)` returns `{code_lines, files, source_tokens}` | VERIFIED | `benchmark/harness/static/count_static.py:21` — `run()` signature `(app_dir: str) -> dict`; returns all three keys |
| 4 | Harness unit tests pass (`pytest benchmark/harness -q`) | VERIFIED | `pytest benchmark/harness -q` → `6 passed in 0.01s` (confirmed live) |
| 5 | Ferro micro-app uses `DB::get()?` (no `Db` extractor param), Dockerfile uses `SERVER_HOST`/`SERVER_PORT` env vars, `rand` in Cargo.toml, `run_perf.py` uses `--output-format json` | VERIFIED | `bench.rs:24,43,60` — `DB::get()?` in each DB handler; `grep "db:\s*Db"` returns empty; `Dockerfile:13-14` — `ENV SERVER_HOST=0.0.0.0` / `ENV SERVER_PORT=3000`; `Cargo.toml:23` — `rand = "0.8"`; `run_perf.py:17` — `"--output-format", "json"` (not `--json`) |
| 6 | Both apps (Ferro and Laravel) pass 4/4 conformance assertions | VERIFIED | `test_conformance.py` — 4 tests covering `/json`, `/db`, `/queries` (clamp), `/updates` (length); `229-05-SUMMARY.md` documents both apps pass 4/4; commits `12ad5a74` + `85c18fb7` capture all fixes |
| 7 | `results/2026-06-15/` contains all required files with real hardware metadata | VERIFIED | `perf-ferro.json`, `perf-laravel.json`, `static-ferro.json`, `static-laravel.json`, `internal.md`, `public.md`, `meta.json`, `NOTES.md` all present; `meta.json` records Apple M1 Pro, 8 cores, 16 GB, Darwin 23.6.0, oha 1.9.0, tokei 12.1.2, pg 16.4 |
| 8 | `public.md` and `NOTES.md` prominently caveat that raw-perf numbers reflect `php artisan serve` (single-process dev server), not a fair Laravel comparison, with Phase 2 deferred | VERIFIED | `public.md` section "Raw performance — harness validation only, NOT a Laravel verdict" explains 5–8s p99 as queuing artifact; `NOTES.md` section "Raw performance — NOT a fair framework comparison" lists evidence and states "must not be cited as a Ferro-vs-Laravel performance result"; Phase 2 named explicitly in both |
| 9 | Workspace build not regressed — `benchmark/` excluded from root `Cargo.toml` workspace members | VERIFIED | `grep -c benchmark /Cargo.toml` → `0`; all 229 implementation commits touch only `benchmark/` files (confirmed via `git show --name-only` for commits `12ad5a74` and `85c18fb7`) |
| 10 | `README.md` contains end-to-end reproducible workflow (D-11) | VERIFIED | `benchmark/README.md` contains 8-step "Running the benchmark" section with exact Docker commands, migrate/seed, load-test, static-count, reporting, and hardware-recording steps |

**Score:** 10/10 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `benchmark/README.md` | Methodology skeleton, reproducibility + honesty note | VERIFIED | 50+ lines, "Running the benchmark" section, honesty note present |
| `benchmark/.gitignore` | Build-artifact ignore rules; `results/` committed | VERIFIED | Excludes `apps/*/target/`, `apps/*/vendor/`, `*.pyc`, `__pycache__/`; explicitly notes `results/` committed |
| `benchmark/contracts/micro-endpoints.md` | Shared 4-endpoint contract | VERIFIED | Documents all 4 endpoints with JSON shapes and `world` schema |
| `benchmark/results/.gitkeep` | Keeps results/ tracked | VERIFIED | 0-byte file present |
| `benchmark/harness/perf/parse_perf.py` | D-07 interface | VERIFIED | `def parse_oha` at line 4 |
| `benchmark/harness/static/count_static.py` | D-08 interface | VERIFIED | `def run(app_dir: str) -> dict` at line 21 |
| `benchmark/harness/report/build_tables.py` | D-09 interface | VERIFIED | `def render_markdown` + `load_results` with prefix filter for `perf-`/`static-` |
| `benchmark/harness/Dockerfile.toolbox` | Pinned toolbox image | VERIFIED | Pins oha 1.9.0 + tokei 12.1.2 + python3/jq/curl (note: 1.4.7 bumped to 1.9.0 — see Notes) |
| `benchmark/harness/perf/run_perf.py` | perf runner | VERIFIED | Uses `--output-format json` (two-token form); imports `parse_oha`; warmup + timed pattern |
| `benchmark/apps/ferro-micro/src/controllers/bench.rs` | Four handlers using `DB::get()?` | VERIFIED | `DB::get()?` at lines 24, 43, 60; `rand_id()` helper avoids `ThreadRng` across `.await` |
| `benchmark/apps/ferro-micro/Dockerfile` | Release build; env-var host/port | VERIFIED | `ENV SERVER_HOST=0.0.0.0` / `ENV SERVER_PORT=3000` |
| `benchmark/apps/ferro-micro/Cargo.toml` | `rand` dependency | VERIFIED | `rand = "0.8"` |
| `benchmark/apps/laravel-micro/routes/web.php` | Four routes with correct shapes | VERIFIED | All 4 endpoints; `randomNumber` JSON key from `random_number` column |
| `benchmark/apps/laravel-micro/Dockerfile` | `pdo_pgsql` + `config:cache` startup | VERIFIED | `pdo_pgsql` installed; `config:cache` in CMD before `artisan serve` |
| `benchmark/contracts/conformance/test_conformance.py` | Contract assertions | VERIFIED | 4 tests against `BASE_URL` env |
| `benchmark/compose.yaml` | postgres:16.4 + both apps, shared DB | VERIFIED | `postgres:16.4` with `pg_isready` healthcheck; `service_healthy` depends_on gate; ports remapped to 3001/8001 |
| `benchmark/results/2026-06-15/meta.json` | Hardware + tooling metadata | VERIFIED | Real hardware recorded |
| `benchmark/results/2026-06-15/perf-ferro.json` | Ferro perf results | VERIFIED | Keys: `/json`, `/db`, `/queries`, `/updates`; each with `rps`, `p50_ms`, `p90_ms`, `p99_ms`, `success_rate=1.0` |
| `benchmark/results/2026-06-15/perf-laravel.json` | Laravel perf results | VERIFIED | Same key structure |
| `benchmark/results/2026-06-15/static-ferro.json` | Ferro static counts | VERIFIED | `{"code_lines": 344, "files": 14, "source_tokens": 1158}` |
| `benchmark/results/2026-06-15/static-laravel.json` | Laravel static counts | VERIFIED | `{"code_lines": 1427, "files": 44, "source_tokens": 8874}` |
| `benchmark/results/2026-06-15/internal.md` | Full perf + static tables | VERIFIED | All 4 endpoints × 2 frameworks + ratio column; static table with all 3 metrics |
| `benchmark/results/2026-06-15/public.md` | Public subset + honesty caveat | VERIFIED | Headline subset; explicit dev-server caveat; Phase 2 deferred |
| `benchmark/results/2026-06-15/NOTES.md` | Honesty caveats for citing | VERIFIED | 3-section doc explaining fair vs unfair data, what Phase 2 must do before perf is publishable |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `benchmark/contracts/micro-endpoints.md` | `benchmark/contracts/conformance/test_conformance.py` | contract is source for conformance assertions | WIRED | `GET /json\|/db\|/queries\|/updates` patterns appear in both files |
| `benchmark/harness/perf/parse_perf.py` | `benchmark/harness/perf/run_perf.py` | `from parse_perf import parse_oha` | WIRED | `run_perf.py:4` imports `parse_oha`; called at line 20 |
| `benchmark/harness/report/build_tables.py` | `results/<date>/*.json` | `load_results` parses `perf-<fw>.json`/`static-<fw>.json` filenames | WIRED | `build_tables.py` filters by `startswith(("perf-", "static-"))` prefix; `render_markdown` consumes `data[fw]["perf"]` and `data[fw]["static"]` |
| `benchmark/apps/ferro-micro/src/controllers/bench.rs` | world table | `DB::get()? + world::Entity::find_by_id` | WIRED | `bench.rs:24,43,60` — `DB::get()?`; `find_by_id(id)` in db_handler, queries, updates (confirmed by manual grep; gsd-tools regex escaping fails on `()`) |
| `benchmark/apps/ferro-micro/Dockerfile` | serve env config | `SERVER_HOST`/`SERVER_PORT` env vars | WIRED | `Dockerfile:13-14` |
| `benchmark/compose.yaml` | `benchmark/apps/ferro-micro`, `benchmark/apps/laravel-micro` | build context + shared DB env | WIRED | `build: ./apps/ferro-micro`; `build: ./apps/laravel-micro`; shared `DATABASE_URL`/`DB_*` env |
| `benchmark/results/2026-06-15/internal.md` | `perf-*.json + static-*.json` | `build_tables.load_results` consumed those filenames at generation time | WIRED | `internal.md` was generated by `build_tables.py` from those files; content matches raw JSON values exactly |

### Data-Flow Trace (Level 4)

Not applicable: the artifacts are Python scripts and benchmark result files, not UI components rendering dynamic data. The data pipeline is: `oha` → `parse_oha` → `perf-<fw>.json` → `build_tables.render_markdown` → `internal.md`. Each step produces non-empty, real data as confirmed by the committed result files.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `pytest benchmark/harness -q` → 6 passing tests | `pytest benchmark/harness -q` | `6 passed in 0.01s` | PASS |
| `parse_oha` returns exact D-07 keys | Python import + dict key check | `['p50_ms', 'p90_ms', 'p99_ms', 'rps', 'success_rate']` | PASS |
| `count_static.run()` is callable with correct signature | Python import + `inspect.signature` | `(app_dir: str) -> dict` | PASS |
| `build_tables.render_markdown` produces ratio column | See test_build_tables.py assertions | `test_render_marks_winner_per_metric` passes with `22.2x`/`22.22x` | PASS |
| `benchmark/` excluded from root workspace | `grep -c benchmark /Cargo.toml` | `0` | PASS |
| `run_perf.py` uses `--output-format json` (not `--json`) | `grep "--output-format" run_perf.py` | Line 17: `"--output-format", "json"` | PASS |
| `DB::get()` pattern in ferro-micro handlers (no `Db` extractor) | `grep "DB::get()" bench.rs` | Found at lines 24, 43, 60 | PASS |
| `grep "db:\s*Db" src/` absent | `grep -rn "db:\s*Db" benchmark/apps/ferro-micro/src/` | No output | PASS |
| Honesty caveat prominent in public.md | Content search for dev-server/artisan serve language | "single-process development server", "5–8 seconds", "Phase 2" all present | PASS |

### Requirements Coverage

No requirements mapped for this phase (`requirements: []` — validation/tooling phase; gated by conformance + pytest units per ROADMAP.md).

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `Dockerfile.toolbox` | 6 | `oha --version 1.9.0` (plan said 1.4.7) | Info | Version bumped from 1.4.7 to 1.9.0 because 1.4.7 lacked `--output-format json`. Intentional, documented in NOTES.md and `229-03-SUMMARY.md`. Correct behavior. |
| `public.md` | — | Missing literal word "expected" (plan 05 `contains: "expected"`) | Info | Plan 05 artifact check wanted the word "expected" but public.md uses stronger wording: "foregone conclusion", "dev server artifact", "must not be cited". D-10 honesty fully satisfied. Not a stub. |

No blocking anti-patterns found. No `TODO`/`FIXME`/placeholder comments in any harness or app source files.

### Human Verification Required

None. All phase goal elements are verifiable from committed artifacts, code, and test output.

---

## Notes (Follow-ups, Not Failures)

- **oha 1.4.7 → 1.9.0 pin bump:** `--output-format json` was introduced in oha 1.9.0; 1.4.7 exposes only `-j`/`--json`. Discovered at build time (documented Assumption A3 in 229-RESEARCH.md). The bump is correct and documented in NOTES.md, SUMMARY, and the Dockerfile comment. The plan 03 `must_have.artifacts.contains: "cargo install oha --version 1.4.7"` is stale text but does not affect the phase goal.
- **Phase 2 (fair Laravel perf comparison):** Laravel was benchmarked under `php artisan serve` (single-process dev server). The NOTES.md explicitly states the Phase 2 requirement: re-run under php-fpm/nginx or Laravel Octane. This is deferred by design (D-05, CONTEXT.md). Not a gap.
- **gsd-tools key-link false negatives:** The tool's regex engine struggles with `()` metacharacters in `DB::get()` and `|` alternation in `perf-|static-`. Manual grep confirms both links are WIRED. These are tool limitations, not implementation gaps.
- **compose.yaml ports remapped:** ferro→3001 (host), laravel→8001 (host); dev ports 3000/8000 were occupied. Conformance test and README updated accordingly. Intentional fix documented in SUMMARY.

---

_Verified: 2026-06-15T10:45:00Z_
_Verifier: Claude (gsd-verifier)_
