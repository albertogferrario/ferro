//! Pure, serializable derivation of state-transition write plans.
//!
//! `derive_transition_plan` reads a declared [`StateMachine`](crate::StateMachine)
//! plus an [`ActionDef`](crate::ActionDef) and produces a [`TransitionPlan`] — the
//! transition facts (`from_states` → `event` → `to_state`, guard, effects) the
//! consumer runtime needs to execute a write. The plan is **data, not behavior**:
//! this crate touches no database and runs no I/O. The single source of truth for
//! the target state is `Transition.to`, so the consumer no longer hand-writes a
//! `match action_name => new_status`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::Transition;

/// A serializable description of a state-transition write, derived from the
/// declared `StateMachine` + `ActionDef`.
///
/// Carries no behavior — the consumer interprets it against a concrete entity.
/// `to_state` is sourced only from `Transition.to`, eliminating the duplicated
/// hand-written transition target on the write path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TransitionPlan {
    /// `ActionDef.name` that produced this plan.
    pub action: String,
    /// = `ActionDef.transition_trigger`; the `StateMachine` event name.
    pub event: String,
    /// Every `Transition.from` carrying this event (multi-source aware).
    pub from_states: Vec<String>,
    /// `Transition.to` — the single target. Replaces the hand-written `match`.
    pub to_state: String,
    /// `Transition.guard`, if any — re-checked LIVE at execution (EXEC-02).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
    /// `Transition.actions` ∪ `ActionDef.effects` — override-hook inputs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<String>,
}

/// Derives the [`TransitionPlan`] for `action_name` from the service's declared
/// state machine.
///
/// Pure: no database access, no async, no closures. Returns a typed `Err` for
/// every failure mode rather than panicking.
///
/// # Errors
/// - [`Error::Validation`](crate::Error::Validation) — no action named `action_name`.
/// - [`Error::NoTransitionTrigger`](crate::Error::NoTransitionTrigger) — the action
///   declares no `transition_trigger`.
/// - [`Error::NoStateMachine`](crate::Error::NoStateMachine) — the service has no
///   state machine.
/// - [`Error::UndeclaredTransition`](crate::Error::UndeclaredTransition) — the trigger
///   names an event no transition carries (the EXEC-04 drift condition).
/// - [`Error::AmbiguousTransition`](crate::Error::AmbiguousTransition) — the event fans
///   out to more than one target state.
pub fn derive_transition_plan(
    svc: &crate::ServiceDef,
    action_name: &str,
) -> Result<TransitionPlan, crate::Error> {
    let action = svc
        .actions
        .iter()
        .find(|a| a.name == action_name)
        .ok_or_else(|| crate::Error::Validation(format!("no action '{action_name}'")))?;

    let event = action
        .transition_trigger
        .as_deref()
        .ok_or_else(|| crate::Error::NoTransitionTrigger(action_name.to_string()))?;

    let sm = svc
        .state_machine
        .as_ref()
        .ok_or_else(|| crate::Error::NoStateMachine(svc.name.clone()))?;

    let matches: Vec<&Transition> = sm.states_for_event(event);
    if matches.is_empty() {
        return Err(crate::Error::UndeclaredTransition {
            action: action_name.to_string(),
            event: event.to_string(),
        });
    }

    let to_state = matches[0].to.clone();
    if matches.iter().any(|t| t.to != to_state) {
        return Err(crate::Error::AmbiguousTransition {
            event: event.to_string(),
        });
    }

    let from_states: Vec<String> = matches.iter().map(|t| t.from.clone()).collect();

    // Guard is per-transition; all matches share one event/target. Take the
    // first transition's guard. Dedup against action.preconditions happens in
    // the consumer runtime (Plan 02), not here.
    let guard = matches[0].guard.clone();

    // effects: union of the transition's actions and the action's effects,
    // order-preserving, deduped by string equality.
    let mut effects: Vec<String> = Vec::new();
    for e in matches[0].actions.iter().chain(action.effects.iter()) {
        if !effects.iter().any(|existing| existing == e) {
            effects.push(e.clone());
        }
    }

    Ok(TransitionPlan {
        action: action_name.to_string(),
        event: event.to_string(),
        from_states,
        to_state,
        guard,
        effects,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::ActionDef;
    use crate::state::{StateMachine, Transition};
    use crate::ServiceDef;

    /// Synthetic order service mirroring the app's EXEC-01 reference fixture.
    /// `cancel` is multi-source (draft + submitted → cancelled) to exercise
    /// the multi-source path. Reproduced inline — no dependency on the `app` crate.
    fn order_service() -> ServiceDef {
        let machine = StateMachine::new("order_lifecycle")
            .initial("draft")
            .transition(Transition::new("draft", "submit", "submitted").actions(vec!["log_submit"]))
            .transition(Transition::new("submitted", "approve", "approved").guard("is_manager"))
            .transition(Transition::new("submitted", "reject", "cancelled"))
            .transition(Transition::new("approved", "ship", "shipped"))
            .transition(Transition::new("shipped", "deliver", "delivered"))
            .transition(Transition::new("draft", "cancel", "cancelled"))
            .transition(Transition::new("submitted", "cancel", "cancelled"));

        ServiceDef::new("order")
            .state_machine(machine)
            .action(
                ActionDef::new("submit")
                    .transition_trigger("submit")
                    .effect("notify"),
            )
            .action(
                ActionDef::new("approve")
                    .transition_trigger("approve")
                    .precondition("is_manager"),
            )
            .action(ActionDef::new("ship").transition_trigger("ship"))
            .action(ActionDef::new("cancel").transition_trigger("cancel"))
    }

    #[test]
    fn derive_transition_plan() {
        let svc = order_service();
        let plan = super::derive_transition_plan(&svc, "submit").unwrap();
        assert_eq!(plan.action, "submit");
        assert_eq!(plan.event, "submit");
        assert_eq!(plan.to_state, "submitted");
        assert_eq!(plan.from_states, vec!["draft".to_string()]);
        assert_eq!(plan.guard, None);
        // transition.actions ∪ action.effects, order-preserving, deduped.
        assert_eq!(
            plan.effects,
            vec!["log_submit".to_string(), "notify".to_string()]
        );
    }

    #[test]
    fn derive_approve_carries_transition_guard() {
        let svc = order_service();
        let plan = super::derive_transition_plan(&svc, "approve").unwrap();
        assert_eq!(plan.to_state, "approved");
        assert_eq!(plan.guard, Some("is_manager".to_string()));
    }

    #[test]
    fn derive_multi_source_event() {
        let svc = order_service();
        let plan = super::derive_transition_plan(&svc, "cancel").unwrap();
        assert_eq!(plan.to_state, "cancelled");
        assert!(plan.from_states.contains(&"draft".to_string()));
        assert!(plan.from_states.contains(&"submitted".to_string()));
        assert_eq!(plan.from_states.len(), 2);
    }

    #[test]
    fn derive_no_trigger() {
        let svc = ServiceDef::new("order")
            .state_machine(StateMachine::new("sm").initial("draft"))
            .action(ActionDef::new("noop"));
        let err = super::derive_transition_plan(&svc, "noop").unwrap_err();
        assert!(matches!(err, crate::Error::NoTransitionTrigger(a) if a == "noop"));
    }

    #[test]
    fn derive_undeclared_trigger_errors() {
        let svc = ServiceDef::new("order")
            .state_machine(
                StateMachine::new("sm")
                    .initial("draft")
                    .transition(Transition::new("draft", "submit", "submitted")),
            )
            .action(ActionDef::new("bogus").transition_trigger("does_not_exist"));
        let err = super::derive_transition_plan(&svc, "bogus").unwrap_err();
        assert!(matches!(
            err,
            crate::Error::UndeclaredTransition { ref action, ref event }
                if action == "bogus" && event == "does_not_exist"
        ));
    }

    #[test]
    fn derive_no_state_machine() {
        let svc =
            ServiceDef::new("order").action(ActionDef::new("submit").transition_trigger("submit"));
        let err = super::derive_transition_plan(&svc, "submit").unwrap_err();
        assert!(matches!(err, crate::Error::NoStateMachine(name) if name == "order"));
    }

    #[test]
    fn derive_no_action() {
        let svc = order_service();
        let err = super::derive_transition_plan(&svc, "ghost").unwrap_err();
        assert!(matches!(err, crate::Error::Validation(_)));
    }

    #[test]
    fn derive_ambiguous_fan_out() {
        // Same event `split` fans out to two different targets per source.
        let svc = ServiceDef::new("order")
            .state_machine(
                StateMachine::new("sm")
                    .initial("a")
                    .transition(Transition::new("a", "split", "b"))
                    .transition(Transition::new("c", "split", "d")),
            )
            .action(ActionDef::new("split").transition_trigger("split"));
        let err = super::derive_transition_plan(&svc, "split").unwrap_err();
        assert!(matches!(err, crate::Error::AmbiguousTransition { event } if event == "split"));
    }

    #[test]
    fn transition_plan_serde_round_trip() {
        let svc = order_service();
        let plan = super::derive_transition_plan(&svc, "submit").unwrap();
        let json = serde_json::to_string(&plan).unwrap();
        let back: TransitionPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, back);
    }
}
