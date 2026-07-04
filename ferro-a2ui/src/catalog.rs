//! Catalog tiers and identifiers.

/// The A2UI Basic catalog ID (open-source renderers ship this catalog).
/// Verified against the v1.0 RC spec tree:
/// <https://raw.githubusercontent.com/a2ui-project/a2ui/main/specification/v1_0/catalogs/basic/catalog.json>
pub const BASIC_CATALOG_ID: &str =
    "https://a2ui.org/specification/v1_0/catalogs/basic/catalog.json";

/// The ferro catalog ID (rich components; negotiated tier).
pub const FERRO_CATALOG_ID: &str = "https://ferro-rs.dev/a2ui/catalog/v1";

/// Basic-catalog component type names (v1.0 RC).
pub const BASIC_COMPONENTS: &[&str] = &[
    "Text",
    "Image",
    "Icon",
    "Video",
    "AudioPlayer",
    "Row",
    "Column",
    "List",
    "Card",
    "Tabs",
    "Modal",
    "Divider",
    "Button",
    "TextField",
    "CheckBox",
    "ChoicePicker",
    "Slider",
    "DateTimeInput",
];

/// Which catalog the renderer emits against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CatalogTier {
    /// Compose every archetype from Basic-catalog primitives (default).
    #[default]
    Basic,
    /// Emit rich ferro-catalog components (negotiated).
    Ferro,
}

impl CatalogTier {
    /// The catalog ID emitted in `createSurface`.
    pub fn catalog_id(&self) -> &'static str {
        match self {
            CatalogTier::Basic => BASIC_CATALOG_ID,
            CatalogTier::Ferro => FERRO_CATALOG_ID,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_components_are_18_unique_names() {
        assert_eq!(BASIC_COMPONENTS.len(), 18);
        let set: std::collections::HashSet<_> = BASIC_COMPONENTS.iter().collect();
        assert_eq!(set.len(), 18);
    }

    #[test]
    fn tier_maps_to_catalog_id() {
        assert_eq!(CatalogTier::Basic.catalog_id(), BASIC_CATALOG_ID);
        assert_eq!(CatalogTier::Ferro.catalog_id(), FERRO_CATALOG_ID);
        assert_eq!(CatalogTier::default(), CatalogTier::Basic);
    }
}
