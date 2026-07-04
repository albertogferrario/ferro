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

/// The ferro catalog definition: rich components a client can negotiate.
///
/// Clients that declare this catalog receive `DataTable`, `KanbanBoard`,
/// `StatCard`, `LineChart`, `BarChart`, and `Timeline` components; clients
/// without it receive Basic-tier compositions of the same data instead.
pub fn ferro_catalog() -> serde_json::Value {
    serde_json::json!({
        "catalogId": FERRO_CATALOG_ID,
        "instructions": "The ferro catalog defines rich data-display components \
            emitted by the ferro projection renderer. Each component receives its \
            data through JSON Pointer bindings ({\"path\": \"/...\"}) into the \
            surface data model; the server lists every bound path in the surface's \
            data contract. DataTable renders a bound row list with named columns. \
            KanbanBoard renders bound [{title, items}] lanes. StatCard renders one \
            pre-formatted aggregate value with a label. LineChart and BarChart \
            render bound [{name, points: [[x, y]]}] series. Timeline renders bound \
            [{time, text, icon?}] events. Clients that do not declare this catalog \
            receive Basic-catalog compositions of the same data instead.",
        "components": {
            "DataTable": {
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "rows": {"type": "object", "description": "Binding to the row list"},
                    "columns": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["id", "rows", "columns"]
            },
            "KanbanBoard": {
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "lanes": {"type": "object", "description": "Binding to [{title, items}] lanes"}
                },
                "required": ["id", "lanes"]
            },
            "StatCard": {
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "value": {"type": "object", "description": "Binding to the formatted stat value"},
                    "label": {"type": "string"}
                },
                "required": ["id", "value", "label"]
            },
            "LineChart": {
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "series": {"type": "object", "description": "Binding to [{name, points: [[x, y]]}]"}
                },
                "required": ["id", "series"]
            },
            "BarChart": {
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "series": {"type": "object", "description": "Binding to [{name, points: [[x, y]]}]"}
                },
                "required": ["id", "series"]
            },
            "Timeline": {
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "events": {"type": "object", "description": "Binding to [{time, text, icon?}] events"}
                },
                "required": ["id", "events"]
            }
        }
    })
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

    #[test]
    fn ferro_catalog_declares_every_ferro_tier_component() {
        let cat = ferro_catalog();
        assert_eq!(cat["catalogId"], serde_json::json!(FERRO_CATALOG_ID));
        assert!(cat["instructions"].as_str().unwrap().len() > 100);
        let comps = cat["components"].as_object().unwrap();
        for name in [
            "DataTable",
            "KanbanBoard",
            "StatCard",
            "LineChart",
            "BarChart",
            "Timeline",
        ] {
            assert!(comps.contains_key(name), "missing schema for {name}");
            assert_eq!(comps[name]["type"], serde_json::json!("object"));
        }
    }

    #[test]
    fn every_emitted_type_is_declared_in_a_catalog() {
        use crate::context::A2uiContext;
        use crate::message::A2uiMessage;
        use crate::test_support::{order_service, scored};
        use crate::A2uiRenderer;
        use ferro_projections::render::Renderer;
        use ferro_projections::Intent;

        let ferro_components = ferro_catalog();
        let ferro_names: std::collections::HashSet<&str> = ferro_components["components"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        let intents = [
            Intent::Browse,
            Intent::Focus,
            Intent::Collect,
            Intent::Process,
            Intent::Summarize,
            Intent::Analyze,
            Intent::Track,
        ];
        for intent in intents {
            let ctx = A2uiContext {
                tier: CatalogTier::Ferro,
                ..Default::default()
            };
            let out = A2uiRenderer
                .render(&order_service(), &scored(intent.clone()), &ctx)
                .unwrap();
            let A2uiMessage::CreateSurface(cs) = &out.messages[0] else {
                panic!()
            };
            for c in &cs.components {
                assert!(
                    BASIC_COMPONENTS.contains(&c.component.as_str())
                        || ferro_names.contains(c.component.as_str()),
                    "{intent:?}: emitted type {} not declared in any catalog",
                    c.component
                );
            }
        }
    }
}
