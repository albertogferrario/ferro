# Getting Started with JSON-UI

JSON-UI is a server-driven UI system where you write JSON spec files. Handlers supply data as `serde_json::Value`; the framework loads the spec, resolves expressions against that data, and renders an HTML page. No frontend toolchain is required.

## Prerequisites

- An existing Ferro application
- No additional dependencies — JSON-UI is built into the framework

## Step 1 — Create the spec file

Create `src/views/dashboard.json` in your application:

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Dashboard",
  "layout": "dashboard",
  "root": "welcome",
  "elements": {
    "welcome": {
      "type": "Card",
      "props": {
        "title": "Welcome"
      },
      "children": ["orders_stat"]
    },
    "orders_stat": {
      "type": "StatCard",
      "props": {
        "label": "Orders Today",
        "value": { "$data": "/orders_today" }
      }
    }
  }
}
```

Key points:

- `"root"` is the ID of the top-level element — the entry point when rendering begins.
- `"elements"` is a flat map. Each key is an element ID; each value describes one component.
- `"children"` is an **array of element IDs**, not nested objects. The renderer looks up each ID in `"elements"` to render child components.
- `{ "$data": "/orders_today" }` is a data expression — it reads the `orders_today` field from handler data at render time.

## Step 2 — Write the handler

Create `src/controllers/dashboard.rs`:

```rust
use ferro::{handler, JsonUi, Response};

#[handler]
pub async fn index() -> Response {
    let data = serde_json::json!({
        "orders_today": 42
    });
    JsonUi::render_file("views/dashboard.json", data)
}
```

The handler assembles data as `serde_json::json!({...})` and passes it to `render_file`. No component building happens in Rust — the spec file defines the structure; the handler defines the data.

## Step 3 — Register the route

Add the route in `src/routes.rs`:

```rust
get!("/dashboard", controllers::dashboard::index).name("dashboard.index");
```

## Step 4 — Run the app

```bash
ferro serve
```

Visit `http://localhost:3000/dashboard`. You will see a Card containing a StatCard showing "42".

## Data binding

The `{ "$data": "/path" }` expression reads from handler data using a slash-separated JSON Pointer path. The leading `/` is followed by a key in the data object.

Example: `{ "$data": "/orders_today" }` reads `data.orders_today`.

For nested data:

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "User Profile",
  "root": "profile_card",
  "elements": {
    "profile_card": {
      "type": "Card",
      "props": {
        "title": { "$data": "/user/name" }
      },
      "children": ["email_field"]
    },
    "email_field": {
      "type": "Text",
      "props": {
        "content": { "$data": "/user/email" }
      }
    }
  }
}
```

Handler:

```rust
#[handler]
pub async fn show() -> Response {
    let data = serde_json::json!({
        "user": {
            "name": "Alice",
            "email": "alice@example.com"
        }
    });
    JsonUi::render_file("views/profile.json", data)
}
```

## Layouts

The `"layout"` field in the spec controls page structure. Available built-in layouts:

| Value | Description |
|-------|-------------|
| `"dashboard"` | Sidebar navigation with header |
| `"app"` | Top navigation bar |
| `"auth"` | Centered card, used for login / register pages |
| `""` or omit | Minimal default — no navigation chrome |

Set the layout in the spec root:

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Settings",
  "layout": "app",
  "root": "settings_card",
  "elements": {
    "settings_card": {
      "type": "Card",
      "props": { "title": "Application Settings" }
    }
  }
}
```

## Next Steps

- **[Components](components.md)** — Reference for all built-in component types and their props
- **[Actions](actions.md)** — Navigation, form submission, confirmations, and outcomes
- **[Data Binding & Visibility](data-binding.md)** — Expressions and conditional rendering
- **[Layouts](layouts.md)** — Page structure and custom layouts
- **[Plugins](plugins.md)** — Extend the component catalog with custom interactive components
