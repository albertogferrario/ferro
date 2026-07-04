//! Browse archetype: template-bound item list.

use crate::actions::{action_button, crud_name};
use crate::builder::{display_fields, emit_title, Emit};
use crate::component::Component;
use crate::context::A2uiContext;
use ferro_projections::{Error, ServiceDef};
use ferro_theme::IntentSlotTemplate;

/// Emits the `items` List with a shared row template; returns its ID.
/// Bindings inside the template are relative (resolved per list item);
/// the contract records them as `/items/*/<field>`.
pub(crate) fn emit_item_list(e: &mut Emit, service: &ServiceDef) -> String {
    e.contract.bind("/items", None, None);
    e.push(Component::new("items", "List").template_children("/items", "item"));
    e.push(Component::new("item", "Card").child("item_row"));
    let mut row_children = Vec::new();
    for f in display_fields(service) {
        let id = format!("item_{}", f.name);
        e.contract.bind(
            format!("/items/*/{}", f.name),
            Some(f.data_type),
            Some(&f.name),
        );
        e.push(Component::new(id.clone(), "Text").bound_prop("text", f.name.clone()));
        row_children.push(id);
    }
    e.push(Component::new("item_row", "Row").children_ids(row_children));
    "items".to_string()
}

pub(crate) fn emit(
    e: &mut Emit,
    service: &ServiceDef,
    _ctx: &A2uiContext,
    template: &IntentSlotTemplate,
) -> Result<Vec<String>, Error> {
    let mut children = Vec::new();
    for slot in &template.slots {
        match slot.as_str() {
            "title" => children.push(emit_title(e, service)),
            "fields" => children.push(emit_item_list(e, service)),
            "pagination" => {
                let name = crud_name("list", service);
                children.push(action_button(
                    e,
                    "load_more",
                    "Load more",
                    &name,
                    serde_json::json!({}),
                ));
            }
            _ => {}
        }
    }
    Ok(children)
}

#[cfg(test)]
mod tests {
    use crate::context::A2uiContext;
    use crate::message::A2uiMessage;
    use crate::test_support::{order_service, scored};
    use crate::A2uiRenderer;
    use ferro_projections::render::Renderer;
    use ferro_projections::Intent;

    #[test]
    fn browse_emits_template_bound_list() {
        let out = A2uiRenderer
            .render(
                &order_service(),
                &scored(Intent::Browse),
                &A2uiContext::default(),
            )
            .unwrap();
        let A2uiMessage::CreateSurface(cs) = &out.messages[0] else {
            panic!()
        };
        let by_id = |id: &str| cs.components.iter().find(|c| c.id == id).unwrap();

        assert_eq!(
            by_id("root").props["children"],
            serde_json::json!(["title", "items", "load_more"])
        );
        let items = by_id("items");
        assert_eq!(items.component, "List");
        assert_eq!(
            items.props["children"],
            serde_json::json!({"path": "/items", "componentId": "item"})
        );
        assert_eq!(by_id("item").component, "Card");
        assert_eq!(by_id("item").props["child"], serde_json::json!("item_row"));
        // display fields: customer_name, total, status, notes — bound RELATIVE (template scope)
        assert_eq!(
            by_id("item_customer_name").props["text"],
            serde_json::json!({"path": "customer_name"})
        );
        assert_eq!(
            by_id("load_more").props["action"]["event"]["name"],
            serde_json::json!("list_order")
        );
    }

    #[test]
    fn browse_contract_lists_items_and_wildcard_fields() {
        let out = A2uiRenderer
            .render(
                &order_service(),
                &scored(Intent::Browse),
                &A2uiContext::default(),
            )
            .unwrap();
        let paths = out.data_contract.paths();
        for p in [
            "/items",
            "/items/*/customer_name",
            "/items/*/total",
            "/items/*/status",
            "/items/*/notes",
        ] {
            assert!(paths.contains(&p), "missing {p}");
        }
    }

    #[test]
    fn custom_intent_falls_back_to_browse() {
        let out = A2uiRenderer
            .render(
                &order_service(),
                &scored(Intent::Custom("inbox".into())),
                &A2uiContext::default(),
            )
            .unwrap();
        let A2uiMessage::CreateSurface(cs) = &out.messages[0] else {
            panic!()
        };
        assert!(cs.components.iter().any(|c| c.id == "items"));
    }
}
