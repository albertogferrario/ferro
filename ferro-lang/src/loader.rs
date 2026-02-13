use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::error::LangError;

/// Normalize a locale identifier to lowercase with hyphens.
///
/// Converts `en_US` to `en-us`, `pt-BR` to `pt-br`, etc.
pub fn normalize_locale(locale: &str) -> String {
    locale.to_lowercase().replace('_', "-")
}

/// Load all translation files from a directory.
///
/// Expects the structure: `{path}/{locale}/*.json`
///
/// Each subdirectory name is treated as a locale identifier. All JSON files
/// within a locale directory are merged into a single flat map using
/// dot-notation keys.
///
/// After loading, fallback translations are pre-merged into each locale
/// so runtime lookup requires only a single `HashMap::get`.
pub fn load_translations(
    path: &str,
    fallback: &str,
) -> Result<HashMap<String, HashMap<String, String>>, LangError> {
    let base = Path::new(path);
    let mut translations: HashMap<String, HashMap<String, String>> = HashMap::new();

    let entries = fs::read_dir(base)?;

    for entry in entries {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if !file_type.is_dir() {
            continue;
        }

        let dir_name = entry.file_name();
        let locale_raw = dir_name.to_string_lossy().to_string();
        let locale = normalize_locale(&locale_raw);

        let locale_map = load_locale_dir(&entry.path())?;
        if !locale_map.is_empty() {
            translations.insert(locale, locale_map);
        }
    }

    if translations.is_empty() {
        return Err(LangError::NoTranslationsLoaded);
    }

    let fallback_normalized = normalize_locale(fallback);

    // Pre-merge fallback: insert missing keys from fallback into each locale.
    if let Some(fallback_map) = translations.get(&fallback_normalized).cloned() {
        for (locale, locale_map) in translations.iter_mut() {
            if *locale == fallback_normalized {
                continue;
            }
            for (key, value) in &fallback_map {
                locale_map.entry(key.clone()).or_insert_with(|| value.clone());
            }
        }
    }

    let total_keys: usize = translations.values().map(|m| m.len()).sum();
    tracing::info!(
        locales = translations.len(),
        total_keys,
        "loaded translations"
    );

    Ok(translations)
}

/// Load all JSON files within a single locale directory.
fn load_locale_dir(dir: &Path) -> Result<HashMap<String, String>, LangError> {
    let mut map = HashMap::new();

    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let parsed: HashMap<String, Value> = serde_json::from_str(&content)?;

        flatten_json(&parsed, "", &mut map);
    }

    Ok(map)
}

/// Flatten a nested JSON object into dot-notation keys.
///
/// Only string leaf values are stored. Non-string leaves are skipped
/// with a warning.
fn flatten_json(obj: &HashMap<String, Value>, prefix: &str, out: &mut HashMap<String, String>) {
    for (key, value) in obj {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };

        match value {
            Value::String(s) => {
                out.insert(full_key, s.clone());
            }
            Value::Object(nested) => {
                let nested_map: HashMap<String, Value> =
                    nested.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                flatten_json(&nested_map, &full_key, out);
            }
            _ => {
                tracing::warn!(
                    key = %full_key,
                    "skipping non-string translation value"
                );
            }
        }
    }
}
