# Phase 240: CRUD Input-Schema Derivation + `list_` Query Polish — Research

**Researched:** 2026-06-23
**Domain:** ferro-projections / ferro-mcp-server schema derivation and dispatch
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01 — Phase scope split:** Phase 240 delivers, for `create_/update_/delete_<svc>`:
tool listing in `render_exposed_tools` + a derived `inputSchema` per verb. It does NOT
make those tools executable — calling them is wired in Phase 241. The existing action
write-path and Phase 205 structured-envelope regression guard must stay green.

**D-02 — `list_` query polish lands fully this phase:** schema derivation in `schema.rs`
AND WHERE/ORDER-BY assembly in `dispatch.rs`, because `list_` already executes.
`limit`/`offset` already derive — do NOT re-implement them; only confirm they remain
covered.

**D-03 — Creatable field set:** `is_server_injected_field` (239 substrate) drops
Identifier, CreatedAt, and the tenant column. Additionally exclude `FieldMeaning::Sensitive`
and list fields (`is_list`). Everything else declared via `field()` is a creatable input.

**D-04 — Status under SM:** when `service.state_machine` is `Some`, the Status field
(`FieldMeaning::Status`) is excluded from the create schema (set server-side to SM
initial state in Phase 241). When no SM exists, Status is an ordinary creatable input.

**D-05 — UpdatedAt excluded from write schemas:** extend the write-schema exclusion
predicate to also drop `FieldMeaning::UpdatedAt` (server-managed). Keep exclusion
logic in ONE shared predicate so create and update agree.

**D-06 — update_<svc> patch schema:** identifier (required) + same data-field set as
create but ALL optional (patch semantics). Reuse the create field-set predicate
(D-03/D-05) so create and update never drift.

**D-07 — Status under SM on update:** same exclusion as D-04 applies to updates.

**D-08 — delete_<svc> schema:** identifier (required) + confirmation-token field,
`destructiveHint=true`. The confirmation mechanism and soft-delete execution are
Phase 241/242; this phase only emits a correctly shaped schema.

**D-09 — Range/comparison filters as flat sibling keys:** `<field>__gt`, `<field>__gte`,
`<field>__lt`, `<field>__lte`, `<field>__ne`, `<field>__in` alongside existing equality
params (which are unchanged). `__in` typed as array; others share the field's scalar
JSON type via `data_type_to_json_schema`.

**D-10 — Op eligibility by field type:** `__ne` and `__in` for every field passing
`is_filter_field`. `__gt/__gte/__lt/__lte` only for numeric (`Integer`/`Float`) and
date/time (`DateTime`/`Date`) columns. Add dedicated `is_range_filter_field`; do NOT
mutate `is_filter_field` (equality back-compat is a stated success criterion).

**D-11 — `sort` param:** single optional string `field` (asc) or `-field` (desc).
Base field allowlisted against the dispatch filter-key allowlist. Existing
Identifier-based deterministic ORDER BY is kept as tiebreaker appended after user
sort. Single sort key only (multi-key deferred).

**D-12 — Dispatch extension for `__op` keys:** split each non-equality key on the
LAST `__`, validate suffix against op allowlist `{gt,gte,lt,lte,ne,in}`, validate
base against field allowlist. Map to SQL operator with bound parameter
(`IN (?,…)` for arrays). Unknown op or non-filterable base → same error as today.
All values bound — no interpolation.

**D-13 — `sort` in dispatch:** validated ORDER BY (asc/desc by `-` prefix), placed
before the deterministic Identifier tiebreaker and `LIMIT/OFFSET`.

**D-14 — Table tests:** create/update field-set derivation; Status inclusion/exclusion
with vs without SM; full `__{gt,gte,lt,lte,ne,in}` param set per eligible field type;
`sort` param; identifier-required-on-update and all-data-fields-optional.

**D-15 — SQLite in-memory dispatch tests:** range filters return correct rows; `__in`
array filtering; `sort=field`/`sort=-field` ordering; `limit`/`offset` still clamp;
back-compat equality params; tool-listing tests assert three write tools appear with
correct schemas.

### Claude's Discretion

- Exact names of new schema builders (`build_create_input_schema` /
  `build_update_input_schema` / `build_delete_input_schema`) and the shared
  write-field exclusion predicate / `is_range_filter_field` helper.
- Whether the write-field exclusion predicate lives in `ferro-projections`
  (next to `is_server_injected_field`) or in `ferro-mcp-server/src/schema.rs`.
  Prefer co-locating projection-level field-classification in `ferro-projections`
  so non-MCP renderers can reuse it.
- JSON Schema niceties (per-op description strings, `format` propagation to range
  params).
- Whether delete-tool emission is feature-gated behind `confirmation` like the
  existing destructive-action path.

### Deferred Ideas (OUT OF SCOPE)

- `derive_crud_plan` + create/update/delete execution through `framework::write` (Phase 241)
- Write authorization + tenant injection + non-disclosure envelope (Phase 242)
- App `order` projection flip + e2e + catalog/docs (Phase 243)
- Multi-key sort (YAGNI)
- Dedicated `get_<svc>` tool; per-field `immutable()`/`read_only()` overrides
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CRUD-01 | `create_<svc>` MCP tool + auto-derived input schema from `field()` declarations; excludes Identifier, CreatedAt, tenant column, Sensitive; Status set to SM initial state when SM exists | §Schema Builders + §Write-Field Exclusion Predicate |
| CRUD-02 | `update_<svc>` patch schema (all fields optional); data fields only; Status never an update input under SM | §Schema Builders + §Write-Field Exclusion Predicate |
| CRUD-04 | `list_<svc>` range/comparison filters `__{gt,gte,lt,lte,ne,in}`, sort `field`/`-field`, `limit`/`offset` on top of existing equality filters | §Range Filter Extension + §Dispatch Extension |
</phase_requirements>

---

## Summary

Phase 240 is a pure schema-derivation-and-read-execution phase. It has three independent deliverables: (a) emit `create_/update_/delete_<svc>` tools in `tools/list` with correctly derived `inputSchema` — schema-only, no execution wiring; (b) enrich the `list_<svc>` `inputSchema` with range/comparison filter params and a `sort` param; (c) extend the read `dispatch` function to honour those new params in SQL. The code substrate from Phase 239 is fully present: `is_server_injected_field`, `resolved_table`, `resolved_soft_delete_column`, and the `deleted_at IS NULL` predicate.

The primary implementation surface is three files: `ferro-projections/src/service.rs` (write-field exclusion predicate), `ferro-mcp-server/src/schema.rs` (all schema builders + range filter helpers), and `ferro-mcp-server/src/dispatch.rs` (op-key splitting + sort assembly). The emission point in `renderer.rs::render_exposed_tools` is the fourth touch.

The "Phase 241 boundary" is enforced by routing: `handle_tools_call` already branches all non-`list_` calls to `handle_write_call`; for the new `create_/update_/delete_<svc>` verbs, `handle_write_call` must return a structured not-yet-implemented envelope (matching the Phase 205 `CallToolResult::structured` shape) rather than a JSON-RPC error, so the Phase 205 regression guard keeps passing and the tools are technically callable without returning malformed envelopes.

**Primary recommendation:** Author the shared write-field exclusion predicate as `ServiceDef::is_write_excluded_field(&self, field: &FieldDef) -> bool` in `ferro-projections/src/service.rs` (composing `is_server_injected_field`, UpdatedAt, Sensitive, and is_list checks). All three schema builders in `ferro-mcp-server/src/schema.rs` call it. Add `is_range_filter_field(field: &FieldDef) -> bool` to `ferro-mcp-server/src/schema.rs` as a second-tier filter on top of `is_filter_field`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Write-field exclusion predicate | `ferro-projections` (ServiceDef) | — | Projection-level field classification; must be renderer-agnostic for reuse by non-MCP renderers (ferro-text, future visual) |
| `create_/update_/delete_` inputSchema builders | `ferro-mcp-server/schema.rs` | — | MCP-specific schema shape; consumes `ferro-projections` classification |
| Range-filter eligibility (`is_range_filter_field`) | `ferro-mcp-server/schema.rs` | — | MCP-specific filter concept; not meaningful to non-MCP renderers |
| Tool emission (`render_exposed_tools`) | `ferro-mcp-server/renderer.rs` | — | The single tool-list assembly point; add CRUD verb emission here |
| Read dispatch extension (`__op` + `sort`) | `ferro-mcp-server/dispatch.rs` | — | SQL assembly lives here; already owns `where_clauses` and `order_str` |
| Phase 205 envelope guard (regression) | `ferro-mcp-server/jsonrpc.rs` | `write_dispatch.rs` | Must route new verbs through same `CallToolResult::structured` path |

---

## Standard Stack

All libraries are already in the workspace. No new dependencies.

### Core (already present)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ferro-projections` | workspace | `ServiceDef`, `FieldDef`, `FieldMeaning`, `DataType` | The schema source of truth |
| `ferro-mcp-server` | workspace | Schema builders, renderer, dispatch | The MCP output crate |
| `serde_json` | 1.0 | JSON Schema assembly (`serde_json::json!`, `Map`) | Already used in schema.rs, dispatch.rs |
| `rmcp` | 0.12 | `Tool`, `ToolAnnotations`, `CallToolResult::structured` | MCP protocol types |
| `sea_orm` | 1.0 | Parameterized SQL, SQLite/Postgres backend | Already used in dispatch.rs |

[VERIFIED: reading Cargo.toml and all source files in this session]

---

## Architecture Patterns

### System Architecture Diagram

```
ServiceDef (ferro-projections)
  ├─ fields: Vec<FieldDef>          (FieldMeaning, DataType, readable, writable, is_list)
  ├─ creatable / updatable / deletable: bool
  ├─ state_machine: Option<StateMachine>
  ├─ is_server_injected_field(&field) → bool   [Phase 239, shipped]
  └─ is_write_excluded_field(&field) → bool    [NEW Phase 240 — composes 239 helper]
         │
         ▼
ferro-mcp-server/src/schema.rs
  ├─ is_filter_field(&field) → bool            [existing, read-only; DO NOT CHANGE]
  ├─ is_range_filter_field(&field) → bool      [NEW — adds numeric/datetime to filter set]
  ├─ data_type_to_json_schema(dt) → Value      [existing, shared by all builders]
  ├─ build_input_schema(&svc) → Value          [extend: add range/sort params]
  ├─ build_create_input_schema(&svc) → Value   [NEW]
  ├─ build_update_input_schema(&svc) → Value   [NEW]
  └─ build_delete_input_schema(&svc) → Value   [NEW]
         │
         ▼
ferro-mcp-server/src/renderer.rs
  └─ render_exposed_tools(&services, &ctx)
       ├─ per service: list_<svc> read tool     [existing]
       ├─ per ActionDef: write tools            [existing]
       ├─ create_<svc> if creatable             [NEW]
       ├─ update_<svc> if updatable             [NEW]
       └─ delete_<svc> if deletable             [NEW]
         │
         ▼
ferro-mcp-server/src/dispatch.rs
  └─ dispatch(&svc, filters, limit, offset, db, tenant_id)
       ├─ equality filter loop [unchanged]
       ├─ __op filter loop     [NEW: split on last __, op allowlist, bind values]
       ├─ sort parsing         [NEW: -field → DESC, field → ASC]
       ├─ tenant predicate     [unchanged]
       ├─ deleted_at IS NULL   [unchanged, Phase 239]
       ├─ ORDER BY user_sort + id tiebreaker [extended]
       └─ LIMIT/OFFSET         [unchanged]
         │
         ▼
ferro-mcp-server/src/jsonrpc.rs / write_dispatch.rs
  └─ handle_tools_call → handle_write_call
       ├─ create_<svc> / update_<svc> / delete_<svc> calls
       │    → not-yet-implemented structured envelope (Phase 240)
       │    → full execution wired in Phase 241
       └─ Phase 205 regression guard: CallToolResult::structured still valid
```

### Recommended Project Structure

No new files needed. All changes land in existing files:

```
ferro-projections/src/service.rs   — add is_write_excluded_field + tests
ferro-mcp-server/src/schema.rs     — add is_range_filter_field, extend build_input_schema,
                                      add build_create/update/delete_input_schema + tests
ferro-mcp-server/src/renderer.rs   — extend render_exposed_tools for CRUD verb emission
ferro-mcp-server/src/dispatch.rs   — extend for __op keys + sort + tests
ferro-mcp-server/src/write_dispatch.rs — add CRUD verb routing returning NTI envelope
```

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON Schema object assembly | Custom struct | `serde_json::json!` + `Map::new()` | Already the pattern in `build_action_input_schema`; consistent and readable |
| Placeholder per backend | Custom match | `placeholder(backend, idx)` in dispatch.rs | Already handles SQLite vs Postgres `$N` vs `?` |
| SQL value binding | String interpolation | `json_to_sea_value` + `Statement::from_sql_and_values` | Existing parameterized path; interpolation = injection |
| IN clause value extraction | Custom JSON walking | `val.as_array()` + map `json_to_sea_value` per element | serde_json already has `as_array()` |
| Identifier field lookup | Manual loop | `service.fields.iter().find(|f| matches!(f.meaning, FieldMeaning::Identifier))` | Used verbatim in `build_action_input_schema` |
| "Is this an ordered type" check | Semantic inference | `is_range_filter_field` based on `DataType` only | DataType enum is the canonical type; no inference needed |

**Key insight:** the entire schema derivation infrastructure already exists and is battle-tested. Phase 240 only adds new callers of `data_type_to_json_schema` and new `where_clauses.push(...)` call sites — no new infrastructure.

---

## Write-Field Exclusion Predicate

This is the most load-bearing new concept. The predicate is called by `build_create_input_schema` and `build_update_input_schema` (they must agree; divergence = schema drift).

### Predicate: `ServiceDef::is_write_excluded_field(&self, field: &FieldDef) -> bool`

Location: `ferro-projections/src/service.rs`, next to `is_server_injected_field` (line 236).

Gates in order:

```rust
// Gate A: server-injected (Identifier, CreatedAt, tenant column) — shipped Phase 239
if self.is_server_injected_field(field) {
    return true;
}
// Gate B: UpdatedAt — server-managed timestamp
if matches!(field.meaning, FieldMeaning::UpdatedAt) {
    return true;
}
// Gate C: Sensitive meaning — never an agent write input
if matches!(field.meaning, FieldMeaning::Sensitive) {
    return true;
}
// Gate D: list fields — not useful as write inputs
if field.is_list {
    return true;
}
// Gate E (create/update context-dependent): Status under SM
// NOTE: this gate is conditional on the SM presence, so callers pass a `bool`
// or the method takes an `exclude_status: bool` parameter.
// See §Pattern 1 below for the correct interface shape.
false
```

### Status-under-SM exclusion

The Status exclusion is conditional: `service.state_machine.is_some()`. Two clean options:

**Option A (preferred):** Single predicate with a `exclude_sm_status: bool` parameter.

```rust
pub fn is_write_excluded_field(&self, field: &FieldDef, exclude_sm_status: bool) -> bool
```

Callers:
- `build_create_input_schema` → passes `self.state_machine.is_some()`
- `build_update_input_schema` → passes `self.state_machine.is_some()`

**Option B:** Two predicates `is_create_excluded` / `is_update_excluded`. Duplicates logic; not preferred.

Option A is recommended: single predicate, one parameter that callers derive from `service.state_machine.is_some()`.

[VERIFIED: reading service.rs and context decisions D-03/D-04/D-05/D-06/D-07]

---

## Schema Builders

### `build_create_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value>`

Located in `ferro-mcp-server/src/schema.rs`. Pattern mirrors `build_action_input_schema`.

```rust
pub fn build_create_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();
    let mut required_fields: Vec<String> = Vec::new();

    let exclude_sm_status = service.state_machine.is_some();

    for field in &service.fields {
        if service.is_write_excluded_field(field, exclude_sm_status) {
            continue;
        }
        let mut prop = data_type_to_json_schema(field.data_type);
        // add description
        properties.insert(field.name.clone(), prop);
        if field.required {
            required_fields.push(field.name.clone());
        }
    }

    Ok(serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required_fields,
    }))
}
```

[VERIFIED: pattern matches `build_action_input_schema` at schema.rs:111]

### `build_update_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value>`

Same field set as create, but:
- The **Identifier field is added first as required** (the record to patch).
- All data fields are **optional** (no `required` entries for them).

```rust
pub fn build_update_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();
    let mut required_fields: Vec<String> = Vec::new();

    // Inject identifier (required) — mirrors build_action_input_schema
    if let Some(id_field) = service.fields.iter()
        .find(|f| matches!(f.meaning, FieldMeaning::Identifier)) {
        let mut prop = data_type_to_json_schema(id_field.data_type);
        // add description
        properties.insert(id_field.name.clone(), prop);
        required_fields.push(id_field.name.clone());
    }

    let exclude_sm_status = service.state_machine.is_some();

    // All data fields are optional (patch semantics — D-06)
    for field in &service.fields {
        if service.is_write_excluded_field(field, exclude_sm_status) {
            continue;
        }
        let mut prop = data_type_to_json_schema(field.data_type);
        // add description
        properties.insert(field.name.clone(), prop);
        // NOT added to required_fields
    }

    Ok(serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required_fields,
    }))
}
```

### `build_delete_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value>`

Minimal: identifier (required) + confirmation_token (optional — wired in Phase 241/242).

```rust
pub fn build_delete_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();
    let mut required_fields: Vec<String> = Vec::new();

    if let Some(id_field) = service.fields.iter()
        .find(|f| matches!(f.meaning, FieldMeaning::Identifier)) {
        let prop = data_type_to_json_schema(id_field.data_type);
        properties.insert(id_field.name.clone(), prop);
        required_fields.push(id_field.name.clone());
    }

    // confirmation_token — schema-only, execution wired in Phase 241
    properties.insert(
        "confirmation_token".to_string(),
        serde_json::json!({ "type": "string", "description": "Confirmation token from request_confirm_delete_<svc>" }),
    );

    Ok(serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required_fields,
    }))
}
```

[VERIFIED: D-08 in CONTEXT.md; pattern from render_confirm_tool in renderer.rs:304]

---

## Range Filter Extension

### `is_range_filter_field(field: &FieldDef) -> bool`

A separate predicate in `schema.rs`. Governs which fields get `__gt/__gte/__lt/__lte` params. Does NOT affect `__ne` and `__in` (those derive for every `is_filter_field` field).

```rust
pub fn is_range_filter_field(field: &FieldDef) -> bool {
    // Must first pass the base filter eligibility (readable, not list, not sensitive,
    // not Json/Binary). Re-use is_filter_field? No — is_filter_field gate 5 (meaning
    // allowlist) excludes Money/Quantity/Percentage. Range ops SHOULD work on those
    // (they are numeric). So is_range_filter_field has its own gate-5 replacement.
    if !field.readable {
        return false;
    }
    if field.is_list {
        return false;
    }
    if matches!(field.meaning, FieldMeaning::Sensitive) {
        return false;
    }
    if matches!(field.data_type, DataType::Json | DataType::Binary) {
        return false;
    }
    // Range ops only make sense on ordered types
    matches!(
        field.data_type,
        DataType::Integer | DataType::Float | DataType::DateTime | DataType::Date
    )
}
```

[VERIFIED: D-10 in CONTEXT.md — numeric (Integer/Float) and date/time (DateTime/Date) only; Money/Quantity/Percentage are NOT in `is_filter_field`'s allowlist but ARE numeric and should get range ops]

### Extended `build_input_schema`

The existing function adds `limit` and `offset` and equality params. Extend it to also emit:

1. For each `is_filter_field` field: `__ne` (same scalar type) and `__in` (array of same scalar type).
2. For each `is_range_filter_field` field: `__gt`, `__gte`, `__lt`, `__lte` (same scalar type as the field).
3. A `sort` optional string param.

Order of properties in the schema (for readability): equality params first, then range/ne/in per field, then `sort` at the end, then `limit`/`offset`.

The `__in` JSON Schema type fragment:

```rust
// For a field of data_type dt:
let scalar_type = data_type_to_json_schema(dt);
serde_json::json!({
    "type": "array",
    "items": scalar_type,
    "description": format!("Filter by {} (any of)", field.name),
})
```

The `sort` param:

```rust
properties.insert(
    "sort".to_string(),
    serde_json::json!({
        "type": "string",
        "description": "Sort field. Prefix with '-' for descending (e.g. 'created_at', '-total')",
    }),
);
```

[VERIFIED: D-09, D-10, D-11 in CONTEXT.md]

---

## Renderer Extension

### `render_exposed_tools` (renderer.rs:69)

After the existing `ActionDef` write-tool loop, add CRUD verb emission:

```rust
// CRUD verb emission (Phase 240 — schema only, execution in Phase 241)
if service.creatable {
    if let Some(tool) = render_create_tool(service)? {
        tagged.push((service.name.clone(), tool));
    }
}
if service.updatable {
    if let Some(tool) = render_update_tool(service)? {
        tagged.push((service.name.clone(), tool));
    }
}
if service.deletable {
    if let Some(tool) = render_delete_tool(service)? {
        tagged.push((service.name.clone(), tool));
    }
}
```

Helper functions follow the `render_action_tool` pattern:
- `render_create_tool`: `readOnly=false`, `destructive=false`, name = `create_<svc>`
- `render_update_tool`: `readOnly=false`, `destructive=false`, name = `update_<svc>`
- `render_delete_tool`: `readOnly=false`, `destructive=true`, name = `delete_<svc>`

**Naming collision note:** `create_/update_/delete_<svc>` all start with a verb prefix, not `list_`. The existing `disambiguate_write_tool_collisions` works on bare `ActionDef` names. CRUD verb names include the service name (`create_order`), so they are inherently unique across services — no collision risk. But the disambiguation pass must skip them (they start with `create_`/`update_`/`delete_`, not `list_`). Verify the existing check (`!tool.name.starts_with("list_")`) would attempt to disambiguate them if two services both had `create_order` — but that can't happen because the name includes the service name. No code change needed here.

[VERIFIED: reading renderer.rs lines 69-138]

---

## Dispatch Extension

### Op-Key Splitting

The current equality loop (dispatch.rs:128-147) iterates `filters.as_object()` and validates each key against `is_filter_field`. This must be EXTENDED, not replaced.

Approach: after the equality loop processes a key, check if it contains `__`. A key that does NOT contain `__` is an equality key (existing path). A key that contains `__` is an op key (new path).

Split on the **last** `__` (not first — field names could theoretically contain `__`):

```rust
fn split_op_key(key: &str) -> Option<(&str, &str)> {
    let pos = key.rfind("__")?;
    Some((&key[..pos], &key[pos + 2..]))
}
```

Op allowlist: `{gt, gte, lt, lte, ne, in}`.

SQL operator mapping:
| Op suffix | SQL operator |
|-----------|-------------|
| `gt`      | `>`         |
| `gte`     | `>=`        |
| `lt`      | `<`         |
| `lte`     | `<=`        |
| `ne`      | `!=`        |
| `in`      | `IN`        |

For `__in`: the value is a JSON array. Expand to `IN (?,?,?)` with one placeholder per element. Iterate the array, push each element as a bound value.

For `__ne`/`__gt` etc.: single bound value.

Field allowlist for the base:
- `__ne` and `__in`: validate base against `is_filter_field` (same allowlist as equality).
- `__gt/__gte/__lt/__lte`: validate base against `is_range_filter_field`.

Error path: unknown op suffix OR non-filterable base OR invalid value type → `Err(crate::Error::InvalidFilter(...))` with the same error message format as today.

### Sort Parsing

Extract `sort` from `filters` before the filter loop (remove it from `filters` like `limit`/`offset`):

```rust
let sort_param = if let Some(obj) = filters.as_object_mut() {
    obj.remove("sort").and_then(|v| v.as_str().map(|s| s.to_string()))
} else {
    None
};
```

Parse into `(field_name, direction)`:
- Starts with `-` → desc; strip `-` for base name
- Otherwise → asc

Validate the base field name against `is_filter_field` (reuse existing allowlist — filterable fields are sortable by the same logic). Unknown field → `Err(InvalidFilter(...))`.

Assemble sort clause:
```rust
let user_sort_str = match (&sort_param_parsed, &order_col) {
    (Some((col, dir)), Some(tiebreaker)) if col != tiebreaker => {
        format!(" ORDER BY \"{col}\" {dir}, \"{tiebreaker}\"")
    }
    (Some((col, dir)), _) => {
        format!(" ORDER BY \"{col}\" {dir}")
    }
    (None, Some(tiebreaker)) => {
        format!(" ORDER BY \"{tiebreaker}\"")  // existing deterministic sort
    }
    (None, None) => String::new(),
};
```

The `idx` counter for Postgres placeholders must be advanced correctly: equality params are processed first, then op params. `IN` arrays expand `idx` by the array length.

[VERIFIED: reading dispatch.rs lines 128-234; the existing `idx` management for LIMIT/OFFSET]

---

## Phase 241 Boundary: The "Not-Yet-Implemented" Envelope

**Critical constraint:** When `create_/update_/delete_<svc>` tools are called in Phase 240 (before Phase 241 wires execution), `handle_write_call` must NOT return a JSON-RPC `-32601` error (Method not found). That error breaks the Phase 205 regression guard and would surface to callers as a protocol-level error.

The correct response is a structured tool result (using `CallToolResult::structured`) with an `error_kind: "not_yet_implemented"` payload. This keeps the `content[]` envelope valid.

In `write_dispatch.rs::handle_write_call`, after the `ActionDef` resolution (which will now fail for CRUD verb names), add CRUD verb detection before `find_action`:

```rust
// Phase 240: CRUD verb tools are listed but not yet executable (Phase 241).
// Return a structured NTI result so the Phase 205 envelope guard stays green.
for prefix in &["create_", "update_", "delete_"] {
    if let Some(svc_name) = tool_name.strip_prefix(prefix) {
        // Verify the service exists and has the corresponding capability
        if services.iter().any(|s| s.mcp_exposed && s.name == svc_name) {
            let result = CallToolResult::structured(serde_json::json!({
                "error_kind": "not_yet_implemented",
                "message": format!("{tool_name} execution is not yet wired (Phase 241)")
            }));
            return json!({ "result": result });
        }
    }
}
```

This satisfies D-01 (schema-only this phase) while keeping the Phase 205 regression test green.

[VERIFIED: reading jsonrpc.rs:215 `tools_call_result_parses_as_valid_mcp_content` test; write_dispatch.rs routing logic]

---

## Common Pitfalls

### Pitfall 1: `__` Key Splitting — Split on LAST, Not FIRST

**What goes wrong:** `rfind("__")` vs `find("__")` produces different results for field names like `created__at` (unlikely but possible). If you split on the first `__`, `"total__gt"` splits correctly, but `"my__field__gt"` would split as `("my", "field__gt")` which fails the op check.
**Why it happens:** field names use `_` not `__`; but `rfind` is still the robust choice.
**How to avoid:** always use `rfind("__")` so the suffix is always the last segment.
**Warning signs:** split produces a base name containing `__` or an op string not in the allowlist.

### Pitfall 2: `__in` Value Must Be an Array — Reject Scalar

**What goes wrong:** agent passes `{"id__in": 5}` (scalar, not array); `val.as_array()` returns `None`; the code panics or silently produces no predicates.
**How to avoid:** when op is `in` and `val.as_array()` is `None`, return `Err(InvalidFilter("'__in' value must be an array"))`. When the array is empty, return `Err(InvalidFilter("'__in' array must not be empty"))` (empty IN clause is invalid SQL on some backends).
**Warning signs:** SQL syntax error from empty `IN ()`.

### Pitfall 3: Postgres Placeholder Index Drift from IN Expansion

**What goes wrong:** `__in` with 3 elements pushes 3 values and advances `idx` by 3, but LIMIT/OFFSET still use `idx` from before the array expansion.
**Why it happens:** `IN (?,?,?)` on SQLite uses `?`, which doesn't need an index. On Postgres it must be `($1,$2,$3)`. The existing `idx` management (lines 213-220) must track the total number of bound values correctly.
**How to avoid:** after processing all filters (equality + op), the final `idx` is the count of all bound values + 1. LIMIT uses `placeholder(backend, idx)` and OFFSET uses `placeholder(backend, idx+1)` — this is already what the code does at line 214. Verify that `idx` is correct after IN expansion.
**Warning signs:** Postgres errors like "incorrect number of parameters" or "parameter $5 not defined".

### Pitfall 4: `sort` Key Left in `filters` Object

**What goes wrong:** if `sort` is not removed from `filters` before the equality-key loop, the loop sees `sort` as a filter key, fails the `is_filter_field` check (there is no field named `sort`), and returns `Err(InvalidFilter("unknown or non-filterable filter field: sort"))`.
**How to avoid:** remove `sort`, `limit`, and `offset` from `filters` before the loop. The existing code removes `limit`/`offset` at lines 127-130 in `handle_tools_call`. In `dispatch`, add `sort` removal at the top (before the filter loop), mirroring limit/offset removal.
**Warning signs:** `InvalidFilter: unknown or non-filterable filter field: sort` errors in tests.

### Pitfall 5: `create_/update_/delete_` Tool Disambiguation

**What goes wrong:** `disambiguate_write_tool_collisions` iterates all non-`list_` tools. If two services both opted into CRUD, it would see `create_order` and `create_invoice` as separate names (no collision, both unique). But if it encounters two tools with the same name across services (can't happen with CRUD verbs since the name includes the service name), it would rename them. Confirm no false collision.
**How to avoid:** CRUD verb names (`create_order`, `update_order`, `delete_order`) embed the service name, making them globally unique. The disambiguation pass is safe but will not trigger for these names. No code change needed — just verify the assertion holds.
**Warning signs:** None expected; verify in tests that CRUD tools are NOT renamed.

### Pitfall 6: Phase 205 Regression Guard Must Parse New Verb Results

**What goes wrong:** if `create_/update_/delete_` tools return a bare `{"error": ...}` JSON-RPC error instead of a `{"result": CallToolResult::structured(...)}`, the Phase 205 test `tools_call_result_parses_as_valid_mcp_content` does not exercise them (it only calls `list_order`). A separate Phase 240 test must call a CRUD verb and assert the result parses as `CallToolResult`.
**Why it happens:** the test coverage gap — existing test only tests `list_`.
**How to avoid:** add a test in `jsonrpc.rs` that calls `create_order` and asserts `response["result"]` deserializes as `CallToolResult` with `is_error: Some(false)`.
**Warning signs:** integration failures in Phase 243 when the visual surface calls these tools and gets protocol-level errors.

### Pitfall 7: `is_write_excluded_field` Excludes the Identifier for Create but NOT for Update

**What goes wrong:** the shared predicate excludes Identifier (via `is_server_injected_field`). For update, the identifier must be re-injected as a required field AFTER the exclusion filter runs.
**Why it happens:** both create and update call `is_write_excluded_field`, which always excludes Identifier. The update builder must separately inject the identifier before iterating over data fields.
**How to avoid:** `build_update_input_schema` does what `build_action_input_schema` does: find the Identifier field explicitly and add it first, THEN iterate remaining fields through `is_write_excluded_field`.
**Warning signs:** update schema with no `id` field in `required`; or create schema that includes `id`.

---

## Code Examples

### Verified Pattern: `is_server_injected_field` (Phase 239, service.rs:236)

```rust
// Source: ferro-projections/src/service.rs (verified in this session)
pub fn is_server_injected_field(&self, field: &FieldDef) -> bool {
    matches!(
        field.meaning,
        FieldMeaning::Identifier | FieldMeaning::CreatedAt
    ) || self
        .tenant_column
        .as_deref()
        .map(|tc| tc == field.name)
        .unwrap_or(false)
}
```

### Verified Pattern: `build_action_input_schema` (schema.rs:111)

```rust
// Source: ferro-mcp-server/src/schema.rs (verified in this session)
// This is the structural template for build_create/update/delete_input_schema.
pub fn build_action_input_schema(action: &ActionDef, service: &ServiceDef) -> crate::Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();
    let mut required_fields: Vec<String> = Vec::new();

    // Inject identifier first (required)
    if let Some(id_field) = service.fields.iter()
        .find(|f| matches!(f.meaning, FieldMeaning::Identifier)) {
        let mut prop = data_type_to_json_schema(id_field.data_type);
        // ... add description ...
        properties.insert(id_field.name.clone(), prop);
        required_fields.push(id_field.name.clone());
    }

    // Map inputs; exclude Sensitive
    for input in &action.inputs {
        if matches!(input.meaning, FieldMeaning::Sensitive) { continue; }
        // ...
    }

    Ok(serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required_fields,
    }))
}
```

### Verified Pattern: `dispatch` filter loop and tenant predicate (dispatch.rs:128-166)

```rust
// Source: ferro-mcp-server/src/dispatch.rs (verified in this session)
if let Some(obj) = filters.as_object() {
    for (key, val) in obj {
        match service.fields.iter().find(|f| &f.name == key) {
            Some(field) if is_filter_field(field) => {}
            _ => { return Err(crate::Error::InvalidFilter(...)); }
        }
        where_clauses.push(format!("\"{}\" = {}", key, placeholder(backend, idx)));
        values.push(json_to_sea_value(val));
        idx += 1;
    }
}

// Tenant predicate (always-on)
if let Some(ref col) = service.tenant_column {
    match tenant_id {
        Some(tid) => {
            where_clauses.push(format!("\"{}\" = {}", col, placeholder(backend, idx)));
            values.push(sea_orm::Value::BigInt(Some(tid)));
            idx += 1;
        }
        None => { return Err(...); }
    }
}
```

### Verified Pattern: `CallToolResult::structured` envelope (write_dispatch.rs:206-213)

```rust
// Source: ferro-mcp-server/src/write_dispatch.rs (verified in this session)
Ok(result) => {
    let payload = json!({ "status": "ok", "action": action.name, "result": result });
    let tool_result = CallToolResult::structured(payload);
    json!({ "result": tool_result })
}
```

### New Pattern: `__in` SQL expansion (conceptual)

```rust
// For op == "in":
let arr = val.as_array().ok_or_else(|| crate::Error::InvalidFilter(
    format!("'__in' value for '{}' must be an array", base_field)
))?;
if arr.is_empty() {
    return Err(crate::Error::InvalidFilter(
        format!("'__in' array for '{}' must not be empty", base_field)
    ));
}
let placeholders: Vec<String> = arr.iter().map(|_| {
    let ph = placeholder(backend, idx);
    idx += 1;
    ph
}).collect();
where_clauses.push(format!("\"{}\" IN ({})", base_field, placeholders.join(", ")));
for item in arr {
    values.push(json_to_sea_value(item));
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hardcoded table name at dispatch.rs:122 | `service.resolved_table()` | Phase 239 | Remove TODO; dispatch uses official resolver |
| No soft-delete filtering | `WHERE deleted_at IS NULL` injected in dispatch | Phase 239 | All reads are soft-delete-aware by construction |
| Equality-only `list_` filters | + range/comparison + sort | Phase 240 | Richer query surface derived for free |
| No write tools for CRUD | `create_/update_/delete_` schema emission | Phase 240 | Agent surface complete (execution Phase 241) |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `sort` should be validated against `is_filter_field` (not a broader allowlist) | §Dispatch Extension | If a non-filterable field (EntityName, Money, etc.) should be sortable, the planner needs to introduce a separate sortability predicate. Low risk — filtering implies indexability implies sortability is a safe approximation. |
| A2 | Empty `__in` array is rejected with `InvalidFilter` (not silently producing 0 rows via `WHERE id IN ()`) | §Pitfall 2 | Some SQL backends accept `IN ()` as always-false; others reject it. Rejecting at the application layer is the safe, consistent choice. |
| A3 | CRUD verb tools (`create_<svc>` etc.) are NOT passed through the disambiguation pass (naming is globally unique by construction) | §Renderer Extension | If the disambiguation pass causes accidental renaming, tool routing in write_dispatch.rs breaks. Verify with a test. |

**If this table is empty:** it is not — three assumptions are flagged for the planner's awareness.

---

## Open Questions

1. **Conditional `confirmation` feature gate on `delete_` tool emission**
   - What we know: existing destructive ActionDef tools are NOT gated behind `#[cfg(feature = "confirmation")]` for emission — they always appear in `tools/list`. The `request_confirm_` / `confirm_` synthesis IS feature-gated.
   - What's unclear: should `delete_<svc>` emission be gated on `confirmation` feature?
   - Recommendation: Do NOT gate tool emission. The `delete_<svc>` schema includes a `confirmation_token` field regardless; the enforcement is in Phase 241/242. Consistent with how `submit_order` (destructive ActionDef) is always emitted but the confirm-flow tools are feature-gated. The planner decides.

2. **`sort` validation — `is_filter_field` vs broader allowlist**
   - What we know: D-11 says "allowlisted against the projection's filterable/sortable fields (reuse the dispatch filter-key allowlist validation)". `is_filter_field` excludes EntityName, Money, Quantity, Percentage.
   - What's unclear: should agents be able to sort by `total` (Money field) even if they can't filter by it for equality?
   - Recommendation: Start with `is_filter_field` as the sort allowlist (matches D-11 literally). If Phase 243 e2e reveals a need to sort Money fields, extend then. YAGNI.

---

## Environment Availability

Step 2.6: SKIPPED — all changes are in-workspace Rust code with no new external dependencies. SQLite in-memory (already in dev-dependencies) and `sea-orm` (already a workspace dep) are the only runtime needs for tests; both are available.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` / `#[tokio::test]` (no external test runner) |
| Config file | `Cargo.toml` (workspace) — `[dev-dependencies]` in `ferro-mcp-server` include `tokio` and `async-trait` |
| Quick run command | `cargo test -p ferro-mcp-server -p ferro-projections -- schema dispatch renderer` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CRUD-01 | `create_<svc>` tool appears in `render_exposed_tools` when `creatable=true` | unit | `cargo test -p ferro-mcp-server test_crud_tool_listing` | ❌ Wave 0 |
| CRUD-01 | `create_<svc>` schema excludes Identifier, CreatedAt, tenant, Sensitive, list fields | unit | `cargo test -p ferro-mcp-server test_create_schema_exclusions` | ❌ Wave 0 |
| CRUD-01 | `create_<svc>` schema excludes Status when SM present; includes Status when no SM | unit | `cargo test -p ferro-mcp-server test_create_schema_status_sm` | ❌ Wave 0 |
| CRUD-01 | `is_write_excluded_field` predicate correctness (table test) | unit | `cargo test -p ferro-projections test_write_excluded_field` | ❌ Wave 0 |
| CRUD-02 | `update_<svc>` schema: identifier required, data fields optional | unit | `cargo test -p ferro-mcp-server test_update_schema_patch_semantics` | ❌ Wave 0 |
| CRUD-02 | `update_<svc>` schema excludes Status under SM (same as create) | unit | `cargo test -p ferro-mcp-server test_update_schema_status_sm` | ❌ Wave 0 |
| CRUD-04 | `build_input_schema` emits `__gt/__gte/__lt/__lte` for Integer/Float/DateTime/Date fields | unit | `cargo test -p ferro-mcp-server test_range_params_in_schema` | ❌ Wave 0 |
| CRUD-04 | `build_input_schema` emits `__ne/__in` for all `is_filter_field` fields | unit | `cargo test -p ferro-mcp-server test_ne_in_params_in_schema` | ❌ Wave 0 |
| CRUD-04 | `build_input_schema` emits `sort` param | unit | `cargo test -p ferro-mcp-server test_sort_param_in_schema` | ❌ Wave 0 |
| CRUD-04 | `dispatch` range filters (`__gt`, `__lte`, etc.) return correct rows (SQLite in-memory) | integration | `cargo test -p ferro-mcp-server range_filter_returns_correct_rows` | ❌ Wave 0 |
| CRUD-04 | `dispatch` `__in` array filter returns correct rows | integration | `cargo test -p ferro-mcp-server in_filter_returns_correct_rows` | ❌ Wave 0 |
| CRUD-04 | `dispatch` `sort=field` / `sort=-field` orders rows correctly | integration | `cargo test -p ferro-mcp-server sort_orders_rows` | ❌ Wave 0 |
| CRUD-04 | Back-compat: existing equality filters produce identical results after extension | integration | `cargo test -p ferro-mcp-server equality_filter_backcompat` | ❌ Wave 0 |
| Phase 205 guard | `create_/update_/delete_` tool calls return valid `CallToolResult` (not JSON-RPC errors) | integration | `cargo test -p ferro-mcp-server crud_tool_call_parses_as_valid_mcp_content` | ❌ Wave 0 |
| CRUD-01 | `create_<svc>` tool NOT emitted when `creatable=false` | unit | (extends `test_crud_tool_listing`) | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-mcp-server -p ferro-projections`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full workspace gate green (`cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`) before `/gsd-verify-work`

### Wave 0 Gaps

All test functions listed above are new. Existing test infrastructure (in `schema.rs`, `renderer.rs`, `dispatch.rs`, `jsonrpc.rs`, `service.rs`) provides the fixture patterns to follow; no new test files or framework config is needed.

- [ ] Table tests in `ferro-projections/src/service.rs` — `is_write_excluded_field` predicate
- [ ] Table tests in `ferro-mcp-server/src/schema.rs` — `build_create/update/delete_input_schema`, `is_range_filter_field`, extended `build_input_schema`
- [ ] Table tests in `ferro-mcp-server/src/renderer.rs` — CRUD tool emission
- [ ] SQLite in-memory integration tests in `ferro-mcp-server/src/dispatch.rs` — range/sort/in
- [ ] Phase 205 guard extension in `ferro-mcp-server/src/jsonrpc.rs` — CRUD verb call returns valid `CallToolResult`

*(No new conftest or framework config needed — test idioms follow established patterns in each file.)*

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — (auth is Phase 242) |
| V3 Session Management | no | — |
| V4 Access Control | partial | Schema-level only: write-excluded fields must not appear in `inputSchema`; execution-level authz is Phase 242 |
| V5 Input Validation | yes | Filter key allowlist (existing); op suffix allowlist (new); `__in` array type check; sort field allowlist |
| V6 Cryptography | no | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Filter key injection (filter on Sensitive field via `__ne`) | Information Disclosure | Base field validated against allowlist before SQL assembly; same discipline as equality filters |
| SQL injection via op suffix | Tampering | Op suffix mapped to a fixed string constant (`>`, `>=`, etc.); never interpolated directly from user input |
| SQL injection via `__in` array element | Tampering | Each array element bound via `json_to_sea_value` + parameterized query — no interpolation |
| Schema exposure via `create_` tool call (Phase 241 boundary) | Information Disclosure | NTI envelope does not reveal table structure or column names; returns `error_kind: not_yet_implemented` only |
| Bypassing write exclusions in schema via `sort` | Information Disclosure | Sort field validated against `is_filter_field` only (no new surface exposure) |

---

## Sources

### Primary (HIGH confidence)
- `ferro-mcp-server/src/schema.rs` — verified in this session; `is_filter_field`, `build_action_input_schema`, `data_type_to_json_schema` exact shapes
- `ferro-mcp-server/src/renderer.rs` — verified in this session; `render_exposed_tools`, confirmation tool pattern
- `ferro-mcp-server/src/dispatch.rs` — verified in this session; full filter loop, tenant predicate, soft-delete predicate, ORDER BY, LIMIT/OFFSET with `idx`
- `ferro-mcp-server/src/write_dispatch.rs` — verified in this session; `handle_write_call` routing, `CallToolResult::structured` usage, NTI error shape
- `ferro-mcp-server/src/jsonrpc.rs` — verified in this session; Phase 205 regression guard at line 215
- `ferro-projections/src/service.rs` — verified in this session; `is_server_injected_field`, `ServiceDef` fields, `validate()`, `resolved_table()`
- `ferro-projections/src/field.rs` — verified in this session; `FieldMeaning` enum, `DataType` enum, `FieldDef` struct
- `240-CONTEXT.md` — all decisions D-01 through D-15 read verbatim

### Secondary (MEDIUM confidence)
- `docs/superpowers/specs/2026-06-23-projection-crud-data-surface-design.md` — anchor spec, sections "Derived tool surface", "Within-Track sequencing", "Non-goals"
- `239-CONTEXT.md` — Phase 239 substrate decisions D-07/D-11

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries verified by reading Cargo.toml and source
- Architecture: HIGH — all integration points verified by reading actual code
- Pitfalls: HIGH — derived from reading the exact code paths this phase modifies
- Test map: HIGH — derived from existing test idioms in each file

**Research date:** 2026-06-23
**Valid until:** 2026-07-23 (stable codebase; Phase 239 substrate is already committed)
