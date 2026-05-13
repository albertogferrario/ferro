//! `SweepReport` + `ReservationKernel::run_sweep_once` (D-21..D-24).
//!
//! This file is a STUB. Plan 154-06 lands the `run_sweep_once` impl on
//! `ReservationKernel<R>` (uses `self.db` owned connection; emits
//! `ReservationEvent::Expired` + `AuditEntry` with `AuditActor::System`
//! for each transitioned row; uses `exec_at_most_one` for concurrent-
//! sweeper idempotency).

#![allow(dead_code)]

use chrono::{DateTime, Utc};

/// Result of one sweeper invocation (D-21). Consumers typically log
/// this for observability; high `expired_count` values indicate a
/// sweep backlog and a need to schedule sweeps more frequently.
#[derive(Clone, Debug)]
pub struct SweepReport {
    pub expired_count: u32,
    pub scanned_at: DateTime<Utc>,
}
