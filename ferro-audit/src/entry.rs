//! `AuditEntry` — the persisted audit log row + chainable builder API.
//!
//! This file is a STUB. The full builder body (`AuditEntry::record(action)`,
//! `actor`, `target`, `before`, `after`, `reason`, `correlation`, `tenant`,
//! `write(&conn)`, query helpers) lands in plan 153-04. The struct shape
//! mirrors the SeaORM entity Model (defined in `entity.rs`, plan 03).

#![allow(dead_code)]

use chrono::NaiveDateTime;
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// One row of the `audit_log` table.
///
/// Constructed only by the builder (`AuditEntry::record(action).…write()`)
/// or returned from a query helper. Plan 153-04 lands the builder body.
#[derive(Clone, Debug, PartialEq)]
pub struct AuditEntry {
    pub id: Uuid,
    pub tenant_id: Option<String>,
    pub actor_kind: String,
    pub actor_id: Option<String>,
    pub action: String,
    pub target_kind: Option<String>,
    pub target_id: Option<String>,
    pub before: Option<JsonValue>,
    pub after: Option<JsonValue>,
    pub reason: Option<String>,
    pub correlation_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
}
