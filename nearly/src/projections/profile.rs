use ferro::{DataType, FieldMeaning, ServiceDef};

/// Profile projection — a user's public identity on the map.
pub fn service_def() -> ServiceDef {
    ServiceDef::new("profile")
        .display_name("Profile")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("user_id", DataType::Integer, FieldMeaning::ForeignKey)
        .field("display_name", DataType::String, FieldMeaning::EntityName)
        .field("status", DataType::String, FieldMeaning::FreeText)
        .field("avatar_url", DataType::String, FieldMeaning::ImageUrl)
        .field("visible", DataType::Boolean, FieldMeaning::Boolean)
        .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
        .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt)
}
