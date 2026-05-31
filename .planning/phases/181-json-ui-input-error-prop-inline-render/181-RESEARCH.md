# Phase 181: json-ui-input-error-prop-inline-render — Research

**Researched:** 2026-05-31
**Domain:** ferro-json-ui resolution pipeline / form-control error rendering
**Confidence:** HIGH — all findings verified by direct source reading

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Re-verify diagnosis before coding. The renderer at form.rs:309-315 already emits the error `<p>`. Bug is upstream in the resolution pipeline.
- **D-02:** Three pipeline suspects must all be investigated: (1) `resolve_expressions` runtime-data scoping, (2) `attach_errors` field-name/shape mismatch, (3) flash round-trip divergence.
- **D-03:** Fix at the pipeline layer, not the renderer. No surface-level shims that calcify dual pathways.
- **D-04:** Both authoring paths must work: `JsonUi::render_validation_error` (blessed) and manual `obj.insert("<field>_error") + .prop("error", $data binding)` (escape hatch).
- **D-05:** Cross-field `has_validation_errors()` / `toast_validation` symptom is in scope.
- **D-06:** Error-state class parity: Checkbox, CheckboxList, Switch, Input-file must reach parity with Input/Select (border-destructive swap + destructive focus-ring + ARIA). Exact class chains locked in UI-SPEC.md.
- **D-07:** Integration tests at `JsonUi` pipeline level (not renderer-isolated). Two required: `$data` binding path + `render_validation_error` path. Both must fail before fix, pass after.
- **D-08:** Clean break — no backward-compat shim for `errors: Vec<String>` → `error: Option<String>`. Cross-repo gestiscilo audit required.
- **D-09:** Docs page covering all four authoring patterns (blessed path, escape hatch, flash round-trip, cross-field summary).

### Claude's Discretion

- Exact class-chain composition for new error-state styling on Checkbox / CheckboxList / Switch / Input-file (UI-SPEC.md locks the strings; discretion is in how to express them in Rust).
- Test placement and exact assertion text for D-07.
- Page filename and section ordering for the docs update (D-09).
- Whether `attach_errors` becomes `error: String` (first message wins) or some other unified shape. Planner decides after diagnosis.

### Deferred Ideas (OUT OF SCOPE)

- Multi-error per field display (`error: Option<String>` carries only first message; revisit later).
- Live (client-side) validation feedback (banned by PROJECT.md expression language cap).
- Toast component structural rework.
- `ferro-projection`-level error projection (v13.0 direction).
</user_constraints>

---

## Summary

Phase 181 closes a visible display gap: error messages bound via `{"$data": "/field_error"}` or via `JsonUi::render_validation_error` are never rendered as `<p>` elements below form controls, even though the renderer is already wired to do so when `props.error` is `Some(String)`.

Source reading has confirmed three distinct failure modes — all verified by direct code inspection. Two of the three are confirmed actual root causes. The third (flash divergence) is a read-path analysis finding that likely resolves to a non-bug in the ferro session layer but may surface as a consumer-side wrapper issue in gestiscilo.

**Root cause 1 (HIGH confidence — confirmed):** `resolve_expressions` reads `spec.data` only. When a handler passes error strings in the runtime `data` argument to `JsonUi::render(spec, data)`, those strings are never visible to `$data` expression resolution, so `{"$data": "/field_error"}` resolves to `Value::Null` and `props.error` remains `None`.

**Root cause 2 (HIGH confidence — confirmed):** `attach_errors` in `ferro-json-ui/src/resolve.rs:191-195` writes `props_obj.insert("errors", Value::Array(...))` (plural, array). All form-control prop structs in `ferro-json-ui/src/component.rs` declare `pub error: Option<String>` (singular). `serde_json::from_value::<InputProps>(el.props.clone())` succeeds and silently drops the `errors` array because serde ignores unknown fields by default. `props.error` remains `None`.

**Root cause 3 (MEDIUM confidence — session layer is clean, consumer-side may differ):** `req.has_validation_errors()` and `req.validation_error("field")` at `framework/src/http/request.rs:273-295` both read the same session key `_flash.old._validation_errors` via the same `session().and_then(|s| s.get(...))` path. Session aging (`age_flash_data`) runs at request boundary; within a single handler both reads hit identical state. They cannot diverge in the ferro layer. The reported divergence in gestiscilo Phase 175 is likely caused by a consumer-side helper or middleware ordering edge case, not by a ferro bug. Researcher verdict: no ferro fix needed for suspect (3) unless gestiscilo audit surfaces one.

**Primary recommendation:** Fix suspect (1) by merging the runtime `data` argument into `spec.data` before `resolve_expressions` runs, using the existing `Spec::merge_data` API. Fix suspect (2) by changing `attach_errors` to insert `"error"` (singular string, first message) instead of `"errors"` (plural array). Apply D-06 class parity to Checkbox/CheckboxList/Switch/Input-file as specified in UI-SPEC.md. Add two pipeline-level integration tests as required by D-07.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| $data expression resolution | ferro-json-ui (pipeline) | framework/json_ui (integration layer) | `resolve_expressions` lives in ferro-json-ui; `JsonUi::resolve` in framework calls it |
| Error attachment (blessed path) | ferro-json-ui (pipeline) | framework/json_ui (integration layer) | `attach_errors` / `resolve_errors` in ferro-json-ui; `JsonUi::resolve_with_errors` calls it |
| Error attachment (escape hatch) | framework/json_ui (integration layer) | Consumer handler | Handler merges error strings into runtime data; pipeline resolves $data |
| Form-control error HTML emission | ferro-json-ui (renderer) | — | `render_input` / `render_select` / etc. in form.rs |
| Error-state class parity (D-06) | ferro-json-ui (renderer) | — | form.rs class strings |
| Flash session read | framework (session layer) | Consumer handler | `req.validation_error()` / `req.has_validation_errors()` in request.rs |
| Integration tests (D-07) | framework/json_ui tests | ferro-json-ui tests | Pipeline-level tests must exercise full `JsonUi::render*` path |

---

## Bug Reproduction — All Three D-02 Suspects

### Suspect 1: `resolve_expressions` runtime-data scoping (CONFIRMED ROOT CAUSE)

**Location:** `ferro-json-ui/src/expression.rs:35-40`

**Code:**
```rust
pub fn resolve_expressions(spec: &mut Spec) {
    let data = spec.data.clone();   // reads spec.data ONLY
    for el in spec.elements.values_mut() {
        resolve_value(&mut el.props, &data);
    }
}
```

**The gap:** When a handler builds an element with `.prop("error", json!({"$data": "/overage_threshold_error"}))` and passes `json!({"overage_threshold_error": "Per il sovrapprezzo..."})` as the second argument to `JsonUi::render(spec, data)`, the pipeline is:

```
JsonUi::render(spec, data)
  -> Self::resolve(spec)           // spec is cloned; data is NOT passed in
       -> resolve_expressions(&mut resolved)  // reads only resolved.data (empty!)
```

The runtime `data` argument never reaches `resolve_expressions`. The path `/overage_threshold_error` does not exist in `spec.data`, so `resolve_path` returns `None` and `*val = Value::Null`. The Input props deserialize with `error: None`.

**Contrast with render_file:** `framework/src/json_ui/mod.rs:202` explicitly calls `(*arc_spec).clone().merge_data(handler_data)` before `Self::resolve()`, which merges handler data into `spec.data` first. That path works. `render(spec, data)` does not.

**Additional confirmation from the gestiscilo discovery:** The report notes `value="2"` is correctly restored via `data_path` binding — because `render_input` at form.rs:157 calls `resolve_path_string(data, dp)` where `data` is the runtime argument passed directly to the renderer. The renderer receives the correct data. Only `resolve_expressions` is scoped to `spec.data`.

**Failing test to write (D-07 test 1):**
```rust
// Must FAIL before fix, PASS after fix
#[test]
fn pipeline_data_binding_error_prop_renders_p_tag() {
    let spec = Spec::builder()
        .element(
            "email-input",
            Element::new("Input")
                .prop("field", "email")
                .prop("label", "Email")
                .prop("error", serde_json::json!({"$data": "/email_error"})),
        )
        .build()
        .expect("spec is valid");

    let data = serde_json::json!({"email_error": "must be valid"});
    let result = JsonUi::render(&spec, &data);
    let body = html_body(ok_response(result));

    assert!(
        body.contains(r#"<p id="err-email" class="text-sm text-destructive">must be valid</p>"#),
        "error paragraph must appear below the input; got: {body}"
    );
    assert!(
        !body.contains("<!-- ferro-json-ui:"),
        "no diagnostic comments in happy path; got: {body}"
    );
}
```

---

### Suspect 2: `attach_errors` field-name/shape mismatch (CONFIRMED ROOT CAUSE)

**Location:** `ferro-json-ui/src/resolve.rs:178-201`

**Code:**
```rust
fn attach_errors(el: &mut Element, errors: &HashMap<String, Vec<String>>, all: bool) {
    // ...
    if let Some(msgs) = errors.get(&k) {
        props_obj.insert(
            "errors".to_string(),             // ← plural "errors"
            Value::Array(msgs.iter().cloned().map(Value::String).collect()),
        );
    }
}
```

**The gap:** `InputProps` (and all form-control props in component.rs) declares:
```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub error: Option<String>,   // ← singular "error"
```

When `JsonUi::render_validation_error(&spec, &data, &ve)` is called:
1. `resolve_errors(&mut resolved, errors)` calls `attach_errors`, which inserts `"errors": ["must be valid"]` into `el.props`.
2. `render_input(el, spec, data, depth)` calls `serde_json::from_value::<InputProps>(el.props.clone())`.
3. Serde sees `"errors"` (unknown field) and silently ignores it. `props.error` remains `None`.
4. `has_error = false`. No `<p>`, no border-destructive swap.

**Confirmed by existing test in framework/src/json_ui/mod.rs:816-840:** The test `render_with_errors_populates_form_fields` uses `response_body(result)` which calls `format!("{body_bytes:?}")`. This captures the Debug-formatted byte representation of the entire page, which includes the `data-view` attribute containing the serialized resolved spec. The spec has `errors: ["Name is required"]` in the JSON, so the string appears in the serialized page. The test passes without the error ever being rendered as a `<p>` tag. The test is a false positive — it asserts string presence in the page, not presence as a rendered DOM element.

**Failing test to write (D-07 test 2):**
```rust
// Must FAIL before fix, PASS after fix
#[test]
fn pipeline_render_validation_error_renders_p_tag() {
    let spec = Spec::builder()
        .element(
            "email-input",
            Element::new("Input")
                .prop("field", "email")
                .prop("label", "Email"),
        )
        .build()
        .expect("spec is valid");

    let mut ve = crate::validation::ValidationError::new();
    ve.add("email", "must be valid");

    let data = serde_json::json!({});
    let result = JsonUi::render_validation_error(&spec, &data, &ve);
    let body = html_body(ok_response(result));  // use html_body, NOT response_body

    assert!(
        body.contains(r#"<p id="err-email" class="text-sm text-destructive">must be valid</p>"#),
        "error paragraph must appear below the input; got: {body}"
    );
    assert!(
        body.contains(r#"aria-invalid="true""#),
        "aria-invalid must be set on the input; got: {body}"
    );
    assert!(
        !body.contains("<!-- ferro-json-ui:"),
        "no diagnostic comments in happy path; got: {body}"
    );
}
```

Note: `html_body` (not `response_body`) is the correct helper here. `response_body` captures the Debug repr which includes the data-view JSON — the test would be a false positive again. `html_body` calls `response.body().to_string()` which is the raw HTML. This distinction is critical for both D-07 test cases.

---

### Suspect 3: Flash round-trip / session middleware (NON-BUG in ferro layer)

**Location:** `framework/src/http/request.rs:273-295`

**Analysis:**
```rust
pub fn validation_error(&self, field: &str) -> Option<String> {
    let errors: Option<HashMap<String, Vec<String>>> =
        crate::session::session().and_then(|s| {
            s.get::<HashMap<String, Vec<String>>>("_flash.old._validation_errors")
        });
    errors.and_then(|map| map.get(field).and_then(|v| v.first()).cloned())
}

pub fn has_validation_errors(&self) -> bool {
    crate::session::session()
        .and_then(|s| s.get::<HashMap<String, Vec<String>>>("_flash.old._validation_errors"))
        .map(|m| !m.is_empty())
        .unwrap_or(false)
}
```

Both methods:
- call `crate::session::session()` which returns the per-request session from thread-local storage
- call `s.get::<HashMap<...>>("_flash.old._validation_errors")` which calls `serde_json::from_value(v.clone()).ok()` on the stored value

These are pure reads. `SessionData::get` does not modify state. `SessionData::age_flash_data` (which moves `_flash.new.*` → `_flash.old.*`) runs at request boundaries (session middleware), not during handler execution. Within a single GET handler both reads will see identical `_flash.old._validation_errors` state.

**Verdict:** `has_validation_errors()` and `validation_error()` cannot disagree within a single handler in the ferro session layer. If gestiscilo Phase 175 UAT observed them disagreeing, the cause is:
- A consumer-side helper or middleware that calls `get_flash` (which clears the key) between the two reads, OR
- `SessionData::get_flash` being used somewhere (note: `get_flash` at store.rs:91-98 calls `self.forget(&flash_key)` after read)
- A consumer wrapper that reads and transforms the errors before passing to the handler

**Action for planner:** Include a Wave 0 task to audit gestiscilo's handler and any middleware for `get_flash("_validation_errors")` calls. If none found, close suspect 3 as a non-issue. Do not modify the ferro session layer.

---

## Diagnosis Summary

| # | Suspect | Status | Confidence | Fix Required |
|---|---------|--------|------------|--------------|
| 1 | `resolve_expressions` reads `spec.data` only, not runtime `data` arg | CONFIRMED ROOT CAUSE | HIGH | Merge runtime `data` into `spec.data` before `resolve_expressions` |
| 2 | `attach_errors` inserts `errors` (plural array) but props declare `error` (singular string) | CONFIRMED ROOT CAUSE | HIGH | Change `attach_errors` to insert `"error": first_message` |
| 3 | Flash session divergence between `has_validation_errors()` and `validation_error()` | NON-BUG in ferro; may be consumer side | MEDIUM | Audit gestiscilo; likely no ferro change |

Both suspect 1 and suspect 2 must be fixed for D-04 to hold (both paths must work end-to-end).

---

## Proposed Minimal Fixes

### Fix A: Merge runtime data before resolve_expressions (Suspect 1)

**Location:** `framework/src/json_ui/mod.rs` — `JsonUi::resolve(spec)` and `JsonUi::resolve_with_errors(spec, errors)`

**Option A1 (recommended): Merge in `render`/`render_with_config` before calling `resolve`**

Change the call site in `render_with_config`:
```rust
pub fn render_with_config(spec: &Spec, data: &serde_json::Value, config: &JsonUiConfig) -> Response {
    let spec_with_data = spec.clone().merge_data(data.clone());
    let resolved = Self::resolve(&spec_with_data);
    Self::build_response(&resolved, data, config)
}
```

And `render_with_errors_config`:
```rust
fn render_with_errors_config(spec: &Spec, data: &serde_json::Value, errors: &HashMap<String, Vec<String>>, config: &JsonUiConfig) -> Response {
    let spec_with_data = spec.clone().merge_data(data.clone());
    let resolved = Self::resolve_with_errors(&spec_with_data, errors);
    Self::build_response(&resolved, data, config)
}
```

`Spec::merge_data` (spec.rs:256-272) is consuming and already handles Null/non-Object gracefully (silent no-op). This preserves `spec.data` for elements that use static embedded data, while also making runtime data visible to `resolve_expressions`. Handler data wins on key collision (existing merge_data semantics).

**Option A2 (alternative): Pass `data` into `resolve_expressions` as a second arg**

Change `resolve_expressions(spec: &mut Spec)` to `resolve_expressions(spec: &mut Spec, extra_data: Option<&Value>)` and merge inside the function. More surgical but changes the ferro-json-ui public API signature.

**Recommendation: Option A1.** It is the minimal change, uses an existing API that already exists and is tested, and aligns exactly with how `render_file` already handles this (line 202: `(*arc_spec).clone().merge_data(handler_data)`). The precedent is already set.

**Risk of A1:** If any caller currently relies on `spec.data` being separate from runtime `data` after the call (i.e., reading `spec.data` post-render to verify no mutation), this merge could surprise them. However, `resolve` already clones the spec (`let mut resolved = spec.clone()`), so the original `spec` is never mutated. The merge happens on the clone. No regression risk to callers that retain the original spec.

---

### Fix B: Unify `attach_errors` to write `error: String` (Suspect 2)

**Location:** `ferro-json-ui/src/resolve.rs:178-201`

Change `attach_errors` from:
```rust
props_obj.insert(
    "errors".to_string(),
    Value::Array(msgs.iter().cloned().map(Value::String).collect()),
);
```

To:
```rust
if let Some(first) = msgs.first() {
    props_obj.insert(
        "error".to_string(),
        Value::String(first.clone()),
    );
}
```

This writes `"error": "first message"` which matches `InputProps.error: Option<String>` exactly. Serde will deserialize it correctly. The `<p>` will render.

**Impact on existing tests in resolve.rs (lines 785-836):** The tests at 785-833 assert `el.props.get("errors")` equals `json!(["required"])`. These tests must be updated to assert `el.props.get("error")` equals `json!("required")` after the fix. This is the D-08 audit step for the ferro-json-ui layer.

**Cross-repo audit for gestiscilo:** Search for any gestiscilo code that reads `props.errors` (plural) from a manually constructed spec or from a post-render spec JSON. `rg '"errors"' ../gestiscilo-it/app/src/` and `rg '\.errors' ../gestiscilo-it/app/src/`. Likely none, since `resolve_errors` has never worked end-to-end (the rendered HTML never showed errors), so no consumer could have built a working feature on top of it. Still required per D-08.

---

## Pipeline Fix Integration Point

The fix belongs in `framework/src/json_ui/mod.rs`. The call chain is:

```
JsonUi::render(spec, data)
  -> render_with_config(spec, data, config)
       -> [NEW] spec_with_data = spec.clone().merge_data(data.clone())
       -> Self::resolve(&spec_with_data)    // expressions now see runtime data
            -> resolve_expressions(&mut resolved)  // finds /email_error in spec.data
       -> build_response(&resolved, data, config)

JsonUi::render_validation_error(spec, data, ve)
  -> render_with_errors(spec, data, errors)
       -> render_with_errors_config(spec, data, errors, config)
            -> [NEW] spec_with_data = spec.clone().merge_data(data.clone())
            -> Self::resolve_with_errors(&spec_with_data, errors)
                 -> resolve_expressions  (finds keys in spec.data)
                 -> resolve_errors       (now writes "error": "msg")  [after Fix B]
            -> build_response(&resolved, data, config)
```

---

## Validation Architecture (D-07)

`workflow.nyquist_validation` is not set to `false` in `.planning/config.json` (key absent — treat as enabled).

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` |
| Config file | none (workspace Cargo.toml) |
| Quick run command | `cargo test -p ferro-rs pipeline_ --test-threads=1` |
| Full suite command | `cargo test --all-features --test-threads=1` |

### Phase Requirements → Test Map

| Req | Behavior | Test Type | Automated Command | Existing? |
|-----|----------|-----------|-------------------|-----------|
| D-07a | `$data` binding path: runtime data → error `<p>` | integration | `cargo test -p ferro-rs pipeline_data_binding_error_prop_renders_p_tag` | No — Wave 0 gap |
| D-07b | Blessed path: `render_validation_error` → error `<p>` | integration | `cargo test -p ferro-rs pipeline_render_validation_error_renders_p_tag` | No — Wave 0 gap |
| D-06 | Checkbox/Switch/CheckboxList/Input-file border-destructive + ARIA when has_error | unit | `cargo test -p ferro-json-ui checkbox_error_emits_border_destructive` (etc.) | No — Wave 0 gap |
| D-02/2 | `attach_errors` fix: resolve.rs tests updated | unit | `cargo test -p ferro-json-ui resolve_errors_matches_by_field_prop` | Yes — update assertion |
| D-07 false-positive | Existing test uses wrong `response_body` helper | integration | `cargo test -p ferro-rs render_with_errors_populates_form_fields` | Exists — must update to `html_body` |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-rs && cargo test -p ferro-json-ui`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `framework/src/json_ui/mod.rs` test module — `pipeline_data_binding_error_prop_renders_p_tag` (D-07a)
- [ ] `framework/src/json_ui/mod.rs` test module — `pipeline_render_validation_error_renders_p_tag` (D-07b)
- [ ] `ferro-json-ui/src/render/form.rs` — `checkbox_error_emits_border_destructive` and sibling tests for Switch, CheckboxList, Input-file (D-06)
- [ ] Update existing `render_with_errors_populates_form_fields` to use `html_body` instead of `response_body` (converts the false positive into a real regression test)

**Critical note on test helper choice:** The `response_body(result)` helper at mod.rs:340 calls `format!("{body_bytes:?}")` which captures the Debug-formatted byte string. This includes the `data-view` attribute which embeds the full serialized spec JSON — strings like `"Name is required"` appear there even if never rendered as HTML. Pipeline-level integration tests for error rendering MUST use `html_body(result)` (calls `response.body().to_string()`) and assert on `<p id="err-` tag presence. Using `response_body` produces false positives that mask the bug.

---

## Pattern Map

### Reusable Class-Chain Patterns (from form.rs:174-184)

**Canonical Input/Select error state (do not modify, reference only):**
```rust
// Border swap
let border_class = if has_error { "border-destructive" } else { "border-border" };

// Focus ring swap
let focus_ring_class = if has_error {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-destructive focus-visible:ring-offset-2"
} else {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
};
```

**Canonical ARIA pairing (from form.rs:277-282):**
```rust
if has_error {
    html.push_str(&format!(
        " aria-invalid=\"true\" aria-describedby=\"err-{}\"",
        html_escape(&props.field)
    ));
}
```

**Canonical error `<p>` (from form.rs:309-315):**
```rust
if let Some(ref error) = props.error {
    html.push_str(&format!(
        "<p id=\"err-{}\" class=\"text-sm text-destructive\">{}</p>",
        html_escape(&props.field),
        html_escape(error)
    ));
}
```

### D-06 Class-Chain Deltas (from UI-SPEC.md — exact strings, do not deviate)

**Checkbox:** swap `border-border` → `border-destructive` on `<input type="checkbox">` + swap `ring-primary` → `ring-destructive` + add ARIA + add `id="err-{field}"` to the `<p>` (already uses `ml-6`).

**CheckboxList:** add `aria-invalid="true" aria-describedby="err-{field}"` on `<fieldset>` + swap `border-border` → `border-destructive` on each individual `<input>` + add `id="err-{field}"` to error `<p>`.

**Switch:** swap `peer-focus:ring-primary/30` → `peer-focus:ring-destructive/30` on pill `<div>` + add `aria-invalid="true" aria-describedby="err-{field}"` on hidden `<input>` + add `id="err-{field}"` to error `<p>`.

**Input (file):** add `ring-1 ring-destructive` to `<input type="file">` when `has_error` + add `aria-invalid="true" aria-describedby="err-{field}"` (existing error `<p>` at line 309-315 already has the correct `id` since it is shared with all non-hidden variants).

### `html_escape` Usage

`html_escape` from `ferro-json-ui/src/render/mod.rs` is re-exported via `super::html_escape` in form.rs. Every string interpolated into HTML attribute or text content must pass through it. Field names for `id` and `aria-describedby` must be escaped. Error message text must be escaped.

### Diagnostic Comment Pattern

On `serde_json::from_value` decode failure, renderers return:
```
<!-- ferro-json-ui: failed to decode {Component} props: {err} -->
```
Post-fix integration tests (D-07) MUST assert this comment does NOT appear in happy-path renders.

---

## Risk Surface

### Call Sites Affected by Fix A (runtime data merge)

**`render_with_config` (framework/src/json_ui/mod.rs:79-86):** Called by `render()`. Adding `merge_data` here affects all `JsonUi::render(spec, data)` call sites. Risk: if any consumer intentionally puts data in `spec.data` AND passes different data in the runtime arg expecting them to remain separate in the rendered output. Assessment: extremely unlikely in practice since the docs and API have never advertised this separation. `merge_data` uses handler data as the winning value on collision (existing semantics), which is the desired behavior for form pre-fill overrides.

**`render_with_errors_config` (line 261-268):** Same change needed here.

**`render_json` (line 213-221):** Not immediately affected by the error rendering fix (render_json returns spec+data JSON, not rendered HTML). However, to avoid a future footgun, consider applying the same merge so that `$data` expressions also resolve correctly in the JSON output. Mark as "Claude's discretion" for planner.

**`render_json_with_errors` (line 275-287):** Same consideration as `render_json`.

### Call Sites Affected by Fix B (`attach_errors` field name change)

**`ferro-json-ui/src/resolve.rs:178-201`:** The `attach_errors` function itself.

**Tests in resolve.rs:785-836:** Three tests assert `props.get("errors")` equals `json!(["required"])`. These must be updated to assert `props.get("error")` equals `json!("required")`. The planner must include this as an explicit task (not just "the fix" — the old tests will now fail and must be updated or replaced).

**`render_with_errors_populates_form_fields` in mod.rs:816-840:** Currently a false positive. After Fix B + updating to use `html_body`, this test will fail until Fix A is also applied (the error string appears in the data-view JSON but not as a rendered `<p>`). The planner must sequence: apply both fixes together, then run the corrected test.

**gestiscilo audit:** `rg '"errors"\|\.errors' ../gestiscilo-it/app/src/` — search for any consumer reading the plural `errors` field from props. Based on the history (the feature has never worked), the probability of consumers depending on the broken shape is near zero, but D-08 mandates the audit.

### False Positive Test Risk

The existing `render_with_errors_populates_form_fields` test (and similarly `render_validation_error_accepts_framework_type`) uses `response_body` which captures the debug-formatted bytes including the `data-view` attribute. These tests will continue to pass after Fix B alone because the `errors` → `error` rename means `props.errors` disappears from the data-view JSON, but the error text still appears in `data-view` as part of `props.error`. However, after Fix A and Fix B together, `props.error` is set and the string is in both the rendered HTML and the data-view JSON. These tests should be upgraded to `html_body` assertions on `<p id="err-` to become meaningful regression guards. The planner should include this upgrade explicitly.

---

## Common Pitfalls

### Pitfall 1: Using `response_body` instead of `html_body` in integration tests

**What goes wrong:** `response_body` captures `format!("{:?}", body_bytes)` — the Debug repr of the entire page bytes, including the serialized spec in `data-view`. Error strings embedded in the `data-view` JSON attribute make the test pass even when no `<p>` is rendered.

**Why it happens:** Two test helpers exist with similar names; the debug-repr helper was the original one; the raw-string helper was added later. The existing `render_with_errors_populates_form_fields` test uses the wrong one.

**How to avoid:** Any test asserting that an error `<p>` tag is rendered must use `html_body`. Any test asserting on URL resolution or JSON payload can use `response_body`.

**Warning signs:** Test contains `response_body(...)` and `assert!(body.contains("error message"))` without asserting on the `<p id="err-` tag structure.

### Pitfall 2: Applying Fix B without Fix A

**What goes wrong:** After Fix B, `attach_errors` correctly writes `"error": "msg"` and `InputProps.error` deserializes correctly. But if Fix A has not been applied, the `$data` binding path (`req.validation_error("field")` → `obj.insert("field_error")` → `{"$data": "/field_error"}`) still fails silently. `render_validation_error` works, but the escape hatch does not.

**How to avoid:** Both fixes must ship in the same release. The D-07 tests cover both paths; the phase gate is that BOTH tests pass.

### Pitfall 3: D-06 Checkbox border swap on the wrong element

**What goes wrong:** `render_checkbox` builds a flex container `<div class="space-y-1">` > `<div class="flex items-center gap-2">` > `<input type="checkbox" ...>`. The border-destructive swap must be applied to the `<input>` class, not the outer `<div>`. Applying it to the outer div is a subtle miss that looks identical without close inspection.

**How to avoid:** The UI-SPEC.md class chain specifies "swap `border-border` → `border-destructive` on the checkbox `<input>`." The test should assert the `<input>` tag itself contains `border-destructive`.

### Pitfall 4: Missing `id` attribute on existing Checkbox / Switch error `<p>`

**What goes wrong:** The existing Checkbox error `<p>` at form.rs:485-490 emits `class="ml-6 text-sm text-destructive"` without an `id`. The Switch error `<p>` at form.rs:705-710 similarly lacks `id`. Without `id="err-{field}"`, `aria-describedby` on the input has no target — screen readers find a dangling reference and may ignore it.

**How to avoid:** The D-06 fix must add `id="err-{field}"` to these existing `<p>` elements, not just add new elements.

---

## Code Examples

### Verified Pattern: `merge_data` API (spec.rs:256-272)

```rust
// Source: ferro-json-ui/src/spec.rs:256
// Consuming builder: merges handler_data keys into spec.data; handler keys win.
pub fn merge_data(mut self, handler_data: serde_json::Value) -> Self {
    if let Some(obj) = handler_data.as_object() {
        if self.data.is_null() { self.data = Value::Object(Map::new()); }
        if let Some(data_map) = self.data.as_object_mut() {
            for (k, v) in obj { data_map.insert(k.clone(), v.clone()); }
        }
    }
    self
}
```

### Verified Pattern: How `render_file` already solves this (mod.rs:194-206)

```rust
// Source: framework/src/json_ui/mod.rs:202
// render_file merges handler_data into spec.data BEFORE resolve.
let spec = (*arc_spec).clone().merge_data(handler_data);
let data = spec.data.clone();
let resolved = Self::resolve(&spec);
Self::build_response(&resolved, &data, config)
```

The fix for `render` should mirror this pattern.

### Verified Pattern: `attach_errors` location and exact change needed

```rust
// BEFORE (resolve.rs:191-195):
props_obj.insert(
    "errors".to_string(),
    Value::Array(msgs.iter().cloned().map(Value::String).collect()),
);

// AFTER:
if let Some(first) = msgs.first() {
    props_obj.insert("error".to_string(), Value::String(first.clone()));
}
```

---

## State of the Art

| Old Behavior | Correct Behavior After Fix | Impact |
|--------------|---------------------------|--------|
| `resolve_expressions` reads only `spec.data` | `JsonUi::render` merges runtime `data` into `spec.data` clone before resolution | `$data` binding error path works |
| `attach_errors` writes `errors: Vec<String>` | `attach_errors` writes `error: String` (first message) | Blessed path works |
| `render_with_errors_populates_form_fields` is a false positive | Test upgraded to `html_body` + `<p id="err-` assertion | Regression guard is real |
| Checkbox / Switch have no border-destructive swap | D-06 class parity applied | Visual + ARIA error treatment consistent across all form controls |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `serde_json::from_value::<InputProps>` silently ignores unknown fields by default (D-02 suspect 2 analysis) | Bug Reproduction §Suspect 2 | If serde returned an error for unknown `errors` field, the renderer would emit a diagnostic comment instead of silently dropping it. The existing passing test `render_with_errors_populates_form_fields` implicitly confirms this — it does not contain a diagnostic comment assertion and passes today. | [VERIFIED: serde_json behavior — `#[serde(deny_unknown_fields)]` is NOT on any of the form prop structs in component.rs] |
| A2 | Gestiscilo has no working code that reads the plural `errors` field from props | Risk Surface §Fix B | If any gestiscilo consumer reads `errors` (plural) it would break after Fix B. Near-zero probability since the feature has never worked end-to-end. Still requires the D-08 audit. | [ASSUMED] |
| A3 | Flash divergence in gestiscilo Phase 175 is consumer-side, not ferro-side | Bug Reproduction §Suspect 3 | If a ferro bug exists in the session layer under specific middleware ordering, suspect 3 would need a ferro fix. Source reading found no evidence of a ferro bug. | [ASSUMED — pending gestiscilo audit] |

---

## Open Questions

1. **Should `render_json` and `render_json_with_errors` also apply the runtime-data merge?**
   - What we know: These methods return spec+data JSON. The `$data` expressions in the resolved spec would also benefit from runtime-data visibility.
   - What's unclear: Whether any consumer relies on `$data` markers surviving unresolved in the JSON output.
   - Recommendation: Apply the same merge for consistency. The resolved spec returned by `render_json` should have the same expression resolution as the rendered HTML.

2. **Should `attach_errors` keep the `resolve_errors_all` variant consistent?**
   - What we know: `resolve_errors_all` (resolve.rs:171-176) also calls `attach_errors` for the full-bag case. The full-bag branch inserts the entire errors map as an object.
   - What's unclear: Whether any consumer uses `resolve_errors_all`.
   - Recommendation: Keep `resolve_errors_all` behavior as-is (it writes the full map under `errors` for structural inspection, not per-field rendering). Only the per-field `attach_errors` path (the `if let Some(k)` branch) needs to change to `"error"`.

3. **`render_json` test `render_json_uses_explicit_data_over_embedded` (mod.rs:639):**
   - The test asserts that explicit data takes precedence over embedded `spec.data`. After Fix A merges both, this test's premise needs review. Since merge_data causes handler data to win on collision, the assertion should still hold. Verify explicitly during implementation.

---

## Environment Availability

Step 2.6: SKIPPED — this phase is a code/fix change to existing Rust crates with no external dependencies.

---

## Recommended Task Structure (Planner Hint)

### Wave 0 — Test Infrastructure (write tests that fail before any fix)

1. **Add `html_body` helper upgrade:** Change existing `render_with_errors_populates_form_fields` and `render_validation_error_accepts_framework_type` tests to use `html_body` and assert on `<p id="err-` tag. These will now FAIL (correct — they expose the bug).
2. **Write D-07 test 1:** `pipeline_data_binding_error_prop_renders_p_tag` — `$data` binding path. Fails before fix.
3. **Write D-07 test 2:** `pipeline_render_validation_error_renders_p_tag` — blessed path. Fails before fix.

### Wave 1 — Pipeline Fixes

4. **Fix A:** `framework/src/json_ui/mod.rs` — merge runtime `data` into spec clone before `resolve` and `resolve_with_errors`. (D-07 test 1 passes after this.)
5. **Fix B:** `ferro-json-ui/src/resolve.rs` — change `attach_errors` to write `"error": first_message`. Update the three tests in resolve.rs:785-833 that assert on `"errors"` plural. (D-07 test 2 passes after this.)

### Wave 2 — Renderer Parity (D-06)

6. **Checkbox error-state parity:** `render_checkbox` — add `has_error` border-destructive swap, focus-ring swap, ARIA attributes, add `id="err-{field}"` to existing error `<p>`.
7. **CheckboxList error-state parity:** `render_checkbox_list` — add ARIA to `<fieldset>`, border-destructive swap on individual checkboxes, add `id` to error `<p>`.
8. **Switch error-state parity:** `render_switch` — swap `ring-primary/30` → `ring-destructive/30`, add ARIA to hidden `<input>`, add `id` to error `<p>`.
9. **Input-file error-state parity:** `render_input` InputType::File branch — add `ring-1 ring-destructive` when `has_error`, add ARIA attributes.

### Wave 3 — Audit + Docs

10. **gestiscilo audit (D-08):** Verify no gestiscilo consumer reads `errors` (plural) from props. Update any gestiscilo handler that can now switch to `JsonUi::render_validation_error` from the manual escape-hatch pattern (smoke test the ~30 `.prop("error")` bindings).
11. **Docs (D-09):** Create or update `docs/src/json-ui/forms.md` covering all four authoring patterns.

---

## Security Domain

Input validation error messages are author-supplied strings (from `ValidationError::new().add("field", "msg")`). The `html_escape` helper is applied to all interpolated error strings in every renderer (`render_input`, `render_select`, `render_checkbox`, `render_switch`, `render_checkbox_list`). No new injection surface is introduced by this fix.

The `merge_data` fix merges handler-supplied `serde_json::Value` objects into spec data before expression resolution. All HTML emission already escapes resolved values. No change to the security model.

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | `html_escape` on all string interpolation into HTML |
| V2 Authentication | no | — |
| V6 Cryptography | no | — |

---

## Sources

### Primary (HIGH confidence — direct source reading)

- `ferro-json-ui/src/expression.rs:35-66` — `resolve_expressions` implementation, confirms spec.data-only scope
- `ferro-json-ui/src/resolve.rs:178-201` — `attach_errors` implementation, confirms `"errors"` plural insertion
- `ferro-json-ui/src/component.rs:282-490` — all form-control prop structs, confirms `error: Option<String>` singular
- `ferro-json-ui/src/render/form.rs:137-318` — `render_input` implementation, confirms renderer is correct
- `framework/src/json_ui/mod.rs:37-311` — full `JsonUi` implementation, confirms render pipeline
- `framework/src/json_ui/mod.rs:816-904` — existing tests, confirms false-positive test issue via `response_body`
- `framework/src/http/request.rs:260-295` — flash helpers, confirms no divergence in ferro layer
- `framework/src/session/store.rs:87-132` — `age_flash_data` and `get_flash`, confirms read-only vs clearing distinction
- `ferro-json-ui/src/spec.rs:256-272` — `merge_data` API, confirms it is the correct tool for Fix A
- `.planning/phases/181-json-ui-input-error-prop-inline-render/181-CONTEXT.md` — locked decisions
- `.planning/phases/181-json-ui-input-error-prop-inline-render/181-UI-SPEC.md` — visual class-chain contract

### Secondary (MEDIUM confidence)

- gestiscilo Phase 175 discovery context (via CONTEXT.md) — confirms the runtime data IS reaching the spec (value="2" restored correctly via data_path), supports suspect 1 as root cause

---

## Metadata

**Confidence breakdown:**
- Bug diagnosis: HIGH — confirmed by direct source reading, not inference
- Proposed fix A: HIGH — directly mirrors existing `render_file` pattern
- Proposed fix B: HIGH — field name change is mechanical; schema matches
- D-06 class chain: HIGH — exact strings verified in UI-SPEC.md
- Flash divergence: MEDIUM — ferro layer clean; consumer side unverified

**Research date:** 2026-05-31
**Valid until:** 2026-06-30 (stable codebase; no external dependencies)
