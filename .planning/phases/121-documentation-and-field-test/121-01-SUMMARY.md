---
phase: 121
plan: "01"
subsystem: framework/json_ui
tags: [json-ui, v2, render-file, file-loader, framework-integration]
dependency_graph:
  requires: [ferro-json-ui load_cached, ferro-json-ui Spec::merge_data]
  provides: [JsonUi::render_file, JsonUi::render_file_with_config]
  affects: [framework/src/json_ui/mod.rs]
tech_stack:
  added: []
  patterns: [file-backed spec cache, dev/prod reload toggle via Config::is_production]
key_files:
  created: []
  modified:
    - framework/src/json_ui/mod.rs
decisions:
  - "Pass spec.data (post-merge) as the data argument to build_response — render pipeline reads from spec.data after merge_data"
  - "reload flag derived from !Config::is_production() — consistent with load_cached API intent"
metrics:
  duration: "~8 min"
  completed: "2026-05-15T16:26:44Z"
  tasks_completed: 1
  files_modified: 1
requirements:
  - FIELD-01
---

# Phase 121 Plan 01: Add JsonUi::render_file — Summary

Added `JsonUi::render_file` and `JsonUi::render_file_with_config` to the framework, closing the missing-method blocker for FIELD-01.

## One-liner

File-backed JSON-UI rendering via process-level spec cache with dev/prod reload toggle and handler data merge.

## What was built

Two new public methods on `JsonUi` in `framework/src/json_ui/mod.rs`:

- `render_file(path, handler_data)` — loads spec from disk, merges handler data, renders HTML. Delegates to `render_file_with_config` with default config.
- `render_file_with_config(path, handler_data, config)` — full control over config. Uses `ferro_json_ui::load_cached(path, reload)` where `reload = !Config::is_production()`. Merges handler data via `Spec::merge_data`. Calls the existing `build_response` method which handles layout, head, plugin assets, and the full HTML shell.

The render pipeline after file load: `load_cached` → `(*arc).clone()` → `merge_data(handler_data)` → `build_response(&spec, &spec.data, config)`. Passing `spec.data` as the data argument is correct because `merge_data` merges handler keys into `spec.data` (shallow top-level merge, handler wins).

## Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add JsonUi::render_file and render_file_with_config | 776f6fdb | framework/src/json_ui/mod.rs |

## Deviations from Plan

None — plan executed exactly as written. The implementation matched the plan's pseudocode. The v2 render entry point (`render_spec_to_html_with_plugins`) is consumed indirectly through the existing `build_response` method rather than called directly, which correctly reuses the full head/layout/plugin pipeline.

## Test Results

- `cargo test -p ferro-rs --all-features -- json_ui`: 42 passed, 0 failed
- New test `render_file_returns_error_for_missing_file` passes (500 on missing path)
- `cargo clippy -p ferro-rs --all-targets --all-features -- -D warnings`: 0 errors

## Known Stubs

None.

## Threat Flags

No new network endpoints, auth paths, or trust-boundary surface beyond what the plan's threat model already covers (T-121-01, T-121-02). Error message uses `LoadError`'s Display impl which does not leak absolute paths beyond what the error type itself exposes.

## Self-Check: PASSED

- `grep -n "pub fn render_file" framework/src/json_ui/mod.rs` → lines 152 and 160 (2 matches)
- `grep -n "load_cached" framework/src/json_ui/mod.rs` → lines 149, 166
- `grep -n "merge_data" framework/src/json_ui/mod.rs` → line 168
- Commit 776f6fdb confirmed in git log
- No file deletions in task commit
