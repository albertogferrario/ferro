---
phase: 197-mcprenderer-ferro-mcp-server
reviewed: 2026-06-10T00:00:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - ferro-mcp-server/src/dispatch.rs
  - ferro-mcp-server/src/schema.rs
  - ferro-mcp-server/src/renderer.rs
  - ferro-mcp-server/src/lib.rs
  - ferro-mcp-server/src/error.rs
  - ferro-mcp-server/tests/dispatch_integration.rs
  - ferro-projections/src/service.rs
findings:
  critical: 0
  warning: 4
  info: 2
  total: 6
status: issues_found
---

# Phase 197: Code Review Report

**Reviewed:** 2026-06-10
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

The new `ferro-mcp-server` crate is structurally sound. The SQL injection concern at the heart of the milestone — filter key allowlisting — is correctly implemented: unknown keys are rejected before any SQL assembly, and filter values plus pagination are parameter-bound through `Statement::from_sql_and_values`. The table name is derived from developer-controlled Rust source (`ServiceDef::new(...)`) and double-quoted in the query string, so it is not an injection vector.

Two security-adjacent gaps exist. First, the dispatch allowlist validates filter key existence in `service.fields` but does not enforce `is_filter_field()` eligibility, so an agent can construct filters on write-only or Sensitive fields that were deliberately excluded from the advertised `inputSchema`. Second, the `dispatch()` function applies no upper bound to the `limit` parameter; the `maximum: 100` constraint lives only in the JSON Schema and is not enforced at the query layer. Both are fixable with minimal changes.

Two correctness issues: rows decoded with an unsigned or decimal DB type silently become `Null` due to the `rows_to_json` type waterfall only covering `String`, `i64`, `f64`, and `bool`; and paginated queries issue no `ORDER BY`, making result sets non-deterministic across calls. Both are pre-existing or known gaps, but they affect observable behavior of the new read path.

## Warnings

### WR-01: Filter key allowlist does not enforce `is_filter_field` eligibility

**File:** `ferro-mcp-server/src/dispatch.rs:107`

**Issue:** The allowlist check (`service.fields.iter().any(|f| &f.name == key)`) confirms the key names a known field but does not verify that the field is actually filter-eligible. This means an agent can pass `{"password_hash": "abc"}` (a write-only / Sensitive field) as a filter, causing the WHERE clause to narrow the result set by that column's value — and `SELECT *` will return the column in the output. This creates an oracle: an agent can infer whether any row matches a given password hash by checking whether rows are returned, even though `inputSchema` deliberately excludes that field. The advertised security boundary (gate 1–5 in `schema.rs`) is not enforced at the query layer.

**Fix:**
```rust
// dispatch.rs, line 107 — replace the existence check with eligibility check
if !service.fields.iter().any(|f| &f.name == key && crate::schema::is_filter_field(f)) {
    return Err(crate::Error::Database(format!(
        "unknown filter field: {key}"
    )));
}
```

This reuses the exact same predicate that governs the `inputSchema`, making the two surfaces consistent. Export `is_filter_field` from `schema.rs` as `pub(crate)` if it isn't already (currently `pub`).

---

### WR-02: `dispatch()` does not clamp `limit` — `maximum: 100` is schema-only

**File:** `ferro-mcp-server/src/dispatch.rs:89-157`

**Issue:** The `inputSchema` advertises `"maximum": 100` for the `limit` parameter, but `dispatch()` accepts any `u64` and forwards it directly to the SQL query. Code paths that call `dispatch()` directly — or future wiring that extracts the limit from call arguments without re-validating against the schema — can issue queries with arbitrarily large limits. At `limit = u64::MAX`, the cast `limit as i64` on line 141 also silently wraps to a negative value, which some databases interpret as 0 or raise an error.

**Fix:**
```rust
// At the top of dispatch(), after receiving limit and offset:
const MAX_LIMIT: u64 = 100;
let limit = limit.min(MAX_LIMIT);
```

This makes the enforcement unconditional at the call site rather than trusting the JSON Schema validation layer.

---

### WR-03: `rows_to_json` type waterfall silently maps unmapped types to `Null`

**File:** `ferro-mcp-server/src/dispatch.rs:40-78`

**Issue:** The waterfall tries `String → i64 → f64 → bool` then falls back to `Value::Null`. DB columns typed as `u32`, `u64`, unsigned integers, `Decimal`, `BigDecimal`, `NaiveDate`, `NaiveDateTime`, or any SeaORM custom type fail all four branches and silently become `Null` in the JSON output with no log or error. For a read-model tool this means fields appear present in the schema but return `null` at runtime without any indication that the type mapping failed. A developer will see empty/null fields and have no way to distinguish "the column has a NULL value" from "the type is not handled."

**Fix:**
```rust
// Add unsigned integer and date handling to the waterfall:
.or_else(|_| {
    row.try_get_by::<u64, _>(col.as_str())
        .map(|v| serde_json::Value::Number(v.into()))
})
.or_else(|_| {
    row.try_get_by::<chrono::NaiveDateTime, _>(col.as_str())
        .map(|v| serde_json::Value::String(v.to_string()))
})
// ...
```

At minimum, log a warning when the final fallback fires so the type gap is visible during development.

---

### WR-04: Paginated queries omit `ORDER BY` — non-deterministic page boundaries

**File:** `ferro-mcp-server/src/dispatch.rs:144`

**Issue:** The data query is `SELECT * FROM "{table}" WHERE ... LIMIT $n OFFSET $m` with no `ORDER BY` clause. Without a stable sort, the database is free to return rows in any order, which means page 2 may overlap with page 1 or skip rows if the underlying storage order changes between queries (e.g., after a concurrent write, VACUUM, or engine-internal reorg). This makes `offset`-based pagination unreliable by design.

**Fix:** Add a deterministic default sort. The conventional choice for a projection read path is to sort by the first `Identifier`-meaning field ascending if one exists:

```rust
// After building where_str, before the data query:
let order_by = service
    .fields
    .iter()
    .find(|f| matches!(f.meaning, FieldMeaning::Identifier))
    .map(|f| format!(" ORDER BY \"{}\" ASC", f.name))
    .unwrap_or_default();

let data_sql = format!("SELECT * FROM \"{table}\"{where_str}{order_by}{limit_str}");
```

If the field name comes from `ServiceDef` (developer-controlled), it is safe to interpolate with double-quoting. No additional allowlist check is needed for this specific field since it is selected from `service.fields` internally, not from the call payload.

---

## Info

### IN-01: `mcp_exposed: false` serializes unconditionally — inconsistent with sibling fields

**File:** `ferro-projections/src/service.rs:83-84`

**Issue:** All other zero-value / empty fields on `ServiceDef` use `#[serde(default, skip_serializing_if = "Vec::is_empty")]` or `#[serde(skip_serializing_if = "Option::is_none")]` to suppress noise in JSON output. `mcp_exposed` has `#[serde(default)]` but no `skip_serializing_if`, so every serialized `ServiceDef` — including all existing ones where the intent is "not exposed" — gains `"mcp_exposed": false` in its JSON representation. This is cosmetically inconsistent and bloats stored/transmitted definitions.

**Fix:**
```rust
// Add a helper function (Rust doesn't provide a bool equivalent of Vec::is_empty):
fn is_false(b: &bool) -> bool { !b }

// Then on the field:
#[serde(default, skip_serializing_if = "is_false")]
pub mcp_exposed: bool,
```

Deserialization of existing documents without the key continues to work correctly via `#[serde(default)]`.

---

### IN-02: `derive_intents` called unnecessarily in `render_exposed_tools`

**File:** `ferro-mcp-server/src/renderer.rs:64`

**Issue:** `render_exposed_tools` calls `ferro_projections::derive_intents(s)` on each service and passes the result to `renderer.render(...)`, but `McpRenderer::render` declares the parameter as `_intents` and ignores it entirely. The intent derivation is non-trivial computation (field analysis + scoring) that produces output that is immediately discarded.

**Fix:** The `Renderer` trait requires the `intents` argument, so the call cannot be dropped from the signature. Pass an empty slice to avoid the computation cost while satisfying the trait:

```rust
services
    .iter()
    .filter(|s| s.mcp_exposed)
    .map(|s| renderer.render(s, &[], ctx))
    .collect()
```

If intents become meaningful to MCP rendering in a future phase, restore the `derive_intents` call at that point.

---

_Reviewed: 2026-06-10_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
