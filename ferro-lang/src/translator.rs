use std::collections::HashMap;

use crate::error::LangError;
use crate::interpolation::interpolate;
use crate::loader::{load_translations, normalize_locale};
use crate::pluralization::select_plural_form;

/// Core translation engine.
///
/// Loads JSON translation files from a directory structure, pre-merges
/// fallback translations, and provides lookup with interpolation and
/// pluralization.
pub struct Translator {
    translations: HashMap<String, HashMap<String, String>>,
    fallback: String,
}

impl Translator {
    /// Load translations from `{path}/{locale}/*.json` with the given fallback locale.
    ///
    /// The fallback locale's keys are pre-merged into every other locale so
    /// runtime lookup is a single `HashMap::get`.
    pub fn load(path: impl AsRef<str>, fallback: impl Into<String>) -> Result<Self, LangError> {
        let fallback = fallback.into();
        let translations = load_translations(path.as_ref(), &fallback)?;
        Ok(Self {
            translations,
            fallback: normalize_locale(&fallback),
        })
    }

    /// Look up a translation key with parameter interpolation.
    ///
    /// Returns the translated string with `:param` placeholders replaced.
    /// If the key is not found, returns the key itself (no panic, no Option).
    ///
    /// Resolution order: exact locale → base language (region stripped) →
    /// configured fallback. The base-language step lets a request for
    /// `it-IT` resolve against translations loaded under `lang/it/`.
    pub fn get(&self, locale: &str, key: &str, params: &[(&str, &str)]) -> String {
        let locale = normalize_locale(locale);
        let value = self
            .translations
            .get(&locale)
            .and_then(|m| m.get(key))
            .or_else(|| {
                base_language(&locale)
                    .and_then(|base| self.translations.get(base).and_then(|m| m.get(key)))
            })
            .or_else(|| {
                self.translations
                    .get(&self.fallback)
                    .and_then(|m| m.get(key))
            });

        match value {
            Some(template) => interpolate(template, params),
            None => {
                tracing::warn!(locale = %locale, key, "translation key not found");
                key.to_string()
            }
        }
    }

    /// Look up a pluralized translation key.
    ///
    /// Selects the correct plural form from pipe-separated values, then
    /// applies parameter interpolation. A `:count` parameter is automatically
    /// added with the string representation of `count`.
    ///
    /// Uses the same resolution order as [`get`]: exact → base language →
    /// configured fallback.
    pub fn choice(&self, locale: &str, key: &str, count: i64, params: &[(&str, &str)]) -> String {
        let locale = normalize_locale(locale);
        let value = self
            .translations
            .get(&locale)
            .and_then(|m| m.get(key))
            .or_else(|| {
                base_language(&locale)
                    .and_then(|base| self.translations.get(base).and_then(|m| m.get(key)))
            })
            .or_else(|| {
                self.translations
                    .get(&self.fallback)
                    .and_then(|m| m.get(key))
            });

        match value {
            Some(template) => {
                let form = select_plural_form(template, count);
                let count_str = count.to_string();
                let mut all_params: Vec<(&str, &str)> = params.to_vec();
                all_params.push(("count", &count_str));
                interpolate(&form, &all_params)
            }
            None => {
                tracing::warn!(locale = %locale, key, "translation key not found");
                key.to_string()
            }
        }
    }

    /// Check if a translation key exists for the given locale.
    pub fn has(&self, locale: &str, key: &str) -> bool {
        let locale = normalize_locale(locale);
        self.translations
            .get(&locale)
            .is_some_and(|m| m.contains_key(key))
    }

    /// Return all available locale identifiers.
    pub fn locales(&self) -> Vec<&str> {
        self.translations.keys().map(|s| s.as_str()).collect()
    }
}

/// Strip the region subtag from a normalized locale.
///
/// Returns `Some("it")` for `"it-it"`, `Some("zh")` for `"zh-hans-cn"`,
/// and `None` for bare language tags like `"it"` that have no region.
fn base_language(locale: &str) -> Option<&str> {
    locale.split_once('-').map(|(base, _)| base)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Create a uniquely-named temp directory per test invocation.
    fn unique_dir(label: &str) -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("ferro_lang_{}_{}_{}", label, std::process::id(), n));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    /// Write the standard en + es fixture into the given directory.
    fn write_fixtures(dir: &Path) {
        let en_dir = dir.join("en");
        fs::create_dir_all(&en_dir).unwrap();
        fs::write(
            en_dir.join("messages.json"),
            serde_json::json!({
                "welcome": "Welcome, :name!",
                "items.count": "One item|:count items",
                "cart.summary": "{0} Your cart is empty|{1} :count item in your cart|[2,*] :count items in your cart",
                "only_en": "English only"
            })
            .to_string(),
        )
        .unwrap();

        let es_dir = dir.join("es");
        fs::create_dir_all(&es_dir).unwrap();
        fs::write(
            es_dir.join("messages.json"),
            serde_json::json!({
                "welcome": "Bienvenido, :name!",
                "items.count": "Un elemento|:count elementos"
            })
            .to_string(),
        )
        .unwrap();
    }

    fn cleanup(dir: &PathBuf) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_succeeds() {
        let dir = unique_dir("load");
        write_fixtures(&dir);
        let t = Translator::load(dir.to_str().unwrap(), "en").unwrap();
        assert!(t.locales().contains(&"en"));
        assert!(t.locales().contains(&"es"));
        cleanup(&dir);
    }

    #[test]
    fn get_with_interpolation() {
        let dir = unique_dir("get_interp");
        write_fixtures(&dir);
        let t = Translator::load(dir.to_str().unwrap(), "en").unwrap();
        assert_eq!(
            t.get("en", "welcome", &[("name", "Alice")]),
            "Welcome, Alice!"
        );
        cleanup(&dir);
    }

    #[test]
    fn get_returns_key_when_missing() {
        let dir = unique_dir("get_missing");
        write_fixtures(&dir);
        let t = Translator::load(dir.to_str().unwrap(), "en").unwrap();
        assert_eq!(t.get("en", "nonexistent.key", &[]), "nonexistent.key");
        cleanup(&dir);
    }

    #[test]
    fn choice_returns_plural_form() {
        let dir = unique_dir("choice_plural");
        write_fixtures(&dir);
        let t = Translator::load(dir.to_str().unwrap(), "en").unwrap();
        assert_eq!(t.choice("en", "items.count", 1, &[]), "One item");
        assert_eq!(t.choice("en", "items.count", 5, &[]), "5 items");
        cleanup(&dir);
    }

    #[test]
    fn choice_auto_adds_count_param() {
        let dir = unique_dir("choice_count");
        write_fixtures(&dir);
        let t = Translator::load(dir.to_str().unwrap(), "en").unwrap();
        assert_eq!(t.choice("en", "items.count", 42, &[]), "42 items");
        cleanup(&dir);
    }

    #[test]
    fn choice_explicit_ranges() {
        let dir = unique_dir("choice_ranges");
        write_fixtures(&dir);
        let t = Translator::load(dir.to_str().unwrap(), "en").unwrap();
        assert_eq!(t.choice("en", "cart.summary", 0, &[]), "Your cart is empty");
        assert_eq!(
            t.choice("en", "cart.summary", 1, &[]),
            "1 item in your cart"
        );
        assert_eq!(
            t.choice("en", "cart.summary", 3, &[]),
            "3 items in your cart"
        );
        cleanup(&dir);
    }

    #[test]
    fn has_returns_correct() {
        let dir = unique_dir("has");
        write_fixtures(&dir);
        let t = Translator::load(dir.to_str().unwrap(), "en").unwrap();
        assert!(t.has("en", "welcome"));
        assert!(!t.has("en", "nonexistent"));
        cleanup(&dir);
    }

    #[test]
    fn locales_returns_all() {
        let dir = unique_dir("locales");
        write_fixtures(&dir);
        let t = Translator::load(dir.to_str().unwrap(), "en").unwrap();
        let mut locales = t.locales();
        locales.sort();
        assert_eq!(locales, vec!["en", "es"]);
        cleanup(&dir);
    }

    #[test]
    fn fallback_locale_works() {
        let dir = unique_dir("fallback");
        write_fixtures(&dir);
        let t = Translator::load(dir.to_str().unwrap(), "en").unwrap();
        // "only_en" exists in en but not es — should be pre-merged into es
        assert_eq!(t.get("es", "only_en", &[]), "English only");
        cleanup(&dir);
    }

    #[test]
    fn locale_normalization() {
        let dir = unique_dir("normalization");
        write_fixtures(&dir);

        let en_us_dir = dir.join("en_US");
        fs::create_dir_all(&en_us_dir).unwrap();
        fs::write(
            en_us_dir.join("messages.json"),
            serde_json::json!({
                "greeting": "Hey!"
            })
            .to_string(),
        )
        .unwrap();

        let t = Translator::load(dir.to_str().unwrap(), "en").unwrap();
        assert!(t.has("en-us", "greeting"));
        assert!(t.has("en_US", "greeting"));
        assert!(t.has("EN_US", "greeting"));
        cleanup(&dir);
    }

    #[test]
    fn nested_json_flattened() {
        let dir = unique_dir("nested");
        write_fixtures(&dir);

        let fr_dir = dir.join("fr");
        fs::create_dir_all(&fr_dir).unwrap();
        fs::write(
            fr_dir.join("auth.json"),
            serde_json::json!({
                "auth": {
                    "login": "Connexion",
                    "register": "Inscription"
                }
            })
            .to_string(),
        )
        .unwrap();

        let t = Translator::load(dir.to_str().unwrap(), "en").unwrap();
        assert_eq!(t.get("fr", "auth.login", &[]), "Connexion");
        assert_eq!(t.get("fr", "auth.register", &[]), "Inscription");
        cleanup(&dir);
    }

    #[test]
    fn empty_dir_errors() {
        let dir = unique_dir("empty");
        fs::create_dir_all(&dir).unwrap();
        let result = Translator::load(dir.to_str().unwrap(), "en");
        assert!(result.is_err());
        cleanup(&dir);
    }

    // ── regional locale → base language fallback ──────────────────────
    //
    // Mobile browsers virtually always send region-tagged Accept-Language
    // (e.g. `it-IT`, `en-US`), but translation files are typically loaded
    // under base-language directories (`lang/it/`, `lang/en/`). The
    // translator must try the base language before falling through to
    // the configured fallback locale, otherwise every mobile user sees
    // the fallback locale's strings regardless of device language.

    #[test]
    fn get_regional_locale_falls_back_to_base_language() {
        let dir = unique_dir("regional_base");
        write_fixtures(&dir);
        let t = Translator::load(dir.to_str().unwrap(), "en").unwrap();
        // Request "es-MX" — no es-MX dir exists, but "es" does.
        // Must return Spanish, not English (the configured fallback).
        assert_eq!(
            t.get("es-MX", "welcome", &[("name", "Ana")]),
            "Bienvenido, Ana!"
        );
        cleanup(&dir);
    }

    #[test]
    fn get_regional_locale_prefers_exact_match_over_base() {
        let dir = unique_dir("regional_exact");
        write_fixtures(&dir);
        // Add an exact es-MX dir with a different greeting
        let es_mx = dir.join("es-mx");
        fs::create_dir_all(&es_mx).unwrap();
        fs::write(
            es_mx.join("messages.json"),
            serde_json::json!({"welcome": "¡Qué onda, :name!"}).to_string(),
        )
        .unwrap();
        let t = Translator::load(dir.to_str().unwrap(), "en").unwrap();
        // Exact es-mx match wins over base "es"
        assert_eq!(
            t.get("es-MX", "welcome", &[("name", "Ana")]),
            "¡Qué onda, Ana!"
        );
        cleanup(&dir);
    }

    #[test]
    fn get_regional_locale_uses_global_fallback_when_no_base() {
        let dir = unique_dir("regional_no_base");
        write_fixtures(&dir);
        let t = Translator::load(dir.to_str().unwrap(), "en").unwrap();
        // Request "fr-CA" — no fr-CA, no fr loaded; falls through to "en".
        assert_eq!(
            t.get("fr-CA", "welcome", &[("name", "Léa")]),
            "Welcome, Léa!"
        );
        cleanup(&dir);
    }

    #[test]
    fn choice_regional_locale_falls_back_to_base_language() {
        let dir = unique_dir("regional_choice");
        write_fixtures(&dir);
        let t = Translator::load(dir.to_str().unwrap(), "en").unwrap();
        // es has its own plural form; es-MX request should reach it
        // via base-language fallback, not slip through to English.
        assert_eq!(t.choice("es-MX", "items.count", 5, &[]), "5 elementos");
        cleanup(&dir);
    }

    #[test]
    fn get_underscore_regional_input_falls_back_to_base() {
        let dir = unique_dir("regional_underscore");
        write_fixtures(&dir);
        let t = Translator::load(dir.to_str().unwrap(), "en").unwrap();
        // Accept-Language hyphens are the common form, but the API also
        // accepts underscore form. Both must normalize and fall back.
        assert_eq!(
            t.get("es_MX", "welcome", &[("name", "Ana")]),
            "Bienvenido, Ana!"
        );
        cleanup(&dir);
    }

    #[test]
    fn base_language_helper() {
        assert_eq!(super::base_language("it-it"), Some("it"));
        assert_eq!(super::base_language("zh-hans-cn"), Some("zh"));
        assert_eq!(super::base_language("it"), None);
        assert_eq!(super::base_language(""), None);
    }
}
