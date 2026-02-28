use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::field::{DataType, FieldMeaning};

/// A business operation schema describing what an action does and what it needs.
///
/// Actions are the verbs of a service: "submit order", "approve review", "send invoice".
/// Each action defines its input contract, preconditions that must hold, effects that
/// will occur, and an optional link to a state machine transition.
///
/// ```
/// use ferro_projections::{ActionDef, InputDef, DataType, FieldMeaning};
///
/// let action = ActionDef::new("submit_order")
///     .display_name("Submit Order")
///     .description("Validates and submits a customer order for processing")
///     .input(InputDef::new("order_id", DataType::Integer, FieldMeaning::Identifier))
///     .precondition("has_items")
///     .precondition("payment_valid")
///     .effect("notify_customer")
///     .transition_trigger("submit");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct ActionDef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<InputDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preconditions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition_trigger: Option<String>,
}

impl ActionDef {
    /// Creates a new action definition with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: None,
            description: None,
            inputs: Vec::new(),
            preconditions: Vec::new(),
            effects: Vec::new(),
            transition_trigger: None,
        }
    }

    /// Sets the human-readable display name.
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Sets the action description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Adds an input parameter to this action.
    pub fn input(mut self, input: InputDef) -> Self {
        self.inputs.push(input);
        self
    }

    /// Adds a precondition guard name that must hold before this action can execute.
    pub fn precondition(mut self, guard: impl Into<String>) -> Self {
        self.preconditions.push(guard.into());
        self
    }

    /// Adds an effect name that occurs when this action executes.
    pub fn effect(mut self, effect: impl Into<String>) -> Self {
        self.effects.push(effect.into());
        self
    }

    /// Sets the state machine transition that this action triggers.
    pub fn transition_trigger(mut self, trigger: impl Into<String>) -> Self {
        self.transition_trigger = Some(trigger.into());
        self
    }
}

/// An input parameter definition for an action.
///
/// Reuses [`DataType`] and [`FieldMeaning`] from the field module to maintain
/// a single type vocabulary across the projection schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct InputDef {
    pub name: String,
    pub data_type: DataType,
    pub meaning: FieldMeaning,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn default_true() -> bool {
    true
}

impl InputDef {
    /// Creates a new required input parameter.
    pub fn new(name: impl Into<String>, data_type: DataType, meaning: FieldMeaning) -> Self {
        Self {
            name: name.into(),
            data_type,
            meaning,
            required: true,
            description: None,
        }
    }

    /// Sets whether this input is required.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Sets the input description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// A named boolean condition that guards action execution or state transitions.
///
/// Guards are declarative checks referenced by name. The actual evaluation logic
/// lives outside the projection schema.
///
/// ```
/// use ferro_projections::GuardDef;
///
/// let guard = GuardDef::new("has_items")
///     .display_name("Has Items")
///     .description("Order must contain at least one line item");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct GuardDef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl GuardDef {
    /// Creates a new guard definition with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: None,
            description: None,
        }
    }

    /// Sets the human-readable display name.
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Sets the guard description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::{DataType, FieldMeaning};

    // -- ActionDef tests --

    #[test]
    fn action_def_minimal() {
        let action = ActionDef::new("submit");
        assert_eq!(action.name, "submit");
        assert!(action.display_name.is_none());
        assert!(action.description.is_none());
        assert!(action.inputs.is_empty());
        assert!(action.preconditions.is_empty());
        assert!(action.effects.is_empty());
        assert!(action.transition_trigger.is_none());
    }

    #[test]
    fn action_def_builder_chain() {
        let action = ActionDef::new("submit_order")
            .display_name("Submit Order")
            .description("Submits a customer order")
            .input(InputDef::new(
                "order_id",
                DataType::Integer,
                FieldMeaning::Identifier,
            ))
            .input(InputDef::new("notes", DataType::String, FieldMeaning::FreeText).required(false))
            .precondition("has_items")
            .precondition("payment_valid")
            .effect("notify_customer")
            .effect("send_confirmation")
            .transition_trigger("submit");

        assert_eq!(action.name, "submit_order");
        assert_eq!(action.display_name.as_deref(), Some("Submit Order"));
        assert_eq!(
            action.description.as_deref(),
            Some("Submits a customer order")
        );
        assert_eq!(action.inputs.len(), 2);
        assert!(action.inputs[0].required);
        assert!(!action.inputs[1].required);
        assert_eq!(action.preconditions, vec!["has_items", "payment_valid"]);
        assert_eq!(action.effects, vec!["notify_customer", "send_confirmation"]);
        assert_eq!(action.transition_trigger.as_deref(), Some("submit"));
    }

    #[test]
    fn action_def_serde_round_trip() {
        let action = ActionDef::new("submit_order")
            .display_name("Submit Order")
            .input(InputDef::new(
                "order_id",
                DataType::Integer,
                FieldMeaning::Identifier,
            ))
            .precondition("has_items")
            .effect("notify")
            .transition_trigger("submit");

        let json = serde_json::to_string(&action).unwrap();
        let parsed: ActionDef = serde_json::from_str(&json).unwrap();
        assert_eq!(action, parsed);
    }

    #[test]
    fn action_def_json_omits_empty_vecs_and_none() {
        let action = ActionDef::new("simple");
        let json = serde_json::to_string(&action).unwrap();
        assert!(!json.contains("display_name"));
        assert!(!json.contains("description"));
        assert!(!json.contains("inputs"));
        assert!(!json.contains("preconditions"));
        assert!(!json.contains("effects"));
        assert!(!json.contains("transition_trigger"));
    }

    #[test]
    fn action_def_json_schema() {
        let schema = schemars::schema_for!(ActionDef);
        let value = schema.to_value();
        let props = value
            .get("properties")
            .expect("ActionDef schema must have properties");
        let obj = props.as_object().unwrap();
        assert!(obj.contains_key("name"), "missing 'name' property");
        assert!(obj.contains_key("inputs"), "missing 'inputs' property");
        assert!(
            obj.contains_key("preconditions"),
            "missing 'preconditions' property"
        );
        assert!(obj.contains_key("effects"), "missing 'effects' property");
    }

    // -- InputDef tests --

    #[test]
    fn input_def_minimal() {
        let input = InputDef::new("order_id", DataType::Integer, FieldMeaning::Identifier);
        assert_eq!(input.name, "order_id");
        assert_eq!(input.data_type, DataType::Integer);
        assert_eq!(input.meaning, FieldMeaning::Identifier);
        assert!(input.required);
        assert!(input.description.is_none());
    }

    #[test]
    fn input_def_builder_chain() {
        let input = InputDef::new("notes", DataType::String, FieldMeaning::FreeText)
            .required(false)
            .description("Optional order notes");

        assert_eq!(input.name, "notes");
        assert!(!input.required);
        assert_eq!(input.description.as_deref(), Some("Optional order notes"));
    }

    #[test]
    fn input_def_serde_round_trip() {
        let input = InputDef::new("email", DataType::String, FieldMeaning::Email)
            .description("Customer email");

        let json = serde_json::to_string(&input).unwrap();
        let parsed: InputDef = serde_json::from_str(&json).unwrap();
        assert_eq!(input, parsed);
    }

    #[test]
    fn input_def_defaults() {
        let json = r#"{"name":"total","data_type":"float","meaning":"money"}"#;
        let parsed: InputDef = serde_json::from_str(json).unwrap();
        assert!(parsed.required);
        assert!(parsed.description.is_none());
    }

    #[test]
    fn input_def_json_schema() {
        let schema = schemars::schema_for!(InputDef);
        let value = schema.to_value();
        let props = value
            .get("properties")
            .expect("InputDef schema must have properties");
        let obj = props.as_object().unwrap();
        assert!(obj.contains_key("name"), "missing 'name' property");
        assert!(
            obj.contains_key("data_type"),
            "missing 'data_type' property"
        );
        assert!(obj.contains_key("meaning"), "missing 'meaning' property");
    }

    // -- GuardDef tests --

    #[test]
    fn guard_def_minimal() {
        let guard = GuardDef::new("has_items");
        assert_eq!(guard.name, "has_items");
        assert!(guard.display_name.is_none());
        assert!(guard.description.is_none());
    }

    #[test]
    fn guard_def_builder_chain() {
        let guard = GuardDef::new("payment_valid")
            .display_name("Payment Valid")
            .description("Customer payment method has been verified");

        assert_eq!(guard.name, "payment_valid");
        assert_eq!(guard.display_name.as_deref(), Some("Payment Valid"));
        assert_eq!(
            guard.description.as_deref(),
            Some("Customer payment method has been verified")
        );
    }

    #[test]
    fn guard_def_serde_round_trip() {
        let guard = GuardDef::new("has_items")
            .display_name("Has Items")
            .description("Order must contain at least one item");

        let json = serde_json::to_string(&guard).unwrap();
        let parsed: GuardDef = serde_json::from_str(&json).unwrap();
        assert_eq!(guard, parsed);
    }

    #[test]
    fn guard_def_json_omits_none() {
        let guard = GuardDef::new("simple");
        let json = serde_json::to_string(&guard).unwrap();
        assert!(!json.contains("display_name"));
        assert!(!json.contains("description"));
    }

    #[test]
    fn guard_def_json_schema() {
        let schema = schemars::schema_for!(GuardDef);
        let value = schema.to_value();
        let props = value
            .get("properties")
            .expect("GuardDef schema must have properties");
        let obj = props.as_object().unwrap();
        assert!(obj.contains_key("name"), "missing 'name' property");
    }

    // -- Additional Phase 86-02 tests --

    #[test]
    fn action_def_without_transition_trigger() {
        let action = ActionDef::new("update_notes")
            .display_name("Update Notes")
            .input(InputDef::new(
                "notes",
                DataType::String,
                FieldMeaning::FreeText,
            ))
            .effect("log_change");

        assert!(action.transition_trigger.is_none());
        assert_eq!(action.effects, vec!["log_change"]);
        assert_eq!(action.inputs.len(), 1);
    }

    #[test]
    fn action_def_serde_minimal_round_trip() {
        let action = ActionDef::new("simple");
        let json = serde_json::to_string(&action).unwrap();
        let parsed: ActionDef = serde_json::from_str(&json).unwrap();
        assert_eq!(action, parsed);
    }
}
