//! Application tests.

use ferro_json_ui::design::lint;
use ferro_json_ui::spec::Spec;

/// Every JSON-UI view must declare a valid `design.intent` and lint clean.
#[test]
fn all_views_lint_clean() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/views");
    let mut checked = 0;
    for entry in std::fs::read_dir(dir)
        .expect("views dir must exist")
        .flatten()
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let content = std::fs::read_to_string(&path).unwrap();
        let spec =
            Spec::from_json(&content).unwrap_or_else(|e| panic!("parse {}: {e:?}", path.display()));
        let findings = lint(&spec);
        assert!(
            findings.is_empty(),
            "{}: {} finding(s)\n{:#?}",
            path.display(),
            findings.len(),
            findings
        );
        checked += 1;
    }
    assert!(
        checked >= 9,
        "expected all views to be linted, only saw {checked}"
    );
}

/// Each service projection must derive at least one intent — the core
/// projection / intent abstraction working end to end.
#[test]
fn projections_derive_intents() {
    for svc in crate::projections::all() {
        let scores = ferro_projections::derive_intents(&svc);
        assert!(!scores.is_empty(), "a projection derived no intents");
    }
}

/// Product principle guard: Nearly has no messaging surface. No view may
/// introduce a chat component or a free-text "message" field — the only
/// signal between users is the (wordless) trillo.
#[test]
fn no_chat_surface() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/views");
    for entry in std::fs::read_dir(dir)
        .expect("views dir must exist")
        .flatten()
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let raw = std::fs::read_to_string(&path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        if let Some(elements) = v["elements"].as_object() {
            for (id, el) in elements {
                let ty = el["type"].as_str().unwrap_or("");
                assert_ne!(
                    ty,
                    "Chat",
                    "{}: element '{id}' is a Chat component",
                    path.display()
                );
                let field = el["props"]["field"].as_str().unwrap_or("");
                assert!(
                    !matches!(field, "message" | "messaggio" | "chat"),
                    "{}: element '{id}' introduces a message field",
                    path.display()
                );
            }
        }
    }
}
