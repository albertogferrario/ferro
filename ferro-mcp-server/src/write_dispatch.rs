//! Write tool dispatch framing for the MCP endpoint.
//!
//! The transition-execution kernel (`WriteDispatcher`, `ExecutorFn`,
//! `GuardEvaluatorFn`, `OverrideFn`, `dispatch_write`) lives in
//! [`ferro_rs::write`]; this module is the MCP/JSON-RPC framing that calls into
//! it. `ferro-mcp-server` owns the channel-specific surface: scope check
//! (Phase 217), action lookup across mcp-exposed services, the confirmation
//! token seam (Phase 220), JSON-RPC error envelopes, and a spec-compliant
//! [`rmcp::model::CallToolResult`] result (D-06). It passes the literal channel
//! `"mcp"` into the kernel so the success-path audit reads `mcp.action.{name}`.

use ferro_audit::{AuditActor, AuditEntry, AuditTarget};
use ferro_projections::{
    derive_crud_plan, derive_transition_plan, ActionDef, CrudVerb, ServiceDef,
};
use ferro_rs::write::{dispatch_write, WriteDispatcher, WriteError};
// merged_guards is only consulted by the confirmation handlers' pre-check loop.
#[cfg(feature = "confirmation")]
use ferro_rs::write::merged_guards;
use rmcp::model::CallToolResult;
use sea_orm::DatabaseConnection;
use serde_json::{json, Value};

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
    let tool_name = call_params["name"].as_str().unwrap_or("").to_string();

    // Phase 242 (CRUD-05, SC#1 second half): write-ability policy Gate, fail-closed.
    // The host evaluates `.mcp_write_ability` via Gate::authorize_for and sets
    // `write_authorized`. Absent/false denies. This is a DEDICATED authorization signal,
    // intentionally separate from `evaluated_guards` (a visibility filter, not an auth gate).
    // The scope gate (read-key rejects write tools) already ran upstream in handle_tools_call.
    if ctx.write_authorized != Some(true) {
        return json!({
            "error": {
                "code": -32603,
                "message": "authorization: write ability denied"
            }
        });
    }

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

    // Phase 241: CRUD verb tools — detection runs BEFORE find_action so a CRUD verb call
    // never falls through to -32601. Gate each prefix on the matching opt-in flag so this
    // path only answers for verbs that are actually emitted as tools — an unflagged service
    // has no such tool, so its call falls through to the genuine -32601 "unknown tool" path.
    let crud_verb_opted_in = |s: &ServiceDef, prefix: &str| match prefix {
        "create_" => s.creatable,
        "update_" => s.updatable,
        "delete_" => s.deletable,
        _ => false,
    };
    for prefix in ["create_", "update_", "delete_"] {
        if let Some(svc_name) = tool_name.strip_prefix(prefix) {
            if let Some(svc) = services
                .iter()
                .find(|s| s.mcp_exposed && s.name == svc_name && crud_verb_opted_in(s, prefix))
            {
                // Fail-closed: writes require an authenticated tenant.
                let tid = match tenant_id {
                    Some(t) => t,
                    None => {
                        return json!({ "error": { "code": -32603, "message": "auth: tenant required" } });
                    }
                };

                let args = call_params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));

                let verb = match prefix {
                    "create_" => CrudVerb::Create,
                    "update_" => CrudVerb::Update,
                    _ => CrudVerb::Delete,
                };

                let plan = match derive_crud_plan(svc, verb, &args) {
                    Ok(p) => p,
                    Err(e) => {
                        return json!({ "result": write_tool_error_result(json!({
                            "error_kind": "validation",
                            "message": e.to_string()
                        })) });
                    }
                };

                // CRUD verbs are not ActionDefs; synthesize a minimal ActionDef whose
                // name is the tool name — drives the audit label + override-hook lookup
                // (e.g. "create_order"). The confirmation seam gates on CrudPlan::Delete
                // so create_/update_ execute immediately; delete_ returns
                // ConfirmationRequired unless called via the confirm_ handler.
                let crud_action = ferro_projections::ActionDef::new(&tool_name);

                return match dispatch_write(
                    &crud_action,
                    &args,
                    tid,
                    db,
                    dispatcher,
                    None, // transition_guard: CRUD has none
                    "mcp",
                    #[cfg(feature = "confirmation")]
                    false, // is_confirmed=false; bare delete triggers ConfirmationRequired
                    Some(&plan),
                )
                .await
                {
                    Ok(result) => {
                        let payload = json!({
                            "status": "ok",
                            "action": tool_name,
                            "result": result
                        });
                        let tool_result = CallToolResult::structured(payload);
                        json!({ "result": tool_result })
                    }
                    #[cfg(feature = "confirmation")]
                    Err(WriteError::ConfirmationRequired(ref name)) => {
                        json!({ "result": write_tool_error_result(json!({
                            "error_kind": "confirmation_required",
                            "message": format!("use request_confirm_{name} first"),
                            "request_tool": format!("request_confirm_{name}")
                        })) })
                    }
                    Err(WriteError::RecordNotFound) => {
                        json!({ "result": write_tool_error_result(json!({
                            "error_kind": "not_found",
                            "message": "record not found or already deleted"
                        })) })
                    }
                    Err(ref e @ WriteError::Validation(_))
                    | Err(ref e @ WriteError::ActionNotFound(_)) => {
                        json!({ "result": write_tool_error_result(json!({
                            "error_kind": "execution_error",
                            "message": e.to_string()
                        })) })
                    }
                    Err(_) => {
                        json!({ "result": write_tool_error_result(json!({
                            "error_kind": "execution_error",
                            "message": "write operation failed"
                        })) })
                    }
                };
            }
        }
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
    // The literal channel "mcp" makes the success-path audit `mcp.action.{name}`.
    match dispatch_write(
        action,
        &args,
        tid,
        db,
        dispatcher,
        transition_guard,
        "mcp",
        #[cfg(feature = "confirmation")]
        false,
        None,
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
        Err(WriteError::GuardFailed(ref msg)) => {
            // Audit the denial for forensic trail (PITFALLS §2 / D-05).
            // This denial audit lives in the MCP framing (not the kernel), so the
            // literal `mcp.action.{name}` is pinned here — the `:1308` regression
            // asserts the exact string. Do NOT parameterize this site.
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
        Err(WriteError::ConfirmationRequired(ref action_name)) => {
            json!({ "result": write_tool_error_result(json!({
                "error_kind": "confirmation_required",
                "message": format!("use request_confirm_{action_name} first"),
                "request_tool": format!("request_confirm_{action_name}")
            })) })
        }
        // Agent-safe variants: pass message through (no internal state in these strings).
        Err(ref e @ WriteError::Validation(_)) | Err(ref e @ WriteError::ActionNotFound(_)) => {
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

    let args = call_params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    // CRUD delete path: find_action returns None for CRUD verbs (they are not ActionDefs).
    // Locate the ServiceDef by stripping the "delete_" prefix instead.
    if let Some(svc_name) = action_name.strip_prefix("delete_") {
        let svc = match services
            .iter()
            .find(|s| s.mcp_exposed && s.name == svc_name && s.deletable)
        {
            Some(s) => s,
            None => {
                return json!({ "error": { "code": -32601, "message": "Method not found" } });
            }
        };
        // Guard pre-check: mirror the transition-action path (lines 474-491).
        // Phase 241: the synthesized CRUD delete verb has no preconditions, so this
        // loop is a correct no-op. Phase 242 wires mcp_write_ability / per-record
        // guards as preconditions here; this loop is the extension point.
        let crud_guards: Vec<String> = vec![]; // Phase 242 populates from svc preconditions
        let _ = svc; // svc used for .deletable + .mcp_exposed lookup and guard derivation above
        for guard_name in &crud_guards {
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
        return json!({ "result": tool_result });
    }

    let (svc, action) = match find_action(services, action_name) {
        Some(pair) => pair,
        None => {
            return json!({ "error": { "code": -32601, "message": "Method not found" } });
        }
    };

    if let Err(msg) = validate_action_inputs(action, &args) {
        return json!({ "result": write_tool_error_result(json!({
            "error_kind": "validation",
            "message": msg
        })) });
    }

    // Derive the transition-level guard so the token-issuance pre-check evaluates
    // the SAME merged_guards union as handle_write_call / dispatch_write (WR-02).
    // `.ok()`: a non-transition action has no plan, so the union equals
    // action.preconditions exactly (back-compatible).
    let plan = derive_transition_plan(svc, &action.name).ok();
    let transition_guard = plan.as_ref().and_then(|p| p.guard.as_deref());
    let guards = merged_guards(&action.preconditions, transition_guard);

    // Re-evaluate guards before issuing token (fail fast — Pitfall 4).
    for guard_name in &guards {
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

    // CRUD delete path: find_action returns None for CRUD verbs.
    // Locate the ServiceDef by stripping the "delete_" prefix instead.
    if let Some(svc_name) = action_name.strip_prefix("delete_") {
        let svc = match services
            .iter()
            .find(|s| s.mcp_exposed && s.name == svc_name && s.deletable)
        {
            Some(s) => s,
            None => {
                return json!({ "error": { "code": -32601, "message": "Method not found" } });
            }
        };

        let crud_plan = match derive_crud_plan(svc, CrudVerb::Delete, stored_inputs) {
            Ok(p) => p,
            Err(e) => {
                return json!({ "result": write_tool_error_result(json!({
                    "error_kind": "validation",
                    "message": e.to_string()
                })) });
            }
        };

        let crud_action = ferro_projections::ActionDef::new(action_name);
        return match dispatch_write(
            &crud_action,
            stored_inputs,
            tid,
            db,
            dispatcher,
            None, // transition_guard: CRUD has none
            "mcp",
            true, // is_confirmed=true — token was validated above
            Some(&crud_plan),
        )
        .await
        {
            Ok(result) => {
                let payload = json!({
                    "status": "ok",
                    "action": action_name,
                    "result": result
                });
                let tool_result = CallToolResult::structured(payload);
                json!({ "result": tool_result })
            }
            Err(WriteError::RecordNotFound) => {
                json!({ "result": write_tool_error_result(json!({
                    "error_kind": "not_found",
                    "message": "record not found or already deleted"
                })) })
            }
            Err(WriteError::GuardFailed(ref msg)) => {
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
        };
    }

    // Find action for guard re-evaluation (transition path).
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
    // Use the SAME merged_guards union as dispatch_write (WR-02) so the confirm
    // pre-check matches the guard set the actual write enforces below.
    let guards = merged_guards(&action.preconditions, transition_guard);
    for guard_name in &guards {
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
    // Literal channel "mcp" → success-path audit `mcp.action.{name}`.
    match dispatch_write(
        action,
        stored_inputs,
        tid,
        db,
        dispatcher,
        transition_guard,
        "mcp",
        true,
        None,
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
        Err(WriteError::GuardFailed(ref msg)) => {
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
        // orders table — required by CRUD dispatch tests (Phase 241-03).
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "CREATE TABLE IF NOT EXISTS orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                status TEXT NOT NULL DEFAULT 'draft',
                amount TEXT,
                created_at TEXT DEFAULT (datetime('now')),
                deleted_at TEXT
            )"
            .to_string(),
        ))
        .await
        .expect("create orders table");
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
            "mcp",
            false, // is_confirmed = false → triggers seam
            None,
        )
        .await;

        assert!(
            matches!(result, Err(WriteError::ConfirmationRequired(_))),
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

    // ── Phase 241-03 CRUD framing tests ──────────────────────────────────────

    /// ServiceDef fixture for CRUD tests — creatable + updatable + deletable,
    /// mcp_write_ability set so validate() passes. Mirrors the Plan 02 fixture.
    fn crud_order_service() -> ServiceDef {
        use ferro_projections::{DataType, FieldMeaning};
        ServiceDef::new("order")
            .mcp_exposed(true)
            .creatable(true)
            .updatable(true)
            .deletable(true)
            .mcp_write_ability("manage-orders")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("amount", DataType::String, FieldMeaning::FreeText)
    }

    /// VALIDATION #16 / D-10: a create_order call returns a well-formed
    /// structured envelope with status == "ok" and a result object.
    #[tokio::test]
    async fn crud_result_structured_envelope() {
        let db = setup_db().await;
        let exec_count = Arc::new(AtomicUsize::new(0));
        let dispatcher = allow_dispatcher(exec_count);
        let store = InMemoryConfirmationStore::new();
        let services = vec![crud_order_service()];
        let ctx = crate::McpContext::default();

        let response = handle_write_call(
            json!({ "name": "create_order", "arguments": { "amount": "10.00" } }),
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            &store,
            &crate::McpServerConfig::default(),
        )
        .await;

        assert_eq!(
            response["result"]["structuredContent"]["status"]
                .as_str()
                .unwrap_or(""),
            "ok",
            "create_order must return status=ok; got: {response:?}"
        );
        assert!(
            response["result"]["structuredContent"]
                .get("result")
                .is_some(),
            "structured envelope must have a 'result' field; got: {response:?}"
        );
    }

    /// A bare delete_order without a confirmation token returns confirmation_required
    /// naming request_confirm_delete_order as the next tool (T-241-10).
    #[tokio::test]
    async fn delete_bare_call_returns_confirmation_required() {
        let db = setup_db().await;
        let exec_count = Arc::new(AtomicUsize::new(0));
        let dispatcher = allow_dispatcher(exec_count);
        let store = InMemoryConfirmationStore::new();
        let services = vec![crud_order_service()];
        let ctx = crate::McpContext::default();

        let response = handle_write_call(
            json!({ "name": "delete_order", "arguments": { "id": 1 } }),
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            &store,
            &crate::McpServerConfig::default(),
        )
        .await;

        assert_eq!(
            response["result"]["structuredContent"]["error_kind"]
                .as_str()
                .unwrap_or(""),
            "confirmation_required",
            "bare delete must return confirmation_required; got: {response:?}"
        );
        assert_eq!(
            response["result"]["structuredContent"]["request_tool"]
                .as_str()
                .unwrap_or(""),
            "request_confirm_delete_order",
            "request_tool must name request_confirm_delete_order; got: {response:?}"
        );
    }

    /// VALIDATION #14: request_confirm_delete_order -> token -> confirm_delete_order
    /// soft-deletes the row (deleted_at IS NOT NULL).
    #[tokio::test]
    async fn delete_two_step_flow() {
        let db = setup_db().await;
        // Insert a row to delete.
        db.execute(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "INSERT INTO orders (id, status, amount) VALUES (1, 'draft', '5.00')".to_string(),
        ))
        .await
        .expect("insert test order");

        let exec_count = Arc::new(AtomicUsize::new(0));
        let dispatcher = allow_dispatcher(exec_count);
        let store = InMemoryConfirmationStore::new();
        let services = vec![crud_order_service()];
        let ctx = crate::McpContext::default();

        // Step 1: request_confirm_delete_order
        let req_response = handle_request_confirm(
            json!({ "name": "request_confirm_delete_order", "arguments": { "id": 1 } }),
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            &store,
            "delete_order",
            300,
        )
        .await;

        let token = req_response["result"]["structuredContent"]["confirmation_token"]
            .as_str()
            .expect("confirmation_token must be present");

        // Step 2: confirm_delete_order
        let confirm_response = handle_confirm(
            json!({ "name": "confirm_delete_order", "arguments": { "confirmation_token": token, "id": 1 } }),
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            &store,
            "delete_order",
        )
        .await;

        assert_eq!(
            confirm_response["result"]["structuredContent"]["status"]
                .as_str()
                .unwrap_or(""),
            "ok",
            "confirm_delete must return status=ok; got: {confirm_response:?}"
        );

        // Verify soft-delete: deleted_at IS NOT NULL
        let row = db
            .query_one(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT deleted_at FROM orders WHERE id = 1".to_string(),
            ))
            .await
            .expect("query ok")
            .expect("row must exist");
        let deleted_at: Option<String> = row.try_get("", "deleted_at").ok();
        assert!(
            deleted_at.is_some() && deleted_at.as_deref().unwrap_or("") != "",
            "deleted_at must be set after soft-delete; got: {deleted_at:?}"
        );
    }

    /// T-241-11: a token issued for id=1 cannot delete id=2 (binding mismatch).
    #[tokio::test]
    async fn delete_wrong_record_token_rejected() {
        let db = setup_db().await;
        let exec_count = Arc::new(AtomicUsize::new(0));
        let dispatcher = allow_dispatcher(exec_count);
        let store = InMemoryConfirmationStore::new();
        let services = vec![crud_order_service()];
        let ctx = crate::McpContext::default();

        // Issue token for id=1
        let req_response = handle_request_confirm(
            json!({ "name": "request_confirm_delete_order", "arguments": { "id": 1 } }),
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            &store,
            "delete_order",
            300,
        )
        .await;

        let token = req_response["result"]["structuredContent"]["confirmation_token"]
            .as_str()
            .expect("token must be present");

        // Attempt to confirm with id=2 — must be rejected
        let mismatch_response = handle_confirm(
            json!({ "name": "confirm_delete_order", "arguments": { "confirmation_token": token, "id": 2 } }),
            &services,
            &db,
            Some(1),
            &ctx,
            &dispatcher,
            &store,
            "delete_order",
        )
        .await;

        assert_eq!(
            mismatch_response["result"]["structuredContent"]["error_kind"]
                .as_str()
                .unwrap_or(""),
            "confirmation_mismatch",
            "wrong-record token must return confirmation_mismatch; got: {mismatch_response:?}"
        );
    }
}
