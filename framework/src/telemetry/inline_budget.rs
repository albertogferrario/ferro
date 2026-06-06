//! Request-scoped inline-vs-preload decisioning.
//!
//! State lives in `Request::extensions` and dies with the request. The decision
//! body, `tracing::warn!` emission, and `Request` integration land in Plan 02.

use std::collections::{HashMap, HashSet};

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
///
/// Plan 02 wires this into `crate::http::Request::inline_budget`.
#[derive(Default)]
#[allow(dead_code)] // Plan 02 reads `cumulative` and `warned` to drive the decision.
pub(crate) struct InlineBudgetState {
    pub(crate) cumulative: HashMap<String, usize>,
    pub(crate) warned: HashSet<String>,
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
}
