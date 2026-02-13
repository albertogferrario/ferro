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
    pub fn get(&self, locale: &str, key: &str, params: &[(&str, &str)]) -> String {
        let locale = normalize_locale(locale);
        let value = self
            .translations
            .get(&locale)
            .and_then(|m| m.get(key))
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
    pub fn choice(
        &self,
        locale: &str,
        key: &str,
        count: i64,
        params: &[(&str, &str)],
    ) -> String {
        let locale = normalize_locale(locale);
        let value = self
            .translations
            .get(&locale)
            .and_then(|m| m.get(key))
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
            .map_or(false, |m| m.contains_key(key))
    }

    /// Return all available locale identifiers.
    pub fn locales(&self) -> Vec<&str> {
        self.translations.keys().map(|s| s.as_str()).collect()
    }
}
