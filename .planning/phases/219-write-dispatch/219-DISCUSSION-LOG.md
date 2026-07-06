# Phase 219: Write Dispatch - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-13
**Phase:** 219-write-dispatch
**Mode:** `--auto` (gray areas auto-selected; recommended defaults chosen and logged)
**Areas discussed:** Execution mechanism, Server-side guard re-eval, Tenant scoping, Idempotency, Audit, Result construction, Routing, Confirmation seam, Sample-app wiring

---

## Action execution mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| App-registered callback `async fn(action_name, inputs, tenant_id, db) -> Result<Value>` | Projection-agnostic; ARCHITECTURE Phase 3 | ✓ |
| Generic SQL write in ferro-mcp-server | Can't express validation/business logic/transitions | |
| Route through the app's HTTP stack | Re-implements auth; REQUIREMENTS says callback suffices | |

**Auto-selected:** app callback. Registration shape (trait vs boxed fn, held where) is research/discretion.

## Server-side guard re-evaluation (THE security mechanism)

| Option | Description | Selected |
|--------|-------------|----------|
| App-registered GuardEvaluator invoked by dispatch_write before the callback, live DB, fail-closed, independent of the 218 list filter | SC#1; PITFALLS §2 structural fix | ✓ |
| Trust the 218 evaluated_guards visibility map at call time | The exact bypass PITFALLS §2 warns against | |

**Auto-selected:** server-side re-eval at execution. Receives validated inputs (incl. record id) for record-scoped guards.

## Tenant scoping for writes

| Option | Description | Selected |
|--------|-------------|----------|
| `find_for_tenant(id, tenant_id)` None→deny before mutation; cross-tenant fixture | SC#2; existing contract | ✓ |
| Add new write methods to TenantScoped | Not load-bearing; find-then-mutate suffices | |

**Auto-selected:** find_for_tenant denial pattern. tenant_id from principal, never payload.

## Idempotency

| Option | Description | Selected |
|--------|-------------|----------|
| Optional `idempotency_key` from arguments; framework `mcp_idempotency_keys` table (consumer migration); store (tenant,key)→result, dup replays | SC#3; mirrors 217 mcp_api_keys | ✓ |
| Reuse ferro-queue idempotency | Different surface (jobs, not tool calls) | |

**Auto-selected:** dedicated mcp_idempotency_keys table. Research: advertise idempotency_key in inputSchema?

## Audit

| Option | Description | Selected |
|--------|-------------|----------|
| Evaluate ferro-audit first; reuse if a per-action event fits; else lightweight mcp_audit_log table | SC#4; no-duplicate-control-surface convention | ✓ |

**Auto-selected:** research ferro-audit fit, fall back to minimal table. Required fields: tool, tenant, action, param ids, recoverable.

## Result construction

| Option | Description | Selected |
|--------|-------------|----------|
| `CallToolResult::structured` for every write response; guard-denied → structured error | SC#5; Phase 205 constructor | ✓ |
| Hand-built content[] arrays | The Phase 205 bug class | |

**Auto-selected:** structured for all outcomes. Error envelope shape research-resolved (consistent with 205/217).

## Routing

| Option | Description | Selected |
|--------|-------------|----------|
| handle_tools_call → handle_write_call → dispatch_write, behind the 217 scope gate, replacing the 218 -32601 | D-07 ordered pipeline | ✓ |

**Auto-selected.** Order: scope → resolve ActionDef → validate inputs → guard re-eval → idempotency → execute → audit → structured result.

## Confirmation seam

| Option | Description | Selected |
|--------|-------------|----------|
| No confirmation, no ferro-ai dep in 219; leave a clean seam for 220 | ARCHITECTURE puts confirmation in Phase 4 (=220) | ✓ |

**Auto-selected:** seam only.

## Sample-app wiring

| Option | Description | Selected |
|--------|-------------|----------|
| Framework provides dispatch_write + registration hooks; sample app registers a concrete tenant-scoped executor + guard evaluator for ≥1 action so SC#1–#5 are testable end-to-end | Synthetic validation per milestone | ✓ |

**Auto-selected.** Flag for planner: phase may warrant a split (framework machinery vs app wiring + fixtures).

## Claude's Discretion

- Registration API (trait object vs boxed async fn; McpServerConfig vs new dispatcher param).
- Serialized idempotency `result` shape; audit entry record shape; whether idempotency_key is advertised in inputSchema; error-result envelope shape.

## Deferred Ideas

- Confirmation gating + confirm_<action> + ferro-ai (220); inbound NL loop (221); DB-backed confirmation store; gestiscilo adoption.
