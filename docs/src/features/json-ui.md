# JSON-UI

> **Experimental:** JSON-UI is functional and field-tested but the component schema and plugin interface may evolve. Pin your Ferro version in production.

JSON-UI is a server-driven UI system built into Ferro. You write JSON spec files that describe page structure; handlers supply data as `serde_json::Value`; the framework renders HTML. No frontend build step, no React, no Node.js required.

## Architecture Overview

```
src/views/dashboard.json   (spec file — structure and layout)
          +
handler   (data assembly only — returns serde_json::Value)
          |
          v
JsonUi::render_file("views/dashboard.json", data)
          |
          v
Full HTML page with Tailwind CSS
```

1. The spec file at `src/views/*.json` declares the element tree, layout, and expressions.
2. The handler assembles data — no component building in Rust.
3. `JsonUi::render_file` loads the spec, merges handler data, resolves expressions, and returns an HTML response.

## Key Concepts

- **Spec file** — A JSON file with `"$schema": "ferro-json-ui/v2"`, a flat `"elements"` map, and a `"root"` key naming the entry element. See [Getting Started](../json-ui/getting-started.md).

- **Elements map** — All elements are defined at the top level of `"elements"`. Children reference sibling elements by ID, not by nesting objects.

- **Expressions** — `{ "$data": "/key" }` reads from handler data at render time. `{ "$template": "Hello {name}" }` interpolates data into strings. See [Expressions](../json-ui/expressions.md).

- **Layouts** — The `"layout"` field controls page chrome: `"dashboard"` (sidebar), `"app"` (top nav), `"auth"` (centered card), or omit for minimal. See [Layouts](../json-ui/layouts.md).

- **Actions** — The `"action"` field on any element declares what happens on interaction: handler name, HTTP method, optional confirmation, and success/error outcomes. See [Actions](../json-ui/actions.md).

## When to Use JSON-UI

JSON-UI is well suited for:

- Admin panels and back-office dashboards
- CRUD applications (list, create, edit, delete flows)
- Internal tools and management interfaces
- Rapid prototyping without a frontend build step
- Server-rendered pages where client-side state is not needed

For rich interactive UIs or SPA behavior, the [Inertia.js](inertia.md) integration is the alternative. Both can coexist in the same application on different routes.

## Quick Example

Spec file (`src/views/users.json`):

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Users",
  "layout": "app",
  "root": "users_table",
  "elements": {
    "users_table": {
      "type": "Table",
      "props": {
        "columns": [
          { "key": "name", "label": "Name" },
          { "key": "email", "label": "Email" }
        ],
        "data_path": "/users",
        "empty_message": "No users found"
      }
    }
  }
}
```

Handler (`src/controllers/users.rs`):

```rust
use ferro::{handler, JsonUi, Response};

#[handler]
pub async fn index() -> Response {
    let data = serde_json::json!({
        "users": [
            { "name": "Alice", "email": "alice@example.com" },
            { "name": "Bob",   "email": "bob@example.com" }
        ]
    });
    JsonUi::render_file("views/users.json", data)
}
```

## Plugin System

JSON-UI supports plugin components that extend the built-in catalog with interactive widgets requiring client-side JS/CSS. Plugin components use the same `{"type": "Map", ...}` JSON syntax as built-in components.

### JsonUiPlugin Trait

Each plugin implements the `JsonUiPlugin` trait:

```rust
use ferro_json_ui::plugin::{Asset, JsonUiPlugin};
use serde_json::Value;

pub struct ChartPlugin;

impl JsonUiPlugin for ChartPlugin {
    fn component_type(&self) -> &str {
        "Chart"
    }

    fn props_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "required": ["data"],
            "properties": {
                "data": { "type": "array" }
            }
        })
    }

    fn render(&self, props: &Value, _data: &Value) -> String {
        format!("<div class=\"chart\">{}</div>", props)
    }

    fn css_assets(&self) -> Vec<Asset> {
        vec![Asset::new("https://cdn.example.com/chart.css")]
    }

    fn js_assets(&self) -> Vec<Asset> {
        vec![Asset::new("https://cdn.example.com/chart.js")]
    }

    fn init_script(&self) -> Option<String> {
        Some("initCharts();".to_string())
    }
}
```

### Asset Loading

Plugin assets are injected into the page automatically:

- **CSS assets** go in `<head>` as `<link>` tags
- **JS assets** go before `</body>` as `<script>` tags
- **Init scripts** run after assets load as inline `<script>` blocks
- Assets are deduplicated by URL when multiple instances of the same plugin appear on a page
- SRI integrity hashes are supported via `Asset::new(url).integrity("sha256-...")`

### Registering a Plugin

Register custom plugins at application startup:

```rust
use ferro_json_ui::plugin::register_plugin;

register_plugin(ChartPlugin);
```

Built-in plugins (like Map) are registered automatically and require no manual setup.

## Map Component

The Map component renders interactive maps using [Leaflet 1.9.4](https://leafletjs.com/). Leaflet CSS and JS are loaded via CDN with SRI integrity verification.

### Props

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `center` | `[f64, f64]` | No* | — | Map center as `[latitude, longitude]` |
| `zoom` | number | No | `13` | Zoom level (0–18) |
| `height` | string | No | `"400px"` | CSS height of the map container |
| `fit_bounds` | boolean | No | — | Auto-zoom to fit all markers. When `true`, `center`/`zoom` are ignored if markers exist |
| `markers` | array | No | `[]` | Markers to display |
| `tile_url` | string | No | OpenStreetMap | Custom tile layer URL template |
| `attribution` | string | No | OSM attribution | Tile layer attribution text |
| `max_zoom` | number | No | `19` | Maximum zoom level |

*`center` is optional when `fit_bounds` is `true` and markers are provided.

#### MapMarker

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `lat` | number | Yes | Latitude |
| `lng` | number | Yes | Longitude |
| `popup` | string | No | Plain text popup content |
| `color` | string | No | Hex color for a colored CSS pin (e.g., `"#3B82F6"`) |
| `popup_html` | string | No | HTML popup content (alternative to `popup`) |
| `href` | string | No | URL to navigate to on marker click |

### Basic Example

```json
{
  "type": "Map",
  "props": {
    "center": [51.505, -0.09],
    "zoom": 13,
    "markers": [
      { "lat": 51.5, "lng": -0.09, "popup": "London" }
    ]
  }
}
```

### Notes

- **Tabs and Modals:** Maps inside hidden containers are handled automatically via `IntersectionObserver`.
- **Multiple maps:** Each map container gets a unique ID; multiple maps on the same page work independently.
- **CSP requirements:** Allow `https://unpkg.com` for scripts and `https://*.tile.openstreetmap.org` for tile images.

## CLI Support

Scaffold spec files with the CLI:

```bash
ferro make:json-view UserIndex
```

The command uses AI-powered generation when an Anthropic API key is configured. It reads your models and routes to produce a complete spec file. Without an API key, it falls back to a static template.

Export the full JSON Schema for spec validation:

```bash
ferro json-ui:schema
```

See [JSON Schema](../json-ui/json-schema.md) for details.

## MCP Tools

Three MCP tools support JSON-UI development:

### `json_ui_catalog`

Returns all available components (built-in + registered plugins) with their prop schemas, required vs optional fields, and example JSON. Use this to discover what components exist and look up exact prop names.

### `json_ui_inspect`

Returns a parsed breakdown of an existing spec file: element tree, data paths referenced, actions and their resolved routes, and visibility rules. Use this to debug a spec that isn't rendering as expected.

### `json_ui_generate`

Returns a complete spec file scaffolded from a model and intent description. Use this to rapidly prototype a new view from a model. Requires the Anthropic API key to be set.
