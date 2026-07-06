---
phase: 172-mcp-tool-wrappers
plan: "02"
subsystem: ferro-mcp
tags: [ai-scaffold, ai-explain, async-core, relocation, prompt-injection-guard]
dependency_graph:
  requires: [ferro-mcp/src/tools/relevance.rs, ENV_LOCK]
  provides: [ferro-mcp/src/tools/ai_scaffold.rs, ferro-mcp/src/tools/ai_explain_core.rs]
  affects: [ferro-mcp/src/tools/mod.rs, ferro-mcp/src/lib.rs]
tech_stack:
  added: []
  patterns: [async-core-relocation, two-branch-explain, prompt-injection-sanitization, zero-token-structured-branch]
key_files:
  created:
    - ferro-mcp/src/tools/ai_scaffold.rs
    - ferro-mcp/src/tools/ai_explain_core.rs
  modified:
    - ferro-mcp/src/tools/mod.rs
    - ferro-mcp/src/lib.rs
decisions:
  - "Tasks 1 and 2 committed together — ai_explain_core.rs module declaration in mod.rs was required for Task 1 to compile; intermediate commit would have been a broken build"
  - "ENV_LOCK #[allow(dead_code)] removed since ai_scaffold and ai_explain_core tests now acquire it"
  - "FieldInfo moved to test-module-scoped import in ai_explain_core.rs — not needed in production code path, only in test fixtures"
metrics:
  duration: 450s
  completed: "2026-06-08"
  tasks_completed: 2
  files_changed: 4
---

# Phase 172 Plan 02: Async Cores — scaffold_core and explain_core Summary

**One-liner:** Relocated ServiceDef generation pipeline (`scaffold_core`) and two-branch projection-framed explanation (`explain_core`, zero-LLM-token structured path + prose fallback) from ferro-cli into ferro-mcp async cores with prompt-injection guard.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create ai_scaffold.rs core (scaffold_core + relocated helpers + tests) | 4d522341 | ferro-mcp/src/tools/ai_scaffold.rs (created), ferro-mcp/src/tools/mod.rs (modified) |
| 2 | Create ai_explain_core.rs (explain_core async, structured+prose branches, tests) | 4d522341 | ferro-mcp/src/tools/ai_explain_core.rs (created), ferro-mcp/src/tools/mod.rs (modified), ferro-mcp/src/lib.rs (modified) |

Note: Both tasks share a single commit (4d522341) because the `mod.rs` registration of `ai_explain_core` was required for the crate to compile during Task 1 verification. An intermediate commit with only `ai_scaffold.rs` registered would have been a broken build.

## Verification

- `cargo test -p ferro-mcp --all-features ai_scaffold`: 8/8 tests pass
- `cargo test -p ferro-mcp --all-features ai_explain`: 12/12 tests pass
- `cargo clippy -p ferro-mcp --all-features -- -D warnings`: clean
- `cargo fmt -p ferro-mcp -- --check`: clean
- `cargo build -p ferro-mcp`: clean

## Acceptance Criteria

### Task 1 (ai_scaffold.rs)

- [x] `grep -q "pub async fn scaffold_core" ferro-mcp/src/tools/ai_scaffold.rs` exits 0
- [x] `grep -q "complete_with::<ServiceDef>" ferro-mcp/src/tools/ai_scaffold.rs` exits 0
- [x] `grep -q "relevance::select_relevant" ferro-mcp/src/tools/ai_scaffold.rs` exits 0
- [x] `grep -q "service.validate()" ferro-mcp/src/tools/ai_scaffold.rs` exits 0
- [x] `! grep -q "process::exit" ferro-mcp/src/tools/ai_scaffold.rs`
- [x] `! grep -q "eprintln!" ferro-mcp/src/tools/ai_scaffold.rs`
- [x] `! grep -q "console::style" ferro-mcp/src/tools/ai_scaffold.rs`
- [x] `! grep -q "block_on" ferro-mcp/src/tools/ai_scaffold.rs`
- [x] `! grep -q "cfg(feature" ferro-mcp/src/tools/ai_scaffold.rs`
- [x] `grep -q "pub mod ai_scaffold" ferro-mcp/src/tools/mod.rs` exits 0
- [x] `cargo test -p ferro-mcp --all-features ai_scaffold` exits 0

### Task 2 (ai_explain_core.rs)

- [x] `grep -q "pub async fn explain_core" ferro-mcp/src/tools/ai_explain_core.rs` exits 0
- [x] `grep -q "pub async fn resolve_target" ferro-mcp/src/tools/ai_explain_core.rs` exits 0
- [x] `! grep -q "rt.block_on" ferro-mcp/src/tools/ai_explain_core.rs`
- [x] `! grep -q "tokio::runtime::Runtime" ferro-mcp/src/tools/ai_explain_core.rs`
- [x] `grep -q "to_value(&detail)" ferro-mcp/src/tools/ai_explain_core.rs` exits 0
- [x] `grep -q '"prose"' ferro-mcp/src/tools/ai_explain_core.rs` exits 0
- [x] `grep -q "schema: None" ferro-mcp/src/tools/ai_explain_core.rs` exits 0
- [x] `! grep -q "process::exit" ferro-mcp/src/tools/ai_explain_core.rs`
- [x] `! grep -q "console::style" ferro-mcp/src/tools/ai_explain_core.rs`
- [x] `! grep -q "cfg(feature" ferro-mcp/src/tools/ai_explain_core.rs`
- [x] `grep -q "pub mod ai_explain_core" ferro-mcp/src/tools/mod.rs` exits 0
- [x] `cargo test -p ferro-mcp --all-features ai_explain` exits 0

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Tasks 1 and 2 committed together as a single unit**
- **Found during:** Task 1 test run
- **Issue:** Adding `pub mod ai_explain_core;` to `mod.rs` (required for alphabetical registration) caused a compile error when `ai_explain_core.rs` did not yet exist. Task 1 could not be compiled or tested in isolation.
- **Fix:** Created both files before running any tests, then committed together. Both tasks verify independently via their separate test filters (`ai_scaffold` and `ai_explain`).
- **Files modified:** ferro-mcp/src/tools/ai_explain_core.rs, ferro-mcp/src/tools/mod.rs
- **Commit:** 4d522341

**2. [Rule 1 - Bug] Removed unused `FieldInfo` import from module scope in ai_explain_core.rs**
- **Found during:** Task 2 clippy run
- **Issue:** `FieldInfo` was imported at module level but only used in the `#[cfg(test)]` block; clippy `-D warnings` flagged it as `unused_imports`.
- **Fix:** Removed `FieldInfo` from the module-level `use` statement and added `use crate::tools::inspect_projection::FieldInfo;` inside the `#[cfg(test)] mod tests` block.
- **Files modified:** ferro-mcp/src/tools/ai_explain_core.rs
- **Commit:** 4d522341 (incorporated before commit)

**3. [Rule 2 - Missing functionality] Removed `#[allow(dead_code)]` from ENV_LOCK**
- **Found during:** Task 1 implementation
- **Issue:** Plan 01 added `#[allow(dead_code)]` to ENV_LOCK anticipating Plan 02 would use it. Now that both `ai_scaffold` and `ai_explain_core` test modules acquire the lock, the allow is no longer needed.
- **Fix:** Removed `#[allow(dead_code)]` attribute; clippy passes clean.
- **Files modified:** ferro-mcp/src/lib.rs
- **Commit:** 4d522341

**4. [Rule 1 - Bug] `resolve_kind_priority` returns `&'static str` not `&str` with lifetime**
- **Found during:** Task 2 implementation
- **Issue:** The CLI version returned `&str` lifetime-tied to the `type_override` parameter, but this breaks the `'static` promise needed when returning fixed string literals. Returning `&'static str` requires matching on the override value and returning the corresponding literal — unknown values return `"not_found"`.
- **Fix:** Changed return type to `&'static str` and added a `match` on known override strings. The test cases from the CLI were relocated unchanged and all pass.
- **Files modified:** ferro-mcp/src/tools/ai_explain_core.rs
- **Commit:** 4d522341

## Known Stubs

None. Both cores implement the full pipeline:
- `scaffold_core`: introspection assembly → relevance filter → sanitize + prompt → `complete_with::<ServiceDef>()` → `validate()` → `Ok(service)`.
- `explain_core`: resolve_target → structured branch (zero LLM) or prose branch (call_llm_prose).

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes introduced beyond what the plan's threat model already covers. All mitigations from the plan's threat register are implemented:

| Threat ID | Mitigation Status |
|-----------|-------------------|
| T-172-PI | IMPLEMENTED — `sanitize_description` relocated intact, applied before embedding in `<description>` block, asserted by 4 unit tests |
| T-172-PI-EXPLAIN | IMPLEMENTED — `target` used as lookup key only; `NotFound` returns `Err` with no LLM call; documented in `explain_core` doc comment |
| T-172-DISK | IMPLEMENTED — `scaffold_core` returns `ServiceDef` value, no file write; no `render_output` call in `ai_scaffold.rs` |
| T-172-CRASH | IMPLEMENTED — both cores return `Result<_, String>`; no `process::exit`/`panic` in either module |

## Self-Check: PASSED

- `ferro-mcp/src/tools/ai_scaffold.rs` exists and contains `pub async fn scaffold_core` ✓
- `ferro-mcp/src/tools/ai_explain_core.rs` exists and contains `pub async fn explain_core` ✓
- `ferro-mcp/src/tools/mod.rs` contains `pub mod ai_scaffold` and `pub mod ai_explain_core` ✓
- `ferro-mcp/src/lib.rs` ENV_LOCK no longer has `#[allow(dead_code)]` ✓
- Commit 4d522341 exists ✓
- `cargo build -p ferro-mcp` clean ✓
- `cargo clippy -p ferro-mcp --all-features -- -D warnings` clean ✓
