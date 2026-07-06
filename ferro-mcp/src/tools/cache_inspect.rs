//! Cache inspection tool - view cache keys, values, and TTL

use crate::error::{McpError, Result};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct CacheInfo {
    pub driver: String,
    pub status: String,
    pub entries: Vec<CacheEntry>,
    pub stats: CacheStats,
}

#[derive(Debug, Serialize)]
pub struct CacheEntry {
    pub key: String,
    pub value_preview: String,
    pub ttl: Option<i64>,
    pub size_bytes: usize,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: usize,
    pub keys_by_prefix: HashMap<String, usize>,
}

pub fn execute(project_root: &Path, key_pattern: Option<&str>) -> Result<CacheInfo> {
    // Load config to determine cache driver
    dotenvy::from_path(project_root.join(".env")).ok();

    let cache_driver = std::env::var("CACHE_DRIVER").unwrap_or_else(|_| "file".to_string());

    match cache_driver.as_str() {
        "file" => inspect_file_cache(project_root, key_pattern),
        "redis" => inspect_redis_cache(key_pattern),
        "memory" => Ok(CacheInfo {
            driver: "memory".to_string(),
            status: "in-memory cache cannot be inspected from MCP".to_string(),
            entries: vec![],
            stats: CacheStats {
                total_entries: 0,
                total_size_bytes: 0,
                keys_by_prefix: HashMap::new(),
            },
        }),
        _ => Err(McpError::ConfigError(format!(
            "Unknown cache driver: {cache_driver}"
        ))),
    }
}

fn inspect_file_cache(project_root: &Path, key_pattern: Option<&str>) -> Result<CacheInfo> {
    let cache_dir = project_root.join("storage/cache");

    if !cache_dir.exists() {
        return Ok(CacheInfo {
            driver: "file".to_string(),
            status: "cache directory does not exist".to_string(),
            entries: vec![],
            stats: CacheStats {
                total_entries: 0,
                total_size_bytes: 0,
                keys_by_prefix: HashMap::new(),
            },
        });
    }

    let mut entries = Vec::new();
    let mut total_size: usize = 0;
    let mut keys_by_prefix: HashMap<String, usize> = HashMap::new();

    // Walk the cache directory
    if let Ok(read_dir) = fs::read_dir(&cache_dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    // Skip if pattern doesn't match
                    if let Some(pattern) = key_pattern {
                        if !filename.contains(pattern) {
                            continue;
                        }
                    }

                    // Read file contents
                    if let Ok(content) = fs::read_to_string(&path) {
                        let size = content.len();
                        total_size += size;

                        // Try to parse cache entry
                        let (value_preview, ttl, tags) = parse_cache_content(&content);

                        // Track by prefix
                        let prefix = filename.split(':').next().unwrap_or("default").to_string();
                        *keys_by_prefix.entry(prefix).or_insert(0) += 1;

                        entries.push(CacheEntry {
                            key: filename.to_string(),
                            value_preview,
                            ttl,
                            size_bytes: size,
                            tags,
                        });
                    }
                }
            }
        }
    }

    // Limit entries to 100 for performance
    entries.truncate(100);

    Ok(CacheInfo {
        driver: "file".to_string(),
        status: "ok".to_string(),
        stats: CacheStats {
            total_entries: entries.len(),
            total_size_bytes: total_size,
            keys_by_prefix,
        },
        entries,
    })
}

fn inspect_redis_cache(key_pattern: Option<&str>) -> Result<CacheInfo> {
    // Get Redis URL from environment
    let redis_url = std::env::var("REDIS_URL")
        .or_else(|_| std::env::var("CACHE_REDIS_URL"))
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Try to connect and inspect
    match redis::Client::open(redis_url.as_str()) {
        Ok(client) => {
            match client.get_connection() {
                Ok(mut conn) => {
                    let pattern = key_pattern.unwrap_or("*");

                    // Get keys matching pattern
                    let keys: Vec<String> = redis::cmd("KEYS")
                        .arg(pattern)
                        .query(&mut conn)
                        .unwrap_or_default();

                    let mut entries = Vec::new();
                    let mut total_size: usize = 0;
                    let mut keys_by_prefix: HashMap<String, usize> = HashMap::new();

                    for key in keys.iter().take(100) {
                        // Get value
                        let value: Option<String> =
                            redis::cmd("GET").arg(key).query(&mut conn).ok();

                        // Get TTL
                        let ttl: Option<i64> = redis::cmd("TTL").arg(key).query(&mut conn).ok();

                        let size = value.as_ref().map(|v| v.len()).unwrap_or(0);
                        total_size += size;

                        // Track by prefix
                        let prefix = key.split(':').next().unwrap_or("default").to_string();
                        *keys_by_prefix.entry(prefix).or_insert(0) += 1;

                        let value_preview = value
                            .as_ref()
                            .map(|v| truncate_value(v, 200))
                            .unwrap_or_else(|| "(nil)".to_string());

                        entries.push(CacheEntry {
                            key: key.clone(),
                            value_preview,
                            ttl: ttl.filter(|&t| t >= 0),
                            size_bytes: size,
                            tags: vec![], // Redis tags would need separate lookup
                        });
                    }

                    Ok(CacheInfo {
                        driver: "redis".to_string(),
                        status: "connected".to_string(),
                        stats: CacheStats {
                            total_entries: keys.len(),
                            total_size_bytes: total_size,
                            keys_by_prefix,
                        },
                        entries,
                    })
                }
                Err(e) => Ok(CacheInfo {
                    driver: "redis".to_string(),
                    status: format!("connection failed: {e}"),
                    entries: vec![],
                    stats: CacheStats {
                        total_entries: 0,
                        total_size_bytes: 0,
                        keys_by_prefix: HashMap::new(),
                    },
                }),
            }
        }
        Err(e) => Ok(CacheInfo {
            driver: "redis".to_string(),
            status: format!("client creation failed: {e}"),
            entries: vec![],
            stats: CacheStats {
                total_entries: 0,
                total_size_bytes: 0,
                keys_by_prefix: HashMap::new(),
            },
        }),
    }
}

fn parse_cache_content(content: &str) -> (String, Option<i64>, Vec<String>) {
    // Try to parse as JSON (common cache format)
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(content) {
        let ttl = parsed
            .get("expires_at")
            .and_then(|v| v.as_i64())
            .map(|exp| exp - chrono::Utc::now().timestamp());

        let tags = parsed
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let value = parsed
            .get("value")
            .map(|v| truncate_value(&v.to_string(), 200))
            .unwrap_or_else(|| truncate_value(content, 200));

        (value, ttl, tags)
    } else {
        (truncate_value(content, 200), None, vec![])
    }
}

fn truncate_value(value: &str, max_len: usize) -> String {
    if value.len() > max_len {
        format!("{}...", &value[..max_len])
    } else {
        value.to_string()
    }
}
