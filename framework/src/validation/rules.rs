//! Built-in validation rules.

use crate::validation::translate_validation;
use crate::validation::Rule;
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

// ============================================================================
// Required Rules
// ============================================================================

/// Field must be present and not empty.
pub struct Required;

pub const fn required() -> Required {
    Required
}

impl Rule for Required {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        let is_empty = match value {
            Value::Null => true,
            Value::String(s) => s.trim().is_empty(),
            Value::Array(a) => a.is_empty(),
            Value::Object(o) => o.is_empty(),
            _ => false,
        };

        if is_empty {
            Err(
                translate_validation("validation.required", &[("attribute", field)])
                    .unwrap_or_else(|| format!("The {} field is required.", field)),
            )
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "required"
    }
}

/// Field is required if another field equals a value.
pub struct RequiredIf {
    other: String,
    value: Value,
}

pub fn required_if(other: impl Into<String>, value: impl Into<Value>) -> RequiredIf {
    RequiredIf {
        other: other.into(),
        value: value.into(),
    }
}

impl Rule for RequiredIf {
    fn validate(&self, field: &str, value: &Value, data: &Value) -> Result<(), String> {
        let other_value = data.get(&self.other);
        if other_value == Some(&self.value) {
            Required.validate(field, value, data)
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "required_if"
    }
}

// ============================================================================
// Type Rules
// ============================================================================

/// Field must be a string.
pub struct IsString;

pub const fn string() -> IsString {
    IsString
}

impl Rule for IsString {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() || value.is_string() {
            Ok(())
        } else {
            Err(
                translate_validation("validation.string", &[("attribute", field)])
                    .unwrap_or_else(|| format!("The {} field must be a string.", field)),
            )
        }
    }

    fn name(&self) -> &'static str {
        "string"
    }
}

/// Field must be an integer.
pub struct IsInteger;

pub const fn integer() -> IsInteger {
    IsInteger
}

impl Rule for IsInteger {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() || value.is_i64() || value.is_u64() {
            Ok(())
        } else if let Some(s) = value.as_str() {
            if s.parse::<i64>().is_ok() {
                Ok(())
            } else {
                Err(
                    translate_validation("validation.integer", &[("attribute", field)])
                        .unwrap_or_else(|| format!("The {} field must be an integer.", field)),
                )
            }
        } else {
            Err(
                translate_validation("validation.integer", &[("attribute", field)])
                    .unwrap_or_else(|| format!("The {} field must be an integer.", field)),
            )
        }
    }

    fn name(&self) -> &'static str {
        "integer"
    }
}

/// Field must be numeric.
pub struct Numeric;

pub const fn numeric() -> Numeric {
    Numeric
}

impl Rule for Numeric {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() || value.is_number() {
            Ok(())
        } else if let Some(s) = value.as_str() {
            if s.parse::<f64>().is_ok() {
                Ok(())
            } else {
                Err(
                    translate_validation("validation.numeric", &[("attribute", field)])
                        .unwrap_or_else(|| format!("The {} field must be a number.", field)),
                )
            }
        } else {
            Err(
                translate_validation("validation.numeric", &[("attribute", field)])
                    .unwrap_or_else(|| format!("The {} field must be a number.", field)),
            )
        }
    }

    fn name(&self) -> &'static str {
        "numeric"
    }
}

/// Field must be a boolean.
pub struct IsBoolean;

pub const fn boolean() -> IsBoolean {
    IsBoolean
}

impl Rule for IsBoolean {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() || value.is_boolean() {
            Ok(())
        } else if let Some(s) = value.as_str() {
            match s.to_lowercase().as_str() {
                "true" | "false" | "1" | "0" | "yes" | "no" => Ok(()),
                _ => Err(
                    translate_validation("validation.boolean", &[("attribute", field)])
                        .unwrap_or_else(|| format!("The {} field must be true or false.", field)),
                ),
            }
        } else if let Some(n) = value.as_i64() {
            if n == 0 || n == 1 {
                Ok(())
            } else {
                Err(
                    translate_validation("validation.boolean", &[("attribute", field)])
                        .unwrap_or_else(|| format!("The {} field must be true or false.", field)),
                )
            }
        } else {
            Err(
                translate_validation("validation.boolean", &[("attribute", field)])
                    .unwrap_or_else(|| format!("The {} field must be true or false.", field)),
            )
        }
    }

    fn name(&self) -> &'static str {
        "boolean"
    }
}

/// Field must be an array.
pub struct IsArray;

pub const fn array() -> IsArray {
    IsArray
}

impl Rule for IsArray {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() || value.is_array() {
            Ok(())
        } else {
            Err(
                translate_validation("validation.array", &[("attribute", field)])
                    .unwrap_or_else(|| format!("The {} field must be an array.", field)),
            )
        }
    }

    fn name(&self) -> &'static str {
        "array"
    }
}

// ============================================================================
// Size Rules
// ============================================================================

/// Field must have a minimum size/length/value.
pub struct Min {
    min: f64,
}

pub fn min(min: impl Into<f64>) -> Min {
    Min { min: min.into() }
}

impl Rule for Min {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }

        let size = get_size(value);
        if size < self.min {
            let unit = get_size_unit(value);
            let type_key = get_size_type_key("min", value);
            let min_str = format!("{}", self.min as i64);
            Err(
                translate_validation(&type_key, &[("attribute", field), ("min", &min_str)])
                    .unwrap_or_else(|| {
                        format!(
                            "The {} field must be at least {} {}.",
                            field, self.min as i64, unit
                        )
                    }),
            )
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "min"
    }
}

/// Field must have a maximum size/length/value.
pub struct Max {
    max: f64,
}

pub fn max(max: impl Into<f64>) -> Max {
    Max { max: max.into() }
}

impl Rule for Max {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }

        let size = get_size(value);
        if size > self.max {
            let unit = get_size_unit(value);
            let type_key = get_size_type_key("max", value);
            let max_str = format!("{}", self.max as i64);
            Err(
                translate_validation(&type_key, &[("attribute", field), ("max", &max_str)])
                    .unwrap_or_else(|| {
                        format!(
                            "The {} field must not be greater than {} {}.",
                            field, self.max as i64, unit
                        )
                    }),
            )
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "max"
    }
}

/// Field must be between min and max size.
pub struct Between {
    min: f64,
    max: f64,
}

pub fn between(min: impl Into<f64>, max: impl Into<f64>) -> Between {
    Between {
        min: min.into(),
        max: max.into(),
    }
}

impl Rule for Between {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }

        let size = get_size(value);
        if size < self.min || size > self.max {
            let unit = get_size_unit(value);
            let type_key = get_size_type_key("between", value);
            let min_str = format!("{}", self.min as i64);
            let max_str = format!("{}", self.max as i64);
            Err(translate_validation(
                &type_key,
                &[("attribute", field), ("min", &min_str), ("max", &max_str)],
            )
            .unwrap_or_else(|| {
                format!(
                    "The {} field must be between {} and {} {}.",
                    field, self.min as i64, self.max as i64, unit
                )
            }))
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "between"
    }
}

// ============================================================================
// Format Rules
// ============================================================================

static EMAIL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());

/// Field must be a valid email address.
pub struct Email;

pub const fn email() -> Email {
    Email
}

impl Rule for Email {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }

        match value.as_str() {
            Some(s) if EMAIL_REGEX.is_match(s) => Ok(()),
            _ => Err(
                translate_validation("validation.email", &[("attribute", field)]).unwrap_or_else(
                    || format!("The {} field must be a valid email address.", field),
                ),
            ),
        }
    }

    fn name(&self) -> &'static str {
        "email"
    }
}

static URL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap());

/// Field must be a valid URL.
pub struct Url;

pub const fn url() -> Url {
    Url
}

impl Rule for Url {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }

        match value.as_str() {
            Some(s) if URL_REGEX.is_match(s) => Ok(()),
            _ => Err(
                translate_validation("validation.url", &[("attribute", field)])
                    .unwrap_or_else(|| format!("The {} field must be a valid URL.", field)),
            ),
        }
    }

    fn name(&self) -> &'static str {
        "url"
    }
}

/// Field must match a regex pattern.
pub struct Regex_ {
    pattern: Regex,
}

pub fn regex(pattern: &str) -> Regex_ {
    Regex_ {
        pattern: Regex::new(pattern).expect("Invalid regex pattern"),
    }
}

impl Rule for Regex_ {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }

        match value.as_str() {
            Some(s) if self.pattern.is_match(s) => Ok(()),
            _ => Err(
                translate_validation("validation.regex", &[("attribute", field)])
                    .unwrap_or_else(|| format!("The {} field format is invalid.", field)),
            ),
        }
    }

    fn name(&self) -> &'static str {
        "regex"
    }
}

static ALPHA_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-zA-Z]+$").unwrap());

/// Field must contain only alphabetic characters.
pub struct Alpha;

pub const fn alpha() -> Alpha {
    Alpha
}

impl Rule for Alpha {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }

        match value.as_str() {
            Some(s) if ALPHA_REGEX.is_match(s) => Ok(()),
            _ => Err(
                translate_validation("validation.alpha", &[("attribute", field)])
                    .unwrap_or_else(|| format!("The {} field must only contain letters.", field)),
            ),
        }
    }

    fn name(&self) -> &'static str {
        "alpha"
    }
}

static ALPHA_NUM_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9]+$").unwrap());

/// Field must contain only alphanumeric characters.
pub struct AlphaNum;

pub const fn alpha_num() -> AlphaNum {
    AlphaNum
}

impl Rule for AlphaNum {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }

        match value.as_str() {
            Some(s) if ALPHA_NUM_REGEX.is_match(s) => Ok(()),
            _ => Err(
                translate_validation("validation.alpha_num", &[("attribute", field)])
                    .unwrap_or_else(|| {
                        format!("The {} field must only contain letters and numbers.", field)
                    }),
            ),
        }
    }

    fn name(&self) -> &'static str {
        "alpha_num"
    }
}

static ALPHA_DASH_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap());

/// Field must contain only alphanumeric characters, dashes, and underscores.
pub struct AlphaDash;

pub const fn alpha_dash() -> AlphaDash {
    AlphaDash
}

impl Rule for AlphaDash {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }

        match value.as_str() {
            Some(s) if ALPHA_DASH_REGEX.is_match(s) => Ok(()),
            _ => Err(
                translate_validation("validation.alpha_dash", &[("attribute", field)])
                    .unwrap_or_else(|| {
                        format!(
                    "The {} field must only contain letters, numbers, dashes, and underscores.",
                    field
                )
                    }),
            ),
        }
    }

    fn name(&self) -> &'static str {
        "alpha_dash"
    }
}

// ============================================================================
// Comparison Rules
// ============================================================================

/// Field must match another field.
pub struct Confirmed {
    confirmation_field: String,
}

pub fn confirmed() -> Confirmed {
    Confirmed {
        confirmation_field: String::new(), // Will be set based on field name
    }
}

impl Rule for Confirmed {
    fn validate(&self, field: &str, value: &Value, data: &Value) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }

        let confirmation_field = if self.confirmation_field.is_empty() {
            format!("{}_confirmation", field)
        } else {
            self.confirmation_field.clone()
        };

        let confirmation_value = data.get(&confirmation_field);
        if confirmation_value == Some(value) {
            Ok(())
        } else {
            Err(
                translate_validation("validation.confirmed", &[("attribute", field)])
                    .unwrap_or_else(|| format!("The {} confirmation does not match.", field)),
            )
        }
    }

    fn name(&self) -> &'static str {
        "confirmed"
    }
}

/// Field must be in a list of values.
pub struct In {
    values: Vec<Value>,
}

pub fn in_array<I, V>(values: I) -> In
where
    I: IntoIterator<Item = V>,
    V: Into<Value>,
{
    In {
        values: values.into_iter().map(|v| v.into()).collect(),
    }
}

impl Rule for In {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }

        if self.values.contains(value) {
            Ok(())
        } else {
            Err(
                translate_validation("validation.in", &[("attribute", field)])
                    .unwrap_or_else(|| format!("The selected {} is invalid.", field)),
            )
        }
    }

    fn name(&self) -> &'static str {
        "in"
    }
}

/// Field must not be in a list of values.
pub struct NotIn {
    values: Vec<Value>,
}

pub fn not_in<I, V>(values: I) -> NotIn
where
    I: IntoIterator<Item = V>,
    V: Into<Value>,
{
    NotIn {
        values: values.into_iter().map(|v| v.into()).collect(),
    }
}

impl Rule for NotIn {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }

        if self.values.contains(value) {
            Err(
                translate_validation("validation.not_in", &[("attribute", field)])
                    .unwrap_or_else(|| format!("The selected {} is invalid.", field)),
            )
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "not_in"
    }
}

/// Field must be different from another field.
pub struct Different {
    other: String,
}

pub fn different(other: impl Into<String>) -> Different {
    Different {
        other: other.into(),
    }
}

impl Rule for Different {
    fn validate(&self, field: &str, value: &Value, data: &Value) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }

        let other_value = data.get(&self.other);
        if other_value == Some(value) {
            Err(translate_validation(
                "validation.different",
                &[("attribute", field), ("other", &self.other)],
            )
            .unwrap_or_else(|| {
                format!("The {} and {} fields must be different.", field, self.other)
            }))
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "different"
    }
}

/// Field must be the same as another field.
pub struct Same {
    other: String,
}

pub fn same(other: impl Into<String>) -> Same {
    Same {
        other: other.into(),
    }
}

impl Rule for Same {
    fn validate(&self, field: &str, value: &Value, data: &Value) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }

        let other_value = data.get(&self.other);
        if other_value != Some(value) {
            Err(translate_validation(
                "validation.same",
                &[("attribute", field), ("other", &self.other)],
            )
            .unwrap_or_else(|| format!("The {} and {} fields must match.", field, self.other)))
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "same"
    }
}

// ============================================================================
// Date Rules
// ============================================================================

/// Field must be a valid date.
pub struct Date;

pub const fn date() -> Date {
    Date
}

impl Rule for Date {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }

        if let Some(s) = value.as_str() {
            // Try common date formats
            if chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
                || chrono::NaiveDate::parse_from_str(s, "%d/%m/%Y").is_ok()
                || chrono::NaiveDate::parse_from_str(s, "%m/%d/%Y").is_ok()
                || chrono::DateTime::parse_from_rfc3339(s).is_ok()
            {
                return Ok(());
            }
        }

        Err(
            translate_validation("validation.date", &[("attribute", field)])
                .unwrap_or_else(|| format!("The {} field must be a valid date.", field)),
        )
    }

    fn name(&self) -> &'static str {
        "date"
    }
}

// ============================================================================
// Special Rules
// ============================================================================

/// Field is optional - only validate if present.
pub struct Nullable;

pub const fn nullable() -> Nullable {
    Nullable
}

impl Rule for Nullable {
    fn validate(&self, _field: &str, _value: &Value, _data: &Value) -> Result<(), String> {
        // This is a marker rule - validator handles it specially
        Ok(())
    }

    fn name(&self) -> &'static str {
        "nullable"
    }
}

/// Field must be accepted (yes, on, 1, true).
pub struct Accepted;

pub const fn accepted() -> Accepted {
    Accepted
}

impl Rule for Accepted {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        let accepted = match value {
            Value::Bool(true) => true,
            Value::Number(n) => n.as_i64() == Some(1),
            Value::String(s) => {
                matches!(s.to_lowercase().as_str(), "yes" | "on" | "1" | "true")
            }
            _ => false,
        };

        if accepted {
            Ok(())
        } else {
            Err(
                translate_validation("validation.accepted", &[("attribute", field)])
                    .unwrap_or_else(|| format!("The {} field must be accepted.", field)),
            )
        }
    }

    fn name(&self) -> &'static str {
        "accepted"
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get the size of a value (length for strings/arrays, value for numbers).
fn get_size(value: &Value) -> f64 {
    match value {
        Value::String(s) => s.chars().count() as f64,
        Value::Array(a) => a.len() as f64,
        Value::Object(o) => o.len() as f64,
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        _ => 0.0,
    }
}

/// Get the appropriate unit for size validation messages.
fn get_size_unit(value: &Value) -> &'static str {
    match value {
        Value::String(_) => "characters",
        Value::Array(_) => "items",
        Value::Object(_) => "items",
        _ => "",
    }
}

/// Get the type-specific translation key for size rules (min, max, between).
fn get_size_type_key(rule: &str, value: &Value) -> String {
    let suffix = match value {
        Value::String(_) => "string",
        Value::Array(_) | Value::Object(_) => "array",
        _ => "numeric",
    };
    format!("validation.{}.{}", rule, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_required() {
        let rule = required();
        let data = json!({});

        assert!(rule.validate("name", &json!("John"), &data).is_ok());
        assert!(rule.validate("name", &json!(null), &data).is_err());
        assert!(rule.validate("name", &json!(""), &data).is_err());
        assert!(rule.validate("name", &json!("  "), &data).is_err());
    }

    #[test]
    fn test_email() {
        let rule = email();
        let data = json!({});

        assert!(rule
            .validate("email", &json!("test@example.com"), &data)
            .is_ok());
        assert!(rule.validate("email", &json!("invalid"), &data).is_err());
        assert!(rule.validate("email", &json!(null), &data).is_ok());
    }

    #[test]
    fn test_min_max() {
        let data = json!({});

        // String length
        assert!(min(3).validate("name", &json!("John"), &data).is_ok());
        assert!(min(5).validate("name", &json!("John"), &data).is_err());

        assert!(max(5).validate("name", &json!("John"), &data).is_ok());
        assert!(max(2).validate("name", &json!("John"), &data).is_err());

        // Numeric value
        assert!(min(18).validate("age", &json!(25), &data).is_ok());
        assert!(min(18).validate("age", &json!(15), &data).is_err());
    }

    #[test]
    fn test_between() {
        let rule = between(1, 10);
        let data = json!({});

        assert!(rule.validate("count", &json!(5), &data).is_ok());
        assert!(rule.validate("count", &json!(0), &data).is_err());
        assert!(rule.validate("count", &json!(11), &data).is_err());
    }

    #[test]
    fn test_in_array() {
        let rule = in_array(["active", "inactive", "pending"]);
        let data = json!({});

        assert!(rule.validate("status", &json!("active"), &data).is_ok());
        assert!(rule.validate("status", &json!("unknown"), &data).is_err());
    }

    #[test]
    fn test_confirmed() {
        let rule = confirmed();
        let data = json!({
            "password": "secret123",
            "password_confirmation": "secret123"
        });

        assert!(rule
            .validate("password", &json!("secret123"), &data)
            .is_ok());

        let bad_data = json!({
            "password": "secret123",
            "password_confirmation": "different"
        });
        assert!(rule
            .validate("password", &json!("secret123"), &bad_data)
            .is_err());
    }

    #[test]
    fn test_url() {
        let rule = url();
        let data = json!({});

        assert!(rule
            .validate("website", &json!("https://example.com"), &data)
            .is_ok());
        assert!(rule
            .validate("website", &json!("http://example.com/path"), &data)
            .is_ok());
        assert!(rule
            .validate("website", &json!("not-a-url"), &data)
            .is_err());
    }

    #[test]
    fn test_rules_call_translate_validation() {
        fn mock(key: &str, params: &[(&str, &str)]) -> Option<String> {
            let attr = params
                .iter()
                .find(|(k, _)| *k == "attribute")
                .map(|(_, v)| *v)
                .unwrap_or("?");
            Some(format!("[translated] {}: attr={}", key, attr))
        }

        // OnceLock: if already set by another test, skip gracefully
        let was_set = crate::validation::bridge::VALIDATION_TRANSLATOR
            .set(mock as crate::validation::TranslatorFn)
            .is_ok();

        let result = required().validate("email", &json!(null), &json!({}));
        assert!(result.is_err());

        if was_set {
            let msg = result.unwrap_err();
            assert!(
                msg.contains("[translated]"),
                "Expected translated message, got: {}",
                msg
            );
            assert!(
                msg.contains("validation.required"),
                "Expected key in message, got: {}",
                msg
            );
        }
    }

    #[test]
    fn test_string() {
        let rule = string();
        let data = json!({});

        assert!(rule.validate("name", &json!("hello"), &data).is_ok());
        assert!(rule.validate("name", &json!(""), &data).is_ok());
        assert!(rule.validate("name", &json!(42), &data).is_err());
        assert!(rule.validate("name", &json!(true), &data).is_err());
        assert!(rule.validate("name", &json!([1, 2]), &data).is_err());
        // Null passthrough
        assert!(rule.validate("name", &json!(null), &data).is_ok());
    }

    #[test]
    fn test_integer() {
        let rule = integer();
        let data = json!({});

        assert!(rule.validate("age", &json!(42), &data).is_ok());
        assert!(rule.validate("age", &json!(0), &data).is_ok());
        assert!(rule.validate("age", &json!(-5), &data).is_ok());
        // String integers are accepted
        assert!(rule.validate("age", &json!("123"), &data).is_ok());
        // Floats are not integers
        assert!(rule.validate("age", &json!(3.17), &data).is_err());
        assert!(rule.validate("age", &json!("hello"), &data).is_err());
        assert!(rule.validate("age", &json!(true), &data).is_err());
        // Null passthrough
        assert!(rule.validate("age", &json!(null), &data).is_ok());
    }

    #[test]
    fn test_numeric() {
        let rule = numeric();
        let data = json!({});

        assert!(rule.validate("price", &json!(42), &data).is_ok());
        assert!(rule.validate("price", &json!(3.17), &data).is_ok());
        assert!(rule.validate("price", &json!(-10), &data).is_ok());
        // String numbers are accepted
        assert!(rule.validate("price", &json!("42.5"), &data).is_ok());
        assert!(rule.validate("price", &json!("hello"), &data).is_err());
        assert!(rule.validate("price", &json!(true), &data).is_err());
        // Null passthrough
        assert!(rule.validate("price", &json!(null), &data).is_ok());
    }

    #[test]
    fn test_boolean() {
        let rule = boolean();
        let data = json!({});

        assert!(rule.validate("active", &json!(true), &data).is_ok());
        assert!(rule.validate("active", &json!(false), &data).is_ok());
        // String booleans are accepted
        assert!(rule.validate("active", &json!("true"), &data).is_ok());
        assert!(rule.validate("active", &json!("false"), &data).is_ok());
        assert!(rule.validate("active", &json!("yes"), &data).is_ok());
        assert!(rule.validate("active", &json!("no"), &data).is_ok());
        assert!(rule.validate("active", &json!("1"), &data).is_ok());
        assert!(rule.validate("active", &json!("0"), &data).is_ok());
        // Integer 0 and 1 are accepted
        assert!(rule.validate("active", &json!(1), &data).is_ok());
        assert!(rule.validate("active", &json!(0), &data).is_ok());
        // Non-boolean strings rejected
        assert!(rule.validate("active", &json!("maybe"), &data).is_err());
        // Other integers rejected
        assert!(rule.validate("active", &json!(42), &data).is_err());
        // Null passthrough
        assert!(rule.validate("active", &json!(null), &data).is_ok());
    }

    #[test]
    fn test_array() {
        let rule = array();
        let data = json!({});

        assert!(rule.validate("items", &json!([1, 2, 3]), &data).is_ok());
        assert!(rule.validate("items", &json!([]), &data).is_ok());
        assert!(rule.validate("items", &json!(["a", "b"]), &data).is_ok());
        assert!(rule.validate("items", &json!("not array"), &data).is_err());
        assert!(rule.validate("items", &json!(42), &data).is_err());
        assert!(rule.validate("items", &json!(true), &data).is_err());
        // Null passthrough
        assert!(rule.validate("items", &json!(null), &data).is_ok());
    }

    #[test]
    fn test_required_if() {
        let data = json!({"role": "admin"});

        // role == admin, so name is required
        assert!(required_if("role", "admin")
            .validate("name", &json!(null), &data)
            .is_err());
        assert!(required_if("role", "admin")
            .validate("name", &json!(""), &data)
            .is_err());
        assert!(required_if("role", "admin")
            .validate("name", &json!("Alice"), &data)
            .is_ok());

        // role != "user", so name is not required
        assert!(required_if("role", "user")
            .validate("name", &json!(null), &data)
            .is_ok());
        assert!(required_if("role", "user")
            .validate("name", &json!(""), &data)
            .is_ok());
    }

    #[test]
    fn test_different() {
        let data = json!({"other_field": "b"});

        // Different values pass
        assert!(different("other_field")
            .validate("field", &json!("a"), &data)
            .is_ok());
        // Same values fail
        assert!(different("other_field")
            .validate("field", &json!("b"), &data)
            .is_err());
        // Null passthrough
        assert!(different("other_field")
            .validate("field", &json!(null), &data)
            .is_ok());
    }

    #[test]
    fn test_same() {
        let data = json!({"other_field": "a"});

        // Same values pass
        assert!(same("other_field")
            .validate("field", &json!("a"), &data)
            .is_ok());
        // Different values fail
        assert!(same("other_field")
            .validate("field", &json!("b"), &data)
            .is_err());
        // Null passthrough
        assert!(same("other_field")
            .validate("field", &json!(null), &data)
            .is_ok());
    }

    #[test]
    fn test_regex() {
        let rule = regex(r"^\d{3}-\d{4}$");
        let data = json!({});

        assert!(rule.validate("phone", &json!("123-4567"), &data).is_ok());
        assert!(rule.validate("phone", &json!("abc"), &data).is_err());
        assert!(rule.validate("phone", &json!("12-345"), &data).is_err());
        // Null passthrough
        assert!(rule.validate("phone", &json!(null), &data).is_ok());
    }

    #[test]
    fn test_alpha() {
        let rule = alpha();
        let data = json!({});

        assert!(rule.validate("name", &json!("Hello"), &data).is_ok());
        assert!(rule.validate("name", &json!("abc"), &data).is_ok());
        assert!(rule.validate("name", &json!("Hello123"), &data).is_err());
        assert!(rule.validate("name", &json!("hello world"), &data).is_err());
        // Empty string does not match alpha regex
        assert!(rule.validate("name", &json!(""), &data).is_err());
        // Null passthrough
        assert!(rule.validate("name", &json!(null), &data).is_ok());
    }

    #[test]
    fn test_alpha_num() {
        let rule = alpha_num();
        let data = json!({});

        assert!(rule.validate("code", &json!("Hello123"), &data).is_ok());
        assert!(rule.validate("code", &json!("abc"), &data).is_ok());
        assert!(rule.validate("code", &json!("123"), &data).is_ok());
        assert!(rule.validate("code", &json!("Hello!@#"), &data).is_err());
        assert!(rule.validate("code", &json!("hello world"), &data).is_err());
        // Null passthrough
        assert!(rule.validate("code", &json!(null), &data).is_ok());
    }

    #[test]
    fn test_alpha_dash() {
        let rule = alpha_dash();
        let data = json!({});

        assert!(rule
            .validate("slug", &json!("hello-world_123"), &data)
            .is_ok());
        assert!(rule.validate("slug", &json!("abc"), &data).is_ok());
        assert!(rule.validate("slug", &json!("a-b_c"), &data).is_ok());
        assert!(rule.validate("slug", &json!("hello world"), &data).is_err());
        assert!(rule.validate("slug", &json!("hello!@#"), &data).is_err());
        // Null passthrough
        assert!(rule.validate("slug", &json!(null), &data).is_ok());
    }

    #[test]
    fn test_not_in() {
        let rule = not_in(["banned", "blocked"]);
        let data = json!({});

        assert!(rule.validate("status", &json!("active"), &data).is_ok());
        assert!(rule.validate("status", &json!("approved"), &data).is_ok());
        assert!(rule.validate("status", &json!("banned"), &data).is_err());
        assert!(rule.validate("status", &json!("blocked"), &data).is_err());
        // Null passthrough
        assert!(rule.validate("status", &json!(null), &data).is_ok());
    }

    #[test]
    fn test_date() {
        let rule = date();
        let data = json!({});

        // ISO date format
        assert!(rule
            .validate("birthday", &json!("2024-01-15"), &data)
            .is_ok());
        // RFC3339 datetime
        assert!(rule
            .validate("created", &json!("2024-01-15T10:30:00Z"), &data)
            .is_ok());
        // Invalid strings
        assert!(rule
            .validate("birthday", &json!("not-a-date"), &data)
            .is_err());
        assert!(rule
            .validate("birthday", &json!("2024-13-01"), &data)
            .is_err());
        // Non-string
        assert!(rule.validate("birthday", &json!(42), &data).is_err());
        // Null passthrough
        assert!(rule.validate("birthday", &json!(null), &data).is_ok());
    }

    #[test]
    fn test_nullable() {
        let rule = nullable();
        let data = json!({});

        // Nullable always passes - it's a marker rule
        assert!(rule.validate("field", &json!(null), &data).is_ok());
        assert!(rule.validate("field", &json!("value"), &data).is_ok());
        assert!(rule.validate("field", &json!(42), &data).is_ok());
        assert!(rule.validate("field", &json!(true), &data).is_ok());
    }

    #[test]
    fn test_accepted() {
        let rule = accepted();
        let data = json!({});

        // Accepted values
        assert!(rule.validate("terms", &json!(true), &data).is_ok());
        assert!(rule.validate("terms", &json!("yes"), &data).is_ok());
        assert!(rule.validate("terms", &json!("on"), &data).is_ok());
        assert!(rule.validate("terms", &json!("1"), &data).is_ok());
        assert!(rule.validate("terms", &json!("true"), &data).is_ok());
        assert!(rule.validate("terms", &json!(1), &data).is_ok());

        // Rejected values
        assert!(rule.validate("terms", &json!(false), &data).is_err());
        assert!(rule.validate("terms", &json!("no"), &data).is_err());
        assert!(rule.validate("terms", &json!("off"), &data).is_err());
        assert!(rule.validate("terms", &json!(0), &data).is_err());
        assert!(rule.validate("terms", &json!(null), &data).is_err());
        assert!(rule.validate("terms", &json!("false"), &data).is_err());
    }

    #[test]
    fn test_size_type_key_selection() {
        // String values use "string" subkey
        assert_eq!(
            get_size_type_key("min", &json!("hello")),
            "validation.min.string"
        );
        assert_eq!(
            get_size_type_key("max", &json!("hello")),
            "validation.max.string"
        );
        assert_eq!(
            get_size_type_key("between", &json!("hello")),
            "validation.between.string"
        );

        // Numeric values use "numeric" subkey
        assert_eq!(
            get_size_type_key("min", &json!(42)),
            "validation.min.numeric"
        );
        assert_eq!(
            get_size_type_key("max", &json!(42)),
            "validation.max.numeric"
        );
        assert_eq!(
            get_size_type_key("between", &json!(42)),
            "validation.between.numeric"
        );

        // Array values use "array" subkey
        assert_eq!(
            get_size_type_key("min", &json!([1, 2, 3])),
            "validation.min.array"
        );
        assert_eq!(
            get_size_type_key("max", &json!([1, 2, 3])),
            "validation.max.array"
        );
        assert_eq!(
            get_size_type_key("between", &json!([1, 2, 3])),
            "validation.between.array"
        );
    }
}
