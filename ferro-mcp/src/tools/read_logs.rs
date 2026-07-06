//! Read logs tool - get recent log entries

use crate::error::{McpError, Result};
use serde::Serialize;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct LogsInfo {
    pub entries: Vec<LogEntry>,
}

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub timestamp: Option<String>,
    pub level: String,
    pub message: String,
}

pub fn execute(project_root: &Path, lines: usize, level_filter: Option<&str>) -> Result<LogsInfo> {
    // Common log file locations
    let log_paths = [
        project_root.join("storage/logs/app.log"),
        project_root.join("logs/app.log"),
        project_root.join("app.log"),
        project_root.join("storage/logs/laravel.log"),
    ];

    // Find the first existing log file
    let log_file = log_paths
        .iter()
        .find(|p| p.exists())
        .ok_or_else(|| McpError::FileNotFound("No log file found".to_string()))?;

    let file = fs::File::open(log_file).map_err(McpError::IoError)?;
    let reader = BufReader::new(file);

    // Read all lines and keep only the last N
    let all_lines: Vec<String> = reader.lines().map_while(|l| l.ok()).collect();

    let start_idx = if all_lines.len() > lines {
        all_lines.len() - lines
    } else {
        0
    };

    let mut entries: Vec<LogEntry> = all_lines[start_idx..]
        .iter()
        .filter_map(|line| parse_log_line(line))
        .collect();

    // Apply level filter if specified
    if let Some(level) = level_filter {
        let level_upper = level.to_uppercase();
        entries.retain(|e| e.level.to_uppercase() == level_upper);
    }

    Ok(LogsInfo { entries })
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    if line.trim().is_empty() {
        return None;
    }

    // Try to parse structured log format: [timestamp] LEVEL: message
    // or: timestamp LEVEL message

    let line = line.trim();

    // Pattern 1: [2024-01-01 12:00:00] LEVEL: message
    if line.starts_with('[') {
        if let Some(bracket_end) = line.find(']') {
            let timestamp = &line[1..bracket_end];
            let rest = line[bracket_end + 1..].trim();

            // Extract level
            let (level, message) = if let Some(colon_idx) = rest.find(':') {
                let level_part = rest[..colon_idx].trim().to_uppercase();
                let message_part = rest[colon_idx + 1..].trim();
                (level_part, message_part.to_string())
            } else {
                let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    (parts[0].to_uppercase(), parts[1].to_string())
                } else {
                    ("INFO".to_string(), rest.to_string())
                }
            };

            return Some(LogEntry {
                timestamp: Some(timestamp.to_string()),
                level,
                message,
            });
        }
    }

    // Pattern 2: Simple format - try to detect log level keywords
    let level = if line.to_uppercase().contains("ERROR") {
        "ERROR"
    } else if line.to_uppercase().contains("WARN") {
        "WARN"
    } else if line.to_uppercase().contains("DEBUG") {
        "DEBUG"
    } else if line.to_uppercase().contains("TRACE") {
        "TRACE"
    } else {
        "INFO"
    };

    Some(LogEntry {
        timestamp: None,
        level: level.to_string(),
        message: line.to_string(),
    })
}
