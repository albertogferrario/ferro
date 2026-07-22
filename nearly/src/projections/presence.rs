use ferro::{DataType, FieldMeaning, ServiceDef};

/// Presence projection — a user's current, expiring location.
pub fn service_def() -> ServiceDef {
    ServiceDef::new("presence")
        .display_name("Presence")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("user_id", DataType::Integer, FieldMeaning::ForeignKey)
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
        .field("last_seen", DataType::DateTime, FieldMeaning::UpdatedAt)
}
