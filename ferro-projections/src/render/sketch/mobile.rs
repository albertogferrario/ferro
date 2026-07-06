//! Mobile card-list sketch renderer (Process-intent anchor).
//!
//! # Research sketch — not stable API

use serde_json::{json, Value};

use crate::error::Error;
use crate::intent::IntentScore;
use crate::service::ServiceDef;

use super::super::{field_display_name, is_system_field, BaseContext, Renderer};

// Research sketch — not stable API
pub(crate) struct MobileCardRenderer;

impl Renderer for MobileCardRenderer {
    type Output = serde_json::Value;
    type Context = BaseContext;

    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &BaseContext,
    ) -> Result<Value, Error> {
        let title = service.display_name.as_deref().unwrap_or(&service.name);
        let intent_label = intents
            .get(ctx.intent_index)
            .map(|s| format!("{:?}", s.intent).to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());

        let mut cards: Vec<Value> = Vec::new();

        // Header card: title + primary intent.
        cards.push(json!({
            "type": "header",
            "title": title,
            "intent": intent_label,
        }));

        // Field card: domain fields only (drop system fields).
        let fields: Vec<Value> = service
            .fields
            .iter()
            .filter(|f| !is_system_field(&f.meaning))
            .map(|f| {
                json!({
                    "label": field_display_name(&f.name),
                    "name": f.name,
                    "meaning": f.meaning,
                })
            })
            .collect();
        if !fields.is_empty() {
            cards.push(json!({ "type": "fields", "items": fields }));
        }

        // Status card from the state machine.
        if let Some(sm) = &service.state_machine {
            let states: Vec<Value> = sm
                .states
                .iter()
                .map(|s| {
                    json!({
                        "name": s.name,
                        "label": s.display_name.as_deref().unwrap_or(&s.name),
                        "is_final": s.is_final,
                    })
                })
                .collect();
            cards.push(json!({
                "type": "status",
                "initial_state": sm.initial_state,
                "states": states,
            }));
        }

        // Action card: one tappable action per workflow action.
        let actions: Vec<Value> = service
            .actions
            .iter()
            .map(|a| {
                json!({
                    "name": a.name,
                    "label": a.display_name.as_deref().unwrap_or(&a.name),
                })
            })
            .collect();
        if !actions.is_empty() {
            cards.push(json!({ "type": "actions", "items": actions }));
        }

        Ok(json!({
            "intent": intent_label,
            "service": title,
            "cards": cards,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{ActionDef, GuardDef};
    use crate::derive::derive_intents;
    use crate::field::{DataType, FieldMeaning};
    use crate::service::ServiceDef;
    use crate::state::{StateDef, StateMachine, Transition};

    fn approval_workflow_fixture() -> ServiceDef {
        ServiceDef::new("approval_workflow")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("title", DataType::String, FieldMeaning::EntityName)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("amount", DataType::Float, FieldMeaning::Money)
            .guard(GuardDef::new("has_required_fields"))
            .guard(GuardDef::new("is_approver"))
            .guard(GuardDef::new("is_cancellable"))
            .state_machine(
                StateMachine::new("approval_lifecycle")
                    .initial("draft")
                    .state(StateDef::new("draft"))
                    .state(StateDef::new("submitted"))
                    .state(StateDef::new("approved").final_state())
                    .state(StateDef::new("rejected").final_state())
                    .state(StateDef::new("cancelled").final_state())
                    .transition(
                        Transition::new("draft", "submit", "submitted")
                            .guard("has_required_fields"),
                    )
                    .transition(
                        Transition::new("submitted", "approve", "approved").guard("is_approver"),
                    )
                    .transition(
                        Transition::new("submitted", "reject", "rejected").guard("is_approver"),
                    )
                    .transition(
                        Transition::new("draft", "cancel", "cancelled").guard("is_cancellable"),
                    )
                    .transition(
                        Transition::new("submitted", "cancel", "cancelled").guard("is_cancellable"),
                    ),
            )
            .action(
                ActionDef::new("submit")
                    .precondition("has_required_fields")
                    .transition_trigger("submit"),
            )
            .action(
                ActionDef::new("approve")
                    .precondition("is_approver")
                    .transition_trigger("approve"),
            )
            .action(
                ActionDef::new("reject")
                    .precondition("is_approver")
                    .transition_trigger("reject"),
            )
            .action(
                ActionDef::new("cancel")
                    .precondition("is_cancellable")
                    .transition_trigger("cancel"),
            )
    }

    #[test]
    fn mobile_card_non_trivial_output() {
        let svc = approval_workflow_fixture();
        let intents = derive_intents(&svc);
        let renderer = MobileCardRenderer;
        let result = renderer
            .render(&svc, &intents, &BaseContext::default())
            .expect("render must succeed");
        let cards = result
            .get("cards")
            .and_then(|c| c.as_array())
            .expect("output must have a 'cards' array");
        assert!(!cards.is_empty(), "card array must not be empty");
    }
}
