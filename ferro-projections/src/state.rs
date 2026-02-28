use std::collections::{HashSet, VecDeque};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A state machine schema describing lifecycle states and transitions.
///
/// Schema-only: defines structure but does not execute transitions.
/// Guards and side effects are string references resolved externally.
///
/// ```
/// use ferro_projections::{StateMachine, StateDef, Transition};
///
/// let machine = StateMachine::new("order_lifecycle")
///     .initial("draft")
///     .state(StateDef::new("draft").display_name("Draft"))
///     .state(StateDef::new("completed").display_name("Completed").final_state())
///     .transition(Transition::new("draft", "complete", "completed"));
///
/// let warnings = machine.validate().unwrap();
/// assert!(warnings.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateMachine {
    /// Machine identifier (e.g., "order_lifecycle").
    pub name: String,
    /// Human-readable name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Description of what this state machine models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Name of the initial state (must exist in states).
    pub initial_state: String,
    /// State definitions.
    pub states: Vec<StateDef>,
    /// Transition definitions.
    pub transitions: Vec<Transition>,
}

/// A state within a state machine.
///
/// States can declare entry/exit side effects as string references
/// and carry optional metadata for rendering hints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateDef {
    /// State identifier (e.g., "draft", "pending", "completed").
    pub name: String,
    /// Human-readable name (e.g., "Pending Review").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Description of what this state means.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether this is a terminal state.
    #[serde(default)]
    pub is_final: bool,
    /// Side effects triggered on entering this state (string references).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub on_enter: Vec<String>,
    /// Side effects triggered on exiting this state (string references).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub on_exit: Vec<String>,
    /// Arbitrary metadata for rendering hints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// A transition between two states, triggered by an event.
///
/// Guards gate the transition; actions fire when the transition is taken.
/// Both are string references resolved externally.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Transition {
    /// Source state name.
    pub from: String,
    /// Event name that triggers this transition.
    pub event: String,
    /// Target state name.
    pub to: String,
    /// Guard condition (string reference, resolved at runtime).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
    /// Side effects executed when transition fires (string references).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<String>,
    /// Description of what this transition represents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Structural warnings from service and state machine validation.
///
/// Warnings indicate potential issues that may be intentional
/// (e.g., unreachable states reached via external means).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Warning {
    /// A state not reachable from the initial state via transitions.
    UnreachableState(String),
    /// A non-final state with no outgoing transitions.
    DeadEndState(String),
    /// No states are marked as final.
    NoFinalStates,
    /// A guard declared but never referenced by any transition or action precondition.
    UnusedGuard(String),
    /// An action has a transition_trigger but the service has no state machine.
    TransitionTriggerWithoutStateMachine(String),
}

impl StateMachine {
    /// Creates a new state machine with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: None,
            description: None,
            initial_state: String::new(),
            states: Vec::new(),
            transitions: Vec::new(),
        }
    }

    /// Sets the human-readable display name.
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Sets the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Sets the initial state name.
    pub fn initial(mut self, state: impl Into<String>) -> Self {
        self.initial_state = state.into();
        self
    }

    /// Adds a state definition.
    pub fn state(mut self, state: StateDef) -> Self {
        self.states.push(state);
        self
    }

    /// Adds a transition definition.
    pub fn transition(mut self, transition: Transition) -> Self {
        self.transitions.push(transition);
        self
    }

    /// Validates structural integrity and returns warnings for potential issues.
    ///
    /// Returns `Err` for fatal issues (missing initial state, invalid references).
    /// Returns `Ok(warnings)` for structural concerns (unreachable states, dead-ends).
    pub fn validate(&self) -> Result<Vec<Warning>, crate::Error> {
        let mut warnings = Vec::new();
        let state_names: HashSet<&str> = self.states.iter().map(|s| s.name.as_str()).collect();

        // 1. Initial state must be set
        if self.initial_state.is_empty() {
            return Err(crate::Error::Validation("initial state not set".into()));
        }

        // 2. Initial state must exist in states
        if !state_names.contains(self.initial_state.as_str()) {
            return Err(crate::Error::Validation(format!(
                "initial state '{}' not found in states",
                self.initial_state
            )));
        }

        // 3. All transition sources/targets must reference existing states
        for t in &self.transitions {
            if !state_names.contains(t.from.as_str()) {
                return Err(crate::Error::Validation(format!(
                    "transition source '{}' not found in states",
                    t.from
                )));
            }
            if !state_names.contains(t.to.as_str()) {
                return Err(crate::Error::Validation(format!(
                    "transition target '{}' not found in states",
                    t.to
                )));
            }
        }

        // 4. Reachability check (BFS from initial state)
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(self.initial_state.as_str());
        reachable.insert(self.initial_state.as_str());

        while let Some(current) = queue.pop_front() {
            for t in &self.transitions {
                if t.from == current && !reachable.contains(t.to.as_str()) {
                    reachable.insert(t.to.as_str());
                    queue.push_back(t.to.as_str());
                }
            }
        }

        for state in &self.states {
            if !reachable.contains(state.name.as_str()) {
                warnings.push(Warning::UnreachableState(state.name.clone()));
            }
        }

        // 5. Dead-end check (non-final states with no outgoing transitions)
        let states_with_outgoing: HashSet<&str> =
            self.transitions.iter().map(|t| t.from.as_str()).collect();
        for state in &self.states {
            if !state.is_final && !states_with_outgoing.contains(state.name.as_str()) {
                warnings.push(Warning::DeadEndState(state.name.clone()));
            }
        }

        // 6. No final states warning
        if !self.states.iter().any(|s| s.is_final) {
            warnings.push(Warning::NoFinalStates);
        }

        Ok(warnings)
    }

    /// Returns all transitions triggered by a given event.
    pub fn states_for_event(&self, event: &str) -> Vec<&Transition> {
        self.transitions
            .iter()
            .filter(|t| t.event == event)
            .collect()
    }

    /// Returns outgoing transitions from a state.
    pub fn events_from_state(&self, state: &str) -> Vec<&Transition> {
        self.transitions
            .iter()
            .filter(|t| t.from == state)
            .collect()
    }
}

impl StateDef {
    /// Creates a new state definition with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: None,
            description: None,
            is_final: false,
            on_enter: Vec::new(),
            on_exit: Vec::new(),
            metadata: None,
        }
    }

    /// Sets the human-readable display name.
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Sets the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Marks this state as a terminal (final) state.
    pub fn final_state(mut self) -> Self {
        self.is_final = true;
        self
    }

    /// Sets the entry side effects (string references).
    pub fn on_enter(mut self, effects: Vec<impl Into<String>>) -> Self {
        self.on_enter = effects.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the exit side effects (string references).
    pub fn on_exit(mut self, effects: Vec<impl Into<String>>) -> Self {
        self.on_exit = effects.into_iter().map(Into::into).collect();
        self
    }

    /// Sets arbitrary metadata for rendering hints.
    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl Transition {
    /// Creates a new transition from source state to target state on event.
    pub fn new(from: impl Into<String>, event: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            event: event.into(),
            to: to.into(),
            guard: None,
            actions: Vec::new(),
            description: None,
        }
    }

    /// Sets the guard condition (string reference).
    pub fn guard(mut self, guard: impl Into<String>) -> Self {
        self.guard = Some(guard.into());
        self
    }

    /// Sets the transition actions (string references).
    pub fn actions(mut self, actions: Vec<impl Into<String>>) -> Self {
        self.actions = actions.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_machine() -> StateMachine {
        StateMachine::new("order_lifecycle")
            .initial("draft")
            .state(StateDef::new("draft").display_name("Draft"))
            .state(
                StateDef::new("pending")
                    .display_name("Pending")
                    .on_enter(vec!["notify_reviewer"]),
            )
            .state(StateDef::new("approved").display_name("Approved"))
            .state(
                StateDef::new("completed")
                    .display_name("Completed")
                    .final_state(),
            )
            .transition(Transition::new("draft", "submit", "pending").guard("has_required_fields"))
            .transition(Transition::new("pending", "approve", "approved").guard("is_reviewer"))
            .transition(Transition::new("approved", "complete", "completed"))
    }

    #[test]
    fn state_machine_serde_round_trip() {
        let machine = sample_machine();
        let json = serde_json::to_string_pretty(&machine).unwrap();
        let parsed: StateMachine = serde_json::from_str(&json).unwrap();

        assert_eq!(machine.name, parsed.name);
        assert_eq!(machine.initial_state, parsed.initial_state);
        assert_eq!(machine.states.len(), parsed.states.len());
        assert_eq!(machine.transitions.len(), parsed.transitions.len());
    }

    #[test]
    fn state_def_serde_round_trip() {
        let state = StateDef::new("pending")
            .display_name("Pending Review")
            .description("Awaiting reviewer approval")
            .on_enter(vec!["notify_reviewer", "start_sla_timer"])
            .on_exit(vec!["stop_sla_timer"])
            .metadata(serde_json::json!({"color": "yellow"}));

        let json = serde_json::to_string(&state).unwrap();
        let parsed: StateDef = serde_json::from_str(&json).unwrap();

        assert_eq!(state.name, parsed.name);
        assert_eq!(state.display_name, parsed.display_name);
        assert_eq!(state.description, parsed.description);
        assert_eq!(state.is_final, parsed.is_final);
        assert_eq!(state.on_enter, parsed.on_enter);
        assert_eq!(state.on_exit, parsed.on_exit);
        assert_eq!(state.metadata, parsed.metadata);
    }

    #[test]
    fn transition_serde_round_trip() {
        let transition = Transition::new("pending", "reject", "rejected")
            .guard("is_reviewer")
            .actions(vec!["log_rejection_reason", "notify_submitter"])
            .description("Reviewer rejects the submission");

        let json = serde_json::to_string(&transition).unwrap();
        let parsed: Transition = serde_json::from_str(&json).unwrap();

        assert_eq!(transition.from, parsed.from);
        assert_eq!(transition.event, parsed.event);
        assert_eq!(transition.to, parsed.to);
        assert_eq!(transition.guard, parsed.guard);
        assert_eq!(transition.actions, parsed.actions);
        assert_eq!(transition.description, parsed.description);
    }

    #[test]
    fn json_omits_empty_optional_fields() {
        let state = StateDef::new("draft");
        let json = serde_json::to_string(&state).unwrap();
        assert!(!json.contains("display_name"));
        assert!(!json.contains("description"));
        assert!(!json.contains("on_enter"));
        assert!(!json.contains("on_exit"));
        assert!(!json.contains("metadata"));

        let transition = Transition::new("a", "go", "b");
        let json = serde_json::to_string(&transition).unwrap();
        assert!(!json.contains("guard"));
        assert!(!json.contains("actions"));
        assert!(!json.contains("description"));

        let machine = StateMachine::new("test").initial("a");
        let json = serde_json::to_string(&machine).unwrap();
        assert!(!json.contains("display_name"));
        assert!(!json.contains("description"));
    }

    #[test]
    fn validate_valid_machine() {
        let machine = sample_machine();
        let warnings = machine.validate().unwrap();
        assert!(warnings.is_empty());
    }

    #[test]
    fn validate_missing_initial_state() {
        let machine = StateMachine::new("test").state(StateDef::new("a"));
        assert!(machine.validate().is_err());
    }

    #[test]
    fn validate_initial_state_not_in_states() {
        let machine = StateMachine::new("test")
            .initial("nonexistent")
            .state(StateDef::new("a").final_state());
        assert!(machine.validate().is_err());
    }

    #[test]
    fn validate_invalid_transition_source() {
        let machine = StateMachine::new("test")
            .initial("a")
            .state(StateDef::new("a").final_state())
            .transition(Transition::new("missing", "go", "a"));
        assert!(machine.validate().is_err());
    }

    #[test]
    fn validate_invalid_transition_target() {
        let machine = StateMachine::new("test")
            .initial("a")
            .state(StateDef::new("a").final_state())
            .transition(Transition::new("a", "go", "missing"));
        assert!(machine.validate().is_err());
    }

    #[test]
    fn validate_unreachable_state() {
        let machine = StateMachine::new("test")
            .initial("a")
            .state(StateDef::new("a").final_state())
            .state(StateDef::new("orphan"));
        let warnings = machine.validate().unwrap();
        assert!(warnings.contains(&Warning::UnreachableState("orphan".into())));
    }

    #[test]
    fn validate_dead_end_state() {
        let machine = StateMachine::new("test")
            .initial("a")
            .state(StateDef::new("a"))
            .state(StateDef::new("b"))
            .transition(Transition::new("a", "go", "b"));
        let warnings = machine.validate().unwrap();
        assert!(warnings.contains(&Warning::DeadEndState("b".into())));
    }

    #[test]
    fn validate_no_final_states() {
        let machine = StateMachine::new("test")
            .initial("a")
            .state(StateDef::new("a"))
            .state(StateDef::new("b"))
            .transition(Transition::new("a", "go", "b"))
            .transition(Transition::new("b", "back", "a"));
        let warnings = machine.validate().unwrap();
        assert!(warnings.contains(&Warning::NoFinalStates));
    }

    #[test]
    fn states_for_event_returns_matching_transitions() {
        let machine = sample_machine();
        let submit_transitions = machine.states_for_event("submit");
        assert_eq!(submit_transitions.len(), 1);
        assert_eq!(submit_transitions[0].from, "draft");
        assert_eq!(submit_transitions[0].to, "pending");
    }

    #[test]
    fn states_for_event_returns_empty_for_unknown() {
        let machine = sample_machine();
        assert!(machine.states_for_event("nonexistent").is_empty());
    }

    #[test]
    fn events_from_state_returns_outgoing() {
        let machine = sample_machine();
        let from_draft = machine.events_from_state("draft");
        assert_eq!(from_draft.len(), 1);
        assert_eq!(from_draft[0].event, "submit");
    }

    #[test]
    fn events_from_state_returns_empty_for_final() {
        let machine = sample_machine();
        assert!(machine.events_from_state("completed").is_empty());
    }

    #[test]
    fn state_machine_json_structure() {
        let machine = sample_machine();
        let json = serde_json::to_string(&machine).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(value.get("name").is_some());
        assert!(value.get("initial_state").is_some());
        assert!(value.get("states").is_some());
        assert!(value.get("transitions").is_some());

        let states = value["states"].as_array().unwrap();
        assert_eq!(states.len(), 4);

        let transitions = value["transitions"].as_array().unwrap();
        assert_eq!(transitions.len(), 3);
    }

    #[test]
    fn state_machine_builder_chain() {
        let machine = StateMachine::new("workflow")
            .display_name("Workflow")
            .description("A test workflow")
            .initial("start")
            .state(StateDef::new("start"))
            .state(StateDef::new("end").final_state())
            .transition(Transition::new("start", "go", "end"));

        assert_eq!(machine.name, "workflow");
        assert_eq!(machine.display_name.as_deref(), Some("Workflow"));
        assert_eq!(machine.description.as_deref(), Some("A test workflow"));
        assert_eq!(machine.initial_state, "start");
        assert_eq!(machine.states.len(), 2);
        assert_eq!(machine.transitions.len(), 1);
    }

    #[test]
    fn state_def_builder_chain() {
        let state = StateDef::new("processing")
            .display_name("Processing")
            .description("Order is being processed")
            .final_state()
            .on_enter(vec!["start_timer", "notify"])
            .on_exit(vec!["stop_timer"])
            .metadata(serde_json::json!({"color": "blue", "icon": "gear"}));

        assert_eq!(state.name, "processing");
        assert_eq!(state.display_name.as_deref(), Some("Processing"));
        assert_eq!(
            state.description.as_deref(),
            Some("Order is being processed")
        );
        assert!(state.is_final);
        assert_eq!(state.on_enter, vec!["start_timer", "notify"]);
        assert_eq!(state.on_exit, vec!["stop_timer"]);
        assert!(state.metadata.is_some());
    }

    #[test]
    fn transition_builder_chain() {
        let transition = Transition::new("draft", "submit", "pending")
            .guard("has_required_fields")
            .actions(vec!["validate", "log_submission"])
            .description("Submit draft for review");

        assert_eq!(transition.from, "draft");
        assert_eq!(transition.event, "submit");
        assert_eq!(transition.to, "pending");
        assert_eq!(transition.guard.as_deref(), Some("has_required_fields"));
        assert_eq!(transition.actions, vec!["validate", "log_submission"]);
        assert_eq!(
            transition.description.as_deref(),
            Some("Submit draft for review")
        );
    }

    #[test]
    fn state_def_defaults() {
        let state = StateDef::new("x");
        assert_eq!(state.name, "x");
        assert!(!state.is_final);
        assert!(state.on_enter.is_empty());
        assert!(state.on_exit.is_empty());
        assert!(state.display_name.is_none());
        assert!(state.description.is_none());
        assert!(state.metadata.is_none());
    }

    #[test]
    fn validate_all_warnings_combined() {
        // Machine with unreachable + dead-end + no final states.
        // "orphan" is both unreachable and a dead-end (no outgoing transitions, not final).
        let machine = StateMachine::new("test")
            .initial("a")
            .state(StateDef::new("a"))
            .state(StateDef::new("b"))
            .state(StateDef::new("orphan"))
            .transition(Transition::new("a", "go", "b"));

        let warnings = machine.validate().unwrap();
        assert!(warnings.contains(&Warning::UnreachableState("orphan".into())));
        assert!(warnings.contains(&Warning::DeadEndState("b".into())));
        assert!(warnings.contains(&Warning::DeadEndState("orphan".into())));
        assert!(warnings.contains(&Warning::NoFinalStates));
        assert_eq!(warnings.len(), 4);
    }

    #[test]
    fn full_order_lifecycle() {
        let machine = StateMachine::new("order_lifecycle")
            .display_name("Order Lifecycle")
            .description("Tracks an order from creation to fulfillment")
            .initial("draft")
            .state(
                StateDef::new("draft")
                    .display_name("Draft")
                    .description("Order is being prepared"),
            )
            .state(
                StateDef::new("submitted")
                    .display_name("Submitted")
                    .on_enter(vec!["validate_inventory", "calculate_totals"]),
            )
            .state(
                StateDef::new("processing")
                    .display_name("Processing")
                    .on_enter(vec!["charge_payment", "reserve_inventory"]),
            )
            .state(
                StateDef::new("shipped")
                    .display_name("Shipped")
                    .on_enter(vec!["generate_tracking", "notify_customer"]),
            )
            .state(
                StateDef::new("delivered")
                    .display_name("Delivered")
                    .final_state(),
            )
            .state(
                StateDef::new("cancelled")
                    .display_name("Cancelled")
                    .final_state()
                    .on_enter(vec!["refund_payment", "release_inventory"]),
            )
            .transition(
                Transition::new("draft", "submit", "submitted")
                    .guard("has_items")
                    .description("Customer submits the order"),
            )
            .transition(
                Transition::new("submitted", "process", "processing")
                    .guard("payment_valid")
                    .actions(vec!["lock_prices"]),
            )
            .transition(
                Transition::new("processing", "ship", "shipped").guard("inventory_fulfilled"),
            )
            .transition(Transition::new("shipped", "deliver", "delivered"))
            .transition(Transition::new("draft", "cancel", "cancelled"))
            .transition(
                Transition::new("submitted", "cancel", "cancelled").guard("cancellation_allowed"),
            )
            .transition(
                Transition::new("processing", "cancel", "cancelled")
                    .guard("cancellation_allowed")
                    .actions(vec!["reverse_payment"]),
            );

        let warnings = machine.validate().unwrap();
        assert!(warnings.is_empty());

        // Cancel event has 3 sources
        let cancel_transitions = machine.states_for_event("cancel");
        assert_eq!(cancel_transitions.len(), 3);

        // Draft has 2 outgoing transitions (submit, cancel)
        let from_draft = machine.events_from_state("draft");
        assert_eq!(from_draft.len(), 2);
    }
}
