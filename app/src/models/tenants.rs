//! Tenant model

pub use super::entities::tenants::*;
use sea_orm::ColumnTrait;

#[allow(dead_code)]
pub type Tenant = Model;

impl Model {
    pub async fn find_by_slug(slug: &str) -> Result<Option<Self>, ferro::FrameworkError> {
        Self::query().filter(Column::Slug.eq(slug)).first().await
    }

    pub async fn find_by_id(id: i64) -> Result<Option<Self>, ferro::FrameworkError> {
        Self::query().filter(Column::Id.eq(id)).first().await
    }
}
