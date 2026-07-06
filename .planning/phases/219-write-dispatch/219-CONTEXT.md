# Phase 219: Write Dispatch - Context

**Gathered:** 2026-06-13
**Status:** Ready for planning
**Mode:** `--auto` (decisions are recommended defaults grounded in the v15.0 research docs + phase success criteria; logged in `219-DISCUSSION-LOG.md`)

<domain>
## Phase Boundary

Make the write tools rendered in Phase 218 **callable**: an agent invokes a write tool and the server executes the action **tenant-scoped**, with the action's guard **re-evaluated server-side at execution time against live DB state**, idempotency enforced, an audit entry recorded, and a spec-compliant `CallToolResult::structured` result returned.

In scope (AMCP-04):
- `dispatch_write()` (new) + `handle_write_call()` routing in `ferro-mcp-server` — replaces the Phase 218 `-32601` placeholder for non-`list_` tools.
- Server-side guard re-evaluation at call time (the agent is never trusted) — independent of the 218 `tools/list` visibility filter.
- Tenant-scoped execution via the `TenantScoped` contract; cross-tenant write blocked.
- Idempotency: an `idempotency_key` makes a retried call a no-op replay.
- Per-call audit log entry.
- `CallToolResult::structured` result for every write response.
- Sample-app wiring of a concrete tenant-scoped executor + guard evaluator for ≥1 action so SC#1–#5 are testable end-to-end (synthetic validation — the milestone's in-repo validation, not gestiscilo migration).

**Out of scope (later phases):** confirmation gating + `confirm_<action>` tools + `ferro-ai` dependency — Phase 220 (AMCP-05); inbound NL classification loop — Phase 221 (AMCP-06). 219 leaves a clean seam for the 220 confirmation wrapper but adds no confirmation logic and no `ferro-ai` dep.

</domain>

<decisions>
## Implementation Decisions

### Action execution mechanism (D-01)
- **D-01:** Actions execute through an **app-registered callback**, not generic SQL and not the app's HTTP stack. Signature (per ARCHITECTURE build-order Phase 3): an async executor `async fn(action_name: &str, inputs: &Value, tenant_id: i64, db: &DatabaseConnection) -> Result<Value, Error>`. `ferro-mcp-server` stays projection-agnostic — it knows *which* action and *which* tenant, the app knows *how* to mutate. This resolves the REQUIREMENTS "out of scope: routing through the app's HTTP stack if a direct callback suffices" → a direct callback suffices. Exact registration shape (a trait object `&dyn ActionExecutor` vs a boxed fn, and where it is held — `McpServerConfig` vs a new `WriteDispatcher` param threaded into `handle_tools_call`) is Claude's discretion / research-resolved.

### Server-side guard re-evaluation (D-02) — THE security mechanism
- **D-02:** `dispatch_write` re-evaluates **every** `action.precondition` at execution time via an app-registered `GuardEvaluator` (`async fn(guard_name: &str, tenant_id: i64, inputs: &Value, db: &DatabaseConnection) -> Result<bool, Error>`), against **live DB state**, BEFORE invoking the executor. Fail-closed: a guard returning `false` OR erroring → the call returns an MCP error, not a successful execution (SC#1). This is **independent of the Phase 218 `tools/list` visibility filter** — a direct `tools/call` on a guarded action with a failing guard is rejected here even though the listing filter is never consulted at call time (PITFALLS §2 — the structural fix for the guard-bypass class). `McpContext.evaluated_guards` (the 218 visibility map) is NOT trusted as the execution gate. Guard evaluation receives the validated inputs (incl. the record identifier) so record-scoped guards (`is_owner`, `has_items`) can query live state.

### Tenant-scoped writes (D-03)
- **D-03:** Tenant scoping for writes is enforced by the `find_for_tenant(id, tenant_id)` contract (`framework/src/tenant/scoped.rs`): the executor (and/or guard evaluator) loads the target record via `TenantScoped::find_for_tenant` and a `None` result (record not owned by the calling tenant) → denial, before any mutation. A cross-tenant write fixture (tenant A targeting tenant B's record) asserts failure, not silent success (SC#2). No new write methods on `TenantScoped` are required — find-then-mutate is the contract; research may suggest an optional write helper but it is not load-bearing. `tenant_id` is always the authenticated principal's, never sourced from the tool-call payload (reuses the 217/dispatch invariant).

### Idempotency (D-04)
- **D-04:** Every write tool accepts an optional `idempotency_key` (string) read from the call `arguments`. Storage: a new **framework-owned `mcp_idempotency_keys` table** (columns: `id`, `tenant_id`, `idempotency_key`, `result` (serialized), `created_at`; UNIQUE on `(tenant_id, idempotency_key)`), consumer runs the migration — mirrors the Phase 217 `mcp_api_keys` ownership pattern. Flow: first call with a key executes the action and stores `(tenant_id, key) → result`; a second call with the same key returns the stored result **without re-executing** (SC#3: exactly one DB write after two identical calls). Absent key = no idempotency guard (each call executes). Research: whether to inject `idempotency_key` as an advertised optional property into the write-tool `inputSchema` (a small `render_action_tool` tweak) so agents know to send it, vs accept-without-advertising.

### Audit log (D-05)
- **D-05:** Each write call produces one append-only audit entry containing at minimum: tool name, tenant ID, action name, and the relevant parameter IDs (e.g. the record identifier), recoverable after the fact (SC#4). **Research must first evaluate `ferro-audit`** (the existing append-only structured before/after audit crate) for fit — reuse it if a per-action event maps cleanly; otherwise a lightweight framework-owned `mcp_audit_log` table (consumer migration, mirroring the idempotency table). Prefer reuse of `ferro-audit` per the no-duplicate-control-surface convention; fall back to a minimal table only if `ferro-audit`'s before/after entity shape is a poor fit for a per-call action event. Record the audit entry on every outcome that mutates (success), and ideally on guard-denied/idempotent-replay too (research to confirm scope vs SC#4's literal "each write tool call").

### Result construction (D-06)
- **D-06:** `CallToolResult::structured(payload)` (the Phase 205 constructor) is the result for **every** write response — success, guard-denied, validation error, idempotent replay, and (future) pending-confirmation. No hand-built bare `content[]` arrays (SC#5). Guard denial / validation failure → a structured **error** result (`isError`-flagged or a structured error payload — exact shape research-resolved, consistent with the Phase 205 read-path shape and the 217 `-32603`/scope-error conventions).

### handle_write_call routing (D-07)
- **D-07:** `handle_tools_call` routes a non-`list_` tool name to `handle_write_call` → `dispatch_write`, replacing the Phase 218 `-32601` placeholder. The 217/218 **scope gate stays in front**: a `read`-scoped key still gets the scope error before any dispatch_write runs. Order at call time: scope check (217) → resolve `ActionDef` by name → validate inputs against `ActionDef.inputs` → re-evaluate guards (D-02) → idempotency check (D-04) → execute callback (D-01) → audit (D-05) → structured result (D-06).

### Confirmation seam (D-08)
- **D-08:** 219 does NOT implement confirmation and does NOT add a `ferro-ai` dependency. `dispatch_write` is structured so the Phase 220 confirmation wrapper can intercept destructive actions (`transition_trigger.is_some()`) before execution. Keep the seam clean; do not pre-wire `ConfirmationStore`.

### Sample-app validation wiring (D-09)
- **D-09:** `ferro-mcp-server` provides `dispatch_write` + the executor/guard-evaluator registration hooks. The **sample `app`** registers a concrete tenant-scoped executor + guard evaluator for at least one action (building on the 217 code-review-fix that already wired auth + scope through `app/src/controllers/mcp.rs` and `bearer_auth.rs`), so the five success criteria are provable end-to-end with synthetic fixtures. This is larger than 217/218's app touch because it needs a real executor — flag for the planner whether the phase warrants a split (framework machinery vs sample-app wiring + the cross-tenant/idempotency/audit fixtures).

### Claude's Discretion
- Exact registration API (trait object vs boxed async fn; held in `McpServerConfig` vs a new dispatcher param).
- Exact serialized shape of the stored idempotency `result` and the audit entry record.
- Whether `idempotency_key` is advertised in the write-tool `inputSchema`.
- Error-result envelope shape for guard-denied/validation-failed (consistent with Phase 205 + 217 conventions).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### v15.0 design
- `.planning/research/ARCHITECTURE.md` §"Data Flow → Write path (new in v15.0)" + §"Build Order → Phase 3 — Write dispatch" — the primary design (callback signature, `dispatch_write`, `handle_write_call`, `write_dispatch.rs`).
- `.planning/research/PITFALLS.md` §1 (cross-tenant write leak — SC#2 fixture), **§2 (SERVER-SIDE GUARD BYPASS — the load-bearing pitfall this phase must structurally prevent; guards re-evaluated at execution, not advisory)**, §5 (idempotency keys + destructive-write retries — SC#3).
- `.planning/research/FEATURES.md` — "Idempotency on write tools" row (maps to UNIQUE / INSERT-OR-IGNORE), "MCP-specific handler code per action" anti-pattern (write dispatch runs the same action contract, not a parallel impl), "Streaming/SSE write results" anti-pattern (synchronous structured result; long writes → ferro-queue job id).
- `.planning/REQUIREMENTS.md` — AMCP-04 (the requirement this phase closes); AMCP-05/06 deferred to 220/221; "Out of Scope: routing write dispatch through the app's HTTP stack if a direct callback suffices" (resolved by D-01).

### Phase 217/218 foundation
- `.planning/phases/217-tenant-context-per-tenant-api-key-auth/217-CONTEXT.md` + `217-SECURITY.md` — `McpContext`, the scope gate (stays in front of dispatch_write), the `mcp_api_keys` framework-table ownership pattern (template for `mcp_idempotency_keys`).
- `.planning/phases/218-write-tool-rendering-from-actiondef/218-CONTEXT.md` — write-tool names/schemas/`ActionDef` mapping `dispatch_write` resolves a call against.

### Code touch-points (read before editing)
- `ferro-mcp-server/src/dispatch.rs` — the read `dispatch()` analog: `tenant_id: Option<i64>`, fail-closed tenant predicate. Mirror its tenant discipline for the write path.
- `ferro-mcp-server/src/jsonrpc.rs` — `handle_tools_call` (the `-32601` write placeholder at ~line 64/90 to replace; the 217 scope gate in front; the Phase 205 `CallToolResult::structured` constructor + `tools_call_result_parses_as_valid_mcp_content` test to reuse for SC#5).
- `ferro-mcp-server/src/config.rs` — `McpServerConfig` (candidate home for the executor/guard-evaluator registration).
- `ferro-mcp-server/src/error.rs` — add `GuardFailed(String)`, `ActionNotFound(String)` (per ARCHITECTURE).
- `framework/src/tenant/scoped.rs` — `TenantScoped::find_for_tenant(id, tenant_id)` (the SC#2 enforcement contract).
- `ferro-audit/src/lib.rs` — evaluate for the SC#4 audit entry (D-05).
- `ferro-ai/src/confirmation/mod.rs` — `ConfirmationStore` trait (the 220 seam; do NOT wire in 219).
- `ferro-projections/src/action.rs` — `ActionDef { name, inputs, preconditions, transition_trigger }` (read-only; the call is resolved against this).
- `app/src/controllers/mcp.rs` + `app/src/middleware/bearer_auth.rs` — the 217-CR-fix-wired auth/scope path; extend `mcp.rs` to invoke the write path and register the sample executor + guard evaluator (D-09).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `dispatch()` (read, `dispatch.rs`): the structural analog — `Option<i64>` tenant, fail-closed, never reads tenant from payload. `dispatch_write` mirrors this discipline.
- `CallToolResult::structured` + `tools_call_result_parses_as_valid_mcp_content` (jsonrpc.rs, Phase 205): the result constructor + strict-deser guard to reuse for every write result (SC#5).
- `TenantScoped::find_for_tenant(id, tenant_id) -> Option<Self>` (framework): the cross-tenant denial primitive (None → not yours) — SC#2.
- 217 `mcp_api_keys` migration + framework-table-ownership pattern: the template for `mcp_idempotency_keys` (and a possible `mcp_audit_log`).
- `ferro-audit` (append-only structured audit) and `ferro-ai::ConfirmationStore` (`request_confirmation`/`confirm`, `InMemoryConfirmationStore::new(Duration)`): the former a D-05 candidate, the latter the 220 seam.
- 217/218 scope gate in `handle_tools_call`: already classifies non-`list_` names as write tools — `handle_write_call` slots in where the `-32601` is returned today.

### Established Patterns
- Tenant value always from the authenticated principal, never the payload (217/dispatch invariant) — extend to writes.
- Framework owns canonical table schemas; consumer runs migrations (217 `mcp_api_keys`).
- Single-source dispatch: the MCP write path runs the same `ActionDef` contract, not a parallel hand-authored handler (FEATURES anti-pattern).

### Integration Points
- `handle_tools_call` in `jsonrpc.rs` — where `handle_write_call`/`dispatch_write` route in behind the scope gate.
- The executor + guard-evaluator registration boundary between `ferro-mcp-server` and the consumer/sample app.
- The sample `app` MCP controller (`mcp.rs`) — registers the concrete tenant-scoped executor + guard evaluator for the SC#1–#5 fixtures.

</code_context>

<specifics>
## Specific Ideas

- "The agent is never trusted" (AMCP-04) is the phase's spine: guards re-evaluated server-side at execution, tenant from principal, cross-tenant denied, idempotent against retries. A reviewer must confirm guard re-evaluation is on the `tools/call` execution path and reads live state — not the cached 218 visibility map.
- Long-running writes are out of band: a genuinely slow action routes through `ferro-queue` and returns a job id (FEATURES anti-pattern note) — not in 219 scope, but `dispatch_write` should return promptly with a structured result.

</specifics>

<deferred>
## Deferred Ideas

- Confirmation gating for destructive actions + synthesized `confirm_<action>` tools + `ferro-ai`/`ConfirmationStore` + TTL — Phase 220 (AMCP-05). 219 only leaves the seam.
- Inbound NL classification loop (`Classifier<ToolSelection>` → tool+args) + `FERRO_AI_LIVE_EVAL` replay/smoke path — Phase 221 (AMCP-06).
- DB-backed confirmation store (production hardening) — deferred per REQUIREMENTS.
- gestiscilo full adoption of the write endpoint — consumer-repo follow-up; 219 delivers framework capability + synthetic validation only.
- Advertising `idempotency_key` in the write-tool `inputSchema` if research finds it unnecessary for the agent to send it reliably.

</deferred>

---

*Phase: 219-write-dispatch*
*Context gathered: 2026-06-13*
