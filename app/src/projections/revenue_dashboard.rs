use ferro::{DataType, FieldMeaning, ServiceDef};

/// Build the Revenue Dashboard service projection.
///
/// Models a read-only financial metrics dashboard.
/// Designed to derive the Summarize intent via >70% non-writable
/// fields with Money, Percentage, and Quantity meanings.
pub fn service_def() -> ServiceDef {
    ServiceDef::new("revenue_dashboard")
        .display_name("Revenue Dashboard")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .read_only_field("total_revenue", DataType::Float, FieldMeaning::Money)
        .read_only_field("profit_margin", DataType::Float, FieldMeaning::Percentage)
        .read_only_field("order_count", DataType::Integer, FieldMeaning::Quantity)
        .read_only_field("avg_order_value", DataType::Float, FieldMeaning::Money)
        .read_only_field("return_rate", DataType::Float, FieldMeaning::Percentage)
}
