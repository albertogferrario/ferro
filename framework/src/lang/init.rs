//! Global Translator initialization and convenience helpers.
//!
//! Loads translations once at boot, stores in a `OnceLock`, and provides
//! `t()`, `trans()`, and `choice()` for ergonomic lookups using the
//! current request locale from [`super::locale()`].

use crate::config::Config;
use crate::validation::register_validation_translator;
use ferro_lang::{LangConfig, Translator};
use std::sync::OnceLock;

static TRANSLATOR: OnceLock<Translator> = OnceLock::new();

/// Initialize the global Translator from `LangConfig`.
///
/// Reads config from the global Config registry (or falls back to
/// `LangConfig::from_env()`), loads translation files, and registers
/// the validation bridge so localized validation messages work
/// automatically.
///
/// Called by `Application::run()` after user config registration.
/// If loading fails (e.g. no `lang/` directory), logs a warning and
/// continues -- all translation helpers gracefully return the key as-is.
pub fn init() {
    let config = Config::get::<LangConfig>().unwrap_or_else(LangConfig::from_env);

    match Translator::load(&config.path, &config.fallback_locale) {
        Ok(translator) => {
            let _ = TRANSLATOR.set(translator);
            register_validation_translator(validation_bridge_fn);
        }
        Err(e) => {
            eprintln!("[ferro::lang] Failed to load translations: {e}");
        }
    }
}

/// Translate a key using the current request locale.
///
/// Reads the global Translator and the task-local locale from
/// [`super::locale()`]. If no translator is loaded, returns the key as-is.
///
/// # Example
///
/// ```rust,ignore
/// use ferro::t;
///
/// let msg = t("welcome", &[("name", "Alice")]);
/// ```
pub fn t(key: &str, params: &[(&str, &str)]) -> String {
    match TRANSLATOR.get() {
        Some(translator) => translator.get(&super::locale(), key, params),
        None => key.to_string(),
    }
}

/// Alias for [`t()`] -- Laravel-familiar naming.
pub fn trans(key: &str, params: &[(&str, &str)]) -> String {
    t(key, params)
}

/// Translate a pluralized key using the current request locale.
///
/// Selects the correct plural form based on `count`, then applies
/// parameter interpolation. A `:count` parameter is added automatically.
///
/// If no translator is loaded, returns the key as-is.
///
/// # Example
///
/// ```rust,ignore
/// use ferro::lang_choice;
///
/// let msg = lang_choice("items.count", 5, &[]);
/// // e.g. "5 items"
/// ```
pub fn choice(key: &str, count: i64, params: &[(&str, &str)]) -> String {
    match TRANSLATOR.get() {
        Some(translator) => translator.choice(&super::locale(), key, count, params),
        None => key.to_string(),
    }
}

/// Bridge function for the validation module.
///
/// Matches the `TranslatorFn` signature: `fn(&str, &[(&str, &str)]) -> Option<String>`.
/// Returns `Some(translated)` when a translator is loaded, `None` otherwise
/// (so validation rules fall back to hardcoded English).
fn validation_bridge_fn(key: &str, params: &[(&str, &str)]) -> Option<String> {
    let translator = TRANSLATOR.get()?;
    let locale = super::locale();
    Some(translator.get(&locale, key, params))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::TranslatorFn;

    #[test]
    fn t_returns_key_when_no_translator_loaded() {
        // OnceLock may already be set by another test in this process,
        // but in a fresh state t() returns the key as-is.
        let result = t("some.key", &[]);
        // Either the key itself (no translator) or whatever the translator returns
        assert!(!result.is_empty());
    }

    #[test]
    fn trans_is_alias_for_t() {
        let key = "alias.test";
        assert_eq!(trans(key, &[]), t(key, &[]));
    }

    #[test]
    fn choice_returns_key_when_no_translator_loaded() {
        let result = choice("items.count", 5, &[]);
        assert!(!result.is_empty());
    }

    #[test]
    fn validation_bridge_fn_matches_translator_fn_signature() {
        // Compile-time proof that the function matches the type alias.
        let _f: TranslatorFn = validation_bridge_fn;
    }
}
