//! `ProjectionRuntime<P>` — orchestrator for the projection runtime.
//!
//! This file is a STUB. Plan 155-05 lands `apply_event`, `read`,
//! `read_required`, and `register`. Plan 155-06 lands `rebuild`.

#![allow(dead_code)]

use dashmap::DashMap;
use std::sync::Arc;

use crate::projection::Projection;

/// Live read-model runtime owning the DB connection, the broadcaster
/// handle, the projection impl, and the per-key Mutex registry (D-13).
pub struct ProjectionRuntime<P: Projection> {
    pub(crate) db: sea_orm::DatabaseConnection,
    pub(crate) broadcaster: Arc<ferro_broadcast::Broadcaster>,
    pub(crate) projection: P,
    pub(crate) locks: DashMap<String, Arc<tokio::sync::Mutex<()>>>,
}

impl<P: Projection> ProjectionRuntime<P> {
    pub fn new(
        db: sea_orm::DatabaseConnection,
        broadcaster: Arc<ferro_broadcast::Broadcaster>,
        projection: P,
    ) -> Self {
        Self {
            db,
            broadcaster,
            projection,
            locks: DashMap::new(),
        }
    }
}
