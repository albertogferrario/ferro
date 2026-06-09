//! Book model.
//!
//! A book sourced from an external catalog (Open Library for universal
//! metadata, Project Gutenberg for public-domain titles with downloadable
//! files) and kept in the user's own collection with a reading `status`.

pub use super::entities::books::*;

use sea_orm::ColumnTrait;

/// Convenient alias.
#[allow(dead_code)]
pub type Book = Model;

impl Model {
    /// Find a saved book by its catalog provenance, to avoid duplicate imports.
    pub async fn find_by_source(
        source: &str,
        source_id: &str,
    ) -> Result<Option<Self>, ferro::FrameworkError> {
        Self::query()
            .filter(Column::Source.eq(source))
            .filter(Column::SourceId.eq(source_id))
            .first()
            .await
    }
}
