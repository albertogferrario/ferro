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
}

/// Sessions table entity for SeaORM
pub mod sessions {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sessions")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub user_id: Option<i64>,
        #[sea_orm(column_type = "Text")]
        pub payload: String,
        pub csrf_token: String,
        pub created_at: Option<chrono::DateTime<chrono::Utc>>,
        pub last_activity: chrono::DateTime<chrono::Utc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
