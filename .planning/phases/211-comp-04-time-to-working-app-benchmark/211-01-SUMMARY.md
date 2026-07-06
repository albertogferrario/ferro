---
phase: 211
plan: 01
subsystem: ferro-cli/benchmarks
tags: [benchmark, criterion, comp-04, ci-gate, cold-cache]
dependency_graph:
  requires: []
  provides: [COMP-04-apparatus, benchmark-new-project-rs, cold-cache-dockerfile, results-template]
  affects: [ferro-cli]
tech_stack:
  added: [criterion 0.8.2 (dev-dep, cargo_bench_support, default-features=false)]
  patterns: [iter_custom-programmatic, FERRO_BENCH-gate, CARGO_BIN_EXE_ferro, per-step-Instant]
key_files:
  created:
    - ferro-cli/tests/benchmark_new_project.rs
    - ferro-cli/tests/fixtures/benchmark/Dockerfile
    - ferro-cli/tests/fixtures/benchmark/RESULTS.md
  modified:
    - ferro-cli/Cargo.toml
    - Cargo.lock
decisions:
  - "Use criterion iter_custom driven programmatically from tests/ (no [[bench]] target, no criterion_main!)"
  - "final_summary() omitted — not public API in criterion 0.8.2; c drops to flush"
  - "sample_size(10) with measurement_time(600s) to accommodate long warm runs"
  - "Per-step Instants captured inside iter_custom closure; printed under --nocapture"
metrics:
  duration: "~5 minutes"
  completed: "2026-06-13T02:01:00Z"
  tasks: 3
  files: 4
---

# Phase 211 Plan 01: COMP-04 Benchmark Apparatus Summary

**One-liner:** Gated criterion iter_custom benchmark (5 CLI steps, per-step Instants, cargo build exit-0 assertion) plus cold-cache Dockerfile and RESULTS.md template with seeded naming-mismatch weakness.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Add criterion dev-dep to ferro-cli | 78b4c32f | ferro-cli/Cargo.toml |
| 2 | Write gated criterion iter_custom benchmark | 296ad095 | ferro-cli/tests/benchmark_new_project.rs |
| 3 | Commit cold-cache Dockerfile + RESULTS.md | 7ced0c7d | ferro-cli/tests/fixtures/benchmark/{Dockerfile,RESULTS.md} |

## What Was Built

### criterion dev-dependency (Task 1)

Added to `ferro-cli/Cargo.toml` `[dev-dependencies]`:
```toml
criterion = { version = "0.8.2", default-features = false, features = ["cargo_bench_support"] }
```
No `[[bench]]` section. The benchmark lives in `tests/`, not `benches/`. `default-features = false` with only `cargo_bench_support` satisfies the no-external-build-tooling constraint.

### Gated benchmark (Task 2)

`ferro-cli/tests/benchmark_new_project.rs` — a single synchronous `#[test]` with belt-and-suspenders gate:
- `#[ignore]` prevents running under plain `cargo test`
- env-var early return (`FERRO_BENCH`) prevents running even under `-- --ignored` without the flag
- `Criterion::default().sample_size(10).measurement_time(600s)` driven programmatically
- Five steps timed with individual `Instant::now()` captures inside `iter_custom`
- Step 1 (`ferro new`) uses `tmp.path()` as CWD; steps 2–5 use `tmp.path().join("bench-app")`
- `make:scaffold` carries `--no-smart-defaults -q -y --api` (suppresses non-TTY stdin hang)
- Step 5 (`cargo build`) asserts exit code 0 — SC#2 wiring present

Gate-OFF verification: `cargo test -p ferro-cli --test benchmark_new_project` exits 0, test listed as `ignored`.

### Cold-cache Docker fixture (Task 3)

`ferro-cli/tests/fixtures/benchmark/Dockerfile`:
- Base: `debian:bookworm-slim` (no pre-installed toolchain, no Cargo registry — truly cold)
- rustup installed with `--proto '=https' --tlsv1.2` TLS pinning (T-211-01 mitigation)
- `cargo install ferro-cli` in a Docker layer (not timed; benchmarks scaffolding only)
- CMD runs the identical 5-step sequence, printing `$((SECONDS - T0))s` per step

`ferro-cli/tests/fixtures/benchmark/RESULTS.md`:
- Full SC#4 env-spec table: 9 rows (Rust toolchain, ferro-rs version, Cache state, Host machine class, CPU cores, Memory, Disk free at run time, Agent-assistance level, Date)
- Per-step wall-clock breakdown table with exact commands (make:scaffold, not make:model)
- Discovered Weaknesses seeded with verified naming-mismatch finding (non-empty, non-placeholder)
- Notes: cold run via Docker, warm run command, CI threshold not asserted (D-07)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] clippy uninlined_format_args in benchmark file**
- **Found during:** Task 2 clippy gate
- **Issue:** 7 `println!`/`assert!` calls used `{:?}` with separate variable arguments instead of inline `{var:?}` syntax; `-D warnings` made these errors
- **Fix:** Rewrote all format strings to use inline variable syntax (`{step1:?}`, `{code:?}`, etc.); extracted `let code = status.code()` before each assert to allow inlining
- **Files modified:** ferro-cli/tests/benchmark_new_project.rs
- **Commit:** 296ad095 (included in same task commit after fix)

**2. [Rule 4 - Not triggered] final_summary() availability**
- Research flagged this as verify-at-execute. At compile time, `final_summary()` is not public API in criterion 0.8.2. The PLAN acceptance note covers this: omit the call and let `c` drop to flush. Applied without deviation.

## Known Stubs

The RESULTS.md numeric cells are intentional placeholders (`TBD (filled by 211-02 cold run)`). This is by design — the plan's objective is the apparatus (SC#1/SC#2 wiring, Dockerfile, env-spec template). The human-action plan 211-02 fills the real numbers and commits them.

## Threat Flags

No new threat surface introduced beyond what the plan's `<threat_model>` documents. The Dockerfile's `curl --proto '=https' --tlsv1.2` TLS mitigation (T-211-01) is present. The `debian:bookworm-slim` base image comment notes digest-pinning as a further hardening step (T-211-02).

## Self-Check

**Files exist:**
- [x] ferro-cli/tests/benchmark_new_project.rs — FOUND
- [x] ferro-cli/tests/fixtures/benchmark/Dockerfile — FOUND
- [x] ferro-cli/tests/fixtures/benchmark/RESULTS.md — FOUND

**Commits exist:**
- [x] 78b4c32f — chore(211-01): add criterion 0.8.2 dev-dep
- [x] 296ad095 — feat(211-01): gated criterion iter_custom benchmark
- [x] 7ced0c7d — feat(211-01): cold-cache Dockerfile + RESULTS.md

**Gate-off test:** `cargo test -p ferro-cli --test benchmark_new_project` exits 0, 1 ignored.
**Clippy:** `cargo clippy -p ferro-cli --all-targets -- -D warnings` — clean.

## Self-Check: PASSED
