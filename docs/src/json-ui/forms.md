# Form Validation

Server-rendered form validation in ferro pairs three primitives: a `ValidationError` value built during request handling, the `_flash.old._validation_errors` session key, and the JSON-UI form-control prop `error`. This page covers the four authoring patterns: the blessed `JsonUi::render_validation_error` path, the manual `$data` binding escape hatch, the flash round-trip on POST→GET, and the cross-field validation summary.

All form-control components (`Input`, `Select`, `Input { input_type: "textarea" }`, `Checkbox`, `CheckboxList`, `Switch`) accept an `error` prop of type `Option<String>`. When set, the renderer emits a destructive-tone class chain on the control and an inline error paragraph below the field:

```html
<p id="err-{field}" class="text-sm text-destructive">{error}</p>
```

The paragraph carries `id="err-{field}"` so the control's `aria-describedby="err-{field}"` pairing announces the error to assistive technology.

## Blessed Path: `JsonUi::render_validation_error`

Most handlers hand the framework a `ValidationError` value and let it plumb messages onto matching fields automatically. The framework matches by the `field` prop on each form-control element.

**GET handler — re-render after a validation failure:**

```rust
use ferro::{JsonUi, session};
use ferro_json_ui::{Element, Spec};
use std::collections::HashMap;

#[handler]
pub async fn show(req: Request) -> Response {
    let spec = build_spec(&req);
    let data = serde_json::json!({});

    let errors: HashMap<String, Vec<String>> = session()
        .and_then(|s| s.get("_flash.old._validation_errors"))
        .unwrap_or_default();
    JsonUi::render_with_errors(&spec, &data, &errors)
}
```

The handler reads the validation errors map from the session flash (written by the POST handler — see Flash Round-Trip below) and passes it explicitly to `render_with_errors`. The framework then matches field names against the spec's form-control elements and populates each matching `error` prop. When no errors were flashed, `unwrap_or_default()` produces an empty map and the form renders without error state.

**POST handler — detect and redirect on failure:**

```rust
#[handler]
pub async fn update(req: Request) -> Response {
    let data = req.input().await?;

    let mut validator = Validator::new(&data);
    validator.rules("email", rules![required(), email()]);

    if let Err(e) = validator.validate() {
        return e
            .with_old_input(&data)
            .redirect_back(req.header("referer"));
    }

    // persist and redirect to success path
}
```

`redirect_back` persists the `ValidationError` into `_flash.new._validation_errors`. The session middleware ages the flash on the next request, making it readable as `_flash.old._validation_errors` in the GET handler.

## Escape Hatch: Manual `$data` Binding

When the blessed path's `field`-keyed match does not fit — for example, a cross-field validation key, a composite key, or an error displayed adjacent to a field with a different name — pass the error string through handler data and reference it from the spec via a `$data` expression.

**Handler:**

```rust
#[handler]
pub async fn show(req: Request) -> Response {
    let mut data = serde_json::Map::new();
    data.insert(
        "overage_threshold_error".to_string(),
        serde_json::json!(req.validation_error("overage_threshold")),
    );
    let data = serde_json::Value::Object(data);

    let spec = Spec::builder()
        .element(
            "field_overage_threshold",
            Element::new("Input")
                .prop("field", "overage_threshold")
                .prop("label", "Overage threshold")
                .prop(
                    "error",
                    serde_json::json!({"$data": "/overage_threshold_error"}),
                ),
        )
        .build()?;

    JsonUi::render(&spec, &data)
}
```

`JsonUi::render` merges the runtime `data` argument into `spec.data` before resolving `$data` expressions, so the binding resolves whether the value originates from the spec's embedded data or from the handler-supplied data object.

The `$data` path must resolve to a string or `null`. When it resolves to `null` (no error), the renderer omits the error paragraph.

Use this path when:
- The validation key differs from the form-control `field` prop.
- The same error message should appear next to a different field than the one that produced it.
- You need to construct the error key dynamically (e.g., `{form_id}_{field}_error`).

## Flash Round-Trip on POST → GET

`ValidationError::with_old_input(&data).redirect_back(req.header("referer"))` stores both the error map and the submitted form values in the session flash. On the following request, the session middleware promotes `_flash.new` to `_flash.old`, making the values readable as `req.old("field")` and `req.validation_error("field")`.

Restore submitted form values on GET re-render using `default_value`:

```rust
Element::new("Input")
    .prop("field", "email")
    .prop("label", "Email")
    .prop(
        "default_value",
        serde_json::json!(
            req.old("email").or_else(|| Some(record.email.clone()))
        ),
    )
```

`req.old("field")` takes precedence — it restores what the user typed on a failed submission. The database value is the fallback for the first GET (before any submission).

`req.old` returns `Option<String>`. `req.validation_error` returns `Option<String>` containing the first error message for the field. Both read from `_flash.old` without clearing the key, so multiple calls in the same handler are safe.

When using `render_validation_error`, the framework plumbs the full error map automatically. When using the manual `$data` escape hatch, call `req.validation_error("field")` explicitly for each field.

## Cross-Field Validation Summary

A top-of-page summary banner is useful when the form has many fields and individual error paragraphs may not be immediately visible after scrolling. Render a summary element conditionally before building the spec:

```rust
let mut builder = Spec::builder();
let mut root_children: Vec<String> = Vec::new();

if req.has_validation_errors() {
    builder = builder.element(
        "validation_summary",
        Element::new("Alert")
            .prop("tone", "destructive")
            .prop("message", "Some fields need attention."),
    );
    root_children.push("validation_summary".to_string());
}

// ... add form fields to builder and root_children ...
```

`req.has_validation_errors()` and `req.validation_error("field")` both read from `_flash.old._validation_errors`. They cannot disagree within a single handler invocation. The session middleware advances `_flash.new` to `_flash.old` once at request boundaries — not during the handler — so the key remains stable across all reads.

If you observe `has_validation_errors()` returning `false` while `validation_error("field")` returns `Some`, audit the handler for a helper that calls a session method consuming the flash key between the two reads.
