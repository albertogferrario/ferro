//! Validation error types.

use serde::Serialize;
use std::collections::HashMap;

/// A collection of validation errors.
///
/// Carries an optional `old_input` payload for the flash round-trip; set via
/// `with_old_input()` before calling `redirect_back()` or `redirect_to()`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ValidationError {
    /// Field-specific errors.
    errors: HashMap<String, Vec<String>>,
    /// Submitted form values to restore after a failed validation round-trip.
    /// Stored separately from `errors` so it is never serialised into API error
    /// responses (the `#[serde(skip)]` attribute).
    #[serde(skip)]
    old_input: Option<serde_json::Value>,
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

    // ── Phase 137: flash round-trip helpers ───────────────────────────────────

    /// Attach submitted form values as "old input" for the next GET request.
    ///
    /// Chain before `redirect_back()` or `redirect_to()`.  The caller typically
    /// passes the same `serde_json::Value` used to construct the `Validator`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Err(e) = validator.validate() {
    ///     return e.with_old_input(&form_data).redirect_back(referer);
    /// }
    /// ```
    pub fn with_old_input(mut self, data: &serde_json::Value) -> Self {
        self.old_input = Some(data.clone());
        self
    }

    /// Flash errors + old input into the session, then redirect to `referer`.
    ///
    /// Falls back to `"/"` when `referer` is `None` or when the Referer header
    /// contains a non-same-origin URL (T-92-05 mitigation).
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Err(e) = validator.validate() {
    ///     let referer = req.header("Referer");
    ///     return e.with_old_input(&form_data).redirect_back(referer);
    /// }
    /// ```
    pub fn redirect_back(self, referer: Option<&str>) -> crate::http::Response {
        // T-92-05: reject non-same-origin Referer values.
        let target = match referer {
            Some(r) if is_same_origin(r) => r.to_string(),
            _ => "/".to_string(),
        };
        self.flash_into_session();
        crate::http::Redirect::to(target).into()
    }

    /// Flash errors + old input into the session, then redirect to an explicit URL.
    ///
    /// Use this instead of `redirect_back()` when the calling controller knows
    /// the exact destination (e.g. a tabbed settings form where the Referer may
    /// lack the `?tab=...` parameter).
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Err(e) = validator.validate() {
    ///     return e.with_old_input(&form_data).redirect_to("/settings?tab=generale");
    /// }
    /// ```
    pub fn redirect_to(self, url: impl Into<String>) -> crate::http::Response {
        self.flash_into_session();
        crate::http::Redirect::to(url.into()).into()
    }

    /// Write the error map and optional old input into the session flash store.
    ///
    /// Uses the reserved key prefix `_validation_errors` / `_old_input.<field>`
    /// under `_flash.new.*` (T-92-03 namespace isolation).
    fn flash_into_session(self) {
        let errors = self.errors;
        let old = self.old_input;
        crate::session::session_mut(|session| {
            session.flash("_validation_errors", &errors);
            if let Some(serde_json::Value::Object(map)) = old {
                for (k, v) in map {
                    let stringified = match v {
                        serde_json::Value::String(s) => s,
                        serde_json::Value::Null => continue,
                        other => other.to_string(),
                    };
                    session.flash(&format!("_old_input.{k}"), &stringified);
                }
            }
        });
    }
}

/// Returns `true` when `url` is a relative path or same-origin absolute URL.
///
/// Rejects any URL that has a scheme (`http://`, `https://`, etc.) pointing
/// to a different origin.  A bare path like `/dashboard/prodotti` is always
/// safe.  This is the T-92-05 Referer-forgery mitigation.
fn is_same_origin(url: &str) -> bool {
    // Relative paths are always safe.
    if url.starts_with('/') {
        return true;
    }
    // Absolute URLs with a scheme pointing to external hosts are rejected.
    false
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

    // ── Phase 137 tests: redirect_back / redirect_to / with_old_input ─────────

    #[test]
    fn test_redirect_back_returns_302_to_fallback_when_no_referer() {
        let mut errors = ValidationError::new();
        errors.add("email", "required");
        let response = errors.redirect_back(None);
        // redirect_back(None) must fall back to "/"
        let resp = response.unwrap();
        assert_eq!(resp.status_code(), 302);
        let hyper_resp = resp.into_hyper();
        let location = hyper_resp
            .headers()
            .get("Location")
            .and_then(|v| v.to_str().ok());
        assert_eq!(location, Some("/"));
    }

    #[test]
    fn test_redirect_back_with_explicit_referer() {
        let mut errors = ValidationError::new();
        errors.add("name", "required");
        let response = errors.redirect_back(Some("/dashboard/prodotti/nuovo"));
        let resp = response.unwrap();
        assert_eq!(resp.status_code(), 302);
        let hyper_resp = resp.into_hyper();
        let location = hyper_resp
            .headers()
            .get("Location")
            .and_then(|v| v.to_str().ok());
        assert_eq!(location, Some("/dashboard/prodotti/nuovo"));
    }

    #[test]
    fn test_redirect_back_rejects_external_referer() {
        // T-92-05: non-same-origin Referer must fall back to "/"
        let mut errors = ValidationError::new();
        errors.add("name", "required");
        let response = errors.redirect_back(Some("https://evil.example.com/phishing"));
        let resp = response.unwrap();
        assert_eq!(resp.status_code(), 302);
        let hyper_resp = resp.into_hyper();
        let location = hyper_resp
            .headers()
            .get("Location")
            .and_then(|v| v.to_str().ok());
        assert_eq!(location, Some("/"));
    }

    #[test]
    fn test_redirect_to_returns_302_to_explicit_url() {
        let mut errors = ValidationError::new();
        errors.add("slug", "invalid");
        let response = errors.redirect_to("/settings?tab=generale");
        let resp = response.unwrap();
        assert_eq!(resp.status_code(), 302);
        let hyper_resp = resp.into_hyper();
        let location = hyper_resp
            .headers()
            .get("Location")
            .and_then(|v| v.to_str().ok());
        assert_eq!(location, Some("/settings?tab=generale"));
    }

    #[test]
    fn test_with_old_input_chaining() {
        // Verify with_old_input() is chainable and does not panic.
        // We cannot inspect session flash in a unit test (no task-local context),
        // but we verify the method compiles and returns Self.
        let mut errors = ValidationError::new();
        errors.add("email", "required");
        let data = serde_json::json!({"email": "bad@"});
        // Should not panic; returns a new ValidationError with old_input set.
        let e = errors.with_old_input(&data);
        assert!(!e.is_empty());
    }
}
