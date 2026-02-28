use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::field::{DataType, FieldDef, FieldMeaning};
use crate::state::StateMachine;

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

    /// Sets the state machine definition for this service.
    pub fn state_machine(mut self, machine: StateMachine) -> Self {
        self.state_machine = Some(machine);
        self
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
        let service = ServiceDef::new("user")
            .write_only_field("password", DataType::String, FieldMeaning::Sensitive);

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
}
