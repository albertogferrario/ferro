---
phase: 163-json-ui-improvements-batch-2-cassa-and-calendario-field-test
plan: "03"
subsystem: ui
tags: [ferro-json-ui, directives, expand_directives, each, if, resolve-pipeline, tdd]

requires:
  - phase: 163-01
    provides: EachDirective struct and Element.each field with serde round-trip
  - phase: 163-02
    provides: Element.if_ field reusing Visibility enum

provides:
  - expand_directives public function in ferro-json-ui/src/resolve.rs with $if-removal, $each-expansion, and parent-children rewriting sub-passes
  - JsonUi::resolve wired to call expand_directives FIRST (before resolve_actions and resolve_expressions)
  - 12 inline unit tests covering all seven directive-expansion behaviors
  - D-04 compliance: Visibility::evaluate is the sole predicate evaluator

affects: [163-04, 163-05, 163-06, 163-08, 163-10]

tech-stack:
  added: []
  patterns:
    - "Resolve pipeline ordering: expand_directives (directives) → resolve_actions (routing) → resolve_expressions (data binding)"
    - "Idempotent directive expansion: clone fields cleared (each=None, if_=None) so re-running is a no-op"
    - "Correlated child indexing: parent clone at index i references child clone at index i when both share the same {path, as}"
    - "Row-path pre-resolution: /{as}/field paths in clone props are inlined to literal values before resolve_expressions runs"

key-files:
  created: []
  modified:
    - ferro-json-ui/src/resolve.rs
    - ferro-json-ui/src/lib.rs
    - framework/src/json_ui/mod.rs

key-decisions:
  - "$if evaluated FIRST when co-occurring with $each: falsy removes the element before any clones are produced"
  - "Correlated child indexes: cloned parent at index i references cloned child at index i when both $each over the same {path, as}"
  - "Sole predicate evaluator is Visibility::evaluate (D-04 reuse mandate) — no parallel evaluator"
  - "Nested $each (a templated element whose body contains another $each on its own descendants) is deferred to Plan 04 validator rejection (SpecError::NestedEach)"

patterns-established:
  - "expand_directives: three sequential sub-passes (remove_if_falsy → expand_each → rewrite_parent_children)"
  - "inline_resolve_row_paths: walks props Value tree, replaces {$data: /{as}/...} with literal row values before downstream resolve_expressions"

requirements-completed: []

duration: pre-committed (verification only)
completed: 2026-05-16
---

# Phase 163 Plan 03: expand_directives Summary

**Resolve-time `expand_directives` pass that materializes `$each` into N clones with auto-suffixed IDs, removes `$if`-falsy elements, rewires parent children lists, and pre-resolves row-scoped `$data` paths before downstream expression resolution**

## Performance

- **Duration:** pre-committed by orchestrator (verification pass only)
- **Started:** 2026-05-16T21:08:54Z
- **Completed:** 2026-05-16
- **Tasks:** 1 (TDD: RED + GREEN)
- **Files modified:** 3

## Accomplishments

- Delivered the killer feature for Phase 163: `expand_directives` transforms `$each` / `$if` directives from dead JSON into live element sets at resolve time
- Three-sub-pass decomposition (`remove_if_falsy` → `expand_each` → `rewrite_parent_children`) is fully idempotent and ordering-safe
- JsonUi::resolve pipeline locked: expand_directives runs FIRST so all downstream resolution (routing, expressions) operates on the already-expanded element set
- D-04 honored end-to-end: `Visibility::evaluate` is the sole predicate evaluator for `$if` (no parallel evaluator function)
- 12 unit tests cover all seven behaviors in must_haves.truths plus idempotency (Test 12)

## Task Commits

TDD cycle (RED → GREEN):

1. **Task 1 RED: add failing tests for expand_directives** - `b0bf708e` (test)
2. **Task 1 GREEN: implement expand_directives resolve-time pass** - `6b35a3a5` (feat)

## Files Created/Modified

- `ferro-json-ui/src/resolve.rs` — `expand_directives` public function with `remove_if_falsy`, `expand_each`, `rewrite_parent_children`, `inline_resolve_row_paths`, `inline_walk`, `interpolate_row_template`, `contains_template_marker`, `value_to_string` helpers, and 12 inline unit tests
- `ferro-json-ui/src/lib.rs` — `expand_directives` added to the `pub use resolve::{...}` re-export list
- `framework/src/json_ui/mod.rs` — `expand_directives` added to import list and called FIRST inside `JsonUi::resolve` and `JsonUi::resolve_with_errors`

## Decisions Made

- `$if`-first ordering when co-occurring with `$each`: eliminates elements before clones are produced (no clones for falsy elements)
- Correlated child indexes: the i-th parent clone references the i-th child clone when both template over the same `{path, as}` pair — mismatched pairs are deferred to Plan 04 (SpecError::MismatchedEach)
- `inline_resolve_row_paths` pre-resolves only `/{as}/` paths; non-row `$data` paths are left for `resolve_expressions`
- `$template` strings with row-scoped `{/{as}/field}` markers are partially interpolated inline; non-row markers are left intact for downstream resolution

## Deviations from Plan

None — plan executed exactly as written. The implementation was pre-committed by the orchestrator before this executor was spawned.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 163-04 (validator gates: SpecError::MismatchedEach, NestedEach, etc.) can proceed immediately — expand_directives is the upstream function the validator will call before checking
- Plan 163-05 (SpecBuilder ergonomic layer) can proceed independently
- Plan 163-08 (end-to-end directive integration tests) is now unblocked — the full pipeline materializes directives at render time

## Self-Check: PASSED

- `ferro-json-ui/src/resolve.rs` contains `pub fn expand_directives` (1 match)
- `ferro-json-ui/src/lib.rs` re-exports `expand_directives` on line 81
- `framework/src/json_ui/mod.rs` calls `expand_directives` BEFORE `resolve_actions` (lines 50, 204)
- No parallel evaluator (`fn evaluate_if` etc.) in resolve.rs
- `.evaluate(` present in resolve.rs (Visibility::evaluate call site)
- Both commits verified: `b0bf708e` (test), `6b35a3a5` (feat)
- 12/12 expand_ tests pass: `cargo test -p ferro-json-ui --lib expand_ --all-features`
- Full test suite clean: `cargo test -p ferro-json-ui --all-features`
- Build clean: `cargo build -p ferro-rs`
- Clippy clean: `cargo clippy -p ferro-json-ui --all-targets -- -D warnings`
- Format clean: `cargo fmt --all -- --check`

---
*Phase: 163-json-ui-improvements-batch-2-cassa-and-calendario-field-test*
*Completed: 2026-05-16*
