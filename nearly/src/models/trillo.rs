//! Trillo model — a wordless ping from one user to another.
//!
//! A trillo has **no message body**. That is the whole point of Nearly: the only
//! resolution is to meet in person. A trillo moves through three states:
//! `pending` → (`accepted` | `declined`).

use ferro::database::{Model as DatabaseModel, ModelMut, QueryBuilder};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::Serialize;

/// Lifecycle states for a trillo.
pub const STATUS_PENDING: &str = "pending";
pub const STATUS_ACCEPTED: &str = "accepted";
pub const STATUS_DECLINED: &str = "declined";

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "trillos")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub from_user_id: i32,
    pub to_user_id: i32,
    /// One of `pending` | `accepted` | `declined`.
    pub status: String,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
impl DatabaseModel for Entity {}
impl ModelMut for Entity {}

pub type Trillo = Model;

impl Model {
    pub fn query() -> QueryBuilder<Entity> {
        QueryBuilder::new()
    }

    /// Send a trillo. Returns the inserted, pending row.
    pub async fn send(from_user_id: i32, to_user_id: i32) -> Result<Self, ferro::FrameworkError> {
        let model = ActiveModel {
            from_user_id: Set(from_user_id),
            to_user_id: Set(to_user_id),
            status: Set(STATUS_PENDING.to_string()),
            created_at: Set(crate::models::now()),
            ..Default::default()
        };
        Entity::insert_one(model).await
    }

    /// Trilli received by a user, newest-first is applied by the caller.
    pub async fn inbox(user_id: i32) -> Result<Vec<Self>, ferro::FrameworkError> {
        Self::query()
            .filter(Column::ToUserId.eq(user_id))
            .all()
            .await
    }

    /// Update a trillo's status (accept / decline).
    pub async fn set_status(id: i32, status: &str) -> Result<(), ferro::FrameworkError> {
        if let Some(row) = Entity::find_by_pk(id).await? {
            let mut active: ActiveModel = row.into();
            active.status = Set(status.to_string());
            Entity::update_one(active).await?;
        }
        Ok(())
    }
}
