# Actions

Actions connect UI elements to Ferro handlers for navigation, form submission, and destructive operations. Actions are declared as an `"action"` field on any element in the `"elements"` map.

## How Actions Work

Every interactive element can carry an `"action"` field alongside its `"type"` and `"props"`. Actions reference handler names (e.g., `"users.store"`) instead of raw URLs. The framework resolves handler names to URLs at render time using the route registry.

- **GET actions** render as links (navigation)
- **Non-GET actions** (POST, PUT, PATCH, DELETE) render as form submissions
- Actions can require confirmation before executing
- `"on_success"` and `"on_error"` control what happens after the server responds

## Basic Action Example

```json
"delete_btn": {
  "type": "Button",
  "props": {
    "label": "Delete",
    "variant": "destructive"
  },
  "action": {
    "handler": "items.destroy",
    "method": "DELETE"
  }
}
```

The `"action"` object lives directly on the element. `"handler"` is the route name; `"method"` is the HTTP verb.

## HTTP Methods

Available methods: `"GET"`, `"POST"`, `"PUT"`, `"PATCH"`, `"DELETE"`.

`"method"` defaults to `"POST"` when omitted.

```json
"save_btn": {
  "type": "Button",
  "props": { "label": "Save" },
  "action": {
    "handler": "items.update",
    "method": "PUT"
  }
}
```

## Confirmation Dialogs

Add a `"confirm"` object to show a confirmation dialog before the action executes:

```json
"delete_btn": {
  "type": "Button",
  "props": {
    "label": "Delete",
    "variant": "destructive"
  },
  "action": {
    "handler": "items.destroy",
    "method": "DELETE",
    "confirm": {
      "title": "Delete this item?",
      "tone": "destructive"
    }
  }
}
```

The `"confirm"` object fields:

| Field | Type | Description |
|-------|------|-------------|
| `"title"` | string | Dialog heading text |
| `"message"` | string (optional) | Additional detail text |
| `"tone"` | string | `"neutral"` (default) or `"destructive"` |

Standard confirmation (no destructive styling):

```json
"action": {
  "handler": "items.store",
  "method": "POST",
  "confirm": {
    "title": "Save changes?"
  }
}
```

## Action Outcomes

The `"on_success"` and `"on_error"` fields control behavior after the server responds. Each is an object with a `"type"` discriminator.

### Redirect

Navigate to a URL after success:

```json
"action": {
  "handler": "items.store",
  "method": "POST",
  "on_success": {
    "type": "redirect",
    "url": "/items"
  }
}
```

### Reload

Reload the current page:

```json
"action": {
  "handler": "settings.update",
  "method": "PUT",
  "on_success": {
    "type": "reload"
  }
}
```

### Notify

Show a notification toast:

```json
"action": {
  "handler": "items.store",
  "method": "POST",
  "on_success": {
    "type": "notify",
    "message": "Item created",
    "tone": "success"
  }
}
```

Notification tones: `"neutral"`, `"success"`, `"warning"`, `"destructive"`. An absent `tone` defaults to `"success"`.

### Show errors

Display validation errors returned from the handler on corresponding form fields:

```json
"action": {
  "handler": "items.store",
  "method": "POST",
  "on_error": {
    "type": "show_errors"
  }
}
```

### Combined example

```json
"action": {
  "handler": "items.store",
  "method": "POST",
  "on_success": {
    "type": "redirect",
    "url": "/items"
  },
  "on_error": {
    "type": "show_errors"
  }
}
```

## Form Actions

The `Form` element uses `"action"` as its submit action. The entire form submits to the handler when the user clicks the submit button.

Complete form element example:

```json
"create_form": {
  "type": "Form",
  "props": {},
  "action": {
    "handler": "items.store",
    "method": "POST",
    "on_success": {
      "type": "redirect",
      "url": "/items"
    },
    "on_error": {
      "type": "show_errors"
    }
  },
  "children": ["name_input", "description_input", "submit_btn"]
}
```

With form fields as sibling elements:

```json
"elements": {
  "create_form": {
    "type": "Form",
    "props": {},
    "action": {
      "handler": "items.store",
      "method": "POST",
      "on_success": { "type": "redirect", "url": "/items" },
      "on_error": { "type": "show_errors" }
    },
    "children": ["name_input", "submit_btn"]
  },
  "name_input": {
    "type": "Input",
    "props": {
      "field": "name",
      "label": "Name",
      "input_type": "text",
      "required": true
    }
  },
  "submit_btn": {
    "type": "Button",
    "props": { "label": "Create" }
  }
}
```

## Navigation Actions (GET)

GET actions render as links. Use `"method": "GET"` on any element to make it a navigation link:

```json
"view_btn": {
  "type": "Button",
  "props": { "label": "View Details" },
  "action": {
    "handler": "items.show",
    "method": "GET"
  }
}
```

## Projection Action Buttons

Action buttons emitted by the [Service Projections](../features/projections.md) renderer use a route-name-free convention: a page-level action button targets `POST /{service}/{action}`, and a DataTable row action targets `POST /{service}/{row_key}/{action}`. These URLs are synthesized from the service and action names rather than resolved from a handler name.

The consumer application provides a handler at that route. The handler must:

- resolve the `(ServiceDef, ActionDef)` for the path parameters (unknown service/action → 404),
- read the tenant from the authenticated principal, never from the request body,
- derive the target state and transition guard from `derive_transition_plan` — never read `to_state`/`status` from the body,
- call `dispatch_write` to run guard re-evaluation, idempotency, audit, and any override.

See the [Write Kernel](../features/write-kernel.md) for the `dispatch_write` pipeline and [Transition Planning](../features/transition-planning.md) for deriving the target state from the state machine.
