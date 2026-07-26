//! Map plugin for JSON-UI using Leaflet 1.9.4.
//!
//! Renders interactive maps from JSON props. Each map container stores its
//! configuration in a `data-ferro-map` attribute; a single init script
//! discovers all containers on the page and initializes Leaflet maps.

use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::plugin::{Asset, JsonUiPlugin};
use crate::render::html_escape;

/// Default zoom level for maps.
fn default_zoom() -> u8 {
    13
}

/// Default height for the map container.
fn default_height() -> String {
    "400px".to_string()
}

/// Typed props for the Map component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapProps {
    /// Map center as `[lat, lng]`. Optional when using `fit_bounds`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub center: Option<[f64; 2]>,
    /// Zoom level (default: 13).
    #[serde(default = "default_zoom")]
    pub zoom: u8,
    /// CSS height of the container (default: "400px").
    #[serde(default = "default_height")]
    pub height: String,
    /// Auto-zoom to fit all markers. When true, center/zoom are ignored if markers exist.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fit_bounds: Option<bool>,
    /// Markers to place on the map.
    #[serde(default)]
    pub markers: Vec<MapMarker>,
    /// Custom tile layer URL template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tile_url: Option<String>,
    /// Tile layer attribution string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
    /// Maximum zoom level for the tile layer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_zoom: Option<u8>,
}

/// A marker on the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapMarker {
    /// Latitude.
    pub lat: f64,
    /// Longitude.
    pub lng: f64,
    /// Optional popup content (plain text).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub popup: Option<String>,
    /// Hex color for DivIcon pin (e.g., "#3B82F6"). When set, renders as colored CSS pin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// HTML content for popup (alternative to plain text popup).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub popup_html: Option<String>,
    /// URL to navigate to on marker click.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

/// Framework-served, self-hosted Leaflet (see `/_ferro/leaflet/*` in the server).
const DEFAULT_LEAFLET_BASE: &str = "/_ferro/leaflet";

/// Whether to opt back into the unpkg CDN via `FERRO_LEAFLET_CDN`.
fn cdn_opt_in() -> bool {
    matches!(
        std::env::var("FERRO_LEAFLET_CDN").ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

/// Resolve the Leaflet asset base.
///
/// - `Some(base)` → self-hosted `{base}/leaflet.{css,js}` (no SRI/crossorigin).
/// - `None` → the SRI-pinned unpkg CDN.
///
/// Precedence: an explicit `FERRO_LEAFLET_BASE` wins; else `FERRO_LEAFLET_CDN=1`
/// selects the CDN; else the **default** is the framework-served
/// `/_ferro/leaflet`, so the Map plugin works offline / behind TLS-terminating
/// proxies with zero configuration.
fn leaflet_base() -> Option<String> {
    if let Some(b) = std::env::var("FERRO_LEAFLET_BASE")
        .ok()
        .map(|s| s.trim_end_matches('/').to_string())
        .filter(|s| !s.is_empty())
    {
        return Some(b);
    }
    if cdn_opt_in() {
        return None;
    }
    Some(DEFAULT_LEAFLET_BASE.to_string())
}

/// Build a Leaflet asset: self-hosted `{base}/{file}` (no SRI) when a base is
/// configured, else the SRI-pinned CDN URL. Pure so the branch is unit-testable
/// without mutating the process environment.
fn leaflet_asset(base: Option<&str>, file: &str, cdn_url: &str, cdn_sri: &str) -> Asset {
    match base {
        Some(b) => Asset::new(format!("{b}/{file}")),
        None => Asset::new(cdn_url).integrity(cdn_sri).crossorigin(""),
    }
}

/// Global counter for unique map container IDs.
static MAP_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Leaflet 1.9.4 CDN base URL.
const LEAFLET_CSS_URL: &str = "https://unpkg.com/leaflet@1.9.4/dist/leaflet.css";
const LEAFLET_JS_URL: &str = "https://unpkg.com/leaflet@1.9.4/dist/leaflet.js";

/// SRI hashes for Leaflet 1.9.4.
const LEAFLET_CSS_SRI: &str = "sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY=";
const LEAFLET_JS_SRI: &str = "sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo=";

/// Map plugin using Leaflet 1.9.4.
///
/// Renders interactive maps from JSON props. Configuration is stored in
/// `data-ferro-map` attributes on container elements; a single init script
/// initializes all maps on the page.
pub struct MapPlugin;

impl JsonUiPlugin for MapPlugin {
    fn component_type(&self) -> &str {
        "Map"
    }

    fn props_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "description": "Interactive map component using Leaflet. Renders a map with configurable center, zoom, markers, and tile layer.",
            "required": [],
            "properties": {
                "center": {
                    "type": "array",
                    "description": "Map center as [latitude, longitude]. Optional when fit_bounds is true.",
                    "items": { "type": "number" },
                    "minItems": 2,
                    "maxItems": 2,
                    "examples": [[51.505, -0.09]]
                },
                "zoom": {
                    "type": "integer",
                    "description": "Initial zoom level (0-18)",
                    "default": 13,
                    "minimum": 0,
                    "maximum": 18
                },
                "height": {
                    "type": "string",
                    "description": "CSS height of the map container",
                    "default": "400px",
                    "examples": ["400px", "100vh", "600px"]
                },
                "fit_bounds": {
                    "type": "boolean",
                    "description": "Auto-zoom to fit all markers. When true, center/zoom are ignored if markers exist.",
                    "default": false
                },
                "markers": {
                    "type": "array",
                    "description": "Markers to display on the map",
                    "items": {
                        "type": "object",
                        "required": ["lat", "lng"],
                        "properties": {
                            "lat": {
                                "type": "number",
                                "description": "Marker latitude"
                            },
                            "lng": {
                                "type": "number",
                                "description": "Marker longitude"
                            },
                            "popup": {
                                "type": "string",
                                "description": "Optional popup text shown on marker click"
                            },
                            "color": {
                                "type": "string",
                                "description": "Hex color for DivIcon pin (e.g., '#3B82F6'). When set, renders as colored CSS pin instead of default marker."
                            },
                            "popup_html": {
                                "type": "string",
                                "description": "HTML content for popup. Takes priority over plain text popup."
                            },
                            "href": {
                                "type": "string",
                                "description": "URL to navigate to on marker click."
                            }
                        }
                    }
                },
                "tile_url": {
                    "type": "string",
                    "description": "Custom tile layer URL template. Defaults to OpenStreetMap.",
                    "examples": ["https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"]
                },
                "attribution": {
                    "type": "string",
                    "description": "Tile layer attribution text"
                },
                "max_zoom": {
                    "type": "integer",
                    "description": "Maximum zoom level for the tile layer",
                    "minimum": 0,
                    "maximum": 22
                }
            }
        })
    }

    fn render(&self, props: &Value, _data: &Value) -> String {
        let map_props: MapProps = match serde_json::from_value(props.clone()) {
            Ok(p) => p,
            Err(e) => {
                return format!(
                    "<div class=\"p-4 bg-red-50 text-red-600 rounded\">Map error: {}</div>",
                    html_escape(&e.to_string())
                );
            }
        };

        // Build the config JSON stored in the data attribute.
        let mut config = serde_json::json!({
            "zoom": map_props.zoom,
            "markers": map_props.markers,
            "tile_url": map_props.tile_url,
            "attribution": map_props.attribution,
            "max_zoom": map_props.max_zoom,
        });

        if let Some(center) = &map_props.center {
            config["center"] = serde_json::json!(center);
        }

        if let Some(true) = map_props.fit_bounds {
            config["fit_bounds"] = serde_json::json!(true);
        }

        let config_json = serde_json::to_string(&config).unwrap_or_default();
        let id = MAP_ID_COUNTER.fetch_add(1, Ordering::Relaxed);

        format!(
            "<div id=\"ferro-map-{}\" data-ferro-map='{}' style=\"height: {}; width: 100%;\"></div>",
            id,
            html_escape(&config_json),
            html_escape(&map_props.height),
        )
    }

    fn css_assets(&self) -> Vec<Asset> {
        vec![Asset::new(LEAFLET_CSS_URL)
            .integrity(LEAFLET_CSS_SRI)
            .crossorigin("")]
    }

    fn js_assets(&self) -> Vec<Asset> {
        vec![Asset::new(LEAFLET_JS_URL)
            .integrity(LEAFLET_JS_SRI)
            .crossorigin("")]
    }

    fn init_script(&self) -> Option<String> {
        Some(INIT_SCRIPT.to_string())
    }
}

/// Leaflet initialization script.
///
/// Discovers all `[data-ferro-map]` elements, parses their JSON config,
/// and creates Leaflet maps. Uses `IntersectionObserver` to handle maps
/// inside hidden containers (tabs, modals).
const INIT_SCRIPT: &str = r#"
(function() {
  if (!document.getElementById('ferro-map-pin-css')) {
    var s = document.createElement('style');
    s.id = 'ferro-map-pin-css';
    s.textContent = '.poi-marker{background:transparent;border:none;}.marker-pin{width:30px;height:30px;border-radius:50% 50% 50% 0;position:absolute;transform:rotate(-45deg);left:50%;top:50%;margin:-15px 0 0 -15px;}.marker-pin::after{content:"";width:18px;height:18px;margin:6px 0 0 6px;background:rgba(255,255,255,0.4);position:absolute;border-radius:50%;}';
    document.head.appendChild(s);
  }
})();
document.addEventListener('DOMContentLoaded', function() {
  document.querySelectorAll('[data-ferro-map]').forEach(function(el) {
    try {
      var cfg = JSON.parse(el.getAttribute('data-ferro-map'));
      var map = L.map(el);
      el._leaflet_map = map;

      var tileUrl = cfg.tile_url || 'https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png';
      var attribution = cfg.attribution || '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>';
      var maxZoom = cfg.max_zoom || 19;

      L.tileLayer(tileUrl, {
        attribution: attribution,
        maxZoom: maxZoom
      }).addTo(map);

      var allMarkers = [];
      if (cfg.markers) {
        cfg.markers.forEach(function(m) {
          var opts = {};
          if (m.color) {
            opts.icon = L.divIcon({
              className: 'poi-marker',
              html: '<div class="marker-pin" style="background:' + m.color + '"></div>',
              iconSize: [30, 42],
              iconAnchor: [15, 42],
              popupAnchor: [0, -42]
            });
          }
          var marker = L.marker([m.lat, m.lng], opts).addTo(map);
          if (m.popup_html) {
            marker.bindPopup(m.popup_html);
          } else if (m.popup) {
            marker.bindPopup(m.popup);
          }
          if (m.href) {
            marker.on('click', function(e) {
              L.DomEvent.stopPropagation(e);
              window.location.href = m.href;
            });
          }
          allMarkers.push(marker);
        });
      }

      if (cfg.fit_bounds && allMarkers.length > 0) {
        map.fitBounds(L.featureGroup(allMarkers).getBounds(), { padding: [50, 50] });
      } else if (cfg.center) {
        map.setView(cfg.center, cfg.zoom || 13);
      } else {
        map.setView([0, 0], 2);
      }

      if (typeof IntersectionObserver !== 'undefined') {
        var observer = new IntersectionObserver(function(entries) {
          entries.forEach(function(entry) {
            if (entry.isIntersecting) {
              map.invalidateSize();
            }
          });
        });
        observer.observe(el);
      }
    } catch (e) {
      console.error('Ferro Map init error:', e);
    }
  });
});
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_data() -> Value {
        serde_json::json!({})
    }

    fn basic_props() -> Value {
        serde_json::json!({
            "center": [51.505, -0.09],
            "zoom": 13,
            "markers": [
                {"lat": 51.5, "lng": -0.09, "popup": "Hello"},
                {"lat": 51.51, "lng": -0.1}
            ]
        })
    }

    #[test]
    fn test_map_renders_container_with_data_attribute() {
        let plugin = MapPlugin;
        let html = plugin.render(&basic_props(), &empty_data());

        assert!(
            html.contains("data-ferro-map"),
            "output should contain data-ferro-map attribute"
        );
        assert!(
            html.contains("51.505"),
            "output should contain center latitude"
        );
        assert!(
            html.contains("-0.09"),
            "output should contain center longitude"
        );
        assert!(
            html.contains("style=\"height: 400px"),
            "output should use default 400px height"
        );
    }

    #[test]
    fn test_map_custom_height() {
        let plugin = MapPlugin;
        let props = serde_json::json!({
            "center": [40.7128, -74.0060],
            "height": "600px"
        });
        let html = plugin.render(&props, &empty_data());

        assert!(
            html.contains("style=\"height: 600px"),
            "output should use custom 600px height"
        );
    }

    #[test]
    fn test_map_with_markers() {
        let plugin = MapPlugin;
        let html = plugin.render(&basic_props(), &empty_data());

        // The data attribute should contain marker coordinates and popup.
        assert!(html.contains("51.5"), "should contain marker lat");
        assert!(html.contains("-0.09"), "should contain marker lng");
        assert!(html.contains("Hello"), "should contain popup text");
    }

    #[test]
    fn test_map_invalid_props_shows_error() {
        let plugin = MapPlugin;
        // center must be an array of numbers, not a string.
        let props = serde_json::json!({"center": "not-an-array"});
        let html = plugin.render(&props, &empty_data());

        assert!(
            html.contains("Map error:"),
            "should show error message for invalid props"
        );
        assert!(html.contains("bg-red-50"), "should use error styling");
        // Must not panic.
    }

    #[test]
    fn test_map_props_schema_valid() {
        let plugin = MapPlugin;
        let schema = plugin.props_schema();

        assert_eq!(schema["type"], "object", "schema type should be 'object'");
        assert!(
            schema["properties"].is_object(),
            "schema should have 'properties'"
        );
        assert!(
            schema["properties"]["center"].is_object(),
            "schema should describe 'center' property"
        );
        assert!(
            schema["properties"]["fit_bounds"].is_object(),
            "schema should describe 'fit_bounds' property"
        );
        assert_eq!(
            schema["required"],
            serde_json::json!([]),
            "no properties should be required"
        );
    }

    #[test]
    fn default_assets_are_self_hosted() {
        // By default the plugin serves Leaflet from the framework (no CDN, no
        // SRI). Skip on a machine that overrides the base/CDN via env so the
        // test never flakes.
        if std::env::var_os("FERRO_LEAFLET_BASE").is_some()
            || std::env::var_os("FERRO_LEAFLET_CDN").is_some()
        {
            return;
        }
        let plugin = MapPlugin;

        let css = plugin.css_assets();
        assert_eq!(css.len(), 1);
        assert_eq!(css[0].url, "/_ferro/leaflet/leaflet.css");
        assert!(
            css[0].integrity.is_none(),
            "self-hosted default must not carry an SRI hash"
        );

        let js = plugin.js_assets();
        assert_eq!(js.len(), 1);
        assert_eq!(js[0].url, "/_ferro/leaflet/leaflet.js");
        assert!(js[0].integrity.is_none());
    }

    #[test]
    fn cdn_opt_in_still_available() {
        // The pure builder still produces the SRI-pinned CDN asset when no base
        // is supplied — the path taken when FERRO_LEAFLET_CDN=1.
        let a = leaflet_asset(None, "leaflet.js", LEAFLET_JS_URL, LEAFLET_JS_SRI);
        assert_eq!(a.url, LEAFLET_JS_URL);
        assert!(a.integrity.as_deref().unwrap_or("").starts_with("sha256-"));
    }

    #[test]
    fn test_map_init_script_present() {
        let plugin = MapPlugin;
        let script = plugin.init_script();

        assert!(script.is_some(), "init_script should return Some");
        let script = script.unwrap();
        assert!(
            script.contains("DOMContentLoaded"),
            "script should listen for DOMContentLoaded"
        );
        assert!(
            script.contains("data-ferro-map"),
            "script should query data-ferro-map elements"
        );
        assert!(
            script.contains("IntersectionObserver"),
            "script should use IntersectionObserver"
        );
        assert!(
            script.contains("fitBounds"),
            "script should support fitBounds auto-zoom"
        );
        assert!(
            script.contains("featureGroup"),
            "script should use featureGroup for bounds calculation"
        );
    }

    #[test]
    fn test_map_component_type() {
        let plugin = MapPlugin;
        assert_eq!(plugin.component_type(), "Map");
    }

    #[test]
    fn test_map_unique_ids() {
        let plugin = MapPlugin;
        let props = serde_json::json!({"center": [0.0, 0.0]});

        let html1 = plugin.render(&props, &empty_data());
        let html2 = plugin.render(&props, &empty_data());

        // Extract IDs: they should differ.
        assert_ne!(html1, html2, "two renders should produce different IDs");
        assert!(
            html1.contains("ferro-map-"),
            "should have ferro-map- prefix"
        );
        assert!(
            html2.contains("ferro-map-"),
            "should have ferro-map- prefix"
        );
    }

    #[test]
    fn test_map_renders_without_center() {
        let plugin = MapPlugin;
        let props = serde_json::json!({
            "fit_bounds": true,
            "markers": [
                {"lat": 51.5, "lng": -0.09, "popup": "A"},
                {"lat": 51.51, "lng": -0.1, "popup": "B"}
            ]
        });
        let html = plugin.render(&props, &empty_data());

        assert!(
            html.contains("data-ferro-map"),
            "should render map container without center"
        );
        assert!(
            !html.contains("Map error:"),
            "should not show error when center is omitted with fit_bounds"
        );
        assert!(
            html.contains("fit_bounds"),
            "config should contain fit_bounds"
        );
    }
}
