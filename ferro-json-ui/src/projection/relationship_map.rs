//! Maps NavigationHint to JSON-UI component JSON for relationship rendering.
//!
//! Translates each of the 5 NavigationHint variants into a JSON-UI component
//! specification suitable for embedding in an intent layout.

use serde_json::{json, Value};

use ferro_projections::{NavigationHint, RelationshipDef};

use ferro_projections::render::field_display_name;

/// Maps a relationship to its component JSON based on NavigationHint.
///
/// Returns `Value::Null` for Hidden relationships.
pub fn relationship_to_component(rel: &RelationshipDef, service_name: &str) -> Value {
    let target = &rel.target;
    let target_label = field_display_name(target);

    match rel.navigation {
        NavigationHint::Inline => json!({
            "type": "Text",
            "content": "",
            "element": "span",
            "key": format!("{service_name}-{}-inline", rel.name),
            "label": target_label,
            "data_path": format!("/data/{}/{}", rel.name, "name"),
        }),
        NavigationHint::Link => json!({
            "type": "Button",
            "variant": "link",
            "label": format!("{target_label} \u{2192}"),
            "key": format!("{service_name}-{}-link", rel.name),
            "data_path": format!("/data/{}", rel.name),
        }),
        NavigationHint::Tab => json!({
            "tab_label": target_label,
            "tab_key": format!("{service_name}-{}-tab", rel.name),
            "content_placeholder": format!("/data/{}", rel.name),
        }),
        NavigationHint::Nested => json!({
            "type": "Table",
            "key": format!("{service_name}-{}-table", rel.name),
            "columns": [
                {"key": "name", "label": "Name"}
            ],
            "data_path": format!("/data/{}", rel.name),
        }),
        NavigationHint::Hidden => Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::Cardinality;

    fn make_rel(name: &str, target: &str, hint: NavigationHint) -> RelationshipDef {
        RelationshipDef::new(name, target, Cardinality::OneToMany).navigation(hint)
    }

    #[test]
    fn inline_produces_text_with_data_path() {
        let rel = make_rel("customer", "customer", NavigationHint::Inline);
        let result = relationship_to_component(&rel, "order");
        assert_eq!(result["type"], "Text");
        assert_eq!(result["data_path"], "/data/customer/name");
        assert_eq!(result["label"], "Customer");
        assert_eq!(result["key"], "order-customer-inline");
    }

    #[test]
    fn link_produces_button_with_arrow() {
        let rel = make_rel("customer", "customer", NavigationHint::Link);
        let result = relationship_to_component(&rel, "order");
        assert_eq!(result["type"], "Button");
        assert_eq!(result["variant"], "link");
        assert_eq!(result["label"], "Customer \u{2192}");
        assert_eq!(result["key"], "order-customer-link");
    }

    #[test]
    fn tab_produces_tab_metadata() {
        let rel = make_rel("line_items", "line_item", NavigationHint::Tab);
        let result = relationship_to_component(&rel, "order");
        assert_eq!(result["tab_label"], "Line Item");
        assert_eq!(result["tab_key"], "order-line_items-tab");
        assert_eq!(result["content_placeholder"], "/data/line_items");
    }

    #[test]
    fn nested_produces_table_with_columns() {
        let rel = make_rel("line_items", "line_item", NavigationHint::Nested);
        let result = relationship_to_component(&rel, "order");
        assert_eq!(result["type"], "Table");
        assert_eq!(result["data_path"], "/data/line_items");
        assert!(result["columns"].is_array());
        assert_eq!(result["columns"][0]["key"], "name");
    }

    #[test]
    fn hidden_returns_null() {
        let rel = make_rel("internal", "internal_ref", NavigationHint::Hidden);
        let result = relationship_to_component(&rel, "order");
        assert!(result.is_null());
    }
}
