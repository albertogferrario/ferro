//! Async validation rule trait.

use async_trait::async_trait;
use serde_json::Value;

/// An async validation rule for DB-backed or I/O-bound checks.
///
/// The async counterpart to [`crate::validation::Rule`]. Implement this for
/// rules that require async I/O (e.g. DB lookups). Stored as
/// `Box<dyn AsyncRule>`, mirroring `Box<dyn Rule>`.
///
/// `#[async_trait]` is required because stable Rust async fn in traits is not
/// dyn-compatible — `Box<dyn AsyncRule>` needs the `async_trait` transform.
///
/// # Error semantics (D-12)
/// `validate` returns `Err(message)` ONLY for field-level validation failures.
/// I/O / infrastructure errors (e.g. a DB connection error) MUST NOT be
/// returned as a validation message. They are signalled to the validator with
/// a message prefixed `__infra_error__:` so [`crate::validation::AsyncValidator`]
/// can re-raise them as a framework error (HTTP 500) instead of a field error.
///
/// # Safety
/// `Send + Sync` are required: boxed trait objects are held across `.await`
/// points and shared across async request tasks.
#[async_trait]
pub trait AsyncRule: Send + Sync {
    /// Validate the field value. `Ok(())` on pass; `Err(message)` on a
    /// field-level validation failure. See trait docs for the
    /// `__infra_error__:` sentinel used for infrastructure failures.
    async fn validate(
        &self,
        field: &str,
        value: &Value,
        data: &Value,
    ) -> Result<(), String>;

    /// Rule name, used for custom-message lookup (e.g. `"unique"`).
    fn name(&self) -> &'static str;
}
