use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::action::{ActionDef, GuardDef};
use crate::field::{DataType, FieldDef, FieldMeaning};
use crate::intent::IntentHint;
use crate::relationship::{Cardinality, RelationshipDef};
use crate::state::{StateMachine, Warning};

/// A service definition describing a domain entity and its fields.
///
/// Constructed via a builder API with method chaining:
///
/// ```
/// use ferro_projections::{ServiceDef, DataType, FieldMeaning};
///
/// let order = ServiceDef::new("order")
///     .display_name("Order")
///     .description("Manages customer orders")
///     .field("id", DataType::Integer, FieldMeaning::Identifier)
///     .field("total", DataType::Float, FieldMeaning::Money)
///     .optional_field("notes", DataType::String, FieldMeaning::FreeText);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ServiceDef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub fields: Vec<FieldDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<ActionDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guards: Vec<GuardDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<RelationshipDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intent_hints: Vec<IntentHint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_machine: Option<StateMachine>,
}

impl ServiceDef {
    /// Creates a new service definition with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: None,
            description: None,
            fields: Vec::new(),
            actions: Vec::new(),
            guards: Vec::new(),
            relationships: Vec::new(),
            intent_hints: Vec::new(),
            state_machine: None,
        }
    }

    /// Sets the human-readable display name.
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Sets the service description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Adds a required read-write field.
    pub fn field(
        mut self,
        name: impl Into<String>,
        data_type: DataType,
        meaning: FieldMeaning,
    ) -> Self {
        self.fields.push(FieldDef {
            name: name.into(),
            data_type,
            meaning,
            required: true,
            is_list: false,
            readable: true,
            writable: true,
        });
        self
    }

    /// Adds an optional (nullable) read-write field.
    pub fn optional_field(
        mut self,
        name: impl Into<String>,
        data_type: DataType,
        meaning: FieldMeaning,
    ) -> Self {
        self.fields.push(FieldDef {
            name: name.into(),
            data_type,
            meaning,
            required: false,
            is_list: false,
            readable: true,
            writable: true,
        });
        self
    }

    /// Adds a required read-write list field.
    pub fn list_field(
        mut self,
        name: impl Into<String>,
        data_type: DataType,
        meaning: FieldMeaning,
    ) -> Self {
        self.fields.push(FieldDef {
            name: name.into(),
            data_type,
            meaning,
            required: true,
            is_list: true,
            readable: true,
            writable: true,
        });
        self
    }

    /// Adds a required read-only field (readable but not writable).
    ///
    /// For system-assigned or computed fields like id, created_at, or totals.
    pub fn read_only_field(
        mut self,
        name: impl Into<String>,
        data_type: DataType,
        meaning: FieldMeaning,
    ) -> Self {
        self.fields.push(FieldDef {
            name: name.into(),
            data_type,
            meaning,
            required: true,
            is_list: false,
            readable: true,
            writable: false,
        });
        self
    }

    /// Adds a required write-only field (writable but not readable).
    ///
    /// For sensitive inputs like passwords or API keys that should not be read back.
    pub fn write_only_field(
        mut self,
        name: impl Into<String>,
        data_type: DataType,
        meaning: FieldMeaning,
    ) -> Self {
        self.fields.push(FieldDef {
            name: name.into(),
            data_type,
            meaning,
            required: true,
            is_list: false,
            readable: false,
            writable: true,
        });
        self
    }

    /// Adds an action definition to this service.
    pub fn action(mut self, action: ActionDef) -> Self {
        self.actions.push(action);
        self
    }

    /// Adds a guard definition to this service.
    pub fn guard(mut self, guard: GuardDef) -> Self {
        self.guards.push(guard);
        self
    }

    /// Adds a relationship definition to this service.
    pub fn relationship(mut self, rel: RelationshipDef) -> Self {
        self.relationships.push(rel);
        self
    }

    /// Adds a many-to-one relationship (this service belongs to target).
    pub fn belongs_to(self, name: impl Into<String>, target: impl Into<String>) -> Self {
        self.relationship(RelationshipDef::new(name, target, Cardinality::ManyToOne))
    }

    /// Adds a one-to-many relationship (this service has many of target).
    pub fn has_many(self, name: impl Into<String>, target: impl Into<String>) -> Self {
        self.relationship(RelationshipDef::new(name, target, Cardinality::OneToMany))
    }

    /// Adds a one-to-one relationship (this service has one of target).
    pub fn has_one(self, name: impl Into<String>, target: impl Into<String>) -> Self {
        self.relationship(RelationshipDef::new(name, target, Cardinality::OneToOne))
    }

    /// Adds a many-to-many relationship (this service belongs to many of target).
    pub fn belongs_to_many(self, name: impl Into<String>, target: impl Into<String>) -> Self {
        self.relationship(RelationshipDef::new(name, target, Cardinality::ManyToMany))
    }

    /// Adds an intent hint for overriding structural derivation.
    pub fn intent_hint(mut self, hint: IntentHint) -> Self {
        self.intent_hints.push(hint);
        self
    }

    /// Sets the state machine definition for this service.
    pub fn state_machine(mut self, machine: StateMachine) -> Self {
        self.state_machine = Some(machine);
        self
    }

    /// Validates the service definition and returns warnings for potential issues.
    ///
    /// This is the single validation entry point that subsumes `StateMachine::validate()`.
    /// Guard names form a shared pool referenced from transitions and action preconditions.
    ///
    /// Returns `Err` for fatal issues (undefined guard references, unmatched triggers).
    /// Returns `Ok(warnings)` for structural concerns (unused guards, missing state machine).
    pub fn validate(&self) -> Result<Vec<Warning>, crate::Error> {
        let mut warnings = Vec::new();

        // 1. Delegate to state machine validation if present
        if let Some(ref sm) = self.state_machine {
            warnings.extend(sm.validate()?);
        }

        // 2. Collect declared guard names
        let declared_guards: HashSet<&str> = self.guards.iter().map(|g| g.name.as_str()).collect();

        // 3. Check action preconditions reference declared guards
        for action in &self.actions {
            for precondition in &action.preconditions {
                if !declared_guards.contains(precondition.as_str()) {
                    return Err(crate::Error::Validation(format!(
                        "action '{}' references undefined guard '{}'",
                        action.name, precondition
                    )));
                }
            }
        }

        // 4. Check transition guards reference declared guards (if state machine exists)
        if let Some(ref sm) = self.state_machine {
            for transition in &sm.transitions {
                if let Some(ref guard) = transition.guard {
                    if !declared_guards.contains(guard.as_str()) {
                        return Err(crate::Error::Validation(format!(
                            "transition '{}' -> '{}' references undefined guard '{}'",
                            transition.from, transition.to, guard
                        )));
                    }
                }
            }
        }

        // 5. Check action transition_triggers match state machine event names
        if let Some(ref sm) = self.state_machine {
            let event_names: HashSet<&str> =
                sm.transitions.iter().map(|t| t.event.as_str()).collect();
            for action in &self.actions {
                if let Some(ref trigger) = action.transition_trigger {
                    if !event_names.contains(trigger.as_str()) {
                        return Err(crate::Error::Validation(format!(
                            "action '{}' has transition_trigger '{}' that does not match any state machine event",
                            action.name, trigger
                        )));
                    }
                }
            }
        }

        // 6. Warn about declared guards never referenced
        let mut referenced_guards: HashSet<&str> = HashSet::new();
        for action in &self.actions {
            for precondition in &action.preconditions {
                referenced_guards.insert(precondition.as_str());
            }
        }
        if let Some(ref sm) = self.state_machine {
            for transition in &sm.transitions {
                if let Some(ref guard) = transition.guard {
                    referenced_guards.insert(guard.as_str());
                }
            }
        }
        for guard in &self.guards {
            if !referenced_guards.contains(guard.name.as_str()) {
                warnings.push(Warning::UnusedGuard(guard.name.clone()));
            }
        }

        // 7. Warn about actions with transition_trigger when no state machine exists
        if self.state_machine.is_none() {
            for action in &self.actions {
                if action.transition_trigger.is_some() {
                    warnings.push(Warning::TransitionTriggerWithoutStateMachine(
                        action.name.clone(),
                    ));
                }
            }
        }

        // 8. Warn about duplicate relationship names
        {
            let mut seen = HashSet::new();
            for rel in &self.relationships {
                if !seen.insert(rel.name.as_str()) {
                    warnings.push(Warning::DuplicateRelationship(rel.name.clone()));
                }
            }
        }

        // 9. Warn if ManyToMany relationship has foreign_key set
        for rel in &self.relationships {
            if rel.cardinality == Cardinality::ManyToMany && rel.foreign_key.is_some() {
                warnings.push(Warning::ManyToManyWithForeignKey {
                    relationship: rel.name.clone(),
                });
            }
        }

        // 10. Check for conflicting intent hints (same intent in both Primary and Exclude)
        {
            let mut primaries = HashSet::new();
            let mut excludes = HashSet::new();
            let mut primary_count = 0u32;

            for hint in &self.intent_hints {
                match hint {
                    IntentHint::Primary(intent) => {
                        primary_count += 1;
                        let serialized = serde_json::to_string(intent)
                            .unwrap_or_default()
                            .trim_matches('"')
                            .to_string();
                        primaries.insert(serialized);
                    }
                    IntentHint::Exclude(intent) => {
                        let serialized = serde_json::to_string(intent)
                            .unwrap_or_default()
                            .trim_matches('"')
                            .to_string();
                        excludes.insert(serialized);
                    }
                }
            }

            for intent_name in primaries.intersection(&excludes) {
                warnings.push(Warning::ConflictingIntentHints {
                    intent: intent_name.clone(),
                });
            }

            if primary_count > 1 {
                warnings.push(Warning::MultiplePrimaryIntentHints);
            }
        }

        Ok(warnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_def_builder_chain() {
        let service = ServiceDef::new("order")
            .display_name("Order")
            .description("Manages customer orders")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .optional_field("notes", DataType::String, FieldMeaning::FreeText)
            .list_field("tags", DataType::String, FieldMeaning::Category);

        assert_eq!(service.name, "order");
        assert_eq!(service.display_name.as_deref(), Some("Order"));
        assert_eq!(
            service.description.as_deref(),
            Some("Manages customer orders")
        );
        assert_eq!(service.fields.len(), 5);

        // Required field
        assert!(service.fields[0].required);
        assert!(!service.fields[0].is_list);

        // Optional field
        assert!(!service.fields[3].required);
        assert!(!service.fields[3].is_list);

        // List field
        assert!(service.fields[4].required);
        assert!(service.fields[4].is_list);
    }

    #[test]
    fn service_def_minimal() {
        let service = ServiceDef::new("user");
        assert_eq!(service.name, "user");
        assert!(service.display_name.is_none());
        assert!(service.description.is_none());
        assert!(service.fields.is_empty());
    }

    #[test]
    fn service_def_serde_round_trip() {
        let service = ServiceDef::new("order")
            .display_name("Order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("total", DataType::Float, FieldMeaning::Money)
            .optional_field("notes", DataType::String, FieldMeaning::FreeText);

        let json = serde_json::to_string(&service).unwrap();
        let parsed: ServiceDef = serde_json::from_str(&json).unwrap();
        assert_eq!(service, parsed);
    }

    #[test]
    fn service_def_json_omits_none_fields() {
        let service = ServiceDef::new("order");
        let json = serde_json::to_string(&service).unwrap();
        assert!(!json.contains("display_name"));
        assert!(!json.contains("description"));
    }

    #[test]
    fn service_def_multiple_fields() {
        let service = ServiceDef::new("product")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("price", DataType::Float, FieldMeaning::Money)
            .field("sku", DataType::String, FieldMeaning::Custom("sku".into()))
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt);

        assert_eq!(service.fields.len(), 5);
        // Order preserved
        assert_eq!(service.fields[0].name, "id");
        assert_eq!(service.fields[1].name, "name");
        assert_eq!(service.fields[2].name, "price");
        assert_eq!(service.fields[3].name, "sku");
        assert_eq!(service.fields[4].name, "created_at");
    }

    #[test]
    fn service_def_json_structure() {
        let service = ServiceDef::new("order")
            .display_name("Order")
            .description("Customer orders")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .optional_field("notes", DataType::String, FieldMeaning::FreeText);

        let json = serde_json::to_string(&service).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(value.get("name").is_some());
        assert!(value.get("display_name").is_some());
        assert!(value.get("description").is_some());
        assert!(value.get("fields").is_some());

        let fields = value["fields"].as_array().unwrap();
        assert_eq!(fields.len(), 2);
    }

    #[test]
    fn order_service_example() {
        let service = ServiceDef::new("order")
            .display_name("Order")
            .description("Manages customer orders and fulfillment")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("customer_id", DataType::Integer, FieldMeaning::ForeignKey)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("email", DataType::String, FieldMeaning::Email)
            .field("notes", DataType::String, FieldMeaning::FreeText)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt);

        assert_eq!(service.fields.len(), 8);
        assert_eq!(service.fields[2].meaning, FieldMeaning::Money);
        assert_eq!(service.fields[3].meaning, FieldMeaning::Status);

        // Serde round-trip
        let json = serde_json::to_string(&service).unwrap();
        let parsed: ServiceDef = serde_json::from_str(&json).unwrap();
        assert_eq!(service, parsed);
    }

    // -- StateMachine integration tests --

    use crate::state::{StateDef, StateMachine, Transition};

    #[test]
    fn service_def_with_state_machine() {
        let machine = StateMachine::new("order_lifecycle")
            .initial("draft")
            .state(StateDef::new("draft"))
            .state(StateDef::new("completed").final_state())
            .transition(Transition::new("draft", "complete", "completed"));

        let service = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .state_machine(machine);

        assert!(service.state_machine.is_some());
        let sm = service.state_machine.as_ref().unwrap();
        assert_eq!(sm.states.len(), 2);
        assert_eq!(sm.transitions.len(), 1);
    }

    #[test]
    fn service_def_state_machine_serde_round_trip() {
        let machine = StateMachine::new("order_lifecycle")
            .initial("draft")
            .state(StateDef::new("draft").display_name("Draft"))
            .state(
                StateDef::new("completed")
                    .display_name("Completed")
                    .final_state(),
            )
            .transition(
                Transition::new("draft", "complete", "completed")
                    .guard("is_valid")
                    .actions(vec!["notify"]),
            );

        let service = ServiceDef::new("order")
            .display_name("Order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status)
            .state_machine(machine);

        let json = serde_json::to_string_pretty(&service).unwrap();
        let parsed: ServiceDef = serde_json::from_str(&json).unwrap();
        assert_eq!(service, parsed);
    }

    #[test]
    fn service_def_without_state_machine_json() {
        let service =
            ServiceDef::new("user").field("id", DataType::Integer, FieldMeaning::Identifier);

        let json = serde_json::to_string(&service).unwrap();
        assert!(!json.contains("state_machine"));
    }

    #[test]
    fn order_service_full_example() {
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

        let service = ServiceDef::new("order")
            .display_name("Order")
            .description("Manages customer orders and fulfillment")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("customer_id", DataType::Integer, FieldMeaning::ForeignKey)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("email", DataType::String, FieldMeaning::Email)
            .field("notes", DataType::String, FieldMeaning::FreeText)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt)
            .state_machine(machine);

        // Field assertions
        assert_eq!(service.fields.len(), 8);

        // State machine assertions
        let sm = service.state_machine.as_ref().unwrap();
        assert_eq!(sm.states.len(), 6);
        assert_eq!(sm.transitions.len(), 7);
        assert_eq!(sm.initial_state, "draft");

        // Validation passes cleanly
        let warnings = sm.validate().unwrap();
        assert!(warnings.is_empty());

        // Serde round-trip
        let json = serde_json::to_string_pretty(&service).unwrap();
        let parsed: ServiceDef = serde_json::from_str(&json).unwrap();
        assert_eq!(service, parsed);
    }

    #[test]
    fn service_def_json_schema() {
        let schema = schemars::schema_for!(ServiceDef);
        let value = schema.to_value();
        let props = value
            .get("properties")
            .expect("ServiceDef schema must have properties");
        let obj = props.as_object().unwrap();
        assert!(obj.contains_key("name"), "missing 'name' property");
        assert!(obj.contains_key("fields"), "missing 'fields' property");
        assert!(
            obj.contains_key("state_machine"),
            "missing 'state_machine' property"
        );
    }

    // -- readable/writable builder tests --

    #[test]
    fn read_only_field_builder() {
        let service = ServiceDef::new("order")
            .read_only_field("id", DataType::Integer, FieldMeaning::Identifier)
            .read_only_field("created_at", DataType::DateTime, FieldMeaning::CreatedAt);

        assert_eq!(service.fields.len(), 2);
        for f in &service.fields {
            assert!(f.readable);
            assert!(!f.writable);
            assert!(f.required);
            assert!(!f.is_list);
        }
    }

    #[test]
    fn write_only_field_builder() {
        let service = ServiceDef::new("user").write_only_field(
            "password",
            DataType::String,
            FieldMeaning::Sensitive,
        );

        assert_eq!(service.fields.len(), 1);
        let f = &service.fields[0];
        assert!(!f.readable);
        assert!(f.writable);
        assert!(f.required);
        assert!(!f.is_list);
    }

    #[test]
    fn mixed_access_fields_serde_round_trip() {
        let service = ServiceDef::new("user")
            .read_only_field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .write_only_field("password", DataType::String, FieldMeaning::Sensitive)
            .read_only_field("created_at", DataType::DateTime, FieldMeaning::CreatedAt);

        let json = serde_json::to_string(&service).unwrap();
        let parsed: ServiceDef = serde_json::from_str(&json).unwrap();
        assert_eq!(service, parsed);

        // Verify access modes survived round-trip
        assert!(parsed.fields[0].readable);
        assert!(!parsed.fields[0].writable);
        assert!(parsed.fields[1].readable);
        assert!(parsed.fields[1].writable);
        assert!(!parsed.fields[2].readable);
        assert!(parsed.fields[2].writable);
        assert!(parsed.fields[3].readable);
        assert!(!parsed.fields[3].writable);
    }

    #[test]
    fn existing_field_builders_default_read_write() {
        let service = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .optional_field("notes", DataType::String, FieldMeaning::FreeText)
            .list_field("tags", DataType::String, FieldMeaning::Category);

        for f in &service.fields {
            assert!(f.readable, "field '{}' should be readable", f.name);
            assert!(f.writable, "field '{}' should be writable", f.name);
        }
    }

    // -- Phase 86-02 tests: actions/guards integration + validate() --

    use crate::action::{ActionDef, GuardDef, InputDef};
    use crate::state::Warning;

    #[test]
    fn service_def_with_actions_and_guards_builder() {
        let service = ServiceDef::new("order")
            .guard(GuardDef::new("has_items"))
            .guard(GuardDef::new("payment_valid"))
            .action(
                ActionDef::new("submit_order")
                    .precondition("has_items")
                    .precondition("payment_valid"),
            )
            .action(ActionDef::new("update_notes"));

        assert_eq!(service.guards.len(), 2);
        assert_eq!(service.actions.len(), 2);
        assert_eq!(service.actions[0].name, "submit_order");
        assert_eq!(service.actions[1].name, "update_notes");
    }

    #[test]
    fn service_def_serde_round_trip_with_actions_guards() {
        let service = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .guard(GuardDef::new("has_items").display_name("Has Items"))
            .action(
                ActionDef::new("submit")
                    .input(InputDef::new(
                        "order_id",
                        DataType::Integer,
                        FieldMeaning::Identifier,
                    ))
                    .precondition("has_items")
                    .effect("notify"),
            );

        let json = serde_json::to_string_pretty(&service).unwrap();
        let parsed: ServiceDef = serde_json::from_str(&json).unwrap();
        assert_eq!(service, parsed);
    }

    #[test]
    fn service_def_json_omits_empty_actions_guards() {
        let service = ServiceDef::new("user");
        let json = serde_json::to_string(&service).unwrap();
        assert!(!json.contains("actions"));
        assert!(!json.contains("guards"));
    }

    #[test]
    fn validate_passes_valid_service() {
        let machine = StateMachine::new("order_lifecycle")
            .initial("draft")
            .state(StateDef::new("draft"))
            .state(StateDef::new("submitted").final_state())
            .transition(Transition::new("draft", "submit", "submitted").guard("has_items"));

        let service = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .guard(GuardDef::new("has_items"))
            .action(
                ActionDef::new("submit_order")
                    .precondition("has_items")
                    .transition_trigger("submit"),
            )
            .state_machine(machine);

        let warnings = service.validate().unwrap();
        assert!(warnings.is_empty());
    }

    #[test]
    fn validate_catches_undefined_action_precondition() {
        let service = ServiceDef::new("order")
            .guard(GuardDef::new("has_items"))
            .action(ActionDef::new("submit").precondition("nonexistent_guard"));

        let result = service.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("nonexistent_guard"));
        assert!(err.contains("submit"));
    }

    #[test]
    fn validate_catches_undefined_transition_guard() {
        let machine = StateMachine::new("lifecycle")
            .initial("draft")
            .state(StateDef::new("draft"))
            .state(StateDef::new("done").final_state())
            .transition(Transition::new("draft", "finish", "done").guard("undefined_guard"));

        let service = ServiceDef::new("order").state_machine(machine);

        let result = service.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("undefined_guard"));
    }

    #[test]
    fn validate_catches_unmatched_transition_trigger() {
        let machine = StateMachine::new("lifecycle")
            .initial("draft")
            .state(StateDef::new("draft"))
            .state(StateDef::new("done").final_state())
            .transition(Transition::new("draft", "finish", "done"));

        let service = ServiceDef::new("order")
            .action(ActionDef::new("submit").transition_trigger("nonexistent_event"))
            .state_machine(machine);

        let result = service.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("nonexistent_event"));
    }

    #[test]
    fn validate_warns_unused_guards() {
        let service = ServiceDef::new("order")
            .guard(GuardDef::new("used_guard"))
            .guard(GuardDef::new("unused_guard"))
            .action(ActionDef::new("submit").precondition("used_guard"));

        let warnings = service.validate().unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings.contains(&Warning::UnusedGuard("unused_guard".into())));
    }

    #[test]
    fn validate_warns_transition_trigger_without_state_machine() {
        let service =
            ServiceDef::new("order").action(ActionDef::new("submit").transition_trigger("submit"));

        let warnings = service.validate().unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(
            warnings.contains(&Warning::TransitionTriggerWithoutStateMachine(
                "submit".into()
            ))
        );
    }

    #[test]
    fn validate_delegates_to_state_machine_validate() {
        // Missing initial state in states — state machine validation catches this
        let machine = StateMachine::new("lifecycle")
            .initial("nonexistent")
            .state(StateDef::new("a").final_state());

        let service = ServiceDef::new("order").state_machine(machine);

        let result = service.validate();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("nonexistent"));
    }

    #[test]
    fn validate_without_state_machine_or_actions_passes_clean() {
        let service =
            ServiceDef::new("simple").field("id", DataType::Integer, FieldMeaning::Identifier);

        let warnings = service.validate().unwrap();
        assert!(warnings.is_empty());
    }

    #[test]
    fn full_order_service_with_guards_actions_validates_clean() {
        let machine = StateMachine::new("order_lifecycle")
            .display_name("Order Lifecycle")
            .initial("draft")
            .state(StateDef::new("draft").display_name("Draft"))
            .state(StateDef::new("submitted").display_name("Submitted"))
            .state(StateDef::new("processing").display_name("Processing"))
            .state(
                StateDef::new("shipped")
                    .display_name("Shipped")
                    .final_state(),
            )
            .state(
                StateDef::new("cancelled")
                    .display_name("Cancelled")
                    .final_state(),
            )
            .transition(Transition::new("draft", "submit", "submitted").guard("has_items"))
            .transition(
                Transition::new("submitted", "process", "processing").guard("payment_valid"),
            )
            .transition(
                Transition::new("processing", "ship", "shipped").guard("inventory_fulfilled"),
            )
            .transition(
                Transition::new("draft", "cancel", "cancelled").guard("cancellation_allowed"),
            )
            .transition(
                Transition::new("submitted", "cancel", "cancelled").guard("cancellation_allowed"),
            );

        let service = ServiceDef::new("order")
            .display_name("Order")
            .description("Full order management")
            .read_only_field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("customer_id", DataType::Integer, FieldMeaning::ForeignKey)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .read_only_field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .guard(GuardDef::new("has_items").display_name("Has Items"))
            .guard(GuardDef::new("payment_valid").display_name("Payment Valid"))
            .guard(GuardDef::new("inventory_fulfilled").display_name("Inventory Fulfilled"))
            .guard(GuardDef::new("cancellation_allowed").display_name("Cancellation Allowed"))
            .action(
                ActionDef::new("submit_order")
                    .display_name("Submit Order")
                    .input(InputDef::new(
                        "order_id",
                        DataType::Integer,
                        FieldMeaning::Identifier,
                    ))
                    .precondition("has_items")
                    .effect("notify_customer")
                    .transition_trigger("submit"),
            )
            .action(
                ActionDef::new("process_order")
                    .precondition("payment_valid")
                    .transition_trigger("process"),
            )
            .action(
                ActionDef::new("ship_order")
                    .precondition("inventory_fulfilled")
                    .transition_trigger("ship"),
            )
            .action(
                ActionDef::new("cancel_order")
                    .precondition("cancellation_allowed")
                    .effect("refund_payment")
                    .transition_trigger("cancel"),
            )
            .state_machine(machine);

        // Validate passes with no warnings
        let warnings = service.validate().unwrap();
        assert!(
            warnings.is_empty(),
            "expected no warnings, got: {warnings:?}"
        );

        // All pieces present
        assert_eq!(service.fields.len(), 5);
        assert_eq!(service.guards.len(), 4);
        assert_eq!(service.actions.len(), 4);
        assert!(service.state_machine.is_some());

        // Serde round-trip
        let json = serde_json::to_string_pretty(&service).unwrap();
        let parsed: ServiceDef = serde_json::from_str(&json).unwrap();
        assert_eq!(service, parsed);
    }

    #[test]
    fn service_def_json_schema_includes_actions_guards() {
        let schema = schemars::schema_for!(ServiceDef);
        let value = schema.to_value();
        let props = value
            .get("properties")
            .expect("ServiceDef schema must have properties");
        let obj = props.as_object().unwrap();
        assert!(obj.contains_key("actions"), "missing 'actions' property");
        assert!(obj.contains_key("guards"), "missing 'guards' property");
    }

    // -- Phase 87-01 tests: relationships --

    use crate::relationship::{Cardinality, NavigationHint, RelationshipDef};

    #[test]
    fn service_def_with_relationships_builder() {
        let service = ServiceDef::new("order").relationship(
            RelationshipDef::new("customer", "customer", Cardinality::ManyToOne)
                .foreign_key("customer_id"),
        );

        assert_eq!(service.relationships.len(), 1);
        assert_eq!(service.relationships[0].name, "customer");
        assert_eq!(service.relationships[0].target, "customer");
        assert_eq!(service.relationships[0].cardinality, Cardinality::ManyToOne);
    }

    #[test]
    fn service_def_belongs_to_convenience() {
        let service = ServiceDef::new("order").belongs_to("customer", "customer");

        assert_eq!(service.relationships.len(), 1);
        let rel = &service.relationships[0];
        assert_eq!(rel.name, "customer");
        assert_eq!(rel.target, "customer");
        assert_eq!(rel.cardinality, Cardinality::ManyToOne);
        assert_eq!(rel.navigation, NavigationHint::Link);
    }

    #[test]
    fn service_def_has_many_convenience() {
        let service = ServiceDef::new("order").has_many("line_items", "order_line_item");

        assert_eq!(service.relationships.len(), 1);
        let rel = &service.relationships[0];
        assert_eq!(rel.name, "line_items");
        assert_eq!(rel.target, "order_line_item");
        assert_eq!(rel.cardinality, Cardinality::OneToMany);
        assert_eq!(rel.navigation, NavigationHint::Nested);
    }

    #[test]
    fn service_def_has_one_convenience() {
        let service = ServiceDef::new("user").has_one("profile", "user_profile");

        assert_eq!(service.relationships.len(), 1);
        let rel = &service.relationships[0];
        assert_eq!(rel.name, "profile");
        assert_eq!(rel.target, "user_profile");
        assert_eq!(rel.cardinality, Cardinality::OneToOne);
        assert_eq!(rel.navigation, NavigationHint::Inline);
    }

    #[test]
    fn service_def_belongs_to_many_convenience() {
        let service = ServiceDef::new("post").belongs_to_many("tags", "tag");

        assert_eq!(service.relationships.len(), 1);
        let rel = &service.relationships[0];
        assert_eq!(rel.name, "tags");
        assert_eq!(rel.target, "tag");
        assert_eq!(rel.cardinality, Cardinality::ManyToMany);
        assert_eq!(rel.navigation, NavigationHint::Nested);
    }

    #[test]
    fn service_def_json_omits_empty_relationships() {
        let service = ServiceDef::new("user");
        let json = serde_json::to_string(&service).unwrap();
        assert!(!json.contains("relationships"));
    }

    #[test]
    fn service_def_relationships_serde_round_trip() {
        let service = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .belongs_to("customer", "customer")
            .has_many("line_items", "order_line_item")
            .has_one("invoice", "invoice")
            .belongs_to_many("tags", "tag");

        let json = serde_json::to_string_pretty(&service).unwrap();
        let parsed: ServiceDef = serde_json::from_str(&json).unwrap();
        assert_eq!(service, parsed);
        assert_eq!(parsed.relationships.len(), 4);
    }

    // -- Validation tests --

    #[test]
    fn validate_warns_duplicate_relationship_names() {
        let service = ServiceDef::new("order")
            .belongs_to("customer", "customer")
            .belongs_to("customer", "other_customer");

        let warnings = service.validate().unwrap();
        assert!(warnings.contains(&Warning::DuplicateRelationship("customer".into())));
    }

    #[test]
    fn validate_warns_many_to_many_with_foreign_key() {
        let service = ServiceDef::new("post").relationship(
            RelationshipDef::new("tags", "tag", Cardinality::ManyToMany).foreign_key("tag_id"),
        );

        let warnings = service.validate().unwrap();
        assert!(warnings.contains(&Warning::ManyToManyWithForeignKey {
            relationship: "tags".into()
        }));
    }

    #[test]
    fn validate_passes_with_valid_relationships() {
        let service = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .belongs_to("customer", "customer")
            .has_many("line_items", "order_line_item");

        let warnings = service.validate().unwrap();
        assert!(
            warnings.is_empty(),
            "expected no warnings, got: {warnings:?}"
        );
    }

    #[test]
    fn order_service_with_relationships_full_example() {
        let machine = StateMachine::new("order_lifecycle")
            .initial("draft")
            .state(StateDef::new("draft").display_name("Draft"))
            .state(
                StateDef::new("submitted")
                    .display_name("Submitted")
                    .final_state(),
            )
            .transition(Transition::new("draft", "submit", "submitted").guard("has_items"));

        let service = ServiceDef::new("order")
            .display_name("Order")
            .description("Full order management with relationships")
            .read_only_field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("customer_id", DataType::Integer, FieldMeaning::ForeignKey)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .guard(GuardDef::new("has_items"))
            .action(
                ActionDef::new("submit_order")
                    .precondition("has_items")
                    .transition_trigger("submit"),
            )
            .belongs_to("customer", "customer")
            .has_many("line_items", "order_line_item")
            .has_one("invoice", "invoice")
            .state_machine(machine);

        // Validate passes with no warnings
        let warnings = service.validate().unwrap();
        assert!(
            warnings.is_empty(),
            "expected no warnings, got: {warnings:?}"
        );

        // All pieces present
        assert_eq!(service.fields.len(), 4);
        assert_eq!(service.guards.len(), 1);
        assert_eq!(service.actions.len(), 1);
        assert_eq!(service.relationships.len(), 3);
        assert!(service.state_machine.is_some());

        // Serde round-trip
        let json = serde_json::to_string_pretty(&service).unwrap();
        let parsed: ServiceDef = serde_json::from_str(&json).unwrap();
        assert_eq!(service, parsed);
    }

    #[test]
    fn service_def_json_schema_includes_relationships() {
        let schema = schemars::schema_for!(ServiceDef);
        let value = schema.to_value();
        let props = value
            .get("properties")
            .expect("ServiceDef schema must have properties");
        let obj = props.as_object().unwrap();
        assert!(
            obj.contains_key("relationships"),
            "missing 'relationships' property"
        );
    }

    // -- Phase 88-01 tests: intent hints --

    use crate::intent::{Intent, IntentHint};

    #[test]
    fn service_def_new_has_empty_intent_hints() {
        let service = ServiceDef::new("order");
        assert!(service.intent_hints.is_empty());
    }

    #[test]
    fn service_def_intent_hint_builder() {
        let service = ServiceDef::new("order")
            .intent_hint(IntentHint::Primary(Intent::Browse))
            .intent_hint(IntentHint::Exclude(Intent::Process));

        assert_eq!(service.intent_hints.len(), 2);
        assert_eq!(service.intent_hints[0], IntentHint::Primary(Intent::Browse));
        assert_eq!(
            service.intent_hints[1],
            IntentHint::Exclude(Intent::Process)
        );
    }

    #[test]
    fn service_def_json_omits_empty_intent_hints() {
        let service = ServiceDef::new("user");
        let json = serde_json::to_string(&service).unwrap();
        assert!(!json.contains("intent_hints"));
    }

    #[test]
    fn service_def_intent_hints_serde_round_trip() {
        let service = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .intent_hint(IntentHint::Primary(Intent::Browse))
            .intent_hint(IntentHint::Exclude(Intent::Collect));

        let json = serde_json::to_string_pretty(&service).unwrap();
        let parsed: ServiceDef = serde_json::from_str(&json).unwrap();
        assert_eq!(service, parsed);
        assert_eq!(parsed.intent_hints.len(), 2);
    }

    #[test]
    fn validate_passes_with_valid_intent_hints() {
        let service = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .intent_hint(IntentHint::Primary(Intent::Browse))
            .intent_hint(IntentHint::Exclude(Intent::Collect));

        let warnings = service.validate().unwrap();
        assert!(
            warnings.is_empty(),
            "expected no warnings, got: {warnings:?}"
        );
    }

    #[test]
    fn validate_warns_conflicting_intent_hints() {
        let service = ServiceDef::new("order")
            .intent_hint(IntentHint::Primary(Intent::Browse))
            .intent_hint(IntentHint::Exclude(Intent::Browse));

        let warnings = service.validate().unwrap();
        assert!(warnings.contains(&Warning::ConflictingIntentHints {
            intent: "browse".into()
        }));
    }

    #[test]
    fn validate_warns_multiple_primary_intent_hints() {
        let service = ServiceDef::new("order")
            .intent_hint(IntentHint::Primary(Intent::Browse))
            .intent_hint(IntentHint::Primary(Intent::Focus));

        let warnings = service.validate().unwrap();
        assert!(warnings.contains(&Warning::MultiplePrimaryIntentHints));
    }

    #[test]
    fn validate_warns_both_conflicting_and_multiple_primary() {
        let service = ServiceDef::new("order")
            .intent_hint(IntentHint::Primary(Intent::Browse))
            .intent_hint(IntentHint::Primary(Intent::Focus))
            .intent_hint(IntentHint::Exclude(Intent::Browse));

        let warnings = service.validate().unwrap();
        assert!(warnings.contains(&Warning::ConflictingIntentHints {
            intent: "browse".into()
        }));
        assert!(warnings.contains(&Warning::MultiplePrimaryIntentHints));
    }

    #[test]
    fn validate_no_warning_for_single_primary() {
        let service = ServiceDef::new("order").intent_hint(IntentHint::Primary(Intent::Browse));

        let warnings = service.validate().unwrap();
        assert!(
            warnings.is_empty(),
            "expected no warnings, got: {warnings:?}"
        );
    }

    #[test]
    fn service_def_json_schema_includes_intent_hints() {
        let schema = schemars::schema_for!(ServiceDef);
        let value = schema.to_value();
        let props = value
            .get("properties")
            .expect("ServiceDef schema must have properties");
        let obj = props.as_object().unwrap();
        assert!(
            obj.contains_key("intent_hints"),
            "missing 'intent_hints' property"
        );
    }

    // -- Phase 88-02 tests: full integration with intent hints --

    #[test]
    fn full_service_with_intent_hints() {
        let machine = StateMachine::new("order_lifecycle")
            .initial("draft")
            .state(StateDef::new("draft").display_name("Draft"))
            .state(
                StateDef::new("submitted")
                    .display_name("Submitted")
                    .on_enter(vec!["validate_inventory"]),
            )
            .state(
                StateDef::new("shipped")
                    .display_name("Shipped")
                    .final_state(),
            )
            .state(
                StateDef::new("cancelled")
                    .display_name("Cancelled")
                    .final_state(),
            )
            .transition(Transition::new("draft", "submit", "submitted").guard("has_items"))
            .transition(
                Transition::new("submitted", "ship", "shipped").guard("inventory_fulfilled"),
            )
            .transition(
                Transition::new("draft", "cancel", "cancelled").guard("cancellation_allowed"),
            )
            .transition(
                Transition::new("submitted", "cancel", "cancelled").guard("cancellation_allowed"),
            );

        let service = ServiceDef::new("order")
            .display_name("Order")
            .description("Full order management with all features including intent hints")
            .read_only_field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("customer_id", DataType::Integer, FieldMeaning::ForeignKey)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .read_only_field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .guard(GuardDef::new("has_items"))
            .guard(GuardDef::new("inventory_fulfilled"))
            .guard(GuardDef::new("cancellation_allowed"))
            .action(
                ActionDef::new("submit_order")
                    .precondition("has_items")
                    .transition_trigger("submit"),
            )
            .action(
                ActionDef::new("ship_order")
                    .precondition("inventory_fulfilled")
                    .transition_trigger("ship"),
            )
            .action(
                ActionDef::new("cancel_order")
                    .precondition("cancellation_allowed")
                    .transition_trigger("cancel"),
            )
            .belongs_to("customer", "customer")
            .has_many("line_items", "order_line_item")
            .has_one("invoice", "invoice")
            .intent_hint(IntentHint::Primary(Intent::Process))
            .intent_hint(IntentHint::Exclude(Intent::Summarize))
            .state_machine(machine);

        // Validate passes with no warnings
        let warnings = service.validate().unwrap();
        assert!(
            warnings.is_empty(),
            "expected no warnings, got: {warnings:?}"
        );

        // All pieces present
        assert_eq!(service.fields.len(), 5);
        assert_eq!(service.guards.len(), 3);
        assert_eq!(service.actions.len(), 3);
        assert_eq!(service.relationships.len(), 3);
        assert_eq!(service.intent_hints.len(), 2);
        assert!(service.state_machine.is_some());

        // Intent hints correct
        assert_eq!(
            service.intent_hints[0],
            IntentHint::Primary(Intent::Process)
        );
        assert_eq!(
            service.intent_hints[1],
            IntentHint::Exclude(Intent::Summarize)
        );

        // Serde round-trip
        let json = serde_json::to_string_pretty(&service).unwrap();
        let parsed: ServiceDef = serde_json::from_str(&json).unwrap();
        assert_eq!(service, parsed);
    }
}
