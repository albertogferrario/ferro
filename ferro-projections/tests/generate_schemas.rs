use std::fs;
use std::path::PathBuf;

use ferro_projections::{
    ActionDef, Cardinality, DataType, FieldDef, FieldMeaning, GuardDef, InputDef, Intent,
    IntentHint, IntentScore, NavigationHint, RelationshipDef, ServiceDef, StateDef, StateMachine,
    Transition, Warning,
};
use serde_json::{Map, Value};

const SCHEMA_DATE: &str = "2026-03-01";
const BASE_URL: &str = "https://ferro-rs.dev/protocol";

fn schemas_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .join("..")
        .join("docs")
        .join("protocol")
        .join("schemas")
}

fn generate_schema(type_name: &str) -> Value {
    let mut value = match type_name {
        "service-def" => schemars::schema_for!(ServiceDef).to_value(),
        "field-def" => schemars::schema_for!(FieldDef).to_value(),
        "data-type" => schemars::schema_for!(DataType).to_value(),
        "field-meaning" => schemars::schema_for!(FieldMeaning).to_value(),
        "state-machine" => schemars::schema_for!(StateMachine).to_value(),
        "state-def" => schemars::schema_for!(StateDef).to_value(),
        "transition" => schemars::schema_for!(Transition).to_value(),
        "warning" => schemars::schema_for!(Warning).to_value(),
        "action-def" => schemars::schema_for!(ActionDef).to_value(),
        "input-def" => schemars::schema_for!(InputDef).to_value(),
        "guard-def" => schemars::schema_for!(GuardDef).to_value(),
        "relationship-def" => schemars::schema_for!(RelationshipDef).to_value(),
        "cardinality" => schemars::schema_for!(Cardinality).to_value(),
        "navigation-hint" => schemars::schema_for!(NavigationHint).to_value(),
        "intent" => schemars::schema_for!(Intent).to_value(),
        "intent-score" => schemars::schema_for!(IntentScore).to_value(),
        "intent-hint" => schemars::schema_for!(IntentHint).to_value(),
        _ => panic!("unknown type: {type_name}"),
    };

    // Inject $id into the schema
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "$id".to_string(),
            Value::String(format!("{BASE_URL}/{SCHEMA_DATE}/{type_name}.json")),
        );
    }

    value
}

const ALL_TYPES: &[&str] = &[
    "service-def",
    "field-def",
    "data-type",
    "field-meaning",
    "state-machine",
    "state-def",
    "transition",
    "warning",
    "action-def",
    "input-def",
    "guard-def",
    "relationship-def",
    "cardinality",
    "navigation-hint",
    "intent",
    "intent-score",
    "intent-hint",
];

#[test]
fn generate_protocol_schemas() {
    let dir = schemas_dir();
    fs::create_dir_all(&dir).expect("failed to create schemas directory");

    let mut combined_defs = Map::new();

    for type_name in ALL_TYPES {
        let schema = generate_schema(type_name);

        // Write individual schema file
        let path = dir.join(format!("{type_name}.json"));
        let json = serde_json::to_string_pretty(&schema).expect("failed to serialize schema");
        fs::write(&path, format!("{json}\n")).expect("failed to write schema file");

        // Collect into combined $defs
        combined_defs.insert(type_name.to_string(), schema);
    }

    // Generate combined protocol.json
    let combined = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("{BASE_URL}/{SCHEMA_DATE}/protocol.json"),
        "title": "Ferro Projections Protocol",
        "description": "Combined schema for all ferro-projections protocol types.",
        "$defs": combined_defs,
    });

    let combined_path = dir.join("protocol.json");
    let combined_json =
        serde_json::to_string_pretty(&combined).expect("failed to serialize combined schema");
    fs::write(&combined_path, format!("{combined_json}\n"))
        .expect("failed to write combined schema");

    // Verify all files exist
    for type_name in ALL_TYPES {
        let path = dir.join(format!("{type_name}.json"));
        assert!(path.exists(), "missing schema file: {}", path.display());
    }
    assert!(
        combined_path.exists(),
        "missing combined schema: {}",
        combined_path.display()
    );

    // Verify individual schemas have $id and $schema
    for type_name in ALL_TYPES {
        let path = dir.join(format!("{type_name}.json"));
        let content = fs::read_to_string(&path).expect("failed to read schema");
        let value: Value = serde_json::from_str(&content).expect("invalid JSON");
        let obj = value.as_object().expect("schema must be object");

        assert!(obj.contains_key("$id"), "{type_name}.json missing $id");

        let id = obj["$id"].as_str().unwrap();
        assert!(
            id.starts_with(BASE_URL),
            "{type_name}.json $id doesn't start with base URL: {id}"
        );
    }
}
