//! `ReservationContext` — per-call audit metadata bundle.
//!
//! This file is a STUB. Plan 154-04 lands the constructors (`system`,
//! `user`, `job`, `anonymous`) and the `with_*` consuming builder methods
//! (`with_correlation`, `with_tenant`, `with_reason`) per D-29.

#![allow(dead_code)]

use ferro_audit::AuditActor;
use uuid::Uuid;

/// Per-call audit metadata bundle (D-29).
///
/// Plan 154-04 adds the `system()` / `user(id)` / `job(name)` /
/// `anonymous()` constructors and the consuming `with_*` builder methods.
#[derive(Clone, Debug)]
pub struct ReservationContext {
    pub actor: AuditActor,
    pub correlation_id: Option<Uuid>,
    pub tenant_id: Option<String>,
    pub reason: Option<String>,
}
