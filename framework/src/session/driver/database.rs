//! Database-backed session storage driver

use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{Condition, QueryFilter, Set};
use std::collections::HashMap;
use std::time::Duration;

use crate::database::DB;
use crate::error::FrameworkError;
use crate::session::store::{SessionData, SessionStore};

/// Database session driver using SeaORM
///
/// Stores sessions in a `sessions` table with dual timeout enforcement:
/// - Idle timeout: expires after inactivity (based on `last_activity`)
/// - Absolute timeout: expires after creation time (based on `created_at`)
///
/// Both timeouts are checked on every `read()` and enforced during `gc()`.
pub struct DatabaseSessionDriver {
    idle_lifetime: Duration,
    absolute_lifetime: Duration,
}

impl DatabaseSessionDriver {
    /// Create a new database session driver with dual timeout configuration
    pub fn new(idle_lifetime: Duration, absolute_lifetime: Duration) -> Self {
        Self {
            idle_lifetime,
            absolute_lifetime,
        }
    }
}

#[async_trait]
impl SessionStore for DatabaseSessionDriver {
    async fn read(&self, id: &str) -> Result<Option<SessionData>, FrameworkError> {
        let db = DB::connection()?;

        let result = sessions::Entity::find_by_id(id)
            .one(db.inner())
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))?;

        if let Some(session) = result {
            let now = chrono::Utc::now();

            // Check idle timeout
            let idle_expiry = session.last_activity
                + chrono::Duration::seconds(self.idle_lifetime.as_secs() as i64);
            if now > idle_expiry {
                let _ = self.destroy(id).await;
                return Ok(None);
            }

            // Check absolute timeout (skip if created_at is NULL for backward compat)
            if let Some(created) = session.created_at {
                let absolute_expiry =
                    created + chrono::Duration::seconds(self.absolute_lifetime.as_secs() as i64);
                if now > absolute_expiry {
                    let _ = self.destroy(id).await;
                    return Ok(None);
                }
            }

            // Parse the payload
            let data: HashMap<String, serde_json::Value> =
                serde_json::from_str(&session.payload).unwrap_or_default();

            Ok(Some(SessionData {
                id: session.id,
                data,
                user_id: session.user_id,
                csrf_token: session.csrf_token,
                dirty: false,
            }))
        } else {
            Ok(None)
        }
    }

    async fn write(&self, session: &SessionData) -> Result<(), FrameworkError> {
        let db = DB::connection()?;

        let payload = serde_json::to_string(&session.data)
            .map_err(|e| FrameworkError::internal(format!("Session serialize error: {e}")))?;

        let now = chrono::Utc::now();

        // Check if session exists
        let existing = sessions::Entity::find_by_id(&session.id)
            .one(db.inner())
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))?;

        if existing.is_some() {
            // Update existing session — preserve original created_at
            let update = sessions::ActiveModel {
                id: Set(session.id.clone()),
                user_id: Set(session.user_id),
                payload: Set(payload),
                csrf_token: Set(session.csrf_token.clone()),
                created_at: sea_orm::NotSet,
                last_activity: Set(now),
            };

            sessions::Entity::update(update)
                .exec(db.inner())
                .await
                .map_err(|e| FrameworkError::database(e.to_string()))?;
        } else {
            // Insert new session with created_at set to now
            let model = sessions::ActiveModel {
                id: Set(session.id.clone()),
                user_id: Set(session.user_id),
                payload: Set(payload),
                csrf_token: Set(session.csrf_token.clone()),
                created_at: Set(Some(now)),
                last_activity: Set(now),
            };

            sessions::Entity::insert(model)
                .exec(db.inner())
                .await
                .map_err(|e| FrameworkError::database(e.to_string()))?;
        }

        Ok(())
    }

    async fn destroy(&self, id: &str) -> Result<(), FrameworkError> {
        let db = DB::connection()?;

        sessions::Entity::delete_by_id(id)
            .exec(db.inner())
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))?;

        Ok(())
    }

    async fn gc(&self) -> Result<u64, FrameworkError> {
        let db = DB::connection()?;

        let now = chrono::Utc::now();
        let idle_threshold = now - chrono::Duration::seconds(self.idle_lifetime.as_secs() as i64);
        let absolute_threshold =
            now - chrono::Duration::seconds(self.absolute_lifetime.as_secs() as i64);

        // Delete sessions expired by idle OR absolute timeout
        let condition = Condition::any()
            .add(sessions::Column::LastActivity.lt(idle_threshold))
            .add(
                Condition::all()
                    .add(sessions::Column::CreatedAt.is_not_null())
                    .add(sessions::Column::CreatedAt.lt(absolute_threshold)),
            );

        let result = sessions::Entity::delete_many()
            .filter(condition)
            .exec(db.inner())
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))?;

        Ok(result.rows_affected)
    }

    async fn destroy_for_user(
        &self,
        user_id: i64,
        except_session_id: Option<&str>,
    ) -> Result<u64, FrameworkError> {
        let db = DB::connection()?;

        let mut condition = Condition::all().add(sessions::Column::UserId.eq(user_id));
        if let Some(except_id) = except_session_id {
            condition = condition.add(sessions::Column::Id.ne(except_id));
        }

        let result = sessions::Entity::delete_many()
            .filter(condition)
            .exec(db.inner())
            .await
            .map_err(|e| FrameworkError::database(e.to_string()))?;

        Ok(result.rows_affected)
    }
}

/// Sessions table entity for SeaORM
pub mod sessions {
    use sea_orm::entity::prelude::*;

    /// SeaORM model for the `sessions` database table.
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sessions")]
    pub struct Model {
        /// Unique session identifier (UUID or random token).
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        /// ID of the authenticated user, or `None` for guest sessions.
        pub user_id: Option<i64>,
        /// Serialized session payload (JSON).
        #[sea_orm(column_type = "Text")]
        pub payload: String,
        /// CSRF token bound to this session.
        pub csrf_token: String,
        /// When the session was first created.
        pub created_at: Option<chrono::DateTime<chrono::Utc>>,
        /// Timestamp of the most recent request that used this session.
        pub last_activity: chrono::DateTime<chrono::Utc>,
    }

    /// SeaORM relation enum for the sessions entity (no relations defined).
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn new_stores_both_lifetimes() {
        let idle = Duration::from_secs(7200);
        let absolute = Duration::from_secs(2_592_000);
        let driver = DatabaseSessionDriver::new(idle, absolute);
        assert_eq!(driver.idle_lifetime, idle);
        assert_eq!(driver.absolute_lifetime, absolute);
    }

    #[test]
    fn sessions_model_has_created_at() {
        // Compile-time verification: created_at field exists on Model
        let model = sessions::Model {
            id: "test".to_string(),
            user_id: None,
            payload: "{}".to_string(),
            csrf_token: "token".to_string(),
            created_at: Some(chrono::Utc::now()),
            last_activity: chrono::Utc::now(),
        };
        assert!(model.created_at.is_some());
    }

    #[test]
    fn sessions_model_created_at_nullable() {
        let model = sessions::Model {
            id: "test".to_string(),
            user_id: None,
            payload: "{}".to_string(),
            csrf_token: "token".to_string(),
            created_at: None,
            last_activity: chrono::Utc::now(),
        };
        assert!(model.created_at.is_none());
    }
}
