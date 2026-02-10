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
    /// Map center as `[lat, lng]`.
    pub center: [f64; 2],
    /// Zoom level (default: 13).
    #[serde(default = "default_zoom")]
    pub zoom: u8,
    /// CSS height of the container (default: "400px").
    #[serde(default = "default_height")]
    pub height: String,
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
            "required": ["center"],
            "properties": {
                "center": {
                    "type": "array",
                    "description": "Map center as [latitude, longitude]",
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
        let config = serde_json::json!({
            "center": map_props.center,
            "zoom": map_props.zoom,
            "markers": map_props.markers,
            "tile_url": map_props.tile_url,
            "attribution": map_props.attribution,
            "max_zoom": map_props.max_zoom,
        });

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
document.addEventListener('DOMContentLoaded', function() {
  document.querySelectorAll('[data-ferro-map]').forEach(function(el) {
    try {
      var cfg = JSON.parse(el.getAttribute('data-ferro-map'));
      var map = L.map(el).setView(cfg.center, cfg.zoom || 13);

      var tileUrl = cfg.tile_url || 'https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png';
      var attribution = cfg.attribution || '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>';
      var maxZoom = cfg.max_zoom || 19;

      L.tileLayer(tileUrl, {
        attribution: attribution,
        maxZoom: maxZoom
      }).addTo(map);

      if (cfg.markers) {
        cfg.markers.forEach(function(m) {
          var marker = L.marker([m.lat, m.lng]).addTo(map);
          if (m.popup) {
            marker.bindPopup(m.popup);
          }
        });
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
