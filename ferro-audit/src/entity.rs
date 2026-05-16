//! SeaORM `Entity` / `Model` / `ActiveModel` / `Column` / `Relation` for the
//! `audit_log` table (D-19).
//!
//! Schema authority is `migration.rs` (`CreateAuditLogTable`). This module's
//! `Model` shape must match the migration's column declarations exactly:
//! adding a column requires editing both files.

use sea_orm::entity::prelude::*;
use serde_json::Value as JsonValue;

/// One row of the `audit_log` table. Persisted by `AuditEntry::write` and
/// read by the query helpers (`history_for_target`, `recent_by_actor`,
/// `recent`).
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "audit_log")]
pub struct Model {
    /// Client-generated UUIDv4 — set at `write()` time so the caller can
    /// reference the entry before it hits the DB (D-21).
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Multi-tenant scoping; stringly-typed because ferro has no first-class
    /// tenant primitive (D-13).
    pub tenant_id: Option<String>,

    /// Snake_case actor kind — see `AuditActor::kind()` in `actor.rs`.
    pub actor_kind: String,

    /// Stringly-typed actor id; `NULL` for `System` and `Anonymous` (D-05).
    pub actor_id: Option<String>,

    /// Required dotted-namespace action verb (e.g. `"inventory.stock.adjust"`).
    pub action: String,

    /// Target kind — `NULL` allowed because pure events have no target (D-10).
    pub target_kind: Option<String>,

    /// Target id (consumer-stringified primary key).
    pub target_id: Option<String>,

    /// JSON snapshot of the target's state BEFORE the action. `None` for
    /// creations and pure events.
    pub before: Option<JsonValue>,

    /// JSON snapshot of the target's state AFTER the action. `None` for
    /// deletions and pure events.
    pub after: Option<JsonValue>,

    /// Free-text reason / cause (e.g. `"order_committed"`,
    /// `"stripe_webhook_payment_failed"`).
    pub reason: Option<String>,

    /// Caller-supplied correlation id; future framework plumbing may
    /// populate this automatically (D-12 — deferred to a later phase).
    pub correlation_id: Option<Uuid>,

    /// DB-stamped timestamp (D-22); set by `DEFAULT CURRENT_TIMESTAMP` and
    /// re-fetched after INSERT per RESEARCH Pitfall 1 / F-12.
    pub created_at: DateTime,
}

/// No foreign keys in v0 (D-19, D-25).
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
