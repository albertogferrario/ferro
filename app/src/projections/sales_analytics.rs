use ferro::{DataType, FieldMeaning, ServiceDef};

/// Build the Sales Analytics service projection.
///
/// Models time-series sales data for analytical queries.
/// Designed to derive the Analyze intent via DateTime + numeric
/// field co-occurrence with read-only fields.
pub fn service_def() -> ServiceDef {
    ServiceDef::new("sales_analytics")
        .display_name("Sales Analytics")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .read_only_field("sale_date", DataType::DateTime, FieldMeaning::DateTime)
        .read_only_field("amount", DataType::Float, FieldMeaning::Money)
        .read_only_field("units_sold", DataType::Integer, FieldMeaning::Quantity)
        .read_only_field("discount", DataType::Float, FieldMeaning::Percentage)
        .read_only_field("region", DataType::String, FieldMeaning::Category)
}
