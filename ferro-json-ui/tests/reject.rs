//! Validation-failure integration tests.
//!
//! Every fixture under `tests/fixtures/reject/` MUST fail [`Spec::from_json`]
//! with a specific [`SpecError`] variant. The assertions pin the exact
//! structural contract of Plan 115-01.

use std::fs;

use ferro_json_ui::{Spec, SpecError};

fn fixture(path: &str) -> String {
    fs::read_to_string(format!("tests/fixtures/{path}"))
        .unwrap_or_else(|e| panic!("failed to read tests/fixtures/{path}: {e}"))
}

#[test]
fn reject_missing_root() {
    let json = fixture("reject/missing_root.json");
    match Spec::from_json(&json) {
        Err(SpecError::RootMissing(id)) => assert_eq!(id, "nope"),
        other => panic!("expected RootMissing, got {other:?}"),
    }
}

#[test]
fn reject_dangling_child() {
    let json = fixture("reject/dangling_child.json");
    match Spec::from_json(&json) {
        Err(SpecError::DanglingChild { element, child }) => {
            assert_eq!(element, "root");
            assert_eq!(child, "ghost");
        }
        other => panic!("expected DanglingChild, got {other:?}"),
    }
}

#[test]
fn reject_simple_cycle() {
    let json = fixture("reject/simple_cycle.json");
    match Spec::from_json(&json) {
        Err(SpecError::Cycle { path }) => {
            assert!(
                path.len() >= 3,
                "cycle path must contain at least start -> mid -> start, got {path:?}"
            );
            assert_eq!(path.first(), path.last());
        }
        other => panic!("expected Cycle, got {other:?}"),
    }
}

#[test]
fn reject_self_cycle() {
    let json = fixture("reject/self_cycle.json");
    match Spec::from_json(&json) {
        Err(SpecError::Cycle { path }) => {
            assert_eq!(path, vec!["A".to_string(), "A".to_string()]);
        }
        other => panic!("expected Cycle (self), got {other:?}"),
    }
}

#[test]
fn reject_six_level_nesting() {
    // Seventeen levels (root + 16 children chain): one past MAX_NESTING_DEPTH=16.
    // Fixture file kept under its original name for test-runner stability.
    let json = fixture("reject/six_level_nesting.json");
    match Spec::from_json(&json) {
        Err(SpecError::DepthExceeded { max, found, path }) => {
            assert_eq!(max, 16, "max must equal MAX_NESTING_DEPTH=16");
            assert_eq!(found, 17, "found must be 17 (one past the limit)");
            assert!(!path.is_empty());
        }
        other => panic!("expected DepthExceeded, got {other:?}"),
    }
}

#[test]
fn reject_invalid_id_space() {
    let json = fixture("reject/invalid_id_space.json");
    match Spec::from_json(&json) {
        Err(SpecError::InvalidId(id)) => assert_eq!(id, "user form"),
        other => panic!("expected InvalidId(space), got {other:?}"),
    }
}

#[test]
fn reject_invalid_id_empty() {
    let json = fixture("reject/invalid_id_empty.json");
    match Spec::from_json(&json) {
        Err(SpecError::InvalidId(id)) => assert_eq!(id, ""),
        other => panic!("expected InvalidId(empty), got {other:?}"),
    }
}

#[test]
fn reject_invalid_id_digit_start() {
    let json = fixture("reject/invalid_id_digit_start.json");
    match Spec::from_json(&json) {
        Err(SpecError::InvalidId(id)) => assert_eq!(id, "1form"),
        other => panic!("expected InvalidId(digit), got {other:?}"),
    }
}

#[test]
fn reject_invalid_id_too_long() {
    let json = fixture("reject/invalid_id_too_long.json");
    match Spec::from_json(&json) {
        Err(SpecError::InvalidId(id)) => assert_eq!(id.len(), 129),
        other => panic!("expected InvalidId(too long), got {other:?}"),
    }
}

#[test]
fn reject_invalid_child_ref_format() {
    let json = fixture("reject/invalid_child_ref_format.json");
    match Spec::from_json(&json) {
        Err(SpecError::InvalidId(id)) => assert_eq!(id, "user form"),
        other => panic!("expected InvalidId from child ref, got {other:?}"),
    }
}

#[test]
fn reject_duplicate_id() {
    let json = fixture("reject/duplicate_id.json");
    match Spec::from_json(&json) {
        Err(SpecError::DuplicateId(id)) => assert_eq!(id, "a"),
        other => panic!("expected DuplicateId, got {other:?}"),
    }
}
