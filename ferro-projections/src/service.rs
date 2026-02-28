use serde::{Deserialize, Serialize};

use crate::field::{DataType, FieldDef, FieldMeaning};

/// A service definition describing a domain entity and its fields.
///
/// Constructed via a builder API with method chaining:
///
/// ```
/// use ferro_projections::{ServiceDef, DataType, FieldMeaning};
///
/// let order = ServiceDef::new("order")
///     .display_name("Order")
///     .description("Manages customer orders")
///     .field("id", DataType::Integer, FieldMeaning::Identifier)
///     .field("total", DataType::Float, FieldMeaning::Money)
///     .optional_field("notes", DataType::String, FieldMeaning::FreeText);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceDef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub fields: Vec<FieldDef>,
}

impl ServiceDef {
    /// Creates a new service definition with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: None,
            description: None,
            fields: Vec::new(),
        }
    }

    /// Sets the human-readable display name.
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Sets the service description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Adds a required field.
    pub fn field(
        mut self,
        name: impl Into<String>,
        data_type: DataType,
        meaning: FieldMeaning,
    ) -> Self {
        self.fields.push(FieldDef {
            name: name.into(),
            data_type,
            meaning,
            required: true,
            is_list: false,
        });
        self
    }

    /// Adds an optional (nullable) field.
    pub fn optional_field(
        mut self,
        name: impl Into<String>,
        data_type: DataType,
        meaning: FieldMeaning,
    ) -> Self {
        self.fields.push(FieldDef {
            name: name.into(),
            data_type,
            meaning,
            required: false,
            is_list: false,
        });
        self
    }

    /// Adds a required list field.
    pub fn list_field(
        mut self,
        name: impl Into<String>,
        data_type: DataType,
        meaning: FieldMeaning,
    ) -> Self {
        self.fields.push(FieldDef {
            name: name.into(),
            data_type,
            meaning,
            required: true,
            is_list: true,
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_def_builder_chain() {
        let service = ServiceDef::new("order")
            .display_name("Order")
            .description("Manages customer orders")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .optional_field("notes", DataType::String, FieldMeaning::FreeText)
            .list_field("tags", DataType::String, FieldMeaning::Category);

        assert_eq!(service.name, "order");
        assert_eq!(service.display_name.as_deref(), Some("Order"));
        assert_eq!(
            service.description.as_deref(),
            Some("Manages customer orders")
        );
        assert_eq!(service.fields.len(), 5);

        // Required field
        assert!(service.fields[0].required);
        assert!(!service.fields[0].is_list);

        // Optional field
        assert!(!service.fields[3].required);
        assert!(!service.fields[3].is_list);

        // List field
        assert!(service.fields[4].required);
        assert!(service.fields[4].is_list);
    }

    #[test]
    fn service_def_minimal() {
        let service = ServiceDef::new("user");
        assert_eq!(service.name, "user");
        assert!(service.display_name.is_none());
        assert!(service.description.is_none());
        assert!(service.fields.is_empty());
    }

    #[test]
    fn service_def_serde_round_trip() {
        let service = ServiceDef::new("order")
            .display_name("Order")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("total", DataType::Float, FieldMeaning::Money)
            .optional_field("notes", DataType::String, FieldMeaning::FreeText);

        let json = serde_json::to_string(&service).unwrap();
        let parsed: ServiceDef = serde_json::from_str(&json).unwrap();
        assert_eq!(service, parsed);
    }

    #[test]
    fn service_def_json_omits_none_fields() {
        let service = ServiceDef::new("order");
        let json = serde_json::to_string(&service).unwrap();
        assert!(!json.contains("display_name"));
        assert!(!json.contains("description"));
    }

    #[test]
    fn service_def_multiple_fields() {
        let service = ServiceDef::new("product")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("price", DataType::Float, FieldMeaning::Money)
            .field("sku", DataType::String, FieldMeaning::Custom("sku".into()))
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt);

        assert_eq!(service.fields.len(), 5);
        // Order preserved
        assert_eq!(service.fields[0].name, "id");
        assert_eq!(service.fields[1].name, "name");
        assert_eq!(service.fields[2].name, "price");
        assert_eq!(service.fields[3].name, "sku");
        assert_eq!(service.fields[4].name, "created_at");
    }

    #[test]
    fn service_def_json_structure() {
        let service = ServiceDef::new("order")
            .display_name("Order")
            .description("Customer orders")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .optional_field("notes", DataType::String, FieldMeaning::FreeText);

        let json = serde_json::to_string(&service).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(value.get("name").is_some());
        assert!(value.get("display_name").is_some());
        assert!(value.get("description").is_some());
        assert!(value.get("fields").is_some());

        let fields = value["fields"].as_array().unwrap();
        assert_eq!(fields.len(), 2);
    }

    #[test]
    fn order_service_example() {
        let service = ServiceDef::new("order")
            .display_name("Order")
            .description("Manages customer orders and fulfillment")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("customer_id", DataType::Integer, FieldMeaning::ForeignKey)
            .field("total", DataType::Float, FieldMeaning::Money)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("email", DataType::String, FieldMeaning::Email)
            .field("notes", DataType::String, FieldMeaning::FreeText)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .field("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt);

        assert_eq!(service.fields.len(), 8);
        assert_eq!(service.fields[2].meaning, FieldMeaning::Money);
        assert_eq!(service.fields[3].meaning, FieldMeaning::Status);

        // Serde round-trip
        let json = serde_json::to_string(&service).unwrap();
        let parsed: ServiceDef = serde_json::from_str(&json).unwrap();
        assert_eq!(service, parsed);
    }
}
