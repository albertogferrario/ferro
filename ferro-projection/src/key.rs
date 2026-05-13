//! `ProjectionKey` — opaque stringly-typed identifier for a projection
//! row (D-11).
//!
//! Newtype around `String`. The stringly-typed convention matches
//! `ferro_audit::AuditTarget::id: String` (Phase 153 D-07) and the
//! broader project-agnostic crate principle (CLAUDE.md §Architecture
//! Principles). A typed `Key` per projection would force the runtime
//! to carry a generic key parameter through the broadcast channel
//! name, the DB column, and the per-key Mutex map — adding noise for
//! no benefit.
//!
//! Consumers construct via [`ProjectionKey::new`] (preferred) or via
//! `From<String>` / `From<&str>`. Multi-tenancy lives inside the key
//! string by convention: `"tenant-7:inventory.warehouse-a"`,
//! `"checkout.user-42.cart"`. The runtime does NOT auto-scope by
//! tenant.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Opaque stringly-typed identifier (D-11).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectionKey(pub(crate) String);

impl ProjectionKey {
    /// Construct from any string-like input.
    ///
    /// ```
    /// use ferro_projection::ProjectionKey;
    /// let k = ProjectionKey::new("warehouse-a");
    /// assert_eq!(k.as_str(), "warehouse-a");
    /// ```
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Borrow the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProjectionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for ProjectionKey {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ProjectionKey {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_as_str_round_trip() {
        let k = ProjectionKey::new("warehouse-a");
        assert_eq!(k.as_str(), "warehouse-a");

        let k2: ProjectionKey = "warehouse-b".into();
        assert_eq!(k2.as_str(), "warehouse-b");

        let k3 = ProjectionKey::from(String::from("warehouse-c"));
        assert_eq!(k3.as_str(), "warehouse-c");
    }

    #[test]
    fn display_renders_inner_string() {
        let k = ProjectionKey::new("warehouse-a");
        assert_eq!(format!("{k}"), "warehouse-a");
    }

    #[test]
    fn serde_round_trip_via_json() {
        let k = ProjectionKey::new("warehouse-a");
        let s = serde_json::to_string(&k).expect("serialize");
        assert_eq!(s, "\"warehouse-a\"");
        let parsed: ProjectionKey =
            serde_json::from_str(&s).expect("deserialize");
        assert_eq!(parsed, k);
    }
}
