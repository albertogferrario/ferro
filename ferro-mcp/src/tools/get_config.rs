//! Get config tool - read configuration values

use crate::error::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct ConfigInfo {
    pub env: HashMap<String, String>,
    pub config: HashMap<String, toml::Value>,
}

pub fn execute(project_root: &Path, key: Option<&str>) -> Result<ConfigInfo> {
    let mut env_vars = HashMap::new();
    let mut config_values = HashMap::new();

    // Read .env file
    let env_file = project_root.join(".env");
    if env_file.exists() {
        if let Ok(content) = fs::read_to_string(&env_file) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    let k = k.trim();
                    let v = v.trim().trim_matches('"').trim_matches('\'');

                    // If key filter is specified, only include matching keys
                    if let Some(filter) = key {
                        if k.to_lowercase().contains(&filter.to_lowercase()) {
                            env_vars.insert(k.to_string(), mask_sensitive(k, v));
                        }
                    } else {
                        env_vars.insert(k.to_string(), mask_sensitive(k, v));
                    }
                }
            }
        }
    }

    // Read config files from config/ directory
    let config_dir = project_root.join("config");
    if config_dir.exists() {
        for entry in fs::read_dir(&config_dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(parsed) = content.parse::<toml::Table>() {
                        let config_name = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();

                        // If key filter is specified, only include matching config files
                        if let Some(filter) = key {
                            if config_name.to_lowercase().contains(&filter.to_lowercase()) {
                                config_values.insert(config_name, toml::Value::Table(parsed));
                            }
                        } else {
                            config_values.insert(config_name, toml::Value::Table(parsed));
                        }
                    }
                }
            }
        }
    }

    // Also check for Ferro.toml
    let ferro_toml = project_root.join("Ferro.toml");
    if ferro_toml.exists() {
        if let Ok(content) = fs::read_to_string(&ferro_toml) {
            if let Ok(parsed) = content.parse::<toml::Table>() {
                if key.is_none()
                    || key
                        .map(|k| "ferro".contains(&k.to_lowercase()))
                        .unwrap_or(false)
                {
                    config_values.insert("ferro".to_string(), toml::Value::Table(parsed));
                }
            }
        }
    }

    Ok(ConfigInfo {
        env: env_vars,
        config: config_values,
    })
}

fn mask_sensitive(key: &str, value: &str) -> String {
    let key_lower = key.to_lowercase();
    let sensitive_keywords = [
        "password",
        "secret",
        "key",
        "token",
        "api_key",
        "apikey",
        "private",
        "credential",
    ];

    for keyword in sensitive_keywords {
        if key_lower.contains(keyword) {
            if value.len() > 4 {
                return format!("{}****", &value[..4]);
            } else {
                return "****".to_string();
            }
        }
    }

    value.to_string()
}
