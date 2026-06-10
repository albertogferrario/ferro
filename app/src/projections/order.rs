use ferro::{
    ActionDef, DataType, FieldMeaning, GuardDef, ServiceDef, StateDef, StateMachine, Transition,
};

/// Build the Order service projection.
///
/// Models an order fulfillment workflow with guarded state transitions.
/// Designed to derive the Process intent via guard density,
/// branching states, and transition triggers on actions.
pub fn service_def() -> ServiceDef {
    ServiceDef::new("order")
        .mcp_exposed(true)
        .tenant_column("tenant_id")   // FK column for dispatch predicate injection (D-02)
        .mcp_ability("view-orders")   // Gate ability required for tools/call (D-04)
        .display_name("Order")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("customer_name", DataType::String, FieldMeaning::EntityName)
        .field("total", DataType::Float, FieldMeaning::Money)
        .field("status", DataType::String, FieldMeaning::Status)
        .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
        .state_machine(
            StateMachine::new("order_lifecycle")
                .initial("draft")
                .state(StateDef::new("draft"))
                .state(StateDef::new("submitted"))
                .state(StateDef::new("approved"))
                .state(StateDef::new("shipped"))
                .state(StateDef::new("delivered").final_state())
                .state(StateDef::new("cancelled").final_state())
                .transition(Transition::new("draft", "submit", "submitted"))
                .transition(Transition::new("submitted", "approve", "approved").guard("is_manager"))
                .transition(Transition::new("submitted", "reject", "cancelled"))
                .transition(Transition::new("approved", "ship", "shipped"))
                .transition(Transition::new("shipped", "deliver", "delivered"))
                .transition(Transition::new("draft", "cancel", "cancelled")),
        )
        .guard(GuardDef::new("is_manager").display_name("Manager Approval Required"))
        .action(ActionDef::new("submit").transition_trigger("submit"))
        .action(
            ActionDef::new("approve")
                .transition_trigger("approve")
                .precondition("is_manager"),
        )
        .action(ActionDef::new("ship").transition_trigger("ship"))
        .belongs_to("customer", "user")
        .has_many("line_items", "line_item")
}
