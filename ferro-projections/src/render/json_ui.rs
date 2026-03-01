//! JSON-UI renderer producing ferro-json-ui/v1 component trees from service definitions.
//!
//! Implements the `Renderer` trait to translate `ServiceDef` + `IntentScore[]` into
//! a JSON view specification with layout strategies for each intent type.

use serde_json::{json, Value};

use crate::error::Error;
use crate::field::FieldMeaning;
use crate::intent::{Intent, IntentScore};
use crate::relationship::NavigationHint;
use crate::service::ServiceDef;

use super::field_map::{field_to_column, field_to_display, field_to_input};
use super::relationship_map::relationship_to_component;
use super::{field_display_name, is_system_field, RenderContext, RenderMode, Renderer};

/// JSON-UI renderer producing ferro-json-ui/v1 component trees.
///
/// Translates service definitions and scored intents into a JSON view specification
/// matching the ferro-json-ui/v1 schema. Each intent maps to a layout strategy
/// that composes field/relationship mapping functions into a component tree.
pub struct JsonUiRenderer;

impl Renderer for JsonUiRenderer {
    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &RenderContext,
    ) -> Result<Value, Error> {
        let intent_score = intents.get(ctx.intent_index).ok_or_else(|| {
            Error::Render(format!(
                "intent_index {} out of bounds (have {} intents)",
                ctx.intent_index,
                intents.len()
            ))
        })?;

        let components = match &intent_score.intent {
            Intent::Browse => match ctx.mode {
                RenderMode::Display => self.render_browse(service),
                RenderMode::Input => self.render_collect(service),
            },
            Intent::Focus => match ctx.mode {
                RenderMode::Display => self.render_focus(service),
                RenderMode::Input => self.render_collect(service),
            },
            Intent::Collect => self.render_collect(service),
            Intent::Summarize => {
                todo!("implemented in Plan 90-02 Task 2")
            }
            Intent::Process => {
                todo!("implemented in Plan 03")
            }
            Intent::Analyze => {
                todo!("implemented in Plan 03")
            }
            Intent::Track => {
                todo!("implemented in Plan 03")
            }
            Intent::Custom(_) => match ctx.mode {
                RenderMode::Display => self.render_focus(service),
                RenderMode::Input => self.render_collect(service),
            },
        };

        let title = service
            .display_name
            .as_deref()
            .unwrap_or(&service.name)
            .to_string();

        Ok(json!({
            "$schema": "ferro-json-ui/v1",
            "title": title,
            "components": components,
        }))
    }
}

impl JsonUiRenderer {
    /// Browse layout: filterable table with pagination.
    fn render_browse(&self, service: &ServiceDef) -> Vec<Value> {
        let columns: Vec<Value> = service
            .fields
            .iter()
            .filter(|f| f.readable && !is_system_field(&f.meaning))
            .map(field_to_column)
            .collect();

        let table = json!({
            "type": "Table",
            "key": format!("{}-table", service.name),
            "columns": columns,
            "data_path": "/data/items",
            "sortable": true,
        });

        let pagination = json!({
            "type": "Pagination",
            "key": format!("{}-pagination", service.name),
            "current_page": 1,
            "per_page": 25,
            "total": 0,
            "base_url": format!("/{}", service.name),
        });

        vec![table, pagination]
    }

    /// Focus layout: detail card with description list and relationship sections.
    fn render_focus(&self, service: &ServiceDef) -> Vec<Value> {
        let mut components = Vec::new();

        // Description list items from readable fields (skip Null results)
        let items: Vec<Value> = service
            .fields
            .iter()
            .filter(|f| f.readable && !is_system_field(&f.meaning))
            .filter_map(|f| {
                let display = field_to_display(f);
                if display.is_null() {
                    return None;
                }
                Some(json!({
                    "term": field_display_name(&f.name),
                    "detail_data_path": format!("/data/{}", f.name),
                }))
            })
            .collect();

        let description_list = json!({
            "type": "DescriptionList",
            "key": format!("{}-details", service.name),
            "items": items,
        });

        // Collect relationship components by hint type
        let mut tab_components = Vec::new();
        let mut inline_components = Vec::new();
        let mut nested_components = Vec::new();

        for rel in &service.relationships {
            if rel.navigation == NavigationHint::Hidden {
                continue;
            }
            let comp = relationship_to_component(rel, &service.name);
            if comp.is_null() {
                continue;
            }
            match rel.navigation {
                NavigationHint::Tab => tab_components.push(comp),
                NavigationHint::Inline | NavigationHint::Link => inline_components.push(comp),
                NavigationHint::Nested => nested_components.push(comp),
                NavigationHint::Hidden => {} // already filtered
            }
        }

        // Build card children: DescriptionList + inline/link relationship components
        let mut card_children = vec![description_list];
        card_children.extend(inline_components);

        let title = service
            .display_name
            .as_deref()
            .unwrap_or(&service.name)
            .to_string();

        let card = json!({
            "type": "Card",
            "key": format!("{}-card", service.name),
            "title": title,
            "children": card_children,
        });
        components.push(card);

        // Tabs component if any Tab relationships exist
        if !tab_components.is_empty() {
            let tabs = json!({
                "type": "Tabs",
                "key": format!("{}-tabs", service.name),
                "tabs": tab_components,
            });
            components.push(tabs);
        }

        // Nested table components below card
        components.extend(nested_components);

        components
    }

    /// Collect layout: data entry form with typed inputs.
    fn render_collect(&self, service: &ServiceDef) -> Vec<Value> {
        let inputs: Vec<Value> = service
            .fields
            .iter()
            .filter(|f| {
                // Skip auto-generated system fields
                if matches!(f.meaning, FieldMeaning::Identifier) && !f.writable {
                    return false;
                }
                if is_system_field(&f.meaning)
                    && matches!(f.meaning, FieldMeaning::CreatedAt | FieldMeaning::UpdatedAt)
                {
                    return false;
                }
                f.writable
            })
            .filter_map(|f| {
                let input = field_to_input(f);
                if input.is_null() {
                    return None;
                }
                Some(input)
            })
            .collect();

        let submit = json!({
            "type": "Button",
            "key": format!("{}-submit", service.name),
            "label": "Save",
            "variant": "default",
            "action_handler": format!("{}.store", service.name),
        });

        let mut children = inputs;
        children.push(submit);

        let form = json!({
            "type": "Form",
            "key": format!("{}-form", service.name),
            "action_handler": format!("{}.store", service.name),
            "method": "POST",
            "children": children,
        });

        vec![form]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::{DataType, FieldDef};
    use crate::intent::IntentScore;
    use crate::relationship::{Cardinality, RelationshipDef};
    use crate::service::ServiceDef;

    fn browse_intent() -> IntentScore {
        IntentScore {
            intent: Intent::Browse,
            confidence: 0.8,
            matching_signals: vec!["test".into()],
        }
    }

    fn focus_intent() -> IntentScore {
        IntentScore {
            intent: Intent::Focus,
            confidence: 0.7,
            matching_signals: vec!["test".into()],
        }
    }

    fn collect_intent() -> IntentScore {
        IntentScore {
            intent: Intent::Collect,
            confidence: 0.6,
            matching_signals: vec!["test".into()],
        }
    }

    fn custom_intent() -> IntentScore {
        IntentScore {
            intent: Intent::Custom("dashboard".into()),
            confidence: 0.5,
            matching_signals: vec!["test".into()],
        }
    }

    fn default_ctx() -> RenderContext {
        RenderContext::default()
    }

    fn input_ctx() -> RenderContext {
        RenderContext {
            intent_index: 0,
            current_state: None,
            mode: RenderMode::Input,
        }
    }

    fn order_service() -> ServiceDef {
        ServiceDef::new("order")
            .display_name("Order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("title", DataType::String, FieldMeaning::EntityName)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt)
    }

    // -- Schema and structure tests --

    #[test]
    fn browse_sets_schema_to_ferro_json_ui_v1() {
        let service = order_service();
        let intents = vec![browse_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        assert_eq!(result["$schema"], "ferro-json-ui/v1");
    }

    #[test]
    fn browse_sets_title_from_display_name() {
        let service = order_service();
        let intents = vec![browse_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        assert_eq!(result["title"], "Order");
    }

    #[test]
    fn browse_falls_back_to_service_name_without_display_name() {
        let service = ServiceDef::new("invoice")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("amount", DataType::Float, FieldMeaning::Money);
        let intents = vec![browse_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        assert_eq!(result["title"], "invoice");
    }

    // -- Browse tests --

    #[test]
    fn browse_produces_table_and_pagination() {
        let service = order_service();
        let intents = vec![browse_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        assert_eq!(components.len(), 2);
        assert_eq!(components[0]["type"], "Table");
        assert_eq!(components[1]["type"], "Pagination");
    }

    #[test]
    fn browse_excludes_system_fields_from_columns() {
        let service = order_service();
        let intents = vec![browse_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let columns = result["components"][0]["columns"].as_array().unwrap();
        let keys: Vec<&str> = columns.iter().map(|c| c["key"].as_str().unwrap()).collect();
        assert!(!keys.contains(&"id"));
        assert!(!keys.contains(&"created_at"));
        assert!(!keys.contains(&"updated_at"));
        assert!(keys.contains(&"title"));
        assert!(keys.contains(&"total"));
        assert!(keys.contains(&"status"));
    }

    #[test]
    fn browse_table_has_data_path_and_sortable() {
        let service = order_service();
        let intents = vec![browse_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        assert_eq!(result["components"][0]["data_path"], "/data/items");
        assert_eq!(result["components"][0]["sortable"], true);
    }

    #[test]
    fn browse_pagination_has_defaults() {
        let service = order_service();
        let intents = vec![browse_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let pagination = &result["components"][1];
        assert_eq!(pagination["current_page"], 1);
        assert_eq!(pagination["per_page"], 25);
        assert_eq!(pagination["total"], 0);
        assert_eq!(pagination["base_url"], "/order");
    }

    // -- Focus tests --

    #[test]
    fn focus_produces_card_with_description_list() {
        let service = order_service();
        let intents = vec![focus_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        assert!(!components.is_empty());
        assert_eq!(components[0]["type"], "Card");
        let children = components[0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "DescriptionList");
    }

    #[test]
    fn focus_excludes_sensitive_and_foreign_key_fields() {
        let service = ServiceDef::new("user")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("password", DataType::String, FieldMeaning::Sensitive)
            .field("org_id", DataType::Integer, FieldMeaning::ForeignKey);
        let intents = vec![focus_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let items = result["components"][0]["children"][0]["items"]
            .as_array()
            .unwrap();
        let terms: Vec<&str> = items.iter().map(|i| i["term"].as_str().unwrap()).collect();
        assert!(terms.contains(&"Name"));
        assert!(!terms.contains(&"Password"));
        assert!(!terms.contains(&"Org Id"));
        // System fields also excluded
        assert!(!terms.contains(&"Id"));
    }

    #[test]
    fn focus_includes_relationship_components_by_hint() {
        let service = ServiceDef::new("order")
            .display_name("Order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("title", DataType::String, FieldMeaning::EntityName)
            .relationship(
                RelationshipDef::new("line_items", "line_item", Cardinality::OneToMany)
                    .navigation(NavigationHint::Tab),
            )
            .relationship(
                RelationshipDef::new("items", "item", Cardinality::OneToMany)
                    .navigation(NavigationHint::Nested),
            )
            .relationship(
                RelationshipDef::new("customer", "customer", Cardinality::ManyToOne)
                    .navigation(NavigationHint::Inline),
            );
        let intents = vec![focus_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();

        // Card has DescriptionList + inline relationship
        let card_children = components[0]["children"].as_array().unwrap();
        assert!(card_children.len() >= 2);
        // Inline component in card children
        assert!(card_children
            .iter()
            .any(|c| c["type"] == "Text" && c["key"].as_str().unwrap().contains("inline")));

        // Tabs component
        assert!(components.iter().any(|c| c["type"] == "Tabs"));

        // Nested table
        assert!(components
            .iter()
            .any(|c| c["type"] == "Table" && c["key"].as_str().unwrap().contains("table")));
    }

    #[test]
    fn focus_hides_hidden_relationships() {
        let service = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("title", DataType::String, FieldMeaning::EntityName)
            .relationship(
                RelationshipDef::new("internal", "internal_ref", Cardinality::OneToOne)
                    .navigation(NavigationHint::Hidden),
            );
        let intents = vec![focus_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        // Only the card, no relationship components
        assert_eq!(components.len(), 1);
        assert_eq!(components[0]["type"], "Card");
    }

    // -- Error handling --

    #[test]
    fn render_returns_error_for_out_of_bounds_intent_index() {
        let service = order_service();
        let intents = vec![browse_intent()];
        let ctx = RenderContext {
            intent_index: 5,
            current_state: None,
            mode: RenderMode::Display,
        };
        let result = JsonUiRenderer.render(&service, &intents, &ctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("intent_index 5 out of bounds"));
    }

    // -- Custom intent fallback --

    #[test]
    fn custom_intent_falls_back_to_focus_layout() {
        let service = order_service();
        let intents = vec![custom_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        // Focus layout: Card with DescriptionList
        assert_eq!(components[0]["type"], "Card");
        let children = components[0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "DescriptionList");
    }

    // -- Mode transitions --

    #[test]
    fn browse_input_mode_renders_collect_form() {
        let service = order_service();
        let intents = vec![browse_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &input_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        assert_eq!(components[0]["type"], "Form");
    }

    #[test]
    fn focus_input_mode_renders_collect_form() {
        let service = order_service();
        let intents = vec![focus_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &input_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        assert_eq!(components[0]["type"], "Form");
    }

    // -- Collect tests --

    #[test]
    fn collect_produces_form_with_inputs() {
        let service = ServiceDef::new("user")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("email", DataType::String, FieldMeaning::Email)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt);
        let intents = vec![collect_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let components = result["components"].as_array().unwrap();
        assert_eq!(components[0]["type"], "Form");
        assert_eq!(components[0]["method"], "POST");
        assert_eq!(components[0]["action_handler"], "user.store");
    }

    #[test]
    fn collect_skips_auto_generated_system_fields() {
        let mut service = ServiceDef::new("user")
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("email", DataType::String, FieldMeaning::Email)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt);
        // Add read-only Identifier (auto-generated)
        service.fields.insert(
            0,
            FieldDef {
                name: "id".to_string(),
                data_type: DataType::Integer,
                meaning: FieldMeaning::Identifier,
                required: true,
                is_list: false,
                readable: true,
                writable: false,
            },
        );
        let intents = vec![collect_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let children = result["components"][0]["children"].as_array().unwrap();
        let names: Vec<&str> = children
            .iter()
            .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(!names.contains(&"id"));
        assert!(!names.contains(&"created_at"));
        assert!(!names.contains(&"updated_at"));
        assert!(names.contains(&"name"));
        assert!(names.contains(&"email"));
    }

    #[test]
    fn collect_includes_submit_button() {
        let service =
            ServiceDef::new("user").field("name", DataType::String, FieldMeaning::EntityName);
        let intents = vec![collect_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let children = result["components"][0]["children"].as_array().unwrap();
        let last = children.last().unwrap();
        assert_eq!(last["type"], "Button");
        assert_eq!(last["label"], "Save");
        assert_eq!(last["variant"], "default");
    }

    #[test]
    fn collect_maps_boolean_to_switch() {
        let service = ServiceDef::new("settings").field(
            "is_active",
            DataType::Boolean,
            FieldMeaning::Boolean,
        );
        let intents = vec![collect_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let children = result["components"][0]["children"].as_array().unwrap();
        // First child is the Switch, last is Button
        assert_eq!(children[0]["type"], "Switch");
        assert_eq!(children[0]["name"], "is_active");
    }

    #[test]
    fn collect_maps_email_to_email_input() {
        let service =
            ServiceDef::new("contact").field("email", DataType::String, FieldMeaning::Email);
        let intents = vec![collect_intent()];
        let result = JsonUiRenderer
            .render(&service, &intents, &default_ctx())
            .unwrap();
        let children = result["components"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "Input");
        assert_eq!(children[0]["input_type"], "email");
    }

    // -- Empty intent slice --

    #[test]
    fn render_returns_error_for_empty_intents() {
        let service = order_service();
        let intents: Vec<IntentScore> = vec![];
        let result = JsonUiRenderer.render(&service, &intents, &default_ctx());
        assert!(result.is_err());
    }
}
