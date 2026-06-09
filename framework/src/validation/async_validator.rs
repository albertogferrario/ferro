//! Async validator builder and error types.

// async_validator is declared in a private module; pub use re-exports are wired
// in Plan 04. Suppress dead_code until the pub use chain is in place.
#![allow(dead_code)]

use std::collections::HashMap;

use serde_json::Value;

use crate::error::FrameworkError;
use crate::validation::{Rule, ValidationError};

use super::async_rule::AsyncRule;

/// Errors from [`AsyncValidator::validate_async`].
///
/// Separates field-level validation failures (→ redirect-back with old input)
/// from infrastructure failures (→ HTTP 500). A DB error is NEVER a validation
/// result.
///
/// # Usage
///
/// ```rust,ignore
/// match validator.validate_async().await {
///     Ok(()) => { /* proceed */ }
///     Err(AsyncValidationError::Validation(e)) => {
///         return Err(e.with_old_input(&data).into_action_error("/back"));
///     }
///     Err(AsyncValidationError::Infra(fe)) => {
///         return Err(ActionError::from(fe));
///     }
/// }
/// ```
#[derive(Debug)]
pub enum AsyncValidationError {
    /// One or more field validation rules failed. Use `.with_old_input()` +
    /// `redirect_back` / `redirect_to` / `into_action_error` as usual.
    Validation(ValidationError),
    /// A DB or infrastructure error occurred during an async rule. Propagate
    /// as a framework error (→ 500).
    Infra(FrameworkError),
}

impl std::fmt::Display for AsyncValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(e) => write!(f, "Validation failed: {e}"),
            Self::Infra(e) => write!(f, "Infrastructure error: {e}"),
        }
    }
}

impl std::error::Error for AsyncValidationError {}

impl From<AsyncValidationError> for crate::http::action::ActionError {
    fn from(e: AsyncValidationError) -> Self {
        match e {
            // Caller is expected to flash errors via with_old_input before
            // converting; validation_failed suppresses the redundant envelope.
            AsyncValidationError::Validation(_) => {
                crate::http::action::ActionError::validation_failed("/")
            }
            AsyncValidationError::Infra(fe) => crate::http::action::ActionError::from(fe),
        }
    }
}

/// Async request validator.
///
/// Mirrors [`crate::validation::Validator`] ergonomics while adding support for
/// `Box<dyn AsyncRule>` rules (e.g. DB uniqueness checks). Sync rules run
/// first; async rules run only on fields with no sync error (fail-fast, D-03).
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::{AsyncValidator, AsyncValidationError, unique};
/// use ferro_rs::validation::rules::*;
/// use ferro_rs::rules;
///
/// let data = req.input::<serde_json::Value>().await?;
/// match AsyncValidator::new(&data)
///     .rules("slug", rules![required(), string()])
///     .async_rule("slug", unique("articles", "slug"))
///     .validate_async()
///     .await
/// {
///     Ok(()) => {}
///     Err(AsyncValidationError::Validation(e)) => {
///         return Err(e.with_old_input(&data).into_action_error("/articles/new"));
///     }
///     Err(AsyncValidationError::Infra(fe)) => {
///         return Err(fe.into());
///     }
/// }
/// ```
pub struct AsyncValidator<'a> {
    data: &'a Value,
    sync_rules: HashMap<String, Vec<Box<dyn Rule>>>,
    async_rules: HashMap<String, Vec<Box<dyn AsyncRule>>>,
    custom_messages: HashMap<String, String>,
    custom_attributes: HashMap<String, String>,
}

impl<'a> AsyncValidator<'a> {
    /// Create a new async validator for the given data.
    pub fn new(data: &'a Value) -> Self {
        Self {
            data,
            sync_rules: HashMap::new(),
            async_rules: HashMap::new(),
            custom_messages: HashMap::new(),
            custom_attributes: HashMap::new(),
        }
    }

    /// Add a single sync validation rule for a field.
    pub fn rule<R: Rule + 'static>(mut self, field: impl Into<String>, rule: R) -> Self {
        let field = field.into();
        self.sync_rules
            .entry(field)
            .or_default()
            .push(Box::new(rule) as Box<dyn Rule>);
        self
    }

    /// Add multiple sync validation rules for a field using boxed rules.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_rs::rules;
    /// use ferro_rs::validation::rules::*;
    ///
    /// AsyncValidator::new(&data)
    ///     .rules("email", rules![required(), email()])
    ///     .rules("name", rules![required(), string(), max(255)]);
    /// ```
    pub fn rules(mut self, field: impl Into<String>, rules: Vec<Box<dyn Rule>>) -> Self {
        self.sync_rules.insert(field.into(), rules);
        self
    }

    /// Add a single async validation rule for a field.
    ///
    /// Async rules run only after all sync rules pass for the field (D-03).
    pub fn async_rule<R: AsyncRule + 'static>(mut self, field: impl Into<String>, rule: R) -> Self {
        self.async_rules
            .entry(field.into())
            .or_default()
            .push(Box::new(rule) as Box<dyn AsyncRule>);
        self
    }

    /// Set a custom error message for a field.rule combination.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// AsyncValidator::new(&data)
    ///     .rules("email", rules![required(), email()])
    ///     .message("email.required", "Please provide your email address");
    /// ```
    pub fn message(mut self, key: impl Into<String>, message: impl Into<String>) -> Self {
        self.custom_messages.insert(key.into(), message.into());
        self
    }

    /// Set custom messages from a map.
    pub fn messages(mut self, messages: HashMap<String, String>) -> Self {
        self.custom_messages.extend(messages);
        self
    }

    /// Set a custom attribute name for a field.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// AsyncValidator::new(&data)
    ///     .rules("email", rules![required()])
    ///     .attribute("email", "email address");
    /// // Error: "The email address field is required."
    /// ```
    pub fn attribute(mut self, field: impl Into<String>, name: impl Into<String>) -> Self {
        self.custom_attributes.insert(field.into(), name.into());
        self
    }

    /// Set custom attributes from a map.
    pub fn attributes(mut self, attributes: HashMap<String, String>) -> Self {
        self.custom_attributes.extend(attributes);
        self
    }

    /// Run async validation. Returns:
    ///
    /// - `Ok(())` — all rules pass.
    /// - `Err(AsyncValidationError::Validation(e))` — field-level failures.
    /// - `Err(AsyncValidationError::Infra(e))` — DB/infra failure (handler → 500).
    ///
    /// # Execution order (D-03)
    ///
    /// Phase 1: all sync rules run across all fields.
    /// Phase 2: async rules run only on fields with no sync error — no DB query
    /// is issued for an already-failed field.
    ///
    /// # Infra sentinel (D-12)
    ///
    /// An async rule that returns `Err(msg)` where `msg` starts with
    /// `__infra_error__:` is treated as an infrastructure failure, not a
    /// field error. The stripped message is wrapped in
    /// `AsyncValidationError::Infra(FrameworkError::database(...))` and
    /// returned immediately.
    pub async fn validate_async(self) -> Result<(), AsyncValidationError> {
        let mut errors = ValidationError::new();

        // Phase 1 — sync rules first (verbatim from Validator::validate).
        for (field, rules) in &self.sync_rules {
            let value = self.get_value(field);
            let display_field = self.get_display_field(field);

            // nullable() rule: skip all other rules for this field if value is null.
            let has_nullable = rules.iter().any(|r| r.name() == "nullable");
            if has_nullable && value.is_null() {
                continue;
            }

            for rule in rules {
                // Skip nullable rule itself — it has no validation message.
                if rule.name() == "nullable" {
                    continue;
                }

                if let Err(default_message) = rule.validate(&display_field, &value, self.data) {
                    let message_key = format!("{}.{}", field, rule.name());
                    let message = self
                        .custom_messages
                        .get(&message_key)
                        .cloned()
                        .unwrap_or(default_message);
                    errors.add(field, message);
                }
            }
        }

        // Phase 2 — async rules only on fields with no sync error (D-03).
        for (field, rules) in &self.async_rules {
            // Fail-fast: no DB query for an already-failed field.
            if errors.has(field) {
                continue;
            }

            let value = self.get_value(field);

            // nullable mirror: if this field carries a sync nullable() rule and
            // the value is null, skip async rules too (prevents a DB query for
            // a null value — mirrors sync behavior).
            if value.is_null() {
                let nullable = self
                    .sync_rules
                    .get(field)
                    .map(|rs| rs.iter().any(|r| r.name() == "nullable"))
                    .unwrap_or(false);
                if nullable {
                    continue;
                }
            }

            let display_field = self.get_display_field(field);

            for rule in rules {
                match rule.validate(&display_field, &value, self.data).await {
                    Ok(()) => {}
                    Err(msg) => {
                        // D-12: infra failures are NOT field errors.
                        if let Some(rest) = msg.strip_prefix("__infra_error__:") {
                            return Err(AsyncValidationError::Infra(FrameworkError::database(
                                rest.trim().to_string(),
                            )));
                        }
                        let message_key = format!("{}.{}", field, rule.name());
                        let message = self
                            .custom_messages
                            .get(&message_key)
                            .cloned()
                            .unwrap_or(msg);
                        errors.add(field, message);
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AsyncValidationError::Validation(errors))
        }
    }

    /// Get a value from the data, supporting dot notation.
    fn get_value(&self, field: &str) -> Value {
        get_nested_value(self.data, field)
            .cloned()
            .unwrap_or(Value::Null)
    }

    /// Get the display name for a field.
    fn get_display_field(&self, field: &str) -> String {
        self.custom_attributes
            .get(field)
            .cloned()
            .unwrap_or_else(|| field.split('_').collect::<Vec<_>>().join(" "))
    }
}

/// Get a nested value from JSON using dot notation (verbatim from validator.rs).
fn get_nested_value<'a>(data: &'a Value, path: &str) -> Option<&'a Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = data;

    for part in parts {
        // Try as object key.
        if let Value::Object(map) = current {
            current = map.get(part)?;
        }
        // Try as array index.
        else if let Value::Array(arr) = current {
            let index: usize = part.parse().ok()?;
            current = arr.get(index)?;
        } else {
            return None;
        }
    }

    Some(current)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use async_trait::async_trait;
    use serde_json::json;
    use serial_test::serial;

    use super::*;
    use crate::validation::rules::*;
    use crate::rules;

    // -----------------------------------------------------------------------
    // Tiny test AsyncRule implementations.
    // -----------------------------------------------------------------------

    /// Always returns Ok(()).
    struct OkRule;

    #[async_trait]
    impl AsyncRule for OkRule {
        async fn validate(&self, _field: &str, _value: &Value, _data: &Value) -> Result<(), String> {
            Ok(())
        }

        fn name(&self) -> &'static str {
            "ok_rule"
        }
    }

    /// Increments a shared counter on every validate() call, then returns Ok(()).
    struct CountingRule {
        counter: Arc<AtomicUsize>,
    }

    impl CountingRule {
        fn new(counter: Arc<AtomicUsize>) -> Self {
            Self { counter }
        }
    }

    #[async_trait]
    impl AsyncRule for CountingRule {
        async fn validate(&self, _field: &str, _value: &Value, _data: &Value) -> Result<(), String> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn name(&self) -> &'static str {
            "counting_rule"
        }
    }

    /// Always returns Err("__infra_error__: boom").
    struct InfraRule;

    #[async_trait]
    impl AsyncRule for InfraRule {
        async fn validate(&self, _field: &str, _value: &Value, _data: &Value) -> Result<(), String> {
            Err("__infra_error__: boom".to_string())
        }

        fn name(&self) -> &'static str {
            "infra_rule"
        }
    }

    /// Always returns a validation failure.
    struct FailRule;

    #[async_trait]
    impl AsyncRule for FailRule {
        async fn validate(&self, field: &str, _value: &Value, _data: &Value) -> Result<(), String> {
            Err(format!("The {field} rule failed."))
        }

        fn name(&self) -> &'static str {
            "fail_rule"
        }
    }

    // -----------------------------------------------------------------------
    // Inline DB fixture (mirrors async_rule_fixture.rs).
    // -----------------------------------------------------------------------

    async fn init_test_db() {
        use crate::database::{DatabaseConfig, DB};
        use sea_orm::{ConnectionTrait, Statement};
        let config = DatabaseConfig::builder().url("sqlite::memory:").build();
        DB::init_with(config).await.expect("init in-memory sqlite");
        let db = DB::connection().expect("connection after init");
        db.execute(Statement::from_string(
            db.get_database_backend(),
            "CREATE TABLE IF NOT EXISTS widgets (id INTEGER PRIMARY KEY, slug TEXT)".to_owned(),
        ))
        .await
        .expect("create widgets scratch table");
    }

    async fn seed_widget(id: i64, slug: &str) {
        use crate::database::DB;
        use sea_orm::{ConnectionTrait, Statement};
        let db = DB::connection().expect("connection for seed_widget");
        db.execute(Statement::from_string(
            db.get_database_backend(),
            format!("INSERT INTO widgets (id, slug) VALUES ({id}, '{slug}')"),
        ))
        .await
        .expect("seed widget row");
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn async_validator_all_pass() {
        let data = json!({"name": "Alice"});
        let result = AsyncValidator::new(&data)
            .rule("name", required())
            .async_rule("name", OkRule)
            .validate_async()
            .await;
        assert!(result.is_ok(), "expected Ok(()), got: {result:?}");
    }

    #[tokio::test]
    async fn async_validator_sync_first() {
        // Sync rule fails → async rule must never run.
        let counter = Arc::new(AtomicUsize::new(0));
        let data = json!({"name": ""});
        let result = AsyncValidator::new(&data)
            .rule("name", required())
            .async_rule("name", CountingRule::new(counter.clone()))
            .validate_async()
            .await;
        assert!(result.is_err(), "expected Err (sync failure)");
        assert_eq!(
            counter.load(Ordering::SeqCst),
            0,
            "async rule must not run when sync rule fails"
        );
    }

    #[tokio::test]
    async fn async_validator_skips_async_on_sync_error() {
        // Same as above but with rules![] helper; checks the error shape.
        let counter = Arc::new(AtomicUsize::new(0));
        let data = json!({"email": ""});
        let result = AsyncValidator::new(&data)
            .rules("email", rules![required()])
            .async_rule("email", CountingRule::new(counter.clone()))
            .validate_async()
            .await;
        match result {
            Err(AsyncValidationError::Validation(e)) => {
                assert!(e.has("email"), "expected 'email' field error");
            }
            other => panic!("expected Validation error, got {other:?}"),
        }
        assert_eq!(
            counter.load(Ordering::SeqCst),
            0,
            "async rule counter must be 0 (no DB query issued)"
        );
    }

    #[tokio::test]
    async fn async_validator_infra_error_shape() {
        // An async rule returning __infra_error__: → AsyncValidationError::Infra,
        // NOT Validation. The field error map must not carry the raw message.
        let data = json!({"slug": "something"});
        let result = AsyncValidator::new(&data)
            .async_rule("slug", InfraRule)
            .validate_async()
            .await;
        match result {
            Err(AsyncValidationError::Infra(_)) => {
                // Correct — infra errors must not be field errors.
            }
            Err(AsyncValidationError::Validation(e)) => {
                // Check the field error does not carry the raw sentinel.
                let msgs = e.get("slug").cloned().unwrap_or_default();
                for m in &msgs {
                    assert!(
                        !m.contains("__infra_error__"),
                        "infra sentinel must not appear in field errors: {m}"
                    );
                }
                panic!("expected Infra error, got Validation with: {msgs:?}");
            }
            Ok(()) => panic!("expected Err(Infra), got Ok(())"),
        }
    }

    #[tokio::test]
    async fn async_validator_nullable_skips_async() {
        // Field with nullable() + null value → async rule never runs.
        let counter = Arc::new(AtomicUsize::new(0));
        let data = json!({"nickname": null});
        let result = AsyncValidator::new(&data)
            .rules("nickname", rules![nullable()])
            .async_rule("nickname", CountingRule::new(counter.clone()))
            .validate_async()
            .await;
        assert!(
            result.is_ok(),
            "nullable null field should pass, got: {result:?}"
        );
        assert_eq!(
            counter.load(Ordering::SeqCst),
            0,
            "async rule must not run for null nullable field"
        );
    }

    #[tokio::test]
    async fn async_validator_validation_failure_shape() {
        // Async rule that fails → AsyncValidationError::Validation with the field.
        let data = json!({"name": "Alice"});
        let result = AsyncValidator::new(&data)
            .async_rule("name", FailRule)
            .validate_async()
            .await;
        match result {
            Err(AsyncValidationError::Validation(e)) => {
                assert!(e.has("name"), "expected 'name' field error");
            }
            other => panic!("expected Validation error, got {other:?}"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn async_validator_unique_duplicate_is_validation() {
        // Real unique rule with a seeded duplicate → AsyncValidationError::Validation.
        init_test_db().await;
        seed_widget(1, "taken").await;

        let data = json!({"slug": "taken"});
        let result = AsyncValidator::new(&data)
            .async_rule("slug", crate::validation::rules_async::unique("widgets", "slug"))
            .validate_async()
            .await;
        match result {
            Err(AsyncValidationError::Validation(e)) => {
                assert!(
                    e.has("slug"),
                    "expected 'slug' field error for duplicate, errors: {e:?}"
                );
            }
            other => panic!("expected Validation error for duplicate, got {other:?}"),
        }
    }
}
