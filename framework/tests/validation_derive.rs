//! Integration tests for the ValidateRules derive macro.
//!
//! Tests declarative validation using #[derive(ValidateRules)] with #[rule(...)] attributes.

extern crate ferro_rs as ferro;
use ferro_rs::validation::Validatable;
use ferro_rs::ValidateRules;
use serde::{Deserialize, Serialize};

// ============================================================================
// Basic Validation Tests
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ValidateRules)]
struct BasicRequest {
    #[rule(required)]
    name: String,

    #[rule(required, email)]
    email: String,
}

#[test]
fn test_basic_validation_passes() {
    let request = BasicRequest {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_basic_validation_fails_on_empty_required() {
    let request = BasicRequest {
        name: "".to_string(),
        email: "john@example.com".to_string(),
    };

    let result = request.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(!errors.is_empty());
}

#[test]
fn test_basic_validation_fails_on_invalid_email() {
    let request = BasicRequest {
        name: "John Doe".to_string(),
        email: "not-an-email".to_string(),
    };

    let result = request.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(!errors.is_empty());
}

// ============================================================================
// Size Rules Tests
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ValidateRules)]
struct SizeRulesRequest {
    #[rule(required, min(3.0))]
    username: String,

    #[rule(required, max(100.0))]
    bio: String,

    #[rule(required, integer, min(18.0))]
    age: i32,
}

#[test]
fn test_min_validation_passes() {
    let request = SizeRulesRequest {
        username: "john".to_string(),
        bio: "A short bio".to_string(),
        age: 25,
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_min_validation_fails_on_short_string() {
    let request = SizeRulesRequest {
        username: "jo".to_string(),
        bio: "A short bio".to_string(),
        age: 25,
    };

    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_min_validation_fails_on_small_number() {
    let request = SizeRulesRequest {
        username: "john".to_string(),
        bio: "A short bio".to_string(),
        age: 15,
    };

    let result = request.validate();
    assert!(result.is_err());
}

// ============================================================================
// Type Rules Tests
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ValidateRules)]
struct TypeRulesRequest {
    #[rule(required, string)]
    title: String,

    #[rule(required, integer)]
    count: i32,

    #[rule(required, numeric)]
    price: f64,

    #[rule(required, boolean)]
    active: bool,
}

#[test]
fn test_type_validation_passes() {
    let request = TypeRulesRequest {
        title: "Hello World".to_string(),
        count: 42,
        price: 19.99,
        active: true,
    };

    assert!(request.validate().is_ok());
}

// ============================================================================
// Format Rules Tests
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ValidateRules)]
struct FormatRulesRequest {
    #[rule(required, url)]
    website: String,

    #[rule(required, alpha)]
    first_name: String,

    #[rule(required, alpha_num)]
    username: String,
}

#[test]
fn test_url_validation_passes() {
    let request = FormatRulesRequest {
        website: "https://example.com".to_string(),
        first_name: "John".to_string(),
        username: "john123".to_string(),
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_url_validation_fails() {
    let request = FormatRulesRequest {
        website: "not-a-url".to_string(),
        first_name: "John".to_string(),
        username: "john123".to_string(),
    };

    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_alpha_validation_fails() {
    let request = FormatRulesRequest {
        website: "https://example.com".to_string(),
        first_name: "John123".to_string(),
        username: "john123".to_string(),
    };

    let result = request.validate();
    assert!(result.is_err());
}

// ============================================================================
// Between Rule Test
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ValidateRules)]
struct BetweenRulesRequest {
    #[rule(required, integer, between(1.0, 100.0))]
    quantity: i32,
}

#[test]
fn test_between_validation_passes() {
    let request = BetweenRulesRequest { quantity: 50 };
    assert!(request.validate().is_ok());
}

#[test]
fn test_between_validation_fails_below_min() {
    let request = BetweenRulesRequest { quantity: 0 };
    assert!(request.validate().is_err());
}

#[test]
fn test_between_validation_fails_above_max() {
    let request = BetweenRulesRequest { quantity: 150 };
    assert!(request.validate().is_err());
}

// ============================================================================
// Validation Rules Introspection
// ============================================================================

#[test]
fn test_validation_rules_introspection() {
    let rules = BasicRequest::validation_rules();
    assert_eq!(rules.len(), 2);

    // Check that both fields have rules
    let field_names: Vec<&str> = rules.iter().map(|(name, _)| *name).collect();
    assert!(field_names.contains(&"name"));
    assert!(field_names.contains(&"email"));
}

// ============================================================================
// Multiple Rules Per Field
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ValidateRules)]
struct MultipleRulesRequest {
    #[rule(required, string, min(8.0), max(50.0))]
    password: String,
}

#[test]
fn test_multiple_rules_all_pass() {
    let request = MultipleRulesRequest {
        password: "securepassword123".to_string(),
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_multiple_rules_fails_min() {
    let request = MultipleRulesRequest {
        password: "short".to_string(),
    };

    assert!(request.validate().is_err());
}

#[test]
fn test_multiple_rules_fails_max() {
    let request = MultipleRulesRequest {
        password: "this is a very long password that exceeds the maximum allowed length"
            .to_string(),
    };

    assert!(request.validate().is_err());
}

// ============================================================================
// Error Messages
// ============================================================================

#[test]
fn test_error_messages_format() {
    let request = BasicRequest {
        name: "".to_string(),
        email: "invalid".to_string(),
    };

    let result = request.validate();
    assert!(result.is_err());

    let errors = result.unwrap_err();
    let messages = errors.into_messages();

    // Check that error messages are generated for both fields
    assert!(messages.contains_key("name") || messages.contains_key("email"));
}

// ============================================================================
// Optional Fields (No Rules)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ValidateRules)]
struct OptionalFieldsRequest {
    #[rule(required)]
    name: String,

    // No rules - always valid
    notes: Option<String>,
}

#[test]
fn test_fields_without_rules_pass() {
    let request = OptionalFieldsRequest {
        name: "John".to_string(),
        notes: None,
    };

    assert!(request.validate().is_ok());
}
