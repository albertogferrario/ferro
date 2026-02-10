# Phase 47: JSON-UI Map Component - Research

**Researched:** 2026-02-10
**Domain:** JSON-UI plugin system + Leaflet.js interactive maps
**Confidence:** HIGH

<research_summary>
## Summary

Researched two interrelated domains: (1) designing a plugin system for ferro-json-ui that supports custom interactive components with client-side JS, and (2) integrating Leaflet.js as the first plugin to render interactive maps from server-side JSON.

The current JSON-UI architecture renders 20 static components via a `Component` enum (serde tagged) to HTML+Tailwind. There is no mechanism for client-side JavaScript or third-party library loading. The plugin system must bridge this gap: allow dynamic component registration, declare JS/CSS assets, and render initialization HTML that bootstraps client-side behavior.

Leaflet 1.9.4 is the correct choice for this use case — lightweight (~42KB gzipped), no API key required, CDN-loadable, battle-tested. The key pattern for server-rendered frameworks is: emit a `<div>` with data attributes containing map configuration, then a single initialization script discovers all map containers and bootstraps Leaflet on `DOMContentLoaded`.

**Primary recommendation:** Build a `JsonUiPlugin` trait with `name()`, `props_schema()`, `render()`, and `assets()` methods. A global `PluginRegistry` (mirroring the existing `LayoutRegistry` pattern) handles registration and lookup. The renderer checks built-in components first, then falls back to the plugin registry. The Map component ships as the first built-in plugin using Leaflet 1.9.4 via CDN.
</research_summary>

<standard_stack>
## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Leaflet | 1.9.4 | Interactive map rendering | De facto standard for raster web maps; no API key; ~42KB gzipped; BSD-2 |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Leaflet.markercluster | 1.4.1 | Marker clustering | When >100 markers on a single map |
| leaflet-providers | (latest) | Pre-configured tile layers | When switching tile providers beyond OSM |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Leaflet 1.9.4 | MapLibre GL JS | MapLibre is WebGL vector — better for custom styles, 3D, large datasets, but ~200KB+ and more complex initialization. Overkill for basic marker maps. |
| Leaflet 1.9.4 | Leaflet 2.0 alpha | 2.0 drops IE support, uses Pointer Events, ESM-only. Not production-ready (alpha since Aug 2025). |
| Leaflet 1.9.4 | Mapbox GL JS | Proprietary license since late 2020, requires API key. Not suitable for a framework default. |

### CDN URLs (with SRI)
```html
<!-- Leaflet CSS -->
<link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"
      integrity="sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY="
      crossorigin="" />

<!-- Leaflet JS -->
<script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"
        integrity="sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo="
        crossorigin=""></script>

<!-- MarkerCluster CSS (when needed) -->
<link rel="stylesheet" href="https://unpkg.com/leaflet.markercluster@1.4.1/dist/MarkerCluster.css" />
<link rel="stylesheet" href="https://unpkg.com/leaflet.markercluster@1.4.1/dist/MarkerCluster.Default.css" />

<!-- MarkerCluster JS (when needed) -->
<script src="https://unpkg.com/leaflet.markercluster@1.4.1/dist/leaflet.markercluster.js"></script>
```
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Pattern 1: Plugin Trait (Mirrors Existing LayoutRegistry)

ferro-json-ui already has a trait-based registry pattern for layouts (`Layout` trait + `LayoutRegistry` + global `OnceLock<RwLock<LayoutRegistry>>`). The plugin system should follow the same pattern for consistency.

**Plugin trait:**
```rust
pub trait JsonUiPlugin: Send + Sync {
    /// Unique component type name (e.g., "Map"). Used in JSON: {"type": "Map", ...}
    fn component_type(&self) -> &str;

    /// JSON Schema describing accepted props. Used by MCP/agents for discovery.
    fn props_schema(&self) -> serde_json::Value;

    /// Render the component to an HTML string.
    /// Receives the raw props as serde_json::Value (plugin-defined shape)
    /// and the view data for data_path resolution.
    fn render(&self, props: &serde_json::Value, data: &serde_json::Value) -> String;

    /// CSS assets to load in <head>. Called once per page, deduplicated.
    fn css_assets(&self) -> Vec<Asset>;

    /// JS assets to load before </body>. Called once per page, deduplicated.
    fn js_assets(&self) -> Vec<Asset>;

    /// Inline initialization JS emitted once per page (after assets load).
    /// Returns None if no initialization needed.
    fn init_script(&self) -> Option<String>;
}
```

**Asset declaration:**
```rust
pub struct Asset {
    pub url: String,
    pub integrity: Option<String>,
    pub crossorigin: Option<String>,
}
```

**Plugin registry (mirrors LayoutRegistry):**
```rust
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn JsonUiPlugin>>,
}

// Global access via OnceLock<RwLock<PluginRegistry>>
pub fn register_plugin(plugin: impl JsonUiPlugin + 'static);
pub fn get_plugin(component_type: &str) -> Option<...>;
```

### Pattern 2: Component Enum Fallback to Plugin Registry

The `render_component` function in `render.rs` currently does an exhaustive match on the `Component` enum. For plugins, the component tree needs to support "unknown" component types that get dispatched to the plugin registry.

**Two approaches:**

**A. Add `Plugin` variant to Component enum (recommended):**
```rust
#[serde(tag = "type")]
pub enum Component {
    Card(CardProps),
    // ... existing 20 variants ...
    #[serde(untagged)]  // Catches unknown types
    Plugin(serde_json::Value),
}
```

The `Plugin` variant captures any JSON object whose `"type"` field doesn't match a built-in. The renderer extracts the `"type"` field and looks it up in the plugin registry.

**B. Separate handling at the view level:**
Plugins live outside the Component enum entirely — the view has a separate field for plugin components. This avoids touching the enum but complicates the component tree (plugins can't be children of Card, Tabs, etc.).

**Recommendation: Approach A.** Plugins should be nestable inside any container just like built-in components.

### Pattern 3: Data Attributes for Server-to-Client Data Transfer

For interactive plugins like Map, the server renders an HTML container with configuration encoded as data attributes. A client-side init script discovers containers and bootstraps the JS library.

**Server emits:**
```html
<div class="ferro-plugin-map"
     data-ferro-map-id="map-abc123"
     data-center="[51.505, -0.09]"
     data-zoom="13"
     data-markers='[{"lat":51.5,"lng":-0.09,"popup":"Hello"}]'
     style="height: 400px; width: 100%;">
</div>
```

**Init script (emitted once per page, after Leaflet JS loads):**
```javascript
document.addEventListener('DOMContentLoaded', function() {
    document.querySelectorAll('.ferro-plugin-map').forEach(function(el) {
        var center = JSON.parse(el.dataset.center || '[0,0]');
        var zoom = parseInt(el.dataset.zoom || '2', 10);
        var map = L.map(el).setView(center, zoom);
        L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
            maxZoom: 19,
            attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
        }).addTo(map);
        if (el.dataset.markers) {
            JSON.parse(el.dataset.markers).forEach(function(m) {
                var marker = L.marker([m.lat, m.lng]).addTo(map);
                if (m.popup) marker.bindPopup(m.popup);
            });
        }
    });
});
```

For large datasets (many markers, GeoJSON), use a `<script type="application/json">` block adjacent to the container instead of data attributes.

### Pattern 4: Asset Deduplication in Layout Pipeline

The rendering pipeline must collect assets from all plugins used on a page and deduplicate them. The flow:

1. `render_to_html()` walks the component tree
2. For each plugin component, record which plugins were used
3. After rendering, collect CSS assets (for `<head>`) and JS assets (for before `</body>`)
4. Deduplicate by URL
5. Inject into `LayoutContext` — CSS goes into `head`, JS goes into a new `scripts` field

This requires extending `LayoutContext` with a `scripts` field:
```rust
pub struct LayoutContext<'a> {
    // ... existing fields ...
    pub scripts: &'a str,  // JS assets + init scripts, injected before </body>
}
```

And `base_document()` must include `{scripts}` before `</body>`.

### Anti-Patterns to Avoid
- **Inline `<script>` per component instance:** Violates CSP `script-src` and is fragile. Use a single init script that discovers all instances.
- **Requiring a build step for plugins:** The whole point of JSON-UI is zero frontend build. CDN-only JS/CSS.
- **Tight coupling to Leaflet in the plugin trait:** The trait must be library-agnostic. Map uses Leaflet, but future plugins (Chart, Editor) will use different libraries.
- **Passing serde_json::Value through the entire render pipeline:** Built-in components stay as typed enums (performance, type safety). Only plugin components use Value.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Map rendering | Custom canvas/SVG map | Leaflet 1.9.4 | Tile management, zoom, pan, touch, projections — decades of solved problems |
| Marker clustering | Distance-based grouping | Leaflet.markercluster | Cluster algorithms, animated transitions, spiderfy on click |
| Tile management | Custom tile loader | Leaflet tile layer | Caching, retina, error handling, CRS projections |
| Geocoding | Lat/lng lookup | Nominatim API or Geoapify | Address-to-coordinate is a complex NLP+data problem |
| Map projections | Custom coordinate math | Leaflet CRS system | EPSG:3857/4326 conversion is non-trivial |
| Asset deduplication | Manual tracking | Simple HashSet by URL | Don't overcomplicate — collect used plugins, deduplicate URLs |

**Key insight:** The plugin system is the novel work here. The Map component itself should be a thin wrapper around Leaflet — data attributes in, working map out. The less custom JS, the better.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Map Container Must Have Explicit Height
**What goes wrong:** Map renders as 0px or shows a single tile in the corner.
**Why it happens:** Leaflet cannot determine its own height from CSS. If the container has no explicit height (e.g., relies on content height), the map has no size.
**How to avoid:** Always render the map container with an explicit `style="height: Xpx"`. Default to `400px`, allow override via props.
**Warning signs:** Map not visible or tiny grey square.

### Pitfall 2: Map Inside Hidden Container (Tabs, Modals)
**What goes wrong:** Map initializes with wrong dimensions, tiles misaligned.
**Why it happens:** Leaflet calculates tile positions at initialization time. If container is `display: none` (inside inactive tab or closed modal), dimensions are 0.
**How to avoid:** Call `map.invalidateSize()` when the container becomes visible. For the Map plugin, document that maps inside Tabs/Modals need the tab-switch handler to call `invalidateSize()`. Consider providing a `data-lazy-init` option that defers initialization until the container is visible (using IntersectionObserver).
**Warning signs:** Tiles appear shifted after opening tab/modal containing map.

### Pitfall 3: CSP Compatibility
**What goes wrong:** Map fails to load in applications with strict Content Security Policy.
**Why it happens:** Leaflet requires `'unsafe-inline'` in `style-src` (popups inject inline styles) and `data:` in `img-src` (uses data URIs for empty images). CDN scripts need allowlisting in `script-src`.
**How to avoid:** Document CSP requirements clearly. The framework should note that plugins may have CSP requirements. Provide a recommended CSP policy.
**Warning signs:** Console errors about CSP violations; map renders but popups don't show.

### Pitfall 4: serde Untagged Variant Ordering
**What goes wrong:** The `#[serde(untagged)]` Plugin variant in the Component enum matches everything, swallowing legitimate built-in components.
**Why it happens:** Serde tries untagged variants in declaration order. If `Plugin(Value)` is tried before named variants, it matches any JSON object.
**How to avoid:** Place the `Plugin` variant LAST in the enum. Serde's tagged enum deserializer tries named variants first (matching the `"type"` field), then falls through to untagged only if no named variant matches.
**Warning signs:** Built-in components like Card suddenly deserialize as Plugin.

### Pitfall 5: Asset Loading Order
**What goes wrong:** Init script runs before Leaflet JS loads, causing `L is not defined`.
**Why it happens:** CSS links and JS scripts loaded out of order, or init script placed before library script.
**How to avoid:** CSS in `<head>`, library JS before init scripts, all before `</body>`. The `DOMContentLoaded` listener in the init script provides a safety net, but script ordering must still be correct.
**Warning signs:** `ReferenceError: L is not defined` in console.

### Pitfall 6: Too Many Markers Without Clustering
**What goes wrong:** Page becomes unresponsive with thousands of individual markers.
**Why it happens:** Each marker is a DOM element. 1000+ DOM elements causes layout thrashing.
**How to avoid:** Auto-detect marker count and conditionally load markercluster plugin. Consider a `cluster` boolean prop (default true when markers > threshold).
**Warning signs:** Slow page load, browser memory warnings on map-heavy pages.
</common_pitfalls>

<code_examples>
## Code Examples

### Existing Rendering Pipeline (How Head Content Gets Assembled)

Source: `framework/src/json_ui/mod.rs` — `build_response()` method:

```rust
// 1. Build head content from config
let mut head = String::new();
if config.tailwind_cdn {
    head.push_str(r#"<script src="https://cdn.tailwindcss.com"></script>"#);
}
if let Some(custom) = &config.custom_head {
    head.push_str(custom);
}

// 2. Render component tree to HTML fragment
let rendered = render_to_html(view, data);

// 3. Pass to layout for full page shell
let ctx = LayoutContext {
    title,
    content: &rendered,
    head: &head,
    body_class: &config.body_class,
    view_json: &view_json,
    data_json: &data_json,
};
let html = render_layout(layout_name, &ctx);
```

**Integration point for plugins:** After `render_to_html()`, collect plugin assets and append CSS to `head`, build `scripts` string with JS + init code. Extend `LayoutContext` with `scripts` field.

### Existing Layout Base Document

Source: `ferro-json-ui/src/layout.rs` — needs `{scripts}` before `</body>`:

```rust
fn base_document(title: &str, head: &str, body_class: &str, body_content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    {head}
</head>
<body class="{body_class}">
    {body_content}
</body>
</html>"#,
        // ...
    )
}
```

After plugin support, this becomes:
```rust
fn base_document(title: &str, head: &str, body_class: &str, body_content: &str, scripts: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    ...
    {head}
</head>
<body class="{body_class}">
    {body_content}
    {scripts}
</body>
</html>"#,
        // ...
    )
}
```

### Minimal Leaflet Initialization (Data Attributes Pattern)

Source: Leaflet official docs + verified community pattern:

```javascript
// Single init script — discovers and bootstraps all map containers on the page
document.addEventListener('DOMContentLoaded', function() {
    document.querySelectorAll('[data-ferro-map]').forEach(function(el) {
        var config = JSON.parse(el.dataset.ferroMap);
        var map = L.map(el).setView(config.center || [0, 0], config.zoom || 2);

        // Tile layer (configurable, defaults to OSM)
        L.tileLayer(config.tile_url || 'https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
            maxZoom: config.max_zoom || 19,
            attribution: config.attribution || '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
        }).addTo(map);

        // Markers
        (config.markers || []).forEach(function(m) {
            var marker = L.marker([m.lat, m.lng]).addTo(map);
            if (m.popup) marker.bindPopup(m.popup);
        });
    });
});
```

### Map Props Structure (Rust Side)

```rust
/// Props for the Map plugin component.
/// Serialized to JSON and passed as data attribute on the container div.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapProps {
    /// Map center coordinates [lat, lng].
    pub center: [f64; 2],
    /// Initial zoom level (1-18, default 13).
    #[serde(default = "default_zoom")]
    pub zoom: u8,
    /// Height in pixels or CSS value (default "400px").
    #[serde(default = "default_height")]
    pub height: String,
    /// Markers to display on the map.
    #[serde(default)]
    pub markers: Vec<MapMarker>,
    /// Custom tile URL template (default: OpenStreetMap).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tile_url: Option<String>,
    /// Attribution text (default: OSM attribution).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
    /// Maximum zoom level (default: 19).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_zoom: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapMarker {
    pub lat: f64,
    pub lng: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub popup: Option<String>,
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Leaflet 1.x (raster) | MapLibre GL JS (vector) | 2020+ (fork of Mapbox) | Vector maps are better for custom styles and large data, but Leaflet is still preferred for simple use cases |
| Mapbox GL JS (free) | Mapbox GL JS (proprietary) | Late 2020 | License change pushed community to MapLibre fork; Leaflet remains fully open |
| Leaflet 1.9.4 (stable) | Leaflet 2.0 alpha | Aug 2025 | ESM, Pointer Events, no IE; NOT production-ready yet |

**New tools/patterns to consider:**
- **Protomaps (PMTiles):** Self-hostable vector tile format; could be future Map plugin option for offline/private maps
- **MapLibre GL JS:** If users need vector maps, consider a second "VectorMap" plugin later (not for this phase)

**Deprecated/outdated:**
- **Leaflet 0.x API patterns:** Some tutorials use `L.map('id')` with string ID; modern pattern is `L.map(element)` with DOM element reference (avoids ID collision with multiple maps)
- **Mapbox GL JS for open-source use:** License is proprietary since 2020; use MapLibre fork if vector maps needed
</sota_updates>

<open_questions>
## Open Questions

1. **How should plugins interact with the existing Component enum?**
   - What we know: The `#[serde(untagged)]` approach works if Plugin variant is last. Serde tries tagged variants first.
   - What's unclear: Whether `serde(untagged)` on a single variant within a `serde(tag = "type")` enum actually works as expected. May need `serde(other)` or a custom deserializer.
   - Recommendation: Spike the serde approach early in planning. If `untagged` doesn't work cleanly, use a custom deserializer that checks the plugin registry for unknown types.

2. **Should plugin assets be loaded conditionally or always?**
   - What we know: Conditional loading (only when a Map component is present) reduces page weight. Always loading simplifies implementation.
   - What's unclear: How to detect which plugins are used without a pre-scan of the component tree.
   - Recommendation: Pre-scan the component tree for Plugin variants before rendering, collect unique plugin types, then gather their assets. The scan is O(n) in component count and trivially fast.

3. **How to handle `invalidateSize()` for maps in Tabs/Modals?**
   - What we know: Leaflet needs `invalidateSize()` when container becomes visible after being hidden.
   - What's unclear: Whether the framework should handle this automatically (adding tab-switch listeners that call `invalidateSize()`) or leave it to the init script.
   - Recommendation: The init script can use `IntersectionObserver` to detect when a map container becomes visible and auto-call `invalidateSize()`. This handles Tabs, Modals, and accordions generically without coupling to specific container components.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- Leaflet 1.9.4 official documentation (leafletjs.com) — CDN setup, quick start, tile layers, markers, popups
- Leaflet GitHub releases — version verification (1.9.4 stable, 2.0.0-alpha.1 Aug 2025)
- Leaflet.markercluster GitHub (Leaflet/Leaflet.markercluster) — clustering API and CDN URLs
- ferro-json-ui source code — Component enum, render.rs, layout.rs, config.rs, view.rs patterns
- framework/src/json_ui/mod.rs — Full rendering pipeline, LayoutContext assembly, asset injection point

### Secondary (MEDIUM confidence)
- Leaflet 2.0 alpha announcement (leafletjs.com/2025/05/18/) — ESM, Pointer Events, not production-ready
- Leaflet plugins directory (leafletjs.com/plugins.html) — 551 plugins catalogued
- OSM Tile Usage Policy (operations.osmfoundation.org/policies/tiles/) — Fair use, attribution required
- Multiple comparison articles (Jawg, Geoapify, GIS People) — Leaflet vs MapLibre vs Mapbox consensus

### Tertiary (LOW confidence - needs validation)
- Leaflet CSP issues (#9168, #2461) — `unsafe-inline` required for style-src; should verify with 1.9.4 in actual usage
- `serde(untagged)` behavior within `serde(tag = "type")` enum — needs spike to validate the deserialization approach
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Leaflet.js 1.9.4 for interactive web maps
- Ecosystem: markercluster, tile providers, free tile sources
- Patterns: Plugin trait design, asset loading pipeline, data attributes initialization
- Pitfalls: Container height, hidden containers, CSP, serde enum fallback, marker performance

**Confidence breakdown:**
- Standard stack: HIGH — Leaflet is well-established, versions verified via official releases
- Architecture: HIGH — Plugin pattern mirrors existing LayoutRegistry; rendering pipeline fully understood from source
- Pitfalls: HIGH — All documented via GitHub issues and official docs
- Code examples: HIGH — Verified against Leaflet docs and existing ferro-json-ui source

**Research date:** 2026-02-10
**Valid until:** 2026-03-12 (30 days — Leaflet ecosystem is stable)
</metadata>

---

*Phase: 47-json-ui-map-component*
*Research completed: 2026-02-10*
*Ready for planning: yes*
