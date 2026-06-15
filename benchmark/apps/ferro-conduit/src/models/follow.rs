//! follows junction — composite PK (follower_id, followed_id), self-referential to users.
//!
//! Per RESEARCH line 202, follow checks query this junction directly
//! (`SELECT WHERE follower_id = ? AND followed_id = ?`) rather than via the
//! `Linked` trait, so no explicit user Relation entries are declared here.

use ferro::database::{Model as DatabaseModel, ModelMut};
use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "follows")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub follower_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub followed_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
impl DatabaseModel for Entity {}
impl ModelMut for Entity {}
