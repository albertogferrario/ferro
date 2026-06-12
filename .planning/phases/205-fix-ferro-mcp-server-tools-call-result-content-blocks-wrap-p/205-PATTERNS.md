# Phase 205: Fix ferro-mcp-server tools/call Result Content Blocks - Pattern Map

**Mapped:** 2026-06-12
**Files analyzed:** 2 (1 modify + 1 modify)
**Analogs found:** 2 / 2

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-mcp-server/src/jsonrpc.rs` | request-response handler + inline unit test | request-response | `ferro-mcp-server/src/dispatch.rs` (test structure) + `ferro-mcp-server/src/jsonrpc.rs` itself (Ok arm) | exact — same module, same `#[cfg(test)] mod tests` convention |
| `app/src/tests/mcp_tenant_isolation.rs` | integration test | request-response | `ferro-mcp-server/src/dispatch.rs` tests (pattern: call function, navigate result value, assert per-row fields) | role-match |

---

## Pattern Assignments

### `ferro-mcp-server/src/jsonrpc.rs` — Ok arm replacement + new test block

**Analog for the fix:** `ferro-mcp-server/src/jsonrpc.rs` lines 83-91 (the defective arm being replaced)
**Analog for the test structure:** `ferro-mcp-server/src/dispatch.rs` lines 226-363

---

#### Current defective arm (lines 83-91) — replace this entirely:

```rust
// ferro-mcp-server/src/jsonrpc.rs lines 83-91 — THE BUG
match dispatch(service, filters, limit, offset, db, tenant_id).await {
    Ok(result) => json!({
        "result": {
            "content": result.rows,
            "total": result.total,
            "limit": result.limit,
            "offset": result.offset
        }
    }),
```

---

#### Import addition — add to existing imports block (currently line 10):

**Existing imports** (lines 7-10 of `jsonrpc.rs`):
```rust
use crate::config::McpServerConfig;
use crate::{dispatch, render_exposed_tools, McpContext};
use ferro_projections::ServiceDef;
use serde_json::{json, Value};
```

**Add one line:**
```rust
use rmcp::model::CallToolResult;
```

`rmcp` is already a declared dependency (`ferro-mcp-server/Cargo.toml` line 15: `rmcp = { version = "0.12", default-features = false, features = ["server", "macros", "base64"] }`). No `Cargo.toml` change needed.

---

#### Fixed Ok arm — replace lines 83-91 with:

Source pattern: `rmcp-0.12.0/src/model.rs:1581-1588` (`CallToolResult::structured`), verified.

```rust
// D-01 + D-02: single structured value, valid camelCase MCP content block
Ok(result) => {
    let payload = serde_json::json!({
        "rows": result.rows,
        "total": result.total,
        "limit": result.limit,
        "offset": result.offset
    });
    let tool_result = CallToolResult::structured(payload);
    json!({ "result": tool_result })
}
```

Notes on this pattern:
- `CallToolResult::structured(value)` (model.rs:1581) sets `content: vec![Content::text(value.to_string())]`, `structured_content: Some(value)`, `is_error: Some(false)`.
- `#[serde(rename_all = "camelCase")]` on `CallToolResult` (model.rs:1532) serializes `structured_content` → `"structuredContent"`, `is_error` → `"isError"`.
- `json!({ "result": tool_result })` serializes `tool_result` inline via its `Serialize` derive. No `serde_json::to_value()` intermediate needed. No `?` operator.
- `total`/`limit`/`offset` are nested inside `payload` (D-02), not as extra top-level keys on the outer `"result"` object.

---

#### New inline test block — append at end of `jsonrpc.rs` (after line 100):

Convention source: `ferro-mcp-server/src/dispatch.rs` lines 226-363 and `ferro-mcp-server/src/renderer.rs` lines 68-179. Both use `#[cfg(test)] mod tests { use super::*; ... }` at module bottom. Tests use `setup_orders_db()` helper + `ServiceDef::new("order")` fixture.

**Test fixture helpers** (copy from `dispatch.rs` lines 233-293 — identical DB + ServiceDef needs):

```rust
// dispatch.rs lines 233-268 — setup_orders_db (reuse verbatim)
async fn setup_orders_db() -> sea_orm::DatabaseConnection {
    let db = sea_orm::Database::connect("sqlite::memory:")
        .await
        .expect("sqlite connect");

    db.execute(sea_orm::Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            customer_name TEXT NOT NULL,
            total REAL NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            tenant_id INTEGER NOT NULL
        )"
        .to_string(),
    ))
    .await
    .expect("create table");

    db.execute(sea_orm::Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "INSERT INTO orders (customer_name, total, status, tenant_id) VALUES
            ('Alice', 100.0, 'pending', 1),
            ('Bob',   200.0, 'shipped', 1),
            ('Carol', 150.0, 'pending', 2),
            ('Dave',  250.0, 'shipped', 2)"
            .to_string(),
    ))
    .await
    .expect("seed rows");

    db
}

// dispatch.rs lines 271-293 — order_service_with_tenant fixture (reuse verbatim)
fn order_service_with_tenant() -> ferro_projections::ServiceDef {
    ferro_projections::ServiceDef::new("order")
        .mcp_exposed(true)
        .tenant_column("tenant_id")
        .mcp_ability("view-orders")
        .field("id", ferro_projections::DataType::Integer, ferro_projections::FieldMeaning::Identifier)
        .field("customer_name", ferro_projections::DataType::String, ferro_projections::FieldMeaning::EntityName)
        .field("total", ferro_projections::DataType::Float, ferro_projections::FieldMeaning::Money)
        .field("status", ferro_projections::DataType::String, ferro_projections::FieldMeaning::Status)
        .field("created_at", ferro_projections::DataType::String, ferro_projections::FieldMeaning::CreatedAt)
        .field("tenant_id", ferro_projections::DataType::Integer, ferro_projections::FieldMeaning::ForeignKey)
}
```

**The D-04 interop test:**

```rust
// New test — append inside #[cfg(test)] mod tests block in jsonrpc.rs
// Imports required inside the mod:
//   use super::*;
//   use ferro_projections::{DataType, FieldMeaning, ServiceDef};
//   use rmcp::model::CallToolResult;
//   use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};

#[tokio::test]
async fn tools_call_result_parses_as_valid_mcp_content() {
    let db = setup_orders_db().await;
    let services = vec![order_service_with_tenant()];
    let call_params = serde_json::json!({
        "name": "list_order",
        "arguments": { "limit": 10 }
    });

    let response = handle_tools_call(call_params, &services, &db, Some(1)).await;

    // D-04: parse the emitted result with the MCP client's own type.
    // serde_json::from_value::<CallToolResult> uses the custom Deserialize impl
    // at rmcp-0.12.0/src/model.rs:1646 which validates mutual-exclusivity.
    let parsed: CallToolResult =
        serde_json::from_value(response["result"].clone())
            .expect("result must parse as CallToolResult (D-04 interop)");

    // isError must be false (structured() sets Some(false))
    assert_eq!(parsed.is_error, Some(false));

    // Exactly one content block (D-03: single text block, not one per row)
    assert_eq!(parsed.content.len(), 1, "structured() produces exactly one content block");

    // The single content block must be a Text variant with the "type" field
    // (RawContent is #[serde(tag = "type", rename_all = "snake_case")] — content.rs:63)
    let content_json = serde_json::to_value(&parsed.content).unwrap();
    assert_eq!(
        content_json[0]["type"].as_str(),
        Some("text"),
        "content[0] must have type=text (was missing before fix)"
    );

    // structuredContent must be present with rows/total/limit/offset (D-02)
    let sc = parsed.structured_content.expect("structuredContent must be present");
    assert!(sc.get("rows").is_some(), "structuredContent.rows must be present");
    assert!(sc.get("total").is_some(), "structuredContent.total must be present");
    assert!(sc.get("limit").is_some(), "structuredContent.limit must be present");
    assert!(sc.get("offset").is_some(), "structuredContent.offset must be present");

    // rows must reflect tenant scoping (tenant_id=1 → 2 rows)
    let rows = sc["rows"].as_array().expect("rows is an array");
    assert_eq!(rows.len(), 2, "tenant 1 has 2 rows");
}
```

**Complete `#[cfg(test)] mod tests` block structure** (following dispatch.rs lines 226-230):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ferro_projections::{DataType, FieldMeaning, ServiceDef};
    use rmcp::model::CallToolResult;
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};

    // [setup_orders_db() — copy from dispatch.rs:233-268]
    // [order_service_with_tenant() — copy from dispatch.rs:271-282, adjusted imports]
    // [tools_call_result_parses_as_valid_mcp_content — as above]
}
```

---

### `app/src/tests/mcp_tenant_isolation.rs` — update two test functions

**Analog:** `ferro-mcp-server/src/dispatch.rs` tests lines 295-325 (navigate `result.rows`, assert `row["tenant_id"].as_i64()`)

The fix is purely a navigation path change. The isolation *behavior* (SQL predicate in dispatch.rs) is unchanged. Only where to find `rows` in the returned `Value` changes.

---

#### Old navigation pattern (lines 256-280 in `tenant_a_isolation`, lines 306-328 in `tenant_b_isolation`) — replace both:

```rust
// OLD — asserting the broken shape; must be replaced in both tests
let rows = result["result"]["content"]
    .as_array()
    .expect("result.content must be an array");
// ...
for row in rows {
    let tid = row["tenant_id"]
        .as_i64()
        .expect("each row must have a tenant_id field");
    // ...
}
let tenant_2_leak = rows.iter().any(|r| r["tenant_id"].as_i64() == Some(2));
```

#### New navigation pattern — replace both occurrences with:

Source: The wire shape produced by `CallToolResult::structured()` (rmcp model.rs:1581-1588), verified.

```rust
// NEW — navigate the valid MCP envelope after the fix

// 1. Assert content[0] is a valid text block (not a bare row object)
let content = result["result"]["content"]
    .as_array()
    .expect("result.content must be an array");
assert_eq!(
    content[0]["type"].as_str(),
    Some("text"),
    "content[0] must be a text block (type=text)"
);

// 2. Read rows from structuredContent (D-02: nested under structuredContent)
let rows = result["result"]["structuredContent"]["rows"]
    .as_array()
    .expect("structuredContent.rows must be an array");
assert!(
    !rows.is_empty(),
    "tenant N must have at least one order in the result"
);

// 3. Same tenant_id assertions as before — only the path to `rows` changes
for row in rows {
    let tid = row["tenant_id"]
        .as_i64()
        .expect("each row must have a tenant_id field");
    // assert_eq!(tid, <expected_tenant_id>, "...");
}
// let tenant_N_leak = rows.iter().any(|r| r["tenant_id"].as_i64() == Some(<other_tid>));
```

**`tenant_a_isolation` specific assertions** (lines 269-280 of the original, replace):
```rust
// tenant_a_isolation: expected_tenant_id = 1, other_tid = 2
assert_eq!(tid, 1, "tenant A isolation: row tenant_id must be 1, got {tid}");
// ...
let tenant_2_leak = rows.iter().any(|r| r["tenant_id"].as_i64() == Some(2));
assert!(!tenant_2_leak, "tenant A isolation: no row must have tenant_id == 2 (cross-tenant leak)");
```

**`tenant_b_isolation` specific assertions** (lines 319-329 of the original, replace):
```rust
// tenant_b_isolation: expected_tenant_id = 2, other_tid = 1
assert_eq!(tid, 2, "tenant B isolation: row tenant_id must be 2, got {tid}");
// ...
let tenant_1_leak = rows.iter().any(|r| r["tenant_id"].as_i64() == Some(1));
assert!(!tenant_1_leak, "tenant B isolation: no row must have tenant_id == 1 (cross-tenant leak)");
```

---

## Shared Patterns

### JSON-RPC result envelope shape (unchanged outer contract)

**Source:** `ferro-mcp-server/src/jsonrpc.rs` lines 1-5 (module doc), `app/src/controllers/mcp.rs` (caller splices `jsonrpc`/`id`)

`handle_tools_call` returns `{ "result": <value> }` or `{ "error": <value> }`. The outer `jsonrpc`/`id` fields are spliced by the caller. The fix only changes what `<value>` is for the `Ok` arm. The error arms (lines 95-99) are unchanged.

```rust
// Unchanged error arms (jsonrpc.rs lines 95-99) — do not touch
Err(crate::Error::InvalidFilter(msg)) => {
    json!({ "error": { "code": -32602, "message": msg } })
}
Err(e) => json!({ "error": { "code": -32603, "message": e.to_string() } }),
```

### Inline test module convention

**Source:** `ferro-mcp-server/src/dispatch.rs` lines 226-227, `ferro-mcp-server/src/renderer.rs` lines 68-69

Both existing modules open their test block identically:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    // ...
}
```

`tokio::test` is available — `tokio = { version = "1", features = ["full", "macros"] }` is already in `ferro-mcp-server` dev-dependencies (Cargo.toml line 24).

### `rmcp::model::CallToolResult` Deserialize availability

**Source:** `rmcp-0.12.0/src/model.rs` lines 1531 (derive — Serialize only, no Deserialize in derive), 1646 (custom `impl<'de> Deserialize<'de> for CallToolResult`)

`serde_json::from_value::<CallToolResult>(v)` compiles and works. The custom impl at line 1646 validates mutual exclusivity of `content`/`structuredContent`. The test must use `CallToolResult` (not just `Vec<Content>`) to exercise this validation.

### `Content = Annotated<RawContent>` tag serialization

**Source:** `rmcp-0.12.0/src/model/content.rs` lines 62-71

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RawContent {
    Text(RawTextContent),
    // ...
}
pub type Content = Annotated<RawContent>;
```

The `#[serde(tag = "type")]` attribute means serializing a `Text` variant always produces `{"type":"text",...}`. This is why `content[0]["type"] == "text"` is the correct assertion — it will always be present when the content block is a `Text` variant.

---

## No Analog Found

None. Both files have direct, confirmed analogs in the existing codebase.

---

## Metadata

**Analog search scope:** `ferro-mcp-server/src/` (all modules), `app/src/tests/`, `~/.cargo/registry/src/index.crates.io-*/rmcp-0.12.0/src/`
**Files read:** `jsonrpc.rs`, `dispatch.rs`, `renderer.rs`, `mcp_tenant_isolation.rs`, `ferro-mcp-server/Cargo.toml`, `rmcp-0.12.0/src/model.rs` (lines 1525-1682), `rmcp-0.12.0/src/model/content.rs`, `rmcp-0.12.0/src/model/annotated.rs`
**Pattern extraction date:** 2026-06-12
