//! Validatable trait for derive-based validation.
//!
//! Structs implementing this trait can be validated using their field attributes.

use crate::validation::{Rule, ValidationError};

/// Trait for types that can validate themselves using declarative rules.
///
/// This trait is typically derived using `#[derive(Validate)]`:
///
/// ```rust,ignore
/// use ferro_rs::Validate;
///
/// #[derive(Validate)]
/// struct CreateUserRequest {
///     #[validate(required, email)]
///     email: String,
///
///     #[validate(required, min(8))]
///     password: String,
/// }
///
/// // Usage
/// let request = CreateUserRequest { ... };
/// request.validate()?;
/// ```
pub trait Validatable {
    /// Validate the struct against its declared rules.
    ///
    /// Returns `Ok(())` if validation passes, or `Err(ValidationError)` with
    /// all validation errors collected.
    fn validate(&self) -> Result<(), ValidationError>;

    /// Get the static rule definitions for this type.
    ///
    /// Returns a list of (field_name, rules) tuples that can be used
    /// for introspection (e.g., by ferro-mcp).
    fn validation_rules() -> Vec<(&'static str, Vec<Box<dyn Rule>>)>;
}
