//! Pure schema-level derivation from a [`ServiceDef`].
//!
//! [`schema_contract`] is the schema sibling of [`crate::derive::derive_intents`]:
//! it projects field access modes, meanings, action definitions (preconditions + inputs),
//! and declared guards — and renders nothing.

use serde::{Deserialize, Serialize};

use crate::action::{ActionDef, InputDef};
use crate::field::{DataType, FieldDef, FieldMeaning};
use crate::service::ServiceDef;

/// Serializable contract describing the full schema of a [`ServiceDef`].
///
/// Produced by [`schema_contract`]. Contains no runtime state; suitable for
/// JSON serialization and for feeding any output renderer that needs the
/// structural description of a service without querying data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaContract {
    /// Canonical service name (lower-snake, matches [`ServiceDef::name`]).
    pub name: String,
    /// Human-readable label, if declared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// All fields declared on the service, in declaration order.
    pub fields: Vec<FieldContract>,
    /// All actions declared on the service, in declaration order.
    pub actions: Vec<ActionContract>,
    /// Names of all guards declared on the service.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guards: Vec<String>,
    /// `true` when the service declares a [`crate::state::StateMachine`].
    pub has_state_machine: bool,
}

/// Per-field projection of a [`FieldDef`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldContract {
    pub name: String,
    pub data_type: DataType,
    pub meaning: FieldMeaning,
    /// `true` when the field is required (not nullable).
    pub required: bool,
    /// `true` when the field may be included in read results.
    pub readable: bool,
    /// `true` when the field may be included in write inputs.
    pub writable: bool,
    /// `true` when the field holds a list of values rather than a scalar.
    pub is_list: bool,
}

impl From<&FieldDef> for FieldContract {
    fn from(f: &FieldDef) -> Self {
        FieldContract {
            name: f.name.clone(),
            data_type: f.data_type,
            meaning: f.meaning.clone(),
            required: f.required,
            readable: f.readable,
            writable: f.writable,
            is_list: f.is_list,
        }
    }
}

/// Per-action projection of an [`ActionDef`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionContract {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Guard names that must not be `false` for this action to be permitted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preconditions: Vec<String>,
    /// Input parameters the action accepts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<InputContract>,
    /// `true` when the action is linked to a state machine transition.
    pub is_transition: bool,
}

impl From<&ActionDef> for ActionContract {
    fn from(a: &ActionDef) -> Self {
        ActionContract {
            name: a.name.clone(),
            display_name: a.display_name.clone(),
            description: a.description.clone(),
            preconditions: a.preconditions.clone(),
            inputs: a.inputs.iter().map(InputContract::from).collect(),
            is_transition: a.transition_trigger.is_some(),
        }
    }
}

/// Per-input projection of an [`InputDef`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputContract {
    pub name: String,
    pub data_type: DataType,
    pub meaning: FieldMeaning,
    pub required: bool,
}

impl From<&InputDef> for InputContract {
    fn from(i: &InputDef) -> Self {
        InputContract {
            name: i.name.clone(),
            data_type: i.data_type,
            meaning: i.meaning.clone(),
            required: i.required,
        }
    }
}

/// Derives the schema contract from a service definition.
///
/// Pure derivation — no runtime deps, no async, no side effects. Renders nothing.
/// Sibling of [`crate::derive::derive_intents`]. Describes fields (with access modes +
/// meanings), action definitions (preconditions + inputs), and declared guards.
///
/// # Example
///
/// ```
/// use ferro_projections::{schema_contract, ServiceDef, DataType, FieldMeaning};
///
/// let service = ServiceDef::new("order")
///     .field("total", DataType::Float, FieldMeaning::Money);
/// let contract = schema_contract(&service);
/// assert_eq!(contract.name, "order");
/// assert_eq!(contract.fields.len(), 1);
/// ```
pub fn schema_contract(service: &ServiceDef) -> SchemaContract {
    SchemaContract {
        name: service.name.clone(),
        display_name: service.display_name.clone(),
        fields: service.fields.iter().map(FieldContract::from).collect(),
        actions: service.actions.iter().map(ActionContract::from).collect(),
        guards: service.guards.iter().map(|g| g.name.clone()).collect(),
        has_state_machine: service.state_machine.is_some(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ActionDef, DataType, FieldMeaning, GuardDef, ServiceDef};

    #[test]
    fn schema_contract_field_set() {
        let service = ServiceDef::new("order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("total", DataType::Float, FieldMeaning::Money);
        let contract = schema_contract(&service);
        assert_eq!(contract.name, "order");
        assert_eq!(contract.fields.len(), 2);
        assert_eq!(contract.fields[0].name, "id");
        assert_eq!(contract.fields[1].name, "total");
    }

    #[test]
    fn read_only_field_has_correct_access_flags() {
        let service = ServiceDef::new("order").read_only_field(
            "id",
            DataType::Integer,
            FieldMeaning::Identifier,
        );
        let contract = schema_contract(&service);
        let id = &contract.fields[0];
        assert!(!id.writable, "read-only field must not be writable");
        assert!(id.readable, "read-only field must be readable");
    }

    #[test]
    fn action_preconditions_and_transition() {
        let service = ServiceDef::new("order")
            .guard(GuardDef::new("is_manager"))
            .action(
                ActionDef::new("approve")
                    .precondition("is_manager")
                    .transition_trigger("approve"),
            )
            .action(ActionDef::new("submit"));
        let contract = schema_contract(&service);
        assert_eq!(contract.guards, vec!["is_manager"]);
        let approve = &contract.actions[0];
        assert_eq!(approve.preconditions, vec!["is_manager"]);
        assert!(approve.is_transition);
        let submit = &contract.actions[1];
        assert!(!submit.is_transition);
    }

    #[test]
    fn schema_contract_serde_round_trip() {
        let service =
            ServiceDef::new("order").field("id", DataType::Integer, FieldMeaning::Identifier);
        let contract = schema_contract(&service);
        let json = serde_json::to_string(&contract).unwrap();
        let parsed: SchemaContract = serde_json::from_str(&json).unwrap();
        assert_eq!(contract.name, parsed.name);
        assert_eq!(contract.fields.len(), parsed.fields.len());
    }
}
