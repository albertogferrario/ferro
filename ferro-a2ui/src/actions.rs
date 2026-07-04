//! Action-event mapping: `ActionDef` and CRUD verbs to A2UI action props.
//!
//! Event names reuse the MCP tool vocabulary (`create_<svc>`, `mark_paid`, …)
//! so surfaces and tools share one dispatch layer.

// Consumed by the archetype builders (Tasks 8–14); removed when dispatch lands.
#![allow(dead_code)]

use crate::builder::Emit;
use crate::component::Component;
use ferro_projections::{ActionDef, ServiceDef};
use serde_json::Value;
use std::collections::HashMap;

/// Returns `true` if the action should render. Hidden only when any
/// precondition maps to an explicit `false` in `evaluated`.
pub(crate) fn action_passes_guards(action: &ActionDef, evaluated: &HashMap<String, bool>) -> bool {
    action
        .preconditions
        .iter()
        .all(|g| evaluated.get(g.as_str()).copied().unwrap_or(true))
}

/// Builds a server-event action prop value.
pub fn event(name: &str, context: Value, want_response: bool) -> Value {
    serde_json::json!({"event": {"name": name, "context": context, "wantResponse": want_response}})
}

/// Emits a `Button` (with label `Text` child) firing `event_name`.
pub(crate) fn action_button(
    e: &mut Emit,
    id: &str,
    label: &str,
    event_name: &str,
    context: Value,
) -> String {
    let label_id = format!("{id}_label");
    e.push(
        Component::new(id, "Button")
            .child(label_id.clone())
            .action(event(event_name, context, true)),
    );
    e.push(Component::new(label_id, "Text").prop("text", label));
    id.to_string()
}

/// Emits a confirmation `Modal` for a destructive action. The confirm button
/// fires `event_name` with `confirmed: true` merged into `context`.
pub(crate) fn confirm_modal(
    e: &mut Emit,
    id: &str,
    trigger_label: &str,
    event_name: &str,
    context: Value,
) -> String {
    let mut confirm_context = context;
    confirm_context
        .as_object_mut()
        .expect("action context must be a JSON object")
        .insert("confirmed".into(), Value::Bool(true));

    let modal_id = format!("{id}_modal");
    let trigger_id = format!("{id}_trigger");
    let trigger_label_id = format!("{id}_trigger_label");
    let content_id = format!("{id}_content");
    let warning_id = format!("{id}_warning");
    let confirm_id = format!("{id}_confirm");
    let confirm_label_id = format!("{id}_confirm_label");

    e.push(
        Component::new(modal_id.clone(), "Modal")
            .prop("entryPointChild", trigger_id.clone())
            .prop("contentChild", content_id.clone()),
    );
    e.push(Component::new(trigger_id, "Button").child(trigger_label_id.clone()));
    e.push(Component::new(trigger_label_id, "Text").prop("text", trigger_label));
    e.push(
        Component::new(content_id, "Column").children_ids([warning_id.clone(), confirm_id.clone()]),
    );
    e.push(Component::new(warning_id, "Text").prop("text", "This action cannot be undone."));
    e.push(
        Component::new(confirm_id, "Button")
            .child(confirm_label_id.clone())
            .action(event(event_name, confirm_context, true)),
    );
    e.push(Component::new(confirm_label_id, "Text").prop("text", "Confirm"));
    modal_id
}

/// CRUD event name matching the derived MCP tool name exactly.
pub fn crud_name(verb: &str, service: &ServiceDef) -> String {
    format!("{verb}_{}", service.name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::Emit;
    use crate::test_support::order_service;
    use ferro_projections::ActionDef;
    use std::collections::HashMap;

    #[test]
    fn guard_filtering_hides_only_explicit_false() {
        let a = ActionDef::new("mark_paid").precondition("is_manager");
        assert!(action_passes_guards(&a, &HashMap::new()));
        assert!(action_passes_guards(
            &a,
            &HashMap::from([("is_manager".into(), true)])
        ));
        assert!(!action_passes_guards(
            &a,
            &HashMap::from([("is_manager".into(), false)])
        ));
    }

    #[test]
    fn event_wire_shape() {
        let v = event(
            "mark_paid",
            serde_json::json!({"id": {"path": "/entity/id"}}),
            true,
        );
        assert_eq!(
            v,
            serde_json::json!({"event": {
                "name": "mark_paid",
                "context": {"id": {"path": "/entity/id"}},
                "wantResponse": true
            }})
        );
    }

    #[test]
    fn action_button_emits_button_and_label() {
        let mut e = Emit::default();
        let id = action_button(
            &mut e,
            "mark_paid_btn",
            "Mark Paid",
            "mark_paid",
            serde_json::json!({}),
        );
        assert_eq!(id, "mark_paid_btn");
        assert_eq!(e.components.len(), 2);
        let btn = &e.components[0];
        assert_eq!(btn.component, "Button");
        assert_eq!(btn.props["child"], serde_json::json!("mark_paid_btn_label"));
        assert_eq!(
            btn.props["action"]["event"]["name"],
            serde_json::json!("mark_paid")
        );
        assert_eq!(
            e.components[1].props["text"],
            serde_json::json!("Mark Paid")
        );
    }

    #[test]
    fn confirm_modal_wraps_trigger_and_confirm() {
        let mut e = Emit::default();
        let id = confirm_modal(
            &mut e,
            "delete",
            "Delete",
            "delete_order",
            serde_json::json!({"id": {"path": "/entity/id"}}),
        );
        assert_eq!(id, "delete_modal");
        let modal = e
            .components
            .iter()
            .find(|c| c.id == "delete_modal")
            .unwrap();
        assert_eq!(modal.component, "Modal");
        assert_eq!(
            modal.props["entryPointChild"],
            serde_json::json!("delete_trigger")
        );
        assert_eq!(
            modal.props["contentChild"],
            serde_json::json!("delete_content")
        );
        let confirm = e
            .components
            .iter()
            .find(|c| c.id == "delete_confirm")
            .unwrap();
        assert_eq!(
            confirm.props["action"]["event"]["name"],
            serde_json::json!("delete_order")
        );
        assert_eq!(
            confirm.props["action"]["event"]["context"]["confirmed"],
            serde_json::json!(true)
        );
    }

    #[test]
    fn crud_names_match_mcp_tool_naming() {
        let svc = order_service();
        assert_eq!(crud_name("create", &svc), "create_order");
        assert_eq!(crud_name("update", &svc), "update_order");
        assert_eq!(crud_name("delete", &svc), "delete_order");
        assert_eq!(crud_name("list", &svc), "list_order");
    }
}
