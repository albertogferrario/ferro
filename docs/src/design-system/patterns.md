# Pattern Catalog

Each section documents one design rule: its rationale, the intents it applies to, a conforming example, a violating example, and how to allow the rule when the exemption is intentional.

Rules are checked by `ferro design:lint` and the `design_lint` MCP tool. A finding is a `Warning` by default — it does not block rendering and does not fail CI unless `--deny` is passed.

---

## `page-header`

**Title:** Dashboard pages start with a PageHeader

**Rationale:** A PageHeader gives every app page a consistent title, breadcrumb, and action-button slot.

**Intents:** all (applies to any spec using the `dashboard` or `app` layout)

### Conforming example

```json
{
  "$schema": "ferro-json-ui/v2",
  "root": "header",
  "layout": "dashboard",
  "elements": {
    "header": { "type": "PageHeader", "props": { "title": "Orders" } },
    "table": { "type": "DataTable", "props": { "empty_message": "No orders" } }
  },
  "design": { "intent": "browse" }
}
```

### Violating example

```json
{
  "$schema": "ferro-json-ui/v2",
  "root": "table",
  "layout": "dashboard",
  "elements": {
    "table": { "type": "DataTable", "props": { "empty_message": "No orders" } }
  },
  "design": { "intent": "browse" }
}
```

### How to allow

Add `"allow": ["page-header"]` to the `design` object when the layout is intentionally exempt (e.g., embedded frames, custom shell layouts):

```json
{ "design": { "intent": "browse", "allow": ["page-header"] } }
```

---

## `prefer-data-table`

**Title:** Prefer DataTable over raw Table

**Rationale:** DataTable adds responsive mobile cards and DropdownMenu row actions the raw Table lacks.

**Intents:** browse

### Conforming example

```json
{
  "elements": {
    "list": { "type": "DataTable", "props": { "empty_message": "No items" } }
  }
}
```

### Violating example

```json
{
  "elements": {
    "list": { "type": "Table" }
  }
}
```

### How to allow

```json
{ "design": { "intent": "browse", "allow": ["prefer-data-table"] } }
```

---

## `list-empty-state`

**Title:** List pages define an empty state

**Rationale:** An empty state with a create CTA turns a blank list into a first-run affordance.

**Intents:** browse

### Conforming example

```json
{
  "elements": {
    "list": {
      "type": "DataTable",
      "props": { "empty_message": "No products yet" }
    }
  }
}
```

An `EmptyState` element anywhere in the spec is also conforming:

```json
{
  "elements": {
    "list": { "type": "DataTable" },
    "empty": {
      "type": "EmptyState",
      "props": { "title": "No products", "action_label": "Add product" }
    }
  }
}
```

### Violating example

```json
{
  "elements": {
    "list": { "type": "DataTable" }
  }
}
```

### How to allow

```json
{ "design": { "intent": "browse", "allow": ["list-empty-state"] } }
```

---

## `row-actions-grouped`

**Title:** Group row/card actions in an ActionGroup

**Rationale:** Loose inline buttons per row are inconsistent and crowd small screens; an ActionGroup/DropdownMenu keeps them tidy.

**Intents:** browse, process

### Conforming example

```json
{
  "elements": {
    "row": {
      "type": "Card",
      "children": ["actions"]
    },
    "actions": {
      "type": "ActionGroup",
      "props": {
        "items": [
          { "label": "Edit", "handler": "items.edit" },
          { "label": "Delete", "destructive": true, "handler": "items.destroy" }
        ]
      }
    }
  }
}
```

### Violating example

```json
{
  "elements": {
    "row": {
      "type": "Card",
      "children": ["btn_edit", "btn_delete"]
    },
    "btn_edit": { "type": "Button", "props": { "label": "Edit" } },
    "btn_delete": { "type": "Button", "props": { "label": "Delete" } }
  }
}
```

### How to allow

```json
{ "design": { "intent": "browse", "allow": ["row-actions-grouped"] } }
```

---

## `breadcrumb-on-subpages`

**Title:** Create/edit/detail pages carry a Breadcrumb

**Rationale:** A breadcrumb back to the list page keeps navigation reversible on nested pages.

**Intents:** collect, focus

### Conforming example

```json
{
  "elements": {
    "header": {
      "type": "PageHeader",
      "props": {
        "title": "New Order",
        "breadcrumb": [{ "label": "Orders", "href": "/orders" }]
      }
    },
    "form": { "type": "Form" }
  },
  "layout": "dashboard"
}
```

### Violating example

```json
{
  "elements": {
    "header": { "type": "PageHeader", "props": { "title": "New Order" } },
    "form": { "type": "Form" }
  },
  "layout": "dashboard"
}
```

### How to allow

```json
{ "design": { "intent": "collect", "allow": ["breadcrumb-on-subpages"] } }
```

---

## `process-kanban`

**Title:** Status-workflow pages use a KanbanBoard

**Rationale:** A KanbanBoard with per-column count badges is the canonical view for status workflows.

**Intents:** process

### Conforming example

```json
{
  "elements": {
    "board": { "type": "KanbanBoard", "props": { "columns": [] } }
  },
  "design": { "intent": "process" }
}
```

### Violating example

```json
{
  "elements": {
    "list": { "type": "DataTable", "props": { "empty_message": "No orders" } }
  },
  "design": { "intent": "process" }
}
```

### How to allow

```json
{ "design": { "intent": "process", "allow": ["process-kanban"] } }
```

---

## `card-actions-in-menu`

**Title:** Kanban card actions belong in the menu, destructive last

**Rationale:** Consistent action order (detail first, destructive last) inside the ActionGroup prevents mis-clicks on cards.

**Intents:** process

### Conforming example

```json
{
  "elements": {
    "board": {
      "type": "KanbanBoard",
      "props": {
        "row_actions": [
          { "label": "View details", "handler": "orders.show" },
          { "label": "Cancel", "destructive": true, "handler": "orders.cancel",
            "confirm": { "title": "Cancel order?", "tone": "destructive" } }
        ]
      }
    }
  }
}
```

### Violating example

```json
{
  "elements": {
    "board": {
      "type": "KanbanBoard",
      "props": {
        "row_actions": [
          { "label": "Cancel", "destructive": true, "handler": "orders.cancel" },
          { "label": "View details", "handler": "orders.show" }
        ]
      }
    }
  }
}
```

### How to allow

```json
{ "design": { "intent": "process", "allow": ["card-actions-in-menu"] } }
```

---

## `create-separate-page`

**Title:** Entity creation is a dedicated page, not a Modal

**Rationale:** A separate create/edit page is linkable, refresh-safe, and leaves room for validation feedback.

**Intents:** collect

### Conforming example

```json
{
  "elements": {
    "form": { "type": "Form" }
  },
  "design": { "intent": "collect" }
}
```

### Violating example

```json
{
  "elements": {
    "modal": { "type": "Modal", "children": ["form"] },
    "form": { "type": "Form" }
  },
  "design": { "intent": "collect" }
}
```

### How to allow

```json
{ "design": { "intent": "collect", "allow": ["create-separate-page"] } }
```

---

## `form-default-values`

**Title:** Edit-form fields pre-fill from data

**Rationale:** On an edit form every field must restore its stored value; a blank field silently discards data on save.

**Intents:** collect

### Conforming example

```json
{
  "elements": {
    "name": {
      "type": "Input",
      "props": {
        "field": "name",
        "default_value": { "$data": "/record/name" }
      }
    },
    "email": {
      "type": "Input",
      "props": {
        "field": "email",
        "default_value": { "$data": "/record/email" }
      }
    }
  }
}
```

### Violating example

Edit form detected (one field has a `$data` default_value), but another field is missing it:

```json
{
  "elements": {
    "name": {
      "type": "Input",
      "props": {
        "field": "name",
        "default_value": { "$data": "/record/name" }
      }
    },
    "email": {
      "type": "Input",
      "props": { "field": "email" }
    }
  }
}
```

### How to allow

```json
{ "design": { "intent": "collect", "allow": ["form-default-values"] } }
```

---

## `destructive-confirmation`

**Title:** Destructive actions require confirmation

**Rationale:** An irreversible action behind a single click is a data-loss hazard; a confirm dialog is the guard.

**Intents:** all

### Conforming example

```json
{
  "elements": {
    "delete_btn": {
      "type": "Button",
      "props": { "label": "Delete", "variant": "destructive" },
      "action": {
        "handler": "items.destroy",
        "method": "DELETE",
        "confirm": { "title": "Delete item?", "tone": "destructive" }
      }
    }
  }
}
```

### Violating example

```json
{
  "elements": {
    "delete_btn": {
      "type": "Button",
      "props": { "label": "Delete", "variant": "destructive" },
      "action": { "handler": "items.destroy", "method": "DELETE" }
    }
  }
}
```

### How to allow

```json
{ "design": { "intent": "collect", "allow": ["destructive-confirmation"] } }
```
