use serde::{Deserialize, Serialize};

/// Abstract data type categories for service fields.
///
/// Represents structural types independent of database storage details.
/// Maps from database column types at introspection time.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldDef {
    pub name: String,
    pub data_type: DataType,
    pub meaning: FieldMeaning,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub is_list: bool,
}

fn default_true() -> bool {
    true
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
        };
        let json = serde_json::to_string(&field).unwrap();
        let parsed: FieldDef = serde_json::from_str(&json).unwrap();
        assert_eq!(field, parsed);
    }

    #[test]
    fn field_def_defaults() {
        // Verify that omitting required/is_list uses correct defaults
        let json = r#"{"name":"total","data_type":"float","meaning":"money"}"#;
        let parsed: FieldDef = serde_json::from_str(json).unwrap();
        assert!(parsed.required);
        assert!(!parsed.is_list);
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
}
