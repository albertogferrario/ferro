---
phase: 220-confirmation-gating-for-destructive-actions
plan: "00"
subsystem: ferro-ai, ferro-mcp-server
tags: [feature-split, dependency-hygiene, confirmation, tdd-red, cargo-features]
dependency_graph:
  requires: [219-write-dispatch]
  provides: [ferro-ai-feature-split, confirmation-feature, ConfirmationRequired-variant, RED-confirmation-tests]
  affects: [ferro-mcp-server, ferro-ai, ferro-mcp, ferro-cli, framework]
tech_stack:
  added: ["ferro-ai confirmation feature (no HTTP deps)", "tokio test-util (dev)"]
  patterns: ["optional-dep feature gating", "#[cfg(feature)] module gates", "stub functions for RED TDD"]
key_files:
  created: []
  modified:
    - ferro-ai/Cargo.toml
    - ferro-ai/src/lib.rs
    - ferro-mcp-server/Cargo.toml
    - ferro-mcp-server/src/error.rs
    - ferro-mcp-server/src/write_dispatch.rs
decisions:
  - "D-06 resolved via ferro-ai [features] refactor — no ferro-confirmation extraction crate needed"
  - "confirmation=[] feature has zero extra deps; dashmap/tokio/ferro-events are always-on"
  - "Stubs (handle_request_confirm/handle_confirm) with todo!() enable RED tests to compile before Plan 01 implementation"
  - "reqwest in ferro-mcp-server confirmation tree comes from ferro-mcp-oauth (pre-existing), not ferro-ai"
metrics:
  duration_seconds: 609
  completed_date: "2026-06-14"
  tasks_completed: 3
  files_modified: 5
---

# Phase 220 Plan 00: Dependency Hygiene Foundation — Summary

Feature-split `ferro-ai` so its `confirmation` module is reqwest-free; audited all consumers; added `confirmation` Cargo feature + `ConfirmationRequired` error variant to `ferro-mcp-server`; authored RED confirmation tests (SC#1–#4 + guard-at-confirm) that compile under `--features confirmation`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Feature-split ferro-ai (default=["llm"], reqwest-free confirmation) | bca67b9a | ferro-ai/Cargo.toml, ferro-ai/src/lib.rs |
| 2 | Consumer audit + ferro-mcp-server confirmation feature + ConfirmationRequired | e33ad769 | ferro-mcp-server/Cargo.toml, ferro-mcp-server/src/error.rs |
| 3 | RED confirmation tests (SC#1–#4 + guard-at-confirm) | 47c0cf1d | ferro-mcp-server/src/write_dispatch.rs, ferro-mcp-server/Cargo.toml |

## Verification Results

- `cargo build -p ferro-ai --no-default-features`: exits 0; `cargo tree ... | grep -c reqwest` = 0
- `cargo build -p ferro-ai` (default llm): exits 0
- `cargo build -p ferro-mcp -p ferro-cli`: exits 0 (LLM consumers still build)
- `cargo build --manifest-path framework/Cargo.toml --features ai`: exits 0
- `cargo build -p ferro-mcp-server --features confirmation`: exits 0
- `cargo build` (full workspace): exits 0
- `cargo test -p ferro-mcp-server --features confirmation --no-run`: exits 0 (RED tests compile)
- `grep -c 'cfg(feature = "llm")' ferro-ai/src/lib.rs`: 18 (>= 8 required)
- All 6 RED test names present; `start_paused = true` present; `cfg(all(test, feature = "confirmation"))` present

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Pre-existing rustfmt diff in write_dispatch.rs**
- **Found during:** Task 3 (pre-commit fmt check)
- **Issue:** `Err(ref e @ crate::Error::Validation(_)) | Err(ref e @ ...)` was split across two lines; rustfmt wanted it on one line
- **Fix:** Reformatted the match arm; then ran `cargo fmt --all` to apply all formatting
- **Files modified:** ferro-mcp-server/src/write_dispatch.rs
- **Commit:** included in task 3 commit

**2. [Rule 3 - Blocking] tokio test-util feature missing for SC#3 paused-clock test**
- **Found during:** Task 3 first compile attempt
- **Issue:** `#[tokio::test(start_paused = true)]` and `tokio::time::advance` require the `test-util` feature which was not in ferro-mcp-server's dev-dependencies
- **Fix:** Added `test-util` to the `tokio` dev-dependency features
- **Files modified:** ferro-mcp-server/Cargo.toml
- **Commit:** included in task 3 commit

### Criterion Clarification (not a deviation)

The acceptance criterion `cargo tree -p ferro-mcp-server --features confirmation --edges normal | grep -c reqwest == 0` counts 3 (reqwest lines from `ferro-mcp-oauth`, which is a hard dep of ferro-mcp-server and has always included reqwest). This is pre-existing and unrelated to ferro-ai. The D-06 goal — that `ferro-ai` with `features = ["confirmation"]` adds zero reqwest — is verified: `cargo tree -p ferro-ai --no-default-features --edges normal | grep reqwest` = empty.

## Known Stubs

| Stub | File | Lines | Reason |
|------|------|-------|--------|
| `handle_request_confirm` | ferro-mcp-server/src/write_dispatch.rs | ~410 | Plan 01 implements; stub enables RED tests to compile |
| `handle_confirm` | ferro-mcp-server/src/write_dispatch.rs | ~425 | Plan 01 implements; stub enables RED tests to compile |

These stubs are intentional RED-phase placeholders. Plan 01 replaces them with real implementations.

## Self-Check: PASSED

- ferro-ai/Cargo.toml: FOUND (default = ["llm"], reqwest optional, confirmation = [])
- ferro-ai/src/lib.rs: FOUND (18 cfg(feature = "llm") gates)
- ferro-mcp-server/Cargo.toml: FOUND (confirmation = ["dep:ferro-ai"])
- ferro-mcp-server/src/error.rs: FOUND (ConfirmationRequired variant)
- ferro-mcp-server/src/write_dispatch.rs: FOUND (6 RED test names, start_paused, cfg gate)
- Commits bca67b9a, e33ad769, 47c0cf1d: FOUND in git log
