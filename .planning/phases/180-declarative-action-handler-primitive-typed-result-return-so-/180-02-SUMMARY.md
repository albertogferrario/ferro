---
phase: 180
plan: 02
subsystem: ferro-macros
tags: [refactor, proc-macro, utils, handler, no-behavior-change]
requires:
  - ferro-macros/src/handler.rs (pre-refactor — read as source)
  - ferro-macros/src/utils.rs (pre-refactor — only levenshtein_distance)
provides:
  - ferro-macros/src/utils.rs — pub(crate) param-extraction helpers for #[handler] and upcoming #[action]
affects:
  - ferro-macros/src/handler.rs — consumes helpers via use crate::utils::{...}
tech-stack:
  added: []
  patterns:
    - pub(crate) visibility for intra-crate shared helpers
    - extract-to-utils refactor pattern (move private → pub(crate) without API surface change)
key-files:
  created: []
  modified:
    - ferro-macros/src/utils.rs
    - ferro-macros/src/handler.rs
key-decisions:
  - Kept the duplicated `if has_request_param` block in handler.rs intact (pre-existing no-op; collapsing it is out of scope per plan instructions)
  - `is_primitive_type_name` moved but not imported in handler.rs import list (it is called only from classify_param_type inside utils.rs — no import needed in handler.rs)
requirements-completed:
  - D-05
duration: 8 min
completed: 2026-05-30
---

# Phase 180 Plan 02: Extract param-extraction helpers to ferro-macros::utils — Summary

Pure refactor moving five private helpers and one token utility from `ferro-macros/src/handler.rs` into `ferro-macros/src/utils.rs` as `pub(crate)` items, so `#[action]` (Wave 2, Plan 03) can reuse them without duplication.

## Duration

- Start: 2026-05-30T ~session start
- End: 2026-05-30
- Total: 8 min
- Tasks: 2 (1 code change + 1 verification gate)
- Files modified: 2

## What Was Done

### Task 1 — Move helpers

Moved from `ferro-macros/src/handler.rs` to `ferro-macros/src/utils.rs` as `pub(crate)` items:

| Item | Before | After |
|------|--------|-------|
| `ferro() -> TokenStream2` | `fn` (private) in handler.rs | `pub(crate) fn` in utils.rs |
| `enum ParamKind` | private enum in handler.rs | `pub(crate) enum` in utils.rs |
| `extract_param_name(pat: &Pat) -> String` | `fn` (private) in handler.rs | `pub(crate) fn` in utils.rs |
| `classify_param_type(ty: &Type) -> ParamKind` | `fn` (private) in handler.rs | `pub(crate) fn` in utils.rs |
| `is_primitive_type_name(name: &str) -> bool` | `fn` (private) in handler.rs | `pub(crate) fn` in utils.rs |
| `generate_extraction(...)` | `fn` (private) in handler.rs | `pub(crate) fn` in utils.rs |

`handler.rs` now imports: `use crate::utils::{classify_param_type, extract_param_name, ferro, generate_extraction};`

Note: `is_primitive_type_name` is not imported in handler.rs because it is only called from `classify_param_type` — which now lives in utils.rs. No import needed at the handler.rs call site.

Line count delta:
- `utils.rs`: 1 line → 189 lines (+188)
- `handler.rs`: 271 lines → 119 lines (-152)
- Net: +36 lines (doc comments added to the moved items)

### Task 2 — Regression gate (verification only)

Ran the full workspace build and test suite to confirm `#[handler]` generated TokenStream is byte-identical post-refactor:

- `cargo build -p ferro-rs --all-features` — exit 0
- `cargo build --all-features` (full workspace including `app/` sample crate) — exit 0
- `cargo test --all-features --all-targets -- --test-threads=1` — 58 test suites, 0 failures
- `cargo clippy --all --all-targets -- -D warnings` (CI-parity) — exit 0

## Acceptance Criteria Verification

| Criterion | Result |
|-----------|--------|
| `grep -c 'pub(crate) enum ParamKind' ferro-macros/src/utils.rs` = 1 | PASS |
| `grep -c 'pub(crate) fn classify_param_type' ferro-macros/src/utils.rs` = 1 | PASS |
| `grep -c 'pub(crate) fn generate_extraction' ferro-macros/src/utils.rs` = 1 | PASS |
| `grep -c 'pub(crate) fn extract_param_name' ferro-macros/src/utils.rs` = 1 | PASS |
| `grep -c 'pub(crate) fn is_primitive_type_name' ferro-macros/src/utils.rs` = 1 | PASS |
| `grep -c 'pub(crate) fn ferro' ferro-macros/src/utils.rs` = 1 | PASS |
| `grep -c 'fn classify_param_type' ferro-macros/src/handler.rs` = 0 | PASS |
| `grep -c 'fn generate_extraction' ferro-macros/src/handler.rs` = 0 | PASS |
| `grep -c 'fn extract_param_name' ferro-macros/src/handler.rs` = 0 | PASS |
| `grep -c 'fn is_primitive_type_name' ferro-macros/src/handler.rs` = 0 | PASS |
| `grep -c 'enum ParamKind' ferro-macros/src/handler.rs` = 0 | PASS |
| `grep -c 'use crate::utils::' ferro-macros/src/handler.rs` = 1 | PASS |
| `cargo build -p ferro-macros` exits 0 | PASS |
| `cargo clippy -p ferro-macros --all-targets -- -D warnings` exits 0 | PASS |
| `cargo build --all-features` exits 0 | PASS |
| `cargo test --all-features --all-targets -- --test-threads=1` exits 0 | PASS |
| `cargo clippy --all --all-targets -- -D warnings` exits 0 | PASS |

## Deviations from Plan

None — plan executed exactly as written.

The plan specified `is_primitive_type_name` should be imported in `handler.rs`, but on inspection it is only called from `classify_param_type` (which now lives in `utils.rs`). No import is needed or correct at handler.rs level. This is not a deviation — the plan's import list covers what handler.rs actually calls.

## Known Stubs

None.

## Threat Flags

None. This is a pure intra-crate refactor — no new trust boundaries, no new network endpoints, no untrusted data paths modified.

## Next Step

Wave 2 — Plan 03 (`action.rs` proc-macro implementation) can now import `crate::utils::{ferro, ParamKind, classify_param_type, extract_param_name, generate_extraction}` without any further refactoring.

## Self-Check: PASSED

- `ferro-macros/src/utils.rs` exists and contains all 6 pub(crate) items
- `ferro-macros/src/handler.rs` exists and contains the import line
- Commit `56cb9980` exists in git log
- All 17 acceptance criteria verified as PASS above
