//! CLI summary sketch renderer (Process-intent anchor).
//!
//! # Research sketch — not stable API

use crate::error::Error;
use crate::intent::IntentScore;
use crate::service::ServiceDef;

use super::super::{field_display_name, is_system_field, BaseContext, Renderer};

// Research sketch — not stable API
pub(crate) struct CliSummaryRenderer;

impl Renderer for CliSummaryRenderer {
    type Output = String;
    type Context = BaseContext;

    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &BaseContext,
    ) -> Result<String, Error> {
        use std::fmt::Write;

        let title = service.display_name.as_deref().unwrap_or(&service.name);
        let intent_label = intents
            .get(ctx.intent_index)
            .map(|s| format!("{:?}", s.intent).to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());

        let mut out = String::new();
        let _ = writeln!(out, "{title} [{intent_label}]");

        // Domain fields (drop system fields).
        let _ = writeln!(out, "Fields:");
        for f in &service.fields {
            if !is_system_field(&f.meaning) {
                let _ = writeln!(out, "  - {} ({:?})", field_display_name(&f.name), f.meaning);
            }
        }

        // State machine: list states + the initial; CLI summary mentions state names.
        if let Some(sm) = &service.state_machine {
            let _ = writeln!(out, "States (initial: {}):", sm.initial_state);
            for s in &sm.states {
                let label = s.display_name.as_deref().unwrap_or(&s.name);
                let marker = if s.is_final { " [final]" } else { "" };
                let _ = writeln!(out, "  - {} ({}){marker}", label, s.name);
            }
        }

        // Actions with their transition targets.
        if !service.actions.is_empty() {
            let _ = writeln!(out, "Actions:");
            for a in &service.actions {
                let label = a.display_name.as_deref().unwrap_or(&a.name);
                let _ = writeln!(out, "  - {} ({})", label, a.name);
            }
        }

        Ok(out)
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
    fn cli_summary_non_trivial_output() {
        let svc = approval_workflow_fixture();
        let intents = derive_intents(&svc);
        let renderer = CliSummaryRenderer;
        let result = renderer
            .render(&svc, &intents, &BaseContext::default())
            .expect("render must succeed");
        assert!(!result.is_empty(), "output must not be empty");
        assert!(
            result.contains("draft") || result.contains("submitted") || result.contains("approved"),
            "output must mention at least one state name; got: {result}"
        );
    }
}
