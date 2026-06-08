---
phase: 171-ferro-ai-make-ferro-ai-explain-cli-commands
plan: "01"
subsystem: ferro-ai
tags: [ai, sdk, structured-output, options]
dependency_graph:
  requires: []
  provides: [complete_with, CompleteOptions]
  affects: [ferro-ai/src/complete.rs, ferro-ai/src/lib.rs]
tech_stack:
  added: []
  patterns: [options-struct delegation, capturing-mock test pattern]
key_files:
  created: []
  modified:
    - ferro-ai/src/complete.rs
    - ferro-ai/src/lib.rs
decisions:
  - CompleteOptions carries max_tokens/system/model_override with Default (4096/None/None) — matches prior hardcoded values exactly, zero behavior change for existing callers
  - complete::<T>() rewritten as a one-liner delegate to complete_with — no duplicated schema-normalization logic
  - CapturingClient mock uses std::sync::Mutex<Option<CompletionRequest>> to assert per-field request values without introducing async lock overhead
metrics:
  duration: 161s
  completed: "2026-06-08"
  tasks_completed: 2
  files_modified: 2
---

# Phase 171 Plan 01: CompleteOptions + complete_with SDK Extension Summary

One-liner: `complete_with::<T>(client, prompt, CompleteOptions { max_tokens, system, model_override })` added to ferro-ai; `complete::<T>()` becomes a zero-config delegate through the same ServiceDef-aware schema-normalizer path.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extract CompleteOptions + complete_with, make complete a delegate | 9d978626 | ferro-ai/src/complete.rs |
| 2 | Re-export complete_with + CompleteOptions from crate root | d18074cf | ferro-ai/src/lib.rs |

## What Was Built

`CompleteOptions` struct (max_tokens: u32, system: Option<String>, model_override: Option<String>) with `Default` producing (4096, None, None). `complete_with::<T>()` accepts this struct and builds the `CompletionRequest` from its fields, routing through `schema::for_structured_output` — the same ServiceDef-aware normalizer path as before. `complete::<T>()` is now a single-line delegate: `complete_with(client, prompt, CompleteOptions::default()).await`. Both symbols are re-exported from the crate root so callers can import `ferro_ai::{AiConfig, CompleteOptions, complete_with}`.

Three new unit tests added:
- `complete_options_default` — asserts Default values
- `complete_with_uses_provided_max_tokens` — uses CapturingClient to assert max_tokens/system/model_override forwarded to CompletionRequest
- `complete_delegates_to_complete_with` — asserts complete passes Default values through

## Deviations from Plan

None — plan executed exactly as written.

## TDD Gate Compliance

RED gate: tests added first, confirmed to fail with `cannot find struct CompleteOptions` / `cannot find function complete_with` compile errors.
GREEN gate: implementation added, all 3 tests passed. 95 total ferro-ai lib tests pass.

## Verification

- `cargo test -p ferro-ai --lib`: 95 passed, 0 failed
- `cargo clippy -p ferro-ai --all-targets -- -D warnings`: clean
- All 5 acceptance criteria verified via grep + build

## Known Stubs

None.

## Threat Flags

None — no new network endpoints, auth paths, or trust-boundary changes. `max_tokens` parameter is the cost-control lever; default remains 4096 (T-171-01 disposition: mitigate via per-call cap, wired to env var in Plans 02/03).

## Self-Check: PASSED

- `ferro-ai/src/complete.rs` modified — confirmed present
- `ferro-ai/src/lib.rs` modified — confirmed present
- Commit `9d978626` — confirmed in git log
- Commit `d18074cf` — confirmed in git log
