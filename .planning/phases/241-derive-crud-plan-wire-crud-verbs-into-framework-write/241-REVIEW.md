---
phase: 241-derive-crud-plan-wire-crud-verbs-into-framework-write
reviewed: 2026-06-24T00:00:00Z
depth: standard
files_reviewed: 11
files_reviewed_list:
  - ferro-projections/src/executor.rs
  - ferro-projections/src/error.rs
  - ferro-projections/src/lib.rs
  - framework/src/write/mod.rs
  - ferro-mcp-server/src/write_dispatch.rs
  - ferro-mcp-server/src/renderer.rs
  - ferro-mcp-server/src/error.rs
  - ferro-mcp-server/src/jsonrpc.rs
  - app/src/controllers/visual_action.rs
  - app/src/tests/single_source.rs
  - app/src/tests/visual_action.rs
findings:
  critical: 0
  warning: 4
  info: 4
  total: 8
status: issues_found
---

# Phase 241: Code Review Report

**Reviewed:** 2026-06-24
**Depth:** standard
**Files Reviewed:** 11
**Status:** issues_found

## Summary

Phase 241 adds `derive_crud_plan`/`CrudPlan`/`CrudVerb` to ferro-projections and wires generic CREATE/UPDATE/soft-DELETE execution through the single `framework::write::dispatch_write` kernel, extending ferro-mcp-server's write framing with real CRUD tool dispatch instead of the prior NTI stub.

SQL injection posture is sound: every value is bound through `Statement::from_sql_and_values`; `now_expr` is a hardcoded constant, not user input; all table/column identifiers originate exclusively from the developer-authored `CrudPlan`. No string-interpolated user values were found.

Soft-delete correctness (CRUD-03) is implemented correctly. The `DELETE` variant emits `UPDATE … SET deleted_at = now WHERE id = ? AND deleted_at IS NULL`; `UPDATE` carries the same `AND deleted_at IS NULL` predicate. Physical `DELETE FROM` is never issued. The tests in `framework/src/write/mod.rs` verify both properties end-to-end.

Single-dispatcher invariant (SC#4) holds. There is no second CRUD dispatcher and no re-encoded transition `match` on the CRUD path — `dispatch_write` is the single execution pipeline for both verbs.

Four warnings were found, none security-critical. They are correctness or robustness issues that could produce silent data loss or confusing behavior in production. Four info-level items round out the review.

Tenant-scope enforcement, `read_write` scope / `.mcp_write_ability` authorization, and cross-tenant non-disclosure are deferred to Phase 242 by design and are not flagged here.

---

## Warnings

### WR-01: `CrudPlan::Update` returns the record **after** soft-delete predicate is gone

**File:** `framework/src/write/mod.rs:406-420`
**Issue:** After a successful `UPDATE`, the post-update `SELECT` fetches the record using only `WHERE {id_column} = {id_ph2}` — without `AND {soft_delete_column} IS NULL`. Under normal usage this is harmless because the `UPDATE` itself succeeded (meaning `deleted_at IS NULL` was true at that moment), but a concurrent soft-delete between the `UPDATE` and the `SELECT` would return the deleted record to the caller, surfacing `deleted_at` as non-null in the JSON result sent back (including to the audit log). The delete case correctly returns only `{"id": ..., "deleted": true}` without a follow-up `SELECT`. The update case should match that robustness.

**Fix:** Add `AND {soft_delete_column} IS NULL` to the post-update `SELECT`:
```rust
let select_sql = format!(
    "SELECT * FROM {table} WHERE {id_column} = {id_ph2} AND {soft_delete_column} IS NULL"
);
```
Alternatively, since `rows_affected() == 1` already implies the predicate passed, the risk is low and the fix is cosmetic robustness — but consistency with the guard invariant stated in the doc-comment is worth preserving.

---

### WR-02: `CrudPlan::Update` with `id_value = Null` silently updates every non-deleted row

**File:** `framework/src/write/mod.rs:386-393` and `ferro-projections/src/executor.rs:254-255`
**Issue:** `derive_crud_plan` for `CrudVerb::Update` extracts `id` as:
```rust
let id_value = inputs.get("id").cloned().unwrap_or(serde_json::Value::Null);
```
If the agent omits `id` (or passes `null`), `id_value` is `Null`, which `json_to_sea_value` converts to `sea_orm::Value::String(None)`. In SQLite, `WHERE id = NULL` always evaluates to NULL (unknown), so zero rows are affected and `RecordNotFound` is returned — safe by accident. In Postgres, the same `WHERE id = $1` with a `NULL` binding also evaluates to unknown — also zero rows affected. So the failure mode returns `RecordNotFound` rather than a mass update, which is the desired behavior.

However, the correctness depends on SQL NULL semantics holding across all supported backends and not on an explicit guard. A missing `id` on an update is a clear input validation error that should be rejected explicitly at derivation time, not silently deferred to the SQL predicate. The matching `delete_plan` has the same pattern.

**Fix:** Reject `Null` ids at derivation time so the error surface is clear:
```rust
// In derive_crud_plan, CrudVerb::Update arm:
let id_value = inputs.get("id")
    .filter(|v| !v.is_null())
    .cloned()
    .ok_or_else(|| crate::Error::Validation("update requires an 'id' field".into()))?;
```
Apply the same guard to `CrudVerb::Delete`.

---

### WR-03: `CrudPlan::Create` columns are sourced from `inputs` without any size validation

**File:** `ferro-projections/src/executor.rs:224-229`
**Issue:** The `Create` arm collects every field that is both writable and present in `inputs`:
```rust
.filter_map(|f| inputs.get(&f.name).map(|v| (f.name.clone(), v.clone())))
```
A string field value of arbitrary length is accepted. There is no length cap. This means an agent can INSERT a string value of unbounded size into any writable text column. For the MCP surface the existing `idempotency_key` cap (128 chars) is a precedent; writable field values have no analogous guard. This is not exploitable for SQL injection (values are bound), but it is a denial-of-service / DB bloat surface.

This is a low-urgency warning because the fix belongs at the schema/ServiceDef level (`FieldDef.max_length`) rather than in the executor, and Phase 242 may add such constraints. Flag now so it is not forgotten.

**Fix:** Add an optional `max_length: Option<usize>` to `FieldDef` and enforce it in `derive_crud_plan` before building the column list. As an interim guard, reject string values longer than a configurable threshold (e.g. 64 KB) in `json_to_sea_value` or `derive_crud_plan`.

---

### WR-04: Confirmation guard re-evaluation is skipped on the CRUD delete path in `handle_request_confirm`

**File:** `ferro-mcp-server/src/write_dispatch.rs:411-449`
**Issue:** For non-CRUD destructive actions, `handle_request_confirm` re-evaluates guards before issuing the token (lines 474-491). For the CRUD delete path (when `action_name.strip_prefix("delete_")` matches), it skips that guard re-evaluation entirely: it immediately generates a token and stores it without calling `dispatcher.guard_evaluator` for any precondition.

CRUD delete tools synthesized in Phase 241 carry no `ActionDef.preconditions` (the synthesized `crud_action` has an empty precondition list). This means there are no guards to re-evaluate right now, so the skip is currently harmless. However, Phase 242 will add `mcp_write_ability` authorization and potentially per-record guards. If those are wired as preconditions on the synthesized `crud_action` (a natural extension point), the CRUD delete path would silently bypass the token-issuance pre-check, issuing a token that later fails at `dispatch_write` guard re-eval instead.

The asymmetry between the transition-action path (guards checked at token issuance) and the CRUD delete path (guards not checked) should be resolved before Phase 242 adds write-ability guards.

**Fix:** After the `svc` lookup succeeds in the CRUD delete arm, add an explicit guard pre-check loop analogous to lines 474-491, using a guard set derived from any preconditions the CRUD verb carries. Until Phase 242, this is a no-op loop (empty preconditions), but it wires the extension point correctly:
```rust
// After svc lookup:
let guards: Vec<String> = vec![]; // Phase 242 adds write_ability guard here
for guard_name in &guards {
    // ... same pattern as the transition path
}
```

---

## Info

### IN-01: `CrudPlan` derives `PartialEq` but not `Eq`

**File:** `ferro-projections/src/executor.rs:156`
**Issue:** `CrudPlan` derives `PartialEq` but not `Eq`. The contained `serde_json::Value` does not implement `Eq` (because JSON numbers have NaN considerations), which makes `Eq` non-derivable. This is the correct and intentional design. However, it is worth documenting explicitly in the struct's doc-comment or with a `#[allow(clippy::derive_partial_eq_without_eq)]` annotation so future readers do not file a bug or accidentally add `Eq` via a blanket impl.

**Fix:** Add a brief comment on the `PartialEq` derive:
```rust
// PartialEq only (not Eq): serde_json::Value does not implement Eq.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum CrudPlan { ... }
```

---

### IN-02: `row_to_json` in `execute_crud_plan` returns `Sensitive` fields verbatim

**File:** `framework/src/write/mod.rs:216-242`
**Issue:** `row_to_json` reads all columns returned by `SELECT *` and maps them into the result `Value`. There is no post-processing to strip fields whose `FieldMeaning` is `Sensitive`. The audit log (`AuditEntry::after`) receives this value verbatim (line 723). The executor doc-comment acknowledges that "the executor is the enforcement point" for audit-safe fields, but the generic executor has no access to the `ServiceDef` field list and therefore cannot filter `Sensitive` fields.

Per the review scope note, tenant-scoping is Phase 242. The Sensitive-field concern is orthogonal to tenancy and is in scope now.

**Fix:** Thread the `ServiceDef` (or a filtered column allow-list) into `execute_crud_plan` and apply a post-filter on `row_to_json`'s output before returning or auditing:
```rust
// After row_to_json(&row), before returning to dispatch_write:
let result = redact_sensitive_fields(result, svc_fields);
```
Alternatively, document explicitly that `ServiceDef.fields` with `FieldMeaning::Sensitive` must not be included in CRUD-accessible tables until Phase 242 adds this filter.

---

### IN-03: `_tenant_id` parameter in `execute_crud_plan` is intentionally unused but silently discarded

**File:** `framework/src/write/mod.rs:272-276`
**Issue:** The `_tenant_id: i64` parameter is prefixed with `_` because the tenant predicate is deferred to Phase 242. This is correct. However, the leading underscore suppresses the compiler warning only — it does not make the intention visible to a reader wondering whether the missing tenant filter is a bug. The existing doc-comment mentions "D-09: `tenant_column` is `None` in Phase 241" which partially covers this, but the parameter rename is not self-explanatory without reading that comment.

**Fix:** Either rename to `_tenant_id_phase_242` (very explicit) or add a `// Phase 242: tenant_column predicate added here` comment adjacent to the `match plan {` branch where the tenant injection would go. The current `tenant_column: _` destructuring in each arm already does this implicitly; a brief standalone comment at the function level would remove all ambiguity.

---

### IN-04: `handle_write_call` CRUD error match arm is missing `GuardFailed`

**File:** `ferro-mcp-server/src/write_dispatch.rs:240-258`
**Issue:** The `dispatch_write` call on the CRUD path in `handle_write_call` (lines 209-259) handles `ConfirmationRequired`, `RecordNotFound`, `Validation`, and `ActionNotFound` explicitly, then falls through to the catch-all `Err(_)` for all other errors. `GuardFailed` lands in the catch-all and returns `error_kind: "execution_error"` — a generic message. The transition-action path (lines 325-343) has an explicit `GuardFailed` arm that also writes a denial audit entry.

For CRUD create/update this is purely cosmetic (those paths have no action guards right now). For a future create/update path that carries preconditions, a guard failure would surface as `execution_error` without an audit entry, breaking the audit trail invariant.

**Fix:** Add an explicit `GuardFailed` arm on the CRUD path, mirroring the transition-action path. Since CRUD verbs synthesize a `crud_action` rather than using a real `ActionDef.name`, the denial audit entry's target should use the tool name. No denial audit entry is strictly required by Phase 241, but adding the arm now future-proofs Phase 242.

---

_Reviewed: 2026-06-24_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
