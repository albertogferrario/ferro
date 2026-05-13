//! `reconstruct_state` — pure function folding `before → after` JSON diffs
//! back into the current state (D-24, the "replay" primitive).
//!
//! This file is a STUB. Plan 153-05 lands the shallow-merge body.

#![allow(dead_code)]

use serde_json::Value;

use crate::entry::AuditEntry;

/// Placeholder. Plan 153-05 replaces this with the shallow-merge fold over
/// `entries[i].after`. Returns `None` until that lands.
pub fn reconstruct_state(_entries: &[AuditEntry]) -> Option<Value> {
    None
}
