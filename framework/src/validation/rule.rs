//! Validation rule trait.

use serde_json::Value;

/// A validation rule that can be applied to a field.
pub trait Rule: Send + Sync {
    /// Validate the given value.
    ///
    /// Returns `Ok(())` if validation passes, or `Err(message)` if it fails.
    fn validate(&self, field: &str, value: &Value, data: &Value) -> Result<(), String>;

    /// Get the rule name for error messages.
    fn name(&self) -> &'static str;
}
