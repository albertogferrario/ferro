# Actions

Actions connect UI elements to Ferro handlers for navigation, form submission, and destructive operations.

## How Actions Work

Every interactive element in JSON-UI uses an `Action` to declare what happens when the user interacts with it. Actions reference handler names (e.g., `"users.store"`) instead of raw URLs. The framework resolves handler names to URLs at render time using the route registry.

- **GET actions** render as links (navigation)
- **Non-GET actions** (POST, PUT, PATCH, DELETE) render as form submissions
- Actions can require confirmation before executing
- Success and error outcomes control what happens after the server responds

## Creating Actions

The `Action` struct provides builder methods for common HTTP methods:

```rust
use ferro::Action;

// Form submission (POST, the default)
Action::new("users.store")

// Navigation (GET)
Action::get("users.index")

// Deletion (DELETE)
Action::delete("users.destroy")
```

To override the HTTP method explicitly:

```rust
use ferro::{Action, HttpMethod};

Action::new("users.update").method(HttpMethod::Put)
```

Available methods: `Get`, `Post`, `Put`, `Patch`, `Delete`.

### JSON Equivalent

```json
{
  "handler": "users.store",
  "method": "POST"
}
```

The `method` field serializes as uppercase (`"GET"`, `"POST"`, `"DELETE"`, etc.) and defaults to `"POST"` when omitted.

## Route Parameters

Pass route parameters using the handler's registered route pattern. The framework resolves `"users.show"` to its registered path (e.g., `/users/{user}`), and the frontend renderer substitutes parameters from row data.

```rust
// Table row actions receive parameters from each row's data
Action::get("users.show")
Action::delete("users.destroy")
    .confirm_danger("Delete this user?")
```

## Confirmations

Actions can show a confirmation dialog before executing. Two variants are available:

```rust
use ferro::Action;

// Standard confirmation
Action::new("users.store")
    .confirm("Save changes?")

// Destructive confirmation (danger styling)
Action::delete("users.destroy")
    .confirm_danger("Delete this user?")
```

The `ConfirmDialog` struct behind these builders has three fields:

| Field | Type | Description |
|-------|------|-------------|
| `title` | `String` | Dialog heading text |
| `message` | `Option<String>` | Optional detail text |
| `variant` | `DialogVariant` | `Default` or `Danger` |

### JSON Equivalent

```json
{
  "handler": "users.destroy",
  "method": "DELETE",
  "confirm": {
    "title": "Delete this user?",
    "variant": "danger"
  }
}
```

## Success Outcomes

The `on_success` field controls what happens after the action completes. Four outcome types are available:

| Outcome | Description |
|---------|-------------|
| `Redirect { url }` | Navigate to a URL |
| `Refresh` | Reload the current page |
| `ShowErrors` | Display validation errors on form fields |
| `Notify { message, variant }` | Show a notification toast |

```rust
use ferro::{Action, ActionOutcome, NotifyVariant};

// Redirect after successful creation
Action::new("users.store")
    .on_success(ActionOutcome::Redirect {
        url: "/users".to_string(),
    })

// Refresh the current page
Action::new("settings.update")
    .on_success(ActionOutcome::Refresh)

// Show a notification
Action::new("users.store")
    .on_success(ActionOutcome::Notify {
        message: "User created".to_string(),
        variant: NotifyVariant::Success,
    })
```

Notification variants: `Success`, `Info`, `Warning`, `Error`.

### Error Outcomes

The `on_error` field works identically and controls behavior when the action fails:

```rust
Action::new("users.store")
    .on_error(ActionOutcome::ShowErrors)
```

### JSON Equivalent

```json
{
  "handler": "users.store",
  "method": "POST",
  "on_success": {
    "type": "redirect",
    "url": "/users"
  },
  "on_error": {
    "type": "show_errors"
  }
}
```

Outcome types serialize with a `type` discriminator: `"redirect"`, `"refresh"`, `"show_errors"`, `"notify"`.

## Actions on Components

Actions attach to components in three places:

### ComponentNode Action

Any component can have an action via the `action` field on `ComponentNode`. This makes the entire component interactive:

```rust
use ferro::{
    ComponentNode, Component, ButtonProps, ButtonVariant, Size, Action,
};

ComponentNode {
    key: "create-btn".to_string(),
    component: Component::Button(ButtonProps {
        label: "Create User".to_string(),
        variant: ButtonVariant::Default,
        size: Size::Default,
        disabled: None,
        icon: None,
        icon_position: None,
    }),
    action: Some(Action::get("users.create")),
    visibility: None,
}
```

### Form Action

Forms have a dedicated `action` field on `FormProps` that defines the submission endpoint:

```rust
use ferro::{Component, FormProps, Action};

Component::Form(FormProps {
    action: Action::new("users.store"),
    fields: vec![/* ... */],
    method: None,
})
```

### Table Row Actions

Tables support per-row actions via `row_actions` on `TableProps`:

```rust
use ferro::Action;

let row_actions = vec![
    Action::get("users.show"),
    Action::delete("users.destroy")
        .confirm_danger("Delete this user?"),
];
```

## URL Resolution

The framework resolves action handler names to URLs automatically during rendering. When `JsonUi::render()` or `JsonUi::render_json()` is called, the view is cloned and all actions are walked recursively. Each handler name (e.g., `"users.store"`) is looked up in the route registry and the resolved URL is set on the action's `url` field.

If a handler cannot be resolved, its `url` remains `None`. The original view is never mutated.

```rust
use ferro::{JsonUi, JsonUiView};

// Actions are resolved automatically during render
let view = JsonUiView::new()
    .title("Users")
    .component(/* component with action */);

JsonUi::render(&view, &serde_json::json!({}))
```
