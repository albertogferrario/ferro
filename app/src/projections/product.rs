use ferro::{DataType, FieldMeaning, ServiceDef};

/// Build the Product service projection.
///
/// Models a product catalog with category browsing and relationships.
/// Designed to derive the Browse intent via EntityName, Category,
/// and OneToMany relationship signals.
pub fn service_def() -> ServiceDef {
    ServiceDef::new("product")
        .display_name("Product")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("name", DataType::String, FieldMeaning::EntityName)
        .field("price", DataType::Float, FieldMeaning::Money)
        .field("category", DataType::String, FieldMeaning::Category)
        .field("image_url", DataType::String, FieldMeaning::ImageUrl)
        .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
        .has_many("reviews", "review")
        .has_many("variants", "product_variant")
        .belongs_to("brand", "brand")
}
