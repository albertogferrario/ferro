use ferro::{DataType, FieldMeaning, ServiceDef};

/// Build the LineItem service projection.
///
/// A child of `order`. CRUD-enabled so an agent can add/remove line items;
/// `order.total` is recomputed from these rows by the post-persist recompute
/// hook registered in `controllers::mcp::make_write_dispatcher`.
pub fn service_def() -> ServiceDef {
    ServiceDef::new("line_item")
        .mcp_exposed(true)
        .tenant_column("tenant_id") // server-side tenant injection + scoping (Phase 242)
        .mcp_ability("view-orders") // reuse the order read ability
        .mcp_write_ability("manage-orders") // reuse the order write gate (defined in bootstrap)
        .creatable(true)
        .updatable(true)
        .deletable(true)
        .soft_delete_column("deleted_at")
        .display_name("Line Item")
        .read_only_field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("order_id", DataType::Integer, FieldMeaning::ForeignKey)
        .field("amount", DataType::Float, FieldMeaning::Money)
        .read_only_field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
}
