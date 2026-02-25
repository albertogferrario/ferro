//! Validation error types.

use serde::Serialize;
use std::collections::HashMap;

/// A collection of validation errors.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ValidationError {
    /// Field-specific errors.
    errors: HashMap<String, Vec<String>>,
}

impl ValidationError {
    /// Create a new empty validation error.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an error message for a field.
    pub fn add(&mut self, field: &str, message: impl Into<String>) {
        self.errors
            .entry(field.to_string())
            .or_default()
            .push(message.into());
    }

    /// Check if there are any errors.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if a specific field has errors.
    pub fn has(&self, field: &str) -> bool {
        self.errors.contains_key(field)
    }

    /// Get errors for a specific field.
    pub fn get(&self, field: &str) -> Option<&Vec<String>> {
        self.errors.get(field)
    }

    /// Get the first error for a field.
    pub fn first(&self, field: &str) -> Option<&String> {
        self.errors.get(field).and_then(|v| v.first())
    }

    /// Get all errors as a map.
    pub fn all(&self) -> &HashMap<String, Vec<String>> {
        &self.errors
    }

    /// Get the total number of errors.
    pub fn count(&self) -> usize {
        self.errors.values().map(|v| v.len()).sum()
    }

    /// Get all error messages as a flat list.
    pub fn messages(&self) -> Vec<&String> {
        self.errors.values().flatten().collect()
    }

    /// Consume the error and return the inner HashMap of field -> messages.
    /// Useful for passing errors to templates.
    pub fn into_messages(self) -> HashMap<String, Vec<String>> {
        self.errors
    }

    /// Convert to JSON-compatible format for API responses.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "message": "The given data was invalid.",
            "errors": self.errors
        })
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let messages: Vec<String> = self
            .errors
            .iter()
            .flat_map(|(field, msgs)| msgs.iter().map(move |m| format!("{field}: {m}")))
            .collect();
        write!(f, "{}", messages.join(", "))
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_add() {
        let mut errors = ValidationError::new();
        errors.add("email", "The email field is required.");
        errors.add("email", "The email must be a valid email address.");
        errors.add("password", "The password must be at least 8 characters.");

        assert!(!errors.is_empty());
        assert!(errors.has("email"));
        assert!(errors.has("password"));
        assert!(!errors.has("name"));
        assert_eq!(errors.count(), 3);
    }

    #[test]
    fn test_validation_error_first() {
        let mut errors = ValidationError::new();
        errors.add("email", "First error");
        errors.add("email", "Second error");

        assert_eq!(errors.first("email"), Some(&"First error".to_string()));
        assert_eq!(errors.first("name"), None);
    }

    #[test]
    fn test_validation_error_to_json() {
        let mut errors = ValidationError::new();
        errors.add("email", "Required");

        let json = errors.to_json();
        assert!(json.get("message").is_some());
        assert!(json.get("errors").is_some());
    }
}
