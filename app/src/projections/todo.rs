use ferro::{DataType, FieldMeaning, ServiceDef};

/// Build the Todo service projection.
///
/// Derived from the Todo model.
/// Describes the Todo entity's fields, relationships,
/// and behavioral semantics for intent derivation and UI rendering.
pub fn service_def() -> ServiceDef {
    ServiceDef::new("todo")
        .display_name("Todo")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("title", DataType::String, FieldMeaning::EntityName)
        .field("description", DataType::String, FieldMeaning::FreeText)
        .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
        .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt)
}
