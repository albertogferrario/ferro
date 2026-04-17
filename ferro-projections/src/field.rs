use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Abstract data type categories for service fields.
///
/// Represents structural types independent of database storage details.
/// Maps from database column types at introspection time.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    Date,
    Json,
    Binary,
    Uuid,
    Enum,
}

/// Semantic meaning of a field, driving rendering and behavior decisions.
///
/// Known variants map to specific UI treatments (e.g., `Money` formats as currency,
/// `Status` renders as a badge). The `Custom` fallback captures domain-specific
/// meanings not covered by built-in variants.
///
/// `Custom(String)` must remain the last variant for correct serde deserialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(
    description = "Semantic field meaning. Known variants: identifier, foreign_key, entity_name, email, phone, url, image_url, money, percentage, quantity, status, category, boolean, free_text, created_at, updated_at, date_time, sensitive. Any other string is a custom domain-specific meaning."
)]
pub enum FieldMeaning {
    Identifier,
    ForeignKey,
    EntityName,
    Email,
    Phone,
    Url,
    ImageUrl,
    Money,
    Percentage,
    Quantity,
    Status,
    Category,
    Boolean,
    FreeText,
    CreatedAt,
    UpdatedAt,
    DateTime,
    Sensitive,
    #[serde(untagged)]
    Custom(String),
}

/// A field definition within a service projection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct FieldDef {
    pub name: String,
    pub data_type: DataType,
    pub meaning: FieldMeaning,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub is_list: bool,
    #[serde(default = "default_true")]
    pub readable: bool,
    #[serde(default = "default_true")]
    pub writable: bool,
}

fn default_true() -> bool {
    true
}

impl DataType {
    /// Infers a DataType from a Rust/SeaORM column type string.
    ///
    /// Strips `Option<>` wrappers before matching. Falls back to `String`
    /// for unrecognized types.
    pub fn from_column_type(type_str: &str) -> Self {
        let inner = if let Some(stripped) = type_str
            .strip_prefix("Option<")
            .and_then(|s| s.strip_suffix('>'))
        {
            stripped
        } else {
            type_str
        };

        match inner {
            "i32" | "i64" | "u32" | "u64" | "i8" | "i16" | "u8" | "u16" => Self::Integer,
            "f32" | "f64" => Self::Float,
            "bool" => Self::Boolean,
            "Uuid" | "uuid::Uuid" => Self::Uuid,
            s if s.contains("Decimal") => Self::Float,
            s if s.starts_with("DateTime") || s.contains("chrono::") => Self::DateTime,
            s if s.starts_with("NaiveDate") => Self::Date,
            "Vec<u8>" => Self::Binary,
            s if s.contains("Json") || s.contains("serde_json") => Self::Json,
            _ => Self::String,
        }
    }
}

/// Infers a [`FieldMeaning`] from a field name using common naming conventions.
///
/// Applies seven inference rules based on patterns found across the codebase:
/// - Exact matches: `id`, `email`, `created_at`, `updated_at`
/// - Suffix: `_id` (foreign key), `_at` (datetime)
/// - Prefix: `is_`, `has_` (boolean)
/// - Contains: `password`, `secret`, `token`, `api_key`, `hashed_key` (sensitive)
/// - Fallback: `Custom(field_name)`
pub fn infer_meaning(field_name: &str) -> FieldMeaning {
    // Exact matches first
    match field_name {
        "id" => return FieldMeaning::Identifier,
        "email" => return FieldMeaning::Email,
        "created_at" => return FieldMeaning::CreatedAt,
        "updated_at" => return FieldMeaning::UpdatedAt,
        _ => {}
    }

    // Suffix patterns
    if field_name.ends_with("_id") {
        return FieldMeaning::ForeignKey;
    }
    if field_name.ends_with("_at") {
        return FieldMeaning::DateTime;
    }

    // Prefix patterns
    if field_name.starts_with("is_") || field_name.starts_with("has_") {
        return FieldMeaning::Boolean;
    }

    // Sensitive field patterns
    const SENSITIVE: &[&str] = &["password", "secret", "token", "api_key", "hashed_key"];
    if SENSITIVE.iter().any(|s| field_name.contains(s)) {
        return FieldMeaning::Sensitive;
    }

    FieldMeaning::Custom(field_name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_column_type_mappings() {
        assert_eq!(DataType::from_column_type("i32"), DataType::Integer);
        assert_eq!(DataType::from_column_type("i64"), DataType::Integer);
        assert_eq!(DataType::from_column_type("u32"), DataType::Integer);
        assert_eq!(DataType::from_column_type("u64"), DataType::Integer);
        assert_eq!(DataType::from_column_type("i8"), DataType::Integer);
        assert_eq!(DataType::from_column_type("i16"), DataType::Integer);
        assert_eq!(DataType::from_column_type("u8"), DataType::Integer);
        assert_eq!(DataType::from_column_type("u16"), DataType::Integer);
        assert_eq!(DataType::from_column_type("f32"), DataType::Float);
        assert_eq!(DataType::from_column_type("f64"), DataType::Float);
        assert_eq!(DataType::from_column_type("bool"), DataType::Boolean);
        assert_eq!(DataType::from_column_type("String"), DataType::String);
        assert_eq!(DataType::from_column_type("Uuid"), DataType::Uuid);
        assert_eq!(DataType::from_column_type("uuid::Uuid"), DataType::Uuid);
        assert_eq!(
            DataType::from_column_type("DateTime<Utc>"),
            DataType::DateTime
        );
        assert_eq!(
            DataType::from_column_type("chrono::DateTime<chrono::Utc>"),
            DataType::DateTime
        );
        assert_eq!(DataType::from_column_type("NaiveDate"), DataType::Date);
        assert_eq!(DataType::from_column_type("Vec<u8>"), DataType::Binary);
        assert_eq!(
            DataType::from_column_type("serde_json::Value"),
            DataType::Json
        );
        assert_eq!(DataType::from_column_type("Json"), DataType::Json);
        assert_eq!(DataType::from_column_type("Decimal"), DataType::Float);
        assert_eq!(
            DataType::from_column_type("UnknownCustomType"),
            DataType::String
        );
    }

    #[test]
    fn from_column_type_option_stripping() {
        assert_eq!(
            DataType::from_column_type("Option<String>"),
            DataType::String
        );
        assert_eq!(DataType::from_column_type("Option<i32>"), DataType::Integer);
        assert_eq!(
            DataType::from_column_type("Option<DateTime<Utc>>"),
            DataType::DateTime
        );
    }

    #[test]
    fn data_type_is_copy() {
        let dt = DataType::Float;
        let dt2 = dt;
        assert_eq!(dt, dt2);
    }

    #[test]
    fn data_type_serde_round_trip() {
        for dt in [
            DataType::String,
            DataType::Integer,
            DataType::Float,
            DataType::Boolean,
            DataType::DateTime,
            DataType::Date,
            DataType::Json,
            DataType::Binary,
            DataType::Uuid,
            DataType::Enum,
        ] {
            let json = serde_json::to_string(&dt).unwrap();
            let parsed: DataType = serde_json::from_str(&json).unwrap();
            assert_eq!(dt, parsed);
        }
    }

    #[test]
    fn field_meaning_known_variants_serde_round_trip() {
        let known = [
            FieldMeaning::Identifier,
            FieldMeaning::ForeignKey,
            FieldMeaning::EntityName,
            FieldMeaning::Email,
            FieldMeaning::Phone,
            FieldMeaning::Url,
            FieldMeaning::ImageUrl,
            FieldMeaning::Money,
            FieldMeaning::Percentage,
            FieldMeaning::Quantity,
            FieldMeaning::Status,
            FieldMeaning::Category,
            FieldMeaning::Boolean,
            FieldMeaning::FreeText,
            FieldMeaning::CreatedAt,
            FieldMeaning::UpdatedAt,
            FieldMeaning::DateTime,
            FieldMeaning::Sensitive,
        ];
        for meaning in known {
            let json = serde_json::to_string(&meaning).unwrap();
            let parsed: FieldMeaning = serde_json::from_str(&json).unwrap();
            assert_eq!(meaning, parsed);
        }
    }

    #[test]
    fn field_meaning_custom_fallback() {
        let parsed: FieldMeaning = serde_json::from_str(r#""tax_rate""#).unwrap();
        assert_eq!(parsed, FieldMeaning::Custom("tax_rate".to_string()));
    }

    #[test]
    fn field_meaning_custom_round_trip() {
        let custom = FieldMeaning::Custom("my_thing".into());
        let json = serde_json::to_string(&custom).unwrap();
        let parsed: FieldMeaning = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, FieldMeaning::Custom("my_thing".into()));
    }

    #[test]
    fn field_meaning_known_not_custom() {
        // Deserializing "money" must match Money variant, not Custom("money")
        let parsed: FieldMeaning = serde_json::from_str(r#""money""#).unwrap();
        assert_eq!(parsed, FieldMeaning::Money);
        assert_ne!(parsed, FieldMeaning::Custom("money".into()));
    }

    #[test]
    fn field_meaning_money_serializes_to_snake_case() {
        let json = serde_json::to_string(&FieldMeaning::Money).unwrap();
        assert_eq!(json, r#""money""#);
    }

    #[test]
    fn field_meaning_foreign_key_serializes_to_snake_case() {
        let json = serde_json::to_string(&FieldMeaning::ForeignKey).unwrap();
        assert_eq!(json, r#""foreign_key""#);
    }

    #[test]
    fn field_def_serde_round_trip() {
        let field = FieldDef {
            name: "total".to_string(),
            data_type: DataType::Float,
            meaning: FieldMeaning::Money,
            required: true,
            is_list: false,
            readable: true,
            writable: true,
        };
        let json = serde_json::to_string(&field).unwrap();
        let parsed: FieldDef = serde_json::from_str(&json).unwrap();
        assert_eq!(field, parsed);
    }

    #[test]
    fn field_def_defaults() {
        // Verify that omitting required/is_list/readable/writable uses correct defaults
        let json = r#"{"name":"total","data_type":"float","meaning":"money"}"#;
        let parsed: FieldDef = serde_json::from_str(json).unwrap();
        assert!(parsed.required);
        assert!(!parsed.is_list);
        assert!(parsed.readable);
        assert!(parsed.writable);
    }

    #[test]
    fn field_def_read_only() {
        let json = r#"{"name":"id","data_type":"integer","meaning":"identifier","readable":true,"writable":false}"#;
        let parsed: FieldDef = serde_json::from_str(json).unwrap();
        assert!(parsed.readable);
        assert!(!parsed.writable);
    }

    #[test]
    fn field_def_write_only() {
        let json = r#"{"name":"password","data_type":"string","meaning":"sensitive","readable":false,"writable":true}"#;
        let parsed: FieldDef = serde_json::from_str(json).unwrap();
        assert!(!parsed.readable);
        assert!(parsed.writable);
    }

    #[test]
    fn infer_meaning_exact_matches() {
        assert_eq!(infer_meaning("id"), FieldMeaning::Identifier);
        assert_eq!(infer_meaning("email"), FieldMeaning::Email);
        assert_eq!(infer_meaning("created_at"), FieldMeaning::CreatedAt);
        assert_eq!(infer_meaning("updated_at"), FieldMeaning::UpdatedAt);
    }

    #[test]
    fn infer_meaning_suffix_patterns() {
        assert_eq!(infer_meaning("user_id"), FieldMeaning::ForeignKey);
        assert_eq!(infer_meaning("order_id"), FieldMeaning::ForeignKey);
        assert_eq!(infer_meaning("deleted_at"), FieldMeaning::DateTime);
        assert_eq!(infer_meaning("expires_at"), FieldMeaning::DateTime);
    }

    #[test]
    fn infer_meaning_prefix_patterns() {
        assert_eq!(infer_meaning("is_active"), FieldMeaning::Boolean);
        assert_eq!(infer_meaning("has_premium"), FieldMeaning::Boolean);
    }

    #[test]
    fn infer_meaning_sensitive_patterns() {
        assert_eq!(infer_meaning("password"), FieldMeaning::Sensitive);
        assert_eq!(infer_meaning("hashed_password"), FieldMeaning::Sensitive);
        assert_eq!(infer_meaning("secret"), FieldMeaning::Sensitive);
        assert_eq!(infer_meaning("api_key"), FieldMeaning::Sensitive);
        assert_eq!(infer_meaning("hashed_key"), FieldMeaning::Sensitive);
        assert_eq!(infer_meaning("remember_token"), FieldMeaning::Sensitive);
    }

    #[test]
    fn infer_meaning_fallback_to_custom() {
        assert_eq!(
            infer_meaning("title"),
            FieldMeaning::Custom("title".to_string())
        );
        assert_eq!(
            infer_meaning("description"),
            FieldMeaning::Custom("description".to_string())
        );
    }

    #[test]
    fn data_type_json_schema() {
        let schema = schemars::schema_for!(DataType);
        let value = schema.to_value();
        let enum_values = value
            .get("enum")
            .expect("DataType schema must have enum key");
        let arr = enum_values.as_array().unwrap();
        let strings: Vec<&str> = arr.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(strings.contains(&"string"));
        assert!(strings.contains(&"integer"));
        assert!(strings.contains(&"float"));
        assert!(strings.contains(&"boolean"));
        assert!(strings.contains(&"date_time"));
        assert!(strings.contains(&"uuid"));
    }

    #[test]
    fn field_meaning_json_schema_has_description() {
        let schema = schemars::schema_for!(FieldMeaning);
        let value = schema.to_value();
        let desc = value
            .get("description")
            .expect("FieldMeaning schema must have description");
        let desc_str = desc.as_str().unwrap();
        assert!(
            desc_str.contains("Known variants"),
            "description should document known variants, got: {desc_str}"
        );
    }

    #[test]
    fn field_def_json_schema() {
        let schema = schemars::schema_for!(FieldDef);
        let value = schema.to_value();
        let props = value
            .get("properties")
            .expect("FieldDef schema must have properties");
        let obj = props.as_object().unwrap();
        assert!(obj.contains_key("name"), "missing 'name' property");
        assert!(
            obj.contains_key("data_type"),
            "missing 'data_type' property"
        );
        assert!(obj.contains_key("meaning"), "missing 'meaning' property");
    }

    // -- Additional Phase 86-02 tests: readable/writable defaults --

    #[test]
    fn field_def_readable_writable_defaults_from_json() {
        // Both readable and writable default to true when omitted — backward compatibility
        let json = r#"{"name":"title","data_type":"string","meaning":"entity_name"}"#;
        let parsed: FieldDef = serde_json::from_str(json).unwrap();
        assert!(parsed.readable);
        assert!(parsed.writable);
    }

    #[test]
    fn field_def_readable_false_writable_true_round_trip() {
        let field = FieldDef {
            name: "password".to_string(),
            data_type: DataType::String,
            meaning: FieldMeaning::Sensitive,
            required: true,
            is_list: false,
            readable: false,
            writable: true,
        };
        let json = serde_json::to_string(&field).unwrap();
        let parsed: FieldDef = serde_json::from_str(&json).unwrap();
        assert_eq!(field, parsed);
        assert!(!parsed.readable);
        assert!(parsed.writable);
    }

    #[test]
    fn field_def_readable_true_writable_false_round_trip() {
        let field = FieldDef {
            name: "id".to_string(),
            data_type: DataType::Integer,
            meaning: FieldMeaning::Identifier,
            required: true,
            is_list: false,
            readable: true,
            writable: false,
        };
        let json = serde_json::to_string(&field).unwrap();
        let parsed: FieldDef = serde_json::from_str(&json).unwrap();
        assert_eq!(field, parsed);
        assert!(parsed.readable);
        assert!(!parsed.writable);
    }

    #[test]
    fn field_def_readable_false_writable_false_round_trip() {
        let field = FieldDef {
            name: "internal_hash".to_string(),
            data_type: DataType::String,
            meaning: FieldMeaning::Sensitive,
            required: true,
            is_list: false,
            readable: false,
            writable: false,
        };
        let json = serde_json::to_string(&field).unwrap();
        let parsed: FieldDef = serde_json::from_str(&json).unwrap();
        assert_eq!(field, parsed);
        assert!(!parsed.readable);
        assert!(!parsed.writable);
    }
}
