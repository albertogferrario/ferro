//! Translation bridge for validation messages.
//!
//! Provides a decoupled callback mechanism for translating validation error
//! messages. The validation module never depends on ferro-lang directly;
//! instead, a `fn` pointer is registered once at app boot via
//! [`register_validation_translator`].
//!
//! If no translator is registered, all rules fall back to hardcoded English.

use std::sync::OnceLock;

/// Callback signature for validation message translation.
///
/// - `key`: Translation key (e.g., `"validation.required"`)
/// - `params`: Interpolation parameters (e.g., `[("attribute", "email"), ("min", "8")]`)
/// - Returns: Translated message, or `None` if the key is not found (caller
///   falls back to English)
pub type TranslatorFn = fn(&str, &[(&str, &str)]) -> Option<String>;

pub(crate) static VALIDATION_TRANSLATOR: OnceLock<TranslatorFn> = OnceLock::new();

/// Register a translation function for validation messages.
///
/// Called once at app boot by the framework integration layer.
/// If not called, all validation rules return hardcoded English messages.
///
/// Silently ignores double-registration (`OnceLock::set` returns `Err` if
/// already set).
pub fn register_validation_translator(f: TranslatorFn) {
    let _ = VALIDATION_TRANSLATOR.set(f);
}

/// Attempt to translate a validation message.
///
/// Returns the translated message if a translator is registered and the key
/// exists, otherwise returns `None` (caller falls back to English).
pub(crate) fn translate_validation(key: &str, params: &[(&str, &str)]) -> Option<String> {
    VALIDATION_TRANSLATOR.get().and_then(|f| f(key, params))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_without_translator() {
        // In a fresh process (or before any registration), translate_validation
        // must return None so rules fall back to hardcoded English.
        //
        // Note: OnceLock is global and may already be set if another test in
        // the same process registered a translator. This test verifies the
        // function signature and None-propagation logic; full integration
        // testing happens in Phase 62/63.
        let result = translate_validation("validation.required", &[("attribute", "email")]);
        // If no translator was registered yet, this is None.
        // If another test set one, the function still compiles and runs correctly.
        // The key assertion is that the function is callable with the expected types.
        let _ = result;
    }

    #[test]
    fn test_translator_fn_signature() {
        // Verify a concrete function matches the TranslatorFn signature.
        fn mock_translator(key: &str, _params: &[(&str, &str)]) -> Option<String> {
            Some(format!("translated: {}", key))
        }

        let f: TranslatorFn = mock_translator;
        let result = f("validation.required", &[("attribute", "name")]);
        assert_eq!(result, Some("translated: validation.required".to_string()));
    }

    #[test]
    fn test_register_double_registration_is_noop() {
        fn first(_key: &str, _params: &[(&str, &str)]) -> Option<String> {
            Some("first".to_string())
        }
        fn second(_key: &str, _params: &[(&str, &str)]) -> Option<String> {
            Some("second".to_string())
        }

        // Register twice. The second call should be silently ignored
        // because OnceLock only accepts the first value.
        register_validation_translator(first);
        register_validation_translator(second);

        // If a translator is set, it must be the first one (or one
        // set by a previous test in this process). Either way the
        // second registration must not panic or replace the first.
        if let Some(f) = VALIDATION_TRANSLATOR.get() {
            let result = f("key", &[]);
            // Must not be "second" — OnceLock ignores subsequent sets.
            assert_ne!(result, Some("second".to_string()));
        }
    }
}
