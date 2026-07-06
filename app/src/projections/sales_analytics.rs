use ferro::{DataType, FieldMeaning, ServiceDef};

/// Build the Sales Analytics service projection.
///
/// Models time-series sales data for analytical queries.
/// Designed to derive the Analyze intent via DateTime + numeric
/// field co-occurrence. Mixed read/write avoids Summarize and Collect
/// dominance (50% writable avoids Collect, 50% non-writable avoids Summarize).
pub fn service_def() -> ServiceDef {
    ServiceDef::new("sales_analytics")
        .display_name("Sales Analytics")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .read_only_field("period_start", DataType::DateTime, FieldMeaning::DateTime)
        .read_only_field("period_end", DataType::DateTime, FieldMeaning::DateTime)
        .field("amount", DataType::Float, FieldMeaning::Money)
        .field("region", DataType::String, FieldMeaning::Category)
}
