---
phase: 162-json-ui-improvements-batch-1-components-expressions-and-spec
plan: "02"
subsystem: ferro-json-ui
tags: [render, data-table, url-interpolation, tdd]
dependency_graph:
  requires: []
  provides:
    - DataTable row_actions arbitrary column-key URL interpolation (D-03/D-04)
  affects:
    - ferro-json-ui/src/render/data.rs
tech_stack:
  added: []
  patterns:
    - row.as_object() iteration for per-row placeholder substitution
key_files:
  modified:
    - ferro-json-ui/src/render/data.rs
decisions:
  - template_actions applies named-column substitution before legacy {row_key}/{id} to ensure column keys always get priority
  - Only String and Number row values substituted; booleans/nulls/arrays/objects skipped via continue — cannot meaningfully appear in URLs
  - Missing placeholders left unsubstituted via String::replace no-op — no special handling needed
metrics:
  duration: 138s
  completed: "2026-05-16T16:55:24Z"
  tasks_completed: 1
  files_modified: 1
---

# Phase 162 Plan 02: DataTable Row_Actions URL Placeholder Generalization Summary

**One-liner:** Extended `template_actions` to substitute any row column key (`{label}`, `{slug_path}`, `{status}`, …) in DataTable row_action URLs, with legacy `{row_key}` and `{id}` preserved and missing keys left unsubstituted.

## What Was Built

Generalized `template_actions` in `ferro-json-ui/src/render/data.rs` (lines 281–341) to satisfy CONTEXT decisions D-03 and D-04:

- Column-key substitution iterates `row.as_object()` and applies `String::replace` for each key whose value is a `String` or `Number`. Booleans, nulls, arrays, and objects are skipped.
- Legacy `{row_key}` substitution runs after column-key substitution (preserves v1 semantics from plan 116-05).
- Legacy `{id}` substitution runs after `{row_key}` (convenience shortcut, also preserved).
- Missing placeholders remain literally in the URL — `String::replace` with no match is a no-op, so no special handling is required.

## Tasks

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extend template_actions + 4 tests | 942819a3 | ferro-json-ui/src/render/data.rs |

## Test Coverage

Four new unit tests added to `render::data::tests`:

| Test | What it covers |
|------|---------------|
| `data_table_url_template_replaces_column_key` | Single column key `{slug_path}` substituted |
| `data_table_url_template_replaces_multiple_keys` | Two column keys `{slug_path}/{status}` both substituted |
| `data_table_url_template_missing_key_leaves_placeholder` | `{nonexistent}` left literal, no panic |
| `data_table_row_href_legacy_placeholders` | Regression guard — `{row_key}` and `{id}` still work |

Full suite: 419 passed, 0 failed (`cargo test -p ferro-json-ui --all-features`).

## Deviations from Plan

None — plan executed exactly as written. TDD RED/GREEN cycle followed:
- RED: 2 tests failed (column keys not substituted), 2 passed (existing behavior)
- GREEN: all 4 pass after `template_actions` extension

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. The substitution is render-time only, using server-controlled row data. Matches the trust model documented in the plan's threat register (T-162-02-01, T-162-02-02 both accepted at planning time).

## Self-Check: PASSED

- [x] `ferro-json-ui/src/render/data.rs` exists and contains `row.as_object()`
- [x] Commit 942819a3 exists
- [x] 4 new test functions present (grep count = 4)
- [x] `cargo test -p ferro-json-ui --all-features` = 419 passed, 0 failed
- [x] `cargo clippy -p ferro-json-ui --all-targets -- -D warnings` clean
- [x] `cargo fmt --all -- --check` clean
