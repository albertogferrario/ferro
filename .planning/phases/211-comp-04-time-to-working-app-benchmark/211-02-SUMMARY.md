---
phase: 211
plan: 02
subsystem: ferro-cli/benchmarks
tags: [benchmark, comp-04, cold-cache, docker, discovered-weaknesses]
dependency_graph:
  requires: [COMP-04-apparatus]
  provides: [COMP-04-cold-result, 211-weaknesses]
  affects: [ferro-cli]
tech_stack:
  added: []
  patterns: [cold-cache-docker-run, manual-checkpoint-resolution]
key_files:
  created:
    - .planning/phases/211-comp-04-time-to-working-app-benchmark/211-WEAKNESSES.md
  modified:
    - ferro-cli/tests/fixtures/benchmark/RESULTS.md
    - ferro-cli/tests/fixtures/benchmark/Dockerfile
    - ferro-cli/tests/benchmark_new_project.rs
decisions:
  - "Human-action Docker checkpoint resolved by running the cold-cache benchmark in-session (Docker daemon available); numbers are real, not fabricated"
  - "Cold run surfaced two apparatus defects (scaffold flag order, missing openssl deps) — fixed rather than worked around, per audit-and-fix discipline"
  - "Headline result recorded honestly: published 0.2.55 scaffold does not compile (52 errors); time-to-working-app not achieved on the published artifact"
metrics:
  duration: "~25 minutes (incl. cold Docker build + run)"
  completed: "2026-06-13"
  tasks: 3
  files: 4
---

# Phase 211 Plan 02: COMP-04 Cold-Cache Result Summary

**One-liner:** First real cold-cache benchmark run (Apple M1 Pro, rustc 1.96.0, ferro 0.2.55) revealed that the published scaffold does not compile — `cargo build` fails with 52 errors; recorded in RESULTS.md with full env spec and three additional concrete weaknesses in 211-WEAKNESSES.md.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Developer runs cold-cache Docker benchmark (checkpoint:human-action) | 025d9778 (apparatus fixes) | Dockerfile, benchmark_new_project.rs |
| 2 | Fill RESULTS.md with real cold-cache row + env spec | 071ca389 | ferro-cli/tests/fixtures/benchmark/RESULTS.md |
| 3 | Finalize SC#5 Discovered Weaknesses finding | 071ca389 | 211-WEAKNESSES.md |

## What Was Built / Measured

### Cold run (Task 1 — human-action checkpoint, resolved in-session)

The Docker daemon was available, so the cold-cache benchmark was run in-session rather than
deferred. `docker build` + `docker run` of the committed Dockerfile on a clean
`debian:bookworm-slim`. Two apparatus defects surfaced and were fixed (see Deviations):

- `cargo install ferro-cli` failed: `openssl-sys` (via `native-tls`) needs system OpenSSL →
  added `libssl-dev pkg-config` to the Dockerfile apt layer.
- `make:scaffold` rejected `--no-smart-defaults` as a field name (greedy `[FIELDS]...`
  positional) → reordered flags before fields in the Dockerfile CMD and the Rust benchmark.

After the fixes, the five-step sequence ran: steps 1–4 each <1s; **Step 5 (`cargo build`)
failed with 52 compile errors** — the generated project does not build against published
`ferro-rs` 0.2.55.

### RESULTS.md (Task 2)

Real env spec: rustc 1.96.0 (stable), ferro-cli/ferro-rs 0.2.55, cold cache, Apple M1 Pro,
8 cores, 16 GB, ~15 GB free, manual commands, 2026-06-13. Per-step table records steps 1–4
as <1s and Step 5 as FAILED with the working-app total marked not-achieved. No TBD cells remain.

### 211-WEAKNESSES.md (Task 3) — SC#5 gate

Four concrete findings, dominant first:
1. **Generated app does not compile against published 0.2.55** (52 errors: `error_response!`
   macro, `#[rule]` attribute, `ferro::Queue`/`QueueConfig`, undeclared `ferro-queue` dep,
   unimported `ActiveValue`, `crate::models::users`, `ferro::database::connection`-as-fn).
2. Cold CLI install needs `libssl-dev` + `pkg-config` (openssl-sys).
3. `make:scaffold` flag ordering swallows flags as field names.
4. `make:model` vs `make:scaffold` spec/impl naming mismatch.

## Deviations from Plan

**1. [Rule — apparatus defect] Scaffold flag ordering (Dockerfile + benchmark_new_project.rs)**
- Found during: Task 1 cold run (`Invalid field name: '--no-smart-defaults'`).
- Fix: reorder to `make:scaffold [OPTIONS] <NAME> [FIELDS]...` in both files. 211-01 never
  executed the scaffold (gate-off skip only), so the bug was latent.
- Commit: 025d9778.

**2. [Rule — apparatus defect] Dockerfile missing OpenSSL build deps**
- Found during: Task 1 image build (`cargo install ferro-cli` exit 101, openssl-sys).
- Fix: add `libssl-dev pkg-config` to the apt layer.
- Commit: 025d9778.

**3. [Scope — checkpoint resolved in-session]** The plan marked Task 1 as a
`checkpoint:human-action` because the autonomous environment has no Docker daemon. Docker was
available in this session and the operator approved running it, so the cold run was performed
directly. Numbers are real (not stubbed).

## Known Stubs

None. All RESULTS.md cells carry real values; the build-failure result is recorded honestly
rather than substituted with a synthetic green number.

## Follow-up (out of scope for this benchmark phase)

Finding 1 is a real published-artifact defect (scaffold↔library API drift) warranting a
follow-up phase: align the `ferro-cli` scaffold templates with the published `ferro` surface
and add a published-artifact smoke test (scaffold → `cargo build`) to CI. Not fixed here — this
phase measures; it does not re-author the scaffolder.

## Self-Check

**Files exist:**
- [x] ferro-cli/tests/fixtures/benchmark/RESULTS.md — FOUND (no TBD, has cold row + env spec)
- [x] .planning/phases/211-comp-04-time-to-working-app-benchmark/211-WEAKNESSES.md — FOUND (66 lines, no placeholder)

**Commits exist:**
- [x] 025d9778 — fix(211): cold-run apparatus fixes
- [x] 071ca389 — docs(211-02): cold-cache result + weaknesses

**Gates:** `cargo fmt --all -- --check` clean; `cargo clippy -p ferro-cli --all-targets -- -D warnings` clean; `cargo test -p ferro-cli --test benchmark_new_project` exits 0 (1 ignored, gate-safe).

## Self-Check: PASSED
