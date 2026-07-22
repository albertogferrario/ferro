//! Presence model — where a user is *right now*.
//!
//! Presence is intentionally coarse and expiring: it carries a `last_seen`
//! timestamp so stale positions can be filtered out (mitigating the battery /
//! precision / fake-location risks in the product brief).

use ferro::database::{Model as DatabaseModel, ModelMut, QueryBuilder};
use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "presences")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub lat: f64,
    pub lng: f64,
    pub last_seen: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
impl DatabaseModel for Entity {}
impl ModelMut for Entity {}

pub type Presence = Model;

impl Model {
    pub fn query() -> QueryBuilder<Entity> {
        QueryBuilder::new()
    }

    /// Latest presence for a user, if any.
    #[allow(dead_code)]
    pub async fn find_by_user(user_id: i32) -> Result<Option<Self>, ferro::FrameworkError> {
        Self::query()
            .filter(Column::UserId.eq(user_id))
            .first()
            .await
    }

    /// All presences (the map handler joins these against visible profiles).
    pub async fn all() -> Result<Vec<Self>, ferro::FrameworkError> {
        Self::query().all().await
    }
}
