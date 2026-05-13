//! `AuditEntry` — the persisted audit log row + chainable builder API.
//!
//! The builder enforces the only validation rule (`action` is required;
//! D-10, D-16) and emits a `tracing::warn!` diagnostic if `write()` is
//! called without a target (also D-10). The DB-stamped `created_at`
//! (D-22) is re-fetched after INSERT per RESEARCH Pitfall 1 / F-12 — the
//! SQLite driver does not return the default value in the INSERT response
//! for UUID-PK entities.

use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, DbErr, EntityTrait};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::actor::AuditActor;
use crate::entity;
use crate::error::AuditError;
use crate::target::AuditTarget;

/// One persisted row of the `audit_log` table.
///
/// Type-aliased to the SeaORM `Model` so query helpers can return
/// `Vec<AuditEntry>` directly without an intermediate conversion.
pub type AuditEntry = entity::Model;

impl AuditEntry {
    /// Entry point. Returns a chainable builder for an audit entry.
    ///
    /// `action` is the only required field (D-10). The builder defaults
    /// `actor` to `AuditActor::System` (D-10) and leaves every other field
    /// at `None` until set. Call `.write(&conn)` to persist.
    ///
    /// # Example
    /// ```rust,ignore
    /// use ferro_audit::{AuditEntry, AuditActor, AuditTarget};
    /// use serde_json::json;
    ///
    /// AuditEntry::record("inventory.stock.adjust")
    ///     .actor(AuditActor::User("u_42".into()))
    ///     .target(AuditTarget::new("inventory.unit", "abc"))
    ///     .before(json!({ "quantity": 5 }))
    ///     .after(json!({ "quantity": 4 }))
    ///     .write(&conn)
    ///     .await?;
    /// ```
    pub fn record(action: impl Into<String>) -> AuditEntryBuilder {
        AuditEntryBuilder {
            action: action.into(),
            actor: AuditActor::System,
            target: None,
            before: None,
            after: None,
            reason: None,
            correlation_id: None,
            tenant_id: None,
        }
    }
}

/// Chainable builder for an audit entry.
///
/// Construct via [`AuditEntry::record`]; every setter consumes the builder
/// and returns `Self`. Terminate with `.write(&conn).await?` to persist.
#[derive(Debug)]
pub struct AuditEntryBuilder {
    action: String,
    actor: AuditActor,
    target: Option<AuditTarget>,
    before: Option<JsonValue>,
    after: Option<JsonValue>,
    reason: Option<String>,
    correlation_id: Option<Uuid>,
    tenant_id: Option<String>,
}

impl AuditEntryBuilder {
    /// Set the actor for this entry. Defaults to `AuditActor::System` (D-10).
    pub fn actor(mut self, actor: AuditActor) -> Self {
        self.actor = actor;
        self
    }

    /// Set the target this entry is about. Optional but strongly recommended
    /// (without a target, `history_for_target` will not return this entry —
    /// D-10 emits a `tracing::warn!` diagnostic at `write()` time).
    pub fn target(mut self, target: AuditTarget) -> Self {
        self.target = Some(target);
        self
    }

    /// JSON snapshot of the target state BEFORE the action.
    pub fn before(mut self, before: JsonValue) -> Self {
        self.before = Some(before);
        self
    }

    /// JSON snapshot of the target state AFTER the action.
    pub fn after(mut self, after: JsonValue) -> Self {
        self.after = Some(after);
        self
    }

    /// Free-text cause / reason (e.g. `"order_committed"`).
    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Caller-supplied correlation id. Optional (D-12); future framework
    /// plumbing may populate this automatically.
    pub fn correlation(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Tenant scoping. Stringly-typed (D-13) — ferro has no first-class
    /// tenant primitive yet.
    pub fn tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// Persist the audit entry. Returns the persisted [`AuditEntry`] with
    /// the generated `id` and DB-stamped `created_at`.
    ///
    /// Errors:
    /// - [`AuditError::MissingAction`] if `action` is empty (D-10, D-16)
    /// - [`AuditError::Db`] on any SeaORM error
    ///
    /// Missing target is NOT an error (D-10) — a `tracing::warn!` diagnostic
    /// is emitted instead.
    pub async fn write<C: ConnectionTrait>(self, conn: &C) -> Result<AuditEntry, AuditError> {
        if self.action.is_empty() {
            return Err(AuditError::MissingAction);
        }

        if self.target.is_none() {
            tracing::warn!(
                action = %self.action,
                "audit entry written without a target — history_for_target will not find this entry"
            );
        }

        let new_id = Uuid::new_v4();
        let (target_kind, target_id) = match self.target {
            Some(t) => (Some(t.kind), Some(t.id)),
            None => (None, None),
        };

        let actor_kind = self.actor.kind().to_string();
        let actor_id = self.actor.id().map(|s| s.to_string());

        let active = entity::ActiveModel {
            id: Set(new_id),
            tenant_id: Set(self.tenant_id),
            actor_kind: Set(actor_kind),
            actor_id: Set(actor_id),
            action: Set(self.action),
            target_kind: Set(target_kind),
            target_id: Set(target_id),
            before: Set(self.before),
            after: Set(self.after),
            reason: Set(self.reason),
            correlation_id: Set(self.correlation_id),
            // created_at intentionally NotSet — DB default
            // `CURRENT_TIMESTAMP` fires at INSERT time (D-22).
            created_at: sea_orm::ActiveValue::NotSet,
        };

        active.insert(conn).await?;

        // RESEARCH Pitfall 1 / F-12: re-fetch by id to populate
        // DB-stamped `created_at`. SeaORM SQLite + UUID PK does not
        // return the default value in the INSERT response.
        let persisted = entity::Entity::find_by_id(new_id)
            .one(conn)
            .await?
            .ok_or_else(|| {
                AuditError::Db(DbErr::RecordNotFound(
                    "audit_log: row vanished after INSERT".to_string(),
                ))
            })?;

        Ok(persisted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use sea_orm::{Database, DatabaseConnection};
    use sea_orm_migration::prelude::*;
    use serde_json::json;

    struct TestMigrator;

    #[async_trait::async_trait]
    impl MigratorTrait for TestMigrator {
        fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
            vec![Box::new(crate::migration::Migration)]
        }
    }

    async fn fresh_db() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("connect sqlite::memory:");
        TestMigrator::up(&conn, None).await.expect("run migration");
        conn
    }

    #[tokio::test]
    async fn happy_path() {
        let conn = fresh_db().await;

        let entry = AuditEntry::record("inventory.stock.adjust")
            .actor(AuditActor::User("u_42".into()))
            .target(AuditTarget::new("inventory.unit", "abc"))
            .before(json!({ "quantity": 5 }))
            .after(json!({ "quantity": 4 }))
            .reason("order_committed")
            .write(&conn)
            .await
            .expect("write happy_path");

        assert_ne!(entry.id, Uuid::nil(), "id should be a fresh UUIDv4");
        assert_ne!(
            entry.created_at,
            NaiveDateTime::default(),
            "created_at should be DB-stamped"
        );
        assert_eq!(entry.action, "inventory.stock.adjust");
        assert_eq!(entry.actor_kind, "user");
        assert_eq!(entry.actor_id, Some("u_42".to_string()));
        assert_eq!(entry.target_kind, Some("inventory.unit".to_string()));
        assert_eq!(entry.target_id, Some("abc".to_string()));
        assert_eq!(entry.before, Some(json!({ "quantity": 5 })));
        assert_eq!(entry.after, Some(json!({ "quantity": 4 })));
        assert_eq!(entry.reason, Some("order_committed".to_string()));
    }

    #[tokio::test]
    async fn missing_action() {
        let conn = fresh_db().await;

        let err = AuditEntry::record("")
            .actor(AuditActor::System)
            .target(AuditTarget::new("inventory.unit", "abc"))
            .write(&conn)
            .await
            .expect_err("empty action must fail");

        assert!(matches!(err, AuditError::MissingAction));
        assert_eq!(err.to_string(), "audit: action is required");
    }

    #[tokio::test]
    async fn missing_target_writes() {
        let conn = fresh_db().await;

        // The tracing::warn! is fire-and-forget — we don't capture it in a
        // subscriber here (would require tracing-subscriber as dev-dep).
        // The test asserts the WRITE succeeds despite the missing target,
        // which is the load-bearing behavior contract (D-10).
        let entry = AuditEntry::record("user.password_reset_requested")
            .actor(AuditActor::User("u_42".into()))
            .write(&conn)
            .await
            .expect("write without target should succeed");

        assert_eq!(entry.target_kind, None);
        assert_eq!(entry.target_id, None);
        assert_eq!(entry.action, "user.password_reset_requested");
    }

    #[tokio::test]
    async fn json_roundtrip() {
        let conn = fresh_db().await;

        let complex = json!({
            "quantity": 42,
            "status": "reserved",
            "tags": ["urgent", "vip"],
            "nested": { "depth": 2, "items": [1, 2, 3] }
        });

        let entry = AuditEntry::record("inventory.unit.updated")
            .target(AuditTarget::new("inventory.unit", "abc"))
            .after(complex.clone())
            .write(&conn)
            .await
            .expect("write json_roundtrip");

        // Re-read via the entity directly to confirm DB round-trip.
        let read_back = entity::Entity::find_by_id(entry.id)
            .one(&conn)
            .await
            .expect("re-fetch")
            .expect("entry exists");

        assert_eq!(read_back.after, Some(complex));
    }

    #[tokio::test]
    async fn actor_null_id() {
        let conn = fresh_db().await;

        // System actor → actor_id is NULL in DB
        let sys_entry = AuditEntry::record("system.cleanup")
            .actor(AuditActor::System)
            .target(AuditTarget::new("system.task", "cleanup"))
            .write(&conn)
            .await
            .expect("write system");
        assert_eq!(sys_entry.actor_kind, "system");
        assert_eq!(sys_entry.actor_id, None);

        // Anonymous actor → actor_id is NULL in DB
        let anon_entry = AuditEntry::record("public.page_view")
            .actor(AuditActor::Anonymous)
            .target(AuditTarget::new("page", "/about"))
            .write(&conn)
            .await
            .expect("write anonymous");
        assert_eq!(anon_entry.actor_kind, "anonymous");
        assert_eq!(anon_entry.actor_id, None);

        // ApiClient with id → actor_id is the contained string
        let api_entry = AuditEntry::record("api.token.refresh")
            .actor(AuditActor::ApiClient("oauth_client_xyz".into()))
            .target(AuditTarget::new("api.client", "oauth_client_xyz"))
            .write(&conn)
            .await
            .expect("write api_client");
        assert_eq!(api_entry.actor_kind, "api_client");
        assert_eq!(api_entry.actor_id, Some("oauth_client_xyz".to_string()));
    }
}
