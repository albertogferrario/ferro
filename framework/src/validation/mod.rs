//! Request validation for Ferro framework.
//!
//! Provides Laravel-inspired validation with declarative rules.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::validation::{Validator, rules};
//!
//! let data = serde_json::json!({
//!     "email": "user@example.com",
//!     "password": "secret123",
//!     "age": 25
//! });
//!
//! let validator = Validator::new(&data)
//!     .rules("email", rules![required, email])
//!     .rules("password", rules![required, min(8)])
//!     .rules("age", rules![required, integer, min(18)]);
//!
//! if let Err(errors) = validator.validate() {
//!     println!("Validation failed: {:?}", errors);
//! }
//! ```

mod bridge;
mod error;
mod rule;
mod rules;
mod validatable;
mod validator;

pub use bridge::{register_validation_translator, TranslatorFn};
pub(crate) use bridge::translate_validation;
pub use error::ValidationError;
pub use rule::Rule;
pub use rules::*;
pub use validatable::Validatable;
pub use validator::{validate, Validator};

/// Macro for creating a vector of boxed validation rules.
///
/// This macro boxes each rule, allowing different rule types to be stored
/// together in a single vector.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::validation::{Validator, rules::*};
/// use ferro_rs::rules;
///
/// let validator = Validator::new(&data)
///     .rules("email", rules![required(), email()])
///     .rules("name", rules![required(), string(), max(255)]);
/// ```
#[macro_export]
macro_rules! rules {
    ($($rule:expr),* $(,)?) => {
        vec![$(Box::new($rule) as Box<dyn $crate::validation::Rule>),*]
    };
}
