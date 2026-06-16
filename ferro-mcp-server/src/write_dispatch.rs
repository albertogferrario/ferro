//! Write tool dispatch for the MCP endpoint.
//!
//! Defines [`WriteDispatcher`], [`ExecutorFn`], and [`GuardEvaluatorFn`] — the
//! app-registered callback pair that makes write tools callable. The actual
//! execution logic lives in the consumer app; `ferro-mcp-server` owns the
//! security envelope: scope check (Phase 217), guard re-evaluation against live
//! DB state (D-02), idempotency replay (D-04), audit (D-05), and a
//! spec-compliant [`rmcp::model::CallToolResult`] result (D-06).

use ferro_audit::{AuditActor, AuditEntry, AuditTarget};
use ferro_projections::{derive_transition_plan, ActionDef, ServiceDef};
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

/// App-registered post-persist override hook (EXEC-03).
///
/// Runs AFTER the derived/base persist, inside [`dispatch_write`], reusing the
/// audit/idempotency envelope. Receives the action name, validated inputs, the
/// authenticated tenant id, a DB connection, and the base persist result — so it
/// can chain related-record writes keyed off the just-persisted state.
///
/// Because it runs strictly after the guarded base persist, an override cannot
/// suppress the base guard or transition (threat T-231-05, mitigated).
pub type OverrideFn = Box<
    dyn Fn(
            &str,   // action_name
            &Value, // validated inputs
            i64,    // tenant_id (from auth, never from payload)
            &DatabaseConnection,
            &Value, // base persist result
        ) -> Pin<Box<dyn Future<Output = crate::Result<()>> + Send>>
        + Send
        + Sync,
>;

/// Holds the app-registered write callback, guard evaluator, and optional
/// post-persist override hooks.
///
/// Constructed by the consumer app and passed to [`handle_write_call`].
/// Not stored in [`crate::McpServerConfig`] — threaded at call-site parallel
/// to `db` and `tenant_id`.
pub struct WriteDispatcher {
    pub executor: ExecutorFn,
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
fn merged_guards(preconditions: &[String], transition_guard: Option<&str>) -> Vec<String> {
    let mut guards: Vec<String> = preconditions.to_vec();
    if let Some(g) = transition_guard {
        if !guards.iter().any(|existing| existing == g) {
            guards.push(g.to_string());
        }
    }
    guards
}

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

// ── Token generator (confirmation feature only) ───────────────────────────────

/// Generate a server-side CSPRNG confirmation token.
///
/// Uses the same BASE62 + rand pattern as `generate_mcp_api_key` in
/// `ferro-mcp-oauth`. Token format: `cfm_` prefix + 43 BASE62 chars (~256-bit
/// entropy). The token is never agent-supplied; it is issued by
/// `handle_request_confirm` and verified by `handle_confirm`.
#[cfg(feature = "confirmation")]
fn generate_confirmation_token() -> String {
    use rand::Rng;
    const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::thread_rng();
    let random: String = (0..43)
        .map(|_| {
            let idx = rng.gen_range(0..62usize);
            BASE62[idx] as char
        })
        .collect();
    format!("cfm_{random}")
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
///
/// The `is_confirmed` parameter (confirmation feature only) signals that this
/// call came from `handle_confirm` after token validation — the D-08 seam is
/// bypassed when `true`. Bare callers always pass `false` (or omit when feature
/// is off).
pub async fn dispatch_write(
    action: &ActionDef,
    inputs: &Value,
    tenant_id: i64,
    db: &DatabaseConnection,
    dispatcher: &WriteDispatcher,
    transition_guard: Option<&str>,
    #[cfg(feature = "confirmation")] is_confirmed: bool,
) -> crate::Result<Value> {
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

    // 3. D-08 SEAM (Phase 220): confirmation gate for destructive actions.
    //
    // When the `confirmation` feature is on, a bare call to a destructive action
    // (transition_trigger.is_some()) without a valid confirmation context returns
    // Err(ConfirmationRequired) — the executor never fires. `handle_confirm` sets
    // is_confirmed=true to bypass this seam after token validation.
    //
    // Feature-off: fall through to executor (Phase 219 behavior preserved).
    #[cfg(feature = "confirmation")]
    if action.transition_trigger.is_some() && !is_confirmed {
        return Err(crate::Error::ConfirmationRequired(action.name.clone()));
    }
    #[cfg(not(feature = "confirmation"))]
    let _ = &action.transition_trigger;

    // 4. Execute callback (D-01).
    //    The executor owns TenantScoped enforcement (D-03): find_for_tenant(id, tenant_id)
    //    returning None is the cross-tenant denial primitive.
    let result = (dispatcher.executor)(&action.name, inputs, tenant_id, db).await?;

    // 4b. Post-persist override hook (EXEC-03).
    //     Runs AFTER the guarded base persist, inside the same audited window —
    //     it cannot suppress the base guard or transition (T-231-05). Absent key
    //     = common path (declaration-only); the override adds nothing.
    if let Some(hook) = dispatcher.overrides.get(&action.name) {
        (hook)(&action.name, inputs, tenant_id, db, &result).await?;
    }

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
///
/// When the `confirmation` feature is on, `store` and `config` are threaded
/// from `handle_tools_call` for the `request_confirm_`/`confirm_` prefix routing.
#[allow(clippy::too_many_arguments)]
pub async fn handle_write_call(
    call_params: Value,
    services: &[ServiceDef],
    db: &DatabaseConnection,
    tenant_id: Option<i64>,
    ctx: &crate::McpContext,
    dispatcher: &WriteDispatcher,
    #[cfg(feature = "confirmation")] store: &dyn ferro_ai::ConfirmationStore,
    #[cfg(feature = "confirmation")] config: &crate::McpServerConfig,
) -> Value {
    // Scope check already ran in handle_tools_call; we are here for the write path.
    // Suppress unused ctx warning — ctx is available for future extensions (e.g. tracing).
    let _ = ctx;

    let tool_name = call_params["name"].as_str().unwrap_or("").to_string();

    // Phase 220: confirmation tool prefix routing.
    // Check `request_confirm_` before `confirm_` (order matters — confirm_ is a
    // shorter prefix that could shadow if checked first).
    #[cfg(feature = "confirmation")]
    if let Some(action_name) = tool_name.strip_prefix("request_confirm_") {
        let action_name = action_name.to_string();
        return handle_request_confirm(
            call_params,
            services,
            db,
            tenant_id,
            ctx,
            dispatcher,
            store,
            &action_name,
            config.confirmation_ttl_seconds,
        )
        .await;
    }
    #[cfg(feature = "confirmation")]
    if let Some(action_name) = tool_name.strip_prefix("confirm_") {
        let action_name = action_name.to_string();
        return handle_confirm(
            call_params,
            services,
            db,
            tenant_id,
            ctx,
            dispatcher,
            store,
            &action_name,
        )
        .await;
    }

    // Fail-closed: writes always require an authenticated tenant.
    // tenant_id is the unwrapped authenticated principal — never from the payload.
    let tid = match tenant_id {
        Some(t) => t,
        None => {
            return json!({ "error": { "code": -32603, "message": "auth: tenant required" } });
        }
    };

    // Resolve the ActionDef by tool name across mcp-exposed services.
    let (svc, action) = match find_action(services, &tool_name) {
        Some(pair) => pair,
        None => {
            return json!({ "error": { "code": -32601, "message": "Method not found" } });
        }
    };

    // Derive the transition-level guard from the declared StateMachine (EXEC-02).
    // `.ok()` (not `?`): a non-transition action legitimately has no plan, so the
    // guard union then equals action.preconditions exactly (back-compatible).
    let plan = derive_transition_plan(svc, &action.name).ok();
    let transition_guard = plan.as_ref().and_then(|p| p.guard.as_deref());

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
    match dispatch_write(
        action,
        &args,
        tid,
        db,
        dispatcher,
        transition_guard,
        #[cfg(feature = "confirmation")]
        false,
    )
    .await
    {
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
        // Confirmation required: agent must use request_confirm_<action> first.
        #[cfg(feature = "confirmation")]
        Err(crate::Error::ConfirmationRequired(ref action_name)) => {
            json!({ "result": write_tool_error_result(json!({
                "error_kind": "confirmation_required",
                "message": format!("use request_confirm_{action_name} first"),
                "request_tool": format!("request_confirm_{action_name}")
            })) })
        }
        // Agent-safe variants: pass message through (no internal state in these strings).
        Err(ref e @ crate::Error::Validation(_)) | Err(ref e @ crate::Error::ActionNotFound(_)) => {
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

// ── Confirmation handlers (Plan 01 implementation) ────────────────────────────

/// Issues a confirmation token for a destructive action.
///
/// Flow: find action → validate inputs → re-evaluate guards (fail fast) →
/// generate CSPRNG token → store binding payload → return token.
///
/// The token is bound to `(tenant_id, action_name, record_id)` so
/// `handle_confirm` can reject cross-action/cross-record use (SC#4).
/// Token is never agent-supplied — always server-generated here.
#[cfg(feature = "confirmation")]
#[allow(clippy::too_many_arguments)]
pub async fn handle_request_confirm(
    call_params: Value,
    services: &[ServiceDef],
    db: &DatabaseConnection,
    tenant_id: Option<i64>,
    _ctx: &crate::McpContext,
    dispatcher: &WriteDispatcher,
    store: &dyn ferro_ai::ConfirmationStore,
    action_name: &str,
    ttl_secs: u64,
) -> Value {
    let tid = match tenant_id {
        Some(t) => t,
        None => {
            return json!({ "result": write_tool_error_result(json!({
                "error_kind": "execution_error",
                "message": "auth: tenant required"
            })) });
        }
    };

    let (_svc, action) = match find_action(services, action_name) {
        Some(pair) => pair,
        None => {
            return json!({ "error": { "code": -32601, "message": "Method not found" } });
        }
    };

    let args = call_params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    if let Err(msg) = validate_action_inputs(action, &args) {
        return json!({ "result": write_tool_error_result(json!({
            "error_kind": "validation",
            "message": msg
        })) });
    }

    // Re-evaluate guards before issuing token (fail fast — Pitfall 4).
    for guard_name in &action.preconditions {
        let passes = match (dispatcher.guard_evaluator)(guard_name, tid, &args, db).await {
            Ok(p) => p,
            Err(e) => {
                return json!({ "result": write_tool_error_result(json!({
                    "error_kind": "guard_denied",
                    "message": format!("precondition '{guard_name}' error: {e}")
                })) });
            }
        };
        if !passes {
            return json!({ "result": write_tool_error_result(json!({
                "error_kind": "guard_denied",
                "message": format!("precondition '{guard_name}' not met")
            })) });
        }
    }

    let token = generate_confirmation_token();
    let record_id = args.get("id").cloned().unwrap_or(Value::Null);
    let binding_payload = json!({
        "_binding": {
            "tenant_id": tid,
            "action_name": action_name,
            "record_id": record_id
        },
        "inputs": args
    });

    // NOTE (WR-04): retried request_confirm calls mint a new token each time
    // (the store is keyed on the token, not on the (tenant, action, record)
    // tuple), leaving the previous token live until TTL expiry. This is
    // acceptable for the v15.0 walking skeleton: each token is single-use,
    // TTL-bounded (≤600 s), and bound to (tenant_id, action_name, record_id) —
    // a write still requires exactly one valid confirm, and dispatch_write
    // idempotency prevents double-execution even on a race. Hardening path:
    // re-key the store on (tenant, action, record) when a persistent /
    // DB-backed store replaces InMemoryConfirmationStore.
    if let Err(_e) = store
        .request_confirmation(
            &token,
            binding_payload,
            std::time::Duration::from_secs(ttl_secs),
        )
        .await
    {
        return json!({ "result": write_tool_error_result(json!({
            "error_kind": "execution_error",
            "message": "failed to store confirmation token"
        })) });
    }

    let tool_result = CallToolResult::structured(json!({
        "confirmation_token": token,
        "expires_in_seconds": ttl_secs
    }));
    json!({ "result": tool_result })
}

/// Validates a confirmation token and executes the action exactly once.
///
/// Flow: read token from args → `store.confirm()` (single-use, None=expired) →
/// verify binding (tenant, action, record) → re-evaluate guards (live state) →
/// `dispatch_write(is_confirmed=true)` (bypasses D-08 seam) → return result.
#[cfg(feature = "confirmation")]
#[allow(clippy::too_many_arguments)]
pub async fn handle_confirm(
    call_params: Value,
    services: &[ServiceDef],
    db: &DatabaseConnection,
    tenant_id: Option<i64>,
    _ctx: &crate::McpContext,
    dispatcher: &WriteDispatcher,
    store: &dyn ferro_ai::ConfirmationStore,
    action_name: &str,
) -> Value {
    let tid = match tenant_id {
        Some(t) => t,
        None => {
            return json!({ "result": write_tool_error_result(json!({
                "error_kind": "execution_error",
                "message": "auth: tenant required"
            })) });
        }
    };

    let args = call_params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    let confirmation_token = match args.get("confirmation_token").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => {
            return json!({ "result": write_tool_error_result(json!({
                "error_kind": "validation",
                "message": "required field 'confirmation_token' missing"
            })) });
        }
    };

    // Consume the token (single-use). None = expired or already used.
    let stored_payload = match store.confirm(&confirmation_token).await {
        Ok(Some(payload)) => payload,
        Ok(None) => {
            return json!({ "result": write_tool_error_result(json!({
                "error_kind": "confirmation_expired",
                "message": "confirmation token expired or not found"
            })) });
        }
        Err(_) => {
            return json!({ "result": write_tool_error_result(json!({
                "error_kind": "execution_error",
                "message": "confirmation store error"
            })) });
        }
    };

    // Verify binding: tenant_id, action_name, record_id.
    let binding = &stored_payload["_binding"];

    if binding["tenant_id"].as_i64() != Some(tid) {
        return json!({ "result": write_tool_error_result(json!({
            "error_kind": "confirmation_mismatch",
            "message": "confirmation token does not belong to this tenant"
        })) });
    }

    if binding["action_name"].as_str() != Some(action_name) {
        return json!({ "result": write_tool_error_result(json!({
            "error_kind": "confirmation_mismatch",
            "message": "confirmation token is for a different action"
        })) });
    }

    let call_record_id = args.get("id");
    let stored_record_id = binding.get("record_id");
    if call_record_id != stored_record_id {
        return json!({ "result": write_tool_error_result(json!({
            "error_kind": "confirmation_mismatch",
            "message": "confirmation token is for a different record"
        })) });
    }

    let stored_inputs = &stored_payload["inputs"];

    // Find action for guard re-evaluation.
    let (svc, action) = match find_action(services, action_name) {
        Some(pair) => pair,
        None => {
            return json!({ "error": { "code": -32601, "message": "Method not found" } });
        }
    };

    // Derive the transition-level guard for the union guard set (EXEC-02),
    // mirroring handle_write_call.
    let plan = derive_transition_plan(svc, &action.name).ok();
    let transition_guard = plan.as_ref().and_then(|p| p.guard.as_deref());

    // Re-evaluate guards at confirm time (live DB state — T-220-03).
    for guard_name in &action.preconditions {
        let passes = match (dispatcher.guard_evaluator)(guard_name, tid, stored_inputs, db).await {
            Ok(p) => p,
            Err(_) => {
                return json!({ "result": write_tool_error_result(json!({
                    "error_kind": "guard_denied",
                    "message": "precondition not met at confirm time"
                })) });
            }
        };
        if !passes {
            return json!({ "result": write_tool_error_result(json!({
                "error_kind": "guard_denied",
                "message": format!("precondition '{guard_name}' not met at confirm time")
            })) });
        }
    }

    // Execute via dispatch_write with is_confirmed=true (bypasses D-08 seam).
    match dispatch_write(
        action,
        stored_inputs,
        tid,
        db,
        dispatcher,
        transition_guard,
        true,
    )
    .await
    {
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
            json!({ "result": write_tool_error_result(json!({
                "error_kind": "guard_denied",
                "message": msg
            })) })
        }
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

    /// Build a synthetic non-destructive `ActionDef` (no transition_trigger).
    /// Used in tests that need executor to run without the confirmation seam.
    fn update_action() -> ActionDef {
        ActionDef::new("update")
    }

    /// Build a minimal [`ServiceDef`] exposing all actions (including non-destructive `update`).
    ///
    /// Carries a state machine so `derive_transition_plan` yields the transition
    /// guard for `approve` (`submitted -> approve -> approved guard("is_manager")`).
    fn order_service_with_actions() -> ServiceDef {
        use ferro_projections::{DataType, FieldMeaning, StateMachine, Transition};
        ServiceDef::new("order")
            .mcp_exposed(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status)
            .state_machine(
                StateMachine::new("order_lifecycle")
                    .initial("draft")
                    .transition(Transition::new("draft", "submit", "submitted"))
                    .transition(
                        Transition::new("submitted", "approve", "approved").guard("is_manager"),
                    )
                    .transition(Transition::new("approved", "ship", "shipped")),
            )
            .action(approve_action())
            .action(submit_action())
            .action(update_action())
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
            overrides: std::collections::HashMap::new(),
        };

        let result = dispatch_write(
            &approve_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            None,
            #[cfg(feature = "confirmation")]
            false,
        )
        .await;

        assert!(
            matches!(result, Err(crate::Error::GuardFailed(_))),
            "expected Err(GuardFailed(_)), got: {result:?}"
        );
    }

    // ── EXEC-02 — transition-guard union + dedup in the live loop ─────────────

    /// A guard_evaluator returning `Ok(false)` for a transition that carries a
    /// `Transition.guard` causes `dispatch_write` to return `Err(GuardFailed(_))`
    /// and the executor never runs (state unchanged). The transition guard is
    /// passed explicitly here (as handle_write_call derives it from the plan).
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
            &submit_action(), // no action.preconditions
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            Some("is_manager"), // transition-level guard only
            #[cfg(feature = "confirmation")]
            false,
        )
        .await;

        assert!(
            matches!(result, Err(crate::Error::GuardFailed(_))),
            "expected Err(GuardFailed(_)) from the transition guard, got: {result:?}"
        );
    }

    /// A transition-level guard present on the PLAN but absent from
    /// `action.preconditions` is still evaluated — proves `Transition.guard` is
    /// enforced, not just `action.preconditions`. The evaluator records every
    /// guard name it sees; we assert `is_manager` was among them.
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

        // submit_action has NO preconditions; the guard comes only from the plan.
        let result = dispatch_write(
            &submit_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            Some("is_manager"),
            #[cfg(feature = "confirmation")]
            true, // bypass the D-08 confirmation seam for this transition action
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
    /// (`is_manager` on `approve`) fires the evaluator exactly ONCE (deduped by
    /// name).
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

        // approve_action has precondition("is_manager"); transition guard is also
        // "is_manager" — the union must dedup to a single evaluation.
        let result = dispatch_write(
            &approve_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            Some("is_manager"),
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

    // ── EXEC-03 — post-persist override hook registry ─────────────────────────

    /// An override registered for an action runs AFTER the base persist (observed
    /// via a counter the override increments). The base transition result is
    /// returned unchanged.
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
                    // The override sees the base persist result.
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

    /// With NO override registered, `dispatch_write` behaves exactly as before —
    /// the common path is declaration-only and untouched.
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
            #[cfg(feature = "confirmation")]
            true,
        )
        .await;

        assert!(result.is_ok(), "no-override path must succeed: {result:?}");
        assert_eq!(result.unwrap(), json!({ "status": "submitted" }));
    }

    /// An override returning `Err` causes `dispatch_write` to return `Err` — the
    /// error propagates. The base write's audit already happened (the override
    /// runs after persist + audit-of-base is recorded inside the executor's
    /// envelope); the error surfaces without panicking.
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
                Box::pin(async { Err(crate::Error::Validation("override failed".into())) })
            }),
        );

        let result = dispatch_write(
            &submit_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            None,
            #[cfg(feature = "confirmation")]
            true,
        )
        .await;

        assert!(
            matches!(result, Err(crate::Error::Validation(ref m)) if m == "override failed"),
            "override error must propagate, got: {result:?}"
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
            overrides: std::collections::HashMap::new(),
        };
        // Use non-destructive "update" action — confirmation seam does not fire for it.
        let success_params = json!({ "name": "update", "arguments": { "id": 1 } });
        let success_response = handle_write_call(
            success_params,
            &services,
            &db,
            Some(1),
            &ctx,
            &success_dispatcher,
            #[cfg(feature = "confirmation")]
            &ferro_ai::InMemoryConfirmationStore::new(),
            #[cfg(feature = "confirmation")]
            &crate::McpServerConfig::default(),
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
            overrides: std::collections::HashMap::new(),
        };
        let deny_params = json!({ "name": "approve", "arguments": { "id": 1 } });
        let deny_response = handle_write_call(
            deny_params,
            &services,
            &db,
            Some(1),
            &ctx,
            &deny_dispatcher,
            #[cfg(feature = "confirmation")]
            &ferro_ai::InMemoryConfirmationStore::new(),
            #[cfg(feature = "confirmation")]
            &crate::McpServerConfig::default(),
        )
        .await;

        let parsed_deny: CallToolResult = serde_json::from_value(deny_response["result"].clone())
            .expect("guard-denied result must parse as CallToolResult");
        assert_eq!(
            parsed_deny.is_error,
            Some(true),
            "guard-denied result must have is_error=true"
        );

        // --- confirmation_required envelope (feature = "confirmation") ---
        #[cfg(feature = "confirmation")]
        {
            let cfm_req_payload = write_tool_error_result(serde_json::json!({
                "error_kind": "confirmation_required",
                "message": "use request_confirm_approve first",
                "request_tool": "request_confirm_approve"
            }));
            let parsed_cfm_req: CallToolResult = serde_json::from_value(cfm_req_payload)
                .expect("confirmation_required envelope must parse");
            assert_eq!(
                parsed_cfm_req.is_error,
                Some(true),
                "confirmation_required must be isError:true"
            );

            let token_issued =
                serde_json::to_value(CallToolResult::structured(serde_json::json!({
                    "confirmation_token": "cfm_test",
                    "expires_in_seconds": 300
                })))
                .unwrap();
            let parsed_issued: CallToolResult =
                serde_json::from_value(token_issued).expect("token-issued envelope must parse");
            assert_eq!(
                parsed_issued.is_error,
                Some(false),
                "token-issued must be isError:false"
            );

            let expired = write_tool_error_result(serde_json::json!({
                "error_kind": "confirmation_expired",
                "message": "confirmation token expired or not found"
            }));
            let parsed_expired: CallToolResult =
                serde_json::from_value(expired).expect("confirmation_expired envelope must parse");
            assert_eq!(
                parsed_expired.is_error,
                Some(true),
                "confirmation_expired must be isError:true"
            );

            let mismatch = write_tool_error_result(serde_json::json!({
                "error_kind": "confirmation_mismatch",
                "message": "confirmation token is for a different action"
            }));
            let parsed_mismatch: CallToolResult = serde_json::from_value(mismatch)
                .expect("confirmation_mismatch envelope must parse");
            assert_eq!(
                parsed_mismatch.is_error,
                Some(true),
                "confirmation_mismatch must be isError:true"
            );

            let guard_denied_cfm = write_tool_error_result(serde_json::json!({
                "error_kind": "guard_denied",
                "message": "precondition 'is_manager' not met at confirm time"
            }));
            let parsed_guard_cfm: CallToolResult = serde_json::from_value(guard_denied_cfm)
                .expect("guard_denied-at-confirm envelope must parse");
            assert_eq!(
                parsed_guard_cfm.is_error,
                Some(true),
                "guard_denied-at-confirm must be isError:true"
            );
        }
    }
}

// ── RED confirmation tests (SC#1–#4 + guard-at-confirm; GREEN in Plan 01) ────

#[cfg(all(test, feature = "confirmation"))]
mod confirmation_tests {
    use super::*;
    use ferro_ai::InMemoryConfirmationStore;
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

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

    fn order_service() -> ServiceDef {
        use ferro_projections::{DataType, FieldMeaning, StateMachine, Transition};
        ServiceDef::new("order")
            .mcp_exposed(true)
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status)
            .state_machine(
                StateMachine::new("order_lifecycle")
                    .initial("draft")
                    .transition(Transition::new("draft", "submit", "submitted"))
                    .transition(
                        Transition::new("submitted", "approve", "approved").guard("is_manager"),
                    ),
            )
            .action(approve_action())
            .action(submit_action())
    }

    fn allow_dispatcher(exec_count: Arc<AtomicUsize>) -> WriteDispatcher {
        WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new(move |_, _, _, _| {
                exec_count.fetch_add(1, Ordering::SeqCst);
                Box::pin(async { Ok(json!({ "status": "approved" })) })
            }),
            overrides: std::collections::HashMap::new(),
        }
    }

    fn deny_guard_dispatcher() -> WriteDispatcher {
        WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(false) })),
            executor: Box::new(|_, _, _, _| {
                Box::pin(async { panic!("executor must not run when guard fails") })
            }),
            overrides: std::collections::HashMap::new(),
        }
    }

    // ── SC#1 — bare destructive write without token returns ConfirmationRequired ──

    /// SC#1: Calling `dispatch_write` on a destructive action (transition_trigger
    /// is_some) without a confirmation context returns `Err(ConfirmationRequired)`
    /// and does NOT invoke the executor.
    ///
    /// RED in Plan 00 — the D-08 seam wiring lands in Plan 01.
    #[tokio::test]
    async fn sc1_bare_destructive_without_token() {
        let db = setup_db().await;
        let exec_count = Arc::new(AtomicUsize::new(0));
        let dispatcher = allow_dispatcher(exec_count.clone());

        let result = dispatch_write(
            &submit_action(),
            &json!({"id": 1}),
            1,
            &db,
            &dispatcher,
            None,
            false, // is_confirmed = false → triggers seam
        )
        .await;

        assert!(
            matches!(result, Err(crate::Error::ConfirmationRequired(_))),
            "expected Err(ConfirmationRequired(_)), got: {result:?}"
        );
        assert_eq!(
            exec_count.load(Ordering::SeqCst),
            0,
            "executor must NOT run when confirmation is required"
        );
    }

    // ── SC#2 — two-step flow executes exactly once ────────────────────────────

    /// SC#2: request_confirm issues a token; confirm with that token executes
    /// exactly once; a second confirm with the same token returns an error
    /// (single-use), executor called exactly once.
    ///
    /// RED in Plan 00 — handle_request_confirm / handle_confirm stubs in Plan 01.
    #[tokio::test]
    async fn sc2_two_step_flow_executes_once() {
        let db = setup_db().await;
        let exec_count = Arc::new(AtomicUsize::new(0));
        let dispatcher = allow_dispatcher(exec_count.clone());
        let store = InMemoryConfirmationStore::new();
        let services = vec![order_service()];
        let ctx = crate::McpContext::default();

        // Step 1: request_confirm
        let req_response = handle_request_confirm(
            json!({ "name": "request_confirm_submit", "arguments": { "id": 1 } }),
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            &store,
            "submit",
            300,
        )
        .await;

        let token = req_response["result"]["structuredContent"]["confirmation_token"]
            .as_str()
            .expect("confirmation_token must be present in request_confirm response");

        // Step 2: first confirm — must execute
        let confirm_response = handle_confirm(
            json!({ "name": "confirm_submit", "arguments": { "confirmation_token": token, "id": 1 } }),
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            &store,
            "submit",
        )
        .await;

        assert_eq!(
            confirm_response["result"]["structuredContent"]["status"]
                .as_str()
                .unwrap_or(""),
            "ok",
            "first confirm must succeed with status=ok"
        );
        assert_eq!(
            exec_count.load(Ordering::SeqCst),
            1,
            "executor must fire exactly once"
        );

        // Step 3: second confirm with same token — must be rejected (single-use)
        let second_confirm = handle_confirm(
            json!({ "name": "confirm_submit", "arguments": { "confirmation_token": token, "id": 1 } }),
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            &store,
            "submit",
        )
        .await;

        let error_kind = second_confirm["result"]["structuredContent"]["error_kind"]
            .as_str()
            .unwrap_or("");
        assert!(
            error_kind == "confirmation_expired" || error_kind == "confirmation_mismatch",
            "second confirm must return expired/not-found, got: {second_confirm:?}"
        );
        assert_eq!(
            exec_count.load(Ordering::SeqCst),
            1,
            "executor must still have fired only once"
        );
    }

    // ── SC#3 — expired token rejected, not executed ───────────────────────────

    /// SC#3: Advance clock past TTL; confirm returns confirmation_expired,
    /// executor NOT called.
    ///
    /// Uses tokio manual-pause protocol: DB is connected first (with real clock),
    /// then clock is paused before the TTL timer is started (via request_confirm),
    /// then advanced. This avoids the `start_paused=true` pool-connect hang where
    /// the pool acquire timeout fires against a frozen clock.
    #[tokio::test]
    async fn sc3_expired_token_rejected() {
        // Connect DB BEFORE pausing the clock — pool acquire uses real tokio timers.
        let db = setup_db().await;
        let exec_count = Arc::new(AtomicUsize::new(0));
        let dispatcher = allow_dispatcher(exec_count.clone());
        let store = InMemoryConfirmationStore::new();
        let services = vec![order_service()];
        let ctx = crate::McpContext::default();

        // Pause clock AFTER DB is ready — TTL timer in request_confirmation will
        // register against the frozen clock and be controllable by advance().
        tokio::time::pause();

        // Request with a 5-second TTL
        let req_response = handle_request_confirm(
            json!({ "name": "request_confirm_submit", "arguments": { "id": 1 } }),
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            &store,
            "submit",
            5, // 5-second TTL for the test
        )
        .await;

        let token = req_response["result"]["structuredContent"]["confirmation_token"]
            .as_str()
            .expect("confirmation_token must be present");

        // Yield to let the TTL timer register
        tokio::task::yield_now().await;

        // Advance clock past TTL
        tokio::time::advance(Duration::from_secs(10)).await;

        // Yield to let the expiry task run
        for _ in 0..5 {
            tokio::task::yield_now().await;
        }

        // Confirm after TTL — must be rejected
        let confirm_response = handle_confirm(
            json!({ "name": "confirm_submit", "arguments": { "confirmation_token": token, "id": 1 } }),
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            &store,
            "submit",
        )
        .await;

        assert_eq!(
            confirm_response["result"]["structuredContent"]["error_kind"]
                .as_str()
                .unwrap_or(""),
            "confirmation_expired",
            "expired token must return confirmation_expired, got: {confirm_response:?}"
        );
        assert_eq!(
            exec_count.load(Ordering::SeqCst),
            0,
            "executor must NOT run after TTL expiry"
        );
    }

    // ── SC#4 — token mismatch (wrong action / wrong record) ──────────────────

    /// SC#4a: Token bound to action "submit" used on action "approve" returns
    /// confirmation_mismatch, executor NOT called.
    ///
    /// RED in Plan 00.
    #[tokio::test]
    async fn sc4_token_mismatch_action() {
        let db = setup_db().await;
        let exec_count = Arc::new(AtomicUsize::new(0));
        let dispatcher = allow_dispatcher(exec_count.clone());
        let store = InMemoryConfirmationStore::new();
        let services = vec![order_service()];
        let ctx = crate::McpContext::default();

        // Request confirm for "submit"
        let req_response = handle_request_confirm(
            json!({ "name": "request_confirm_submit", "arguments": { "id": 1 } }),
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            &store,
            "submit",
            300,
        )
        .await;

        let token = req_response["result"]["structuredContent"]["confirmation_token"]
            .as_str()
            .expect("token must be present");

        // Try to confirm "approve" with a token issued for "submit"
        let mismatch_response = handle_confirm(
            json!({ "name": "confirm_approve", "arguments": { "confirmation_token": token, "id": 1 } }),
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            &store,
            "approve", // different action
        )
        .await;

        assert_eq!(
            mismatch_response["result"]["structuredContent"]["error_kind"]
                .as_str()
                .unwrap_or(""),
            "confirmation_mismatch",
            "wrong-action token must return confirmation_mismatch, got: {mismatch_response:?}"
        );
        assert_eq!(
            exec_count.load(Ordering::SeqCst),
            0,
            "executor must NOT run on mismatch"
        );
    }

    /// SC#4b: Token bound to record id=1 used with id=2 returns
    /// confirmation_mismatch, executor NOT called.
    ///
    /// RED in Plan 00.
    #[tokio::test]
    async fn sc4_token_mismatch_record() {
        let db = setup_db().await;
        let exec_count = Arc::new(AtomicUsize::new(0));
        let dispatcher = allow_dispatcher(exec_count.clone());
        let store = InMemoryConfirmationStore::new();
        let services = vec![order_service()];
        let ctx = crate::McpContext::default();

        // Request confirm for record id=1
        let req_response = handle_request_confirm(
            json!({ "name": "request_confirm_submit", "arguments": { "id": 1 } }),
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            &store,
            "submit",
            300,
        )
        .await;

        let token = req_response["result"]["structuredContent"]["confirmation_token"]
            .as_str()
            .expect("token must be present");

        // Try to confirm with a different record id
        let mismatch_response = handle_confirm(
            json!({ "name": "confirm_submit", "arguments": { "confirmation_token": token, "id": 2 } }),
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            &store,
            "submit",
        )
        .await;

        assert_eq!(
            mismatch_response["result"]["structuredContent"]["error_kind"]
                .as_str()
                .unwrap_or(""),
            "confirmation_mismatch",
            "wrong-record token must return confirmation_mismatch, got: {mismatch_response:?}"
        );
        assert_eq!(
            exec_count.load(Ordering::SeqCst),
            0,
            "executor must NOT run on record mismatch"
        );
    }

    // ── Guard-at-confirm — guard denied at confirm time, not executed ─────────

    /// Guard passes at request_confirm; guard denies at confirm (live state changed).
    /// Confirm must return guard_denied without executing.
    ///
    /// RED in Plan 00.
    #[tokio::test]
    async fn sc_guard_denied_at_confirm_time() {
        let db = setup_db().await;
        let exec_count = Arc::new(AtomicUsize::new(0));
        let store = InMemoryConfirmationStore::new();
        let services = vec![order_service()];
        let ctx = crate::McpContext::default();

        // Guard passes during request_confirm
        let allow_dispatcher = WriteDispatcher {
            guard_evaluator: Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
            executor: Box::new({
                let count = exec_count.clone();
                move |_, _, _, _| {
                    count.fetch_add(1, Ordering::SeqCst);
                    Box::pin(async { Ok(json!({ "status": "approved" })) })
                }
            }),
            overrides: std::collections::HashMap::new(),
        };

        let req_response = handle_request_confirm(
            json!({ "name": "request_confirm_approve", "arguments": { "id": 1 } }),
            &services,
            &db,
            Some(1),
            &ctx,
            &allow_dispatcher,
            &store,
            "approve",
            300,
        )
        .await;

        let token = req_response["result"]["structuredContent"]["confirmation_token"]
            .as_str()
            .expect("token must be present");

        // Guard now denies at confirm time (live state changed)
        let deny_dispatcher = deny_guard_dispatcher();

        let confirm_response = handle_confirm(
            json!({ "name": "confirm_approve", "arguments": { "confirmation_token": token, "id": 1 } }),
            &services,
            &db,
            Some(1),
            &ctx,
            &deny_dispatcher,
            &store,
            "approve",
        )
        .await;

        assert_eq!(
            confirm_response["result"]["structuredContent"]["error_kind"]
                .as_str()
                .unwrap_or(""),
            "guard_denied",
            "guard-denied-at-confirm must return guard_denied, got: {confirm_response:?}"
        );
        assert_eq!(
            exec_count.load(Ordering::SeqCst),
            0,
            "executor must NOT run when guard denies at confirm"
        );
    }
}
