//! Request-scoped inline-vs-preload decisioning.
//!
//! State lives in `Request::extensions` and dies with the request. Cumulative
//! bytes per `key` are compared against
//! [`crate::AppConfig::inline_budget_threshold_bytes`] (default 102_400 — 100 KiB).
//! A `tracing::warn!` fires exactly once per `(key, request)` when the cumulative
//! byte count first crosses the threshold.
//!
//! Writers reach this module through [`crate::http::Request::inline_budget`].
//!
//! The state machine is implemented on `InlineBudgetState::record_and_decide`
//! (pure: no `Request` involved; `pub(crate)`, not part of the public surface).
//! The `decide` function (also `pub(crate)`) is a thin wrapper that reads the
//! threshold + route_pattern from a `&mut Request`, lazy-inits the state in
//! `Request::extensions`, then delegates to `record_and_decide`. This
//! split keeps the state machine unit-testable without constructing a synthetic
//! `Request` (`Request` has no `Default` impl — see
//! `framework/tests/action_handler.rs:47-90` for the only viable constructor
//! pattern, which requires a real TCP loopback).

use std::collections::{HashMap, HashSet};

use crate::http::Request;

/// Default cumulative-byte threshold per `key` for
/// [`crate::http::Request::inline_budget`]. 100 KiB.
///
/// Single source of truth for the default consumed by both
/// [`crate::AppConfig::from_env`] (via the `INLINE_BUDGET_BYTES` env var) and
/// the in-`decide()` fallback when `Config::get::<AppConfig>()` returns `None`
/// (i.e. unit-test contexts where no `AppConfig` is registered).
pub const DEFAULT_INLINE_BUDGET_THRESHOLD_BYTES: usize = 102_400;

/// Outcome of `crate::http::Request::inline_budget`: inline the bytes into the
/// response, or preload them via the caller-provided fallback URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Cumulative bytes for this key are still under the threshold — inline.
    Inline,
    /// Cumulative bytes for this key have crossed the threshold — preload from
    /// the caller-supplied URL.
    Preload(String),
}

/// Per-request state stored in `Request::extensions`. Tracks cumulative bytes
/// per `key` and which keys have already emitted the once-per-request warning.
#[derive(Default)]
pub(crate) struct InlineBudgetState {
    pub(crate) cumulative: HashMap<String, usize>,
    pub(crate) warned: HashSet<String>,
}

impl InlineBudgetState {
    /// Pure state-machine method: accumulate `bytes` under `key`, compare to
    /// `threshold`, decide Inline-or-Preload, and fire-once-per-key warning.
    ///
    /// No `Request` involved — this is the testable core. The thin wrapper
    /// [`decide`] handles the `Request`-side concerns (threshold lookup via
    /// `Config::get::<AppConfig>()`, `route_pattern()` capture, and lazy-init in
    /// `req.extensions`) before delegating here.
    ///
    /// Semantics: `cumulative <= threshold` returns `Inline`; `cumulative >
    /// threshold` returns `Preload(fallback_url.to_string())`. At-exact-threshold
    /// is NOT a cross.
    pub(crate) fn record_and_decide(
        &mut self,
        key: &str,
        bytes: usize,
        threshold: usize,
        fallback_url: &str,
        route_pattern: &str,
    ) -> Decision {
        // 1. Accumulate bytes for this key. saturating_add prevents arithmetic
        //    overflow on pathological (TB-scale) inputs.
        let entry = self.cumulative.entry(key.to_string()).or_insert(0);
        *entry = entry.saturating_add(bytes);
        let cumulative = *entry;

        // 2. Decision: at or below threshold → Inline; past threshold → Preload.
        if cumulative <= threshold {
            return Decision::Inline;
        }

        // 3. Fire-once warning per (key, request). `HashSet::insert` returns
        //    true on first insert, false on subsequent crosses — atomic
        //    test-and-set, no double-hash on the hot already-warned path.
        if self.warned.insert(key.to_string()) {
            tracing::warn!(
                key = %key,
                cumulative_bytes = cumulative,
                threshold_bytes = threshold,
                fallback_url = %fallback_url,
                route_pattern = %route_pattern,
                "inline_budget: threshold crossed; flipping to Preload"
            );
        }

        Decision::Preload(fallback_url.to_string())
    }
}

/// Thin Request-side wrapper. Reads `&self`-borrowing values (threshold,
/// route_pattern) FIRST, lazy-inits [`InlineBudgetState`] in `req.extensions`,
/// then delegates to [`InlineBudgetState::record_and_decide`].
///
/// Borrow-checker note: ALL `&self`-borrowing reads MUST be captured into local
/// owned values BEFORE the `&mut self` borrow on `get_mut::<InlineBudgetState>`.
/// Reordering will compile-fail (per RESEARCH Pitfall 1).
///
/// `crate::AppConfig` resolves via the pre-existing re-export at
/// `framework/src/lib.rs:60-63` (NOT added by Plan 01).
pub(crate) fn decide(req: &mut Request, key: &str, bytes: usize, fallback_url: &str) -> Decision {
    // 1. Read &self-borrowing values first.
    let threshold = crate::Config::get::<crate::AppConfig>()
        .map(|c| c.inline_budget_threshold_bytes)
        .unwrap_or(DEFAULT_INLINE_BUDGET_THRESHOLD_BYTES);
    let route_pattern = req.route_pattern().unwrap_or_default();

    // 2. Lazy-init InlineBudgetState in extensions.
    if req.get::<InlineBudgetState>().is_none() {
        req.insert(InlineBudgetState::default());
    }
    let state = req
        .get_mut::<InlineBudgetState>()
        .expect("InlineBudgetState was just inserted above");

    // 3. Delegate to the pure state machine.
    state.record_and_decide(key, bytes, threshold, fallback_url, &route_pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_enum_is_clone_and_eq() {
        let a = Decision::Inline;
        let b = a.clone();
        assert_eq!(a, b);
        let c = Decision::Preload("/x".to_string());
        let d = c.clone();
        assert_eq!(c, d);
        assert_ne!(a, c);
    }

    #[test]
    fn inline_budget_state_default_is_empty() {
        let s = InlineBudgetState::default();
        assert!(s.cumulative.is_empty());
        assert!(s.warned.is_empty());
    }

    // ----- Phase 184 Plan 02 — state-machine tests (no Request involved) -----

    #[test]
    fn decides_inline_below_threshold() {
        let mut state = InlineBudgetState::default();
        let d = state.record_and_decide("k", 50_000, 102_400, "/fb", "");
        assert_eq!(d, Decision::Inline);
        assert_eq!(*state.cumulative.get("k").unwrap(), 50_000);
        assert!(state.warned.is_empty());
    }

    #[test]
    fn decides_inline_at_exact_threshold() {
        let mut state = InlineBudgetState::default();
        let d = state.record_and_decide("k", 102_400, 102_400, "/fb", "");
        assert_eq!(d, Decision::Inline);
        assert_eq!(*state.cumulative.get("k").unwrap(), 102_400);
        assert!(state.warned.is_empty());
    }

    #[test]
    fn decides_preload_above_threshold() {
        let mut state = InlineBudgetState::default();
        let d = state.record_and_decide("k", 102_401, 102_400, "/fb", "");
        assert_eq!(d, Decision::Preload("/fb".to_string()));
        assert_eq!(*state.cumulative.get("k").unwrap(), 102_401);
        assert!(state.warned.contains("k"));
        assert_eq!(state.warned.len(), 1);
    }

    #[test]
    fn decides_preload_after_accumulation() {
        let mut state = InlineBudgetState::default();

        // 40k → Inline (cumulative 40k ≤ 102_400).
        let d1 = state.record_and_decide("k", 40_000, 102_400, "/fb", "");
        assert_eq!(d1, Decision::Inline);
        assert_eq!(*state.cumulative.get("k").unwrap(), 40_000);
        assert!(state.warned.is_empty());

        // +40k → Inline (cumulative 80k ≤ 102_400).
        let d2 = state.record_and_decide("k", 40_000, 102_400, "/fb", "");
        assert_eq!(d2, Decision::Inline);
        assert_eq!(*state.cumulative.get("k").unwrap(), 80_000);
        assert!(state.warned.is_empty());

        // +40k → Preload (cumulative 120k > 102_400, first cross).
        let d3 = state.record_and_decide("k", 40_000, 102_400, "/fb", "");
        assert_eq!(d3, Decision::Preload("/fb".to_string()));
        assert_eq!(*state.cumulative.get("k").unwrap(), 120_000);
        assert!(state.warned.contains("k"));
        assert_eq!(state.warned.len(), 1);
    }

    #[test]
    fn warn_fires_once_per_key() {
        let mut state = InlineBudgetState::default();

        // First past-threshold call: warned set gets "k".
        let d1 = state.record_and_decide("k", 200_000, 102_400, "/fb", "");
        assert!(matches!(d1, Decision::Preload(_)));
        assert!(state.warned.contains("k"));
        assert_eq!(state.warned.len(), 1);

        // Second past-threshold call for the same key: warned set is unchanged.
        // This is the SC-2 contract — the fire-once guard works regardless of
        // whether tracing actually emitted (state machine is the source of truth).
        let d2 = state.record_and_decide("k", 1, 102_400, "/fb", "");
        assert!(matches!(d2, Decision::Preload(_)));
        assert!(state.warned.contains("k"));
        assert_eq!(state.warned.len(), 1); // still 1 — NOT re-modified.
    }

    #[test]
    fn warn_independent_per_key() {
        let mut state = InlineBudgetState::default();

        // Key "key_a" crosses → warned += "key_a".
        let _ = state.record_and_decide("key_a", 200_000, 102_400, "/fb-a", "");
        // Key "key_b" crosses → warned += "key_b".
        let _ = state.record_and_decide("key_b", 200_000, 102_400, "/fb-b", "");

        assert!(state.warned.contains("key_a"));
        assert!(state.warned.contains("key_b"));
        assert_eq!(state.warned.len(), 2);
    }
}
