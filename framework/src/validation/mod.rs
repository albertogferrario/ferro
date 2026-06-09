//! Request validation for Ferro framework.
//!
//! Provides Laravel-inspired validation with declarative rules.
//!
//! ## Flash round-trip (Phase 137)
//!
//! On validation failure a POST handler can redirect back and preserve both
//! errors and old form input in the session flash:
//!
//! ```rust,ignore
//! use ferro_rs::validation::{Validator, rules::*};
//! use ferro_rs::rules;
//!
//! // POST handler
//! let data = req.input::<serde_json::Value>().await?;
//! if let Err(e) = Validator::new(&data)
//!     .rules("name", rules![required()])
//!     .validate()
//! {
//!     let referer = req.header("Referer");
//!     return e.with_old_input(&data).redirect_back(referer);
//! }
//!
//! // GET handler — repopulate form fields
//! InputProps {
//!     default_value: req.old("name"),
//!     error: req.validation_error("name"),
//!     ..Default::default()
//! }
//! ```
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

mod async_rule;
mod async_validator;
mod bridge;
mod constraint_map;
mod error;
mod rule;
mod rules;
mod rules_async;
mod validatable;
mod validator;

pub use async_rule::AsyncRule;
pub use async_validator::{AsyncValidationError, AsyncValidator};
pub(crate) use bridge::translate_validation;
pub use bridge::{register_validation_translator, TranslatorFn};
pub use constraint_map::{ConstraintMap, MapConstraintExt};
pub use error::ValidationError;
pub use rule::Rule;
pub use rules::*;
pub use rules_async::unique;
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
