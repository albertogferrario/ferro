# Layouts

Layouts wrap JSON-UI pages with consistent navigation, headers, and page structure.

## How Layouts Work

The `"layout"` field in a spec file selects the HTML shell used to wrap the rendered elements. At render time the framework looks up the layout by name and wraps the component output in a full HTML page — nav chrome, sidebars, header, or a bare container, depending on the layout chosen.

Omitting the field (or leaving it empty) uses the minimal default shell with no navigation.

## Selecting a Layout in a Spec File

Set `"layout"` at the top level of the spec:

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Dashboard",
  "layout": "dashboard",
  "root": "main_card",
  "elements": {
    "main_card": {
      "type": "Card",
      "props": { "title": "Welcome" }
    }
  }
}
```

## Built-in Layouts

| Layout name | Description |
|-------------|-------------|
| `"dashboard"` | Sidebar navigation, sticky header, main content area. For admin panels. |
| `"app"` | Top navigation bar, full-width main area. For app pages. |
| `"auth"` | Centered card, no navigation chrome. For login and register forms. |
| (omit) | Minimal default shell. No navigation chrome. |

### `"dashboard"` layout

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Orders",
  "layout": "dashboard",
  "root": "orders_card",
  "elements": {
    "orders_card": {
      "type": "Card",
      "props": { "title": "Orders" },
      "children": ["orders_table"]
    },
    "orders_table": {
      "type": "DataTable",
      "props": {
        "columns": [
          { "key": "id", "label": "#" },
          { "key": "customer", "label": "Customer" },
          { "key": "total", "label": "Total" }
        ],
        "data_path": "/orders"
      }
    }
  }
}
```

### `"app"` layout

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Profile",
  "layout": "app",
  "root": "profile_card",
  "elements": {
    "profile_card": {
      "type": "Card",
      "props": { "title": "Your Profile" }
    }
  }
}
```

### `"auth"` layout

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Sign In",
  "layout": "auth",
  "root": "login_form",
  "elements": {
    "login_form": {
      "type": "Form",
      "props": {
        "action": {
          "handler": "auth.login",
          "method": "POST",
          "on_success": { "type": "redirect", "url": "/" },
          "on_error": { "type": "show_errors" }
        }
      },
      "children": ["email_input", "password_input", "submit_btn"]
    },
    "email_input": {
      "type": "Input",
      "props": { "field": "email", "input_type": "email", "label": "Email" }
    },
    "password_input": {
      "type": "Input",
      "props": { "field": "password", "input_type": "password", "label": "Password" }
    },
    "submit_btn": {
      "type": "Button",
      "props": { "label": "Sign In", "button_type": "submit", "variant": "primary" }
    }
  }
}
```

### Default (no layout field)

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Report",
  "root": "report_card",
  "elements": {
    "report_card": {
      "type": "Card",
      "props": { "title": "Monthly Report" }
    }
  }
}
```

## Custom Layouts

Implement the `Layout` trait and register the layout at application startup. After registration, the layout name is available in any spec file.

### Implementing the trait

```rust
use ferro_json_ui::{Layout, LayoutContext};

pub struct MyLayout;

impl Layout for MyLayout {
    fn render(&self, ctx: &LayoutContext) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{title}</title>
    {head}
</head>
<body class="{body_class}">
    <header>My App</header>
    <main>{content}</main>
    {scripts}
</body>
</html>"#,
            title = ctx.title,
            head = ctx.head,
            body_class = ctx.body_class,
            content = ctx.content,
            scripts = ctx.scripts,
        )
    }
}
```

### Registering in app bootstrap

```rust
use ferro_json_ui::register_layout;

// In src/bootstrap.rs or main.rs, before the server starts:
register_layout("my-layout", MyLayout);
```

After registration, use the name in any spec file:

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Custom Page",
  "layout": "my-layout",
  "root": "root_element",
  "elements": {
    "root_element": {
      "type": "Card",
      "props": { "title": "Custom layout example" }
    }
  }
}
```

Registering a name that already exists replaces the previous layout. Registration order does not matter as long as registration completes before the first request is served.

## LayoutContext Fields

Custom layout implementations receive a `LayoutContext` with all data needed to produce a complete HTML page:

| Field | Type | Description |
|-------|------|-------------|
| `title` | `&str` | Page title from the spec `"title"` field |
| `content` | `&str` | Rendered element HTML fragment |
| `head` | `&str` | Additional `<head>` content (CSS links, meta tags) |
| `body_class` | `&str` | CSS classes for the `<body>` element |
| `scripts` | `&str` | JS assets and init scripts for plugins, placed before `</body>` |

Always include `ctx.scripts` in custom layouts — it carries plugin JS assets injected automatically by the render pipeline.

---

## fill_viewport

The `fill_viewport: true` spec-level flag switches from whole-page scroll to internal per-pane scrolling. Each child pane of the root `Grid` gets its own independent scroll container via the `ferro-fill` CSS chain, so content that overflows one pane does not cause the page to scroll.

Set `fill_viewport` at the spec level and `fill: true` on the root `Grid` element:

```json
{
  "$schema": "ferro-json-ui/v2",
  "fill_viewport": true,
  "layout": "dashboard",
  "root": "root_grid",
  "elements": {
    "root_grid": {
      "type": "Grid",
      "props": { "fill": true },
      "children": ["left_pane", "right_pane"]
    }
  }
}
```

**Requirements and lint rules:**

| Condition | Lint rule | Fired when |
|-----------|-----------|------------|
| `fill_viewport: true` required | `register-fill-viewport` | Spec contains `TileGrid`, `SelectionPanel`, or `Numpad` but `fill_viewport` is not set |
| Root `Grid` must have `fill: true` | `register-grid-fill` | `fill_viewport: true` but the root `Grid` element lacks `fill: true` |
| Supported layouts only | `fill-viewport-layout-unknown` | `fill_viewport: true` with a layout other than `"app"` or `"dashboard"` |
| `TileGrid` needs a paired `SelectionPanel` | `register-selection-present` | Spec contains a `TileGrid` but no `SelectionPanel` (applies regardless of `fill_viewport`) |

Only the `"app"` and `"dashboard"` shell layouts support `fill_viewport`. Using any other layout with `fill_viewport: true` causes silent whole-page scroll — the `ferro-fill` CSS chain is only wired into these two built-in shells. Validate with `design_lint` to catch this before serving.

---

## Register Layout Template

The `register_template()` helper overrides the Collect intent's display layout to `"Register"`, emitting a fill-viewport two-pane composition from the projection layer. The seven-intent vocabulary (`Browse`, `Collect`, `Focus`, `Process`, `Summarize`, `Analyze`, `Track`) is unchanged — `"Register"` is a layout template name, not a new intent.

Pass it via `VisualContext`:

```rust
use ferro::{
    derive_intents, handler, register_template, JsonUi, JsonUiRenderer, Renderer, Response,
    ServiceDef, VisualContext,
};

#[handler]
pub async fn index() -> Response {
    let service = my_service_def();
    let intents = derive_intents(&service);
    let ctx = VisualContext {
        templates: Some(register_template()),
        ..Default::default()
    };
    let spec = JsonUiRenderer
        .render(&service, &intents, &ctx)
        .map_err(|e| ferro::error_response!(500, format!("projection failed: {e}")))?;
    // Nest rows under the "data" key — the derived data_path points at /data/{service}
    let data = serde_json::json!({ "data": { "products": load_products() } });
    JsonUi::render(&spec, &data)
}
```

The projection emits:
- `fill_viewport: true` on the `Spec`
- A root `Grid` with `fill: true` and `"dashboard"` layout
- A `Form` element as the common ancestor of both panes
- A **TileGrid pane** iterating items via the `$each` directive, with `Tile` children bound to the item rows
- A **SelectionPanel pane** with a confirm `Button` in its `children` slot

Existing `Collect` projections without `register_template()` in their `VisualContext` are unaffected — the built-in Collect default (Form layout) stays unchanged.

The projection-derived `/cassa` sample (`app/src/controllers/cassa.rs`) is the reference composition. Cross-reference: [TileGrid](components.md#tilegrid), [SelectionPanel](components.md#selectionpanel).
