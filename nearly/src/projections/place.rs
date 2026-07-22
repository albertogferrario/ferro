use ferro::{DataType, FieldMeaning, ServiceDef};

/// Place projection — a venue on the map (trend + premium).
pub fn service_def() -> ServiceDef {
    ServiceDef::new("place")
        .display_name("Place")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("name", DataType::String, FieldMeaning::EntityName)
        .field("category", DataType::String, FieldMeaning::Category)
        .field(
            "lat",
            DataType::Float,
            FieldMeaning::Custom("latitude".into()),
        )
        .field(
            "lng",
            DataType::Float,
            FieldMeaning::Custom("longitude".into()),
        )
        .field("premium", DataType::Boolean, FieldMeaning::Boolean)
        .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
}
