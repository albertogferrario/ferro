---
phase: 172-mcp-tool-wrappers
plan: "01"
subsystem: ferro-mcp
tags: [relevance, utility, relocation, test-infrastructure]
dependency_graph:
  requires: []
  provides: [ferro-mcp/src/tools/relevance.rs, ENV_LOCK]
  affects: [ferro-mcp/src/tools/mod.rs, ferro-mcp/src/lib.rs]
tech_stack:
  added: []
  patterns: [verbatim-relocation, pub-visibility-raise, test-mutex-pattern]
key_files:
  created:
    - ferro-mcp/src/tools/relevance.rs
  modified:
    - ferro-mcp/src/tools/mod.rs
    - ferro-mcp/src/lib.rs
decisions:
  - "#[allow(dead_code)] added to ENV_LOCK to suppress unused-static warning until Plans 02+ consume it"
metrics:
  duration: 187s
  completed: "2026-06-08"
  tasks_completed: 2
  files_changed: 3
---

# Phase 172 Plan 01: Relevance Filter Relocation Summary

**One-liner:** Verbatim relocation of the lexical relevance filter (`tokenize`, `Candidate`, `select_relevant`, `INPUT_BUDGET_CHARS`) from `ferro-cli` into `ferro-mcp/src/tools/relevance.rs` with `pub(crate)` → `pub` visibility, plus a `#[cfg(test)]` `ENV_LOCK` mutex in `ferro-mcp/src/lib.rs`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Relocate relevance.rs verbatim into ferro-mcp with pub visibility | 64176673 | ferro-mcp/src/tools/relevance.rs (created), ferro-mcp/src/tools/mod.rs (modified) |
| 2 | Add test-only ENV_LOCK to ferro-mcp | aeb8929c | ferro-mcp/src/lib.rs (modified) |

## Verification

- `cargo test -p ferro-mcp --all-features relevance`: 3/3 tests pass
- `cargo build -p ferro-mcp`: clean (no warnings)
- `cargo build -p ferro-cli`: clean (both crates coexist; ferro-cli copy retained for Plan 04 deletion wave)
- `cargo test -p ferro-mcp --all-features --no-run`: clean compile including test build
- No `pub(crate)` remains on exported relevance items
- No `cfg(feature)` guards in the relocated file

## Acceptance Criteria

- [x] `grep -q "pub fn select_relevant" ferro-mcp/src/tools/relevance.rs` exits 0
- [x] `grep -q "pub fn tokenize" ferro-mcp/src/tools/relevance.rs` exits 0
- [x] `grep -q "pub const INPUT_BUDGET_CHARS" ferro-mcp/src/tools/relevance.rs` exits 0
- [x] `grep -q "pub mod relevance" ferro-mcp/src/tools/mod.rs` exits 0
- [x] `grep -c "pub(crate)" ferro-mcp/src/tools/relevance.rs` returns 0
- [x] No `cfg(feature)` guard in the relocated file
- [x] `cargo test -p ferro-mcp --all-features relevance` exits 0
- [x] `grep -q "static ENV_LOCK" ferro-mcp/src/lib.rs` exits 0
- [x] `grep -q "cfg(test)" ferro-mcp/src/lib.rs` exits 0
- [x] `cargo test -p ferro-mcp --all-features --no-run` exits 0
- [x] `cargo build -p ferro-mcp` exits 0

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] Suppress dead_code warning on ENV_LOCK**
- **Found during:** Task 2 verification
- **Issue:** `#[cfg(test)] pub(crate) static ENV_LOCK` generated a `dead_code` warning because no test in Plans 01 uses it yet. With `-D warnings` in CI clippy, this would cause a build failure.
- **Fix:** Added `#[allow(dead_code)]` above the static. The allow annotation will be removed by Plans 02+ when the first `let _guard = ENV_LOCK.lock()` appears in a test.
- **Files modified:** ferro-mcp/src/lib.rs
- **Commit:** aeb8929c (incorporated into Task 2 commit)

## Known Stubs

None. The relocated `relevance.rs` is fully functional — all function bodies are verbatim from `ferro-cli/src/relevance.rs` with no placeholder logic.

## Threat Flags

None. This plan relocates pure-lexical deterministic utility code and a `#[cfg(test)]`-gated mutex. No new network endpoints, auth paths, file access patterns, or schema changes introduced.

## Self-Check: PASSED

- `ferro-mcp/src/tools/relevance.rs` exists and contains `pub fn select_relevant` ✓
- `ferro-mcp/src/tools/mod.rs` contains `pub mod relevance` ✓
- `ferro-mcp/src/lib.rs` contains `static ENV_LOCK` under `#[cfg(test)]` ✓
- Commit 64176673 exists (Task 1) ✓
- Commit aeb8929c exists (Task 2) ✓
