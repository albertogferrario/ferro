use ferro::{DataType, FieldMeaning, ServiceDef};

/// Build the Book service projection.
///
/// Describes the Book entity's fields and their semantic meaning so Ferro's
/// intent/rendering layer can present a book consistently across modalities.
pub fn service_def() -> ServiceDef {
    ServiceDef::new("book")
        .display_name("Book")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("title", DataType::String, FieldMeaning::EntityName)
        .field("author", DataType::String, FieldMeaning::FreeText)
        .field("isbn", DataType::String, FieldMeaning::FreeText)
        .field("cover_url", DataType::String, FieldMeaning::FreeText)
        .field("description", DataType::String, FieldMeaning::FreeText)
        .field("year", DataType::Integer, FieldMeaning::FreeText)
        .field("source", DataType::String, FieldMeaning::FreeText)
        .field("status", DataType::String, FieldMeaning::Status)
        .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
        .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt)
}
