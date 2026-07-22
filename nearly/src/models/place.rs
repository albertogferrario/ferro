//! Place model — a venue on the map (trend + premium).

use ferro::database::{Model as DatabaseModel, ModelMut, QueryBuilder};
use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "places")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub category: String,
    pub lat: f64,
    pub lng: f64,
    /// Premium venues stay visible next to the organic trend area.
    pub premium: bool,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
impl DatabaseModel for Entity {}
impl ModelMut for Entity {}

pub type Place = Model;

impl Model {
    pub fn query() -> QueryBuilder<Entity> {
        QueryBuilder::new()
    }

    /// Every place, for the map + places list.
    pub async fn all() -> Result<Vec<Self>, ferro::FrameworkError> {
        Self::query().all().await
    }
}
