//! Browser logs tool - Read frontend/browser error logs

use crate::error::{McpError, Result};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct BrowserLogEntry {
    pub timestamp: Option<String>,
    pub level: String,
    pub message: String,
    pub source: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub stack: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BrowserLogsResult {
    pub entries: Vec<BrowserLogEntry>,
    pub total_count: usize,
    pub file_path: String,
}

/// Read browser/frontend logs from storage/logs/browser.log
///
/// These logs are typically written by the frontend error handler
/// when JavaScript errors occur in the browser.
pub fn execute(
    project_root: &Path,
    lines: usize,
    level: Option<&str>,
) -> Result<BrowserLogsResult> {
    // Check multiple possible locations
    let possible_paths = [
        project_root.join("storage/logs/browser.log"),
        project_root.join("logs/browser.log"),
        project_root.join("storage/browser.log"),
    ];

    let log_path = possible_paths
        .iter()
        .find(|p| p.exists())
        .ok_or_else(|| McpError::FileNotFound("browser.log".to_string()))?;

    let content = std::fs::read_to_string(log_path)
        .map_err(|e| McpError::FileNotFound(format!("Failed to read browser.log: {e}")))?;

    let mut entries: Vec<BrowserLogEntry> = Vec::new();

    // Parse JSON lines format (common for browser logs)
    for line in content.lines().rev() {
        if line.trim().is_empty() {
            continue;
        }

        // Try to parse as JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            let entry = BrowserLogEntry {
                timestamp: json
                    .get("timestamp")
                    .or_else(|| json.get("time"))
                    .or_else(|| json.get("ts"))
                    .and_then(|v| v.as_str())
                    .map(String::from),
                level: json
                    .get("level")
                    .or_else(|| json.get("severity"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("error")
                    .to_uppercase(),
                message: json
                    .get("message")
                    .or_else(|| json.get("msg"))
                    .or_else(|| json.get("error"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                source: json
                    .get("source")
                    .or_else(|| json.get("file"))
                    .or_else(|| json.get("url"))
                    .and_then(|v| v.as_str())
                    .map(String::from),
                line: json
                    .get("line")
                    .or_else(|| json.get("lineno"))
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32),
                column: json
                    .get("column")
                    .or_else(|| json.get("colno"))
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32),
                stack: json
                    .get("stack")
                    .or_else(|| json.get("stacktrace"))
                    .and_then(|v| v.as_str())
                    .map(String::from),
            };

            // Filter by level if specified
            if let Some(filter_level) = level {
                if !entry.level.eq_ignore_ascii_case(filter_level) {
                    continue;
                }
            }

            entries.push(entry);
        } else {
            // Parse as plain text log
            let entry = parse_plain_log_line(line);
            if let Some(filter_level) = level {
                if !entry.level.eq_ignore_ascii_case(filter_level) {
                    continue;
                }
            }
            entries.push(entry);
        }

        if entries.len() >= lines {
            break;
        }
    }

    let total_count = entries.len();

    Ok(BrowserLogsResult {
        entries,
        total_count,
        file_path: log_path.display().to_string(),
    })
}

fn parse_plain_log_line(line: &str) -> BrowserLogEntry {
    // Try to parse common formats like:
    // [2024-01-01 12:00:00] ERROR: message
    // 2024-01-01T12:00:00Z ERROR message

    let line = line.trim();
    let mut timestamp = None;
    let mut level = "ERROR".to_string();
    let mut message = line.to_string();

    // Extract timestamp in brackets
    if line.starts_with('[') {
        if let Some(end) = line.find(']') {
            timestamp = Some(line[1..end].to_string());
            let rest = line[end + 1..].trim();

            // Check for level
            for lvl in ["ERROR", "WARN", "INFO", "DEBUG"] {
                if let Some(stripped) = rest.strip_prefix(lvl) {
                    level = lvl.to_string();
                    message = stripped.trim_start_matches(':').trim().to_string();
                    break;
                } else {
                    message = rest.to_string();
                }
            }
        }
    }

    BrowserLogEntry {
        timestamp,
        level,
        message,
        source: None,
        line: None,
        column: None,
        stack: None,
    }
}
