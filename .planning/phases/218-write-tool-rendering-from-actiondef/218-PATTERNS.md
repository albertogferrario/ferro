# Phase 218: Write-Tool Rendering from ActionDef - Pattern Map

**Mapped:** 2026-06-13
**Files analyzed:** 4 (2 MODIFY, 1 READ/MODIFY for test extension, 2 READ-ONLY)
**Analogs found:** 4 / 4 — all within the same files being modified

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-mcp-server/src/schema.rs` | utility | transform | `build_input_schema` + `data_type_to_json_schema` in the SAME file (lines 44–101) | exact — parallel function over `InputDef` instead of `FieldDef` |
| `ferro-mcp-server/src/renderer.rs` | renderer | request-response | `McpRenderer::render` + `render_exposed_tools` in the SAME file (lines 31–78) | exact — write-tool path mirrors read-tool path |
| `ferro-mcp-server/src/jsonrpc.rs` | test extension | request-response | `tools_call_result_parses_as_valid_mcp_content` at line 188 in the SAME file | role-match — SC#5 test covers `tools/list` definitions, not `tools/call` result |
| `ferro-mcp-server/tests/jsonrpc_integration.rs` | integration test | request-response | existing `tools_list_returns_only_exposed` test (lines 27–42) | role-match — same `handle_tools_list` + service fixture pattern |

---

## Pattern Assignments

### `ferro-mcp-server/src/schema.rs` — new `build_action_input_schema`

**Analog:** `build_input_schema` (lines 67–101) + `data_type_to_json_schema` (lines 44–55) + `is_filter_field` gate 3 (lines 21–23) — all in the same file.

**Visibility promotion** (line 44 — one-word change):
```rust
// BEFORE (private):
fn data_type_to_json_schema(dt: DataType) -> serde_json::Value {

// AFTER (promote to pub(crate) — D-02 requires reuse from build_action_input_schema):
pub(crate) fn data_type_to_json_schema(dt: DataType) -> serde_json::Value {
```

**Sensitive-field exclusion gate pattern** (`is_filter_field`, lines 21–23):
```rust
// ferro-mcp-server/src/schema.rs lines 21-23
if matches!(field.meaning, FieldMeaning::Sensitive) {
    return false;
} // gate 3
```
Copy this pattern verbatim into `build_action_input_schema` as:
```rust
if matches!(input.meaning, FieldMeaning::Sensitive) {
    continue; // PITFALLS §3: no sensitive fields in write tool schemas
}
```

**Core schema-builder pattern** (`build_input_schema`, lines 67–101):
```rust
// ferro-mcp-server/src/schema.rs lines 67-101
pub fn build_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();

    // [insert static properties here — e.g., limit/offset for read, or id for write]

    for field in service.fields.iter().filter(|f| is_filter_field(f)) {
        let mut prop = data_type_to_json_schema(field.data_type);
        if let serde_json::Value::Object(ref mut m) = prop {
            m.insert(
                "description".into(),
                serde_json::Value::String(format!("Filter by {}", field.name)),
            );
        }
        properties.insert(field.name.clone(), prop);
    }

    Ok(serde_json::json!({ "type": "object", "properties": properties }))
}
```

**`build_action_input_schema` must mirror this structure exactly**, replacing:
- the field iteration over `service.fields.iter().filter(is_filter_field)` with `action.inputs.iter()` (with the Sensitive skip)
- the static `limit`/`offset` block with the identifier injection block
- adding a `required_fields: Vec<String>` accumulator (read schema has no `required` array; write schema does)
- the final `json!` emitting `{ "type": "object", "properties": properties, "required": required_fields }`

**Identifier injection block** (new, no analog — follows the same `data_type_to_json_schema` + `properties.insert` pattern):
```rust
// Find first FieldMeaning::Identifier in service.fields; inject as required integer param
if let Some(id_field) = service.fields.iter().find(|f| {
    matches!(f.meaning, FieldMeaning::Identifier)
}) {
    let mut prop = data_type_to_json_schema(id_field.data_type);
    if let serde_json::Value::Object(ref mut m) = prop {
        m.insert(
            "description".into(),
            serde_json::Value::String(format!(
                "ID of the {} record to act on",
                service.display_name.as_deref().unwrap_or(&service.name)
            )),
        );
    }
    properties.insert(id_field.name.clone(), prop);
    required_fields.push(id_field.name.clone());
}
```

**Test pattern** (mirror `schema.rs` tests at lines 104–201):
```rust
// ferro-mcp-server/src/schema.rs lines 115-122 — test fixture pattern to replicate
fn sample_service() -> ServiceDef {
    ServiceDef::new("order")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("status", DataType::String, FieldMeaning::Status)
        .field("customer_id", DataType::Integer, FieldMeaning::ForeignKey)
        .field("name", DataType::String, FieldMeaning::EntityName)
        .field("password", DataType::String, FieldMeaning::Sensitive)
}
```
SC#2 tests must assert: identifier injected as required param, `InputDef` fields present, `FieldMeaning::Sensitive` inputs absent, `required` array contains required inputs.

---

### `ferro-mcp-server/src/renderer.rs` — extend `render_exposed_tools` + add `render_action_tool`

**Analog:** `McpRenderer::render` (lines 35–60) — the read-tool path is the direct analog for the write-tool path.

**Tool construction pattern** (lines 49–59 — copy shape exactly for write tools):
```rust
// ferro-mcp-server/src/renderer.rs lines 49-59
let schema_value = crate::schema::build_input_schema(service)
    .map_err(|e| ProjError::Render(e.to_string()))?;

let schema_map = match schema_value {
    serde_json::Value::Object(m) => m,
    _ => return Err(ProjError::Render("inputSchema must be an object".into())),
};

let annotations = ToolAnnotations::new().read_only(true);  // <-- change to .read_only(false).destructive(...)

Ok(Tool::new(name, description, Arc::new(schema_map)).annotate(annotations))
```

For write tools, replace:
- `crate::schema::build_input_schema(service)` → `crate::schema::build_action_input_schema(action, service)`
- `ToolAnnotations::new().read_only(true)` → `ToolAnnotations::new().read_only(false).destructive(action.transition_trigger.is_some())`
- `"inputSchema must be an object"` error string → `"action inputSchema must be an object"`

**`render_exposed_tools` extension pattern** (lines 68–78 — current body to replace):
```rust
// ferro-mcp-server/src/renderer.rs lines 68-78 — CURRENT (read tools only)
pub fn render_exposed_tools(
    services: &[ServiceDef],
    ctx: &McpContext,
) -> std::result::Result<Vec<Tool>, ProjError> {
    let renderer = McpRenderer;
    services
        .iter()
        .filter(|s| s.mcp_exposed)
        .map(|s| renderer.render(s, &ferro_projections::derive_intents(s), ctx))
        .collect()
}
```
Replace iterator `.map().collect()` with an explicit `for` loop that calls `renderer.render(...)` first, then iterates `service.actions` calling the new `render_action_tool` helper. Return type and signature are unchanged.

**Guard-filter pattern** (D-03 — no existing analog in renderer.rs; mirrors `BaseContext.evaluated_guards` semantics described in comments at lines 13–14):
```rust
// In render_action_tool: check ALL preconditions; any explicit false = omit
for precondition in &action.preconditions {
    if ctx.evaluated_guards.get(precondition) == Some(&false) {
        return Ok(None);
    }
}
// Absent key = show (same semantics as BaseContext; see renderer.rs line 13-14 comment)
```

**Description fallback chain** (D-05 — no existing analog; straightforward option chaining):
```rust
let description = action.description.clone()
    .or_else(|| action.display_name.clone())
    .unwrap_or_else(|| format!("{} {}", action.name, service.name));
```

**Name collision post-processing** (D-01 — no analog; post-processing pass after all tools collected):
Collect all write tool names across all services. Detect duplicates. For any name appearing in more than one service, rewrite those specific tool entries to `<action.name>_on_<service.name>`. Read tool names (`list_<name>`) are never affected.

**Existing test fixture to extend** (lines 85–100 — use as base for new SC#1/SC#3/SC#4 test fixtures):
```rust
// ferro-mcp-server/src/renderer.rs lines 85-100
fn order_service() -> ServiceDef {
    ServiceDef::new("order")
        .display_name("Order")
        .description("Manages customer orders")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("status", DataType::String, FieldMeaning::Status)
        .field("customer_id", DataType::Integer, FieldMeaning::ForeignKey)
}
```
Extend with `.mcp_exposed(true).action(ActionDef::new(...).precondition("has_items").transition_trigger("submit")).action(ActionDef::new("update_notes")...)` for guard-filter and annotation tests.

**Existing annotation assertion pattern** (lines 108–116 — mirror for write-tool assertions):
```rust
// ferro-mcp-server/src/renderer.rs lines 108-116
#[test]
fn test_render_read_only() {
    let tool = render_service(&order_service());
    let annotations = tool.annotations.expect("annotations present");
    assert_eq!(
        annotations.read_only_hint,
        Some(true),
        "readOnlyHint must be true"
    );
}
```
SC#4 test: assert `read_only_hint == Some(false)` and `destructive_hint == Some(true)` for transition action, `destructive_hint == Some(false)` for non-transition action.

---

### `ferro-mcp-server/src/jsonrpc.rs` — SC#5 strict-deserialization test extension

**Analog:** `tools_call_result_parses_as_valid_mcp_content` (lines 188–231) — the Phase 205 regression test. SC#5 is the write-tool-definition parallel of this test.

**Phase 205 test body** (lines 188–231 — the load-bearing structure to mirror):
```rust
// ferro-mcp-server/src/jsonrpc.rs lines 188-231
#[tokio::test]
async fn tools_call_result_parses_as_valid_mcp_content() {
    let db = setup_orders_db().await;
    let services = vec![order_service_with_tenant()];
    let call_params = serde_json::json!({
        "name": "list_order",
        "arguments": { "limit": 10 }
    });

    let response =
        handle_tools_call(call_params, &services, &db, Some(1), &McpContext::default()).await;

    // The load-bearing assertion: the client's own type must deserialize it.
    let parsed: CallToolResult = serde_json::from_value(response["result"].clone())
        .expect("result must parse as CallToolResult (D-04 interop)");

    assert_eq!(parsed.is_error, Some(false));
    // ... further assertions on content and structured_content
}
```

**SC#5 test** differs in three ways:
1. Calls `handle_tools_list` (not `handle_tools_call`) — no DB needed
2. Deserializes each entry in `response["result"]["tools"]` as `rmcp::model::Tool` (not `CallToolResult`)
3. Asserts annotation fields (`readOnlyHint`, `destructiveHint`) on write tool entries

**SC#5 test fixture shape** — must use a service with 1 read tool + ≥2 write actions (one with `transition_trigger`, one without, one guard-blocked):
```rust
// Service fixture for SC#5 (mirror order_service_with_tenant at lines 172-183)
fn order_service_with_actions() -> ServiceDef {
    ServiceDef::new("order")
        .mcp_exposed(true)
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("status", DataType::String, FieldMeaning::Status)
        .guard(GuardDef::new("has_items"))
        .action(
            ActionDef::new("submit_order")
                .description("Submit an order for processing")
                .input(InputDef::new("notes", DataType::String, FieldMeaning::FreeText).required(false))
                .precondition("has_items")
                .transition_trigger("submit"),
        )
        .action(
            ActionDef::new("update_notes")
                .input(InputDef::new("notes", DataType::String, FieldMeaning::FreeText)),
        )
}
```

**`handle_tools_list` call pattern** (lines 34–42 of `jsonrpc.rs` — no DB, just services + ctx + config):
```rust
// ferro-mcp-server/src/jsonrpc.rs lines 34-42
pub async fn handle_tools_list(
    services: &[ServiceDef],
    ctx: &McpContext,
    _config: &McpServerConfig,
) -> Value {
    match render_exposed_tools(services, ctx) {
        Ok(tools) => json!({ "result": { "tools": tools } }),
        Err(e) => json!({ "error": { "code": -32603, "message": e.to_string() } }),
    }
}
```

**Existing integration test fixture** (lines 9–41 of `tests/jsonrpc_integration.rs` — `test_config()` + `McpContext::default()` pattern):
```rust
// ferro-mcp-server/tests/jsonrpc_integration.rs lines 10-16
fn test_config() -> McpServerConfig {
    McpServerConfig {
        app_name: "TestApp".to_string(),
        app_url: "https://test.example".to_string(),
        version: "0.0.0".to_string(),
    }
}
```
SC#5 integration test (if placed in `tests/jsonrpc_integration.rs`) uses this exact `test_config()` helper.

**`rmcp::model::Tool` strict-deserializer** (D-07 load-bearing assertion pattern):
```rust
// The load-bearing assertion (parallel to CallToolResult parse at jsonrpc.rs:201)
let tool: rmcp::model::Tool = serde_json::from_value(tool_json.clone())
    .expect("each write tool definition must parse as rmcp::model::Tool");
assert!(!tool.name.is_empty(), "tool name must not be empty");
```

**Decision on test location:** Place SC#5 in `ferro-mcp-server/src/jsonrpc.rs` inline `#[cfg(test)]` module (same location as Phase 205 test at line 188), NOT in `tests/jsonrpc_integration.rs`. Rationale: SC#5 tests the tool-definition rendering path (no DB needed); placing it beside the Phase 205 test keeps both "MCP protocol wire format" regression tests co-located.

---

## Shared Patterns

### Tool Construction (`Tool::new(...).annotate(annotations)`)
**Source:** `ferro-mcp-server/src/renderer.rs` line 59
**Apply to:** `render_action_tool` helper (new) in `renderer.rs`
```rust
Ok(Tool::new(name, description, Arc::new(schema_map)).annotate(annotations))
```
The `Arc::new(schema_map)` wrapping is required — `Tool::new` takes `Arc<Map<String, Value>>`. The `schema_map` must come from destructuring `serde_json::Value::Object(m)` (see the match block at lines 52–55).

### Error Mapping (`ProjError::Render`)
**Source:** `ferro-mcp-server/src/renderer.rs` lines 50, 54
**Apply to:** `render_action_tool` helper
```rust
.map_err(|e| ProjError::Render(e.to_string()))?
// and
return Err(ProjError::Render("action inputSchema must be an object".into()))
```
All schema-building errors convert to `ProjError::Render(string)` — no new error variants.

### `serde_json::Map` property accumulation
**Source:** `ferro-mcp-server/src/schema.rs` lines 68–99
**Apply to:** `build_action_input_schema`
```rust
let mut properties = serde_json::Map::new();
// ... insert entries ...
properties.insert(field.name.clone(), prop);
```
The `prop` value comes from `data_type_to_json_schema(...)` (a `serde_json::Value::Object`), then mutably borrowed to insert a `description` key before inserting into `properties`.

### `McpContext::default()` in tests
**Source:** `ferro-mcp-server/src/renderer.rs` line 98, `ferro-mcp-server/tests/jsonrpc_integration.rs` line 38
**Apply to:** All new unit and integration tests
```rust
&McpContext::default()
```
`McpContext` derives `Default`; `evaluated_guards` is `HashMap::new()` (empty = show all). Override with explicit `HashMap::from([("guard_name".into(), false)])` for guard-filter tests.

### `use` imports for new code in `renderer.rs`
**Source:** `ferro-mcp-server/src/renderer.rs` lines 1–6
```rust
use std::collections::HashMap;
use std::sync::Arc;

use ferro_projections::render::Renderer;
use ferro_projections::{Error as ProjError, IntentScore, ServiceDef};
use rmcp::model::{Tool, ToolAnnotations};
```
Add to the existing import list: `use ferro_projections::{ActionDef, FieldMeaning};` (needed in `render_action_tool`). These types are already in the workspace — no new `Cargo.toml` dependencies.

### `use` imports for new code in `schema.rs`
**Source:** `ferro-mcp-server/src/schema.rs` line 1
```rust
use ferro_projections::{DataType, FieldDef, FieldMeaning, ServiceDef};
```
Add `ActionDef, InputDef` to this import when adding `build_action_input_schema`.

---

## No Analog Found

All files have close analogs within the same codebase. No file requires falling back to RESEARCH.md patterns.

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | — |

---

## Critical Implementation Notes (from RESEARCH.md contradictions)

### `handle_tools_call` service lookup — intentional no-op for 218
`ferro-mcp-server/src/jsonrpc.rs` line 62:
```rust
let service_name = tool_name.strip_prefix("list_").unwrap_or(tool_name);
```
For write tool names (e.g., `"submit_order"`), `strip_prefix("list_")` returns `None`, so `service_name = "submit_order"`. The service lookup fails → `-32601 Method not found`. This is correct 218 behavior (no executor yet). The planner must add a comment here noting write-tool dispatch is Phase 219. **Do not "fix" this line in Phase 218.**

### `data_type_to_json_schema` visibility
Currently `fn` (private, line 44). Must become `pub(crate)` — single-character change. Do not duplicate the function body.

### `FieldMeaning::Sensitive` is the only exclusion variant
There is no `Password`, `Token`, or `Secret` variant in `FieldMeaning`. The gate is exactly `matches!(input.meaning, FieldMeaning::Sensitive)` — nothing else.

### `destructive_hint` defaults to `true` when absent
`ToolAnnotations::is_destructive()` returns `true` when `destructive_hint` is `None`. Always call `.destructive(action.transition_trigger.is_some())` to emit an explicit `false` for non-transition actions. Never omit this call.

---

## Metadata

**Analog search scope:** `ferro-mcp-server/src/` (renderer.rs, schema.rs, jsonrpc.rs), `ferro-mcp-server/tests/`, `ferro-projections/src/action.rs`, `ferro-projections/src/service.rs`
**Files scanned:** 8 source files + 1 test common module + rmcp 0.12.0 ToolAnnotations source
**Pattern extraction date:** 2026-06-13
