use ferro::{DataType, FieldMeaning, ServiceDef};

/// Build the ApiKey service projection.
///
/// Derived from the ApiKey model.
/// Describes the ApiKey entity's fields, relationships,
/// and behavioral semantics for intent derivation and UI rendering.
pub fn service_def() -> ServiceDef {
    ServiceDef::new("api_key")
        .display_name("Api Key")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("name", DataType::String, FieldMeaning::EntityName)
        .field("prefix", DataType::String, FieldMeaning::FreeText)
        .optional_field("scopes", DataType::String, FieldMeaning::FreeText)
        .field("last_used_at", DataType::DateTime, FieldMeaning::DateTime)
        .field("expires_at", DataType::DateTime, FieldMeaning::DateTime)
        .field("revoked_at", DataType::DateTime, FieldMeaning::DateTime)
        .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
}
