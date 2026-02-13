use std::collections::HashMap;

use crate::error::LangError;

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
    /// Load translations from `{path}/{locale}/*.json` with fallback locale.
    pub fn load(
        _path: impl AsRef<str>,
        _fallback: impl Into<String>,
    ) -> Result<Self, LangError> {
        todo!("implemented in Task 2")
    }

    /// Look up a translation key with parameter interpolation.
    pub fn get(&self, _locale: &str, _key: &str, _params: &[(&str, &str)]) -> String {
        todo!("implemented in Task 2")
    }

    /// Look up a pluralized translation key.
    pub fn choice(
        &self,
        _locale: &str,
        _key: &str,
        _count: i64,
        _params: &[(&str, &str)],
    ) -> String {
        todo!("implemented in Task 2")
    }

    /// Check if a key exists for the given locale.
    pub fn has(&self, _locale: &str, _key: &str) -> bool {
        todo!("implemented in Task 2")
    }

    /// Return all available locale names.
    pub fn locales(&self) -> Vec<&str> {
        todo!("implemented in Task 2")
    }
}
