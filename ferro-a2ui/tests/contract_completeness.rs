//! Every data binding emitted by any archetype must appear in the
//! DataContract — the host fills the data model from the contract alone.

use ferro_a2ui::{A2uiContext, A2uiMessage, A2uiRenderer, CatalogTier};
use ferro_projections::render::Renderer;
use ferro_projections::{
    ActionDef, DataType, FieldMeaning, Intent, IntentScore, ServiceDef, StateDef, StateMachine,
    Transition,
};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};

// Duplicated fixture (test_support is crate-private; integration tests build
// their own — keep in sync with src/test_support.rs).
fn order_service() -> ServiceDef {
    ServiceDef::new("order")
        .display_name("Order")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("customer_name", DataType::String, FieldMeaning::EntityName)
        .field("total", DataType::Float, FieldMeaning::Money)
        .field("status", DataType::String, FieldMeaning::Status)
        .optional_field("notes", DataType::String, FieldMeaning::FreeText)
        .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
        .action(
            ActionDef::new("mark_paid")
                .display_name("Mark Paid")
                .precondition("is_manager"),
        )
        .action(ActionDef::new("archive").display_name("Archive"))
        .state_machine(
            StateMachine::new("order_lifecycle")
                .initial("new")
                .state(state("new"))
                .state(state("paid"))
                .state(state("done"))
                .transition(Transition {
                    from: "new".into(),
                    event: "mark_paid".into(),
                    to: "paid".into(),
                    guard: Some("is_manager".into()),
                    actions: vec![],
                    description: None,
                })
                .transition(Transition {
                    from: "paid".into(),
                    event: "archive".into(),
                    to: "done".into(),
                    guard: None,
                    actions: vec![],
                    description: None,
                }),
        )
        .creatable(true)
        .updatable(true)
        .deletable(true)
        .mcp_write_ability("orders.write")
}

fn state(name: &str) -> StateDef {
    StateDef {
        name: name.into(),
        display_name: None,
        description: None,
        is_final: name == "done",
        on_enter: vec![],
        on_exit: vec![],
        metadata: None,
    }
}

/// Collects every bound path, resolving relative bindings through template
/// scopes: a component reached via `children: {path, componentId}` resolves
/// relative paths as `<list path>/*/<relative>`.
fn collect_paths(components: &[(String, Value)]) -> BTreeSet<String> {
    let index: HashMap<&str, &Value> = components.iter().map(|(id, v)| (id.as_str(), v)).collect();
    let mut out = BTreeSet::new();
    for (_, v) in components {
        collect_from_value(v, None, &index, &mut out, false);
    }
    out
}

fn collect_from_value(
    v: &Value,
    scope: Option<&str>,
    index: &HashMap<&str, &Value>,
    out: &mut BTreeSet<String>,
    in_template: bool,
) {
    match v {
        Value::Object(map) => {
            if let (Some(Value::String(path)), Some(Value::String(component_id))) =
                (map.get("path"), map.get("componentId"))
            {
                // Template binding: walk the template subtree under the list scope.
                if let Some(template) = index.get(component_id.as_str()) {
                    collect_from_value(template, Some(path), index, out, true);
                }
                out.insert(path.clone());
                return;
            }
            if let Some(Value::String(path)) = map.get("path") {
                if map.len() == 1 {
                    if path.starts_with('/') {
                        out.insert(path.clone());
                    } else if let Some(scope) = scope {
                        out.insert(format!("{scope}/*/{path}"));
                    } else if in_template {
                        out.insert(path.clone());
                    }
                    return;
                }
            }
            for val in map.values() {
                collect_from_value(val, scope, index, out, in_template);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_from_value(item, scope, index, out, in_template);
            }
        }
        _ => {}
    }
}

/// Lane-scoped wildcards: per-state lists are contracted either exactly
/// (`/lanes/0/items`) or via the shared row wildcard (`/lanes/*/items/*/<f>`);
/// normalize indices after `lanes` so both spellings resolve.
fn normalize(p: &str) -> String {
    let mut parts: Vec<String> = p.split('/').map(str::to_string).collect();
    for i in 0..parts.len() {
        if i > 0 && parts[i - 1] == "lanes" && parts[i].parse::<usize>().is_ok() {
            parts[i] = "*".into();
        }
    }
    parts.join("/")
}

#[test]
fn every_emitted_binding_is_in_the_contract() {
    let intents = [
        Intent::Browse,
        Intent::Focus,
        Intent::Collect,
        Intent::Process,
        Intent::Summarize,
        Intent::Analyze,
        Intent::Track,
    ];
    for tier in [CatalogTier::Basic, CatalogTier::Ferro] {
        for intent in &intents {
            let ctx = A2uiContext {
                tier,
                ..Default::default()
            };
            let scored = vec![IntentScore {
                intent: intent.clone(),
                confidence: 1.0,
                matching_signals: vec![],
            }];
            let out = A2uiRenderer
                .render(&order_service(), &scored, &ctx)
                .unwrap();
            let A2uiMessage::CreateSurface(cs) = &out.messages[0] else {
                panic!()
            };
            let components: Vec<(String, Value)> = cs
                .components
                .iter()
                .map(|c| (c.id.clone(), serde_json::to_value(c).unwrap()))
                .collect();
            let emitted = collect_paths(&components);
            let contract: BTreeSet<String> = out
                .data_contract
                .paths()
                .iter()
                .map(|s| s.to_string())
                .collect();
            for path in &emitted {
                assert!(
                    contract.contains(path) || contract.contains(&normalize(path)),
                    "{intent:?}/{tier:?}: emitted binding {path} missing from contract {contract:?}"
                );
            }
        }
    }
}
