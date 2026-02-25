//! List lang files tool - scan for localization configuration and translation coverage

use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct LangFilesInfo {
    pub default_locale: String,
    pub fallback_locale: String,
    pub lang_path: String,
    pub total_keys: usize,
    pub locales: Vec<LocaleInfo>,
    pub coverage: CoverageReport,
}

#[derive(Debug, Serialize)]
pub struct LocaleInfo {
    pub locale: String,
    pub files: Vec<String>,
    pub key_count: usize,
}

#[derive(Debug, Serialize)]
pub struct CoverageReport {
    /// Keys present in fallback but missing in each non-fallback locale
    pub missing_keys: HashMap<String, Vec<String>>,
}

pub fn execute(project_root: &Path, locale_filter: Option<&str>) -> Result<LangFilesInfo, String> {
    // Read .env for locale configuration
    let (default_locale, fallback_locale, lang_path) = read_env_config(project_root);

    let lang_dir = project_root.join(&lang_path);
    if !lang_dir.exists() {
        return Err(format!(
            "Language directory '{}' not found. Run `ferro make:lang <locale>` to create one.",
            lang_path
        ));
    }

    // Scan locale directories
    let mut locale_keys: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut locale_files: HashMap<String, Vec<String>> = HashMap::new();

    let entries =
        fs::read_dir(&lang_dir).map_err(|e| format!("Failed to read lang directory: {}", e))?;

    for entry in entries.flatten() {
        if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            continue;
        }

        let dir_name = entry.file_name().to_string_lossy().to_string();
        let locale = normalize_locale(&dir_name);

        // If filter provided, skip non-matching locales
        if let Some(filter) = locale_filter {
            if normalize_locale(filter) != locale {
                continue;
            }
        }

        let (keys, files) = scan_locale_dir(&entry.path())?;
        locale_keys.insert(locale.clone(), keys);
        locale_files.insert(locale, files);
    }

    if locale_keys.is_empty() {
        return Err(if let Some(filter) = &locale_filter {
            format!("No locale matching '{}' found", filter)
        } else {
            "No locale directories found in lang/".to_string()
        });
    }

    // Build coverage report: find keys in fallback missing from other locales
    let fallback_normalized = normalize_locale(&fallback_locale);
    let mut missing_keys: HashMap<String, Vec<String>> = HashMap::new();

    if let Some(fallback_map) = locale_keys.get(&fallback_normalized) {
        for (locale, locale_map) in &locale_keys {
            if *locale == fallback_normalized {
                continue;
            }
            let mut missing = Vec::new();
            for key in fallback_map.keys() {
                if !locale_map.contains_key(key) {
                    missing.push(key.clone());
                }
            }
            missing.sort();
            if !missing.is_empty() {
                missing_keys.insert(locale.clone(), missing);
            }
        }
    }

    // Calculate total unique keys across all locales
    let total_keys: usize = locale_keys.values().map(|m| m.len()).sum();

    // Build locale info list sorted alphabetically
    let mut locales: Vec<LocaleInfo> = locale_keys
        .keys()
        .map(|locale| {
            let key_count = locale_keys.get(locale).map(|m| m.len()).unwrap_or(0);
            let files = locale_files.get(locale).cloned().unwrap_or_default();
            LocaleInfo {
                locale: locale.clone(),
                files,
                key_count,
            }
        })
        .collect();
    locales.sort_by(|a, b| a.locale.cmp(&b.locale));

    Ok(LangFilesInfo {
        default_locale,
        fallback_locale,
        lang_path,
        total_keys,
        locales,
        coverage: CoverageReport { missing_keys },
    })
}

/// Read locale configuration from .env file.
fn read_env_config(project_root: &Path) -> (String, String, String) {
    let env_path = project_root.join(".env");
    let mut default_locale = "en".to_string();
    let mut fallback_locale = "en".to_string();
    let mut lang_path = "lang".to_string();

    if let Ok(content) = fs::read_to_string(&env_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let value = value.trim().trim_matches('"');
                match key.trim() {
                    "APP_LOCALE" => default_locale = value.to_string(),
                    "APP_FALLBACK_LOCALE" => fallback_locale = value.to_string(),
                    "LANG_PATH" => lang_path = value.to_string(),
                    _ => {}
                }
            }
        }
    }

    (default_locale, fallback_locale, lang_path)
}

/// Scan a single locale directory, returning flattened keys and file names.
fn scan_locale_dir(dir: &Path) -> Result<(HashMap<String, String>, Vec<String>), String> {
    let mut keys = HashMap::new();
    let mut files = Vec::new();

    let entries =
        fs::read_dir(dir).map_err(|e| format!("Failed to read locale directory: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let file_name = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        files.push(file_name);

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        let parsed: HashMap<String, serde_json::Value> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;

        flatten_json(&parsed, "", &mut keys);
    }

    files.sort();
    Ok((keys, files))
}

/// Flatten nested JSON object into dot-notation keys.
///
/// Mirrors ferro-lang's loader behavior: `{"auth": {"login": "..."}}` becomes
/// key `"auth.login"`.
fn flatten_json(
    obj: &HashMap<String, serde_json::Value>,
    prefix: &str,
    out: &mut HashMap<String, String>,
) {
    for (key, value) in obj {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };

        match value {
            serde_json::Value::String(s) => {
                out.insert(full_key, s.clone());
            }
            serde_json::Value::Object(nested) => {
                let nested_map: HashMap<String, serde_json::Value> =
                    nested.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                flatten_json(&nested_map, &full_key, out);
            }
            _ => {
                // Skip non-string, non-object values (same as ferro-lang)
            }
        }
    }
}

/// Normalize locale to lowercase with hyphens (same as ferro-lang).
fn normalize_locale(locale: &str) -> String {
    locale.to_lowercase().replace('_', "-")
}
