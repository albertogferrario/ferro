//! Write tool dispatch for the MCP endpoint.
//!
//! Defines [`WriteDispatcher`], [`ExecutorFn`], and [`GuardEvaluatorFn`] — the
//! app-registered callback pair that makes write tools callable. The actual
//! execution logic lives in the consumer app; `ferro-mcp-server` owns the
//! security envelope: scope check (Phase 217), guard re-evaluation against live
//! DB state (D-02), idempotency replay (D-04), audit (D-05), and a
//! spec-compliant [`rmcp::model::CallToolResult`] result (D-06).
//!
//! Wave 0 stub: `dispatch_write` and `handle_write_call` return minimal error
//! bodies so the crate compiles. Wave 1 fills the implementations.

use ferro_projections::{ActionDef, ServiceDef};
use sea_orm::DatabaseConnection;
use serde_json::Value;
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

// ── Stubs (Wave 0 — compile-only bodies) ─────────────────────────────────────

/// Locate an [`ActionDef`] by tool name across all mcp-exposed services.
///
/// Returns `Some((&ServiceDef, &ActionDef))` or `None`.
#[allow(dead_code)]
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
#[allow(dead_code)]
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
/// adapter layer).
#[allow(dead_code)]
fn write_tool_error_result(message: &str, error_kind: &str, payload: Value) -> Value {
    serde_json::json!({
        "content": [{ "type": "text", "text": message }],
        "isError": true,
        "structuredContent": {
            "error_kind": error_kind,
            "message": message,
            "detail": payload
        }
    })
}

/// Stub idempotency lookup — Wave 1 implements real SQL.
///
/// Returns `Ok(None)` always (no stored result).
#[allow(dead_code)]
async fn lookup_idempotency(
    _tenant_id: i64,
    _key: &str,
    _db: &DatabaseConnection,
) -> crate::Result<Option<Value>> {
    Ok(None)
}

/// Stub idempotency store — Wave 1 implements real SQL.
#[allow(dead_code)]
async fn store_idempotency(
    _tenant_id: i64,
    _key: &str,
    _result: &Value,
    _db: &DatabaseConnection,
) -> crate::Result<()> {
    Ok(())
}

/// Execute a write action with guard re-evaluation, idempotency, and audit.
///
/// Wave 0 stub: returns `Err(Validation("not implemented"))` so tests compile
/// and fail on assertion rather than panicking. Wave 1 fills the full pipeline.
pub async fn dispatch_write(
    action: &ActionDef,
    inputs: &Value,
    tenant_id: i64,
    db: &DatabaseConnection,
    dispatcher: &WriteDispatcher,
) -> crate::Result<Value> {
    // Wave 0 stub — Wave 1 implements:
    //   1. Re-evaluate guards (D-02 — LIVE state, never ctx.evaluated_guards)
    //   2. Idempotency check (D-04)
    //   3. D-08 seam: if action.transition_trigger.is_some() → Phase 220 intercept
    //   4. Execute callback (D-01)
    //   5. Store idempotency result (D-04)
    //   6. Audit (D-05)
    let _ = (action, inputs, tenant_id, db, dispatcher);
    Err(crate::Error::Validation("not implemented".into()))
}

/// Route a non-`list_` tool call to the write dispatch path.
///
/// Wave 0 stub: returns a -32601 error envelope. Wave 1 implements the full
/// pipeline described in D-07: scope check → resolve ActionDef → validate
/// inputs → re-evaluate guards → idempotency check → execute → audit →
/// structured result.
pub async fn handle_write_call(
    call_params: Value,
    services: &[ServiceDef],
    db: &DatabaseConnection,
    tenant_id: Option<i64>,
    ctx: &crate::McpContext,
    dispatcher: &WriteDispatcher,
) -> Value {
    let _ = (call_params, services, db, tenant_id, ctx, dispatcher);
    serde_json::json!({ "error": { "code": -32601, "message": "not implemented" } })
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

    /// In-memory SQLite with the `mcp_idempotency_keys` table created via raw
    /// SQL (matches the `MigrationMcpIdempotencyKeys` schema defined in Task 2).
    /// Avoids pulling `async-trait` or `sea-orm-migration` into
    /// `ferro-mcp-server`'s dev-dependencies.
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
    ///
    /// RED: Wave 0 stub returns `Err(Validation(...))`, not `Err(GuardFailed(_))`.
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

        // RED assertion: stub returns Validation, not GuardFailed.
        // Wave 1 makes this pass by implementing the guard loop.
        assert!(
            matches!(result, Err(crate::Error::GuardFailed(_))),
            "expected Err(GuardFailed(_)), got: {result:?}"
        );
    }

    // ── SC#3 — T-219-03 ──────────────────────────────────────────────────────

    /// SC#3 (T-219-03): Two identical `dispatch_write` calls with the same
    /// `idempotency_key` must produce equal results and fire the executor
    /// exactly once. Validates that the idempotency table prevents re-execution.
    ///
    /// RED: Wave 0 stub always returns `Err(Validation(...))` and never stores
    /// or replays; exec_count will be 0 after two calls, not 1.
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

        // RED assertions: stub always returns Err, count == 0.
        // Wave 1 makes these pass by implementing idempotency storage + replay.
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
    ///
    /// RED: `handle_write_call` stub returns an `error`-keyed envelope, not a
    /// `result`-keyed `CallToolResult`. Wave 1 makes this pass by emitting
    /// `CallToolResult::structured(payload)` for success and the isError:true
    /// envelope for guard-denied outcomes.
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

        // RED: stub returns { "error": {...} } which has no "result" key.
        // Wave 1 emits { "result": <CallToolResult> } → parsing succeeds.
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

        // RED: stub returns error envelope, not CallToolResult.
        // Wave 1 emits { "result": { "content": [...], "isError": true } }.
        let parsed_deny: CallToolResult = serde_json::from_value(deny_response["result"].clone())
            .expect("guard-denied result must parse as CallToolResult");
        assert_eq!(
            parsed_deny.is_error,
            Some(true),
            "guard-denied result must have is_error=true"
        );
    }
}
