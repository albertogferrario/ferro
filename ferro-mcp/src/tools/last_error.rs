//! Last error tool - get the most recent error from logs with route correlation

use crate::error::{McpError, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Error categories for classification
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    /// Validation errors (field validation, format issues)
    Validation,
    /// Database errors (connection, query, migration)
    Database,
    /// Not found errors (404, missing resources)
    NotFound,
    /// Permission errors (401, 403, auth issues)
    Permission,
    /// Internal server errors (500, unhandled)
    Internal,
    /// Panic/fatal errors (crashes, unwrap failures)
    Panic,
}

impl ErrorCategory {
    /// Get a human-readable description of the category
    pub fn description(&self) -> &'static str {
        match self {
            Self::Validation => "Input validation failed",
            Self::Database => "Database operation failed",
            Self::NotFound => "Resource not found",
            Self::Permission => "Permission denied or authentication required",
            Self::Internal => "Internal server error",
            Self::Panic => "Application panic or fatal error",
        }
    }
}

/// Route context extracted from error
#[derive(Debug, Serialize)]
pub struct RouteContext {
    /// Handler function name if extracted
    pub handler: Option<String>,
    /// Route path if extracted (e.g., "/users/123")
    pub path: Option<String>,
    /// HTTP method if extracted
    pub method: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LastErrorInfo {
    pub found: bool,
    pub error: Option<ErrorDetails>,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetails {
    pub timestamp: Option<String>,
    pub level: String,
    pub message: String,
    pub stacktrace: Option<String>,
    /// Error category classification
    pub category: ErrorCategory,
    /// Route context if extractable from error
    pub route_context: Option<RouteContext>,
    /// Related routes that might be affected by similar errors
    pub related_routes: Vec<String>,
}

// Cached regex patterns for performance
static ROUTE_PATH_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?:at |path[=:]\s*|route[=:]\s*)["']?(/[^\s"']+)"#).unwrap());
static HANDLER_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:handler[=:]\s*|at\s+)(\w+(?:::\w+)+)").unwrap());
static METHOD_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS)\b").unwrap());

pub fn execute(project_root: &Path) -> Result<LastErrorInfo> {
    // Common log file locations
    let log_paths = [
        project_root.join("storage/logs/app.log"),
        project_root.join("logs/app.log"),
        project_root.join("app.log"),
        project_root.join("storage/logs/laravel.log"),
    ];

    // Find the first existing log file
    let log_file = log_paths.iter().find(|p| p.exists());

    let Some(log_file) = log_file else {
        return Ok(LastErrorInfo {
            found: false,
            error: None,
        });
    };

    let file = fs::File::open(log_file).map_err(McpError::IoError)?;
    let reader = BufReader::new(file);

    // Read all lines and find the last error
    let all_lines: Vec<String> = reader.lines().map_while(|l| l.ok()).collect();

    // Search from end to find last error
    let mut last_error_idx: Option<usize> = None;
    let mut error_level = "ERROR";
    for (idx, line) in all_lines.iter().enumerate().rev() {
        let line_upper = line.to_uppercase();
        if line_upper.contains("PANIC") || line_upper.contains("FATAL") {
            last_error_idx = Some(idx);
            error_level = "PANIC";
            break;
        }
        if line_upper.contains("ERROR") {
            last_error_idx = Some(idx);
            break;
        }
    }

    let Some(error_idx) = last_error_idx else {
        return Ok(LastErrorInfo {
            found: false,
            error: None,
        });
    };

    let error_line = &all_lines[error_idx];

    // Try to collect stacktrace (lines after the error that look like stack frames)
    let mut stacktrace_lines = Vec::new();
    for line in all_lines.iter().skip(error_idx + 1) {
        // Stop if we hit another log entry or empty line
        if line.starts_with('[') || line.is_empty() {
            break;
        }
        // Stack trace lines often start with whitespace or "at"
        if line.starts_with(' ') || line.starts_with('\t') || line.trim().starts_with("at ") {
            stacktrace_lines.push(line.clone());
        }
    }

    let stacktrace = if stacktrace_lines.is_empty() {
        None
    } else {
        Some(stacktrace_lines.join("\n"))
    };

    // Parse the error line
    let (timestamp, message) = parse_error_line(error_line);

    // Categorize the error
    let category = categorize_error(&message, error_level);

    // Extract route context from error message and stacktrace
    let full_context = format!("{}\n{}", message, stacktrace.as_deref().unwrap_or_default());
    let route_context = extract_route_context(&full_context);

    // Find related routes based on error context
    let related_routes = find_related_routes(&message, &route_context);

    Ok(LastErrorInfo {
        found: true,
        error: Some(ErrorDetails {
            timestamp,
            level: error_level.to_string(),
            message,
            stacktrace,
            category,
            route_context,
            related_routes,
        }),
    })
}

/// Categorize an error message into an ErrorCategory
pub fn categorize_error(message: &str, level: &str) -> ErrorCategory {
    let msg_lower = message.to_lowercase();

    // Panic/fatal errors take priority
    if level == "PANIC" || msg_lower.contains("panic") || msg_lower.contains("fatal") {
        return ErrorCategory::Panic;
    }

    // Validation patterns
    if msg_lower.contains("validation")
        || msg_lower.contains("invalid")
        || msg_lower.contains("required field")
        || msg_lower.contains("must be")
        || msg_lower.contains("422")
    {
        return ErrorCategory::Validation;
    }

    // Not found patterns
    if msg_lower.contains("not found")
        || msg_lower.contains("404")
        || msg_lower.contains("no such")
        || msg_lower.contains("does not exist")
        || msg_lower.contains("missing")
    {
        return ErrorCategory::NotFound;
    }

    // Database patterns
    if msg_lower.contains("database")
        || msg_lower.contains("sql")
        || msg_lower.contains("query")
        || msg_lower.contains("migration")
        || msg_lower.contains("connection refused")
        || msg_lower.contains("dberr")
        || msg_lower.contains("sea_orm")
        || msg_lower.contains("sqlx")
    {
        return ErrorCategory::Database;
    }

    // Permission patterns
    if msg_lower.contains("permission")
        || msg_lower.contains("forbidden")
        || msg_lower.contains("unauthorized")
        || msg_lower.contains("401")
        || msg_lower.contains("403")
        || msg_lower.contains("access denied")
        || msg_lower.contains("authentication")
    {
        return ErrorCategory::Permission;
    }

    // Default to internal
    ErrorCategory::Internal
}

/// Extract route context from error message and stacktrace
fn extract_route_context(context: &str) -> Option<RouteContext> {
    let path = ROUTE_PATH_PATTERN
        .captures(context)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());

    let handler = HANDLER_PATTERN
        .captures(context)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());

    let method = METHOD_PATTERN
        .captures(context)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());

    if path.is_some() || handler.is_some() || method.is_some() {
        Some(RouteContext {
            handler,
            path,
            method,
        })
    } else {
        None
    }
}

/// Find related routes based on error context
fn find_related_routes(message: &str, route_context: &Option<RouteContext>) -> Vec<String> {
    let mut related = Vec::new();
    let msg_lower = message.to_lowercase();

    // If we have a path, suggest similar routes
    if let Some(ctx) = route_context {
        if let Some(path) = &ctx.path {
            // Extract base path for related routes
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() > 1 {
                let base = format!("/{}", parts.get(1).unwrap_or(&""));
                related.push(format!("Routes under {base}/..."));
            }
        }
    }

    // Add suggestions based on error type
    if msg_lower.contains("user") || msg_lower.contains("auth") {
        related.push("/auth/login".to_string());
        related.push("/auth/register".to_string());
    }
    if msg_lower.contains("session") {
        related.push("/auth/logout".to_string());
    }

    // Limit to 5 suggestions
    related.truncate(5);
    related
}

fn parse_error_line(line: &str) -> (Option<String>, String) {
    let line = line.trim();

    // Try to extract timestamp from [timestamp] format
    if line.starts_with('[') {
        if let Some(bracket_end) = line.find(']') {
            let timestamp = line[1..bracket_end].to_string();
            let message = line[bracket_end + 1..].trim().to_string();
            return (Some(timestamp), message);
        }
    }

    (None, line.to_string())
}
