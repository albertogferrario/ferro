# Write Kernel

The write kernel is a single, channel-agnostic execution pipeline for guarded state-transition writes. One call — `dispatch_write` — backs both the MCP write path and the visual/form write path. There is no per-channel executor: the same guard re-evaluation, idempotency, audit, and override logic runs regardless of which surface initiated the write.

A channel (MCP, web form, …) is responsible only for *framing*: resolving the target `ServiceDef`/`ActionDef`, reading the authenticated tenant, deriving the transition guard, and mapping `WriteError` into its own response shape. Everything load-bearing lives in the kernel.

## `dispatch_write`

```rust
use ferro::write::dispatch_write;

let result = dispatch_write(
    action,            // &ActionDef — the resolved action
    &inputs,           // &Value — validated/opaque inputs (kernel reads only "id"/"idempotency_key")
    tenant_id,         // i64 — from the authenticated principal, never from the payload
    db,                // &DatabaseConnection
    &dispatcher,       // &WriteDispatcher
    transition_guard,  // Option<&str> — the transition-level guard, derived from the StateMachine
    "web",             // &str — the audit channel prefix
    // is_confirmed: bool — present only under the `confirmation` feature
)
.await;
```

The pipeline runs in a fixed order:

1. **Guard re-evaluation** — every guard in the union of `action.preconditions` and the transition-level guard is re-checked against live database state via the dispatcher's guard evaluator. Fail-closed: a guard returning `Ok(false)` or any `Err` returns `Err(WriteError::GuardFailed)` and the executor never runs.
2. **Idempotency check** — if `inputs["idempotency_key"]` is present and a result is already stored for `(tenant_id, key)`, the stored result is returned without re-executing or re-auditing.
3. **Confirmation seam** (`confirmation` feature) — a destructive action (one with a `transition_trigger`) called with `is_confirmed = false` returns `Err(WriteError::ConfirmationRequired)`; the executor never fires.
4. **Execute** — the dispatcher's executor performs the tenant-scoped mutation.
5. **Seal idempotency** — the result is stored under `(tenant_id, idempotency_key)`.
6. **Seal audit** — an append-only audit entry is written.
7. **Post-persist override** — the registered override hook (if any) runs.

The kernel owns its own `WriteError`. It does not depend on any channel's error type; each channel maps `WriteError` into its own error at the framing boundary.

## Guard re-evaluation is server-side and fail-closed

Authorization at call time uses **live database state**, never a cached visibility decision. The list-time visibility cache (`ctx.evaluated_guards`, used by renderers to decide which action affordances to show) is intentionally not consulted here. A caller may bypass `tools/list` entirely and call a write directly; only the live re-evaluation in step 1 prevents a guard-bypass privilege escalation.

The guard evaluator is fail-closed: an unrecognized guard name is denied, not silently allowed.

```rust
use ferro::write::WriteDispatcher;

let dispatcher = WriteDispatcher::new(
    // executor: perform the tenant-scoped mutation
    Box::new(|action_name, inputs, tenant_id, db| {
        let action_name = action_name.to_string();
        let id = inputs["id"].as_i64();
        let db = db.clone();
        Box::pin(async move {
            // find_for_tenant: filter by id AND tenant_id — None => cross-tenant denial.
            // ... perform the mutation, return an audit-safe Value ...
            Ok(serde_json::json!({ "id": id, "status": "submitted" }))
        })
    }),
    // guard_evaluator: re-check preconditions against LIVE DB state, fail-closed
    Box::new(|guard_name, tenant_id, _inputs, db| {
        let guard_name = guard_name.to_string();
        let db = db.clone();
        Box::pin(async move {
            match guard_name.as_str() {
                "is_manager" => Ok(check_is_manager(tenant_id, &db).await),
                // Fail-closed: an unknown guard is denied, never allowed.
                _ => Err(ferro::write::WriteError::GuardFailed(format!(
                    "unknown guard '{guard_name}': no evaluator registered"
                ))),
            }
        })
    }),
);
```

A guard appearing on both the action precondition and the transition guard — as `is_manager` does on the order projection — is evaluated exactly once (deduplicated by name).

## The executor and guard contracts

| Callback | Signature | Responsibility |
|----------|-----------|----------------|
| `ExecutorFn` | `(action_name, inputs, tenant_id, db) -> WriteResult<Value>` | Perform the tenant-scoped mutation; return an audit-safe `Value` |
| `GuardEvaluatorFn` | `(guard_name, tenant_id, inputs, db) -> WriteResult<bool>` | Re-check one precondition against live DB state; `Ok(true)` allows |
| `OverrideFn` | `(action_name, inputs, tenant_id, db, base_result) -> WriteResult<()>` | App-specific side effect, run after the base persist |

The executor's returned `Value` is stored **verbatim** in the append-only audit log. It must not contain secrets, credentials, or PII — return only audit-safe fields (typically identifiers and status values, e.g. `{"id": 42, "status": "approved"}`). The kernel does not scrub the payload; the executor is the enforcement point.

`tenant_id` always comes from the authenticated principal, never from the call payload. The executor's `find_for_tenant(id, tenant_id)` returning `None` is the cross-tenant denial primitive: a row owned by another tenant resolves to `None` and the call fails before any mutation.

## Post-persist override

An app can register a post-persist hook per action with `with_override`. The hook runs strictly after the base persist, so it can chain related-record writes keyed off the just-persisted state.

```rust
use ferro::write::WriteDispatcher;

let dispatcher = WriteDispatcher::new(executor, guard_evaluator)
    .with_override(
        "submit",
        Box::new(|_action, _inputs, _tenant, _db, base_result| {
            // base_result is the executor's output; chain side effects here.
            let _persisted_status = &base_result["status"];
            Box::pin(async move { Ok(()) })
        }),
    );
```

`with_override` is a consuming builder (`mut self -> Self`); register one hook per action name. An action with no registered hook is the common path — the override seam adds nothing.

### Ordering guarantee

The override runs **after** the base persist's idempotency key and audit entry are sealed (steps 5–6 above). This ordering is a durability guarantee, not an implementation detail:

- The base persist is already durable when the hook fires.
- There is no surrounding transaction, so an override returning `Err` does **not** roll back the base persist.
- The base persist's audit entry and idempotency key remain committed even when the override fails.

The consequence: a failing override surfaces its error to the caller, but it can never leave an unaudited or replayable write. The "every successful base persist is audited" invariant holds regardless of what an app-specific side effect does afterwards. An override author must therefore treat the base transition as already applied when the hook fires.

## The `channel` audit parameter

The `channel` argument parameterizes the audit reason prefix: a successful write records `format!("{channel}.action.{name}")`. Each channel passes its own literal — the MCP path passes `"mcp"`, the visual surface passes `"web"` — so a write is never auditable without a channel tag, and the audit log distinguishes which surface drove each transition.

## `WriteError`

| Variant | Meaning |
|---------|---------|
| `WriteError::Database(String)` | A database operation failed. May contain SQL fragments — channels MUST redact it before returning to an untrusted caller |
| `WriteError::Serialization(serde_json::Error)` | JSON (de)serialization of an idempotency result or persist payload failed |
| `WriteError::GuardFailed(String)` | A precondition or transition guard returned false or errored; never discloses which guard or what state it checked |
| `WriteError::Validation(String)` | Input validation failed (required field missing, wrong type, etc.) |
| `WriteError::ActionNotFound(String)` | The resolved action name is not found in any exposed `ServiceDef` |
| `WriteError::ConfirmationRequired(String)` | A destructive action was called without a valid confirmation context (only under the `confirmation` feature) |

`WriteResult<T>` is the alias `Result<T, WriteError>`.

## Reference

| Item | Description |
|------|-------------|
| `dispatch_write` | The channel-agnostic write pipeline: guard re-eval → idempotency → confirm seam → persist → audit → override |
| `WriteDispatcher` | Holds the executor, guard evaluator, and per-action override registry |
| `WriteDispatcher::new(executor, guard_evaluator)` | Construct a dispatcher with an empty override registry |
| `WriteDispatcher::with_override(action, hook)` | Register a post-persist override hook (consuming builder) |
| `ExecutorFn` | App-registered write executor; performs the tenant-scoped mutation |
| `GuardEvaluatorFn` | App-registered guard evaluator; re-checks a precondition against live DB state |
| `OverrideFn` | App-registered post-persist hook; runs after audit + idempotency are sealed |
| `WriteError` | Self-contained kernel error enum |
| `WriteResult<T>` | `Result<T, WriteError>` |

## See also

- [Transition Planning](transition-planning.md) — deriving the write target (`to_state`, guard) from the StateMachine.
- [Service Projections](projections.md) — the `ServiceDef` that declares the actions and state machine.
- [Agent-Operable App (Consumer MCP)](agent-operable-app.md) — the MCP channel that drives this kernel.
- [Actions](../json-ui/actions.md) — the `POST /{service}/{action}` affordances the visual channel handles.
