//! User model — account identity.

use ferro::database::{Model as DatabaseModel, ModelMut, QueryBuilder};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
impl DatabaseModel for Entity {}
impl ModelMut for Entity {}

/// Convenient alias.
pub type User = Model;

impl Model {
    /// Start a query builder.
    pub fn query() -> QueryBuilder<Entity> {
        QueryBuilder::new()
    }

    /// Find a user by email.
    pub async fn find_by_email(email: &str) -> Result<Option<Self>, ferro::FrameworkError> {
        Self::query().filter(Column::Email.eq(email)).first().await
    }

    /// Verify a plaintext password against the stored hash.
    pub fn verify_password(&self, password: &str) -> Result<bool, ferro::FrameworkError> {
        ferro::hashing::verify(password, &self.password)
    }

    /// Create a user with a hashed password. Returns the inserted row.
    pub async fn create(
        name: impl Into<String>,
        email: impl Into<String>,
        password: &str,
    ) -> Result<Self, ferro::FrameworkError> {
        let hashed = ferro::hashing::hash(password)?;
        let now = crate::models::now();
        let model = ActiveModel {
            name: Set(name.into()),
            email: Set(email.into()),
            password: Set(hashed),
            created_at: Set(now.clone()),
            updated_at: Set(now),
            ..Default::default()
        };
        Entity::insert_one(model).await
    }
}
