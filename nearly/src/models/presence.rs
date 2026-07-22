//! Presence model — where a user is *right now*.
//!
//! Presence is intentionally coarse and expiring: it carries a `last_seen`
//! timestamp so stale positions can be filtered out (mitigating the battery /
//! precision / fake-location risks in the product brief).

use ferro::database::{Model as DatabaseModel, ModelMut, QueryBuilder};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::Serialize;

/// A presence older than this (minutes) is considered stale and hidden from
/// the map — the "coarse and expiring" property from the product brief.
pub const FRESH_TTL_MINUTES: i64 = 120;

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

    /// Whether this presence was seen within `ttl_minutes`. An unparseable
    /// timestamp is treated as fresh (never silently hide a live user).
    pub fn is_fresh(&self, ttl_minutes: i64) -> bool {
        match chrono::DateTime::parse_from_rfc3339(&self.last_seen) {
            Ok(dt) => {
                let age = chrono::Utc::now().signed_duration_since(dt.with_timezone(&chrono::Utc));
                age <= chrono::Duration::minutes(ttl_minutes)
            }
            Err(_) => true,
        }
    }

    /// Set the current user's location, inserting or refreshing their presence.
    pub async fn upsert(user_id: i32, lat: f64, lng: f64) -> Result<(), ferro::FrameworkError> {
        let now = crate::models::now();
        if let Some(existing) = Self::find_by_user(user_id).await? {
            let mut active: ActiveModel = existing.into();
            active.lat = Set(lat);
            active.lng = Set(lng);
            active.last_seen = Set(now);
            Entity::update_one(active).await?;
        } else {
            let active = ActiveModel {
                user_id: Set(user_id),
                lat: Set(lat),
                lng: Set(lng),
                last_seen: Set(now),
                ..Default::default()
            };
            Entity::insert_one(active).await?;
        }
        Ok(())
    }

    /// Refresh `last_seen` without moving — the "I'm still here" check-in.
    /// Returns whether a presence existed to touch.
    pub async fn touch(user_id: i32) -> Result<bool, ferro::FrameworkError> {
        if let Some(existing) = Self::find_by_user(user_id).await? {
            let mut active: ActiveModel = existing.into();
            active.last_seen = Set(crate::models::now());
            Entity::update_one(active).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
