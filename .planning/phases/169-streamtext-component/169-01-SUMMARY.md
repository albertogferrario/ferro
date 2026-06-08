---
phase: 169-streamtext-component
plan: "01"
subsystem: ferro-json-ui
tags: [json-ui, streaming, sse, xss-mitigation, component]
dependency_graph:
  requires: []
  provides: [StreamTextProps, render_streamtext]
  affects: [ferro-json-ui/src/component.rs, ferro-json-ui/src/render/atoms.rs]
tech_stack:
  added: []
  patterns: [RawHtmlProps-analog, html_escape-attribute-mitigation, decode_props-decode_diagnostic]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/atoms.rs
decisions:
  - StreamTextProps uses #[serde(default)] on sse_url and skip_serializing_if on Option fields (matches RawHtmlProps/SkeletonProps analogs exactly)
  - render_streamtext is pub(crate) with no dispatch arm yet — Plan 02 wires it in; dead_code warning is expected in this intermediate state
  - All three props (sse_url, placeholder, loading_text) pass through html_escape before HTML emission (T-169-01 and T-169-01b mitigation)
  - Container emits no layout classes — host application owns layout per UI-SPEC
metrics:
  duration: "275s"
  completed: "2026-06-08"
  tasks_completed: 3
  files_modified: 2
requirements_completed: [AISSE-02]
---

# Phase 169 Plan 01: StreamTextProps struct and render_streamtext leaf renderer

`StreamTextProps` props contract and `render_streamtext` HTML emitter for the StreamText JSON-UI component, with four unit tests including T-169-01 XSS acceptance.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add StreamTextProps struct to component.rs | 328bb87f | ferro-json-ui/src/component.rs |
| 2 | Add render_streamtext leaf renderer to atoms.rs | b0499ad2 | ferro-json-ui/src/render/atoms.rs |
| 3 | Add unit tests (SC#1, SC#2a, SC#2b, T-169-01) | c7aaff47 | ferro-json-ui/src/render/atoms.rs |
| — | Apply rustfmt formatting | 589cdc77 | ferro-json-ui/src/render/atoms.rs |

## Decisions Made

- **StreamTextProps derive set:** `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]` — exact match to `RawHtmlProps` and `SkeletonProps` analogs. No `#[serde(rename_all)]` (enum-only attribute; field names are already snake_case).
- **html_escape on all three props:** `sse_url` is escaped before attribute interpolation (T-169-01); `placeholder` and `loading_text` are escaped before span content emission (T-169-01b). Mirrored from the workspace-wide "raw `<script>` must not appear" bar in existing render tests.
- **pub(crate) with no dispatch arm:** `render_streamtext` is intentionally not referenced by any dispatch arm in this plan. Plan 02 adds the `BUILTIN_TYPES` entry and `match` arm. The dead_code warning on the lib target is expected and resolved by Plan 02.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] rustfmt formatting diff**
- **Found during:** Post-task-3 pre-commit lint gate
- **Issue:** `cargo fmt --all -- --check` reported a diff on the `format!` call in `render_streamtext` (multi-line → single-line collapse) and on the two `assert!` calls in `stream_text_props_minimal_serde_roundtrip` (long line wrapping).
- **Fix:** Applied `cargo fmt --all` and committed the formatting adjustments as a separate style commit.
- **Files modified:** ferro-json-ui/src/render/atoms.rs
- **Commit:** 589cdc77

### Expected Intermediate State

`cargo clippy --all --all-targets -- -D warnings` reports one error in the lib target:

```
error: function `render_streamtext` is never used
```

This is expected per the plan: the function is `pub(crate)` with no dispatch arm yet. Plan 02 adds `"StreamText"` to `BUILTIN_TYPES` and the dispatch `match` arm, making the function live. The full CI gate (`-D warnings`) is designed to run after Plan 02 completes, not after Plan 01 alone. The `#[cfg(test)]` module references the function for testing, but those references do not suppress `dead_code` for the lib target.

## Tests

All 4 tests pass:

- `render::atoms::tests::stream_text_props_serde_roundtrip` — full struct round-trip
- `render::atoms::tests::stream_text_props_minimal_serde_roundtrip` — None fields absent from JSON
- `render::atoms::tests::render_streamtext_emits_data_attribute` — `data-ferro-stream-url` attribute emitted
- `render::atoms::tests::render_streamtext_escapes_url` — T-169-01 acceptance: raw `&b=` and `<script>` absent after escape

## Threat Surface

T-169-01 and T-169-01b mitigated in this plan:

| Flag | File | Description |
|------|------|-------------|
| threat_mitigated: T-169-01 | ferro-json-ui/src/render/atoms.rs | sse_url passed through html_escape before data-ferro-stream-url attribute interpolation |
| threat_mitigated: T-169-01b | ferro-json-ui/src/render/atoms.rs | placeholder and loading_text passed through html_escape before span content emission |

## Self-Check

- [x] `pub struct StreamTextProps` present in ferro-json-ui/src/component.rs
- [x] `fn render_streamtext` present in ferro-json-ui/src/render/atoms.rs
- [x] 4 commits exist: 328bb87f, b0499ad2, c7aaff47, 589cdc77
- [x] `cargo build -p ferro-json-ui` exits 0
- [x] `cargo test -p ferro-json-ui stream_text` and `render_streamtext` — 4 tests green
