use ferro::{DataType, FieldMeaning, ServiceDef};

/// Trillo projection — a wordless ping between users. There is deliberately no
/// message field: the trillo *is* the whole payload.
pub fn service_def() -> ServiceDef {
    ServiceDef::new("trillo")
        .display_name("Trillo")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("from_user_id", DataType::Integer, FieldMeaning::ForeignKey)
        .field("to_user_id", DataType::Integer, FieldMeaning::ForeignKey)
        .field("status", DataType::String, FieldMeaning::Status)
        .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
}
