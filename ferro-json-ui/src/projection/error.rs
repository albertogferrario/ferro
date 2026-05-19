//! Error type for the schema-driven projection pipeline.
//!
//! `ProjectionError` is the Result type returned by `Spec::from_service_def`
//! and the intent-layout dispatch. It wraps upstream `CatalogError` from
//! catalog validation (D-06) and `SpecError` from structural spec building.

use thiserror::Error;

use crate::catalog::CatalogError;
use crate::spec::SpecError;

/// Errors returned by the schema-driven projection pipeline.
#[derive(Debug, Error)]
pub enum ProjectionError {
    /// `ctx.intent_index` is past the end of the supplied `intents` slice.
    #[error("intent_index {requested} out of bounds (have {available} intents)")]
    IntentIndexOutOfBounds { requested: usize, available: usize },

    /// Caller supplied an empty intents slice — no projection target exists.
    #[error("cannot project service with no intents")]
    EmptyIntents,

    /// The projector produced an element referencing a component name not
    /// present in the built-in or plugin catalog. Caught before catalog
    /// validation; indicates a bug in `MEANING_COMPONENT_TABLE` or
    /// `RELATIONSHIP_COMPONENT_TABLE`.
    #[error("projector referenced unknown component '{type_name}'")]
    UnknownComponent { type_name: String },

    /// The generated spec failed `Catalog::validate` — the projector and the
    /// catalog are inconsistent. In `cfg(debug_assertions)` builds, this
    /// condition additionally panics (see `Spec::from_service_def`).
    #[error("catalog validation failed: {}", format_catalog_errors(.0))]
    CatalogValidation(Vec<CatalogError>),

    /// `Spec::builder().build()` rejected the assembled spec (cycle, depth,
    /// dangling reference, or ID-format violation).
    #[error("spec build failed: {0}")]
    SpecBuild(#[from] SpecError),
}

fn format_catalog_errors(errors: &[CatalogError]) -> String {
    errors
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("; ")
}
