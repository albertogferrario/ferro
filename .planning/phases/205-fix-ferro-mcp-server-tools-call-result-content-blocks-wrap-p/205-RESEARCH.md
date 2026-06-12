# Phase 205: Fix ferro-mcp-server tools/call Result Content Blocks - Research

**Researched:** 2026-06-12
**Domain:** Rust / rmcp 0.12 MCP protocol types, JSON-RPC envelope assembly
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Build the result with `rmcp::model::CallToolResult::structured(value)` rather than hand-assembling JSON.
- **D-02:** Nest pagination metadata inside a single structured value `{ "rows": [...], "total": N, "limit": N, "offset": N }` passed to `CallToolResult::structured`.
- **D-03:** Emit a single text content block (the default of `structured()`). Do not emit one block per row.
- **D-04:** The interop regression test must parse the emitted `result` with the MCP client's own types. Deserialize `result.content` into `Vec<rmcp::model::Content>` and assert every block parses, plus assert `structuredContent` is present and round-trips.
- **D-05:** Leave the existing JSON-RPC error envelope unchanged (`-32601/-32602/-32603`).
- **D-06:** After the fix, re-run the live `:8090` browser-OAuth dogfood end-to-end — alice@acme.test → list_order — confirming (a) no Zod errors and (b) tenant scoping returns only Acme's 2 of 4 orders.

### Claude's Discretion
- Exact naming of any helper introduced to assemble the structured value.
- Whether the `result` JSON-RPC wrapping is produced inline or via a small serialize step.
- Compact vs pretty JSON inside the text block.

### Deferred Ideas (OUT OF SCOPE)
- Tool-level error results (converting invalid-filter / internal failures from JSON-RPC errors into `CallToolResult { isError: true }`).
- `_meta` for pagination (placing `total`/`limit`/`offset` in `CallToolResult._meta`).
</user_constraints>

---

## Summary

Phase 205 is a single-site bug fix in `ferro-mcp-server/src/jsonrpc.rs`. The `Ok(result)` arm of `handle_tools_call` (lines 84-91) serializes `DispatchResult.rows` — a `Vec<serde_json::Value>` of bare database row objects — directly into `content[]`. Each row object has no `"type"` field, so strict MCP clients (Claude Code's SDK) Zod-reject every content item on parse. The result object also exposes `total`/`limit`/`offset` as non-standard top-level keys alongside `content` inside the JSON-RPC `result`.

The canonical fix is one expression replacement: construct a `serde_json::Value` from `{ rows, total, limit, offset }` and pass it to `rmcp::model::CallToolResult::structured(value)`, then serialize the resulting struct as the JSON-RPC `result` value. `rmcp 0.12` is already a declared dependency. The `structured()` constructor produces the valid MCP shape by construction: `content: [{"type":"text","text":"..."}]` + `structuredContent: <value>` + `isError: false`.

The regression test must deserialize the emitted `result` value using `rmcp::model::CallToolResult` (which has a custom `Deserialize` implementation, line 1646 of rmcp model.rs) and `rmcp::model::Content` (= `Annotated<RawContent>`, derives `Serialize + Deserialize`). Both types are fully deserializable and available without adding any new dependency.

A secondary task updates the existing `app/src/tests/mcp_tenant_isolation.rs` tests, which assert the **old broken shape** (`result["result"]["content"]` as bare row objects). After the fix those assertions will fail. They must be updated to navigate the new envelope (`result["result"]["content"][0]["text"]` for the text block, and `result["result"]["structuredContent"]["rows"]` for the data).

**Primary recommendation:** Replace lines 84-91 of `ferro-mcp-server/src/jsonrpc.rs` with a `CallToolResult::structured(...)` call; add an inline `#[cfg(test)]` block in `jsonrpc.rs` that deserializes the emitted value with `CallToolResult`; update the two tenant isolation tests in `mcp_tenant_isolation.rs` to navigate the new envelope.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| MCP result envelope construction | `ferro-mcp-server` (library crate) | — | `handle_tools_call` owns result formatting; the app controller just splices `jsonrpc`/`id` |
| JSON-RPC envelope (`jsonrpc`/`id` splice) | `app/src/controllers/mcp.rs` | — | The controller wraps the inner value; fix must not change the outer envelope |
| MCP client schema compliance | `rmcp::model::CallToolResult` | — | The rmcp type is authoritative for the wire format |
| Regression test | `ferro-mcp-server/src/jsonrpc.rs` | — | Inline `#[cfg(test)]` following existing dispatch.rs convention |
| Tenant isolation test update | `app/src/tests/mcp_tenant_isolation.rs` | — | Existing integration test asserting old shape — must be updated |

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rmcp | 0.12 | MCP protocol types (`CallToolResult`, `Content`) | Already declared dep; `structured()` produces spec-valid output by construction |
| serde_json | 1.0 | Assemble the structured payload `json!({...})` | Already a dep |

**No new dependencies required.** [VERIFIED: ferro-mcp-server/Cargo.toml line 15]

---

## Architecture Patterns

### System Architecture Diagram

```
tools/call request
        |
        v
app/src/controllers/mcp.rs::handle()
  - origin check, bearer validation, policy gate
  - extracts tenant_id from current_tenant()
        |
        v
ferro_mcp_server::handle_tools_call(params, services, db, tenant_id)
  - strip "list_" prefix, find ServiceDef
  - parse limit/offset/filters from arguments
        |
        v
ferro_mcp_server::dispatch(...)
  - filter allowlist check
  - SQL SELECT with tenant predicate
  - returns DispatchResult { rows, total, limit, offset }
        |
        v  [BUG IS HERE - lines 84-91]
  json!({ "result": { "content": result.rows, ... } })
        |  [FIX: replace with CallToolResult::structured(...)]
  serde_json::to_value(CallToolResult::structured(payload))
        |  where payload = json!({ "rows": rows, "total": ..., "limit": ..., "offset": ... })
        v
  json!({ "result": <CallToolResult> })
        |
        v
app/src/controllers/mcp.rs (splice jsonrpc + id onto object)
        |
        v
HTTP response body (JSON-RPC 2.0 envelope)
        |
        v
MCP client parses result as CallToolResult
  - content[0]: {"type":"text","text":"<JSON string>"}  <-- was missing "type", now present
  - structuredContent: { rows: [...], total: N, ... }   <-- was absent, now present
  - isError: false                                       <-- was absent, now present
```

### Recommended Project Structure

No structural changes. The fix is localized to `ferro-mcp-server/src/jsonrpc.rs` with a test addition in the same file, plus an update to `app/src/tests/mcp_tenant_isolation.rs`.

### Pattern 1: Result Envelope Construction (the Fix)

**What:** Replace the hand-assembled JSON at lines 84-91 with `CallToolResult::structured`.

**When to use:** Any time a projection dispatch result must be wrapped as a valid MCP tool result.

**Example:**
```rust
// Source: rmcp-0.12.0/src/model.rs:1581
// BEFORE (broken — content items have no "type" field):
Ok(result) => json!({
    "result": {
        "content": result.rows,
        "total": result.total,
        "limit": result.limit,
        "offset": result.offset
    }
})

// AFTER (D-01 + D-02 — single structured value, valid content block):
Ok(result) => {
    let payload = serde_json::json!({
        "rows": result.rows,
        "total": result.total,
        "limit": result.limit,
        "offset": result.offset
    });
    let tool_result = rmcp::model::CallToolResult::structured(payload);
    json!({ "result": tool_result })
}
```

**Import required:** `use rmcp::model::CallToolResult;` — add to `jsonrpc.rs` imports.

### Pattern 2: Interop Regression Test (D-04)

**What:** Deserialize the emitted `result` object with `rmcp::model::CallToolResult` (custom Deserialize, verified) to prove the client parses it.

**Convention:** Inline `#[cfg(test)] mod tests` at the bottom of `jsonrpc.rs` — dispatch.rs and renderer.rs both follow this pattern.

**Test shape:**
```rust
// Source: rmcp-0.12.0/src/model.rs:1646 (custom Deserialize for CallToolResult)
//         rmcp-0.12.0/src/model/content.rs:73 (Content = Annotated<RawContent>, derives Deserialize)
#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::{DataType, FieldMeaning, ServiceDef};
    use rmcp::model::{CallToolResult, Content};
    use sea_orm::{Database, DatabaseBackend, Statement};

    // Reuse dispatch.rs test pattern: setup_orders_db + an order ServiceDef fixture.

    #[tokio::test]
    async fn tools_call_result_parses_as_valid_mcp_content() {
        // 1. Set up in-memory SQLite with orders (same pattern as dispatch.rs tests).
        // 2. Call handle_tools_call(...).
        // 3. Extract result["result"] as a serde_json::Value.
        // 4. Deserialize into CallToolResult — must not panic/error (D-04).
        // 5. Assert content[0] is a Text block with non-empty text.
        // 6. Assert structured_content is Some and round-trips back to the original rows count.
        // 7. Assert is_error == Some(false).

        let result_value: serde_json::Value = ...; // from handle_tools_call

        let parsed: CallToolResult =
            serde_json::from_value(result_value["result"].clone())
                .expect("result must parse as CallToolResult (D-04 interop)");

        assert_eq!(parsed.is_error, Some(false));
        assert_eq!(parsed.content.len(), 1);

        // Every content block must be parseable as Content (Zod-equivalent assertion)
        let _blocks: Vec<Content> = serde_json::from_value(
            serde_json::to_value(&parsed.content).unwrap()
        ).expect("all content blocks must deserialize as rmcp::model::Content");

        // structured_content carries { rows, total, limit, offset }
        let sc = parsed.structured_content.expect("structuredContent must be present");
        assert!(sc.get("rows").is_some(), "structuredContent.rows must be present");
        assert!(sc.get("total").is_some(), "structuredContent.total must be present");
    }
}
```

### Pattern 3: Existing Tenant Isolation Test Update (mandatory, distinct from D-04)

**What:** `app/src/tests/mcp_tenant_isolation.rs` `tenant_a_isolation` and `tenant_b_isolation` currently assert:
```rust
// OLD (asserting broken shape):
let rows = result["result"]["content"].as_array()...
// then: row["tenant_id"].as_i64() on each bare object in content[]
```

After the fix, `content[]` contains one text block (`{"type":"text","text":"..."}`), not bare row objects. The tenant_id data now lives in `structuredContent.rows`. Both tests must be updated to:
1. Assert `content[0]["type"] == "text"` (valid block present)
2. Read `result["result"]["structuredContent"]["rows"]` as the array
3. Assert `row["tenant_id"]` on each row from that array

**This is a required fix to the integration test — it is not optional.** The isolation *behavior* is unchanged (dispatch.rs still enforces tenant predicate); only the assertion path changes.

### Anti-Patterns to Avoid

- **Changing the outer JSON-RPC envelope.** The `{"jsonrpc":"2.0","id":...}` splicing in `app/src/controllers/mcp.rs` (line 161-163) is correct and must not change. The fix is only to the `"result"` value returned by `handle_tools_call`.
- **Serializing `CallToolResult` via `serde_json::to_value` with a manual intermediate.** Just pass the `Value` to `json!({ "result": tool_result })` directly — serde will serialize the `CallToolResult` struct inline when the outer `json!()` is evaluated. No intermediate `to_value()` call is needed.
- **Changing `dispatch.rs` or `DispatchResult`.** `DispatchResult` is correct and unchanged; the fix only changes how `handle_tools_call` wraps the result.
- **Emitting one text block per row (D-03).** `CallToolResult::structured()` already emits a single text block; do not override it.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Valid MCP content-block envelope | `json!({"content":[{"type":"text",...}],...})` | `CallToolResult::structured(value)` | `structured()` produces the correct shape by construction, including camelCase keys, is_error, structuredContent |
| Content block `"type"` tag | Manual `"type": "text"` injection | `RawContent::Text(...)` / `Content::text(...)` | `RawContent` has `#[serde(tag = "type", rename_all = "snake_case")]` — text variant always serializes with correct tag |

---

## Common Pitfalls

### Pitfall 1: Forgetting to Update the Tenant Isolation Integration Tests

**What goes wrong:** `cargo test` in `app/` will fail on `tenant_a_isolation` and `tenant_b_isolation` because they assert `result["result"]["content"]` as a `Vec<bare-row-objects>`. After the fix, `content[0]` is a text block, not a row object; `row["tenant_id"].as_i64()` returns `None` on the text block.

**Why it happens:** The tests were written to assert the server's own (broken) output shape — they inadvertently validated the bug.

**How to avoid:** Update both tenant isolation tests as part of the same plan as the jsonrpc.rs fix. Do not commit the fix without updating the tests; `cargo test --all-features` must be green before commit.

**Warning signs:** `cargo test` in `app/` panics with "each row must have a tenant_id field" on the text-type content block.

### Pitfall 2: `CallToolResult` derives `Serialize` but NOT `#[derive(Deserialize)]` — it has a custom `Deserialize`

**What goes wrong:** A developer might grep for `#[derive(... Deserialize ...)]` on `CallToolResult` and conclude it cannot be deserialized.

**Why it happens:** rmcp 0.12 model.rs line 1531 has `#[derive(Debug, Serialize, Clone, PartialEq)]` — no `Deserialize` in the derive list. But line 1646 provides `impl<'de> Deserialize<'de> for CallToolResult` manually (mutual exclusivity validation).

**How to avoid:** The D-04 test `serde_json::from_value::<CallToolResult>(...)` will compile and work correctly. The custom impl is present and tested by rmcp itself.

**Warning signs:** If the test doesn't compile, check the import path: `use rmcp::model::CallToolResult;` — rmcp re-exports `model` at the crate root.

### Pitfall 3: `json!({ "result": tool_result })` requires serde `Serialize` — not `to_value`

**What goes wrong:** Using `serde_json::to_value(tool_result)?` and then inserting the result manually, adding error handling that isn't needed.

**Why it happens:** `CallToolResult` derives `Serialize`. `serde_json::json!()` calls `serde_json::to_value()` internally for each interpolated expression. Direct interpolation `json!({ "result": tool_result })` is idiomatic and infallible for types that derive `Serialize`.

**How to avoid:** Write `json!({ "result": tool_result })` directly. No `?` operator, no intermediate variable needed.

### Pitfall 4: The `content` field change also invalidates the `make_tool_deny_response` contract in `app/src/controllers/mcp.rs`

**What goes wrong:** `make_tool_deny_response` (line 33-44 of `mcp.rs`) already hand-assembles a valid content block `[{"type":"text","text":...}]`. This is **correct and must not change** — it handles the policy-deny path (not covered by the fix). Tests in `mcp.rs` (`policy_deny_tool_error_shape`, `deny_response_is_jsonrpc_success_not_transport_error`) assert this shape and remain valid.

**How to avoid:** The fix is scoped to `ferro-mcp-server/src/jsonrpc.rs::handle_tools_call`'s `Ok(result)` arm only. The `make_tool_deny_response` function in the app controller is a separate code path and is already correct.

### Pitfall 5: Non-standard top-level keys after the fix

**What goes wrong:** If the fix assembles `json!({ "result": tool_result, "total": ..., "limit": ..., "offset": ... })` by mistake, the non-standard keys remain at the outer result level.

**How to avoid:** The `total`/`limit`/`offset` fields belong inside the `payload` value passed to `CallToolResult::structured()`, NOT as additional fields on the outer `result` JSON object. D-02 explicitly requires nesting all four fields under `structuredContent`.

---

## Code Examples

### Final shape emitted (verified by reading rmcp model.rs)

```json
// Outer JSON-RPC envelope (composed by app/src/controllers/mcp.rs, unchanged)
{
  "jsonrpc": "2.0",
  "id": 42,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"rows\":[...],\"total\":2,\"limit\":25,\"offset\":0}"
      }
    ],
    "structuredContent": {
      "rows": [
        { "id": 1, "customer_name": "Alice Acme", "total": 10.0, "status": "submitted", "tenant_id": 1 },
        { "id": 2, "customer_name": "Alice Acme", "total": 20.0, "status": "submitted", "tenant_id": 1 }
      ],
      "total": 2,
      "limit": 25,
      "offset": 0
    },
    "isError": false
  }
}
```

Source: `rmcp-0.12.0/src/model.rs:1581` (`structured()`), `content.rs:73` (`Content = Annotated<RawContent>`), `content.rs:63` (`#[serde(tag = "type", rename_all = "snake_case")]`).

### Import additions required in `ferro-mcp-server/src/jsonrpc.rs`

```rust
use rmcp::model::CallToolResult;
```

The existing imports are `use serde_json::{json, Value}` — `CallToolResult` is an addition.

---

## Runtime State Inventory

> Not applicable — this is a code-only bug fix. No rename/refactor, no stored state migration.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `target/debug/app` binary | D-06 live dogfood | Yes | built | Requires `cargo build` if stale |
| `app/database.db` | D-06 live dogfood | Yes (seeded) | — | — |
| `app/.env` | D-06 live dogfood | Yes | — | — |
| chrome-devtools-3 (`/tmp/chrome-mcp-3`) | D-06 OAuth flow | Available (MCP configured) | — | Use chrome-devtools-2 |

**Live dogfood (D-06) is a manual verification step.** The binary exists but may be stale if the fix hasn't been built yet. Run `cargo build --bin app` before the dogfood.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (tokio::test for async) |
| Config file | none (workspace Cargo.toml) |
| Quick run command | `cargo test -p ferro-mcp-server` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AMCP-03 (content fix) | `tools/call` result parses as valid `CallToolResult` with `type:text` content block | unit | `cargo test -p ferro-mcp-server tools_call_result_parses` | No — Wave 0 |
| AMCP-03 (structuredContent) | `structuredContent` present, `rows`/`total`/`limit`/`offset` nested correctly | unit | `cargo test -p ferro-mcp-server tools_call_result_parses` | No — Wave 0 |
| AMCP-10 (tenant scoping preserved) | tenant_a_isolation still returns only tenant 1 rows after envelope change | integration | `cargo test -p app tenant_a_isolation` | Yes — update required |
| AMCP-10 (cross-tenant) | tenant_b_isolation still returns only tenant 2 rows | integration | `cargo test -p app tenant_b_isolation` | Yes — update required |
| D-06 (live dogfood) | Claude Code MCP client parses result without Zod errors; alice@acme.test sees 2/4 orders | manual | n/a — manual `autonomous: false` | n/a |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-mcp-server && cargo test -p app`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green + D-06 manual dogfood GO before `/gsd-verify-work`

### Wave 0 Gaps

- `ferro-mcp-server/src/jsonrpc.rs` — add `#[cfg(test)] mod tests { ... }` covering `tools_call_result_parses_as_valid_mcp_content` (REQ: AMCP-03 interop, D-04)
- No new test infrastructure files needed; `tokio = { version = "1", features = ["full", "macros"] }` already in `ferro-mcp-server/dev-dependencies`

---

## Security Domain

Security is not the focus of this phase (it is a result-formatting bug fix). The relevant security properties are preserved, not changed:

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | no change | filter allowlist in dispatch.rs unchanged |
| V4 Access Control (tenant scoping) | no change | SQL predicate injection in dispatch.rs unchanged |
| V4 Access Control (policy gate) | no change | Gate::authorize_for in mcp.rs unchanged |

No new attack surface is introduced by the fix. `CallToolResult::structured()` only changes how the rows are wrapped; the rows themselves come from the same `dispatch()` function with the same security properties.

---

## Confirmed Facts for the Planner

### 1. JSON-RPC envelope composition (the caller contract)

[VERIFIED: ferro-mcp-server/src/jsonrpc.rs line 1-6 + app/src/controllers/mcp.rs lines 160-164]

`handle_tools_call` returns a `serde_json::Value` with shape `{ "result": ... }` or `{ "error": ... }`. The caller in `app/src/controllers/mcp.rs` splices `jsonrpc` and `id` onto this object at lines 161-163 (`obj.insert("jsonrpc", ...)`, `obj.insert("id", ...)`). The fix must produce `{ "result": <serialized-CallToolResult> }` — the outer envelope remains intact.

### 2. `rmcp::model::Content` / `CallToolResult` Deserialize availability

[VERIFIED: rmcp-0.12.0/src/model/content.rs line 62 (`RawContent` derives `Deserialize`); annotated.rs line 39 (`Annotated<T>` derives `Deserialize`); model.rs line 1646 (custom `Deserialize` impl for `CallToolResult`)]

- `Content = Annotated<RawContent>` — derives `Serialize + Deserialize` (both via standard derive)
- `CallToolResult` — derives `Serialize` only, has a **custom `impl<'de> Deserialize<'de>`** at model.rs line 1646 that validates mutual exclusivity of `content`/`structuredContent`. `serde_json::from_value::<CallToolResult>` compiles and works.
- The interop test MUST use `CallToolResult` (not just `Vec<Content>`) to exercise the mutual-exclusivity validation and confirm the complete shape.

### 3. camelCase serialization confirmed

[VERIFIED: rmcp-0.12.0/src/model.rs line 1532 `#[serde(rename_all = "camelCase")]` on `CallToolResult`]

Rust field `structured_content` serializes as `"structuredContent"`, `is_error` as `"isError"`. The fix produces the correct wire keys.

### 4. Existing inline test conventions

[VERIFIED: dispatch.rs lines 226-363; renderer.rs lines 68-179]

Both modules use `#[cfg(test)] mod tests { ... }` inline at the bottom. New tests in `jsonrpc.rs` follow this convention. `tokio::test` is available via `dev-dependencies`.

### 5. Existing callers asserting the old shape — must be updated

[VERIFIED: app/src/tests/mcp_tenant_isolation.rs lines 256-280 and 306-328]

Both `tenant_a_isolation` and `tenant_b_isolation` iterate over `result["result"]["content"]` as bare row objects and call `row["tenant_id"].as_i64()`. After the fix, `content[0]` is `{"type":"text","text":"..."}` — a text block. Both tests will fail unless updated to read from `result["result"]["structuredContent"]["rows"]`.

The tenant isolation **behavior** (dispatch.rs SQL predicate) is unchanged. Only the test navigation path changes.

### 6. Live dogfood harness (D-06)

[VERIFIED: app/.env exists, app/database.db seeded, target/debug/app binary present]

Run command: from the `ferro` workspace root, `cd app && ../target/debug/app` (port 8090). Binary may be stale if the fix hasn't been compiled — run `cargo build --bin app` first. Chrome MCP instance 3 (`/tmp/chrome-mcp-3`) is pre-configured. The OAuth verify link is in the DOM (not in button href). Clear `/tmp/chrome-mcp-3/Singleton{Lock,Socket,Cookie}` before each test run to avoid stale sessions.

D-06 is `autonomous: false` — requires a human to observe "no Zod errors" in Claude Code's MCP output.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `json!({ "result": tool_result })` where `tool_result: CallToolResult` serializes correctly because `CallToolResult` derives `Serialize` | Code Examples | Low — rmcp model.rs line 1531 confirms `#[derive(Serialize)]`; serde_json::json! handles it |

All other claims are verified by direct source file inspection.

---

## Open Questions

1. **Does `cargo test --all-features` trigger the disk-full issue?**
   - What we know: The project memory records a recurring `ENOSPC` issue with `cargo test --all-features` (see `project_ferro_disk_full_test_gate.md` in memory).
   - What's unclear: Current disk state.
   - Recommendation: Run `df -h` before the full gate; if disk is tight, run `cargo test -p ferro-mcp-server && cargo test -p app` first, then the full suite.

---

## Sources

### Primary (HIGH confidence)
- `ferro-mcp-server/src/jsonrpc.rs` — bug site, confirmed lines 84-91
- `ferro-mcp-server/src/dispatch.rs` — `DispatchResult` fields confirmed
- `ferry-mcp-server/Cargo.toml` — rmcp 0.12 dependency confirmed
- `app/src/controllers/mcp.rs` — caller envelope splice confirmed, `handle_tools_call` invocation at line 156
- `app/src/tests/mcp_tenant_isolation.rs` — existing tests asserting old shape, confirmed lines 256-280 and 306-328
- `rmcp-0.12.0/src/model.rs:1531-1682` — `CallToolResult` struct, derives, `structured()` constructor, custom `Deserialize`
- `rmcp-0.12.0/src/model/content.rs` — `Content = Annotated<RawContent>`, `#[serde(tag = "type", rename_all = "snake_case")]`
- `rmcp-0.12.0/src/model/annotated.rs` — `Annotated<T>` derives `Serialize + Deserialize`

### Secondary (MEDIUM confidence)
- None required — all claims verified directly from source files.

---

## Metadata

**Confidence breakdown:**
- Bug site and fix: HIGH — source files read directly
- rmcp API (`structured()`, `CallToolResult::Deserialize`): HIGH — rmcp-0.12.0 source read directly
- Test breakage (`mcp_tenant_isolation.rs`): HIGH — test source read, assertions confirmed
- Live dogfood harness: HIGH — files verified to exist on disk

**Research date:** 2026-06-12
**Valid until:** Stable — rmcp 0.12 is pinned; no external documentation drift expected
