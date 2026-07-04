//! Process archetype: state-machine lanes. Template mode binds lanes to the
//! data model; materialized mode builds guard-accurate per-record cards
//! (A2UI's conditional-UI mechanism: the server owns the component stream).

use crate::actions::{action_button, action_passes_guards};
use crate::builder::{display_fields, emit_title, Emit};
use crate::catalog::CatalogTier;
use crate::component::Component;
use crate::context::{A2uiContext, EmissionMode};
use ferro_projections::render::field_display_name;
use ferro_projections::{Error, FieldMeaning, ServiceDef, StateMachine};
use ferro_theme::IntentSlotTemplate;
use serde_json::Value;

pub(crate) fn emit(
    e: &mut Emit,
    service: &ServiceDef,
    ctx: &A2uiContext,
    template: &IntentSlotTemplate,
) -> Result<Vec<String>, Error> {
    let machine = service
        .state_machine
        .as_ref()
        .ok_or_else(|| Error::Render("process intent requires a state machine".into()))?;
    let mode = ctx.emission_mode.unwrap_or(if ctx.records.is_some() {
        EmissionMode::Materialized
    } else {
        EmissionMode::Template
    });

    let mut children = Vec::new();
    for slot in &template.slots {
        match slot.as_str() {
            "title" => children.push(emit_title(e, service)),
            "body" => children.push(match mode {
                // Materialized emission stays a Basic composition even at the
                // Ferro tier — guard-accurate per-record cards need explicit
                // components, not a data-bound board.
                EmissionMode::Template if ctx.tier == CatalogTier::Ferro => emit_kanban_board(e),
                EmissionMode::Template => emit_template_lanes(e, service, machine),
                EmissionMode::Materialized => emit_materialized_lanes(e, service, ctx, machine),
            }),
            _ => {}
        }
    }
    Ok(children)
}

/// Ferro-tier template body: a `KanbanBoard` bound to `/lanes`.
fn emit_kanban_board(e: &mut Emit) -> String {
    e.contract.bind("/lanes", None, None);
    e.push(Component::new("lanes", "KanbanBoard").bound_prop("lanes", "/lanes"));
    "lanes".to_string()
}

fn emit_template_lanes(e: &mut Emit, service: &ServiceDef, machine: &StateMachine) -> String {
    // Shared row template, bound relative; recorded once with wildcards.
    for f in display_fields(service) {
        e.contract.bind(
            format!("/lanes/*/items/*/{}", f.name),
            Some(f.data_type),
            Some(&f.name),
        );
    }
    let mut row_children = Vec::new();
    for f in display_fields(service) {
        let id = format!("lane_card_{}", f.name);
        e.push(Component::new(id.clone(), "Text").bound_prop("text", f.name.clone()));
        row_children.push(id);
    }
    e.push(Component::new("lane_card_row", "Row").children_ids(row_children));
    e.push(Component::new("lane_card", "Card").child("lane_card_row"));

    let mut lane_ids = Vec::new();
    for (i, state) in machine.states.iter().enumerate() {
        let lane_id = format!("lane_{}", state.name);
        let title_id = format!("lane_{}_title", state.name);
        let items_id = format!("lane_{}_items", state.name);
        let items_path = format!("/lanes/{i}/items");
        e.contract.bind(items_path.clone(), None, None);
        let label = state
            .display_name
            .clone()
            .unwrap_or_else(|| field_display_name(&state.name));
        e.push(Component::new(title_id.clone(), "Text").prop("text", label));
        e.push(Component::new(items_id.clone(), "List").template_children(items_path, "lane_card"));
        e.push(Component::new(lane_id.clone(), "Column").children_ids([title_id, items_id]));
        lane_ids.push(lane_id);
    }
    e.push(Component::new("lanes", "Row").children_ids(lane_ids));
    "lanes".to_string()
}

fn state_field_name(service: &ServiceDef) -> String {
    service
        .fields
        .iter()
        .find(|f| f.meaning == FieldMeaning::Status)
        .map(|f| f.name.clone())
        .unwrap_or_else(|| "status".to_string())
}

fn value_to_text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn record_allows(record: &Value, action_name: &str, fallback: bool) -> bool {
    match record.get("_allowed_actions").and_then(Value::as_array) {
        Some(list) => list.iter().any(|a| a.as_str() == Some(action_name)),
        None => fallback,
    }
}

fn emit_materialized_lanes(
    e: &mut Emit,
    service: &ServiceDef,
    ctx: &A2uiContext,
    machine: &StateMachine,
) -> String {
    let records = ctx.records.as_deref().unwrap_or(&[]);
    let state_field = state_field_name(service);
    let mut lane_ids = Vec::new();

    for state in &machine.states {
        let lane_id = format!("lane_{}", state.name);
        let title_id = format!("lane_{}_title", state.name);
        let label = state
            .display_name
            .clone()
            .unwrap_or_else(|| field_display_name(&state.name));
        e.push(Component::new(title_id.clone(), "Text").prop("text", label));

        let mut lane_children = vec![title_id];
        for (n, record) in records.iter().enumerate() {
            if record.get(&state_field).map(value_to_text).as_deref() != Some(state.name.as_str()) {
                continue;
            }
            let card_id = format!("card_{n}");
            let col_id = format!("card_{n}_col");
            let mut col_children = Vec::new();
            for f in display_fields(service) {
                let id = format!("card_{n}_{}", f.name);
                let text = record.get(&f.name).map(value_to_text).unwrap_or_default();
                e.push(Component::new(id.clone(), "Text").prop("text", text));
                col_children.push(id);
            }
            for action in &service.actions {
                let fallback = action_passes_guards(action, &ctx.base.evaluated_guards);
                if !record_allows(record, &action.name, fallback) {
                    continue;
                }
                let label = action
                    .display_name
                    .clone()
                    .unwrap_or_else(|| field_display_name(&action.name));
                let context =
                    serde_json::json!({"id": record.get("id").cloned().unwrap_or(Value::Null)});
                col_children.push(action_button(
                    e,
                    &format!("card_{n}_{}", action.name),
                    &label,
                    &action.name,
                    context,
                ));
            }
            e.push(Component::new(col_id.clone(), "Column").children_ids(col_children));
            e.push(Component::new(card_id.clone(), "Card").child(col_id));
            lane_children.push(card_id);
        }
        e.push(Component::new(lane_id.clone(), "Column").children_ids(lane_children));
        lane_ids.push(lane_id);
    }
    e.push(Component::new("lanes", "Row").children_ids(lane_ids));
    "lanes".to_string()
}

#[cfg(test)]
mod tests {
    use crate::context::A2uiContext;
    use crate::message::A2uiMessage;
    use crate::test_support::{order_service, scored};
    use crate::A2uiRenderer;
    use ferro_projections::render::Renderer;
    use ferro_projections::{Error, Intent, ServiceDef};

    fn render(ctx: &A2uiContext) -> (Vec<crate::component::Component>, crate::DataContract) {
        let out = A2uiRenderer
            .render(&order_service(), &scored(Intent::Process), ctx)
            .unwrap();
        let A2uiMessage::CreateSurface(cs) = &out.messages[0] else {
            panic!()
        };
        (cs.components.clone(), out.data_contract)
    }

    #[test]
    fn process_without_state_machine_errors() {
        let svc = ServiceDef::new("plain");
        let err = A2uiRenderer
            .render(&svc, &scored(Intent::Process), &A2uiContext::default())
            .unwrap_err();
        assert!(matches!(err, Error::Render(_)));
    }

    #[test]
    fn template_mode_emits_lane_per_state_with_no_actions() {
        let (cs, contract) = render(&A2uiContext::default());
        let by_id = |id: &str| cs.iter().find(|c| c.id == id).unwrap();
        assert_eq!(
            by_id("lanes").props["children"],
            serde_json::json!(["lane_new", "lane_paid", "lane_done"])
        );
        assert_eq!(
            by_id("lane_new_items").props["children"],
            serde_json::json!({"path": "/lanes/0/items", "componentId": "lane_card"})
        );
        assert!(
            !cs.iter().any(|c| c.component == "Button"),
            "template mode must emit no buttons"
        );
        assert!(contract.paths().contains(&"/lanes/0/items"));
        assert!(contract.paths().contains(&"/lanes/*/items/*/customer_name"));
    }

    #[test]
    fn ferro_tier_template_mode_emits_kanban_board() {
        let ctx = A2uiContext {
            tier: crate::CatalogTier::Ferro,
            ..Default::default()
        };
        let (cs, contract) = render(&ctx);
        let lanes = cs.iter().find(|c| c.id == "lanes").unwrap();
        assert_eq!(lanes.component, "KanbanBoard");
        assert_eq!(lanes.props["lanes"], serde_json::json!({"path": "/lanes"}));
        assert!(contract.paths().contains(&"/lanes"));
    }

    #[test]
    fn materialized_mode_emits_guard_accurate_cards() {
        let ctx = A2uiContext {
            records: Some(vec![
                serde_json::json!({"id": 1, "customer_name": "Ada", "status": "new", "_allowed_actions": ["mark_paid"]}),
                serde_json::json!({"id": 2, "customer_name": "Bob", "status": "paid", "_allowed_actions": []}),
            ]),
            ..Default::default()
        };
        let (cs, contract) = render(&ctx);
        let by_id = |id: &str| cs.iter().find(|c| c.id == id).unwrap();
        // record 0 sits in lane "new" and has exactly its allowed action
        assert_eq!(
            by_id("card_0_customer_name").props["text"],
            serde_json::json!("Ada")
        );
        assert!(cs.iter().any(|c| c.id == "card_0_mark_paid"));
        assert_eq!(
            by_id("card_0_mark_paid").props["action"]["event"]["context"]["id"],
            serde_json::json!(1)
        );
        // record 1: no allowed actions → no buttons
        assert!(!cs.iter().any(|c| c.id == "card_1_mark_paid"));
        assert!(!cs.iter().any(|c| c.id == "card_1_archive"));
        // fully materialized → nothing left to bind
        assert!(contract.paths().is_empty());
    }
}
