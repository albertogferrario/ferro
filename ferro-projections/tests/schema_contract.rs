//! Snapshot test for `schema_contract` (SUBST-01).
//!
//! Three tests covering field access modes, action preconditions + guards,
//! and serde round-trip losslessness.

use ferro_projections::{schema_contract, ActionDef, DataType, FieldMeaning, GuardDef, ServiceDef};

#[test]
fn schema_contract_field_names_and_access() {
    let service = ServiceDef::new("order")
        .read_only_field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("customer_name", DataType::String, FieldMeaning::EntityName)
        .optional_field("notes", DataType::String, FieldMeaning::FreeText);

    let contract = schema_contract(&service);
    assert_eq!(contract.fields.len(), 3);

    let id = &contract.fields[0];
    assert_eq!(id.name, "id");
    assert!(!id.writable, "id must not be writable (read-only field)");
    assert!(id.readable, "id must be readable");

    let name = &contract.fields[1];
    assert_eq!(name.name, "customer_name");
    assert!(name.writable, "customer_name must be writable");
    assert!(name.readable, "customer_name must be readable");
}

#[test]
fn schema_contract_actions_and_preconditions() {
    let service = ServiceDef::new("order")
        .guard(GuardDef::new("is_manager"))
        .action(ActionDef::new("approve").precondition("is_manager"))
        .action(ActionDef::new("submit"));

    let contract = schema_contract(&service);
    assert_eq!(contract.actions.len(), 2);
    assert_eq!(contract.guards, vec!["is_manager"]);

    let approve = &contract.actions[0];
    assert_eq!(approve.name, "approve");
    assert_eq!(approve.preconditions, vec!["is_manager"]);

    let submit = &contract.actions[1];
    assert_eq!(submit.name, "submit");
    assert!(submit.preconditions.is_empty());
}

#[test]
fn schema_contract_serde_round_trip() {
    let service = ServiceDef::new("order")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .guard(GuardDef::new("g"))
        .action(ActionDef::new("act").precondition("g"));
    let c = schema_contract(&service);
    let json = serde_json::to_string(&c).unwrap();
    let parsed: ferro_projections::SchemaContract = serde_json::from_str(&json).unwrap();
    assert_eq!(c.name, parsed.name);
    assert_eq!(c.fields.len(), parsed.fields.len());
    assert_eq!(c.guards, parsed.guards);
    assert_eq!(c.actions.len(), parsed.actions.len());
}
