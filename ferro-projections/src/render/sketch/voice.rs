//! Voice narration sketch renderer (Process-intent anchor).
//!
//! # Research sketch — not stable API

use crate::error::Error;
use crate::intent::IntentScore;
use crate::service::ServiceDef;

use super::super::{BaseContext, Renderer};

// Research sketch — not stable API
pub(crate) struct VoiceRenderer;

impl Renderer for VoiceRenderer {
    type Output = String;
    type Context = BaseContext;

    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &BaseContext,
    ) -> Result<String, Error> {
        let title = service.display_name.as_deref().unwrap_or(&service.name);

        // Narrate the current state if one is supplied; otherwise the initial state.
        let state_phrase = match (&ctx.current_state, &service.state_machine) {
            (Some(cur), _) => format!("The {title} is currently {cur}."),
            (None, Some(sm)) => {
                format!("The {title} starts in the {} state.", sm.initial_state)
            }
            (None, None) => format!("This is the {title}."),
        };

        // List the available action verbs as a spoken sentence.
        let verbs: Vec<&str> = service
            .actions
            .iter()
            .map(|a| a.display_name.as_deref().unwrap_or(a.name.as_str()))
            .collect();

        let action_phrase = match verbs.len() {
            0 => "There are no actions you can take.".to_string(),
            1 => format!("You can {}.", verbs[0]),
            _ => {
                let (last, head) = verbs.split_last().unwrap();
                format!("You can {} or {}.", head.join(", "), last)
            }
        };

        let _ = intents.get(ctx.intent_index); // intent-aware hook; prose is intent-agnostic here

        Ok(format!("{state_phrase} {action_phrase}"))
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
    fn voice_non_trivial_output() {
        let svc = approval_workflow_fixture();
        let intents = derive_intents(&svc);
        let renderer = VoiceRenderer;
        let result = renderer
            .render(&svc, &intents, &BaseContext::default())
            .expect("render must succeed");
        assert!(!result.is_empty(), "output must not be empty");
        assert!(
            result.contains("submit")
                || result.contains("approve")
                || result.contains("reject")
                || result.contains("cancel"),
            "voice output must mention at least one action verb; got: {result}"
        );
    }
}
