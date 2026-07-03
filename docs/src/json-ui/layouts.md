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
