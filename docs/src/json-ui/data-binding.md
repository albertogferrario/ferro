# Data Binding & Visibility

Pre-fill form fields from handler data and conditionally show or hide components based on data state.

## Handler Data

JSON-UI views receive data from the handler at render time. This data drives form pre-filling and visibility conditions.

```rust
use ferro_rs::{JsonUi, JsonUiView};

let view = JsonUiView::new()
    .title("Edit User")
    .component(/* ... */);

let data = serde_json::json!({
    "user": {
        "name": "Alice",
        "email": "alice@example.com",
        "role": "admin"
    }
});

// Data passed as second argument
JsonUi::render(&view, &data)
```

Components reference this data via slash-separated paths to pre-fill values or control visibility.

## Data Paths

Data paths are slash-separated strings that resolve against the handler's JSON data. They follow the format `/segment/segment/...` where each segment is an object key or array index.

```
/user/name         -> "Alice"
/user/email        -> "alice@example.com"
/users/0/name      -> first user's name (array index)
/meta/total        -> numeric value
```

### Path Resolution Rules

- Leading slash is required for non-empty paths
- Empty path or `"/"` returns the root value
- Object keys are matched by name
- Array elements are accessed by numeric index
- Missing keys or out-of-bounds indices return `None`

### Form Field Pre-filling

Form field components (`Input`, `Select`, `Checkbox`, `Switch`) support the `data_path` field. At render time, the path is resolved against the handler data and the result pre-fills the field.

```rust
use ferro_rs::{Component, InputProps, InputType};

Component::Input(InputProps {
    field: "name".to_string(),
    label: "Name".to_string(),
    input_type: InputType::Text,
    placeholder: None,
    required: None,
    disabled: None,
    error: None,
    description: None,
    default_value: None,
    data_path: Some("/user/name".to_string()),
})
```

### JSON Equivalent

```json
{
  "type": "Input",
  "field": "name",
  "label": "Name",
  "input_type": "text",
  "data_path": "/user/name"
}
```

When the handler data contains `{"user": {"name": "Alice"}}`, the input is pre-filled with `"Alice"`.

## View Data

Data can come from two sources:

### Embedded Data

Views can carry their own data via `JsonUiView::data()`. This is useful for self-contained views that don't need handler data:

```rust
let view = JsonUiView::new()
    .title("Dashboard")
    .data(serde_json::json!({
        "stats": { "total_users": 150, "active": 42 }
    }))
    .component(/* ... */);
```

### Explicit Handler Data

When rendering, explicit data is passed as the second argument to `JsonUi::render()`:

```rust
let data = serde_json::json!({"users": users_list});
JsonUi::render(&view, &data)
```

When both sources exist, explicit handler data takes priority. This is the "live data override" pattern: embedded data provides defaults, explicit data provides current state.

For `JsonUi::render_json()`, if the explicit data is `null`, the view's embedded data is used as fallback.

## Visibility Rules

Components can be conditionally shown or hidden based on data conditions. Attach a visibility rule via the `visibility` field on `ComponentNode`.

### Simple Conditions

A `VisibilityCondition` checks a data path against an operator and optional value:

```rust
use ferro_rs::{JsonUiVisibility, VisibilityCondition, VisibilityOperator};

// Show component only when users array is not empty
JsonUiVisibility::Condition(VisibilityCondition {
    path: "/data/users".to_string(),
    operator: VisibilityOperator::NotEmpty,
    value: None,
})
```

### JSON Equivalent

```json
{
  "path": "/data/users",
  "operator": "not_empty"
}
```

### Available Operators

| Operator | Serialized | Value Required | Description |
|----------|-----------|----------------|-------------|
| `Exists` | `exists` | No | Path resolves to a non-null value |
| `NotExists` | `not_exists` | No | Path does not resolve |
| `Eq` | `eq` | Yes | Value equals |
| `NotEq` | `not_eq` | Yes | Value does not equal |
| `Gt` | `gt` | Yes | Greater than |
| `Lt` | `lt` | Yes | Less than |
| `Gte` | `gte` | Yes | Greater than or equal |
| `Lte` | `lte` | Yes | Less than or equal |
| `Contains` | `contains` | Yes | String or array contains value |
| `NotEmpty` | `not_empty` | No | Value is not empty (non-null, non-empty string/array) |
| `Empty` | `empty` | No | Value is empty or null |

### Condition with Value

For comparison operators, pass the value to compare against:

```rust
JsonUiVisibility::Condition(VisibilityCondition {
    path: "/auth/user/role".to_string(),
    operator: VisibilityOperator::Eq,
    value: Some(serde_json::json!("admin")),
})
```

```json
{
  "path": "/auth/user/role",
  "operator": "eq",
  "value": "admin"
}
```

## Compound Visibility

Visibility rules support logical composition with `And`, `Or`, and `Not` operators.

### And

All conditions must be true:

```rust
use ferro_rs::{JsonUiVisibility, VisibilityCondition, VisibilityOperator};

JsonUiVisibility::And {
    and: vec![
        JsonUiVisibility::Condition(VisibilityCondition {
            path: "/auth/user".to_string(),
            operator: VisibilityOperator::Exists,
            value: None,
        }),
        JsonUiVisibility::Condition(VisibilityCondition {
            path: "/auth/user/role".to_string(),
            operator: VisibilityOperator::Eq,
            value: Some(serde_json::json!("admin")),
        }),
    ],
}
```

```json
{
  "and": [
    { "path": "/auth/user", "operator": "exists" },
    { "path": "/auth/user/role", "operator": "eq", "value": "admin" }
  ]
}
```

### Or

Any condition must be true:

```json
{
  "or": [
    { "path": "/data/status", "operator": "eq", "value": "active" },
    { "path": "/data/status", "operator": "eq", "value": "pending" }
  ]
}
```

### Not

Negate a condition:

```json
{
  "not": { "path": "/data/is_deleted", "operator": "exists" }
}
```

Compound rules can be nested arbitrarily. The `Visibility` enum uses serde's untagged representation, so the JSON format is clean without a `type` discriminator.

### Attaching to Components

```rust
use ferro_rs::{ComponentNode, Component, TextProps, TextElement, JsonUiVisibility, VisibilityCondition, VisibilityOperator};

ComponentNode {
    key: "admin-notice".to_string(),
    component: Component::Text(TextProps {
        content: "Admin access granted".to_string(),
        element: TextElement::P,
    }),
    action: None,
    visibility: Some(JsonUiVisibility::Condition(VisibilityCondition {
        path: "/auth/user/role".to_string(),
        operator: VisibilityOperator::Eq,
        value: Some(serde_json::json!("admin")),
    })),
}
```

## Validation Errors

The framework integrates validation errors with form field components. When a form submission fails validation, errors are resolved onto matching form fields automatically.

### Rendering with Errors

Use `JsonUi::render_with_errors()` to populate error messages on form fields:

```rust
use std::collections::HashMap;
use ferro_rs::{JsonUi, JsonUiView};

let mut errors = HashMap::new();
errors.insert("email".to_string(), vec!["Email is required".to_string()]);
errors.insert("name".to_string(), vec!["Name is too short".to_string()]);

JsonUi::render_with_errors(&view, &data, &errors)
```

Or use the `ValidationError` type directly:

```rust
use ferro_rs::{JsonUi, JsonUiView};
use ferro_rs::validation::ValidationError;

let validation_error: ValidationError = /* from validator */;
JsonUi::render_validation_error(&view, &data, &validation_error)
```

### How Error Resolution Works

The framework walks the component tree and matches error keys to form field `field` names:

1. For each `Input`, `Select`, `Checkbox`, and `Switch` component, the resolver checks if the errors map contains the field name
2. If found, the first error message is set on the component's `error` prop
3. Existing explicit errors are never overwritten (do-not-overwrite rule)
4. The full errors map is also set on `view.errors` for global display

### View-Level Errors

After error resolution, `view.errors` contains the complete error map. Frontend renderers can use this for displaying a global error summary above the form:

```json
{
  "errors": {
    "email": ["Email is required"],
    "name": ["Name is too short", "Name must be alphanumeric"]
  }
}
```

Individual field components receive the first error message on their `error` prop. Use `resolve_errors_all()` at the crate level to join all messages with `". "` instead.
