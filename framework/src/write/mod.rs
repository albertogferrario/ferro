//! Channel-agnostic transition-execution kernel.
//!
//! Defines [`WriteDispatcher`], [`ExecutorFn`], [`GuardEvaluatorFn`], and
//! [`OverrideFn`] — the app-registered callback set that makes write actions
//! executable — and [`dispatch_write`], the single execution pipeline
//! (guard re-eval → idempotency → confirm seam → persist → audit → override).
//!
//! This kernel was relocated out of `ferro-mcp-server` so more than one channel
//! (MCP, visual/form, …) can drive the same guarded write path. It owns its own
//! [`WriteError`] (it does NOT depend on any channel's error type); each channel
//! maps `WriteError` into its own error at the framing boundary. The audit reason
//! prefix is parameterized by a `channel` argument — the kernel never hardcodes a
//! channel name.

use ferro_audit::{AuditActor, AuditEntry, AuditTarget};
use ferro_projections::ActionDef;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;

// ── Kernel-local error ───────────────────────────────────────────────────────

/// Errors returned by the transition-execution kernel.
///
/// Self-contained: the kernel does not depend on any channel's error type. Each
/// channel that drives [`dispatch_write`] maps these variants into its own error
/// at the framing boundary (e.g. `From<WriteError> for ferro_mcp_server::Error`).
#[derive(Error, Debug)]
pub enum WriteError {
    /// A database operation failed. May contain SQL fragments / table or column
    /// names — channels MUST redact this before returning it to an untrusted caller.
    #[error("database error: {0}")]
    Database(String),
    /// JSON (de)serialization of an idempotency result or persist payload failed.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// A precondition or transition guard returned false or errored at execution
    /// time. Never discloses which guard or what state it checked.
    #[error("guard failed: {0}")]
    GuardFailed(String),
    /// Input validation failed (required field missing, wrong type, etc.).
    #[error("validation error: {0}")]
    Validation(String),
    /// The resolved action name is not found in any exposed `ServiceDef`.
    #[error("action not found: {0}")]
    ActionNotFound(String),
    /// A destructive action was called without a valid confirmation context.
    /// Feature-gated: only reachable when the `confirmation` feature is enabled.
    #[cfg(feature = "confirmation")]
    #[error("confirmation required for action: {0}")]
    ConfirmationRequired(String),
}

/// Result alias for the transition-execution kernel.
pub type WriteResult<T> = Result<T, WriteError>;

// ── Boxed-future callback types (no async-trait dep) ─────────────────────────

/// App-registered write executor.
///
/// Called by [`dispatch_write`] after guards pass and idempotency is checked.
/// Receives the action name, validated inputs, the authenticated tenant id
/// (never from the call payload), and a DB connection.
/// Returns a JSON `Value` that becomes the `result` payload in the structured
/// channel response.
///
/// # Audit contract
///
/// The returned `Value` is stored **verbatim** in the append-only `audit_log`
/// table via [`ferro_audit::AuditEntry`]. It MUST NOT contain secrets,
/// credentials, PII, or any field that should not appear in a forensic log
/// readable by audit reviewers. Executors are responsible for returning only
/// audit-safe fields (typically identifiers and status values, e.g.
/// `{"id": 42, "status": "approved"}`). A full PII scrub at the
/// [`dispatch_write`] call site is not performed; the executor is the enforcement
/// point.
pub type ExecutorFn = Box<
    dyn Fn(
            &str,   // action_name
            &Value, // validated inputs
            i64,    // tenant_id (from auth, never from payload)
            &DatabaseConnection,
        ) -> Pin<Box<dyn Future<Output = WriteResult<Value>> + Send>>
        + Send
        + Sync,
>;

/// App-registered guard evaluator.
///
/// Called once per precondition name in `action.preconditions` BEFORE the
/// executor runs. Must read LIVE DB state — never `ctx.evaluated_guards`.
/// Returns `Ok(true)` to allow, `Ok(false)` or `Err(_)` to deny.
pub type GuardEvaluatorFn = Box<
    dyn Fn(
            &str,   // guard_name
            i64,    // tenant_id
            &Value, // validated inputs (for record-scoped guards)
            &DatabaseConnection,
        ) -> Pin<Box<dyn Future<Output = WriteResult<bool>> + Send>>
        + Send
        + Sync,
>;

/// App-registered post-persist override hook (EXEC-03).
///
/// Runs AFTER the derived/base persist, inside [`dispatch_write`]. Receives the
/// action name, validated inputs, the authenticated tenant id, a DB connection,
/// and the base persist result — so it can chain related-record writes keyed off
/// the just-persisted state.
///
/// Because it runs strictly after the guarded base persist, an override cannot
/// suppress the base guard or transition (threat T-231-05, mitigated).
///
/// # Ordering vs. audit/idempotency (WR-01)
///
/// The override runs AFTER the base persist's idempotency key and audit entry
/// are sealed (steps 5–6 in [`dispatch_write`]). There is no surrounding
/// transaction, so an override that returns `Err` does NOT roll back the base
/// persist — and the base persist's audit entry and idempotency key remain
/// committed. An override author must therefore treat the base transition as
/// already durable when the hook fires: a failing override surfaces its error to
/// the caller, but the base write stays applied (and audited).
pub type OverrideFn = Box<
    dyn Fn(
            &str,   // action_name
            &Value, // validated inputs
            i64,    // tenant_id (from auth, never from payload)
            &DatabaseConnection,
            &Value, // base persist result
        ) -> Pin<Box<dyn Future<Output = WriteResult<()>> + Send>>
        + Send
        + Sync,
>;

/// Holds the app-registered write callback, guard evaluator, and optional
/// post-persist override hooks.
///
/// Constructed by the consumer app and passed to the channel framing layer,
/// which threads it into [`dispatch_write`].
pub struct WriteDispatcher {
    /// The write executor (base persist).
    pub executor: ExecutorFn,
    /// The live-state guard evaluator.
    pub guard_evaluator: GuardEvaluatorFn,
    /// Optional app-specific post-persist side effects, keyed by action name.
    /// An absent key is the common path (declaration-only) — the override seam
    /// adds nothing to a write whose action has no registered hook.
    pub overrides: std::collections::HashMap<String, OverrideFn>,
}

impl WriteDispatcher {
    /// Construct a dispatcher with an empty override registry.
    pub fn new(executor: ExecutorFn, guard_evaluator: GuardEvaluatorFn) -> Self {
        Self {
            executor,
            guard_evaluator,
            overrides: std::collections::HashMap::new(),
        }
    }

    /// Register a post-persist override hook for `action` (consuming builder).
    pub fn with_override(mut self, action: impl Into<String>, hook: OverrideFn) -> Self {
        self.overrides.insert(action.into(), hook);
        self
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build the live guard set for a write: `preconditions` followed by the
/// transition-level guard (if any), deduplicated by name with order preserved.
///
/// A guard appearing on BOTH the action precondition and the transition guard
/// — as `is_manager` does on the order projection — is evaluated exactly once
/// (EXEC-02). The common case (no transition guard) returns `preconditions`
/// unchanged, keeping the non-transition path back-compatible.
///
/// Exposed so channel framing (e.g. the MCP confirm pre-check) evaluates the
/// SAME guard union as [`dispatch_write`].
pub fn merged_guards(preconditions: &[String], transition_guard: Option<&str>) -> Vec<String> {
    let mut guards: Vec<String> = preconditions.to_vec();
    if let Some(g) = transition_guard {
        if !guards.iter().any(|existing| existing == g) {
            guards.push(g.to_string());
        }
    }
    guards
}

/// Look up a stored idempotency result scoped by BOTH tenant_id AND idempotency_key.
///
/// Cross-tenant replay is prevented at the SQL level: the WHERE clause requires
/// BOTH columns, matching the UNIQUE index on `(tenant_id, idempotency_key)`.
async fn lookup_idempotency(
    tenant_id: i64,
    key: &str,
    db: &DatabaseConnection,
) -> WriteResult<Option<Value>> {
    let backend = db.get_database_backend();
    let (sql, values) = match backend {
        DatabaseBackend::Postgres => (
            "SELECT result FROM mcp_idempotency_keys WHERE tenant_id = $1 AND idempotency_key = $2"
                .to_string(),
            vec![
                sea_orm::Value::BigInt(Some(tenant_id)),
                sea_orm::Value::String(Some(Box::new(key.to_string()))),
            ],
        ),
        _ => (
            "SELECT result FROM mcp_idempotency_keys WHERE tenant_id = ? AND idempotency_key = ?"
                .to_string(),
            vec![
                sea_orm::Value::BigInt(Some(tenant_id)),
                sea_orm::Value::String(Some(Box::new(key.to_string()))),
            ],
        ),
    };
    let stmt = Statement::from_sql_and_values(backend, &sql, values);
    match db
        .query_one(stmt)
        .await
        .map_err(|e| WriteError::Database(e.to_string()))?
    {
        None => Ok(None),
        Some(row) => {
            let json_text: String = row
                .try_get("", "result")
                .map_err(|e| WriteError::Database(e.to_string()))?;
            let value: Value = serde_json::from_str(&json_text)
                .map_err(|e| WriteError::Database(e.to_string()))?;
            Ok(Some(value))
        }
    }
}

/// Store an idempotency result scoped by (tenant_id, idempotency_key).
///
/// Uses INSERT OR IGNORE (SQLite) / ON CONFLICT DO NOTHING (Postgres) for
/// concurrency safety — a second concurrent identical request will not cause
/// a UNIQUE constraint error.
async fn store_idempotency(
    tenant_id: i64,
    key: &str,
    result: &Value,
    db: &DatabaseConnection,
) -> WriteResult<()> {
    let backend = db.get_database_backend();
    let json_text = serde_json::to_string(result).map_err(WriteError::Serialization)?;

    let (sql, values) = match backend {
        DatabaseBackend::Postgres => (
            "INSERT INTO mcp_idempotency_keys (tenant_id, idempotency_key, result, created_at) \
             VALUES ($1, $2, $3, NOW()) ON CONFLICT (tenant_id, idempotency_key) DO NOTHING"
                .to_string(),
            vec![
                sea_orm::Value::BigInt(Some(tenant_id)),
                sea_orm::Value::String(Some(Box::new(key.to_string()))),
                sea_orm::Value::String(Some(Box::new(json_text))),
            ],
        ),
        _ => (
            "INSERT OR IGNORE INTO mcp_idempotency_keys \
             (tenant_id, idempotency_key, result) VALUES (?, ?, ?)"
                .to_string(),
            vec![
                sea_orm::Value::BigInt(Some(tenant_id)),
                sea_orm::Value::String(Some(Box::new(key.to_string()))),
                sea_orm::Value::String(Some(Box::new(json_text))),
            ],
        ),
    };
    let stmt = Statement::from_sql_and_values(backend, &sql, values);
    db.execute(stmt)
        .await
        .map_err(|e| WriteError::Database(e.to_string()))?;
    Ok(())
}

// ── Core pipeline ─────────────────────────────────────────────────────────────

/// Execute a write action with guard re-evaluation, idempotency, and audit.
///
/// Pipeline order (D-07):
/// 1. Guard re-evaluation (D-02) — LIVE state via `dispatcher.guard_evaluator`.
///    CRITICAL: `ctx.evaluated_guards` is the 218 list-time visibility cache and
///    is NEVER consulted here. Authorization at call time must use live DB state.
/// 2. Idempotency check (D-04) — replay stored result without re-executing.
/// 3. D-08 confirmation seam — gate for destructive actions (Phase 220).
/// 4. Execute callback (D-01).
/// 5. Store idempotency result (D-04).
/// 6. Audit via ferro-audit (D-05).
/// 7. Post-persist override hook (EXEC-03).
///
/// The `channel` argument is the audit reason prefix: the success-path audit
/// records `format!("{channel}.action.{name}")`. Each channel passes its own
/// literal (e.g. MCP passes `"mcp"`, the visual surface passes `"web"`), so a
/// write is never auditable without a channel tag.
///
/// WR-01: steps 5–6 (idempotency-store + audit of the base persist) run BEFORE
/// the override hook (step 7). There is no surrounding transaction, so the base
/// persist (step 4) has already committed by the time the override runs; sealing
/// its idempotency key and audit entry first guarantees a committed base
/// transition is always recorded and never re-executable, even if the
/// app-specific override side effect later fails.
///
/// The `is_confirmed` parameter (confirmation feature only) signals that this
/// call came from the channel's confirm handler after token validation — the
/// D-08 seam is bypassed when `true`. Bare callers always pass `false` (or omit
/// when the feature is off).
#[allow(clippy::too_many_arguments)]
pub async fn dispatch_write(
    action: &ActionDef,
    inputs: &Value,
    tenant_id: i64,
    db: &DatabaseConnection,
    dispatcher: &WriteDispatcher,
    transition_guard: Option<&str>,
    channel: &str,
    #[cfg(feature = "confirmation")] is_confirmed: bool,
) -> WriteResult<Value> {
    // 1. Guard re-evaluation (D-02, T-219-02 — load-bearing security gate).
    //
    // Calls the app-registered GuardEvaluatorFn for EVERY guard in the UNION of
    // action.preconditions and the transition-level guard (EXEC-02), deduped by
    // name, against LIVE DB state. Fail-closed: a guard returning Ok(false) OR
    // any Err immediately returns Err(GuardFailed).
    //
    // IMPORTANT: ctx.evaluated_guards (the 218 list-time visibility cache) is
    // intentionally NOT consulted here. An agent may bypass tools/list entirely
    // and call tools/call directly — only this live re-evaluation prevents the
    // guard-bypass privilege-escalation class (PITFALLS §2 / T-219-02).
    let guards = merged_guards(&action.preconditions, transition_guard);
    for guard_name in &guards {
        let passes = (dispatcher.guard_evaluator)(guard_name, tenant_id, inputs, db)
            .await
            .map_err(|e| WriteError::GuardFailed(format!("{guard_name}: {e}")))?;
        if !passes {
            return Err(WriteError::GuardFailed(format!(
                "precondition '{guard_name}' not met"
            )));
        }
    }

    // 2. Idempotency check (D-04).
    //
    // Lookup is scoped by BOTH tenant_id AND idempotency_key to prevent
    // cross-tenant replay (T-219-01). Absent key = no guard (key is optional).
    //
    // Length cap: idempotency_key is not declared in ActionDef.inputs so
    // validate_action_inputs() never sees it. Reject keys longer than 128
    // characters to prevent unbounded TEXT storage (storage DoS surface).
    if let Some(key) = inputs.get("idempotency_key").and_then(|v| v.as_str()) {
        if key.len() > 128 {
            return Err(WriteError::Validation(
                "idempotency_key must not exceed 128 characters".into(),
            ));
        }
    }
    let idempotency_key = inputs.get("idempotency_key").and_then(|v| v.as_str());
    if let Some(key) = idempotency_key {
        if let Some(stored_result) = lookup_idempotency(tenant_id, key, db).await? {
            // Replay: return stored result without re-executing or re-auditing.
            // The original call was already audited; replaying does not add a new entry.
            return Ok(stored_result);
        }
    }

    // 3. D-08 SEAM (Phase 220): confirmation gate for destructive actions.
    //
    // When the `confirmation` feature is on, a bare call to a destructive action
    // (transition_trigger.is_some()) without a valid confirmation context returns
    // Err(ConfirmationRequired) — the executor never fires. The channel's confirm
    // handler sets is_confirmed=true to bypass this seam after token validation.
    //
    // Feature-off: fall through to executor (Phase 219 behavior preserved).
    #[cfg(feature = "confirmation")]
    if action.transition_trigger.is_some() && !is_confirmed {
        return Err(WriteError::ConfirmationRequired(action.name.clone()));
    }
    #[cfg(not(feature = "confirmation"))]
    let _ = &action.transition_trigger;

    // 4. Execute callback (D-01).
    //    The executor owns TenantScoped enforcement (D-03): find_for_tenant(id, tenant_id)
    //    returning None is the cross-tenant denial primitive.
    let result = (dispatcher.executor)(&action.name, inputs, tenant_id, db).await?;

    // 5. Store idempotency result (D-04).
    //    INSERT OR IGNORE / ON CONFLICT DO NOTHING for concurrency safety.
    //
    //    WR-01: sealed BEFORE the override hook. There is no surrounding
    //    transaction, so the executor's base persist has already committed by
    //    this point. If the override (an app-specific side effect) ran first and
    //    failed, its `?` would short-circuit BEFORE the key is stored, leaving a
    //    committed base transition re-executable on retry (no stored key). Storing
    //    the key here guarantees a committed base persist is never re-executed,
    //    even when a later override fails.
    if let Some(key) = idempotency_key {
        store_idempotency(tenant_id, key, &result, db).await?;
    }

    // 6. Audit (D-05, SC#4) — record after every successful base persist.
    //    Denial audit (guard-failed path) is recorded in the channel framing.
    //
    //    WR-01: sealed BEFORE the override hook for the same reason. The base
    //    persist committed at step 4; the "every successful execution is audited"
    //    invariant must hold regardless of whether the app-specific override side
    //    effect later succeeds. An override failure does NOT roll back the base
    //    persist (no transaction), so the base persist is audited here
    //    unconditionally — the audit entry never overstates what persisted.
    let record_id = inputs.get("id").map(|v| v.to_string()).unwrap_or_default();
    AuditEntry::record(format!("{channel}.action.{}", &action.name))
        .tenant(tenant_id.to_string())
        .actor(AuditActor::User(tenant_id.to_string()))
        .target(AuditTarget::new(&action.name, record_id))
        .after(result.clone())
        .reason(&action.name)
        .write(db)
        .await
        .map_err(|e| WriteError::Database(e.to_string()))?;

    // 7. Post-persist override hook (EXEC-03).
    //    Runs AFTER the base persist is durable, idempotency-keyed, and audited
    //    (steps 4–6) — it cannot suppress the base guard or transition
    //    (T-231-05). An override `Err` propagates via `?` WITHOUT erasing the
    //    base persist's idempotency key or audit entry: those are already
    //    committed above. Absent key = common path (declaration-only); the
    //    override adds nothing.
    if let Some(hook) = dispatcher.overrides.get(&action.name) {
        (hook)(&action.name, inputs, tenant_id, db, &result).await?;
    }

    Ok(result)
}

// ── Kernel unit tests (relocated from ferro-mcp-server) ──────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::ActionDef;
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// In-memory SQLite with the `mcp_idempotency_keys` and `audit_log` tables
    /// created via raw SQL.
    async fn setup_db() -> sea_orm::DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("in-memory SQLite connect failed");
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE IF NOT EXISTS mcp_idempotency_keys (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tenant_id INTEGER NOT NULL,
                idempotency_key TEXT NOT NULL,
                result TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE (tenant_id, idempotency_key)
            )"
            .to_string(),
        ))
        .await
        .expect("create mcp_idempotency_keys table");
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY NOT NULL,
                tenant_id TEXT,
                actor_kind TEXT NOT NULL,
                actor_id TEXT,
                action TEXT NOT NULL,
                target_kind TEXT,
                target_id TEXT,
                before TEXT,
                after TEXT,
                reason TEXT,
                correlation_id TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
            .to_string(),
        ))
        .await
        .expect("create audit_log table");
        db
    }

    fn approve_action() -> ActionDef {
        ActionDef::new("approve")
            .transition_trigger("approve")
            .precondition("is_manager")
    }

    fn submit_action() -> ActionDef {
        ActionDef::new("submit").transition_trigger("submit")
    }

    fn update_action() -> ActionDef {
        ActionDef::new("update")
    }

    /// SC#1 (T-219-02): A guard evaluator returning `Ok(false)` must cause
    /// `dispatch_write` to return `Err(GuardFailed(_))` WITHOUT invoking the
    /// executor.
    #[tokio::test]
    async fn guard_denied_at_call_time() {
        let db = setup_db().await;

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(false) })),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { panic!("executor must not run when guard fails") })
            }),
            overrides: std::collections::HashMap::new(),
        };

        let result = dispatch_write(
            &approve_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
        )
        .await;

        assert!(
            matches!(result, Err(WriteError::GuardFailed(_))),
            "expected Err(GuardFailed(_)), got: {result:?}"
        );
    }

    /// A guard_evaluator returning `Ok(false)` for a transition that carries a
    /// `Transition.guard` causes `dispatch_write` to return `Err(GuardFailed(_))`
    /// and the executor never runs.
    #[tokio::test]
    async fn guard_rejects_illegal_transition() {
        let db = setup_db().await;

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(false) })),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { panic!("executor must not run when transition guard fails") })
            }),
            overrides: std::collections::HashMap::new(),
        };

        let result = dispatch_write(
            &submit_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            Some("is_manager"),
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
        )
        .await;

        assert!(
            matches!(result, Err(WriteError::GuardFailed(_))),
            "expected Err(GuardFailed(_)) from the transition guard, got: {result:?}"
        );
    }

    /// A transition-level guard present on the PLAN but absent from
    /// `action.preconditions` is still evaluated.
    #[tokio::test]
    async fn transition_guard_evaluated_at_call_time() {
        let db = setup_db().await;
        let saw_transition_guard = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new({
                let flag = saw_transition_guard.clone();
                move |name: &str, _, _, _| {
                    if name == "is_manager" {
                        flag.store(true, Ordering::SeqCst);
                    }
                    Box::pin(async { Ok(true) })
                }
            }),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { Ok(json!({ "status": "submitted" })) })
            }),
            overrides: std::collections::HashMap::new(),
        };

        let result = dispatch_write(
            &submit_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            Some("is_manager"),
            "mcp",
            #[cfg(feature = "confirmation")]
            true,
        )
        .await;

        assert!(
            result.is_ok(),
            "guard returns true, write must succeed: {result:?}"
        );
        assert!(
            saw_transition_guard.load(Ordering::SeqCst),
            "the transition-level guard 'is_manager' must be evaluated live"
        );
    }

    /// A guard name present on BOTH the transition AND the action precondition
    /// fires the evaluator exactly ONCE (deduped by name).
    #[tokio::test]
    async fn guard_deduped_when_on_both() {
        let db = setup_db().await;
        let call_count = Arc::new(AtomicUsize::new(0));

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new({
                let count = call_count.clone();
                move |name: &str, _, _, _| {
                    if name == "is_manager" {
                        count.fetch_add(1, Ordering::SeqCst);
                    }
                    Box::pin(async { Ok(true) })
                }
            }),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { Ok(json!({ "status": "approved" })) })
            }),
            overrides: std::collections::HashMap::new(),
        };

        let result = dispatch_write(
            &approve_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            Some("is_manager"),
            "mcp",
            #[cfg(feature = "confirmation")]
            true,
        )
        .await;

        assert!(
            result.is_ok(),
            "both guards pass; write must succeed: {result:?}"
        );
        assert_eq!(
            call_count.load(Ordering::SeqCst),
            1,
            "is_manager must be evaluated exactly once (deduped), not twice"
        );
    }

    /// An override registered for an action runs AFTER the base persist.
    #[tokio::test]
    async fn override_hook_runs_post_persist() {
        let db = setup_db().await;
        let order = Arc::new(std::sync::Mutex::new(Vec::<&'static str>::new()));

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new({
                let order = order.clone();
                move |_, _, _, _| {
                    order.lock().unwrap().push("persist");
                    Box::pin(async { Ok(json!({ "status": "submitted" })) })
                }
            }),
            overrides: std::collections::HashMap::new(),
        }
        .with_override(
            "submit",
            Box::new({
                let order = order.clone();
                move |_action, _inputs, _tenant, _db, base_result: &Value| {
                    assert_eq!(base_result["status"], "submitted");
                    order.lock().unwrap().push("override");
                    Box::pin(async { Ok(()) })
                }
            }),
        );

        let result = dispatch_write(
            &submit_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            true,
        )
        .await;

        assert!(result.is_ok(), "write must succeed: {result:?}");
        assert_eq!(
            result.unwrap(),
            json!({ "status": "submitted" }),
            "base result must be returned unchanged by the override"
        );
        assert_eq!(
            *order.lock().unwrap(),
            vec!["persist", "override"],
            "override must run AFTER the base persist"
        );
    }

    /// With NO override registered, `dispatch_write` behaves exactly as before.
    #[tokio::test]
    async fn no_override_is_declaration_only() {
        let db = setup_db().await;

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { Ok(json!({ "status": "submitted" })) })
            }),
            overrides: std::collections::HashMap::new(),
        };

        let result = dispatch_write(
            &submit_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            true,
        )
        .await;

        assert!(result.is_ok(), "no-override path must succeed: {result:?}");
        assert_eq!(result.unwrap(), json!({ "status": "submitted" }));
    }

    /// An override returning `Err` causes `dispatch_write` to return `Err`, while
    /// the base persist's idempotency key AND audit entry remain sealed (WR-01).
    #[tokio::test]
    async fn override_error_surfaces() {
        let db = setup_db().await;

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { Ok(json!({ "status": "submitted" })) })
            }),
            overrides: std::collections::HashMap::new(),
        }
        .with_override(
            "submit",
            Box::new(|_action, _inputs, _tenant, _db, _base| {
                Box::pin(async { Err(WriteError::Validation("override failed".into())) })
            }),
        );

        let result = dispatch_write(
            &submit_action(),
            &json!({"id": 1, "idempotency_key": "ovr-fail-1"}),
            1,
            &db,
            &dispatcher,
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            true,
        )
        .await;

        assert!(
            matches!(result, Err(WriteError::Validation(ref m)) if m == "override failed"),
            "override error must propagate, got: {result:?}"
        );

        // WR-01: the base persist's audit entry must exist despite the override
        // failure — and it must carry the channel-prefixed action `mcp.action.submit`.
        let audit_count: i64 = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) AS c FROM audit_log WHERE action = 'mcp.action.submit'"
                    .to_string(),
            ))
            .await
            .expect("audit query must succeed")
            .expect("audit count row must exist")
            .try_get::<i64>("", "c")
            .expect("audit count column");
        assert_eq!(
            audit_count, 1,
            "base persist must be audited even when the override fails (WR-01)"
        );

        // WR-01: the idempotency key must be stored despite the override failure.
        let stored = lookup_idempotency(1, "ovr-fail-1", &db)
            .await
            .expect("idempotency lookup must succeed");
        assert_eq!(
            stored,
            Some(json!({ "status": "submitted" })),
            "base persist's idempotency key must be stored even when the override fails (WR-01)"
        );
    }

    /// SC#3 (T-219-03): Two identical `dispatch_write` calls with the same
    /// `idempotency_key` produce equal results and fire the executor once.
    #[tokio::test]
    async fn idempotent_replay_does_not_re_execute() {
        let db = setup_db().await;
        let exec_count = Arc::new(AtomicUsize::new(0));

        let dispatcher = WriteDispatcher {
            executor: Box::new({
                let count = exec_count.clone();
                move |_, _, _, _| {
                    count.fetch_add(1, Ordering::SeqCst);
                    Box::pin(async { Ok(json!({ "status": "submitted" })) })
                }
            }),
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            overrides: std::collections::HashMap::new(),
        };

        let args = json!({ "id": 1, "idempotency_key": "k-abc" });

        let result1 = dispatch_write(
            &update_action(),
            &args,
            1,
            &db,
            &dispatcher,
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
        )
        .await;
        let result2 = dispatch_write(
            &update_action(),
            &args,
            1,
            &db,
            &dispatcher,
            None,
            "mcp",
            #[cfg(feature = "confirmation")]
            false,
        )
        .await;

        assert!(result1.is_ok(), "first call must succeed; got: {result1:?}");
        assert!(
            result2.is_ok(),
            "second call must succeed (replay); got: {result2:?}"
        );
        assert_eq!(
            result1.unwrap(),
            result2.unwrap(),
            "idempotent replay must return the same result"
        );
        assert_eq!(
            exec_count.load(Ordering::SeqCst),
            1,
            "executor must fire exactly once despite two identical calls"
        );
    }

    /// The audit reason prefix is built from the `channel` argument: passing a
    /// different channel writes `web.action.submit`, not `mcp.action.submit`.
    #[tokio::test]
    async fn audit_channel_is_parameterized() {
        let db = setup_db().await;

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { Ok(json!({ "status": "submitted" })) })
            }),
            overrides: std::collections::HashMap::new(),
        };

        let result = dispatch_write(
            &submit_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            None,
            "web",
            #[cfg(feature = "confirmation")]
            true,
        )
        .await;
        assert!(result.is_ok(), "write must succeed: {result:?}");

        let web_count: i64 = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) AS c FROM audit_log WHERE action = 'web.action.submit'"
                    .to_string(),
            ))
            .await
            .expect("audit query must succeed")
            .expect("audit count row must exist")
            .try_get::<i64>("", "c")
            .expect("audit count column");
        assert_eq!(
            web_count, 1,
            "audit action must be prefixed by the channel argument ('web')"
        );

        let mcp_count: i64 = db
            .query_one(Statement::from_string(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(*) AS c FROM audit_log WHERE action = 'mcp.action.submit'"
                    .to_string(),
            ))
            .await
            .expect("audit query must succeed")
            .expect("audit count row must exist")
            .try_get::<i64>("", "c")
            .expect("audit count column");
        assert_eq!(
            mcp_count, 0,
            "no mcp-channel audit must be written when channel is 'web'"
        );
    }
}
