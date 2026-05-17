---
phase: 164
plan: "04"
subsystem: ferro-json-ui / framework
tags: [spec, title-binding, data-binding, renderer, d-12, v7-runtime-friction]
dependency_graph:
  requires: [164-01, 164-03]
  provides: [TitleBinding, DataRef, Spec.title-binding-resolution]
  affects: [ferro-json-ui/spec.rs, ferro-json-ui/lib.rs, framework/src/json_ui/mod.rs]
tech_stack:
  added: [TitleBinding enum, DataRef struct, schemars::JsonSchema on spec types]
  patterns: [serde untagged enum, JSON Pointer resolution via serde_json::Value::pointer, SpecBuilder title_binding() method]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/spec.rs
    - ferro-json-ui/src/lib.rs
    - framework/src/json_ui/mod.rs
decisions:
  - "Used serde untagged enum (TitleBinding) mirroring Visibility pattern rather than a raw serde_json::Value binding to preserve round-trip shape (Pitfall 5)"
  - "Added title_binding() method to SpecBuilder so Rust callers can construct binding titles; title() remains a String convenience method"
  - "Binding resolution uses serde_json::Value::pointer (already present in framework) rather than exposing crate-internal resolve_path across crate boundary"
  - "title_owned: String + let title: &str = &title_owned preserves the existing LayoutContext.title: &str call shape without changing the layout API"
metrics:
  duration_minutes: 35
  completed: "2026-05-17T01:47:16Z"
  tasks_completed: 3
  files_modified: 3
---

# Phase 164 Plan 04: TitleBinding — Spec.title accepts literal or `$data` binding

## One-liner

Typed `TitleBinding` enum (Literal/Binding variants) replaces `Option<String>` on `Spec.title`; renderer resolves bindings against `spec.data` at response-build time via JSON Pointer.

## What was built

Closes V7-RUNTIME friction **F1 / D-12**: 23 gestiscilo specs had to strip `{"$data": "/path"}` bindings from title fields via `sed` because `Spec.title` only accepted `String`. This plan introduces the correct type so the binding is preserved in the spec document and resolved by the renderer.

### New types in `ferro-json-ui/src/spec.rs`

- `TitleBinding` — `#[serde(untagged)]` enum with `Literal(String)` and `Binding(DataRef)` variants. Derives `Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema`.
- `DataRef` — `pub struct DataRef { pub data: String }` with `#[serde(rename = "$data")]`. Preserves the `{"$data": "/path"}` wire shape exactly on round-trip.
- `SpecWire.title` changed from `Option<String>` to `Option<TitleBinding>` (internal deserializer wire struct).
- `Spec.title` changed from `Option<String>` to `Option<TitleBinding>`.
- `SpecBuilder.title` field changed from `Option<String>` to `Option<TitleBinding>`.
- `SpecBuilder.title()` method now wraps the string in `TitleBinding::Literal`.
- `SpecBuilder.title_binding()` new method wraps a path string in `TitleBinding::Binding(DataRef { data })`.

### Updated renderer in `framework/src/json_ui/mod.rs`

Replaced the literal-only extraction:
```rust
let title = spec.title.as_deref().unwrap_or("Ferro");
```
with a full match resolving both variants:
```rust
let title_owned: String = match &spec.title {
    None => "Ferro".to_string(),
    Some(TitleBinding::Literal(s)) => s.clone(),
    Some(TitleBinding::Binding(r)) => {
        spec.data.pointer(&r.data)
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| "Ferro".to_string())
    }
};
let title: &str = &title_owned;
```

The `LayoutContext.title: &'a str` contract is preserved — no layout API changes.

### Re-exports in `ferro-json-ui/src/lib.rs`

`TitleBinding` and `DataRef` added to the `pub use spec::{...}` block.

## Tests added

### ferro-json-ui/src/spec.rs (4 round-trip tests)

| Test | Purpose |
|------|---------|
| `spec_title_literal_roundtrip` | Literal JSON string → `TitleBinding::Literal` → serializes back to `"title":"Hello"` |
| `spec_title_binding_roundtrip` | `{"$data":"/page_title"}` → `TitleBinding::Binding(DataRef)` → serializes back with `"$data"` key preserved |
| `spec_title_absent` | Spec without `title` field → `spec.title.is_none()` |
| `spec_title_invalid_shape_rejected` | `{"foo":"bar"}` title shape → serde parse error |

### framework/src/json_ui/mod.rs (3 render integration tests)

| Test | Purpose |
|------|---------|
| `render_title_literal` | Spec with literal title "Hello" → `<title>Hello</title>` in HTML output |
| `render_title_binding_resolves` | Spec with `title_binding("/page_title")` and data `{"page_title":"Dynamic"}` → `<title>Dynamic</title>` |
| `render_title_binding_missing_path_falls_back` | Spec with `title_binding("/missing")` and empty data → `<title>Ferro</title>` |

## Consumers of `spec.title` updated

| File | Line | Change |
|------|------|--------|
| `ferro-json-ui/src/spec.rs` | 82 | `pub title: Option<TitleBinding>` (was `Option<String>`) |
| `ferro-json-ui/src/spec.rs` | 327 | `SpecBuilder.title: Option<TitleBinding>` |
| `ferro-json-ui/src/spec.rs` | 634 | `SpecWire.title: Option<TitleBinding>` (internal) |
| `framework/src/json_ui/mod.rs` | 89 | match on TitleBinding variants (was `as_deref().unwrap_or`) |
| `ferro-json-ui/src/projection/builder.rs` | 204, 273 | No change needed — calls `.title(String)` on SpecBuilder, which still works via `TitleBinding::Literal` wrapping |

## Deviations from Plan

None — plan executed exactly as written. The SpecBuilder API update (wrapping String in `TitleBinding::Literal` inside `title()`) was implied by the plan and required for backward compatibility with all call sites including `ferro-json-ui/src/projection/builder.rs`.

## Security (T-164-04-02 audit)

Confirmed: `base_document()` in `ferro-json-ui/src/layout.rs` applies `html_escape(title)` before emitting into `<title>...</title>`. The binding-resolved title string goes through the same escape path as the literal title. No new XSS surface introduced. Existing test `html_escaping_prevents_xss_in_title` covers this path.

## Self-Check: PASSED

- `ferro-json-ui/src/spec.rs` — TitleBinding, DataRef, Spec.title updated: FOUND
- `ferro-json-ui/src/lib.rs` — TitleBinding, DataRef re-exported: FOUND
- `framework/src/json_ui/mod.rs` — TitleBinding match, render_title tests: FOUND
- Commit `ac937896`: spec.rs + lib.rs changes
- Commit `7a047434`: framework mod.rs changes
- `cargo fmt --all -- --check`: PASSED
- `cargo clippy --all --all-targets -- -D warnings`: PASSED
- `cargo test --all-features`: PASSED (0 failures)
