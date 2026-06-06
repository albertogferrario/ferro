//! Request-scoped telemetry primitives.
//!
//! Two primitives live in this module:
//!
//! 1. **InlineBudget** ([`Decision`]) — request-scoped accumulator that decides
//!    whether a chunk of bytes should be inlined into the HTML response or
//!    preloaded via `<link rel=preload>`. State lives in `Request::extensions`
//!    and dies with the request. Cumulative byte counts per `key` are compared
//!    against [`crate::AppConfig::inline_budget_threshold_bytes`] (default
//!    102_400 — 100 KiB; override via env `INLINE_BUDGET_BYTES`). A
//!    `tracing::warn!` fires exactly once per `(key, request)` when the threshold
//!    is first crossed.
//!
//! 2. **RequestTelemetry** ([`RequestTelemetry`], [`Sample`]) — per-key in-process
//!    ring buffer keyed by `(key, scope)`. Writers call
//!    `crate::http::Request::telemetry_record`; readers call
//!    [`RequestTelemetry::snapshot`]. Process-global storage bounded at 128
//!    samples per `(key, scope)`; oldest dropped on overflow. **Lost on process
//!    restart** — long histories belong in external systems (Prometheus,
//!    OpenTelemetry, custom DB sinks).
//!
//! Both primitives are unconditionally available; no feature flag.
//!
//! Consumers MUST treat `key` and `scope` as a controlled vocabulary owned by
//! handler code. User-controlled strings must NOT be passed directly — the
//! number of distinct `(key, scope)` pairs is bounded only by caller discipline.

pub mod inline_budget;
pub mod request_telemetry;

pub use inline_budget::Decision;
pub use request_telemetry::{RequestTelemetry, Sample};
