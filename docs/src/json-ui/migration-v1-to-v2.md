# Migration: v1 → v2

This guide covers the changes required when moving controllers from the v1 builder API to the v2 JSON-based spec format. It is organized around the seven most common patterns encountered during migration.

The canonical reference throughout this guide is `app/src/views/pagamenti.json` plus `app/src/controllers/pagamenti.rs` — a minimal working v2 page. Read those two files first.

---

## Cheat sheet

Quick reference for the most common v1-to-v2 rewrites, drawn from production migration work.

| v1 pattern | v2 equivalent | Notes |
|-----------|--------------|-------|
| `JsonUiView::new("name").card(...)` | `JsonUi::render_file("src/views/…/name.json", data)` | UI structure moves to a `.json` file; handler provides data only |
| `Component::Card { children: vec![Component::Text(...)] }` | Flat `elements` map: `Card` with `"children": ["text_id"]` + sibling `Text` element | v2 is flat; nesting expressed through ID references, not inline objects |
| `Component::Plugin { plugin_type: "Stripe", props: {...} }` | Register a `JsonUiPlugin` with a named type (e.g. `"StripeConnectStatus"`); for one-off HTML islands use `"type": "RawHtml"` | Generic `Plugin` dispatch removed (Phase 115 D-01); every plugin has its own type name |
| `Button::new("Submit").action(Action::post("users.store"))` | `"action": { "method": "POST", "handler": "users.store" }` on the element | HTTP method MUST be uppercase (`"POST"`, `"GET"`, etc.) |
| `DetailForm { fields, mode: EditMode::Edit }` | `Form` with `Input` children pre-populated via `data_path`; read/edit modes toggled by `visible` on `query.mode` | See section 4 for the full pattern |
| `view.with_validation_errors(errs)` | Errors in handler data, referenced by `data_path` or `"error"` prop on form inputs | No special view method; shape the data in the handler |
| `make_node_with_action(...)` helper | Hand-author JSON spec or use `Spec::builder()`; run `ferro json-ui:migrate-v1` for simple cases | Codemod handles the common subset; emit uppercase HTTP methods |
| Auth layout implicit `Card` wrapper | Each auth spec declares its own `Card` root with `"variant": "elevated"` | Layout no longer injects a Card; specs own their chrome |
| Conditional rendering via Rust `if/else` in controller | `"visible": { ... }` condition or `"$if": { ... }` directive | `$if` removes the element entirely; `visible` keeps DOM present but hidden |
| `KanbanBoard` with hand-coded columns | `"type": "KanbanBoard", "props": { "data_path": "/order_columns" }` driven by handler data | `data_path` decodes each array entry as `KanbanColumnProps` |

---

## 1. `JsonUi::render_file` vs `Spec::builder()`

**v1 approach:** Controllers called builder methods such as `JsonUiView::new()`, `Component::Card(...)`, and assembled component trees directly in Rust code.

**v2 approach:** UI structure lives in a `.json` file; the controller provides only the data the spec needs.

**v1 (removed):**

```rust
// v1 — JsonUiView, Component, ComponentNode are removed in v2
use ferro::json_ui::{JsonUiView, Component, CardProps};

pub async fn index() -> Response {
    let view = JsonUiView::new()
        .root(Component::Card(CardProps {
            title: "Payments".into(),
            // ...
        }));
    view.render()
}
```

**v2 (current):**

```rust
use ferro::{handler, serde_json, JsonUi, Response};

#[handler]
pub async fn index() -> Response {
    let data = serde_json::json!({
        "meta": {
            "totale_formattato": "€ 1.245,00"
        },
        "pagamenti": [
            { "data": "2026-04-20", "descrizione": "Abbonamento mensile",
              "importo": "€ 99,00", "stato": "Completato" }
        ]
    });
    JsonUi::render_file("src/views/pagamenti.json", data)
}
```

**The corresponding spec file** (`src/views/pagamenti.json`):

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Payments",
  "layout": "dashboard",
  "root": "root",
  "elements": {
    "root": {
      "type": "Card",
      "props": { "title": "Payments" },
      "children": ["stats_row", "payments_table"]
    },
    "stats_row": {
      "type": "StatCard",
      "props": {
        "label": "Total",
        "value": { "$data": "/meta/totale_formattato" }
      }
    },
    "payments_table": {
      "type": "DataTable",
      "props": {
        "columns": [
          { "key": "data", "label": "Date" },
          { "key": "descrizione", "label": "Description" },
          { "key": "importo", "label": "Amount" }
        ],
        "data_path": "/pagamenti",
        "empty_message": "No payments found."
      }
    }
  }
}
```

**When to use `Spec::builder()` instead:** Use the builder escape hatch only when UI structure must vary programmatically in ways that cannot be expressed with a static spec plus `visible` conditions. That is rare in practice. Prefer JSON files — they are readable, diffable, and introspectable via the `json_ui_catalog` MCP tool without compiling the project.

---

## 2. `Card + Form + Alert` depth-flattening

**v1 pattern:** Props structs had nested `children` fields that accepted `Vec<Component>`. This produced deeply nested Rust expressions:

```rust
// v1 — CardProps.children: Vec<Component> is removed in v2
Component::Card(CardProps {
    title: "Account".into(),
    children: vec![
        Component::Form(FormProps {
            fields: vec![
                Component::Input(InputProps { field: "email".into(), label: "Email".into(), ..Default::default() }),
            ],
            action: Some(Action { handler: "account.update".into(), method: "POST".into() }),
        }),
        Component::Alert(AlertProps {
            message: "Saved.".into(),
            variant: AlertVariant::Success,
        }),
    ],
})
```

**v2 approach:** All elements are siblings in a flat `elements` map. A container references its children by their string IDs.

**v2 spec file:**

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Account Settings",
  "layout": "dashboard",
  "root": "page_card",
  "elements": {
    "page_card": {
      "type": "Card",
      "props": { "title": "Account Settings" },
      "children": ["account_form", "save_alert"]
    },
    "account_form": {
      "type": "Form",
      "props": { "max_width": "md" },
      "children": ["email_input", "submit_btn"],
      "action": { "handler": "account.update", "method": "POST" }
    },
    "email_input": {
      "type": "Input",
      "props": {
        "field": "email",
        "label": "Email Address",
        "input_type": "email",
        "data_path": "/user/email"
      }
    },
    "submit_btn": {
      "type": "Button",
      "props": { "label": "Save", "button_type": "submit" }
    },
    "save_alert": {
      "type": "Alert",
      "props": { "message": "Settings saved.", "variant": "success" },
      "visible": { "ne": ["query.flash", null] }
    }
  }
}
```

**Rules for the flat map:**
- `children` is an ordered array of element IDs — the renderer resolves each ID from the `elements` map.
- Element IDs must be unique within the spec.
- Depth is limited to 5 levels (`MAX_NESTING_DEPTH`). Specs exceeding depth 5 fail validation. Most layouts fit comfortably within this limit; if a design requires deeper nesting, promote inner containers to named top-level elements.
- `FormProps.fields`, `CardProps.children` (as `Vec<Component>`), `GridProps.children`, `CollapsibleProps.children`, `FormSectionProps.children`, and `ButtonGroupProps.buttons` from v1 are all removed. Use `"children": ["id1", "id2"]` on the element instead.

---

## 3. Per-row action interpolation in DataTable

**v1 pattern:** Per-row actions required constructing action URLs in the controller and embedding them in each row object.

**v2 approach:** The spec declares action URLs with `{column_key}` placeholders. The renderer substitutes column values for each row at render time.

**Spec example** (manages a list of pages by `slug_path`):

```json
"pages_table": {
  "type": "DataTable",
  "props": {
    "data_path": "/pages",
    "row_key": "slug_path",
    "columns": [
      { "key": "slug_path", "label": "Slug" },
      { "key": "title",     "label": "Title" },
      { "key": "status",    "label": "Status" }
    ],
    "row_actions": [
      { "label": "Edit",    "action": { "url": "/p/{slug_path}/edit" } },
      { "label": "Delete",  "action": { "url": "/p/{slug_path}/delete",
                                         "confirm": { "message": "Delete this page?" } } }
    ]
  }
}
```

**Row data shape** (from the handler):

```rust
let data = serde_json::json!({
    "pages": [
        { "slug_path": "about",   "title": "About",   "status": "published" },
        { "slug_path": "contact", "title": "Contact", "status": "draft" }
    ]
});
```

At render time, for the first row the renderer substitutes `{slug_path}` → `about`, producing:

- Edit URL: `/p/about/edit`
- Delete URL: `/p/about/delete`

**Supported placeholder grammar:**
- Any column key in the row object: `{slug_path}`, `{label}`, `{status}`, `{id}`, etc.
- The `row_key` prop is the primary key used for CSS class and aria attributes; it does not restrict which column keys are available for URL interpolation.
- Missing keys leave the placeholder text unchanged (no panic, no silent removal). Verify your spec against actual handler output using the `json_ui_verify_action` MCP tool.

---

## 4. Read+edit detail pattern

The v1 `DetailFormProps` / `DetailField` / `EditMode` are removed in v2. The replacement is a standard `Form` element whose children include both read-only display elements and editable input elements, each toggled by a `visible` condition on a query parameter.

**v2 spec** (a user detail page with `?mode=edit` toggle):

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "User Details",
  "layout": "dashboard",
  "root": "details_card",
  "elements": {
    "details_card": {
      "type": "Card",
      "props": { "title": "User Details" },
      "children": ["details_form", "edit_btn"]
    },
    "edit_btn": {
      "type": "Button",
      "props": { "label": "Edit", "variant": "outline" },
      "action": { "url": "?mode=edit" },
      "visible": { "ne": ["query.mode", "edit"] }
    },
    "details_form": {
      "type": "Form",
      "props": { "max_width": "md" },
      "children": [
        "name_view", "name_edit",
        "email_view", "email_edit",
        "submit_btn"
      ],
      "action": { "handler": "users.update", "method": "POST" }
    },
    "name_view": {
      "type": "DescriptionList",
      "props": {
        "items": [{ "label": "Name", "value": { "$data": "/user/name" } }]
      },
      "visible": { "ne": ["query.mode", "edit"] }
    },
    "name_edit": {
      "type": "Input",
      "props": {
        "field": "name",
        "label": "Name",
        "data_path": "/user/name"
      },
      "visible": { "eq": ["query.mode", "edit"] }
    },
    "email_view": {
      "type": "DescriptionList",
      "props": {
        "items": [{ "label": "Email", "value": { "$data": "/user/email" } }]
      },
      "visible": { "ne": ["query.mode", "edit"] }
    },
    "email_edit": {
      "type": "Input",
      "props": {
        "field": "email",
        "label": "Email Address",
        "input_type": "email",
        "data_path": "/user/email"
      },
      "visible": { "eq": ["query.mode", "edit"] }
    },
    "submit_btn": {
      "type": "Button",
      "props": { "label": "Save Changes", "button_type": "submit" },
      "visible": { "eq": ["query.mode", "edit"] }
    }
  }
}
```

**Handler** (no EditMode enum needed):

```rust
#[handler]
pub async fn show(req: Request, id: Path<i32>) -> Response {
    let user = User::find_by_id(*id).one(req.db()).await?
        .ok_or_else(|| not_found("User not found"))?;
    let data = serde_json::json!({ "user": user });
    JsonUi::render_file("src/views/users/show.json", data)
}
```

The `?mode=edit` query parameter is read from `query.mode` in `visible` conditions. No Rust code distinguishes the two modes — the spec drives the behavior entirely.

---

## 5. Data-driven options with `CheckboxList`

`CheckboxList` (added in v2, D-01) replaces hand-rolled checkbox loops in Rust view builders. It supports both static option lists and data-driven options resolved from the spec data at render time.

**Static options:**

```json
"services_list": {
  "type": "CheckboxList",
  "props": {
    "field": "services",
    "label": "Available Services",
    "options": [
      { "value": "consulting", "label": "Consulting" },
      { "value": "support",    "label": "Support" },
      { "value": "training",   "label": "Training" }
    ]
  }
}
```

**Data-driven options** (resolved from handler data at render time):

```json
"services_list": {
  "type": "CheckboxList",
  "props": {
    "field": "services",
    "label": "Choose Services",
    "options_path": "/available_services",
    "selected_path": "/user/selected_services"
  }
}
```

**Handler** (provides the option list and pre-selected values):

```rust
let data = serde_json::json!({
    "available_services": [
        { "value": "consulting", "label": "Consulting" },
        { "value": "support",    "label": "Support" },
        { "value": "training",   "label": "Training" }
    ],
    "user": {
        "selected_services": ["consulting", "training"]
    }
});
JsonUi::render_file("src/views/onboarding/services.json", data)
```

**Props reference:**

| Prop | Type | Description |
|------|------|-------------|
| `field` | `string` | Form field name; each selected checkbox submits as `field=value` |
| `options` | `array \| null` | Static option list: `[{ "value": "...", "label": "..." }]` |
| `options_path` | `string \| null` | JSON Pointer to a data array of options (used when `options` is empty) |
| `selected_path` | `string \| null` | JSON Pointer to a `string[]` of pre-selected values |
| `label` | `string \| null` | Group label |
| `description` | `string \| null` | Help text below the group |
| `disabled` | `boolean \| null` | Disable all checkboxes |
| `error` | `string \| null` | Validation error message |

`options_path` and `selected_path` are plain JSON Pointer strings, not `$data` expressions.

---

## 6. Variant string round-trip with strum derives

In v2 (D-11), `AlertVariant`, `BadgeVariant`, `ButtonVariant`, `ToastVariant`, and related enums derive `strum::AsRefStr`. This means Rust call sites can pass typed enum values instead of hand-typing lowercase strings.

**v1 style (hand-typed strings):**

```rust
// Still works but fragile — typos compile silently
let spec = Spec::builder()
    .alert("alert1", serde_json::json!({
        "message": "Saved.",
        "variant": "success"   // typo "succes" would silently produce unknown variant
    }))
    .build();
```

**v2 style (typed enum):**

```rust
use ferro_json_ui::{AlertVariant, AlertProps};

// AlertVariant::Success serializes to "success" automatically
let props = AlertProps {
    variant: Some(AlertVariant::Success),
    message: "Saved.".to_string(),
    title: None,
};
// serde serialization produces: { "variant": "success", "message": "Saved." }
```

**At AsRefStr call sites:**

```rust
use ferro_json_ui::AlertVariant;

// AsRefStr provides .as_ref() returning the serialized string
assert_eq!(AlertVariant::Success.as_ref(), "success");
assert_eq!(AlertVariant::Warning.as_ref(), "warning");
```

The JSON wire format is unchanged — the spec file still uses lowercase strings. These derives are purely Rust call-site ergonomics.

**Accepted variant strings** (case-insensitive on parse, lowercase on emit):

| Enum | Values |
|------|--------|
| `AlertVariant` | `info`, `success`, `warning`, `error` |
| `BadgeVariant` | `default`, `secondary`, `destructive`, `outline` |
| `ButtonVariant` | `default`, `secondary`, `destructive`, `outline`, `ghost`, `link` |
| `ToastVariant` | `info`, `success`, `warning`, `error` |

---

## 7. Handler-name verification with `json_ui_verify_action`

A common friction point in v1-to-v2 migration is knowing whether a handler name used in a spec action (e.g., `"handler": "dashboard.show"`) is actually registered. Mistyped names produce a 404 at runtime with no compile-time signal.

**v2 approach (D-09):** Use the `json_ui_verify_action` MCP tool before writing or migrating a spec. The tool reads the live route registry and returns the registered route or the closest candidate.

**Tool call:**

```
mcp__ferro__json_ui_verify_action({
  "handler": "dashboard.show",
  "method": "GET"
})
```

**Response on hit:**

```json
{
  "found": true,
  "route": {
    "name": "dashboard.show",
    "method": "GET",
    "path": "/dashboard"
  },
  "candidate": null
}
```

**Response on miss:**

```json
{
  "found": false,
  "route": null,
  "candidate": "dashboard.index"
}
```

**Workflow for migration:**

1. For each `"action": { "handler": "...", "method": "..." }` in the spec being migrated, call `json_ui_verify_action`.
2. If `found: true`, the spec is correct.
3. If `found: false`, the `candidate` field gives the closest registered name by edit distance. Update the spec to use the candidate, then verify again.

This eliminates the need to read `src/routes.rs` manually to confirm handler names during migration.

---

## Summary of removed v1 types

| v1 type | v2 replacement |
|---------|----------------|
| `JsonUiView` | `JsonUi::render_file("src/views/.../*.json", data)` |
| `Component` enum | `"type"` string in the flat `elements` map |
| `ComponentNode` | Element entry in the `elements` map |
| `CardProps.children: Vec<Component>` | `"children": ["id1", "id2"]` on the element |
| `FormProps.fields: Vec<Component>` | `"children": ["id1", "id2"]` on the element |
| `GridProps.children: Vec<Component>` | `"children": ["id1", "id2"]` on the element |
| `CollapsibleProps.children` | `"children": ["id1", "id2"]` on the element |
| `FormSectionProps.children` | `"children": ["id1", "id2"]` on the element |
| `ButtonGroupProps.buttons: Vec<Component>` | `"children": ["id1", "id2"]` on the element |
| `DetailFormProps` / `DetailField` / `EditMode` | `Form` + `DescriptionList` + `visible` conditions (section 4) |
