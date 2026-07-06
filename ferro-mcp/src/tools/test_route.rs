//! Test route tool - simulate HTTP request and show response

use crate::error::{McpError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize)]
pub struct RouteTestResult {
    pub request: RequestInfo,
    pub response: ResponseInfo,
    pub timing_ms: u64,
    pub route_matched: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RequestInfo {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResponseInfo {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body_preview: String,
    pub body_size_bytes: usize,
    pub content_type: Option<String>,
    pub is_redirect: bool,
    pub redirect_location: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TestRouteParams {
    pub method: String,
    pub path: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
    pub follow_redirects: Option<bool>,
}

pub async fn execute(project_root: &Path, params: TestRouteParams) -> Result<RouteTestResult> {
    // Get the app URL from environment
    dotenvy::from_path(project_root.join(".env")).ok();

    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let full_url = format!("{}{}", app_url.trim_end_matches('/'), params.path);

    let start = std::time::Instant::now();

    // Build curl command for the request
    let mut curl_args = vec![
        "-s".to_string(), // Silent mode
        "-w".to_string(), // Write out format
        "\n---CURL_INFO---\n%{http_code}\n%{content_type}\n%{redirect_url}\n%{time_total}"
            .to_string(),
        "-D".to_string(), // Dump headers to stdout
        "-".to_string(),
        "-X".to_string(),
        params.method.to_uppercase(),
    ];

    // Add headers
    let mut request_headers = HashMap::new();
    request_headers.insert(
        "Accept".to_string(),
        "application/json, text/html".to_string(),
    );

    if let Some(headers) = &params.headers {
        for (key, value) in headers {
            request_headers.insert(key.clone(), value.clone());
            curl_args.push("-H".to_string());
            curl_args.push(format!("{key}: {value}"));
        }
    }

    // Add body if present
    if let Some(body) = &params.body {
        curl_args.push("-d".to_string());
        curl_args.push(body.clone());
        if !request_headers.contains_key("Content-Type") {
            curl_args.push("-H".to_string());
            curl_args.push("Content-Type: application/json".to_string());
        }
    }

    // Handle redirects
    if params.follow_redirects.unwrap_or(false) {
        curl_args.push("-L".to_string());
    }

    curl_args.push(full_url.clone());

    // Execute curl
    let output = Command::new("curl")
        .args(&curl_args)
        .output()
        .map_err(|e| McpError::ExecutionError(format!("Failed to execute curl: {e}")))?;

    let timing_ms = start.elapsed().as_millis() as u64;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.split("---CURL_INFO---").collect();

    let (headers_and_body, curl_info) = if parts.len() >= 2 {
        (parts[0], parts[1])
    } else {
        (stdout.as_ref(), "")
    };

    // Parse curl info
    let curl_info_lines: Vec<&str> = curl_info.trim().lines().collect();
    let status_code: u16 = curl_info_lines
        .first()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let content_type = curl_info_lines
        .get(1)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let redirect_url = curl_info_lines
        .get(2)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    // Parse response headers and body
    let (response_headers, body) = parse_response(headers_and_body);

    let body_size = body.len();
    let body_preview = if body.len() > 2000 {
        format!(
            "{}...\n\n[truncated, total {} bytes]",
            &body[..2000],
            body_size
        )
    } else {
        body.clone()
    };

    let is_redirect = (300..400).contains(&status_code);

    Ok(RouteTestResult {
        request: RequestInfo {
            method: params.method.to_uppercase(),
            url: full_url,
            headers: request_headers,
            body: params.body,
        },
        response: ResponseInfo {
            status_code,
            status_text: status_text(status_code),
            headers: response_headers,
            body_preview,
            body_size_bytes: body_size,
            content_type,
            is_redirect,
            redirect_location: redirect_url,
        },
        timing_ms,
        route_matched: None, // Would need framework integration to determine
    })
}

fn parse_response(raw: &str) -> (HashMap<String, String>, String) {
    let mut headers = HashMap::new();
    let mut body = String::new();
    let mut in_body = false;
    let mut header_section_ended = false;

    for line in raw.lines() {
        if !header_section_ended {
            // HTTP response headers end with empty line
            if line.is_empty() || line == "\r" {
                header_section_ended = true;
                in_body = true;
                continue;
            }

            // Skip HTTP status line
            if line.starts_with("HTTP/") {
                continue;
            }

            // Parse header
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
        } else if in_body {
            if !body.is_empty() {
                body.push('\n');
            }
            body.push_str(line);
        }
    }

    (headers, body)
}

fn status_text(code: u16) -> String {
    match code {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Unknown",
    }
    .to_string()
}
