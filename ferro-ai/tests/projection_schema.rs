//! SC#3 structural-guarantee tests.
//!
//! Verifies that the ServiceDef-aware normalized schema rejects invalid `FieldMeaning`
//! and `Intent` values and accepts valid ones, using `jsonschema::draft202012`.
//!
//! These tests are offline (no network). They operate on the output of
//! `schemars::schema_for!(ServiceDef)` passed through `ferro_ai::schema::for_structured_output`.

use ferro_projections::ServiceDef;
use schemars::schema_for;
use serde_json::json;

/// Normalize the real ServiceDef schema using the ServiceDef-aware path.
fn normalized_servicedef_schema() -> serde_json::Value {
    let raw = serde_json::to_value(schema_for!(ServiceDef)).unwrap();
    ferro_ai::schema::for_structured_output(raw)
}

// ── FieldMeaning tests ────────────────────────────────────────────────────────

/// SC#3: an invalid `FieldMeaning` value fails validation against the normalized schema.
///
/// The ServiceDef-aware path closes `FieldMeaning` to the 18 known snake_case variants.
/// `"totally_bogus"` is not among them, so validation must fail (D-06, T-166-PI-01).
#[test]
fn servicedef_schema_rejects_invalid_field_meaning() {
    let schema = normalized_servicedef_schema();
    let validator =
        jsonschema::draft202012::new(&schema).expect("normalized ServiceDef schema must compile");

    // Invalid: "totally_bogus" is not a known FieldMeaning variant.
    let invalid = json!({
        "name": "order",
        "fields": [{
            "name": "total",
            "data_type": "float",
            "meaning": "totally_bogus"
        }]
    });
    assert!(
        validator.validate(&invalid).is_err(),
        "invalid FieldMeaning 'totally_bogus' must fail validation"
    );

    // Valid: "money" is a known FieldMeaning variant.
    let valid = json!({
        "name": "order",
        "fields": [{
            "name": "total",
            "data_type": "float",
            "meaning": "money"
        }]
    });
    assert!(
        validator.validate(&valid).is_ok(),
        "valid FieldMeaning 'money' must pass validation"
    );
}

/// SC#3: a minimal ServiceDef with no fields is valid.
///
/// `fields` has no default in schemars but an empty array is still a valid `Vec<FieldDef>`.
#[test]
fn servicedef_schema_accepts_minimal_servicedef() {
    let schema = normalized_servicedef_schema();
    let validator =
        jsonschema::draft202012::new(&schema).expect("normalized ServiceDef schema must compile");

    let minimal = json!({ "name": "user", "fields": [] });
    assert!(
        validator.validate(&minimal).is_ok(),
        "minimal ServiceDef with empty fields must pass validation; got error"
    );
}

// ── Intent tests ─────────────────────────────────────────────────────────────

/// SC#3: an invalid `Intent` value in `intent_hints` fails validation.
///
/// `IntentHint` is externally tagged: `{"primary": <Intent>}` or `{"exclude": <Intent>}`.
/// After closing, `Intent` only allows the 7 known snake_case variants.
#[test]
fn servicedef_schema_rejects_invalid_intent() {
    let schema = normalized_servicedef_schema();
    let validator =
        jsonschema::draft202012::new(&schema).expect("normalized ServiceDef schema must compile");

    // Invalid: "totally_bogus_intent" is not a known Intent variant.
    let invalid = json!({
        "name": "order",
        "fields": [],
        "intent_hints": [{ "primary": "totally_bogus_intent" }]
    });
    assert!(
        validator.validate(&invalid).is_err(),
        "invalid Intent 'totally_bogus_intent' must fail validation"
    );

    // Valid: "browse" is a known Intent variant.
    let valid = json!({
        "name": "order",
        "fields": [],
        "intent_hints": [{ "primary": "browse" }]
    });
    assert!(
        validator.validate(&valid).is_ok(),
        "valid Intent 'browse' must pass validation"
    );
}

/// SC#3: all 7 known Intent variants are accepted.
#[test]
fn servicedef_schema_accepts_all_known_intent_variants() {
    let schema = normalized_servicedef_schema();
    let validator =
        jsonschema::draft202012::new(&schema).expect("normalized ServiceDef schema must compile");

    for intent in [
        "browse",
        "focus",
        "collect",
        "process",
        "summarize",
        "analyze",
        "track",
    ] {
        let instance = json!({
            "name": "service",
            "fields": [],
            "intent_hints": [{ "primary": intent }]
        });
        assert!(
            validator.validate(&instance).is_ok(),
            "known Intent variant '{intent}' must pass validation"
        );
    }
}

/// SC#3: all 18 known FieldMeaning variants are accepted.
#[test]
fn servicedef_schema_accepts_all_known_field_meaning_variants() {
    let schema = normalized_servicedef_schema();
    let validator =
        jsonschema::draft202012::new(&schema).expect("normalized ServiceDef schema must compile");

    let known_meanings = [
        "identifier",
        "foreign_key",
        "entity_name",
        "email",
        "phone",
        "url",
        "image_url",
        "money",
        "percentage",
        "quantity",
        "status",
        "category",
        "boolean",
        "free_text",
        "created_at",
        "updated_at",
        "date_time",
        "sensitive",
    ];

    for meaning in known_meanings {
        let instance = json!({
            "name": "service",
            "fields": [{ "name": "f", "data_type": "string", "meaning": meaning }]
        });
        assert!(
            validator.validate(&instance).is_ok(),
            "known FieldMeaning variant '{meaning}' must pass validation"
        );
    }
}

/// Regression guard: the normalized schema must have no surviving `$ref` pointers.
///
/// Any surviving `$ref` would cause `jsonschema::draft202012::new` to fail with
/// `PointerToNowhere` once `$defs` is removed, breaking all SC#3 validation tests.
#[test]
fn normalized_schema_has_no_surviving_refs() {
    let schema = normalized_servicedef_schema();
    let schema_str = serde_json::to_string_pretty(&schema).unwrap();
    let surviving: Vec<&str> = schema_str.lines().filter(|l| l.contains("$ref")).collect();
    assert!(
        surviving.is_empty(),
        "Surviving $refs found in normalized schema (breaks jsonschema compilation):\n{}",
        surviving.join("\n")
    );
}
