# Plugins

Plugins extend the JSON-UI component catalog with custom interactive components that require client-side JavaScript or CSS. A plugin registers itself under a unique type name; the renderer calls it whenever a `Component::Plugin` with that type name is encountered.

## What Plugins Are

The 26 built-in components cover most server-driven UI patterns. Plugins fill the gap for components that require rich client-side behavior: interactive maps, chart libraries, rich text editors, video players, calendar widgets, and similar.

A plugin is a Rust struct implementing the `JsonUiPlugin` trait. It declares:

- A unique component type name (e.g., `"Map"`)
- A JSON Schema for its props (used by MCP and agents for discovery)
- A render function that produces an HTML string from props and page data
- CSS and JS asset declarations collected once per page and deduplicated

## The JsonUiPlugin Trait

```rust
pub trait JsonUiPlugin: Send + Sync {
    /// Unique component type name. Must not collide with built-in component names.
    fn component_type(&self) -> &str;

    /// JSON Schema describing accepted props.
    /// Used by MCP/agents for discovery and validation.
    fn props_schema(&self) -> serde_json::Value;

    /// Render the component to an HTML string.
    /// Receives raw props JSON and the view's data for data_path resolution.
    fn render(&self, props: &serde_json::Value, data: &serde_json::Value) -> String;

    /// CSS assets to load in <head>. Deduplicated by URL across the page.
    fn css_assets(&self) -> Vec<Asset>;

    /// JS assets to load before </body>. Deduplicated by URL across the page.
    fn js_assets(&self) -> Vec<Asset>;

    /// Inline initialization JS emitted once per page after assets load.
    /// Returns None if no initialization is needed.
    fn init_script(&self) -> Option<String>;
}
```

The `Asset` type represents a CSS or JS URL with optional Subresource Integrity (SRI) attributes:

```rust
Asset::new("https://cdn.example.com/lib.js")
    .integrity("sha256-...")
    .crossorigin("")
```

## Registering a Plugin

Register plugins at application startup, before handling any requests:

```rust
use ferro::{register_plugin, JsonUiPlugin, Asset};

struct ChartPlugin;

impl JsonUiPlugin for ChartPlugin {
    fn component_type(&self) -> &str { "Chart" }
    // ... other methods
}

// In your app startup (e.g., main.rs or bootstrap):
register_plugin(ChartPlugin);
```

`register_plugin` writes into a global `RwLock<PluginRegistry>`. Registering a type name that already exists replaces the previous plugin. Registration is idempotent-safe: the last registration wins.

If you have multiple plugins, register them all before the server starts accepting connections:

```rust
register_plugin(ChartPlugin);
register_plugin(RichTextPlugin);
register_plugin(VideoPlugin);
```

## Using a Plugin in a View

Use `Component::Plugin` with the registered type name:

```rust
use ferro::*;

let node = ComponentNode {
    key: "sales-chart".to_string(),
    component: Component::Plugin(PluginProps {
        plugin_type: "Chart".to_string(),
        props: serde_json::json!({
            "type": "bar",
            "data_path": "/data/sales",
            "x_key": "month",
            "y_key": "revenue"
        }),
    }),
    action: None,
    visibility: None,
};
```

The framework serializes this as:

```json
{
  "key": "sales-chart",
  "type": "Chart",
  "type": "bar",
  "data_path": "/data/sales",
  "x_key": "month",
  "y_key": "revenue"
}
```

The `plugin_type` field becomes the `"type"` discriminant in JSON. The remaining `props` fields are merged at the same level.

## How Assets Are Collected and Injected

When rendering a page with `render_to_html_with_plugins`, the renderer:

1. Renders all components in order
2. Collects the plugin type names encountered during rendering
3. Calls `css_assets()` and `js_assets()` on each unique plugin type
4. Deduplicates assets by URL (two Map components on the same page load Leaflet once)
5. Injects CSS `<link>` tags into `<head>`, JS `<script>` tags before `</body>`
6. Emits `init_script()` output inline after the JS assets

```rust
use ferro::{JsonUi, JsonUiView};

// render_to_html_with_plugins collects and injects plugin assets automatically
let response = JsonUi::render_with_plugins(&view, &data);
```

If you use the bare `JsonUi::render`, plugin HTML is still rendered but assets are not injected — use `render_with_plugins` in production.

## Example: Building a Chart Plugin

This example shows the full trait implementation pattern for a hypothetical Chart plugin using Chart.js:

```rust
use ferro::{JsonUiPlugin, Asset};
use serde::{Deserialize, Serialize};

pub struct ChartPlugin;

impl JsonUiPlugin for ChartPlugin {
    fn component_type(&self) -> &str {
        "Chart"
    }

    fn props_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "description": "A Chart.js bar or line chart.",
            "required": ["data_path"],
            "properties": {
                "chart_type": {
                    "type": "string",
                    "enum": ["bar", "line", "pie"],
                    "default": "bar",
                    "description": "Chart type"
                },
                "data_path": {
                    "type": "string",
                    "description": "JSON pointer to the data array (e.g., '/data/sales')"
                },
                "x_key": {
                    "type": "string",
                    "description": "Key in each data object for the X-axis"
                },
                "y_key": {
                    "type": "string",
                    "description": "Key in each data object for the Y-axis"
                },
                "height": {
                    "type": "string",
                    "default": "300px",
                    "description": "CSS height of the chart container"
                }
            }
        })
    }

    fn render(&self, props: &serde_json::Value, _data: &serde_json::Value) -> String {
        let chart_type = props.get("chart_type").and_then(|v| v.as_str()).unwrap_or("bar");
        let data_path = props.get("data_path").and_then(|v| v.as_str()).unwrap_or("/data");
        let x_key = props.get("x_key").and_then(|v| v.as_str()).unwrap_or("label");
        let y_key = props.get("y_key").and_then(|v| v.as_str()).unwrap_or("value");
        let height = props.get("height").and_then(|v| v.as_str()).unwrap_or("300px");

        let config = serde_json::json!({
            "type": chart_type,
            "data_path": data_path,
            "x_key": x_key,
            "y_key": y_key
        });

        format!(
            "<canvas data-ferro-chart='{}' style=\"height: {}; width: 100%;\"></canvas>",
            serde_json::to_string(&config).unwrap_or_default(),
            height
        )
    }

    fn css_assets(&self) -> Vec<Asset> {
        // Chart.js has no CSS dependency
        vec![]
    }

    fn js_assets(&self) -> Vec<Asset> {
        vec![
            Asset::new("https://cdn.jsdelivr.net/npm/chart.js@4/dist/chart.umd.min.js")
                .integrity("sha256-...")
                .crossorigin("anonymous"),
        ]
    }

    fn init_script(&self) -> Option<String> {
        Some(r#"
document.querySelectorAll('[data-ferro-chart]').forEach(function(canvas) {
    var cfg = JSON.parse(canvas.getAttribute('data-ferro-chart'));
    // Initialize Chart.js using cfg.type, cfg.data_path, etc.
});
"#.to_string())
    }
}
```

Register it at startup:

```rust
register_plugin(ChartPlugin);
```

Use it in a view:

```rust
ComponentNode {
    key: "monthly-sales".to_string(),
    component: Component::Plugin(PluginProps {
        plugin_type: "Chart".to_string(),
        props: serde_json::json!({
            "chart_type": "bar",
            "data_path": "/data/sales",
            "x_key": "month",
            "y_key": "revenue",
            "height": "400px"
        }),
    }),
    action: None,
    visibility: None,
}
```

## Built-in Plugins

### MapPlugin (Leaflet Integration)

The `MapPlugin` is auto-registered in the global plugin registry when the first view is rendered. It uses Leaflet 1.9.4 to render interactive maps.

**Component type:** `"Map"`

**Props:**

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `center` | `[f64; 2]` | No | - | Map center `[lat, lng]`. Optional with `fit_bounds: true` |
| `zoom` | `u8` | No | `13` | Initial zoom level (0-18) |
| `height` | `String` | No | `"400px"` | CSS height of the map container |
| `fit_bounds` | `bool` | No | `false` | Auto-zoom to fit all markers; ignores `center`/`zoom` if markers present |
| `markers` | `Vec<MapMarker>` | No | `[]` | Markers to place on the map |
| `tile_url` | `String` | No | OpenStreetMap | Custom tile layer URL template |
| `attribution` | `String` | No | OSM credit | Tile layer attribution string |
| `max_zoom` | `u8` | No | `19` | Maximum zoom level |

**MapMarker** fields:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `lat` | `f64` | Yes | Marker latitude |
| `lng` | `f64` | Yes | Marker longitude |
| `popup` | `String` | No | Plain text popup on click |
| `popup_html` | `String` | No | HTML popup content (takes priority over `popup`) |
| `color` | `String` | No | Hex color for a colored pin icon (e.g., `"#3B82F6"`) |
| `href` | `String` | No | URL to navigate to on marker click |

**Assets loaded:** Leaflet CSS and JS from unpkg CDN with SRI hashes. Both assets include `crossorigin=""` for SRI verification.

**Example:**

```rust
use ferro::*;

ComponentNode {
    key: "office-locations".to_string(),
    component: Component::Plugin(PluginProps {
        plugin_type: "Map".to_string(),
        props: serde_json::json!({
            "fit_bounds": true,
            "height": "500px",
            "markers": [
                {
                    "lat": 51.505,
                    "lng": -0.09,
                    "popup": "London HQ",
                    "color": "#3B82F6"
                },
                {
                    "lat": 48.8566,
                    "lng": 2.3522,
                    "popup": "Paris Office",
                    "href": "/offices/paris"
                }
            ]
        }),
    }),
    action: None,
    visibility: None,
}
```

## Discovering Registered Plugins

To see all currently registered plugin types (useful in MCP tools and diagnostics):

```rust
use ferro::registered_plugin_types;

let types = registered_plugin_types();
// Returns a sorted Vec<String> e.g. ["Map", "Chart"]
```

The `ferro-mcp` server also exposes plugin types via the `list_json_ui_plugins` tool for agent discovery.
