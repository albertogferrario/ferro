//! Per-key in-process ring buffer for sampled time-series telemetry.
//!
//! Process-global storage. Bounded at 128 samples per `(key, scope)` pair.
//! Oldest samples are dropped on overflow. Lost on process restart.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::OnceLock;
use std::time::SystemTime;

/// One sample recorded against a `(key, scope)` bucket.
///
/// `value` is a `serde_json::Value` so heterogeneous payloads from different
/// writers can share a snapshot reader.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    /// Wall-clock timestamp the sample was recorded at.
    pub recorded_at: SystemTime,
    /// Caller-chosen payload.
    pub value: serde_json::Value,
}

impl Sample {
    /// Record `value` at `SystemTime::now()`.
    pub fn now(value: serde_json::Value) -> Self {
        Self {
            recorded_at: SystemTime::now(),
            value,
        }
    }

    /// Record `value` at a caller-provided timestamp (for backfilled batches).
    pub fn at(when: SystemTime, value: serde_json::Value) -> Self {
        Self {
            recorded_at: when,
            value,
        }
    }
}

/// Namespace for operator-side static methods over the global telemetry store.
///
/// Writers do not interact with this type directly — they call
/// `crate::http::Request::telemetry_record` or
/// `crate::http::Request::telemetry_record_scoped`.
pub struct RequestTelemetry;

/// `(key, scope)` bucket identifier. `scope` is caller-defined; convention is
/// `"tenant:42"`, `"route:/api/products"`, etc.
type BucketKey = (String, Option<String>);

/// Per-bucket sample storage. `VecDeque` is used for O(1) push-back / pop-front
/// ring-buffer semantics.
type TelemetryStore = DashMap<BucketKey, VecDeque<Sample>>;

static TELEMETRY_STORE: OnceLock<TelemetryStore> = OnceLock::new();

fn telemetry_store() -> &'static TelemetryStore {
    TELEMETRY_STORE.get_or_init(DashMap::new)
}

/// Per-bucket ring-buffer capacity. Power of 2; chosen for cheap arithmetic
/// and a "lots of samples without crowding memory" budget.
pub(crate) const RING_BUFFER_CAPACITY: usize = 128;

/// Push `sample` into the `(key, scope)` bucket. Drops the oldest if the bucket
/// exceeds [`RING_BUFFER_CAPACITY`].
///
/// Called from `crate::http::Request::telemetry_record` and
/// `crate::http::Request::telemetry_record_scoped`.
pub(crate) fn record(key: &str, scope: Option<&str>, sample: Sample) {
    let map_key = (key.to_string(), scope.map(|s| s.to_string()));
    let mut entry = telemetry_store()
        .entry(map_key)
        .or_insert_with(|| VecDeque::with_capacity(RING_BUFFER_CAPACITY));
    entry.push_back(sample);
    while entry.len() > RING_BUFFER_CAPACITY {
        entry.pop_front();
    }
}

impl RequestTelemetry {
    /// Clone all samples recorded under `(key, scope)` into a new `Vec` (FIFO order).
    pub fn snapshot(key: &str, scope: Option<&str>) -> Vec<Sample> {
        let scope_owned = scope.map(|s| s.to_string());
        telemetry_store()
            .get(&(key.to_string(), scope_owned))
            .map(|entry| entry.value().iter().cloned().collect())
            .unwrap_or_default()
    }

    /// List every `(key, scope)` pair that has at least one recorded sample.
    pub fn keys() -> Vec<(String, Option<String>)> {
        telemetry_store().iter().map(|e| e.key().clone()).collect()
    }

    /// Drop every sample in every bucket. Intended for operator-driven resets.
    pub fn clear() {
        if let Some(r) = TELEMETRY_STORE.get() {
            r.clear();
        }
    }

    /// Test-isolation helper. Same effect as [`RequestTelemetry::clear`] but
    /// scoped to `#[cfg(test)]` builds so production code cannot reach for it.
    #[cfg(test)]
    pub(crate) fn reset() {
        Self::clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use serial_test::serial;
    use std::thread;
    use std::time::Duration;

    #[test]
    #[serial]
    fn sample_constructors() {
        let before = SystemTime::now();
        let s = Sample::now(json!({"x": 1}));
        let after = SystemTime::now();
        assert!(s.recorded_at >= before && s.recorded_at <= after);
        assert_eq!(s.value, json!({"x": 1}));

        let epoch = SystemTime::UNIX_EPOCH;
        let s2 = Sample::at(epoch, json!({"x": 2}));
        assert_eq!(s2.recorded_at, epoch);
        assert_eq!(s2.value, json!({"x": 2}));
    }

    #[test]
    #[serial]
    fn record_and_snapshot_round_trip() {
        RequestTelemetry::reset();
        record("k1", None, Sample::now(json!({"i": 0})));
        record("k1", None, Sample::now(json!({"i": 1})));
        record("k1", None, Sample::now(json!({"i": 2})));
        let snap = RequestTelemetry::snapshot("k1", None);
        assert_eq!(snap.len(), 3);
        assert_eq!(snap[0].value, json!({"i": 0}));
        assert_eq!(snap[1].value, json!({"i": 1}));
        assert_eq!(snap[2].value, json!({"i": 2}));
    }

    #[test]
    #[serial]
    fn snapshot_empty_when_no_record() {
        RequestTelemetry::reset();
        let snap = RequestTelemetry::snapshot("nonexistent", None);
        assert!(snap.is_empty());
    }

    #[test]
    #[serial]
    fn ring_buffer_caps_at_128() {
        RequestTelemetry::reset();
        for i in 0..200usize {
            record("cap", None, Sample::now(json!({"i": i})));
        }
        let snap = RequestTelemetry::snapshot("cap", None);
        assert_eq!(snap.len(), 128);
        // Oldest 72 dropped; first remaining sample's "i" field == 72.
        assert_eq!(snap[0].value, json!({"i": 72}));
        assert_eq!(snap[127].value, json!({"i": 199}));
    }

    #[test]
    #[serial]
    fn scope_isolation() {
        RequestTelemetry::reset();
        record("s", Some("a"), Sample::now(json!({"side": "a"})));
        record("s", Some("b"), Sample::now(json!({"side": "b"})));
        let snap_a = RequestTelemetry::snapshot("s", Some("a"));
        let snap_b = RequestTelemetry::snapshot("s", Some("b"));
        assert_eq!(snap_a.len(), 1);
        assert_eq!(snap_b.len(), 1);
        assert_eq!(snap_a[0].value, json!({"side": "a"}));
        assert_eq!(snap_b[0].value, json!({"side": "b"}));
    }

    #[test]
    #[serial]
    fn concurrent_record_no_deadlock() {
        RequestTelemetry::reset();
        let handles: Vec<_> = (0..8u32)
            .map(|t| {
                thread::spawn(move || {
                    for i in 0..50usize {
                        record("concurrent", None, Sample::now(json!({"t": t, "i": i})));
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().expect("thread panicked");
        }
        let snap = RequestTelemetry::snapshot("concurrent", None);
        // 8 threads * 50 samples = 400 attempted; capped at 128.
        assert_eq!(snap.len(), 128);
        // Smoke test for liveness: this test must complete in under the
        // default cargo-test timeout. If it deadlocks, cargo test hangs.
        let _ = Duration::from_secs(5);
    }

    #[test]
    #[serial]
    fn reset_clears_store() {
        RequestTelemetry::reset();
        record("transient", None, Sample::now(json!({"x": 1})));
        assert_eq!(RequestTelemetry::snapshot("transient", None).len(), 1);
        RequestTelemetry::reset();
        assert!(RequestTelemetry::snapshot("transient", None).is_empty());
    }

    #[test]
    #[serial]
    fn keys_lists_all_buckets() {
        RequestTelemetry::reset();
        record("a", None, Sample::now(json!({})));
        record("b", Some("x"), Sample::now(json!({})));
        record("c", Some("y"), Sample::now(json!({})));
        let keys = RequestTelemetry::keys();
        assert_eq!(keys.len(), 3);
    }
}
