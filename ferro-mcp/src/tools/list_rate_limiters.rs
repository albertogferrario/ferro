//! List rate limiters tool - scan for RateLimiter::define and Throttle usage

use crate::error::Result;
use serde::Serialize;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct RateLimitersInfo {
    pub limiters: Vec<RateLimiterInfo>,
    pub route_usage: Vec<ThrottleUsage>,
}

#[derive(Debug, Serialize)]
pub struct RateLimiterInfo {
    pub name: String,
    pub max_requests: Option<u64>,
    pub window_seconds: Option<u64>,
    pub source_file: String,
    pub line: usize,
}

#[derive(Debug, Serialize)]
pub struct ThrottleUsage {
    pub limiter_name: Option<String>,
    pub inline_description: Option<String>,
    pub route_group: String,
    pub source_file: String,
    pub line: usize,
}

pub fn execute(project_root: &Path) -> Result<RateLimitersInfo> {
    let src_path = project_root.join("src");
    let mut limiters = Vec::new();
    let mut route_usage = Vec::new();

    if src_path.exists() {
        scan_directory(&src_path, &mut limiters, &mut route_usage, project_root);
    }

    Ok(RateLimitersInfo {
        limiters,
        route_usage,
    })
}

fn scan_directory(
    dir: &Path,
    limiters: &mut Vec<RateLimiterInfo>,
    route_usage: &mut Vec<ThrottleUsage>,
    project_root: &Path,
) {
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            let relative_path = entry
                .path()
                .strip_prefix(project_root)
                .unwrap_or(entry.path())
                .to_string_lossy()
                .to_string();

            // Quick check: skip files without relevant patterns
            let has_define = content.contains("RateLimiter::define(");
            let has_throttle = content.contains("Throttle::");

            if has_define {
                extract_limiter_definitions(&content, &relative_path, limiters);
            }
            if has_throttle {
                extract_throttle_usage(&content, &relative_path, route_usage);
            }
        }
    }
}

/// Extract `RateLimiter::define("name", ...)` calls from file content.
fn extract_limiter_definitions(content: &str, path: &str, limiters: &mut Vec<RateLimiterInfo>) {
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Match: RateLimiter::define("name"
        if let Some(define_pos) = trimmed.find("RateLimiter::define(") {
            let after = &trimmed[define_pos + "RateLimiter::define(".len()..];

            // Extract name from first quoted string argument
            if let Some(name) = extract_quoted_string(after) {
                // Try to extract Limit configuration from surrounding context
                let (max_requests, window_seconds) = extract_limit_from_context(content, line_idx);

                limiters.push(RateLimiterInfo {
                    name,
                    max_requests,
                    window_seconds,
                    source_file: path.to_string(),
                    line: line_idx + 1,
                });
            }
        }
    }
}

/// Extract Throttle usage patterns from file content.
///
/// Looks for:
/// - `Throttle::named("name")` - named limiter reference
/// - `Throttle::per_minute(N)` / `Throttle::per_hour(N)` etc. - inline throttles
fn extract_throttle_usage(content: &str, path: &str, route_usage: &mut Vec<ThrottleUsage>) {
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Match: Throttle::named("name")
        if let Some(named_pos) = trimmed.find("Throttle::named(") {
            let after = &trimmed[named_pos + "Throttle::named(".len()..];
            if let Some(name) = extract_quoted_string(after) {
                let route_group = extract_route_group_context(content, line_idx);

                route_usage.push(ThrottleUsage {
                    limiter_name: Some(name),
                    inline_description: None,
                    route_group,
                    source_file: path.to_string(),
                    line: line_idx + 1,
                });
            }
        }

        // Match inline throttles: Throttle::per_minute(N), per_hour(N), per_second(N), per_day(N)
        for method in &["per_minute", "per_hour", "per_second", "per_day"] {
            let pattern = format!("Throttle::{}(", method);
            if let Some(pos) = trimmed.find(&pattern) {
                let after = &trimmed[pos + pattern.len()..];
                if let Some(paren_end) = after.find(')') {
                    let value_str = after[..paren_end].trim();
                    let route_group = extract_route_group_context(content, line_idx);

                    route_usage.push(ThrottleUsage {
                        limiter_name: None,
                        inline_description: Some(format!("{}({})", method, value_str)),
                        route_group,
                        source_file: path.to_string(),
                        line: line_idx + 1,
                    });
                }
            }
        }
    }
}

/// Extract the first double-quoted string from text.
fn extract_quoted_string(text: &str) -> Option<String> {
    let q1 = text.find('"')?;
    let rest = &text[q1 + 1..];
    let q2 = rest.find('"')?;
    let value = &rest[..q2];
    if value.is_empty() {
        return None;
    }
    Some(value.to_string())
}

/// Try to extract Limit configuration (max_requests, window_seconds) from the
/// define() call context. Looks at the same line and a few following lines for
/// `Limit::per_minute(N)`, `Limit::per_hour(N)`, etc.
fn extract_limit_from_context(content: &str, define_line: usize) -> (Option<u64>, Option<u64>) {
    let lines: Vec<&str> = content.lines().collect();
    // Check the define line and a few lines after for Limit patterns
    let end = (define_line + 10).min(lines.len());
    let context = lines[define_line..end].join(" ");

    for (method, seconds) in &[
        ("per_second", 1u64),
        ("per_minute", 60u64),
        ("per_hour", 3600u64),
        ("per_day", 86400u64),
    ] {
        let pattern = format!("Limit::{}(", method);
        if let Some(pos) = context.find(&pattern) {
            let after = &context[pos + pattern.len()..];
            if let Some(paren_end) = after.find(')') {
                let value_str = after[..paren_end].trim();
                if let Ok(max) = value_str.parse::<u64>() {
                    return (Some(max), Some(*seconds));
                }
            }
        }
    }

    (None, None)
}

/// Extract route group context around a Throttle usage.
///
/// Looks backwards from the usage line for `group!("/path"` patterns to
/// determine the route group prefix.
fn extract_route_group_context(content: &str, usage_line: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    // Search backwards for group!("/path" pattern
    let start = usage_line.saturating_sub(20);
    for i in (start..=usage_line).rev() {
        let trimmed = lines[i].trim();
        if let Some(group_pos) = trimmed.find("group!(") {
            let after = &trimmed[group_pos + "group!(".len()..];
            if let Some(name) = extract_quoted_string(after) {
                return name;
            }
        }
    }
    "(unknown)".to_string()
}
