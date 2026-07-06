//! Article entity — Conduit `articles` table.

use ferro::database::{Model as DatabaseModel, ModelMut};
use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "articles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub slug: String,
    pub title: String,
    pub description: String,
    #[sea_orm(column_type = "Text")]
    pub body: String,
    pub author_id: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::AuthorId",
        to = "super::user::Column::Id"
    )]
    Author,
    #[sea_orm(has_many = "super::comment::Entity")]
    Comment,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Author.def()
    }
}

impl Related<super::comment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Comment.def()
    }
}

/// Generate a unique URL slug from a title: slugified title + a random suffix.
pub fn generate_slug(title: &str) -> String {
    format!("{}-{}", slug::slugify(title), rand::random::<u32>())
}

impl ActiveModelBehavior for ActiveModel {}
impl DatabaseModel for Entity {}
impl ModelMut for Entity {}
