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
                locale_map
                    .entry(key.clone())
                    .or_insert_with(|| value.clone());
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
            format!("{prefix}.{key}")
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // ── normalize_locale ──────────────────────────────────────────────

    #[test]
    fn normalize_locale_lowercase() {
        assert_eq!(normalize_locale("en"), "en");
    }

    #[test]
    fn normalize_locale_underscore() {
        assert_eq!(normalize_locale("en_US"), "en-us");
    }

    #[test]
    fn normalize_locale_uppercase() {
        assert_eq!(normalize_locale("pt-BR"), "pt-br");
    }

    #[test]
    fn normalize_locale_mixed() {
        assert_eq!(normalize_locale("zh_Hans_CN"), "zh-hans-cn");
    }

    // ── flatten_json (tested via load_translations) ───────────────────

    #[test]
    fn flatten_simple_object() {
        let dir = tempdir().unwrap();
        let en = dir.path().join("en");
        fs::create_dir_all(&en).unwrap();
        fs::write(
            en.join("messages.json"),
            serde_json::json!({"key": "val"}).to_string(),
        )
        .unwrap();

        let t = load_translations(dir.path().to_str().unwrap(), "en").unwrap();
        assert_eq!(t["en"]["key"], "val");
    }

    #[test]
    fn flatten_nested_object() {
        let dir = tempdir().unwrap();
        let en = dir.path().join("en");
        fs::create_dir_all(&en).unwrap();
        fs::write(
            en.join("messages.json"),
            serde_json::json!({"a": {"b": "c"}}).to_string(),
        )
        .unwrap();

        let t = load_translations(dir.path().to_str().unwrap(), "en").unwrap();
        assert_eq!(t["en"]["a.b"], "c");
    }

    #[test]
    fn flatten_deeply_nested() {
        let dir = tempdir().unwrap();
        let en = dir.path().join("en");
        fs::create_dir_all(&en).unwrap();
        fs::write(
            en.join("messages.json"),
            serde_json::json!({"a": {"b": {"c": "deep"}}}).to_string(),
        )
        .unwrap();

        let t = load_translations(dir.path().to_str().unwrap(), "en").unwrap();
        assert_eq!(t["en"]["a.b.c"], "deep");
    }

    #[test]
    fn flatten_skips_non_string_leaves() {
        let dir = tempdir().unwrap();
        let en = dir.path().join("en");
        fs::create_dir_all(&en).unwrap();
        fs::write(
            en.join("messages.json"),
            serde_json::json!({
                "valid": "hello",
                "number": 42,
                "boolean": true
            })
            .to_string(),
        )
        .unwrap();

        let t = load_translations(dir.path().to_str().unwrap(), "en").unwrap();
        assert_eq!(t["en"]["valid"], "hello");
        assert!(!t["en"].contains_key("number"));
        assert!(!t["en"].contains_key("boolean"));
    }

    // ── load_locale_dir (tested via load_translations) ────────────────

    #[test]
    fn load_locale_dir_single_file() {
        let dir = tempdir().unwrap();
        let en = dir.path().join("en");
        fs::create_dir_all(&en).unwrap();
        fs::write(
            en.join("auth.json"),
            serde_json::json!({"login": "Login", "register": "Register"}).to_string(),
        )
        .unwrap();

        let t = load_translations(dir.path().to_str().unwrap(), "en").unwrap();
        assert_eq!(t["en"]["login"], "Login");
        assert_eq!(t["en"]["register"], "Register");
    }

    #[test]
    fn load_locale_dir_multiple_files() {
        let dir = tempdir().unwrap();
        let en = dir.path().join("en");
        fs::create_dir_all(&en).unwrap();
        fs::write(
            en.join("auth.json"),
            serde_json::json!({"login": "Login"}).to_string(),
        )
        .unwrap();
        fs::write(
            en.join("messages.json"),
            serde_json::json!({"welcome": "Welcome"}).to_string(),
        )
        .unwrap();

        let t = load_translations(dir.path().to_str().unwrap(), "en").unwrap();
        assert_eq!(t["en"]["login"], "Login");
        assert_eq!(t["en"]["welcome"], "Welcome");
    }

    #[test]
    fn load_locale_dir_ignores_non_json() {
        let dir = tempdir().unwrap();
        let en = dir.path().join("en");
        fs::create_dir_all(&en).unwrap();
        fs::write(
            en.join("messages.json"),
            serde_json::json!({"hello": "Hello"}).to_string(),
        )
        .unwrap();
        fs::write(en.join("notes.txt"), "should be ignored").unwrap();

        let t = load_translations(dir.path().to_str().unwrap(), "en").unwrap();
        assert_eq!(t["en"].len(), 1);
        assert_eq!(t["en"]["hello"], "Hello");
    }

    // ── load_translations ─────────────────────────────────────────────

    #[test]
    fn load_translations_single_locale() {
        let dir = tempdir().unwrap();
        let en = dir.path().join("en");
        fs::create_dir_all(&en).unwrap();
        fs::write(
            en.join("messages.json"),
            serde_json::json!({"greeting": "Hi"}).to_string(),
        )
        .unwrap();

        let t = load_translations(dir.path().to_str().unwrap(), "en").unwrap();
        assert_eq!(t.len(), 1);
        assert!(t.contains_key("en"));
    }

    #[test]
    fn load_translations_fallback_premerge() {
        let dir = tempdir().unwrap();

        let en = dir.path().join("en");
        fs::create_dir_all(&en).unwrap();
        fs::write(
            en.join("messages.json"),
            serde_json::json!({"greeting": "Hello", "farewell": "Goodbye"}).to_string(),
        )
        .unwrap();

        let es = dir.path().join("es");
        fs::create_dir_all(&es).unwrap();
        fs::write(
            es.join("messages.json"),
            serde_json::json!({"greeting": "Hola"}).to_string(),
        )
        .unwrap();

        let t = load_translations(dir.path().to_str().unwrap(), "en").unwrap();

        // es has its own "greeting"
        assert_eq!(t["es"]["greeting"], "Hola");
        // es got "farewell" from fallback pre-merge
        assert_eq!(t["es"]["farewell"], "Goodbye");
        // en still has both
        assert_eq!(t["en"]["greeting"], "Hello");
        assert_eq!(t["en"]["farewell"], "Goodbye");
    }

    #[test]
    fn load_translations_empty_dir_errors() {
        let dir = tempdir().unwrap();
        // No locale subdirectories
        let result = load_translations(dir.path().to_str().unwrap(), "en");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, LangError::NoTranslationsLoaded),
            "expected NoTranslationsLoaded, got: {err}"
        );
    }

    #[test]
    fn load_translations_normalizes_locale_dirs() {
        let dir = tempdir().unwrap();
        let en_us = dir.path().join("en_US");
        fs::create_dir_all(&en_us).unwrap();
        fs::write(
            en_us.join("messages.json"),
            serde_json::json!({"color": "Color"}).to_string(),
        )
        .unwrap();

        let t = load_translations(dir.path().to_str().unwrap(), "en_US").unwrap();
        // Directory "en_US" should be normalized to "en-us" key
        assert!(t.contains_key("en-us"));
        assert_eq!(t["en-us"]["color"], "Color");
    }
}
