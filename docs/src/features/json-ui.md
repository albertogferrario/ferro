# JSON-UI

> **Experimental:** JSON-UI is functional and field-tested but the component schema and plugin interface may evolve. Pin your Ferro version in production.

JSON-UI is a server-driven UI system that renders Tailwind-styled HTML from Rust data structures. No frontend build step, no React, no Node.js -- define your interface as a component tree and the framework renders it to HTML.

## How It Works

1. Define a `JsonUiView` containing a tree of `ComponentNode` values
2. Attach data, actions, and visibility rules to components
3. Call `JsonUi::render()` to produce a full HTML page with Tailwind classes
4. The framework resolves route names to URLs and binds data automatically

JSON-UI is an alternative to [Inertia.js](inertia.md). Both use the same handler pattern and return `Response`, but JSON-UI outputs server-rendered HTML while Inertia delegates rendering to a React frontend.

## When to Use JSON-UI vs Inertia

| Use Case | JSON-UI | Inertia |
|----------|---------|---------|
| Admin panels and dashboards | Ideal | Overkill |
| CRUD applications | Ideal | Works, but heavier setup |
| Rapid prototyping | Ideal | Slower iteration |
| Server-rendered pages | Built for this | Not designed for this |
| Rich interactive UIs | Limited | Ideal |
| Complex client state | Not suited | Ideal |
| SPA behavior | Not suited | Ideal |

Both can coexist in the same application on different routes.

## Quick Example

```rust
use ferro::{handler, JsonUi, JsonUiView, ComponentNode, Component, CardProps, TableProps,
    Column, Action, Response};

#[handler]
pub async fn index() -> Response {
    let view = JsonUiView::new()
        .title("Users")
        .layout("app")
        .component(ComponentNode {
            key: "header".to_string(),
            component: Component::Card(CardProps {
                title: "User Management".to_string(),
                description: Some("View and manage users".to_string()),
                children: vec![],
                footer: vec![],
            }),
            action: None,
            visibility: None,
        })
        .component(ComponentNode {
            key: "users-table".to_string(),
            component: Component::Table(TableProps {
                columns: vec![
                    Column { key: "name".to_string(), label: "Name".to_string(), format: None },
                    Column { key: "email".to_string(), label: "Email".to_string(), format: None },
                ],
                data_path: "/data/users".to_string(),
                row_actions: None,
                empty_message: Some("No users found".to_string()),
                sortable: None,
                sort_column: None,
                sort_direction: None,
            }),
            action: None,
            visibility: None,
        });

    let data = serde_json::json!({
        "users": [
            {"name": "Alice", "email": "alice@example.com"},
            {"name": "Bob", "email": "bob@example.com"},
        ]
    });

    JsonUi::render(&view, &data)
}
```

## Key Concepts

- **[Components](../json-ui/components.md)** -- 20 built-in component types: Card, Table, Form, Button, Input, Select, Alert, Badge, Modal, Text, Checkbox, Switch, Separator, DescriptionList, Tabs, Breadcrumb, Pagination, Progress, Avatar, and Skeleton.

- **[Actions](../json-ui/actions.md)** -- Route-based navigation and form submission. Actions reference handler names (`"users.store"`) that resolve to URLs at render time.

- **[Data Binding & Visibility](../json-ui/data-binding.md)** -- Pre-fill form fields from handler data via `data_path`, and conditionally show/hide components with visibility rules.

- **[Layouts](../json-ui/layouts.md)** -- Page structure with navigation. Built-in `"app"` layout includes sidebar and header; `"auth"` layout centers content. Custom layouts via the `Layout` trait.

## Plugin System

JSON-UI supports plugin components that extend the built-in set with interactive widgets requiring client-side JS/CSS. Plugin components use the same `{"type": "Map", ...}` JSON syntax as built-in components.

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
| `center` | `Option<[f64; 2]>` | No* | -- | Map center as `[latitude, longitude]` |
| `zoom` | `u8` | No | `13` | Zoom level (0-18) |
| `height` | `String` | No | `"400px"` | CSS height of the map container |
| `fit_bounds` | `Option<bool>` | No | -- | Auto-zoom to fit all markers. When `true`, `center`/`zoom` are ignored if markers exist |
| `markers` | `Vec<MapMarker>` | No | `[]` | Markers to display |
| `tile_url` | `String` | No | OpenStreetMap | Custom tile layer URL template |
| `attribution` | `String` | No | OSM attribution | Tile layer attribution text |
| `max_zoom` | `u8` | No | `19` | Maximum zoom level |

*`center` is optional when `fit_bounds` is `true` and markers are provided.

#### MapMarker

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `lat` | `f64` | Yes | Latitude |
| `lng` | `f64` | Yes | Longitude |
| `popup` | `Option<String>` | No | Plain text popup content |
| `color` | `Option<String>` | No | Hex color for a colored CSS pin (e.g., `"#3B82F6"`). Renders as a DivIcon instead of the default marker |
| `popup_html` | `Option<String>` | No | HTML content for the popup (alternative to plain text `popup`) |
| `href` | `Option<String>` | No | URL to navigate to on marker click |

### Basic Example

```json
{
  "type": "Map",
  "center": [51.505, -0.09],
  "zoom": 13,
  "markers": [
    {"lat": 51.5, "lng": -0.09, "popup": "London"}
  ]
}
```

### Colored Markers with HTML Popups

```json
{
  "type": "Map",
  "fit_bounds": true,
  "markers": [
    {
      "lat": 45.464,
      "lng": 9.190,
      "color": "#3B82F6",
      "popup_html": "<strong>Milan</strong><br>Fashion capital",
      "href": "/places/milan"
    },
    {
      "lat": 41.902,
      "lng": 12.496,
      "color": "#EF4444",
      "popup_html": "<strong>Rome</strong><br>Eternal city",
      "href": "/places/rome"
    }
  ]
}
```

When `fit_bounds` is `true`, the map auto-zooms to fit all markers. Colored markers render as CSS DivIcon pins. Clicking a marker with `href` navigates to that URL.

### Custom Tiles and Height

```json
{
  "type": "Map",
  "center": [40.7128, -74.0060],
  "zoom": 12,
  "height": "600px",
  "tile_url": "https://{s}.tile.opentopomap.org/{z}/{x}/{y}.png",
  "attribution": "Map data: OpenTopoMap",
  "max_zoom": 17
}
```

### Notes

- **Tabs and Modals:** Maps inside hidden containers (Tabs, Modals) are handled automatically. An `IntersectionObserver` calls `invalidateSize()` when the map becomes visible.
- **Multiple maps:** Each map container gets a unique ID. Multiple maps on the same page work independently.
- **CSP requirements:** If using Content Security Policy headers, allow `https://unpkg.com` for scripts and `https://*.tile.openstreetmap.org` for tile images.

## CLI Support

Scaffold views with the CLI:

```bash
ferro make:json-view UserIndex
```

The command uses AI-powered generation when an Anthropic API key is configured. It reads your models and routes to produce a complete view file. Without an API key, it falls back to a static template.

See [CLI Reference](../reference/cli.md) for details.
