# Phase 181: json-ui-input-error-prop-inline-render — Pattern Map

**Mapped:** 2026-05-31
**Files analyzed:** 4 (3 modified, 1 created)
**Analogs found:** 4 / 4

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `framework/src/json_ui/mod.rs` | service / integration layer | request-response, transform | `render_file_with_config` in same file (lines 194-206) | exact — same merge_data pattern |
| `ferro-json-ui/src/resolve.rs` | service / transform | transform | `attach_errors` in same file (lines 178-201) | exact — in-place fix |
| `ferro-json-ui/src/render/form.rs` | utility / renderer | transform | `render_input` / `render_select` in same file (lines 174-184, 213-218, 277-282, 309-315) | exact — parity extension |
| `docs/src/json-ui/forms.md` (new) | doc | n/a | `docs/src/json-ui/data-binding.md` + `docs/src/json-ui/components.md` | structural match |

---

## Pattern Assignments

### `framework/src/json_ui/mod.rs` (service, request-response)

**Changes:** Fix A (merge runtime `data` into spec clone before resolve), upgrade 2 existing tests to `html_body`, add 2 new D-07 integration tests.

**Analog:** `render_file_with_config` in the same file (lines 194-206) — the only call site that already does the merge-before-resolve correctly.

**Merge-before-resolve pattern** (lines 200-205):
```rust
let spec = (*arc_spec).clone().merge_data(handler_data);
let data = spec.data.clone();
let resolved = Self::resolve(&spec);
Self::build_response(&resolved, &data, config)
```

**Fix A target — `render_with_config`** (lines 79-86, current):
```rust
pub fn render_with_config(
    spec: &Spec,
    data: &serde_json::Value,
    config: &JsonUiConfig,
) -> Response {
    let resolved = Self::resolve(spec);
    Self::build_response(&resolved, data, config)
}
```

Post-fix must become:
```rust
pub fn render_with_config(
    spec: &Spec,
    data: &serde_json::Value,
    config: &JsonUiConfig,
) -> Response {
    let spec_with_data = spec.clone().merge_data(data.clone());
    let resolved = Self::resolve(&spec_with_data);
    Self::build_response(&resolved, data, config)
}
```

**Fix A target — `render_with_errors_config`** (lines 261-269, current):
```rust
fn render_with_errors_config(
    spec: &Spec,
    data: &serde_json::Value,
    errors: &HashMap<String, Vec<String>>,
    config: &JsonUiConfig,
) -> Response {
    let resolved = Self::resolve_with_errors(spec, errors);
    Self::build_response(&resolved, data, config)
}
```

Post-fix must become:
```rust
fn render_with_errors_config(
    spec: &Spec,
    data: &serde_json::Value,
    errors: &HashMap<String, Vec<String>>,
    config: &JsonUiConfig,
) -> Response {
    let spec_with_data = spec.clone().merge_data(data.clone());
    let resolved = Self::resolve_with_errors(&spec_with_data, errors);
    Self::build_response(&resolved, data, config)
}
```

**`merge_data` API** (`ferro-json-ui/src/spec.rs:256-272`):
```rust
pub fn merge_data(mut self, handler_data: serde_json::Value) -> Self {
    // consuming (&mut self -> Self). Null/non-Object input is silently ignored.
    if let Some(obj) = handler_data.as_object() {
        if self.data.is_null() {
            self.data = Value::Object(Map::new());
        }
        if let Some(data_map) = self.data.as_object_mut() {
            for (k, v) in obj {
                data_map.insert(k.clone(), v.clone());
            }
        }
    }
    self
}
```

**Test helper pattern** — two helpers already in `mod tests` (lines 333-352):
```rust
fn response_body(response: HttpResponse) -> String {
    let hyper = response.into_hyper();
    let body_bytes = hyper.into_body();
    format!("{body_bytes:?}")           // Debug repr — includes data-view JSON
}

fn html_body(response: HttpResponse) -> String {
    response.body().to_string()         // Raw HTML string — use this for error <p> assertions
}

fn ok_response(result: Response) -> HttpResponse {
    match result {
        Ok(r) => r,
        Err(_) => panic!("expected Ok response, got Err"),
    }
}
```

**CRITICAL:** All new D-07 tests and upgraded existing tests MUST use `html_body(ok_response(result))` not `response_body`. See RESEARCH §Pitfall 1.

**Test naming convention** (from existing test names lines 816-904):
- Existing: `render_with_errors_populates_form_fields`, `render_validation_error_accepts_framework_type`
- New D-07 tests: `pipeline_data_binding_error_prop_renders_p_tag`, `pipeline_render_validation_error_renders_p_tag`
- Prefix convention: `pipeline_*` for full-pipeline integration tests; `render_*` for render-path tests.

**Existing test to upgrade — `render_with_errors_populates_form_fields`** (lines 816-840, current):
```rust
#[test]
fn render_with_errors_populates_form_fields() {
    let spec = form_spec_with_inputs();
    let errors = make_errors(&[
        ("name", &["Name is required"]),
        ("email", &["Email is invalid"]),
    ]);
    let data = serde_json::json!({});
    let result = JsonUi::render_with_errors(&spec, &data, &errors);

    assert!(result.is_ok());
    let body = response_body(ok_response(result));   // ← WRONG: change to html_body
    assert!(body.contains("Name is required"), ...);  // ← WRONG: change to <p id="err-name" assertion
    assert!(body.contains("Email is invalid"), ...);  // ← WRONG: change to <p id="err-email" assertion
}
```

Post-upgrade pattern:
```rust
#[test]
fn render_with_errors_populates_form_fields() {
    let spec = form_spec_with_inputs();
    let errors = make_errors(&[
        ("name", &["Name is required"]),
        ("email", &["Email is invalid"]),
    ]);
    let data = serde_json::json!({});
    let result = JsonUi::render_with_errors(&spec, &data, &errors);

    assert!(result.is_ok());
    let body = html_body(ok_response(result));
    assert!(
        body.contains(r#"<p id="err-name" class="text-sm text-destructive">Name is required</p>"#),
        "error <p> must appear below name input; got: {body}"
    );
    assert!(
        body.contains(r#"<p id="err-email" class="text-sm text-destructive">Email is invalid</p>"#),
        "error <p> must appear below email input; got: {body}"
    );
    assert!(!body.contains("<!-- ferro-json-ui:"), "no diagnostic comments in happy path; got: {body}");
}
```

**D-07 test 1 shape** (from RESEARCH §Suspect 1):
```rust
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
    assert!(!body.contains("<!-- ferro-json-ui:"), "no diagnostic comments in happy path; got: {body}");
}
```

**D-07 test 2 shape** (from RESEARCH §Suspect 2):
```rust
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
    let body = html_body(ok_response(result));

    assert!(
        body.contains(r#"<p id="err-email" class="text-sm text-destructive">must be valid</p>"#),
        "error paragraph must appear below the input; got: {body}"
    );
    assert!(body.contains(r#"aria-invalid="true""#), "aria-invalid must be set; got: {body}");
    assert!(!body.contains("<!-- ferro-json-ui:"), "no diagnostic comments; got: {body}");
}
```

**Spec / import pattern** — test module already imports (lines 330-331):
```rust
use super::*;
use ferro_json_ui::{Action, Element, HttpMethod, Spec};
```

`ValidationError` is accessed as `crate::validation::ValidationError` (used at line 887).

---

### `ferro-json-ui/src/resolve.rs` (service, transform)

**Changes:** Fix B — change `attach_errors` to write `"error": first_message` (singular String) instead of `"errors": Value::Array(...)`. Update 3 existing tests at lines 785-833.

**Current `attach_errors`** (lines 178-201):
```rust
fn attach_errors(el: &mut Element, errors: &HashMap<String, Vec<String>>, all: bool) {
    let Some(props_obj) = el.props.as_object_mut() else {
        return;
    };
    let key = props_obj
        .get("name")
        .or_else(|| props_obj.get("field"))
        .and_then(|v| v.as_str())
        .map(String::from);
    if let Some(k) = key {
        if let Some(msgs) = errors.get(&k) {
            props_obj.insert(
                "errors".to_string(),                              // ← BUG: plural key
                Value::Array(msgs.iter().cloned().map(Value::String).collect()),  // ← BUG: Array
            );
        }
    } else if all {
        if let Ok(errors_value) = serde_json::to_value(errors) {
            props_obj.insert("errors".to_string(), errors_value);  // ← keep as-is (all-bag path)
        }
    }
}
```

Post-fix per-field branch (only the `if let Some(k)` arm changes):
```rust
if let Some(k) = key {
    if let Some(msgs) = errors.get(&k) {
        if let Some(first) = msgs.first() {
            props_obj.insert("error".to_string(), Value::String(first.clone()));
        }
    }
}
```

The `else if all { ... }` branch is NOT changed (it serves `resolve_errors_all` which writes the full bag and is a different contract).

**Three tests to update** (lines 785-833):

Test 1 — `resolve_errors_matches_by_name_prop` (line 785):
```rust
// Current assertion (line 797-798) — WRONG after fix:
let err_val = el.props.as_object().unwrap().get("errors").unwrap();
assert_eq!(err_val, &serde_json::json!(["required"]));

// Post-fix assertion:
let err_val = el.props.as_object().unwrap().get("error").unwrap();
assert_eq!(err_val, &serde_json::json!("required"));
```

Test 2 — `resolve_errors_matches_by_field_prop` (line 802):
```rust
// Current assertion (line 814-815) — WRONG after fix:
let err_val = el.props.as_object().unwrap().get("errors").unwrap();
assert_eq!(err_val, &serde_json::json!(["required"]));

// Post-fix assertion:
let err_val = el.props.as_object().unwrap().get("error").unwrap();
assert_eq!(err_val, &serde_json::json!("required"));
```

Test 3 — `resolve_errors_all_writes_full_bag_when_no_match` (line 819):
The `all` path still uses `"errors"` (the full-bag shape). This test asserts on `el.props.as_object().unwrap().get("errors")` which is correct for the `resolve_errors_all` path. **Do NOT change this test.**

**Naming convention** — existing test names at lines 785-833:
- `resolve_errors_matches_by_name_prop`
- `resolve_errors_matches_by_field_prop`
- `resolve_errors_all_writes_full_bag_when_no_match`

---

### `ferro-json-ui/src/render/form.rs` (renderer, transform)

**Changes:** D-06 parity — add `has_error` class swaps, ARIA attributes, and `id` attribute to error `<p>` for `render_checkbox`, `render_checkbox_list`, `render_switch`, and `render_input` (InputType::File branch). Add 4 new unit tests.

#### Canonical patterns to copy (from `render_input` / `render_select`)

**`has_error` flag** (line 174):
```rust
let has_error = props.error.is_some();
```

**Border swap** (lines 175-179):
```rust
let border_class = if has_error {
    "border-destructive"
} else {
    "border-border"
};
```

**Focus-ring swap** (lines 180-184):
```rust
let focus_ring_class = if has_error {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-destructive focus-visible:ring-offset-2"
} else {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
};
```

**ARIA pair on `<input>`** (lines 277-282):
```rust
if has_error {
    html.push_str(&format!(
        " aria-invalid=\"true\" aria-describedby=\"err-{}\"",
        html_escape(&props.field)
    ));
}
```

**Error `<p>` with `id`** (lines 309-315):
```rust
if let Some(ref error) = props.error {
    html.push_str(&format!(
        "<p id=\"err-{}\" class=\"text-sm text-destructive\">{}</p>",
        html_escape(&props.field),
        html_escape(error)
    ));
}
```

---

#### D-06 Checkbox (lines 434-493)

**Current `<input type="checkbox">` class string** (lines 455-459):
```rust
"<input type=\"checkbox\" id=\"{}\" name=\"{}\" value=\"{}\" class=\"h-4 w-4 rounded-sm border-border text-primary transition-colors duration-150 motion-reduce:transition-none disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2\""
```

Post-fix: introduce `has_error` before the `<input>` emission, then use conditional strings for `border-border`/`border-destructive` and `ring-primary`/`ring-destructive`. Pattern mirrors `render_input` lines 174-184 exactly. Add ARIA block mirroring lines 277-282.

**Current error `<p>` at lines 485-490** (MISSING `id`):
```rust
if let Some(ref error) = props.error {
    html.push_str(&format!(
        "<p class=\"ml-6 text-sm text-destructive\">{}</p>",
        html_escape(error)
    ));
}
```

Post-fix — add `id` attribute:
```rust
if let Some(ref error) = props.error {
    html.push_str(&format!(
        "<p id=\"err-{}\" class=\"ml-6 text-sm text-destructive\">{}</p>",
        html_escape(&props.field),
        html_escape(error)
    ));
}
```

---

#### D-06 CheckboxList (lines 498-590)

**Current `<fieldset>` open tag** (line 544):
```rust
let mut html = String::from("<fieldset class=\"space-y-2\">");
```

Post-fix — conditional ARIA on fieldset:
```rust
let has_error = props.error.is_some();
let mut html = if has_error {
    format!(
        "<fieldset class=\"space-y-2\" aria-invalid=\"true\" aria-describedby=\"err-{}\">",
        html_escape(&props.field)
    )
} else {
    String::from("<fieldset class=\"space-y-2\">")
};
```

**Current per-option `<input>` class** (lines 562-566):
```rust
html.push_str(&format!(
    "<input type=\"checkbox\" id=\"{}\" name=\"{}\" value=\"{}\" \
     class=\"h-4 w-4 rounded-sm border-border text-primary\"",
    ...
));
```

Post-fix — swap `border-border` → `border-destructive` when `has_error`:
```rust
let checkbox_border = if has_error { "border-destructive" } else { "border-border" };
html.push_str(&format!(
    "<input type=\"checkbox\" id=\"{}\" name=\"{}\" value=\"{}\" \
     class=\"h-4 w-4 rounded-sm {} text-primary\"",
    html_escape(&checkbox_id),
    html_escape(&props.field),
    html_escape(&option.value),
    checkbox_border
));
```

**Current error `<p>` at lines 582-587** (MISSING `id`):
```rust
if let Some(ref err) = props.error {
    html.push_str(&format!(
        "<p class=\"text-sm text-destructive mt-1\">{}</p>",
        html_escape(err)
    ));
}
```

Post-fix — add `id`:
```rust
if let Some(ref err) = props.error {
    html.push_str(&format!(
        "<p id=\"err-{}\" class=\"text-sm text-destructive mt-1\">{}</p>",
        html_escape(&props.field),
        html_escape(err)
    ));
}
```

---

#### D-06 Switch (lines 598-718)

**Current pill `<div>` class string** (line 701):
```rust
html.push_str("<div class=\"w-11 h-6 bg-border rounded-full peer peer-checked:bg-primary peer-focus:ring-2 peer-focus:ring-primary/30 after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-background after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full\"></div>");
```

Post-fix — introduce `has_error` above the hidden `<input>` block and use a conditional ring class:
```rust
let has_error = props.error.is_some();
let peer_ring_class = if has_error {
    "peer-focus:ring-2 peer-focus:ring-destructive/30"
} else {
    "peer-focus:ring-2 peer-focus:ring-primary/30"
};
```

Then emit pill as:
```rust
html.push_str(&format!(
    "<div class=\"w-11 h-6 bg-border rounded-full peer peer-checked:bg-primary {} after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-background after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full\"></div>",
    peer_ring_class
));
```

**ARIA on hidden `<input>`** — add after existing attributes (line 684-690 block):
```rust
if has_error {
    html.push_str(&format!(
        " aria-invalid=\"true\" aria-describedby=\"err-{}\"",
        html_escape(&props.field)
    ));
}
```

**Current error `<p>` at lines 705-710** (MISSING `id`):
```rust
if let Some(ref error) = props.error {
    html.push_str(&format!(
        "<p class=\"text-sm text-destructive\">{}</p>",
        html_escape(error)
    ));
}
```

Post-fix — add `id`:
```rust
if let Some(ref error) = props.error {
    html.push_str(&format!(
        "<p id=\"err-{}\" class=\"text-sm text-destructive\">{}</p>",
        html_escape(&props.field),
        html_escape(error)
    ));
}
```

---

#### D-06 Input (file) — `InputType::File` branch (lines 221-237)

**Current `<input type="file">` emission** (lines 222-236) — no ARIA, no ring:
```rust
html.push_str(&format!(
    "<input type=\"file\" id=\"{}\" name=\"{}\" class=\"block w-full text-sm text-text file:mr-4 file:py-2 file:px-4 file:rounded-md file:border-0 file:text-sm file:font-medium file:bg-surface file:text-text hover:file:bg-surface/80\"",
    html_escape(&props.field),
    html_escape(&props.field),
));
```

Post-fix — add conditional `ring-1 ring-destructive` and ARIA block:
```rust
let file_ring_class = if has_error { " ring-1 ring-destructive" } else { "" };
html.push_str(&format!(
    "<input type=\"file\" id=\"{}\" name=\"{}\" class=\"block w-full text-sm text-text file:mr-4 file:py-2 file:px-4 file:rounded-md file:border-0 file:text-sm file:font-medium file:bg-surface file:text-text hover:file:bg-surface/80{}\"",
    html_escape(&props.field),
    html_escape(&props.field),
    file_ring_class
));
// ...existing accept/required/disabled attributes...
if has_error {
    html.push_str(&format!(
        " aria-invalid=\"true\" aria-describedby=\"err-{}\"",
        html_escape(&props.field)
    ));
}
html.push('>');
```

Note: the shared error `<p>` block at lines 309-315 (shared by all non-hidden variants including File) already has `id="err-{field}"` — no change needed there.

---

#### Test patterns for D-06 (new unit tests)

**Test fixture helpers** — copy from existing tests (lines 750-771):
```rust
fn mk_spec(root: &str, el: Element) -> Spec {
    let mut spec = Spec::builder()
        .element("__tmp__", Element::new("Text"))
        .build()
        .expect("builder accepts trivial spec");
    spec.root = root.to_string();
    spec.elements.clear();
    spec.elements.insert(root.to_string(), el);
    spec
}

fn mk_element(type_name: &str, props: Value) -> Element {
    Element {
        type_name: type_name.to_string(),
        props,
        children: Vec::new(),
        action: None,
        visible: None,
        each: None,
        if_: None,
    }
}
```

**Canonical test shape** — mirror `input_error_emits_aria_describedby` at lines 835-851:
```rust
#[test]
fn checkbox_error_renders_destructive_class_and_aria() {
    let el = mk_element(
        "Checkbox",
        json!({"field": "agreed", "label": "Agree", "error": "required"}),
    );
    let spec = mk_spec("root", el.clone());
    let html = render_checkbox(&el, &spec, &json!({}), 1);
    assert!(html.contains("border-destructive"), "got: {html}");
    assert!(html.contains("ring-destructive"), "got: {html}");
    assert!(html.contains("aria-invalid=\"true\""), "got: {html}");
    assert!(html.contains("aria-describedby=\"err-agreed\""), "got: {html}");
    assert!(
        html.contains("<p id=\"err-agreed\" class=\"ml-6 text-sm text-destructive\">required</p>"),
        "got: {html}"
    );
}
```

**Test naming convention** (from VALIDATION.md):
- `checkbox_error_renders_destructive_class_and_aria`
- `checkbox_list_error_renders_fieldset_aria`
- `switch_error_renders_destructive_ring_and_aria`
- `input_file_error_renders_destructive_ring_and_aria`

**Pitfall 3 guard** — assert that `border-destructive` is on the `<input>` tag, not the outer `<div>`. The test must verify the `<input>` tag contains the class, not just that the string appears anywhere in the output. Use position-based assertion if needed:
```rust
// Find the <input> tag and assert border-destructive appears in it, not just anywhere
let input_pos = html.find("<input type=\"checkbox\"").expect("<input not found");
let input_end = html[input_pos..].find('>').expect("> not found") + input_pos;
let input_tag = &html[input_pos..=input_end];
assert!(input_tag.contains("border-destructive"), "border-destructive must be on <input>; input tag: {input_tag}");
```

---

### `docs/src/json-ui/forms.md` (doc, new file)

**Analog structural pattern:** `docs/src/json-ui/data-binding.md` (handler-side code examples + inline code blocks), `docs/src/json-ui/components.md` (component prop tables with Rust code examples).

**No existing `forms.md`** — this is a new file. The directory is `docs/src/json-ui/`.

**D-09 required sections** (from CONTEXT.md D-09):
1. Blessed path — `JsonUi::render_validation_error` end-to-end example
2. Escape hatch — `obj.insert("<field>_error")` + `$data` binding
3. Flash round-trip — `errors.with_old_input(&data).redirect_back(...)` on POST, `req.old(...)` + `req.validation_error(...)` on GET re-render
4. Cross-field summary — `if req.has_validation_errors() { ... }`

**Existing doc style** (from `data-binding.md`):
- H1 title
- Introductory paragraph (1-2 sentences)
- H2 section per major concept
- Rust code blocks for handler examples
- JSON code blocks for spec examples
- No marketing language — neutral, instructional voice

**Section structure to use** (Claude's discretion per CONTEXT.md):
```markdown
# Form Validation

## Blessed Path: render_validation_error
...

## Escape Hatch: Manual $data Binding
...

## Flash Round-Trip Pattern
...

## Cross-Field Validation Summary
...
```

---

## Shared Patterns

### Error `<p>` DOM Shape (locked)
**Source:** `ferro-json-ui/src/render/form.rs:309-315`
**Apply to:** All form-control error `<p>` emitters (checkbox line 485-490, switch line 705-710, checkbox_list line 582-587)

```rust
// The locked DOM shape for error messages.
// Checkbox variant adds ml-6; Switch and CheckboxList omit it.
if let Some(ref error) = props.error {
    html.push_str(&format!(
        "<p id=\"err-{}\" class=\"text-sm text-destructive\">{}</p>",
        html_escape(&props.field),
        html_escape(error)
    ));
}
```

The `id="err-{field}"` attribute is non-negotiable — it is the ARIA `aria-describedby` target.

### ARIA Pairing Pattern
**Source:** `ferro-json-ui/src/render/form.rs:213-218` (textarea branch) and `277-282` (text/email/etc. branch)
**Apply to:** All form controls being patched in D-06 (Checkbox input, Switch hidden input, CheckboxList fieldset, File input)

```rust
if has_error {
    html.push_str(&format!(
        " aria-invalid=\"true\" aria-describedby=\"err-{}\"",
        html_escape(&props.field)
    ));
}
```

### `html_escape` Usage
**Source:** `ferro-json-ui/src/render/mod.rs` (re-exported via `super::html_escape` in form.rs)
**Apply to:** Every string interpolated into any HTML attribute value or text content

Field names used in `id` and `aria-describedby` must be escaped via `html_escape(&props.field)`. Error message text must be escaped via `html_escape(error)`.

### Decode-Failure Diagnostic Comment
**Source:** `ferro-json-ui/src/render/form.rs:437-442` (Checkbox example):
```rust
let props: CheckboxProps = match serde_json::from_value(el.props.clone()) {
    Ok(p) => p,
    Err(e) => {
        return format!(
            "<!-- ferro-json-ui: failed to decode Checkbox props: {} -->",
            html_escape(&e.to_string())
        );
    }
};
```

**Apply to:** All D-07 tests must assert this comment does NOT appear in happy-path renders:
```rust
assert!(!body.contains("<!-- ferro-json-ui:"), "no diagnostic comments; got: {body}");
```

### Test Helper Selection Rule
**Source:** `framework/src/json_ui/mod.rs:340-352`
- `html_body(ok_response(result))` — for any assertion on rendered HTML tags (`<p id="err-`, `aria-invalid`, `border-destructive`)
- `response_body(ok_response(result))` — only for assertions on metadata (URL presence, JSON payload, `data-view` attribute content)

Never use `response_body` when asserting that a DOM element was rendered. The Debug repr includes the serialized spec JSON which contains error strings regardless of whether they were rendered as HTML.

---

## No Analog Found

None — all files have close analogs in the codebase.

---

## Metadata

**Analog search scope:** `framework/src/json_ui/`, `ferro-json-ui/src/resolve.rs`, `ferro-json-ui/src/render/form.rs`, `docs/src/json-ui/`
**Files scanned:** 6 source files + 2 docs files
**Pattern extraction date:** 2026-05-31
