---
phase: 163
plan: 08
subsystem: ferro-json-ui
tags: [testing, directives, each, if, integration]
dependency_graph:
  requires: [163-03, 163-04]
  provides: [e2e-directive-validation]
  affects: [ferro-json-ui]
tech_stack:
  added: []
  patterns: [full-pipeline integration testing, spec-fixture TDD]
key_files:
  created:
    - ferro-json-ui/tests/directives_e2e.rs
  modified: []
decisions:
  - "render_spec_to_html takes (spec, data) separately — tests clone spec.data before expand_directives and pass it at render time"
  - "All four test names match the plan spec exactly for grep-based acceptance check"
metrics:
  duration: 98s
  completed: "2026-05-16T21:31:52Z"
  tasks_completed: 1
  files_modified: 1
---

# Phase 163 Plan 08: E2E Integration Tests for Directive Lifecycle Summary

End-to-end integration test file covering the full pipeline `Spec::from_json` → `expand_directives` → `render_spec_to_html` for the `$each` and `$if` directives.

## What Was Built

`ferro-json-ui/tests/directives_e2e.rs` — four integration tests:

1. **`e2e_orders_kanban_each_produces_n_cards`** — kanban fixture with `$each` over 3-row orders array; verifies all 3 order numbers and customer names appear in rendered HTML. Mirrors the cassa friction site.

2. **`e2e_conditional_action_button_if_truthy_renders`** — button with `$if` predicate; verifies label present when data is truthy, absent when data is falsy. Both paths exercised in a single test.

3. **`e2e_correlated_each_children_groups_per_row`** — sibling `card` and `badge` templates with identical `{path, as}` directives; verifies all four values render and that positional ordering confirms per-row grouping (ITEM_ONE → BADGE_ONE → ITEM_TWO → BADGE_TWO).

4. **`e2e_static_spec_unchanged_by_expand_directives`** — spec with no directives; verifies `expand_directives` produces no changes (idempotency).

## Test Results

- `cargo test -p ferro-json-ui --test directives_e2e --all-features`: 4/4 pass
- `cargo test -p ferro-json-ui --all-features`: 496 total (unit + integration + doc-tests), all pass
- `cargo clippy --all --all-targets -- -D warnings`: clean
- `cargo fmt --all -- --check`: clean (fmt applied once)

## Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create directives_e2e.rs integration tests | 1db9fbf3 | ferro-json-ui/tests/directives_e2e.rs |

## Deviations from Plan

None — plan executed exactly as written. The render API is `render_spec_to_html(spec, data)` (not the with_plugins variant), which is re-exported from `ferro-json-ui/src/lib.rs:79`. Test helpers clone `spec.data` before calling `expand_directives` and pass the clone at render time (matching the actual signature).

## Known Stubs

None.

## Threat Flags

None — tests only, no new attack surface.

## Self-Check: PASSED

- `ferro-json-ui/tests/directives_e2e.rs`: FOUND
- Commit `1db9fbf3`: FOUND
- 4 named test functions: FOUND (grep confirms all four names)
