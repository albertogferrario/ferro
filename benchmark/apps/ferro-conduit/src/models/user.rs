//! User entity — Conduit `users` table.

use ferro::database::{Model as DatabaseModel, ModelMut};
use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub email: String,
    #[sea_orm(unique)]
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub password: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::article::Entity")]
    Article,
    #[sea_orm(has_many = "super::comment::Entity")]
    Comment,
}

impl Related<super::article::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Article.def()
    }
}

impl Related<super::comment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Comment.def()
    }
}

impl Model {
    /// Hash `plain` (bcrypt via `ferro::hashing`) and store it on `password`.
    pub fn set_password(&mut self, plain: &str) -> Result<(), ferro::FrameworkError> {
        self.password = ferro::hashing::hash(plain)?;
        Ok(())
    }

    /// Constant-time bcrypt verify against the stored hash. Returns false on any error.
    pub fn verify_password(&self, plain: &str) -> bool {
        ferro::hashing::verify(plain, &self.password).unwrap_or(false)
    }
}

impl ActiveModelBehavior for ActiveModel {}
impl DatabaseModel for Entity {}
impl ModelMut for Entity {}
