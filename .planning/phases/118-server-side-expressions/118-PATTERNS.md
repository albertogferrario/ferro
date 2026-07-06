# Phase 118: Server-Side Expressions — Pattern Map

**Mapped:** 2026-04-19
**Files analyzed:** 3 (1 new, 2 modified)
**Analogs found:** 3 / 3

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-json-ui/src/expression.rs` *(new)* | pre-render resolver (infallible, mutating walker) | transform (`&mut Spec` in place) | `ferro-json-ui/src/resolve.rs` | exact — same role, same signature shape, same crate |
| `ferro-json-ui/src/lib.rs` *(modified)* | module re-export index | module surface | existing `pub mod resolve; pub use resolve::{…};` pair (lines 38, 74) | exact — adding a sibling module with sibling re-exports |
| `framework/src/json_ui/mod.rs` *(modified)* | pipeline integration + test block | request-response | same file, `JsonUi::resolve` (lines 38-43) and `JsonUi::resolve_with_errors` (lines 149-155) | exact — inserting a single call into two existing bridge methods |

Secondary analog for the "pure helper + inline tests" pattern: `ferro-json-ui/src/visibility.rs` (another infallible resolver in the same crate; its `Visibility::evaluate` uses `crate::data::resolve_path` directly).

Secondary analog for `pub(crate)` path helpers consumed verbatim: `ferro-json-ui/src/data.rs`.

---

## Pattern Assignments

### `ferro-json-ui/src/expression.rs` (NEW — resolver, transform)

**Primary analog:** `ferro-json-ui/src/resolve.rs`

**Why:** `resolve.rs` already ships the exact shape Phase 118 needs — an infallible, in-place `pub fn <resolve_*>(spec: &mut Spec, …)` that iterates `spec.elements.values_mut()` and mutates per-element fields. `resolve_expressions(&mut Spec)` is the sibling function for the expression surface.

#### Module header and imports (mirror `resolve.rs:1-14`)

Source — `ferro-json-ui/src/resolve.rs:1-14`:
```rust
//! Resolvers for v2 JSON-UI Spec element maps.
//!
//! Walks a `Spec`'s flat element map and resolves action handler names to
//! URLs, or populates per-field validation errors on form-like elements.
//!
//! Phase 115: flat iteration only. No tree descent — children are ID
//! strings, not nested structs. Action resolution is per-element.

use std::collections::HashMap;

use serde_json::Value;

use crate::action::Action;
use crate::spec::{Element, Spec};
```

**Delta for `expression.rs`:** Keep the module-doc voice neutral and phase-anchored. Drop `HashMap`, `Action`, `Element` imports (not needed). Keep `use serde_json::Value;` and `use crate::spec::Spec;`. Add nothing else — `crate::data::resolve_path` and `crate::data::resolve_path_string` are consumed via fully-qualified path.

#### Infallible public entry point (mirror `resolve.rs:31-41`)

Source — `ferro-json-ui/src/resolve.rs:31-41`:
```rust
/// Resolve every `Element.action` via the provided resolver closure.
///
/// Mutates in place. Silent on missing handlers — use
/// `resolve_actions_strict` if you want to collect missing names.
pub fn resolve_actions(spec: &mut Spec, resolver: impl Fn(&str) -> Option<String>) {
    for el in spec.elements.values_mut() {
        if let Some(action) = el.action.as_mut() {
            resolve_action(action, &resolver);
        }
    }
}
```

**Delta for `expression.rs`:**
- **Drop the closure parameter.** `resolve_actions` takes a resolver because handler→URL lookup is side-effectful and external. `resolve_expressions` reads from `spec.data` only and needs no injection. Signature is exactly `pub fn resolve_expressions(spec: &mut Spec)` per CONTEXT.md D-01 / D-09.
- **Clone `spec.data` before the mutable element loop** (Pitfall 1 in RESEARCH.md). The `values_mut()` iterator holds a mutable borrow of `spec`; `spec.data` must be cloned to a local binding first.
- **Walk every `el.props`, not `el.action`.** The walker target is `el.props` (per D-04). `el.action`, `el.children`, `el.visible`, and `Spec.data` itself are explicitly excluded.

Canonical form (from RESEARCH.md Pattern 4):
```rust
pub fn resolve_expressions(spec: &mut Spec) {
    let data = spec.data.clone();
    for el in spec.elements.values_mut() {
        resolve_value(&mut el.props, &data);
    }
}
```

#### Visibility — helpers stay private (mirror `resolve.rs:19, 82`)

Source — `ferro-json-ui/src/resolve.rs:19` and `:82`:
```rust
fn resolve_action(action: &mut Action, resolver: &impl Fn(&str) -> Option<String>) {
fn attach_errors(el: &mut Element, errors: &HashMap<String, Vec<String>>, all: bool) {
```

**Delta:** Private helpers in `expression.rs` follow the same convention — plain `fn`, no `pub`, no `pub(crate)`. Per CONTEXT.md D-11 the helper names are `resolve_value`, `resolve_data_expr`, `resolve_template_expr`, `is_data_expr`, `is_template_expr`, `substitute_template`. None are re-exported from `lib.rs` (Claude's Discretion point: "only export what `framework`/tests actually consume").

#### Constants convention

No direct analog for module-level string constants inside `resolve.rs`, but `spec.rs:30, 37` shows the project convention for `pub const`:

Source — `ferro-json-ui/src/spec.rs:30` and `:37`:
```rust
pub const SCHEMA_VERSION: &str = "ferro-json-ui/v2";

pub const MAX_NESTING_DEPTH: usize = 3;
```

**Delta:** Phase 118 constants stay module-private (D-11 lists `EXPR_DATA_KEY = "$data"` and `EXPR_TEMPLATE_KEY = "$template"` as internal). Use:
```rust
const EXPR_DATA_KEY: &str = "$data";
const EXPR_TEMPLATE_KEY: &str = "$template";
```
Private, not `pub const` — these are internal sentinel strings, not part of the public surface.

#### Path-helper consumption (mirror `visibility.rs:80-82`)

`ferro-json-ui/src/visibility.rs` is the second infallible resolver in this crate and already consumes `crate::data::resolve_path` exactly the way `expression.rs` will. This confirms the pattern.

Source — `ferro-json-ui/src/visibility.rs:80-82`:
```rust
fn evaluate_condition(c: &VisibilityCondition, data: &serde_json::Value) -> bool {
    use crate::data::resolve_path;
    let resolved = resolve_path(data, &c.path);
```

**Delta for `expression.rs`:** Either `use crate::data::{resolve_path, resolve_path_string};` at the module top, or call them via the fully-qualified path inline (both work; the `visibility.rs` style uses a `use` inside the function body — matching that style keeps the import surface minimal). Both helpers are `pub(crate)` already — no visibility change required (CONTEXT.md D-11 confirms). Verify:

Source — `ferro-json-ui/src/data.rs:19` and `:55`:
```rust
pub(crate) fn resolve_path<'a>(data: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() || path == "/" {
        return Some(data);
    }
    …
}

pub(crate) fn resolve_path_string(data: &Value, path: &str) -> Option<String> {
    let value = resolve_path(data, path)?;
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Null => None,
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).ok(),
    }
}
```

**Do NOT copy / re-implement:** neither function. Re-use verbatim.

#### Inline `#[cfg(test)] mod tests` layout (mirror `resolve.rs:107-225`)

Source — `ferro-json-ui/src/resolve.rs:107-142`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, HttpMethod};
    use crate::spec::{Element, Spec};

    fn action(handler: &str) -> Action {
        Action {
            handler: handler.to_string(),
            url: None,
            method: HttpMethod::Post,
            confirm: None,
            on_success: None,
            on_error: None,
            target: None,
        }
    }

    #[test]
    fn resolve_actions_populates_url_from_resolver() {
        let mut spec = Spec::builder()
            .element("btn", Element::new("Button").action(action("users.create")))
            .build()
            .unwrap();

        resolve_actions(&mut spec, |h| {
            if h == "users.create" {
                Some("/users".to_string())
            } else {
                None
            }
        });

        let el = spec.elements.get("btn").unwrap();
        assert_eq!(el.action.as_ref().unwrap().url.as_deref(), Some("/users"));
    }
    …
}
```

Secondary example — `ferro-json-ui/src/data.rs:66-75` (same inline layout, uses `serde_json::json!`):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn simple_key_resolution() {
        let data = json!({"name": "Alice"});
        assert_eq!(resolve_path(&data, "/name"), Some(&json!("Alice")));
    }
    …
}
```

**Delta for `expression.rs` tests:**
- Use `use super::*;` + `use crate::spec::{Element, Spec};` + `use serde_json::{json, Value};` at the top of the `tests` module. No `Action`/`HttpMethod` needed.
- Use `Spec::builder().element(…).data(json!({…})).build().unwrap()` to construct fixtures inline (same pattern as `resolve.rs` tests — see `resolve.rs:127-130`).
- Construct `Element` with `.prop(name, value)` where `value` is either a literal string OR a `serde_json::json!({"$data": "/path"})` object to exercise the resolver.
- Test coverage list is fixed by CONTEXT.md D-12 — mirror the checklist in RESEARCH.md "Test Coverage Checklist".

#### Spec / Element field shape (consumed, not modified)

Source — `ferro-json-ui/src/spec.rs:48-92`:
```rust
pub struct Spec {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub root: String,
    pub elements: HashMap<String, Element>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub data: Value,
}

pub struct Element {
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub props: Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<Visibility>,
}
```

**Delta:** Zero changes. `spec.elements: HashMap<String, Element>` is iterated via `values_mut()`. `el.props: Value` is the walker's `&mut Value` target. `spec.data: Value` is the resolution source (cloned once before the loop). All other fields (`title`, `layout`, `schema`, `root`, `el.type_name`, `el.children`, `el.action`, `el.visible`) are not touched.

#### Error-handling pattern

**Delta:** There is no error handling in `expression.rs`. Per CONTEXT.md D-06, D-09 the resolver is infallible, returns `()`, and never logs. Malformed expressions (non-string `$data` value, sibling keys, etc.) pass through as literal JSON. This mirrors `resolve_actions` (silent on missing handlers — see the doc comment at `resolve.rs:33-34`).

**Do NOT copy** the `_strict` variant pattern from `resolve_actions_strict` (`resolve.rs:46-64`). No `resolve_expressions_strict` exists or is planned — D-09 is explicit that the resolver has exactly one surface.

---

### `ferro-json-ui/src/lib.rs` (MODIFIED — module re-export index)

**Primary analog:** the existing `resolve` module + re-export pair already in this file.

#### Module declaration block (line 38)

Source — `ferro-json-ui/src/lib.rs:29-40`:
```rust
pub mod action;
pub mod catalog;
pub mod component;
pub mod config;
pub mod data;
pub mod layout;
pub mod plugin;
pub mod plugins;
pub mod render;
pub mod resolve;
pub mod spec;
pub mod visibility;
```

**Delta:** Add `pub mod expression;` between `pub mod data;` and `pub mod layout;` to preserve alphabetic ordering of the module block. (`expression` sorts between `data` and `layout`.)

#### Re-export pair (line 74)

Source — `ferro-json-ui/src/lib.rs:73-74`:
```rust
pub use render::{render_spec_to_html, render_spec_to_html_with_plugins, RenderResult};
pub use resolve::{resolve_actions, resolve_actions_strict, resolve_errors, resolve_errors_all};
```

**Delta:** Add a single new `pub use` line immediately above the `resolve::*` line (to keep alphabetic grouping `expression < resolve`):
```rust
pub use expression::resolve_expressions;
pub use resolve::{resolve_actions, resolve_actions_strict, resolve_errors, resolve_errors_all};
```

**Do NOT copy** the `{…}` brace grouping from `resolve::{…}` — `expression` re-exports exactly one symbol (`resolve_expressions`), so use `pub use expression::resolve_expressions;` without braces. Matches the single-symbol style on line 58 (`pub use config::JsonUiConfig;`).

**Note on the existing comment at line 59** — `"resolve_path and resolve_path_string are pub(crate) — internal render pipeline helpers"`. Phase 118 keeps this comment intact; both helpers remain `pub(crate)` per CONTEXT.md D-11.

---

### `framework/src/json_ui/mod.rs` (MODIFIED — pipeline wiring + tests)

**Primary analog:** the same file, existing `JsonUi::resolve` and `JsonUi::resolve_with_errors` methods.

#### Import block (line 26-29)

Source — `framework/src/json_ui/mod.rs:26-29`:
```rust
use ferro_json_ui::{
    render_layout, render_spec_to_html_with_plugins, resolve_actions, resolve_errors, JsonUiConfig,
    LayoutContext, Spec,
};
```

**Delta:** Add `resolve_expressions` to the import list, alphabetically between `resolve_actions` and `resolve_errors`:
```rust
use ferro_json_ui::{
    render_layout, render_spec_to_html_with_plugins, resolve_actions, resolve_expressions,
    resolve_errors, JsonUiConfig, LayoutContext, Spec,
};
```
(Existing list uses a trailing comma + multi-line block; keep that style.)

#### `JsonUi::resolve` — the one-line insertion (lines 38-43)

Source — `framework/src/json_ui/mod.rs:38-43`:
```rust
    /// Clone the spec and resolve all action handler names to URLs.
    fn resolve(spec: &Spec) -> Spec {
        let mut resolved = spec.clone();
        resolve_actions(&mut resolved, |handler| crate::routing::route(handler, &[]));
        resolved
    }
```

**Delta:** Insert `resolve_expressions(&mut resolved);` immediately after the `resolve_actions` call, preserving the pipeline order from CONTEXT.md D-08 (actions → expressions → [validate → render]):
```rust
    /// Clone the spec and resolve all action handler names to URLs,
    /// then resolve `$data` / `$template` expression nodes in element props.
    fn resolve(spec: &Spec) -> Spec {
        let mut resolved = spec.clone();
        resolve_actions(&mut resolved, |handler| crate::routing::route(handler, &[]));
        resolve_expressions(&mut resolved);
        resolved
    }
```

Update the doc comment to reflect the expanded responsibility (neutral voice; see project convention on repository voice).

#### `JsonUi::resolve_with_errors` — same one-line insertion (lines 149-155)

Source — `framework/src/json_ui/mod.rs:149-155`:
```rust
    /// Clone the spec, resolve actions, and populate validation errors on form fields.
    fn resolve_with_errors(spec: &Spec, errors: &HashMap<String, Vec<String>>) -> Spec {
        let mut resolved = spec.clone();
        resolve_actions(&mut resolved, |handler| crate::routing::route(handler, &[]));
        resolve_errors(&mut resolved, errors);
        resolved
    }
```

**Delta:** Insert `resolve_expressions(&mut resolved);` between `resolve_actions(…)` and `resolve_errors(…)`. Order is actions → expressions → errors, so that error attachment runs against the resolved props (CONTEXT.md D-12 integration-test item: `render_with_errors_resolves_expressions_then_applies_errors`):
```rust
    fn resolve_with_errors(spec: &Spec, errors: &HashMap<String, Vec<String>>) -> Spec {
        let mut resolved = spec.clone();
        resolve_actions(&mut resolved, |handler| crate::routing::route(handler, &[]));
        resolve_expressions(&mut resolved);
        resolve_errors(&mut resolved, errors);
        resolved
    }
```

**Do NOT copy** any signature change or `Result` return. Both methods stay `fn <name>(…) -> Spec` with identical arity. Single-line additions only.

**Pitfall 3 from RESEARCH.md** — both `resolve` and `resolve_with_errors` must get the new line. Missing either one skips expression resolution on the error-rendering paths (`render_with_errors`, `render_json_with_errors`, `render_validation_error`, `render_json_validation_error`).

#### Integration test block placement

Source — test-block structure in `framework/src/json_ui/mod.rs`:
- Line 236: `mod tests` start.
- Lines 236-524: "action/render" test section (`sample_spec`, `render_produces_valid_html`, `render_json_returns_json`, `render_resolves_action_urls`, etc.).
- Line 539: `// render_with_errors tests` section header.
- Line 705: `// Layout integration tests` section header.
- Line 832: `// Theme CSS injection tests` section header.
- Line 959: `// Plugin integration tests` section header.

**Delta:** Add a new section `// Expression resolution tests` immediately after the `render_with_errors` block (roughly after line 703, before the `Layout integration tests` header at line 705). Match the commented-section banner style used elsewhere in the file (8-dash rule + blank line). Cover the four integration cases from CONTEXT.md D-12:

1. `render_resolves_data_expression_before_html_emission` — `JsonUi::render` against a spec whose `el.props` contains `{"$data": "/greeting"}` and `spec.data = {"greeting": "Hello"}`; assert the rendered body contains `"Hello"` and does NOT contain `"$data"` or `"/greeting"` literally.
2. `render_json_returns_spec_with_no_expression_markers` — `JsonUi::render_json` against the same spec; assert the serialized `spec` has the resolved value, no `"$data"` key remains anywhere in output.
3. `render_with_config_honors_expression_resolution` — same spec through `render_with_config(&spec, &data, &JsonUiConfig::new())`; same assertion.
4. `render_with_errors_resolves_expressions_then_applies_errors` — form spec with a `{"$template": "Errors for {/field_label}"}` prop and a non-empty `errors` map; assert both the resolved template text and the error messages appear in the response (order: actions → expressions → errors, per the updated `resolve_with_errors`).

Reference fixture style — copy the `sample_spec()` helper convention from lines 256-267 and extend it for an `expression_spec()` helper that sets `.data(json!({…}))` and uses `.prop("<slot>", json!({"$data": "/…"}))`.

**Test-block conventions already in use in this file (follow verbatim):**
- `fn ok_response(result: Response) -> HttpResponse` (line 241) — extract the Ok variant.
- `fn response_body(response: HttpResponse) -> String` (line 248) — extract body string for substring assertions.
- `fn has_content_type(…)` (line 272) — check `Content-Type` header (re-use if asserting JSON shape on `render_json`).
- `let result = JsonUi::render(&spec, &data);` → `assert!(result.is_ok());` → `let body = response_body(ok_response(result));` → `assert!(body.contains("…"));` — the standard response-body assertion chain (see lines 284-301 for canonical example).

---

## Shared Patterns

### Infallible-resolver posture

**Source:** `ferro-json-ui/src/resolve.rs` (all four public functions), `ferro-json-ui/src/visibility.rs::Visibility::evaluate`.
**Apply to:** `expression.rs::resolve_expressions`.

Both existing analogs return `()` or a plain bool on malformed input — they never panic, never log, never emit diagnostics. CONTEXT.md D-06 and D-09 lock the same posture for `resolve_expressions`.

Excerpt — `ferro-json-ui/src/visibility.rs:55-59`:
```rust
/// Infallible: malformed conditions, missing paths, and type mismatches all
/// resolve to `false` (visibility hides the element) without panicking. This
/// is the contract Phase 116's renderer relies on per CONTEXT D-13.
```

The `expression.rs` module doc should include an analogous sentence: malformed expressions degrade to literal JSON, missing `$data` paths resolve to `Value::Null`, missing `$template` placeholders resolve to `""`.

### `pub(crate)` helper consumption across sibling modules

**Source:** `ferro-json-ui/src/data.rs::{resolve_path, resolve_path_string}` (both `pub(crate)`, consumed by `visibility.rs::evaluate_condition` via `use crate::data::resolve_path;`).
**Apply to:** `expression.rs` calls `crate::data::resolve_path` and `crate::data::resolve_path_string`.

No visibility change required. Do NOT bump either to `pub`. The `lib.rs:59` comment (`// resolve_path and resolve_path_string are pub(crate) — internal render pipeline helpers`) is the load-bearing contract — keep it.

### Inline `#[cfg(test)] mod tests` convention

**Source:** `resolve.rs:107-225`, `data.rs:66-204`, `visibility.rs:139-415`.
**Apply to:** `expression.rs` (new inline test block), `framework/src/json_ui/mod.rs` (new tests in the existing test block).

Pattern is universal across the crate. No sibling test file; all tests live in the same `.rs` under `#[cfg(test)] mod tests { use super::*; … }`.

### Single-line pipeline additions in `JsonUi::resolve*`

**Source:** `framework/src/json_ui/mod.rs::JsonUi::resolve` (lines 38-43) and `JsonUi::resolve_with_errors` (lines 149-155) — each call in the pipeline (`resolve_actions`, `resolve_errors`) is exactly one statement.
**Apply to:** the new `resolve_expressions(&mut resolved);` call in both methods.

No helper method extraction. No struct-level changes. No new method. Just one line per existing method.

### `Cargo.toml` invariant

**Source:** `ferro-json-ui/Cargo.toml` — current dependencies are `serde`, `serde_json`, `schemars`, `thiserror`, `jsonschema`, plus optional `ferro-projections` / `ferro-theme` behind the `projections` feature.
**Apply to:** `ferro-json-ui/Cargo.toml` stays byte-identical. No `regex`, no `winnow`, no new deps. CONTEXT.md D-11 locks this; the hand-rolled template scanner in RESEARCH.md Pattern 3 is the implementation consequence.

---

## No Analog Found

None. Every target file has a direct or close analog in the same crate (`resolve.rs`, `visibility.rs`, `data.rs`) or in the same framework file being edited. Phase 118 is purely additive and reuses the shape of existing infrastructure.

---

## "Do NOT Copy" Notes

- **`resolve_actions` signature includes a closure parameter (`impl Fn(&str) -> Option<String>`) — `resolve_expressions` does NOT.** The resolver reads `spec.data` only; no injection needed. Signature is `pub fn resolve_expressions(spec: &mut Spec)` with no additional parameters.
- **`resolve_actions_strict` returns `Result<(), Vec<String>>` — `resolve_expressions` has NO strict variant.** D-09 locks a single, infallible public surface. Do not duplicate the `_strict` pattern.
- **`resolve_errors` accepts a `&HashMap<String, Vec<String>>` payload — `resolve_expressions` takes only `&mut Spec`.** No parallel between the two; the error-attachment pattern is not a template here.
- **`visibility.rs` uses `schemars::JsonSchema` derives on its types.** `expression.rs` defines no public types — no `JsonSchema`, no `Serialize`/`Deserialize` derives, no `pub struct`. Module constants stay private (`const`, not `pub const`).
- **`data.rs::resolve_path` and `resolve_path_string` stay `pub(crate)`.** Do NOT promote to `pub`. The `lib.rs:59` comment is binding.
- **Do not add a `pub use expression::{resolve_expressions, …}` block with multiple symbols.** Only `resolve_expressions` is re-exported. Helper functions (`substitute_template`, `is_data_expr`, etc.) stay module-private per CONTEXT.md Claude's Discretion ("only export what `framework`/tests actually consume").
- **Do not skip `JsonUi::resolve_with_errors`.** Pitfall 3 in RESEARCH.md: both `resolve` and `resolve_with_errors` need the `resolve_expressions(&mut resolved);` line, or the `_with_errors` family of render methods silently skips expression resolution.
- **Do not walk `Spec.data`, `Element.children`, `Element.action`, or `Element.visible`.** CONTEXT.md D-04 locks this. The walker only recurses into `el.props` `Value::Object` and `Value::Array` nodes. Spec metadata and structural fields stay literal.
- **Do not recurse into resolved `$data` output.** CONTEXT.md D-07 / RESEARCH.md Pitfall 2: after `*val = resolved_value`, return immediately. Re-walking the resolved value would re-evaluate inner `$data` markers, opening an inner-platform-effect surface.

---

## Metadata

**Analog search scope:** `ferro-json-ui/src/*.rs`, `framework/src/json_ui/mod.rs`, `ferro-json-ui/Cargo.toml`.
**Files scanned:** 6 (resolve.rs, data.rs, visibility.rs, spec.rs, lib.rs, framework/src/json_ui/mod.rs) + Cargo.toml for dependency baseline.
**Pattern extraction date:** 2026-04-19.
