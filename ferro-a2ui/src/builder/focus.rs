//! Focus archetype: single-entity detail card with guard-filtered actions.

use crate::actions::{action_button, action_passes_guards, confirm_modal, crud_name};
use crate::builder::{display_fields, emit_title, Emit};
use crate::component::Component;
use crate::context::A2uiContext;
use ferro_projections::render::field_display_name;
use ferro_projections::{DataType, Error, ServiceDef};
use ferro_theme::IntentSlotTemplate;

pub(crate) fn emit(
    e: &mut Emit,
    service: &ServiceDef,
    ctx: &A2uiContext,
    template: &IntentSlotTemplate,
) -> Result<Vec<String>, Error> {
    let mut children = Vec::new();
    for slot in &template.slots {
        match slot.as_str() {
            "title" => children.push(emit_title(e, service)),
            "fields" => children.push(emit_detail(e, service)),
            "actions" => children.extend(emit_actions(e, service, ctx)),
            _ => {}
        }
    }
    Ok(children)
}

fn emit_detail(e: &mut Emit, service: &ServiceDef) -> String {
    let mut rows = Vec::new();
    for f in display_fields(service) {
        let row_id = format!("{}_row", f.name);
        let label_id = format!("{}_label", f.name);
        let value_id = format!("{}_value", f.name);
        let path = format!("/entity/{}", f.name);
        e.contract
            .bind(path.clone(), Some(f.data_type), Some(&f.name));
        e.push(Component::new(label_id.clone(), "Text").prop("text", field_display_name(&f.name)));
        e.push(Component::new(value_id.clone(), "Text").bound_prop("text", path));
        e.push(Component::new(row_id.clone(), "Row").children_ids([label_id, value_id]));
        rows.push(row_id);
    }
    e.push(Component::new("detail_fields", "Column").children_ids(rows));
    e.push(Component::new("detail", "Card").child("detail_fields"));
    "detail".to_string()
}

/// Contracts `/entity/id` and returns the action context binding it. The `id`
/// field is assumed present; when absent the binding is still emitted and the
/// host resolves it.
fn id_context(e: &mut Emit) -> serde_json::Value {
    e.contract
        .bind("/entity/id", Some(DataType::Integer), Some("id"));
    serde_json::json!({"id": {"path": "/entity/id"}})
}

fn emit_actions(e: &mut Emit, service: &ServiceDef, ctx: &A2uiContext) -> Vec<String> {
    let mut ids = Vec::new();
    for action in &service.actions {
        if !action_passes_guards(action, &ctx.base.evaluated_guards) {
            continue;
        }
        let label = action
            .display_name
            .clone()
            .unwrap_or_else(|| field_display_name(&action.name));
        let context = id_context(e);
        ids.push(action_button(
            e,
            &format!("{}_btn", action.name),
            &label,
            &action.name,
            context,
        ));
    }
    if service.deletable {
        let context = id_context(e);
        ids.push(confirm_modal(
            e,
            "delete",
            "Delete",
            &crud_name("delete", service),
            context,
        ));
    }
    ids
}

#[cfg(test)]
mod tests {
    use crate::context::A2uiContext;
    use crate::message::A2uiMessage;
    use crate::test_support::{order_service, scored};
    use crate::A2uiRenderer;
    use ferro_projections::render::Renderer;
    use ferro_projections::Intent;
    use std::collections::HashMap;

    fn components(ctx: &A2uiContext) -> Vec<crate::component::Component> {
        let out = A2uiRenderer
            .render(&order_service(), &scored(Intent::Focus), ctx)
            .unwrap();
        let A2uiMessage::CreateSurface(cs) = &out.messages[0] else {
            panic!()
        };
        cs.components.clone()
    }

    #[test]
    fn focus_emits_detail_rows_bound_to_entity_paths() {
        let cs = components(&A2uiContext::default());
        let by_id = |id: &str| cs.iter().find(|c| c.id == id).unwrap();
        assert_eq!(by_id("detail").component, "Card");
        assert_eq!(
            by_id("customer_name_label").props["text"],
            serde_json::json!("Customer Name")
        );
        assert_eq!(
            by_id("customer_name_value").props["text"],
            serde_json::json!({"path": "/entity/customer_name"})
        );
    }

    #[test]
    fn focus_actions_are_guard_filtered_and_delete_gets_modal() {
        // Unevaluated guards: both actions render, plus the delete modal.
        let cs = components(&A2uiContext::default());
        assert!(cs.iter().any(|c| c.id == "mark_paid_btn"));
        assert!(cs.iter().any(|c| c.id == "archive_btn"));
        assert!(cs.iter().any(|c| c.id == "delete_modal"));
        assert!(
            cs.iter().find(|c| c.id == "delete_confirm").unwrap().props["action"]["event"]["name"]
                == serde_json::json!("delete_order")
        );
        // is_manager = false hides mark_paid, keeps archive.
        let mut ctx = A2uiContext::default();
        ctx.base.evaluated_guards = HashMap::from([("is_manager".to_string(), false)]);
        let cs = components(&ctx);
        assert!(!cs.iter().any(|c| c.id == "mark_paid_btn"));
        assert!(cs.iter().any(|c| c.id == "archive_btn"));
    }
}
