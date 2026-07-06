//! A2UI renderer for Ferro service projections.
//!
//! Projects a [`ferro_projections::ServiceDef`] and its derived intents into
//! [A2UI](https://a2ui.org/) surfaces — flat streaming component lists with
//! JSON Pointer data bindings. Targets the A2UI v1.0 release-candidate wire
//! format; the crate is experimental and unpublished until v1.0 stable.
//!
//! # Crate boundary
//!
//! This crate is the sole home of the A2UI `Renderer` implementation.
//! `ferro-projections` owns the trait and schema types; this crate owns the
//! wire types and emission logic. It has no dependency on `ferro-json-ui`.

pub mod catalog;
pub mod component;
pub mod context;
pub mod message;
pub mod surface;
pub mod template;

pub(crate) mod actions;
mod builder;
#[cfg(test)]
pub(crate) mod test_support;

pub use catalog::CatalogTier;
pub use context::{A2uiConfig, A2uiContext, EmissionMode};
pub use message::{A2uiMessage, A2UI_MIME_TYPE};
pub use surface::{DataBinding, DataContract, SurfaceRendering};

use ferro_projections::render::Renderer;
use ferro_projections::{Error, IntentScore, ServiceDef};

/// Renders a service projection to an A2UI surface skeleton.
///
/// Dispatches on the intent selected by `ctx.base.intent_index`, resolves the
/// slot template (theme override or built-in default), and emits a
/// `createSurface` message whose data bindings are listed in the returned
/// [`DataContract`]. Returns [`Error::NoIntents`] when the index is out of
/// bounds.
pub struct A2uiRenderer;

impl Renderer for A2uiRenderer {
    type Output = SurfaceRendering;
    type Context = A2uiContext;

    fn render(
        &self,
        service: &ServiceDef,
        intents: &[IntentScore],
        ctx: &A2uiContext,
    ) -> Result<SurfaceRendering, Error> {
        builder::build(service, intents, ctx)
    }
}
