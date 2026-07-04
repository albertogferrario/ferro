//! Shared test fixtures.

use ferro_projections::{
    ActionDef, DataType, FieldMeaning, Intent, IntentScore, ServiceDef, StateDef, StateMachine,
    Transition,
};

/// Canonical fixture: an order service with money/status/state-machine shape.
pub(crate) fn order_service() -> ServiceDef {
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

/// A single scored intent for direct renders.
pub(crate) fn scored(intent: Intent) -> Vec<IntentScore> {
    vec![IntentScore {
        intent,
        confidence: 1.0,
        matching_signals: vec![],
    }]
}
