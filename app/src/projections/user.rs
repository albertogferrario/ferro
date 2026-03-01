use ferro::{DataType, FieldMeaning, ServiceDef};

/// Build the User service projection.
///
/// Derived from the User model.
/// Describes the User entity's fields, relationships,
/// and behavioral semantics for intent derivation and UI rendering.
pub fn service_def() -> ServiceDef {
    ServiceDef::new("user")
        .display_name("User")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("name", DataType::String, FieldMeaning::EntityName)
        .field("email", DataType::String, FieldMeaning::Email)
        .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
        .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt)
}
