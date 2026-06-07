//! Debug introspection endpoints for development
//!
//! These endpoints expose runtime application state for AI-assisted development
//! and debugging. They are automatically disabled in production.

use crate::config::Config;
use crate::container::get_registered_services;
use crate::metrics;
use crate::middleware::get_global_middleware_info;
use crate::routing::get_registered_routes;
use bytes::Bytes;
use chrono::Utc;
use http_body_util::Full;
use serde::Serialize;

/// Response wrapper for debug endpoints
#[derive(Debug, Serialize)]
pub struct DebugResponse<T: Serialize> {
    /// Whether the debug operation succeeded.
    pub success: bool,
    /// The debug payload.
    pub data: T,
    /// RFC 3339 timestamp of when the response was generated.
    pub timestamp: String,
}

/// Error response for debug endpoints
#[derive(Debug, Serialize)]
pub struct DebugErrorResponse {
    /// Always `false` for error responses.
    pub success: bool,
    /// Human-readable error description.
    pub error: String,
    /// RFC 3339 timestamp of when the error occurred.
    pub timestamp: String,
}

/// Check if debug endpoints should be enabled
pub fn is_debug_enabled() -> bool {
    // Disabled in production unless explicitly enabled
    if Config::is_production() {
        return std::env::var("FERRO_DEBUG_ENDPOINTS")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
    }
    true
}

/// Build a JSON response for debug endpoints
fn json_response<T: Serialize>(data: T, status: u16) -> hyper::Response<Full<Bytes>> {
    let body = serde_json::to_string_pretty(&data).unwrap_or_else(|_| "{}".to_string());
    hyper::Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(body)))
        .unwrap()
}

/// Handle /_ferro/routes endpoint
pub fn handle_routes() -> hyper::Response<Full<Bytes>> {
    if !is_debug_enabled() {
        return json_response(
            DebugErrorResponse {
                success: false,
                error: "Debug endpoints disabled in production".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            },
            403,
        );
    }

    let routes = get_registered_routes();
    json_response(
        DebugResponse {
            success: true,
            data: routes,
            timestamp: Utc::now().to_rfc3339(),
        },
        200,
    )
}

/// Global middleware info for introspection
#[derive(Debug, Serialize)]
pub struct MiddlewareInfo {
    /// Names of globally registered middleware, in registration order.
    pub global: Vec<String>,
}

/// Handle /_ferro/middleware endpoint
pub fn handle_middleware() -> hyper::Response<Full<Bytes>> {
    if !is_debug_enabled() {
        return json_response(
            DebugErrorResponse {
                success: false,
                error: "Debug endpoints disabled in production".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            },
            403,
        );
    }

    let global = get_global_middleware_info();
    json_response(
        DebugResponse {
            success: true,
            data: MiddlewareInfo { global },
            timestamp: Utc::now().to_rfc3339(),
        },
        200,
    )
}

/// Handle /_ferro/services endpoint
pub fn handle_services() -> hyper::Response<Full<Bytes>> {
    if !is_debug_enabled() {
        return json_response(
            DebugErrorResponse {
                success: false,
                error: "Debug endpoints disabled in production".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            },
            403,
        );
    }

    let services = get_registered_services();
    json_response(
        DebugResponse {
            success: true,
            data: services,
            timestamp: Utc::now().to_rfc3339(),
        },
        200,
    )
}

/// Handle /_ferro/metrics endpoint
pub fn handle_metrics() -> hyper::Response<Full<Bytes>> {
    if !is_debug_enabled() {
        return json_response(
            DebugErrorResponse {
                success: false,
                error: "Debug endpoints disabled in production".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            },
            403,
        );
    }

    let snapshot = metrics::get_metrics();
    json_response(
        DebugResponse {
            success: true,
            data: snapshot,
            timestamp: Utc::now().to_rfc3339(),
        },
        200,
    )
}

/// Queue jobs response
#[derive(Debug, Serialize)]
pub struct QueueJobsInfo {
    /// Pending jobs (ready to process)
    pub pending: Vec<ferro_queue::JobInfo>,
    /// Delayed jobs (waiting for available_at)
    pub delayed: Vec<ferro_queue::JobInfo>,
    /// Failed jobs
    pub failed: Vec<ferro_queue::FailedJobInfo>,
}

/// Handle /_ferro/queue/jobs endpoint
pub async fn handle_queue_jobs() -> hyper::Response<Full<Bytes>> {
    if !is_debug_enabled() {
        return json_response(
            DebugErrorResponse {
                success: false,
                error: "Debug endpoints disabled in production".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            },
            403,
        );
    }

    // Check if queue is initialized
    if !ferro_queue::Queue::is_initialized() {
        return json_response(
            DebugErrorResponse {
                success: false,
                error: "Queue not initialized (QUEUE_CONNECTION=sync or Redis not configured)"
                    .to_string(),
                timestamp: Utc::now().to_rfc3339(),
            },
            503,
        );
    }

    let conn = ferro_queue::Queue::connection();
    let default_queue = ferro_queue::QueueConfig::from_env().default_queue;
    let default_queue = default_queue.as_str();

    // Fetch jobs from the default queue (DB-backed, D-18).
    let pending = ferro_queue::get_pending_jobs(conn, default_queue, 100)
        .await
        .unwrap_or_default();
    let delayed = ferro_queue::get_delayed_jobs(conn, default_queue, 100)
        .await
        .unwrap_or_default();
    let failed = ferro_queue::get_failed_jobs(conn, 100)
        .await
        .unwrap_or_default();

    json_response(
        DebugResponse {
            success: true,
            data: QueueJobsInfo {
                pending,
                delayed,
                failed,
            },
            timestamp: Utc::now().to_rfc3339(),
        },
        200,
    )
}

/// Handle /_ferro/queue/stats endpoint
pub async fn handle_queue_stats() -> hyper::Response<Full<Bytes>> {
    if !is_debug_enabled() {
        return json_response(
            DebugErrorResponse {
                success: false,
                error: "Debug endpoints disabled in production".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            },
            403,
        );
    }

    // Check if queue is initialized
    if !ferro_queue::Queue::is_initialized() {
        return json_response(
            DebugErrorResponse {
                success: false,
                error: "Queue not initialized (QUEUE_CONNECTION=sync or Redis not configured)"
                    .to_string(),
                timestamp: Utc::now().to_rfc3339(),
            },
            503,
        );
    }

    let conn = ferro_queue::Queue::connection();
    let default_queue = ferro_queue::QueueConfig::from_env().default_queue;
    let default_queue = default_queue.as_str();

    // Get stats for default queue (DB-backed, D-18).
    let stats = ferro_queue::get_stats(conn, &[default_queue])
        .await
        .unwrap_or_default();

    json_response(
        DebugResponse {
            success: true,
            data: stats,
            timestamp: Utc::now().to_rfc3339(),
        },
        200,
    )
}
