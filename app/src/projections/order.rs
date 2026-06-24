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
        .tenant_column("tenant_id") // FK column for dispatch predicate injection (D-02)
        .mcp_ability("view-orders") // Gate ability required for tools/call (D-04)
        .mcp_write_ability("manage-orders") // write gate: scopes create_/update_/delete_ tools (D-04)
        .creatable(true) // derives create_order tool (CRUD-01)
        .updatable(true) // derives update_order tool (CRUD-02)
        .deletable(true) // derives delete_order tool, confirmation-gated (CRUD-03)
        .soft_delete_column("deleted_at") // CRUD-03/04: list_order excludes soft-deleted rows
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

#[cfg(test)]
mod tests {
    use super::service_def;

    /// CRUD-07: a write-enabled projection MUST carry mcp_write_ability, else
    /// ServiceDef::validate() rejects it at boot. The order flip sets
    /// .mcp_write_ability("manage-orders") alongside .creatable/.updatable/.deletable,
    /// so validate() passes. This pins the boot-time contract for the flipped projection.
    #[test]
    fn order_projection_validates_after_crud_flip() {
        let svc = service_def();
        assert!(svc.creatable, "order must be creatable after the flip");
        assert!(svc.updatable, "order must be updatable after the flip");
        assert!(svc.deletable, "order must be deletable after the flip");
        svc.validate()
            .expect("CRUD-07: write flags + mcp_write_ability must pass validate() at boot");
    }
}
