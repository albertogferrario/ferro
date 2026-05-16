# Plugins

Plugins extend the JSON-UI component catalog with custom or third-party components that ship their own JavaScript and CSS assets.

## What Plugins Are

The 39 built-in components cover most server-driven UI patterns. Plugins fill the gap for components that require rich client-side behavior: interactive maps, chart libraries, rich text editors, video players, calendar widgets, and similar.

A plugin is a Rust struct implementing the `JsonUiPlugin` trait. It declares:

- A unique component type name (e.g., `"Map"`)
- A JSON Schema for its props (used by MCP and agents for discovery)
- A render function that produces an HTML string from props
- CSS and JS asset declarations collected once per page and deduplicated

## Using a Built-in Plugin in a Spec File

Plugin components appear in a spec file exactly like any other element — just set `"type"` to the plugin's registered name:

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Locations",
  "layout": "dashboard",
  "root": "map_view",
  "elements": {
    "map_view": {
      "type": "Map",
      "props": {
        "center": [51.505, -0.09],
        "zoom": 13,
        "height": "400px",
        "markers": [
          { "lat": 51.5, "lng": -0.09, "popup": "London" }
        ]
      }
    }
  }
}
```

No Rust code is needed to use a registered plugin — the type name in the spec is sufficient.

## How Assets Are Injected

When rendering a spec that contains plugin elements, the framework:

1. Renders all elements in the spec
2. Collects the plugin type names encountered
3. Calls each plugin's `css_assets()` and `js_assets()` methods
4. Deduplicates assets by URL (two `Map` elements on the same page load Leaflet once)
5. Injects CSS `<link>` tags into `<head>` automatically
6. Injects JS `<script>` tags before `</body>` automatically

No manual `<link>` or `<script>` tags are needed. Asset injection is automatic.

## Writing a Custom Plugin

Implement `JsonUiPlugin` and register the plugin at application startup.

### Trait implementation

```rust
use ferro_json_ui::{JsonUiPlugin, Asset};

pub struct ChartPlugin;

impl JsonUiPlugin for ChartPlugin {
    fn component_type(&self) -> &str {
        "Chart"
    }

    fn props_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "required": ["data_path"],
            "properties": {
                "data_path": { "type": "string" },
                "type": { "type": "string", "enum": ["bar", "line", "pie"], "default": "bar" },
                "height": { "type": "string", "default": "300px" }
            }
        })
    }

    fn render(&self, props: &serde_json::Value, _data: &serde_json::Value) -> String {
        let config = serde_json::to_string(props).unwrap_or_default();
        format!(r#"<canvas data-ferro-chart='{}'></canvas>"#, config)
    }

    fn css_assets(&self) -> Vec<Asset> {
        vec![]
    }

    fn js_assets(&self) -> Vec<Asset> {
        vec![
            Asset::new("https://cdn.jsdelivr.net/npm/chart.js@4/dist/chart.umd.min.js")
        ]
    }

    fn init_script(&self) -> Option<String> {
        Some(r#"
document.querySelectorAll('[data-ferro-chart]').forEach(function(canvas) {
    var cfg = JSON.parse(canvas.getAttribute('data-ferro-chart'));
    // initialize Chart.js with cfg
});
"#.to_string())
    }
}
```

### Registering in app bootstrap

```rust
use ferro_json_ui::register_plugin;

// In src/bootstrap.rs or main.rs, before the server starts:
register_plugin(ChartPlugin);
```

After registration, use the plugin in a spec file by setting `"type"` to the registered name:

```json
"revenue_chart": {
  "type": "Chart",
  "props": {
    "data_path": "/revenue_by_month",
    "type": "bar",
    "height": "300px"
  }
}
```

A complete spec using the custom plugin:

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Revenue",
  "layout": "dashboard",
  "root": "revenue_chart",
  "elements": {
    "revenue_chart": {
      "type": "Chart",
      "props": {
        "data_path": "/revenue_by_month",
        "type": "bar",
        "height": "300px"
      }
    }
  }
}
```

## Built-in Plugins

### Map (Leaflet-based)

**Component type:** `"Map"`

Renders an interactive map using Leaflet 1.9.4. Requires internet access for the OpenStreetMap tile CDN.

**Props:**

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `center` | `[lat, lng]` | No | — | Map center coordinates. Optional when `fit_bounds` is `true` |
| `zoom` | `number` | No | `13` | Initial zoom level (0–18) |
| `height` | `string` | No | `"400px"` | CSS height of the map container |
| `fit_bounds` | `boolean` | No | `false` | Auto-zoom to fit all markers; overrides `center`/`zoom` |
| `markers` | `array` | No | `[]` | Markers to place on the map |
| `tile_url` | `string` | No | OpenStreetMap | Custom tile layer URL template |
| `attribution` | `string` | No | OSM credit | Tile layer attribution string |
| `max_zoom` | `number` | No | `19` | Maximum zoom level |

**Marker object fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `lat` | `number` | Yes | Latitude |
| `lng` | `number` | Yes | Longitude |
| `popup` | `string` | No | Plain text popup on click |
| `popup_html` | `string` | No | HTML popup content (takes priority over `popup`) |
| `color` | `string` | No | Hex color for the marker pin (e.g., `"#3B82F6"`) |
| `href` | `string` | No | URL to navigate to on marker click |

**Complete example with multiple markers:**

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Offices",
  "layout": "dashboard",
  "root": "office_map",
  "elements": {
    "office_map": {
      "type": "Map",
      "props": {
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
      }
    }
  }
}
```

Assets loaded automatically: Leaflet CSS (`<head>`) and Leaflet JS (`</body>`), both from unpkg CDN with SRI hashes.

### RichTextEditor (Quill-based)

**Component type:** `"RichTextEditor"`

Renders an interactive rich text editor backed by Quill 2.0.3. The editor container stores its content in a companion hidden input field; the hidden input is submitted with the form on POST.

**Props:**

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `field` | `string` | Yes | — | Form field name for the hidden input |
| `label` | `string` | Yes | — | Label text above the editor |
| `default_value` | `string \| null` | No | `""` | Initial HTML content (static) |
| `data_path` | `string \| null` | No | — | JSON Pointer to pre-fill the editor from handler data |
| `error` | `string \| null` | No | — | Validation error message displayed below the editor |

`data_path` takes precedence over `default_value` when both are set.

**Security:** The editor produces user-controlled HTML. Content sanitization on form submit is the application's responsibility; the framework does not sanitize submitted values.

**Spec example:**

```json
"description_editor": {
  "type": "RichTextEditor",
  "props": {
    "field": "description",
    "label": "Description",
    "data_path": "/document/description"
  }
}
```

**In a form:**

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "Edit Document",
  "layout": "dashboard",
  "root": "edit_card",
  "elements": {
    "edit_card": {
      "type": "Card",
      "props": { "title": "Edit Document" },
      "children": ["doc_form"]
    },
    "doc_form": {
      "type": "Form",
      "props": { "max_width": "lg" },
      "children": ["title_input", "description_editor", "submit_btn"],
      "action": { "handler": "documents.update", "method": "POST" }
    },
    "title_input": {
      "type": "Input",
      "props": { "field": "title", "label": "Title", "data_path": "/document/title" }
    },
    "description_editor": {
      "type": "RichTextEditor",
      "props": {
        "field": "description",
        "label": "Description",
        "data_path": "/document/description"
      }
    },
    "submit_btn": {
      "type": "Button",
      "props": { "label": "Save", "button_type": "submit" }
    }
  }
}
```

Assets loaded automatically: Quill Snow CSS (`<head>`) and Quill JS (`</body>`), both from jsDelivr CDN. SRI hashes are pending verification before production use — see `ferro-json-ui/src/plugins/rich_text_editor.rs` for the TODO marker.

---

## Catalog Discoverability

Plugin components registered in the global registry are automatically surfaced by the `json_ui_catalog` MCP tool, which agents use to discover available components:

```
mcp__ferro__json_ui_catalog({})
```

The response includes a `plugin_components` section listing each registered plugin, its props schema, and a usage example. This means agents authoring specs do not need to read plugin source code — the catalog provides the same discovery surface as built-in components.

The built-in plugins (`Map`, `RichTextEditor`) are pre-registered by `ferro_json_ui::global_plugin_registry()` at framework startup. Custom plugins registered via `register_plugin(MyPlugin)` at application startup are included in the same catalog response.
