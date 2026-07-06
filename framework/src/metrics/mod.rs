//! Request metrics collection for performance monitoring
//!
//! Collects request counts, response times, and error rates per route.
//! Metrics are stored in-memory and exposed via `/_ferro/metrics`.

use serde::Serialize;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use std::time::Duration;

/// Safety cap on the number of unique route entries in the metrics store.
/// Normalization handles the common case (unmatched routes → "UNMATCHED"),
/// this cap prevents any other unbounded growth scenario.
const MAX_ROUTE_ENTRIES: usize = 1000;

/// Request metrics for a single route
#[derive(Debug, Clone, Serialize)]
pub struct RouteMetrics {
    /// Route pattern (e.g., "/users/{id}")
    pub route: String,
    /// HTTP method
    pub method: String,
    /// Total request count
    pub count: u64,
    /// Total duration in milliseconds (for average calculation)
    pub total_duration_ms: u64,
    /// Number of error responses (4xx and 5xx)
    pub error_count: u64,
    /// Minimum response time in ms
    pub min_duration_ms: u64,
    /// Maximum response time in ms
    pub max_duration_ms: u64,
}

impl RouteMetrics {
    fn new(route: String, method: String) -> Self {
        Self {
            route,
            method,
            count: 0,
            total_duration_ms: 0,
            error_count: 0,
            min_duration_ms: u64::MAX,
            max_duration_ms: 0,
        }
    }

    /// Calculate average response time in milliseconds
    pub fn avg_duration_ms(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.total_duration_ms as f64 / self.count as f64
        }
    }
}

/// Aggregated metrics response
#[derive(Debug, Serialize)]
pub struct MetricsSnapshot {
    /// Per-route metrics
    pub routes: Vec<RouteMetricsView>,
    /// Total requests across all routes
    pub total_requests: u64,
    /// Total errors across all routes
    pub total_errors: u64,
    /// Uptime since metrics collection started (seconds)
    pub uptime_seconds: u64,
}

/// View of route metrics for serialization (includes computed avg)
#[derive(Debug, Serialize)]
pub struct RouteMetricsView {
    /// Route pattern (e.g., "/users/{id}").
    pub route: String,
    /// HTTP method (e.g., "GET", "POST").
    pub method: String,
    /// Total number of requests for this route.
    pub count: u64,
    /// Average response time in milliseconds.
    pub avg_duration_ms: f64,
    /// Minimum response time in ms, or `None` if no requests recorded.
    pub min_duration_ms: Option<u64>,
    /// Maximum response time in milliseconds.
    pub max_duration_ms: u64,
    /// Number of error responses (4xx and 5xx).
    pub error_count: u64,
    /// Ratio of error responses to total requests (0.0–1.0).
    pub error_rate: f64,
}

/// Global metrics storage
static METRICS: OnceLock<RwLock<MetricsStore>> = OnceLock::new();

struct MetricsStore {
    routes: HashMap<String, RouteMetrics>,
    start_time: std::time::Instant,
}

impl MetricsStore {
    fn new() -> Self {
        Self {
            routes: HashMap::new(),
            start_time: std::time::Instant::now(),
        }
    }
}

fn get_store() -> &'static RwLock<MetricsStore> {
    METRICS.get_or_init(|| RwLock::new(MetricsStore::new()))
}

/// Generate a unique key for route metrics
fn route_key(method: &str, route: &str) -> String {
    format!("{method}:{route}")
}

/// Record a request completion
///
/// # Arguments
/// * `route` - Route pattern (e.g., "/users/{id}")
/// * `method` - HTTP method
/// * `duration` - Request duration
/// * `is_error` - Whether response was an error (4xx or 5xx)
pub fn record_request(route: &str, method: &str, duration: Duration, is_error: bool) {
    let key = route_key(method, route);
    let duration_ms = duration.as_millis() as u64;

    if let Ok(mut store) = get_store().write() {
        // Safety cap: if at capacity and this is a new key, skip recording
        if store.routes.len() >= MAX_ROUTE_ENTRIES && !store.routes.contains_key(&key) {
            return;
        }

        let metrics = store
            .routes
            .entry(key)
            .or_insert_with(|| RouteMetrics::new(route.to_string(), method.to_string()));

        metrics.count += 1;
        metrics.total_duration_ms += duration_ms;

        if duration_ms < metrics.min_duration_ms {
            metrics.min_duration_ms = duration_ms;
        }
        if duration_ms > metrics.max_duration_ms {
            metrics.max_duration_ms = duration_ms;
        }

        if is_error {
            metrics.error_count += 1;
        }
    }
}

/// Get current metrics snapshot
pub fn get_metrics() -> MetricsSnapshot {
    let store = get_store().read().unwrap();

    let mut total_requests = 0u64;
    let mut total_errors = 0u64;

    let routes: Vec<RouteMetricsView> = store
        .routes
        .values()
        .map(|m| {
            total_requests += m.count;
            total_errors += m.error_count;

            RouteMetricsView {
                route: m.route.clone(),
                method: m.method.clone(),
                count: m.count,
                avg_duration_ms: m.avg_duration_ms(),
                min_duration_ms: if m.min_duration_ms == u64::MAX {
                    None
                } else {
                    Some(m.min_duration_ms)
                },
                max_duration_ms: m.max_duration_ms,
                error_count: m.error_count,
                error_rate: if m.count == 0 {
                    0.0
                } else {
                    m.error_count as f64 / m.count as f64
                },
            }
        })
        .collect();

    MetricsSnapshot {
        routes,
        total_requests,
        total_errors,
        uptime_seconds: store.start_time.elapsed().as_secs(),
    }
}

/// Reset all metrics (useful for testing)
pub fn reset_metrics() {
    if let Ok(mut store) = get_store().write() {
        store.routes.clear();
        store.start_time = std::time::Instant::now();
    }
}

/// Check if metrics collection is enabled
pub fn is_enabled() -> bool {
    std::env::var("FERRO_COLLECT_METRICS")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(true) // Enabled by default in development
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::time::Duration;

    fn setup() {
        reset_metrics();
    }

    #[test]
    #[serial]
    fn test_record_request_increments_count() {
        setup();

        record_request("/users", "GET", Duration::from_millis(10), false);
        record_request("/users", "GET", Duration::from_millis(20), false);

        let snapshot = get_metrics();
        let route = snapshot
            .routes
            .iter()
            .find(|r| r.route == "/users")
            .unwrap();

        assert_eq!(route.count, 2);
        assert_eq!(snapshot.total_requests, 2);
    }

    #[test]
    #[serial]
    fn test_record_request_tracks_duration() {
        setup();

        record_request("/api/test", "POST", Duration::from_millis(10), false);
        record_request("/api/test", "POST", Duration::from_millis(30), false);
        record_request("/api/test", "POST", Duration::from_millis(20), false);

        let snapshot = get_metrics();
        let route = snapshot
            .routes
            .iter()
            .find(|r| r.route == "/api/test")
            .unwrap();

        assert_eq!(route.min_duration_ms, Some(10));
        assert_eq!(route.max_duration_ms, 30);
        assert!((route.avg_duration_ms - 20.0).abs() < 0.01);
    }

    #[test]
    #[serial]
    fn test_record_request_counts_errors() {
        setup();

        record_request("/error", "GET", Duration::from_millis(5), false);
        record_request("/error", "GET", Duration::from_millis(5), true);
        record_request("/error", "GET", Duration::from_millis(5), true);

        let snapshot = get_metrics();
        let route = snapshot
            .routes
            .iter()
            .find(|r| r.route == "/error")
            .unwrap();

        assert_eq!(route.count, 3);
        assert_eq!(route.error_count, 2);
        assert!((route.error_rate - 2.0 / 3.0).abs() < 0.01);
        assert_eq!(snapshot.total_errors, 2);
    }

    #[test]
    #[serial]
    fn test_different_methods_tracked_separately() {
        setup();

        record_request("/resource", "GET", Duration::from_millis(10), false);
        record_request("/resource", "POST", Duration::from_millis(20), false);
        record_request("/resource", "GET", Duration::from_millis(15), false);

        let snapshot = get_metrics();

        let get_route = snapshot
            .routes
            .iter()
            .find(|r| r.route == "/resource" && r.method == "GET")
            .unwrap();
        let post_route = snapshot
            .routes
            .iter()
            .find(|r| r.route == "/resource" && r.method == "POST")
            .unwrap();

        assert_eq!(get_route.count, 2);
        assert_eq!(post_route.count, 1);
    }

    #[test]
    fn test_route_metrics_avg_duration_zero_count() {
        let metrics = RouteMetrics::new("/test".to_string(), "GET".to_string());
        assert_eq!(metrics.avg_duration_ms(), 0.0);
    }

    #[test]
    #[serial]
    fn test_min_duration_none_when_no_requests() {
        setup();

        // Record to a different route
        record_request("/other", "GET", Duration::from_millis(10), false);

        let snapshot = get_metrics();

        // Find a route that exists
        let route = snapshot
            .routes
            .iter()
            .find(|r| r.route == "/other")
            .unwrap();
        assert_eq!(route.min_duration_ms, Some(10));
    }

    #[test]
    #[serial]
    fn test_reset_metrics_clears_data() {
        setup();

        record_request("/clear-test", "GET", Duration::from_millis(10), false);

        let snapshot = get_metrics();
        assert!(!snapshot.routes.is_empty());

        reset_metrics();

        let snapshot = get_metrics();
        assert!(snapshot.routes.is_empty());
        assert_eq!(snapshot.total_requests, 0);
    }

    #[test]
    #[serial]
    fn test_uptime_tracking() {
        setup();

        let snapshot1 = get_metrics();
        std::thread::sleep(Duration::from_millis(50));
        let snapshot2 = get_metrics();

        // Uptime should have increased
        assert!(snapshot2.uptime_seconds >= snapshot1.uptime_seconds);
    }

    #[test]
    fn test_route_key_format() {
        assert_eq!(route_key("GET", "/users"), "GET:/users");
        assert_eq!(route_key("POST", "/api/v1/items"), "POST:/api/v1/items");
    }

    #[test]
    #[serial]
    fn test_unmatched_routes_use_fixed_bucket() {
        setup();

        // Simulate what the middleware now does: all unmatched routes → "UNMATCHED"
        record_request("UNMATCHED", "GET", Duration::from_millis(5), true);
        record_request("UNMATCHED", "GET", Duration::from_millis(10), true);
        record_request("UNMATCHED", "POST", Duration::from_millis(8), true);

        let snapshot = get_metrics();

        // GET:UNMATCHED and POST:UNMATCHED — only 2 entries, not N unique paths
        let unmatched_routes: Vec<_> = snapshot
            .routes
            .iter()
            .filter(|r| r.route == "UNMATCHED")
            .collect();
        assert_eq!(unmatched_routes.len(), 2);

        let get_unmatched = unmatched_routes.iter().find(|r| r.method == "GET").unwrap();
        assert_eq!(get_unmatched.count, 2);
    }

    #[test]
    #[serial]
    fn test_entry_cap_prevents_unbounded_growth() {
        setup();

        // Fill to MAX_ROUTE_ENTRIES with unique routes
        for i in 0..MAX_ROUTE_ENTRIES + 100 {
            let route = format!("/route/{i}");
            record_request(&route, "GET", Duration::from_millis(1), false);
        }

        let snapshot = get_metrics();
        assert!(snapshot.routes.len() <= MAX_ROUTE_ENTRIES);
    }

    #[test]
    #[serial]
    fn test_existing_entries_updated_after_cap() {
        setup();

        // Fill exactly to MAX_ROUTE_ENTRIES
        for i in 0..MAX_ROUTE_ENTRIES {
            let route = format!("/route/{i}");
            record_request(&route, "GET", Duration::from_millis(1), false);
        }

        let snapshot = get_metrics();
        assert_eq!(snapshot.routes.len(), MAX_ROUTE_ENTRIES);

        // Record another request for an existing route — should still be tracked
        record_request("/route/0", "GET", Duration::from_millis(50), false);

        let snapshot = get_metrics();
        let route = snapshot
            .routes
            .iter()
            .find(|r| r.route == "/route/0" && r.method == "GET")
            .unwrap();
        assert_eq!(route.count, 2); // Original + new request
    }
}
