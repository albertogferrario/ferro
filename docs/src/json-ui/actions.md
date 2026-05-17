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
      "variant": "danger"
    }
  }
}
```

The `"confirm"` object fields:

| Field | Type | Description |
|-------|------|-------------|
| `"title"` | string | Dialog heading text |
| `"message"` | string (optional) | Additional detail text |
| `"variant"` | string | `"default"` or `"danger"` |

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
    "variant": "success"
  }
}
```

Notification variants: `"success"`, `"info"`, `"warning"`, `"error"`.

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
