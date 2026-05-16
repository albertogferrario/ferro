---
phase: 162-json-ui-improvements-batch-1-components-expressions-and-spec
plan: "07"
subsystem: ferro-json-ui/spec
tags: [validation, spec, error-handling, d07, d08]
dependency_graph:
  requires: ["162-05"]
  provides: ["SpecError::FooterMissing", "validate_footer_ids"]
  affects: ["ferro-json-ui/src/spec.rs", "ferro-json-ui/src/render/containers.rs"]
tech_stack:
  added: []
  patterns: ["parse-time structural validation", "eprintln! stderr warning for non-fatal issues"]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/spec.rs
    - ferro-json-ui/src/render/containers.rs
decisions:
  - "D-07: dangling footer IDs caught at parse/build time via SpecError::FooterMissing, not silently at render time"
  - "D-08: duplicate footer+children entry emits eprintln! warning (non-fatal, no new dependency on tracing)"
  - "Existing render-time diagnostic test updated: dangling footer now rejected at spec construction, not emitted as HTML comment"
metrics:
  duration: "~15 minutes"
  completed: "2026-05-16T17:25:19Z"
  tasks_completed: 2
  files_modified: 2
---

# Phase 162 Plan 07: Footer ID Spec Validation Summary

**One-liner:** `SpecError::FooterMissing` catches dangling Card/Modal footer references at parse time; eprintln! warns on footer+children duplicates without breaking the parse.

## What Was Built

### Task 1: SpecError::FooterMissing + validate_footer_ids

Added to `ferro-json-ui/src/spec.rs`:

- `SpecError::FooterMissing { element_id: String, footer_id: String }` — structured error variant carrying the parent element ID and the unknown footer reference ID.
- `validate_footer_ids(spec: &Spec) -> Result<(), SpecError>` — walks every element's `props.footer` array (via `serde_json::Value::get`), checks each ID against `spec.elements`, returns `FooterMissing` on first miss. Also emits a stderr warning (D-08) when a footer ID also appears in the element's `children` list — non-fatal, no new crate dependency.
- Wired into `validate_structure` between `validate_no_dangling` and `detect_cycle`.
- Two new tests: `from_json_rejects_missing_footer_id` and `spec_warns_duplicate_footer_child`.

Updated `ferro-json-ui/src/render/containers.rs`:

- `card_missing_footer_id_emits_diagnostic` renamed to `card_missing_footer_id_rejected_at_parse_time` — the old render-time HTML diagnostic comment is superseded by the parse-time error. Test now asserts `SpecBuilder::build()` returns `Err(SpecError::FooterMissing { .. })` instead of checking for an HTML comment.

### Task 2: Wave 2 full-suite gate

`cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` all exit 0.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated pre-existing render-time diagnostic test**

- **Found during:** Task 1 — `cargo test -p ferro-json-ui --all-features` reported `render::containers::tests::card_missing_footer_id_emits_diagnostic` panicking because `build_spec(...)` now returns `Err(FooterMissing)` rather than constructing a spec that renders an HTML diagnostic comment.
- **Issue:** The old test was written when D-07 wasn't implemented — it verified the render-time HTML fallback (`<!-- ferro-json-ui: element references missing id 'ghost' -->`). Our new `validate_footer_ids` is called from `validate_structure`, which `SpecBuilder::build()` also invokes, so the spec construction itself now fails.
- **Fix:** Replaced the test body to assert `build()` returns `Err(SpecError::FooterMissing { element_id: "root", footer_id: "ghost" })`. Renamed test to `card_missing_footer_id_rejected_at_parse_time` to match the new behavior.
- **Files modified:** `ferro-json-ui/src/render/containers.rs`
- **Commit:** eb86c70e (same commit as the feature, since the fix was needed for the test suite to pass)

## Acceptance Criteria Verification

- `grep -c "FooterMissing" ferro-json-ui/src/spec.rs` = 4 (>= 3 required)
- `grep -c "fn validate_footer_ids" ferro-json-ui/src/spec.rs` = 1
- `grep -c "validate_footer_ids(spec)" ferro-json-ui/src/spec.rs` = 1
- `grep -c "fn from_json_rejects_missing_footer_id\|fn spec_warns_duplicate_footer_child" ferro-json-ui/src/spec.rs` = 2
- `cargo test -p ferro-json-ui --all-features` exits 0 (431 lib + 11 validation + 8 round_trip + 5 doc-tests)
- Full suite exits 0

## Known Stubs

None.

## Threat Flags

None — no new network endpoints, auth paths, or trust boundary changes. `FooterMissing` exposes author-provided element IDs in error messages, same trust model as existing `DanglingChild` (T-162-07-01 accepted per threat register).

## Self-Check: PASSED

- `ferro-json-ui/src/spec.rs` — exists, contains `FooterMissing`, `validate_footer_ids`, both new tests
- `ferro-json-ui/src/render/containers.rs` — exists, updated test present
- Commit eb86c70e — confirmed in `git log --oneline -3`
