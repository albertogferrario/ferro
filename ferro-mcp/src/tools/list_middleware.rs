//! List middleware tool - scan for registered middleware
//!
//! This tool tries to fetch middleware from the running application first via
//! the `/_ferro/middleware` debug endpoint, falling back to static file parsing
//! when the app isn't running.

use crate::error::Result;
use crate::introspection::middleware::{self, MiddlewareItem};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

/// Timeout for HTTP requests to the running application
const HTTP_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Serialize)]
pub struct MiddlewareInfo {
    pub middleware: Vec<MiddlewareItem>,
    /// Indicates whether middleware came from runtime or static analysis
    pub source: MiddlewareSource,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MiddlewareSource {
    /// Middleware fetched from running application via HTTP endpoint
    Runtime,
    /// Middleware parsed from source files (fallback when app not running)
    StaticAnalysis,
}

/// Response format from the `/_ferro/middleware` endpoint
#[derive(Debug, Deserialize)]
struct DebugResponse {
    success: bool,
    data: RuntimeMiddlewareInfo,
}

/// Middleware info as returned by the runtime endpoint
#[derive(Debug, Deserialize)]
struct RuntimeMiddlewareInfo {
    global: Vec<String>,
}

/// Try to fetch middleware from the running application
async fn fetch_runtime_middleware(base_url: &str) -> Option<Vec<MiddlewareItem>> {
    let url = format!("{base_url}/_ferro/middleware");

    let client = reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .build()
        .ok()?;
    let response = client.get(&url).send().await.ok()?;

    if !response.status().is_success() {
        return None;
    }

    let debug_response: DebugResponse = response.json().await.ok()?;

    if !debug_response.success {
        return None;
    }

    Some(
        debug_response
            .data
            .global
            .into_iter()
            .map(|name| MiddlewareItem {
                name,
                path: String::new(), // Runtime doesn't expose file paths
                global: true,
            })
            .collect(),
    )
}

pub async fn execute(project_root: &Path) -> Result<MiddlewareInfo> {
    // Try runtime endpoint first
    for base_url in ["http://localhost:8080", "http://127.0.0.1:8080"] {
        if let Some(middleware) = fetch_runtime_middleware(base_url).await {
            return Ok(MiddlewareInfo {
                middleware,
                source: MiddlewareSource::Runtime,
            });
        }
    }

    // Fall back to static analysis
    let middleware_items = middleware::scan_middleware(project_root);
    Ok(MiddlewareInfo {
        middleware: middleware_items,
        source: MiddlewareSource::StaticAnalysis,
    })
}
