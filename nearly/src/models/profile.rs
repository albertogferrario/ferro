//! Profile model — a user's public identity on the map.

use ferro::database::{Model as DatabaseModel, ModelMut, QueryBuilder};
use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "profiles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub display_name: String,
    /// Short, self-authored status line shown in the pop-up.
    pub status: String,
    pub avatar_url: Option<String>,
    /// When false the profile is hidden from the map (Settings → visibility).
    pub visible: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
impl DatabaseModel for Entity {}
impl ModelMut for Entity {}

pub type Profile = Model;

impl Model {
    pub fn query() -> QueryBuilder<Entity> {
        QueryBuilder::new()
    }

    /// Find the profile belonging to a given user.
    pub async fn find_by_user(user_id: i32) -> Result<Option<Self>, ferro::FrameworkError> {
        Self::query()
            .filter(Column::UserId.eq(user_id))
            .first()
            .await
    }

    /// All profiles currently visible on the map.
    pub async fn all_visible() -> Result<Vec<Self>, ferro::FrameworkError> {
        Self::query().filter(Column::Visible.eq(true)).all().await
    }
}
