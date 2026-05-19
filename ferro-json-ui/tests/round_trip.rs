//! Round-trip and builder-parity integration tests for [`Spec`].
//!
//! Each fixture under `tests/fixtures/ok/` MUST parse via [`Spec::from_json`]
//! and then re-parse into an equal [`Spec`] after re-serialization. The
//! builder-parity test verifies [`Spec::builder`] produces an output equal
//! to the fixture parse result (D-31).

use std::fs;

use ferro_json_ui::{Element, Spec};

fn fixture(path: &str) -> String {
    fs::read_to_string(format!("tests/fixtures/{path}"))
        .unwrap_or_else(|e| panic!("failed to read tests/fixtures/{path}: {e}"))
}

fn assert_round_trip(fixture_path: &str) {
    let json = fixture(fixture_path);
    let spec1 = Spec::from_json(&json).expect("fixture must parse");
    let reserialized = serde_json::to_string(&spec1).expect("serialize must succeed");
    let spec2 = Spec::from_json(&reserialized).expect("reserialized must parse");
    assert_eq!(spec1, spec2, "round-trip inequality in {fixture_path}");
}

#[test]
fn ok_minimal_round_trips() {
    assert_round_trip("ok/minimal_single_element.json");
}

#[test]
fn ok_three_level_nested_round_trips() {
    assert_round_trip("ok/three_level_nested.json");
}

#[test]
fn ok_with_actions_round_trips() {
    assert_round_trip("ok/with_actions.json");
}

#[test]
fn ok_with_visibility_round_trips() {
    assert_round_trip("ok/with_visibility.json");
}

#[test]
fn ok_with_plugin_named_type_round_trips() {
    assert_round_trip("ok/with_plugin_named_type.json");
}

#[test]
fn ok_with_data_payload_round_trips() {
    assert_round_trip("ok/with_data_payload.json");
}

#[test]
fn ok_omitted_optional_fields_round_trips() {
    assert_round_trip("ok/omitted_optional_fields.json");
}

/// D-31 — [`Spec::builder`] produces identical output to the fixture's `from_json`.
#[test]
fn builder_parity_minimal() {
    let from_json = Spec::from_json(&fixture("ok/minimal_single_element.json")).unwrap();
    let from_builder = Spec::builder()
        .element("a", Element::new("Text").prop("content", "Hi"))
        .build()
        .unwrap();
    assert_eq!(from_json, from_builder);
}
