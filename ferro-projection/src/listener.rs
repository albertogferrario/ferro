//! `ProjectionListener<P>` — internal `ferro_events::Listener<P::Event>`
//! adapter that wires `register()` into `global_dispatcher`.
//!
//! This file is a STUB. Plan 155-05 lands the full impl block with
//! `impl<P: Projection> ferro_events::Listener<P::Event> for ProjectionListener<P>`.
//!
//! NOTE: `ProjectionListener<P>` is `pub(crate)` only — implementation
//! detail of `ProjectionRuntime::register`. Consumers don't construct
//! one directly.

#![allow(dead_code)]

use std::sync::Arc;

use crate::projection::Projection;
use crate::runtime::ProjectionRuntime;

pub(crate) struct ProjectionListener<P: Projection> {
    pub(crate) runtime: Arc<ProjectionRuntime<P>>,
}
