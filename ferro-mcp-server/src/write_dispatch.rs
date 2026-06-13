//! Write tool dispatch for the MCP endpoint.
//!
//! Defines [`WriteDispatcher`], [`ExecutorFn`], and [`GuardEvaluatorFn`] — the
//! app-registered callback pair that makes write tools callable. The actual
//! execution logic lives in the consumer app; `ferro-mcp-server` owns the
//! security envelope: scope check (Phase 217), guard re-evaluation against live
//! DB state (D-02), idempotency replay (D-04), audit (D-05), and a
//! spec-compliant [`rmcp::model::CallToolResult`] result (D-06).

use ferro_audit::{AuditActor, AuditEntry, AuditTarget};
use ferro_projections::{ActionDef, ServiceDef};
use rmcp::model::CallToolResult;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;

// ── Boxed-future callback types (no async-trait dep) ─────────────────────────

/// App-registered write executor.
///
/// Called by `dispatch_write` after guards pass and idempotency is checked.
/// Receives the action name, validated inputs, the authenticated tenant id
/// (never from the call payload), and a DB connection.
/// Returns a JSON `Value` that becomes the `result` payload in the structured
/// MCP response.
///
/// # Audit contract
///
/// The returned `Value` is stored **verbatim** in the append-only `audit_log`
/// table via [`ferro_audit::AuditEntry`]. It MUST NOT contain secrets,
/// credentials, PII, or any field that should not appear in a forensic log
/// readable by audit reviewers. Executors are responsible for returning only
/// audit-safe fields (typically identifiers and status values, e.g.
/// `{"id": 42, "status": "approved"}`). A full PII scrub at the
/// `dispatch_write` call site is not performed; the executor is the enforcement
/// point.
pub type ExecutorFn = Box<
    dyn Fn(
            &str,   // action_name
            &Value, // validated inputs
            i64,    // tenant_id (from auth, never from payload)
            &DatabaseConnection,
        ) -> Pin<Box<dyn Future<Output = crate::Result<Value>> + Send>>
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
        ) -> Pin<Box<dyn Future<Output = crate::Result<bool>> + Send>>
        + Send
        + Sync,
>;

/// Holds the app-registered write callback and guard evaluator.
///
/// Constructed by the consumer app and passed to [`handle_write_call`].
/// Not stored in [`crate::McpServerConfig`] — threaded at call-site parallel
/// to `db` and `tenant_id`.
pub struct WriteDispatcher {
    pub executor: ExecutorFn,
    pub guard_evaluator: GuardEvaluatorFn,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Locate an [`ActionDef`] by tool name across all mcp-exposed services.
///
/// Returns `Some((&ServiceDef, &ActionDef))` or `None`.
fn find_action<'a>(
    services: &'a [ServiceDef],
    tool_name: &str,
) -> Option<(&'a ServiceDef, &'a ActionDef)> {
    for svc in services {
        if !svc.mcp_exposed {
            continue;
        }
        for action in &svc.actions {
            if action.name == tool_name {
                return Some((svc, action));
            }
        }
    }
    None
}

/// Validate that required inputs declared in `action.inputs` are present in
/// `args`. Returns `Err(String)` with the first missing field name.
fn validate_action_inputs(action: &ActionDef, args: &Value) -> Result<(), String> {
    for input in &action.inputs {
        if input.required && args.get(&input.name).is_none() {
            return Err(format!("required field '{}' missing", input.name));
        }
    }
    Ok(())
}

/// Build an isError:true structured MCP result envelope for write-path errors.
///
/// Shape mirrors `make_tool_deny_response` in `app/src/controllers/mcp.rs`
/// but without the outer jsonrpc/id fields (those are spliced by the HTTP
/// adapter layer). This is the ONLY error-result constructor — no bare
/// content[] arrays constructed elsewhere (D-06).
pub fn write_tool_error_result(payload: Value) -> Value {
    let message = payload
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("error")
        .to_string();
    json!({
        "content": [{ "type": "text", "text": message }],
        "isError": true,
        "structuredContent": payload
    })
}

/// Look up a stored idempotency result scoped by BOTH tenant_id AND idempotency_key.
///
/// Cross-tenant replay is prevented at the SQL level: the WHERE clause requires
/// BOTH columns, matching the UNIQUE index on `(tenant_id, idempotency_key)`.
async fn lookup_idempotency(
    tenant_id: i64,
    key: &str,
    db: &DatabaseConnection,
) -> crate::Result<Option<Value>> {
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
        .map_err(|e| crate::Error::Database(e.to_string()))?
    {
        None => Ok(None),
        Some(row) => {
            let json_text: String = row
                .try_get("", "result")
                .map_err(|e| crate::Error::Database(e.to_string()))?;
            let value: Value = serde_json::from_str(&json_text)
                .map_err(|e| crate::Error::Database(e.to_string()))?;
            Ok(Some(value))
        }
    }
}

/// Store an idempotency result scoped by (tenant_id, idempotency_key).
///
/// Uses INSERT OR IGNORE (SQLite) / ON CONFLICT DO NOTHING (Postgres) for
/// concurrency safety — a second concurrent identical request will not cause
/// a UNIQUE constraint error (PITFALLS §5).
async fn store_idempotency(
    tenant_id: i64,
    key: &str,
    result: &Value,
    db: &DatabaseConnection,
) -> crate::Result<()> {
    let backend = db.get_database_backend();
    let json_text = serde_json::to_string(result).map_err(crate::Error::Serialization)?;

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
        .map_err(|e| crate::Error::Database(e.to_string()))?;
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
/// 3. D-08 confirmation seam — pass-through in Phase 219.
/// 4. Execute callback (D-01).
/// 5. Store idempotency result (D-04).
/// 6. Audit via ferro-audit (D-05).
pub async fn dispatch_write(
    action: &ActionDef,
    inputs: &Value,
    tenant_id: i64,
    db: &DatabaseConnection,
    dispatcher: &WriteDispatcher,
) -> crate::Result<Value> {
    // 1. Guard re-evaluation (D-02, T-219-02 — load-bearing security gate).
    //
    // Calls the app-registered GuardEvaluatorFn for EVERY precondition in
    // action.preconditions against LIVE DB state. Fail-closed: a guard returning
    // Ok(false) OR any Err immediately returns Err(GuardFailed).
    //
    // IMPORTANT: ctx.evaluated_guards (the 218 list-time visibility cache) is
    // intentionally NOT consulted here. An agent may bypass tools/list entirely
    // and call tools/call directly — only this live re-evaluation prevents the
    // guard-bypass privilege-escalation class (PITFALLS §2 / T-219-02).
    for guard_name in &action.preconditions {
        let passes = (dispatcher.guard_evaluator)(guard_name, tenant_id, inputs, db)
            .await
            .map_err(|e| crate::Error::GuardFailed(format!("{guard_name}: {e}")))?;
        if !passes {
            return Err(crate::Error::GuardFailed(format!(
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
            return Err(crate::Error::Validation(
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

    // 3. D-08 SEAM: Phase 220 inserts confirmation gating here for destructive actions
    //    (transition_trigger.is_some()). In 219: pass through directly.
    //    Do NOT wire ferro-ai / ConfirmationStore here.
    //    if action.transition_trigger.is_some() { /* Phase 220 will intercept */ }
    let _ = &action.transition_trigger; // reference to avoid unused-field lint during seam

    // 4. Execute callback (D-01).
    //    The executor owns TenantScoped enforcement (D-03): find_for_tenant(id, tenant_id)
    //    returning None is the cross-tenant denial primitive.
    let result = (dispatcher.executor)(&action.name, inputs, tenant_id, db).await?;

    // 5. Store idempotency result (D-04).
    //    INSERT OR IGNORE / ON CONFLICT DO NOTHING for concurrency safety.
    if let Some(key) = idempotency_key {
        store_idempotency(tenant_id, key, &result, db).await?;
    }

    // 6. Audit (D-05, SC#4) — record after every successful execution.
    //    Denial audit (guard-failed path) is recorded in handle_write_call.
    let record_id = inputs.get("id").map(|v| v.to_string()).unwrap_or_default();
    AuditEntry::record(format!("mcp.action.{}", &action.name))
        .tenant(tenant_id.to_string())
        .actor(AuditActor::User(tenant_id.to_string()))
        .target(AuditTarget::new(&action.name, record_id))
        .after(result.clone())
        .reason(&action.name)
        .write(db)
        .await
        .map_err(|e| crate::Error::Database(e.to_string()))?;

    Ok(result)
}

/// Route a non-`list_` tool call to the write dispatch path.
///
/// Implements the D-07 pipeline after the scope check (which stays in
/// `handle_tools_call`, in front of this function):
/// resolve ActionDef → validate inputs → dispatch_write (guard re-eval +
/// idempotency + execute + audit) → structured result envelope.
pub async fn handle_write_call(
    call_params: Value,
    services: &[ServiceDef],
    db: &DatabaseConnection,
    tenant_id: Option<i64>,
    ctx: &crate::McpContext,
    dispatcher: &WriteDispatcher,
) -> Value {
    // Scope check already ran in handle_tools_call; we are here for the write path.
    // Suppress unused ctx warning — ctx is available for future extensions (e.g. tracing).
    let _ = ctx;

    let tool_name = call_params["name"].as_str().unwrap_or("");

    // Fail-closed: writes always require an authenticated tenant.
    // tenant_id is the unwrapped authenticated principal — never from the payload.
    let tid = match tenant_id {
        Some(t) => t,
        None => {
            return json!({ "error": { "code": -32603, "message": "auth: tenant required" } });
        }
    };

    // Resolve the ActionDef by tool name across mcp-exposed services.
    let (_svc, action) = match find_action(services, tool_name) {
        Some(pair) => pair,
        None => {
            return json!({ "error": { "code": -32601, "message": "Method not found" } });
        }
    };

    let args = call_params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    // Validate required inputs against ActionDef.inputs before dispatching.
    if let Err(msg) = validate_action_inputs(action, &args) {
        return json!({ "result": write_tool_error_result(json!({
            "error_kind": "validation",
            "message": msg
        })) });
    }

    // Dispatch: guard re-eval → idempotency → seam → execute → store → audit.
    match dispatch_write(action, &args, tid, db, dispatcher).await {
        Ok(result) => {
            let payload = json!({
                "status": "ok",
                "action": action.name,
                "result": result
            });
            let tool_result = CallToolResult::structured(payload);
            json!({ "result": tool_result })
        }
        Err(crate::Error::GuardFailed(ref msg)) => {
            // Audit the denial for forensic trail (PITFALLS §2 / D-05).
            let record_id = args.get("id").map(|v| v.to_string()).unwrap_or_default();
            let _ = AuditEntry::record(format!("mcp.action.{}", action.name))
                .tenant(tid.to_string())
                .actor(AuditActor::User(tid.to_string()))
                .target(AuditTarget::new(&action.name, record_id))
                .after(json!({ "denied": true, "reason": "guard_failed", "guard": msg }))
                .reason(&action.name)
                .write(db)
                .await;
            json!({ "result": write_tool_error_result(json!({
                "error_kind": "guard_denied",
                "message": msg
            })) })
        }
        // Agent-safe variants: pass message through (no internal state in these strings).
        Err(ref e @ crate::Error::Validation(_))
        | Err(ref e @ crate::Error::ActionNotFound(_)) => {
            json!({ "result": write_tool_error_result(json!({
                "error_kind": "execution_error",
                "message": e.to_string()
            })) })
        }
        // All other variants (Database, Serialization, Auth, etc.) may contain SQL
        // fragments, table names, column names, or constraint names — redact them.
        Err(_) => {
            json!({ "result": write_tool_error_result(json!({
                "error_kind": "execution_error",
                "message": "write operation failed"
            })) })
        }
    }
}

// ── RED unit tests (Wave 0 — compile and FAIL; Wave 1 makes them GREEN) ──────

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::CallToolResult;
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    // ── Test DB setup ─────────────────────────────────────────────────────────

    /// In-memory SQLite with the `mcp_idempotency_keys` and `audit_log` tables
    /// created via raw SQL (matches MigrationMcpIdempotencyKeys and
    /// CreateAuditLogTable schemas). Avoids pulling `async-trait` or
    /// `sea-orm-migration` into `ferro-mcp-server`'s dev-dependencies.
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
        // audit_log table — required by AuditEntry::write() called inside dispatch_write.
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

    /// Build a synthetic `ActionDef` with a precondition guard (the `approve` fixture).
    fn approve_action() -> ActionDef {
        ActionDef::new("approve")
            .transition_trigger("approve")
            .precondition("is_manager")
    }

    /// Build a synthetic `ActionDef` with no preconditions (the `submit` fixture).
    fn submit_action() -> ActionDef {
        ActionDef::new("submit").transition_trigger("submit")
    }

    /// Build a minimal [`ServiceDef`] exposing both actions.
    fn order_service_with_actions() -> ServiceDef {
        use ferro_projections::{DataType, FieldMeaning};
        ServiceDef::new("order")
            .mcp_exposed(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status)
            .action(approve_action())
            .action(submit_action())
    }

    // ── SC#1 — T-219-02 ──────────────────────────────────────────────────────

    /// SC#1 (T-219-02): A guard evaluator returning `Ok(false)` must cause
    /// `dispatch_write` to return `Err(GuardFailed(_))` WITHOUT invoking the
    /// executor. Validates that guard re-evaluation happens BEFORE execution and
    /// reads the `GuardEvaluatorFn`, never `ctx.evaluated_guards`.
    #[tokio::test]
    async fn guard_denied_at_call_time() {
        let db = setup_db().await;

        let dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(false) })),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { panic!("executor must not run when guard fails") })
            }),
        };

        let result =
            dispatch_write(&approve_action(), &json!({"id": 1}), 1, &db, &dispatcher).await;

        assert!(
            matches!(result, Err(crate::Error::GuardFailed(_))),
            "expected Err(GuardFailed(_)), got: {result:?}"
        );
    }

    // ── SC#3 — T-219-03 ──────────────────────────────────────────────────────

    /// SC#3 (T-219-03): Two identical `dispatch_write` calls with the same
    /// `idempotency_key` must produce equal results and fire the executor
    /// exactly once. Validates that the idempotency table prevents re-execution.
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
        };

        let args = json!({ "id": 1, "idempotency_key": "k-abc" });

        let result1 = dispatch_write(&submit_action(), &args, 1, &db, &dispatcher).await;
        let result2 = dispatch_write(&submit_action(), &args, 1, &db, &dispatcher).await;

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

    // ── SC#5 ─────────────────────────────────────────────────────────────────

    /// SC#5: Every write-path result envelope — success and guard-denied —
    /// must parse as [`rmcp::model::CallToolResult`] with the correct
    /// `is_error` flag.
    ///
    /// Mirrors the Phase 205 `tools_call_result_parses_as_valid_mcp_content`
    /// test in `jsonrpc.rs`.
    #[tokio::test]
    async fn write_tool_result_parses_as_valid_mcp_content() {
        let db = setup_db().await;
        let services = vec![order_service_with_actions()];
        let ctx = crate::McpContext::default();

        // --- success case: guard passes, executor returns Ok ---
        let success_dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { Ok(json!({ "status": "submitted" })) })
            }),
        };
        let success_params = json!({ "name": "submit", "arguments": { "id": 1 } });
        let success_response = handle_write_call(
            success_params,
            &services,
            &db,
            Some(1),
            &ctx,
            &success_dispatcher,
        )
        .await;

        let parsed_success: CallToolResult =
            serde_json::from_value(success_response["result"].clone())
                .expect("success result must parse as CallToolResult");
        assert_eq!(
            parsed_success.is_error,
            Some(false),
            "success result must have is_error=false"
        );
        assert_eq!(
            parsed_success.content.len(),
            1,
            "structured() produces exactly one content block"
        );
        let content_json = serde_json::to_value(&parsed_success.content).unwrap();
        assert_eq!(
            content_json[0]["type"].as_str(),
            Some("text"),
            "content[0] must have type=text"
        );

        // --- guard-denied case: guard returns false ---
        let deny_dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(false) })),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { panic!("executor must not run when guard fails") })
            }),
        };
        let deny_params = json!({ "name": "approve", "arguments": { "id": 1 } });
        let deny_response =
            handle_write_call(deny_params, &services, &db, Some(1), &ctx, &deny_dispatcher).await;

        let parsed_deny: CallToolResult = serde_json::from_value(deny_response["result"].clone())
            .expect("guard-denied result must parse as CallToolResult");
        assert_eq!(
            parsed_deny.is_error,
            Some(true),
            "guard-denied result must have is_error=true"
        );
    }
}
