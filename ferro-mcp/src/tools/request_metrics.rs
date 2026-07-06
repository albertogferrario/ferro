//! Request metrics tool - show performance data per route
//!
//! This tool fetches metrics from the running application via
//! the `/_ferro/metrics` debug endpoint.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Timeout for HTTP requests to the running application
const HTTP_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Serialize)]
pub struct MetricsInfo {
    pub metrics: MetricsSnapshot,
    /// Indicates whether metrics came from runtime
    pub source: MetricsSource,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricsSource {
    /// Metrics fetched from running application via HTTP endpoint
    Runtime,
    /// App not running or endpoint unavailable
    Unavailable,
}

/// Response format from the `/_ferro/metrics` endpoint
#[derive(Debug, Deserialize, Serialize)]
pub struct DebugResponse {
    pub success: bool,
    pub data: MetricsSnapshot,
}

/// Metrics snapshot from the runtime
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct MetricsSnapshot {
    pub routes: Vec<RouteMetricsView>,
    pub total_requests: u64,
    pub total_errors: u64,
    pub uptime_seconds: u64,
}

/// Per-route metrics
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RouteMetricsView {
    pub route: String,
    pub method: String,
    pub count: u64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: Option<u64>,
    pub max_duration_ms: u64,
    pub error_count: u64,
    pub error_rate: f64,
}

/// Try to fetch metrics from the running application
async fn fetch_runtime_metrics(base_url: &str) -> Option<MetricsSnapshot> {
    let url = format!("{base_url}/_ferro/metrics");

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

    Some(debug_response.data)
}

pub async fn execute() -> Result<MetricsInfo> {
    // Try runtime endpoints
    for base_url in ["http://localhost:8080", "http://127.0.0.1:8080"] {
        if let Some(metrics) = fetch_runtime_metrics(base_url).await {
            return Ok(MetricsInfo {
                metrics,
                source: MetricsSource::Runtime,
            });
        }
    }

    // No metrics available - app not running
    Ok(MetricsInfo {
        metrics: MetricsSnapshot::default(),
        source: MetricsSource::Unavailable,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_snapshot_default() {
        let snapshot = MetricsSnapshot::default();
        assert!(snapshot.routes.is_empty());
        assert_eq!(snapshot.total_requests, 0);
        assert_eq!(snapshot.total_errors, 0);
        assert_eq!(snapshot.uptime_seconds, 0);
    }

    #[test]
    fn test_metrics_snapshot_serialization() {
        let snapshot = MetricsSnapshot {
            routes: vec![RouteMetricsView {
                route: "/api/users".to_string(),
                method: "GET".to_string(),
                count: 100,
                avg_duration_ms: 15.5,
                min_duration_ms: Some(5),
                max_duration_ms: 50,
                error_count: 2,
                error_rate: 0.02,
            }],
            total_requests: 100,
            total_errors: 2,
            uptime_seconds: 3600,
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let restored: MetricsSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.routes.len(), 1);
        assert_eq!(restored.routes[0].route, "/api/users");
        assert_eq!(restored.total_requests, 100);
        assert_eq!(restored.uptime_seconds, 3600);
    }

    #[test]
    fn test_route_metrics_view_serialization() {
        let view = RouteMetricsView {
            route: "/users/{id}".to_string(),
            method: "POST".to_string(),
            count: 50,
            avg_duration_ms: 25.0,
            min_duration_ms: None,
            max_duration_ms: 100,
            error_count: 5,
            error_rate: 0.1,
        };

        let json = serde_json::to_string(&view).unwrap();
        let restored: RouteMetricsView = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.route, "/users/{id}");
        assert_eq!(restored.method, "POST");
        assert!(restored.min_duration_ms.is_none());
        assert!((restored.error_rate - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_metrics_source_serialization() {
        assert_eq!(
            serde_json::to_string(&MetricsSource::Runtime).unwrap(),
            "\"runtime\""
        );
        assert_eq!(
            serde_json::to_string(&MetricsSource::Unavailable).unwrap(),
            "\"unavailable\""
        );
    }

    #[test]
    fn test_metrics_source_debug() {
        assert!(format!("{:?}", MetricsSource::Runtime).contains("Runtime"));
        assert!(format!("{:?}", MetricsSource::Unavailable).contains("Unavailable"));
    }

    #[test]
    fn test_metrics_info_debug() {
        let info = MetricsInfo {
            metrics: MetricsSnapshot::default(),
            source: MetricsSource::Unavailable,
        };
        assert!(format!("{info:?}").contains("MetricsInfo"));
    }

    #[test]
    fn test_debug_response_deserialization() {
        let json = r#"{
            "success": true,
            "data": {
                "routes": [],
                "total_requests": 0,
                "total_errors": 0,
                "uptime_seconds": 100
            }
        }"#;

        let response: DebugResponse = serde_json::from_str(json).unwrap();
        assert!(response.success);
        assert_eq!(response.data.uptime_seconds, 100);
    }

    // Note: Testing the execute() function and fetch_runtime_metrics()
    // requires integration tests with a running application server.
}
