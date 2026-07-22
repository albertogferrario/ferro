//! Embedded Leaflet 1.9.4 assets.
//!
//! Served by the framework at `GET /_ferro/leaflet/*` so the Map plugin works
//! offline and behind TLS-terminating proxies with no external CDN and no SRI
//! integrity check to fail. Embedded at compile time — no runtime file I/O.
//!
//! Upstream: Leaflet 1.9.4, <https://leafletjs.com> — BSD-2-Clause,
//! © 2010-2023 Volodymyr Agafonkin. License vendored alongside the assets at
//! `ferro-json-ui/assets/leaflet/LICENSE`.

/// Leaflet 1.9.4 JavaScript.
pub const LEAFLET_JS: &str = include_str!("../../assets/leaflet/leaflet.js");

/// Leaflet 1.9.4 CSS. References `images/*` (served alongside at
/// `/_ferro/leaflet/images/…`), so default markers resolve without config.
pub const LEAFLET_CSS: &str = include_str!("../../assets/leaflet/leaflet.css");

/// Marker/layer images referenced by `leaflet.css`, keyed by filename.
pub const LEAFLET_IMAGES: &[(&str, &[u8])] = &[
    (
        "layers.png",
        include_bytes!("../../assets/leaflet/images/layers.png"),
    ),
    (
        "layers-2x.png",
        include_bytes!("../../assets/leaflet/images/layers-2x.png"),
    ),
    (
        "marker-icon.png",
        include_bytes!("../../assets/leaflet/images/marker-icon.png"),
    ),
    (
        "marker-icon-2x.png",
        include_bytes!("../../assets/leaflet/images/marker-icon-2x.png"),
    ),
    (
        "marker-shadow.png",
        include_bytes!("../../assets/leaflet/images/marker-shadow.png"),
    ),
];

/// Look up an embedded Leaflet image by filename (e.g. `"marker-icon.png"`).
pub fn leaflet_image(name: &str) -> Option<&'static [u8]> {
    LEAFLET_IMAGES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, b)| *b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn embedded_leaflet_is_present() {
        assert!(!LEAFLET_JS.is_empty(), "leaflet.js must be embedded");
        assert!(
            LEAFLET_CSS.contains(".leaflet-"),
            "leaflet.css must be embedded"
        );
        assert!(LEAFLET_JS.contains("1.9.4"), "expected Leaflet 1.9.4");
        assert_eq!(LEAFLET_IMAGES.len(), 5);
        assert!(leaflet_image("marker-icon.png").is_some());
        assert!(leaflet_image("nope.png").is_none());
    }
}
