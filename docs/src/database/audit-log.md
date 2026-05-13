# Audit Log

Ferro's `ferro-audit` crate provides an append-only structured before/after audit log for state-changing operations. Every recorded entry captures *what happened* — for forensic investigation, regulatory evidence, and state replay. The crate ships a SeaORM migration that consumer apps register in their own `Migrator`, plus a chainable builder API for recording entries and pure functions for replaying them.

Audit entries are the *historical twin* of `ferro-events`: events are "something happened, react now"; audit entries are "something happened, here is the evidence forever".

## The Anti-Pattern

Without a structured audit log, applications either log to unstructured text (`tracing::info!`), persist ad-hoc JSON columns on every domain table, or rely on database transaction logs that disappear when the storage engine rolls. Each approach loses information: unstructured logs are hard to query, ad-hoc JSON columns drift across tables, transaction logs are operational data not preserved evidence.

```rust,ignore
// Anti-pattern: scattered logging with no structure
tracing::info!(user = %user_id, "decremented inventory unit {} by {}", unit_id, qty);

// Anti-pattern: ad-hoc JSON column on every table
inventory_unit.history.push(json!({ "action": "adjust", "qty": qty, "user": user_id }));
```

Neither approach lets you answer "what was this unit's history?" without writing custom query code per domain table, and neither preserves before/after state for replay.

## The Replacement

`ferro-audit` provides a typed, queryable audit primitive:

```rust,ignore
use ferro_audit::{AuditEntry, AuditActor, AuditTarget};
use serde_json::json;

AuditEntry::record("inventory.stock.adjust")
    .actor(AuditActor::User(user_id.to_string()))
    .target(AuditTarget::new("inventory.unit", unit_id.to_string()))
    .before(json!({ "quantity": old }))
    .after(json!({ "quantity": new }))
    .reason("order_committed")
    .write(&conn)
    .await?;
```

One row in the `audit_log` table per state change. Query helpers traverse it by target, by actor, or globally. The `reconstruct_state` helper folds the recorded `after` payloads into the current state.

## API

### `AuditEntry::record(action) -> AuditEntryBuilder`

Entry point. `action` is a dotted-namespace verb (e.g. `"inventory.stock.adjust"`, `"user.password_reset_requested"`) — required, only validation is non-empty. The builder defaults `actor` to `AuditActor::System` and all other fields to `None`.

### Builder methods (all consuming `self`, returning `Self`)

| Method | Effect |
|--------|--------|
| `.actor(AuditActor)` | Set the actor (default `AuditActor::System`) |
| `.target(AuditTarget)` | Set the target (optional; missing target emits `tracing::warn!`) |
| `.before(serde_json::Value)` | JSON snapshot of state BEFORE the action |
| `.after(serde_json::Value)` | JSON snapshot of state AFTER the action |
| `.reason(impl Into<String>)` | Free-text cause / reason |
| `.correlation(Uuid)` | Caller-supplied correlation id |
| `.tenant(impl Into<String>)` | Tenant scoping (stringly-typed) |
| `.write(&conn).await` | Persist the entry and return the `AuditEntry` |

### Query helpers

| Helper | Returns | Order |
|--------|---------|-------|
| `history_for_target(&target, &conn)` | `Vec<AuditEntry>` for the target | `created_at ASC` |
| `recent_by_actor(&actor, &conn, limit)` | `Vec<AuditEntry>` for the actor | `created_at DESC` |
| `recent(&conn, limit)` | `Vec<AuditEntry>` globally | `created_at DESC` |

For pagination or custom filters, drop down to SeaORM directly via the re-exported `AuditLogEntity`.

### Retention

| Helper | Effect |
|--------|--------|
| `prune_older_than(cutoff: NaiveDateTime, &conn)` | Deletes rows with `created_at < cutoff`; returns count |

## `AuditActor`

Typed enum, stringly-keyed so ferro doesn't bind to a consumer's user-id type:

| Variant | DB `actor_kind` | DB `actor_id` |
|---------|-----------------|---------------|
| `User(String)` | `"user"` | the contained string |
| `System` | `"system"` | NULL |
| `Job(String)` | `"job"` | the contained string (job name) |
| `ApiClient(String)` | `"api_client"` | the contained string |
| `Anonymous` | `"anonymous"` | NULL |

## `AuditTarget`

Open struct so ferro doesn't bind to a closed set of target types:

```rust,ignore
pub struct AuditTarget {
    pub kind: String,  // "inventory.unit", "user", "checkout.session"
    pub id: String,    // consumer-stringified primary key
}
```

The dotted-namespace convention for `kind` (and for `action`) is a convention, not enforced at compile time. Use it for consistency across your audit codebase.

## Schema

The `audit_log` table created by `CreateAuditLogTable`:

| Column | Type | Nullable | Notes |
|--------|------|----------|-------|
| `id` | UUID | NO | Client-generated UUIDv4 at write time |
| `tenant_id` | VARCHAR | YES | Multi-tenant scoping |
| `actor_kind` | VARCHAR | NO | snake_case enum variant name |
| `actor_id` | VARCHAR | YES | NULL for `System` / `Anonymous` |
| `action` | VARCHAR | NO | Required verb (dotted namespace) |
| `target_kind` | VARCHAR | YES | NULL for pure events |
| `target_id` | VARCHAR | YES | NULL for pure events |
| `before` | JSON | YES | Pre-state snapshot |
| `after` | JSON | YES | Post-state snapshot |
| `reason` | VARCHAR | YES | Free-text cause |
| `correlation_id` | UUID | YES | Caller-supplied request correlation |
| `created_at` | TIMESTAMP | NO | DB-stamped via `DEFAULT CURRENT_TIMESTAMP` |

Indexes:

| Name | Columns |
|------|---------|
| `idx_audit_target` | `(tenant_id, target_kind, target_id, created_at)` |
| `idx_audit_actor` | `(tenant_id, actor_kind, actor_id, created_at)` |

Register the migration in your `Migrator`:

```rust,ignore
use ferro_audit::CreateAuditLogTable;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(CreateAuditLogTable),
            // ... your app migrations
        ]
    }
}
```

## Replay

`history_for_target(&target, &conn)` returns the entries in ascending order. Pair it with `reconstruct_state` to fold the recorded `after` payloads into the current state:

```rust,ignore
use ferro_audit::{history_for_target, reconstruct_state, AuditTarget};

let target = AuditTarget::new("inventory.unit", "abc");
let entries = history_for_target(&target, &conn).await?;
let final_state = reconstruct_state(&entries);
```

The fold is a **shallow object merge**: keys from newer entries overwrite older keys at the top level. Nested objects and arrays are replaced wholesale, not deep-merged. A consumer needing deep-merge runs its own fold over the `Vec<AuditEntry>`.

Returns `None` if the slice is empty or no entry has a non-None `after`. Returns `Some(non_object_value)` if any entry's `after` is a non-object — that value replaces the running state from that point on (useful for "tombstone" patterns like recording `after: "DELETED"`).

## Retention and Pruning

Audit trails are evidence. Aggressive pruning is usually wrong. GDPR / privacy law may force retention limits; in that case, 1–3 years is the conventional default. Run `prune_older_than` from a scheduled job in your application — `ferro-queue` is the natural fit.

**PII responsibility**: `ferro-audit` does NOT automatically redact `before` / `after` payloads. The caller must remove or hash PII fields BEFORE passing JSON to the builder. The `audit_log` table is treated as a normal application table for backup and access-control purposes; design accordingly.

## Errors

`AuditError` variants:

| Variant | Display | When |
|---------|---------|------|
| `MissingAction` | `audit: action is required` | `write()` called with empty `action` |
| `Db(DbErr)` | `audit: db error: {0}` | Underlying SeaORM error (transparent forwarding) |
| `Json(serde_json::Error)` | `audit: json serialization error: {0}` | Rare; malformed `serde_json::Value` |

Missing `target` is NOT an error — it emits a `tracing::warn!` diagnostic and the write succeeds (audit must never refuse a write). Pure events (without a target) are intentional in the model.

## Postgres vs SQLite

`ferro-audit` works against both backends transparently. The `before` / `after` columns use SeaORM's `json()` column type (stored as TEXT in SQLite, native `json` in Postgres); the `id` and `correlation_id` columns use the `uuid()` column type (TEXT in SQLite, native `uuid` in Postgres). `serde_json::Value` round-trips identically on both. Tests run against in-memory SQLite for speed; production deployments on Postgres get the same semantics.
