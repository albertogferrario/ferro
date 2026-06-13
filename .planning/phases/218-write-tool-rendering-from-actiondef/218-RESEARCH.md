# Phase 218: Write-Tool Rendering from ActionDef — Research

**Researched:** 2026-06-13
**Domain:** `ferro-mcp-server` — `McpRenderer` / `render_exposed_tools` / `schema.rs` extension
**Confidence:** HIGH (all claims verified against source files read in this session)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Write tool name = `action.name` verbatim; disambiguate collisions as `<action.name>_on_<service.name>`. Must not start with `list_`.
- **D-02:** `build_action_input_schema(action: &ActionDef, service: &ServiceDef) -> Result<Value>` in `schema.rs`. Inject the parent `ServiceDef`'s first `FieldMeaning::Identifier` field as a required integer param. Promote `data_type_to_json_schema` to `pub(crate)`. Exclude `FieldMeaning::Sensitive` inputs (same gate as `is_filter_field`).
- **D-03:** For each `action.precondition`, check `ctx.evaluated_guards.get(precondition)`. Any `Some(false)` → omit tool. Absent key → show tool. Runtime guard population is 219 — 218 uses explicit-map tests.
- **D-04:** `ToolAnnotations::new().read_only(false).destructive(action.transition_trigger.is_some())`. Do NOT set `idempotentHint` — no `ActionDef` attribute for it in 218.
- **D-05:** Extend `render_exposed_tools`: per `mcp_exposed` service emit `list_<service>` first, then one write tool per `ActionDef` in declaration order.
- **D-06:** All logic in `ferro-mcp-server` (`renderer.rs` + `schema.rs`). Do not modify `ferro-projections`.
- **D-07:** Extend the Phase 205 strict-deserialization regression test (located in `ferro-mcp-server/src/jsonrpc.rs`, test `tools_call_result_parses_as_valid_mcp_content`) to cover write tool definitions via `tools/list`.

### Claude's Discretion
- Exact signature/return shape of `build_action_input_schema` (mirror `build_input_schema`).
- Whether `data_type_to_json_schema` is promoted to `pub(crate)` or wrapped in a shared helper.
- Test fixture shape (a service with 1 read + ≥2 actions, one guarded, one with `transition_trigger`).

### Deferred Ideas (OUT OF SCOPE)
- `dispatch_write()` + server-side guard re-evaluation at call time + idempotency key + audit log — Phase 219.
- `ferro-ai` confirmation gating + synthesized `confirm_<action>` tools + TTL — Phase 220.
- `idempotentHint` annotation and any explicit `destructive`/`irreversible`/`requires_confirmation` flag on `ActionDef` — revisit in 219/220.
- Inbound NL classification to tool+args — Phase 221.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AMCP-03 | Each `ServiceDef`'s guarded actions are projected into MCP write tools (input schema derived from `ActionDef` inputs), exposed in `tools/list` only when the tenant's guards for that action pass, and annotated for the agent (read-only vs destructive). Tool definitions derived purely from `ServiceDef` — no hand-authored per-tool surface. | Fully supported: `ActionDef`/`InputDef` fields verified; `render_exposed_tools` extension point identified; `ToolAnnotations` API confirmed; guard filter pattern established in `McpContext.evaluated_guards`; SC#5 regression test location pinned. |
</phase_requirements>

---

## Summary

Phase 218 is a rendering-only extension to `ferro-mcp-server`. All five success criteria are achievable with surgical changes to two files (`renderer.rs`, `schema.rs`) plus test additions in `jsonrpc.rs` and a new unit-test module.

The Phase 217 foundation (`McpContext` with `tenant_id`, `evaluated_guards`, `scope`) is already shipped and committed. `McpContext` is at `ferro-mcp-server/src/renderer.rs:18–22` with `evaluated_guards: HashMap<String, bool>` fully present. The `render_exposed_tools` function (line 68–78) is the single extension point: it currently emits one `list_<name>` read tool per `mcp_exposed` service; 218 adds a second loop inside the same function to emit one write tool per `ActionDef`.

The `data_type_to_json_schema` function in `schema.rs` (line 44, currently `fn` = private) must be promoted to `pub(crate)` so `build_action_input_schema` (new function in the same file) can reuse the exact same type mapping. The sensitive-field exclusion for write tool inputs is `FieldMeaning::Sensitive` only — this is the single meaning the existing `is_filter_field` excludes by name; no other `FieldMeaning` variants function as a "password/secret/token" analog in the current codebase.

The Phase 205 strict-deserialization regression test is NOT in `ferro-mcp-server/tests/jsonrpc_integration.rs` — it is an inline `#[tokio::test]` at the bottom of `ferro-mcp-server/src/jsonrpc.rs` (line 188, test name `tools_call_result_parses_as_valid_mcp_content`). D-07's instruction to extend it must target that location. However, that test covers `tools/call` results; SC#5 in Phase 218 concerns write *tool definitions* emitted in `tools/list` (not callable yet). The correct extension is to add a parallel test that deserializes each write tool via rmcp's `Tool` type from the `tools/list` response.

**Primary recommendation:** Add `render_action_tool(service, action, ctx)` as a private helper in `renderer.rs`; add `build_action_input_schema(action, service)` in `schema.rs` (mirrors `build_input_schema` over `InputDef`). Extend `render_exposed_tools` with an inner loop. Tests run as pure unit tests with `McpContext { evaluated_guards, .. }` set explicitly — no runtime guard population hook needed for 218.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Write tool emission (tools/list) | MCP Server (`ferro-mcp-server`) | — | `McpRenderer` already lives here; boundary rule: concrete renderers stay in output crates |
| Action input schema derivation | MCP Server (`schema.rs`) | — | Mirrors existing `build_input_schema`; parallel function |
| Guard-filter decision | MCP Server (`renderer.rs`) | — | Reads `ctx.evaluated_guards` at render time; same logic as visual renderer's `BaseContext` |
| `ToolAnnotations` (destructiveHint) | MCP Server (`renderer.rs`) | — | Annotation derived from `ActionDef.transition_trigger`, emitted at render time |
| `ActionDef` / `InputDef` schema | ferro-projections | — | Already owns these; 218 reads but does NOT modify them |
| Write tool execution (dispatch) | MCP Server (Phase 219) | — | Out of scope for 218 |

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rmcp` | 0.12.0 | MCP protocol types (`Tool`, `ToolAnnotations`, `CallToolResult`) | Already in `ferro-mcp-server/Cargo.toml`; no upgrade needed |
| `ferro-projections` | workspace | `ActionDef`, `InputDef`, `ServiceDef`, `FieldMeaning` | Canonical projection schema; 218 reads only |
| `serde_json` | 1.0 | `json!` macro, `Value`, `Map` for schema assembly | Already present |

### No new dependencies
Phase 218 adds zero new crate dependencies. All required types are already imported.

---

## Architecture Patterns

### Verified Touch-Points with Line References

**`ferro-mcp-server/src/renderer.rs`**

```
L1–6:   use std::collections::HashMap; ... use rmcp::model::{Tool, ToolAnnotations};
L17–22: McpContext { tenant_id, evaluated_guards, scope } — already populated by Phase 217
L29:    pub struct McpRenderer;
L31–60: impl Renderer for McpRenderer — produces the list_<name> read tool
L57:    let annotations = ToolAnnotations::new().read_only(true);  ← pattern to mirror with .read_only(false).destructive(...)
L59:    Ok(Tool::new(name, description, Arc::new(schema_map)).annotate(annotations))
L68–78: pub fn render_exposed_tools(...) → THE extension point
```

**`ferro-mcp-server/src/schema.rs`**

```
L14–38: pub fn is_filter_field(field: &FieldDef) -> bool  — gate 3 excludes FieldMeaning::Sensitive
L44–55: fn data_type_to_json_schema(dt: DataType) -> Value  — currently PRIVATE; must become pub(crate)
L67–101: pub fn build_input_schema(service: &ServiceDef) -> Result<Value>  — template for build_action_input_schema
```

**`ferro-mcp-server/src/jsonrpc.rs`**

```
L62–78: is_write_tool = !tool_name.starts_with("list_")  — the 217 scope gate; action-named tools slot in correctly
L80–88: service lookup via service_name = tool_name.strip_prefix("list_")  — CRITICAL: this breaks for write tools (see Contradiction §1)
L188–231: inline #[tokio::test] tools_call_result_parses_as_valid_mcp_content  — THE Phase 205 regression test
```

**`ferro-projections/src/action.rs`** (read-only in 218)

```
ActionDef fields: name, display_name, description, inputs: Vec<InputDef>, preconditions: Vec<String>, effects: Vec<String>, transition_trigger: Option<String>
InputDef fields: name, data_type: DataType, meaning: FieldMeaning, required: bool (default true), description: Option<String>
```

**`ferro-projections/src/service.rs`**

```
ServiceDef.actions: Vec<ActionDef>  (line 71)
ServiceDef.mcp_exposed: bool  (line 84, default false)
— Identifier field: first FieldDef where meaning == FieldMeaning::Identifier; must scan service.fields
```

---

### rmcp `ToolAnnotations` API — Verified

Source: `/Users/alberto/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rmcp-0.12.0/src/model/tool.rs`

```rust
pub struct ToolAnnotations {
    pub title: Option<String>,
    pub read_only_hint: Option<bool>,    // field name (camelCase in JSON: readOnlyHint)
    pub destructive_hint: Option<bool>,  // field name (camelCase in JSON: destructiveHint)
    pub idempotent_hint: Option<bool>,   // field name (camelCase in JSON: idempotentHint)
    pub open_world_hint: Option<bool>,   // field name (camelCase in JSON: openWorldHint)
}

impl ToolAnnotations {
    pub fn new() -> Self { Self::default() }
    pub fn read_only(self, read_only: bool) -> Self { ... }    // sets read_only_hint
    pub fn destructive(self, destructive: bool) -> Self { ... } // sets destructive_hint
    pub fn idempotent(self, idempotent: bool) -> Self { ... }  // sets idempotent_hint
    pub fn open_world(self, open_world: bool) -> Self { ... }  // sets open_world_hint
}
```

The builder methods are:
- `.read_only(false)` sets `read_only_hint: Some(false)` — correct for write tools
- `.destructive(bool)` sets `destructive_hint: Some(bool)` — driven by `action.transition_trigger.is_some()` per D-04
- `.idempotent(bool)` — NOT set in 218 per D-04 (no `ActionDef` attribute)

**D-04 is confirmed correct:** `ToolAnnotations::new().read_only(false).destructive(action.transition_trigger.is_some())` is the exact API.

Note: `destructive_hint` defaults (when absent) to `true` per `is_destructive() -> bool { self.destructive_hint.unwrap_or(true) }`. Always setting it explicitly from `transition_trigger.is_some()` is therefore slightly conservative (non-transition actions get `destructive_hint: false` rather than letting it default to `true`). This is the correct behavior per D-04 and SC#4.

---

### Concrete Shape: `build_action_input_schema`

```rust
// ferro-mcp-server/src/schema.rs — new function

/// Builds the MCP tool `inputSchema` for a write tool derived from `action`.
///
/// Injects the parent service's first `FieldMeaning::Identifier` field as a
/// required integer parameter (the record to act on). Then adds each `InputDef`
/// from `action.inputs`, mapping `data_type` via `data_type_to_json_schema` and
/// respecting `required`. `FieldMeaning::Sensitive` inputs are excluded (matches
/// `is_filter_field` gate 3 precedent).
///
/// `action.preconditions` are NOT in the schema — they drive list-time guard
/// filtering only. `action.effects` are not rendered.
pub fn build_action_input_schema(
    action: &ActionDef,
    service: &ServiceDef,
) -> crate::Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();
    let mut required_fields: Vec<String> = Vec::new();

    // Inject the identifier field (the record to act on) — always required.
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

    // Map each InputDef; exclude Sensitive meanings.
    for input in &action.inputs {
        if matches!(input.meaning, FieldMeaning::Sensitive) {
            continue; // gate: no sensitive fields in action inputs (D-02 + PITFALLS §3)
        }
        let mut prop = data_type_to_json_schema(input.data_type);
        if let serde_json::Value::Object(ref mut m) = prop {
            if let Some(ref desc) = input.description {
                m.insert("description".into(), serde_json::Value::String(desc.clone()));
            }
        }
        properties.insert(input.name.clone(), prop);
        if input.required {
            required_fields.push(input.name.clone());
        }
    }

    Ok(serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required_fields,
    }))
}
```

**Key design notes:**
- If `service.fields` has no `FieldMeaning::Identifier` field, no identifier is injected — the schema is still valid (some actions may act on a resource identified differently, or create a new record). The planner should decide whether to make this a soft warning or hard error; the code above skips silently.
- The `InputDef.data_type` field is `DataType` (same type as `FieldDef.data_type`), so `data_type_to_json_schema` accepts it directly after the promotion to `pub(crate)`.
- `required_fields` is a `Vec<String>` to maintain declaration order.

---

### How `render_exposed_tools` Should Be Extended

Current implementation (lines 68–78):

```rust
pub fn render_exposed_tools(services: &[ServiceDef], ctx: &McpContext)
    -> std::result::Result<Vec<Tool>, ProjError>
{
    let renderer = McpRenderer;
    services
        .iter()
        .filter(|s| s.mcp_exposed)
        .map(|s| renderer.render(s, &derive_intents(s), ctx))
        .collect()
}
```

218 extension pattern:

```rust
pub fn render_exposed_tools(services: &[ServiceDef], ctx: &McpContext)
    -> std::result::Result<Vec<Tool>, ProjError>
{
    let renderer = McpRenderer;
    let mut tools = Vec::new();

    for service in services.iter().filter(|s| s.mcp_exposed) {
        // 1. Read tool (existing behavior)
        tools.push(renderer.render(service, &derive_intents(service), ctx)?);

        // 2. Write tools: one per ActionDef, in declaration order, guard-filtered
        for action in &service.actions {
            if let Some(tool) = render_action_tool(service, action, ctx)? {
                tools.push(tool);
            }
        }
    }

    Ok(tools)
}

/// Renders one write tool from an `ActionDef`, or `None` if any precondition
/// guard evaluates to explicit `false` for the calling tenant (D-03).
fn render_action_tool(
    service: &ServiceDef,
    action: &ActionDef,
    ctx: &McpContext,
) -> std::result::Result<Option<Tool>, ProjError> {
    // Guard filter (D-03): any precondition explicitly false → omit
    for precondition in &action.preconditions {
        if ctx.evaluated_guards.get(precondition) == Some(&false) {
            return Ok(None);
        }
    }

    let name = action.name.clone(); // D-01: verbatim, no hand-authored override
    let description = action.description.clone()
        .or_else(|| action.display_name.clone())
        .unwrap_or_else(|| format!("{} {}", action.name, service.name));

    let schema_value = crate::schema::build_action_input_schema(action, service)
        .map_err(|e| ProjError::Render(e.to_string()))?;

    let schema_map = match schema_value {
        serde_json::Value::Object(m) => m,
        _ => return Err(ProjError::Render("action inputSchema must be an object".into())),
    };

    let annotations = ToolAnnotations::new()
        .read_only(false)
        .destructive(action.transition_trigger.is_some()); // D-04

    Ok(Some(Tool::new(name, description, Arc::new(schema_map)).annotate(annotations)))
}
```

**Name collision handling (D-01):** The caller (or a future validator) is responsible for detecting collisions across services. The renderer should emit `action.name` verbatim for each service individually; the collision guard is a post-processing step or a `ServiceDef.validate()` check. For 218, a simple approach: collect all action names across all services; for any name that appears in more than one service, rewrite those tools' names to `<action.name>_on_<service.name>`. This can be a post-processing pass over the collected `Vec<Tool>` before returning.

---

## Critical Finding: Contradiction in `handle_tools_call` (Resolved)

**Location:** `ferro-mcp-server/src/jsonrpc.rs:62`

```rust
let service_name = tool_name.strip_prefix("list_").unwrap_or(tool_name);
```

**Problem:** When `tool_name` is a write tool name (e.g., `"submit_order"`), `strip_prefix("list_")` returns `None`, so `unwrap_or(tool_name)` makes `service_name = "submit_order"`. The subsequent service lookup `services.iter().find(|s| s.name == service_name && s.mcp_exposed)` looks for a service named `"submit_order"` — which does not exist. The result is always `-32601 Method not found` for any write tool call in Phase 218.

**This is intentional for 218.** Write tools are visible (SC#1–SC#4) but not dispatched (SC#5 is about tool *definitions*, not execution). The existing "method not found" path is the correct 218 behavior for any write tool call attempt — tools are well-formed and visible, they just have no executor. Phase 219 adds `dispatch_write` and routes write tool calls properly.

**Recommendation:** Add a comment in `handle_tools_call` at line 62 noting that write-tool routing is wired in Phase 219. No code change is needed in `jsonrpc.rs` for Phase 218 except the addition of the SC#5 regression test.

The 217 scope gate (`!tool_name.starts_with("list_")`) correctly classifies write tools because action names never start with `list_` (they are verbs like `submit_order`, `cancel_order`). No change needed.

---

## Guard-Filter Testability (SC#3 Resolved)

**Question:** Is `ctx.evaluated_guards` actually populated at runtime after Phase 217, or always empty?

**Answer confirmed from code:** `McpContext` comment at line 13–14 in `renderer.rs` reads: `"evaluated_guards: populated in Phase 218/219; absent key = allow, explicit false = deny (same semantics as BaseContext). Empty in 217."` Phase 217 was a context foundation phase — `evaluated_guards` is always `HashMap::new()` at runtime until a population hook exists. There is no population code in `jsonrpc.rs` (confirmed by reading `handle_tools_list` — it passes `ctx` through but nothing fills `evaluated_guards` before calling `render_exposed_tools`).

**SC#3 is fully testable with render-level unit tests alone.** Because the guard filter logic lives inside `render_action_tool` and reads from `ctx.evaluated_guards`, tests can set `ctx.evaluated_guards` explicitly:

```rust
// Tool with guard "has_items" omitted when guard = false
let mut ctx = McpContext::default();
ctx.evaluated_guards.insert("has_items".into(), false);
let tools = render_exposed_tools(&[service_with_action], &ctx).unwrap();
// assert: no write tool "submit_order" in tools

// Same guard absent → tool present
let ctx2 = McpContext::default(); // evaluated_guards is empty
let tools2 = render_exposed_tools(&[service_with_action], &ctx2).unwrap();
// assert: write tool "submit_order" present

// Guard = true → tool present
let mut ctx3 = McpContext::default();
ctx3.evaluated_guards.insert("has_items".into(), true);
let tools3 = render_exposed_tools(&[service_with_action], &ctx3).unwrap();
// assert: write tool "submit_order" present
```

**No minimal runtime population hook is needed in Phase 218 for AMCP-03 to be satisfied.** SC#3 requires testing at the render function level; the planner should include unit tests covering all three states (absent, true, false) with explicit map construction.

---

## Phase 205 Strict-Deserialization Regression Test (SC#5 Resolved)

**Location:** `ferro-mcp-server/src/jsonrpc.rs`, lines 186–231, test name: `tools_call_result_parses_as_valid_mcp_content`

**What the existing test does:**
1. Calls `handle_tools_call` with `list_order` args against an in-memory SQLite DB
2. Asserts `response["result"]` deserializes as `rmcp::model::CallToolResult` (the strict rmcp type)
3. Asserts `parsed.content[0]["type"] == "text"` (the Phase 205 content-block bug fix)
4. Asserts `parsed.structured_content.rows/total/limit/offset` present

**What SC#5 means for 218:** Write tools cannot be called yet (no executor), so there is no `tools/call` result to test. SC#5 in the context of 218 means: verify that the write tool *definitions* emitted by `tools/list` are well-formed and deserialize strictly via rmcp's `Tool` type (catching any malformed `inputSchema` shape, missing `type` field, or broken annotation shape).

**Concrete extension for SC#5:**

Add a new test in `ferro-mcp-server/src/jsonrpc.rs` (inline module) or `ferro-mcp-server/tests/jsonrpc_integration.rs`:

```rust
#[tokio::test]
async fn write_tools_definitions_parse_as_valid_mcp_tool() {
    use rmcp::model::Tool;

    // Build a service with one read tool and two write tools
    let service = ServiceDef::new("order")
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
        );

    let ctx = McpContext::default();
    let config = McpServerConfig { app_name: "Test".into(), app_url: "https://test".into(), version: "0.0.0".into() };
    let resp = handle_tools_list(&[service], &ctx, &config).await;

    let tools_json = resp["result"]["tools"].as_array().expect("tools array");
    // 1 read tool + 2 write tools = 3 total
    assert_eq!(tools_json.len(), 3);

    for tool_json in tools_json {
        // Load-bearing: rmcp's strict Tool deserializer must accept each definition
        let tool: Tool = serde_json::from_value(tool_json.clone())
            .expect("each tool definition must parse as rmcp::model::Tool");
        assert!(!tool.name.is_empty(), "tool name must not be empty");
        // inputSchema must be a JSON object
        assert!(tool.input_schema.contains_key("type") || tool.input_schema.contains_key("properties"),
            "inputSchema must have 'type' or 'properties': {:?}", &*tool.input_schema);
    }

    // Write tool "submit_order" must have readOnlyHint: false and destructiveHint: true
    let submit = tools_json.iter().find(|t| t["name"] == "submit_order").expect("submit_order tool");
    assert_eq!(submit["annotations"]["readOnlyHint"], false);
    assert_eq!(submit["annotations"]["destructiveHint"], true);

    // Write tool "update_notes" must have readOnlyHint: false and destructiveHint: false
    let update = tools_json.iter().find(|t| t["name"] == "update_notes").expect("update_notes tool");
    assert_eq!(update["annotations"]["readOnlyHint"], false);
    assert_eq!(update["annotations"]["destructiveHint"], false);
}
```

This test is the correct SC#5 analog: it verifies that write tool *definitions* are well-formed at the rmcp type level, parallel to how the Phase 205 test verifies that `tools/call` *results* are well-formed.

---

## Sensitive-Field Exclusion: Verified Scope

**Finding:** `FieldMeaning` in `ferro-projections/src/field.rs` has ONE sensitive variant: `Sensitive` (line 53). There is no `Password`, `Token`, or `Secret` variant. The infer_meaning function maps common field name patterns (`password`, `hashed_password`, `secret`, `api_key`, `hashed_key`) all to `FieldMeaning::Sensitive`.

**Implication for `build_action_input_schema`:** The exclusion gate is:
```rust
if matches!(input.meaning, FieldMeaning::Sensitive) { continue; }
```
This is the complete exclusion set. No other variant functions as a "secret" in the current schema.

**Existing precedent in `is_filter_field`:** Gate 3 (line 21–22 of `schema.rs`) is `if matches!(field.meaning, FieldMeaning::Sensitive) { return false; }` — exactly this pattern. The write path mirrors it.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| `DataType → JSON Schema` mapping | A second copy of the match table | Promote `data_type_to_json_schema` to `pub(crate)` and reuse | Already handles all 8 DataType variants with correct formats |
| Guard filter logic | Per-action manual checks | `ctx.evaluated_guards.get(name) == Some(&false)` pattern (mirror `BaseContext.evaluated_guards` semantics) | Already tested in Phase 215/217 context |
| `Tool` construction | Raw JSON object construction | `Tool::new(name, description, Arc::new(schema_map)).annotate(annotations)` | rmcp's own type enforces correct structure |
| Tool annotation | Setting `read_only_hint` / `destructive_hint` directly | Builder methods `.read_only(false).destructive(bool)` | Builder handles the `Option<bool>` wrapping |

---

## Common Pitfalls

### Pitfall 1: Breaking `handle_tools_call` service lookup for write tools
**What goes wrong:** The existing lookup `services.iter().find(|s| s.name == service_name)` uses `service_name = tool_name.strip_prefix("list_").unwrap_or(tool_name)`. For write tool names like `"submit_order"`, this makes `service_name = "submit_order"` — no service matches. This is correct behavior for 218 (no executor), but could be confusing.
**Prevention:** Add a comment at the write-tool no-match path noting that write-tool dispatch is Phase 219.

### Pitfall 2: Forgetting the `scope` gate check for write tools
**What happens:** `handle_tools_call` already checks `!tool_name.starts_with("list_")` to detect write tools and rejects them for `read`-scoped keys. This check correctly classifies all action-named tools as write tools because action names are verbs (submit, cancel, approve) — none start with `list_`. No change needed, but implementors must ensure no new action name is prefixed `list_`.

### Pitfall 3: Sensitive InputDef not excluded
**What goes wrong:** An `InputDef` with `meaning: FieldMeaning::Sensitive` (e.g., a hypothetical action input `password_confirm`) appears in the tool schema, potentially leaking that the endpoint accepts sensitive values in plaintext through an agent.
**Prevention:** The `matches!(input.meaning, FieldMeaning::Sensitive)` gate in `build_action_input_schema` is non-negotiable.

### Pitfall 4: `data_type_to_json_schema` not promoted
**What goes wrong:** Adding a second copy of the `DataType` match table in `build_action_input_schema` creates two sources of truth. When a new `DataType` variant is added, only one copy gets updated.
**Prevention:** Promote `data_type_to_json_schema` to `pub(crate)` — one line change in `schema.rs:44`. Do not duplicate the function.

### Pitfall 5: `ToolAnnotations::new()` default behavior for `destructive_hint`
**What goes wrong:** Not setting `destructive_hint` at all causes `ToolAnnotations::is_destructive()` to return `true` (default). Every write tool would appear destructive to MCP clients. Setting `.destructive(action.transition_trigger.is_some())` correctly emits `destructive_hint: false` for non-transition actions.
**Prevention:** Always call `.destructive(action.transition_trigger.is_some())` in D-04 pattern.

### Pitfall 6: Confusing `tools/list` guard-filter with authorization
**Risk:** Reviewers may read the guard filter in `render_action_tool` and assume it is the security boundary. It is not — it is a visibility mechanism. Phase 219 adds server-side re-evaluation at `tools/call` time.
**Prevention:** Comment `render_action_tool` with `// Visibility filter, not auth gate — see Phase 219 for enforcement`.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `#[tokio::test]` |
| Config file | none (uses Cargo's built-in test runner) |
| Quick run command | `cargo test -p ferro-mcp-server --lib` |
| Full suite command | `cargo test -p ferro-mcp-server` |

### Phase Requirements → Test Map

| SC | Behavior | Test Type | Location | File Exists? |
|----|----------|-----------|----------|--------------|
| SC#1 | `tools/list` includes one write tool per `ActionDef`; name = `action.name` | unit | `renderer.rs` `#[cfg(test)]` mod | Wave 0 |
| SC#2 | Input schema derived from `ActionDef.inputs` via `build_action_input_schema`; identifier injected | unit | `schema.rs` `#[cfg(test)]` mod | Wave 0 |
| SC#3 | Guard `Some(false)` → tool absent; `None`/`Some(true)` → tool present | unit | `renderer.rs` `#[cfg(test)]` mod | Wave 0 |
| SC#4 | `readOnlyHint: false`; `destructiveHint` true for transition actions, false for others | unit | `renderer.rs` `#[cfg(test)]` mod | Wave 0 |
| SC#5 | Every write tool's definition deserializes via `rmcp::model::Tool` (strict) | integration | new test in `tests/jsonrpc_integration.rs` or inline in `jsonrpc.rs` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-mcp-server --lib` (unit tests only, fast)
- **Per wave merge:** `cargo test -p ferro-mcp-server` (all tests including integration)
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] Unit test module in `renderer.rs` — extend existing `#[cfg(test)]` with guard-filter tests (SC#1, SC#3, SC#4)
- [ ] Unit test module in `schema.rs` — `test_build_action_input_schema_*` functions (SC#2)
- [ ] Integration test in `tests/jsonrpc_integration.rs` or `src/jsonrpc.rs` — `write_tools_definitions_parse_as_valid_mcp_tool` (SC#5)

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Auth done in Phase 217 |
| V3 Session Management | no | Stateless JSON-RPC |
| V4 Access Control | yes (advisory) | Guard filter in `render_action_tool`; NOT the auth gate (Phase 219 re-evaluates) |
| V5 Input Validation | no | Rendering-only; no input handling |
| V6 Cryptography | no | No new crypto surface |

### Phase 218-Specific Security Notes

- **Guard filter is NOT an authorization gate.** Document this in code and in phase review. Phase 219 is the auth gate.
- **Sensitive fields excluded from schemas.** `FieldMeaning::Sensitive` inputs are never emitted in tool schemas, preventing agents from being prompted to supply passwords or API keys via a write tool parameter.
- **No executor in 218.** The scope gate in `handle_tools_call` already rejects write tool calls for `read`-scoped keys. A write tool name reaching the service lookup returns `-32601 Method not found` — not an exploitable surface.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | — | — | — |

**All claims in this research were verified against source files read in this session — no unverified assumptions.**

---

## Open Questions

1. **Identifier injection when `service.fields` has no `FieldMeaning::Identifier`**
   - What we know: `build_action_input_schema` silently skips identifier injection if none found.
   - What's unclear: Should this be a `crate::Error` or a silent no-op?
   - Recommendation: Silent no-op for 218 (some actions may not need a record ID, e.g., a "create" action). Phase 219 (dispatch) can validate at call time.

2. **Name collision detection across services**
   - What we know: D-01 says disambiguate collisions as `<action.name>_on_<service.name>`.
   - What's unclear: When to detect collisions — at render time (in `render_exposed_tools`) or as a post-processing pass?
   - Recommendation: Post-processing pass in `render_exposed_tools` after collecting all tools. Scan for duplicate names; rename only the colliding entries. This avoids two passes over the service list.

3. **`data_type_to_json_schema` for `DataType::Json` and `DataType::Binary`**
   - What we know: The existing function returns `{ "type": "string" }` for both (the `_` arm).
   - What's unclear: Should `Json`-typed `InputDef` fields be excluded from write schemas (as they are from filter schemas via `is_filter_field` gate 4)?
   - Recommendation: For write tools, `Json`/`Binary` inputs should be allowed — an action may legitimately take a JSON payload as input. The `is_filter_field` exclusion is specific to equality-filter uselessness, not to writability. No exclusion needed for write schemas.

---

## Sources

### Primary (HIGH confidence)
- `ferro-mcp-server/src/renderer.rs` — `McpRenderer`, `McpContext`, `render_exposed_tools`; read in full
- `ferro-mcp-server/src/schema.rs` — `build_input_schema`, `data_type_to_json_schema`, `is_filter_field`; read in full
- `ferro-mcp-server/src/jsonrpc.rs` — `handle_tools_call`, `handle_tools_list`, Phase 205 regression test; read in full
- `ferro-mcp-server/src/lib.rs` — public exports; read in full
- `ferro-mcp-server/Cargo.toml` — rmcp 0.12.0 confirmed
- `ferro-projections/src/action.rs` — `ActionDef`, `InputDef`, `GuardDef`; read in full
- `ferro-projections/src/service.rs` — `ServiceDef` with `actions`, `guards`, `mcp_exposed`, `fields`; read in full
- `ferro-projections/src/field.rs` — `FieldMeaning` enum, `FieldMeaning::Sensitive` as the only sensitive variant
- `ferro-mcp-server/tests/jsonrpc_integration.rs` — integration test structure
- `ferro-mcp-server/tests/mcp_tenant_isolation.rs` — Phase 217 test patterns; confirmed `evaluated_guards` is empty at runtime in 217
- `ferro-mcp-server/tests/common/mod.rs` — test fixture helpers
- `/Users/alberto/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rmcp-0.12.0/src/model/tool.rs` — `ToolAnnotations` struct and builder methods; read in full
- Cargo.lock — rmcp version 0.12.0 confirmed

### Secondary (MEDIUM confidence)
- `.planning/research/ARCHITECTURE.md` — Design decisions (a), (b), (c), (d); verified against actual code
- `.planning/research/PITFALLS.md` — Pitfall §2 (guard advisory vs. enforcement), §3 (sensitive field scoping), §5 (destructiveHint)
- `.planning/research/FEATURES.md` — "Tool input schema derived from ServiceDef", "MCP-specific handler code per action" anti-pattern

---

## Metadata

**Confidence breakdown:**
- `ActionDef`/`InputDef` fields: HIGH — verified from source
- `ToolAnnotations` API (field names, builder methods): HIGH — verified from rmcp 0.12.0 source
- `render_exposed_tools` extension pattern: HIGH — verified from source; proposed extension is a clean insertion
- `build_action_input_schema` shape: HIGH — designed as direct mirror of `build_input_schema` using same helpers
- Guard-filter testability (SC#3): HIGH — confirmed `evaluated_guards` empty in 217 runtime from renderer.rs comment + test inspection
- Phase 205 test location (SC#5): HIGH — pinned to `ferro-mcp-server/src/jsonrpc.rs:188`, test `tools_call_result_parses_as_valid_mcp_content`

**Research date:** 2026-06-13
**Valid until:** 2026-07-13 (stable codebase, no fast-moving dependencies in this phase)
