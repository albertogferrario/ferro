//! `ProjectionKey` — opaque stringly-typed identifier for a projection
//! row (D-11).
//!
//! This file is a STUB. Plan 155-04 adds the full impl block
//! (`as_str`, `Display`, `From<String>`, `From<&str>`) and the round-trip
//! unit tests (D-45 #1).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Opaque stringly-typed identifier (D-11). Newtype around `String` for
/// type-safety; mirrors `ferro_audit::AuditTarget::id: String` convention.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectionKey(pub(crate) String);

impl ProjectionKey {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}
