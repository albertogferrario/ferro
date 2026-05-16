//! `ProjectionListener<P>` — internal `ferro_events::Listener<P::Event>`
//! adapter that wires `register()` into `global_dispatcher`.
//!
//! `pub(crate)` only (Claude's Discretion — implementation detail of
//! `ProjectionRuntime::register`). Consumers don't construct one
//! directly; `register` builds and registers it.

use std::sync::Arc;

use crate::projection::Projection;
use crate::runtime::ProjectionRuntime;

pub(crate) struct ProjectionListener<P: Projection> {
    pub(crate) runtime: Arc<ProjectionRuntime<P>>,
}

#[async_trait::async_trait]
impl<P: Projection> ferro_events::Listener<P::Event> for ProjectionListener<P> {
    async fn handle(&self, event: &P::Event) -> Result<(), ferro_events::Error> {
        self.runtime.apply_event(event).await.map_err(|e| {
            ferro_events::Error::listener_failed(std::any::type_name::<Self>(), e.to_string())
        })
    }
}
