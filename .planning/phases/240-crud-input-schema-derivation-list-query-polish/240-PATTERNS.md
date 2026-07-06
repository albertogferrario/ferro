# Phase 240: CRUD Input-Schema Derivation + `list_` Query Polish - Pattern Map

**Mapped:** 2026-06-23
**Files analyzed:** 5 (all modifications to existing files; no new files)
**Analogs found:** 5 / 5

---

## File Classification

| Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------|------|-----------|----------------|---------------|
| `ferro-projections/src/service.rs` | model / predicate | transform | `is_server_injected_field` (~line 236) | exact — same struct, same gate-chain shape |
| `ferro-mcp-server/src/schema.rs` | utility / schema-builder | transform | `build_action_input_schema` (line 111) + `is_filter_field` (line 14) | exact — new builders mirror existing ones |
| `ferro-mcp-server/src/renderer.rs` | utility / tool-emitter | request-response | `render_action_tool` (line 179) + `render_confirm_tool` (line 279) | exact — same loop, same Tool construction |
| `ferro-mcp-server/src/dispatch.rs` | service / query-builder | CRUD (read) | existing filter loop (lines 128–148) + ORDER BY (lines 197–211) + LIMIT/OFFSET (lines 213–221) | exact — all additions are new call sites in the same function |
| `ferro-mcp-server/src/write_dispatch.rs` | service / write-router | request-response | prefix routing (lines 123–153) + NTI `CallToolResult::structured` envelope (lines 206–213) | exact — new CRUD verb detection inserts before `find_action` |

---

## Pattern Assignments

### `ferro-projections/src/service.rs` — add `is_write_excluded_field`

**Analog:** `is_server_injected_field` (lines 236–244)

**Existing predicate to compose with** (lines 236–244):
```rust
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

**New predicate to add immediately after** (gate order is load-bearing):
```rust
/// Returns `true` if a field must be excluded from write input schemas
/// (create and update). Composes `is_server_injected_field` and adds
/// UpdatedAt, Sensitive, and list-field exclusions.
///
/// `exclude_sm_status`: pass `self.state_machine.is_some()` from callers;
/// when true, a Status field is also excluded (SM controls it server-side).
pub fn is_write_excluded_field(&self, field: &FieldDef, exclude_sm_status: bool) -> bool {
    // Gate A: server-injected — Identifier, CreatedAt, tenant column
    if self.is_server_injected_field(field) {
        return true;
    }
    // Gate B: UpdatedAt — server-managed timestamp (D-05)
    if matches!(field.meaning, FieldMeaning::UpdatedAt) {
        return true;
    }
    // Gate C: Sensitive — never an agent write input (D-03)
    if matches!(field.meaning, FieldMeaning::Sensitive) {
        return true;
    }
    // Gate D: list fields — not useful as scalar write inputs (D-03)
    if field.is_list {
        return true;
    }
    // Gate E: Status under SM — set server-side to initial state (D-04/D-07)
    if exclude_sm_status && matches!(field.meaning, FieldMeaning::Status) {
        return true;
    }
    false
}
```

**Test pattern** (mirror existing `service_def_builder_chain` table-test style at line 639):
```rust
#[test]
fn is_write_excluded_field_gates() {
    // table test: (field, sm_present, expected_excluded)
    let cases: &[(&str, DataType, FieldMeaning, bool, bool, bool)] = &[
        ("id",         DataType::Integer, FieldMeaning::Identifier, false, false, true),
        ("created_at", DataType::DateTime, FieldMeaning::CreatedAt, false, false, true),
        ("updated_at", DataType::DateTime, FieldMeaning::UpdatedAt, false, false, true),
        ("password",   DataType::String,  FieldMeaning::Sensitive,  false, false, true),
        // status excluded when SM present
        ("status",     DataType::String,  FieldMeaning::Status,     false, true,  true),
        // status included when no SM
        ("status",     DataType::String,  FieldMeaning::Status,     false, false, false),
        // ordinary writable field always included
        ("notes",      DataType::String,  FieldMeaning::FreeText,   false, false, false),
    ];
    for &(name, dt, ref meaning, is_list, sm_present, expected) in cases {
        let mut svc = ServiceDef::new("order").field(name, dt, meaning.clone());
        if sm_present {
            svc = svc.state_machine(/* minimal SM */);
        }
        // build a FieldDef matching the case and call is_write_excluded_field
        ...
    }
}
```

---

### `ferro-mcp-server/src/schema.rs` — add `is_range_filter_field`, extend `build_input_schema`, add `build_create/update/delete_input_schema`

**Analog 1: `is_filter_field`** (lines 14–38) — gate structure to mirror for `is_range_filter_field`:
```rust
pub fn is_filter_field(field: &FieldDef) -> bool {
    if !field.readable { return false; }      // gate 1
    if field.is_list   { return false; }      // gate 2
    if matches!(field.meaning, FieldMeaning::Sensitive) { return false; } // gate 3
    if matches!(field.data_type, DataType::Json | DataType::Binary) { return false; } // gate 4
    // gate 5: conservative meaning allowlist
    matches!(
        field.meaning,
        FieldMeaning::Identifier | FieldMeaning::ForeignKey | FieldMeaning::Status
            | FieldMeaning::Category | FieldMeaning::Boolean | FieldMeaning::Custom(_)
    )
}
```

**New `is_range_filter_field`** (place after `is_filter_field`; has its own gate-5 to allow Money/Quantity/Percentage — D-10):
```rust
/// Returns `true` if this field should get `__gt/__gte/__lt/__lte` range params.
///
/// Gate order:
/// 1. Must be readable.
/// 2. Must not be a list.
/// 3. Must not carry `Sensitive` meaning.
/// 4. DataType must not be `Json` or `Binary`.
/// 5. DataType must be ordered/comparable: Integer, Float, DateTime, or Date.
///
/// Note: gate 5 is DataType-based, NOT meaning-based — Money/Quantity/Percentage
/// fields pass even though they are excluded by `is_filter_field`'s meaning gate.
pub fn is_range_filter_field(field: &FieldDef) -> bool {
    if !field.readable { return false; }
    if field.is_list   { return false; }
    if matches!(field.meaning, FieldMeaning::Sensitive) { return false; }
    if matches!(field.data_type, DataType::Json | DataType::Binary) { return false; }
    matches!(
        field.data_type,
        DataType::Integer | DataType::Float | DataType::DateTime | DataType::Date
    )
}
```

**Analog 2: `data_type_to_json_schema`** (lines 44–55) — reused as-is by all builders:
```rust
pub(crate) fn data_type_to_json_schema(dt: DataType) -> serde_json::Value {
    match dt {
        DataType::Integer  => serde_json::json!({ "type": "integer" }),
        DataType::Float    => serde_json::json!({ "type": "number" }),
        DataType::Boolean  => serde_json::json!({ "type": "boolean" }),
        DataType::DateTime => serde_json::json!({ "type": "string", "format": "date-time" }),
        DataType::Date     => serde_json::json!({ "type": "string", "format": "date" }),
        DataType::Uuid     => serde_json::json!({ "type": "string", "format": "uuid" }),
        _                  => serde_json::json!({ "type": "string" }),
    }
}
```

**Analog 3: `build_input_schema`** (lines 67–102) — extend this function to add range/sort params after the existing equality loop:

Existing structure (do not alter `limit`/`offset` insertion or the equality filter loop):
```rust
pub fn build_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();

    // limit and offset — KEEP UNCHANGED (D-02)
    properties.insert("limit".into(), serde_json::json!({
        "type": "integer", "description": "Maximum number of records to return",
        "default": 25, "maximum": 100, "minimum": 1
    }));
    properties.insert("offset".into(), serde_json::json!({
        "type": "integer", "description": "Number of records to skip",
        "default": 0, "minimum": 0
    }));

    // Equality filter params — KEEP UNCHANGED
    for field in service.fields.iter().filter(|f| is_filter_field(f)) {
        let mut prop = data_type_to_json_schema(field.data_type);
        if let serde_json::Value::Object(ref mut m) = prop {
            m.insert("description".into(),
                serde_json::Value::String(format!("Filter by {}", field.name)));
        }
        properties.insert(field.name.clone(), prop);
    }

    // === NEW: range/comparison filter params ===
    // __ne and __in for every is_filter_field field (D-09, D-10)
    for field in service.fields.iter().filter(|f| is_filter_field(f)) {
        let scalar = data_type_to_json_schema(field.data_type);
        // __ne: same scalar type
        let mut ne_prop = scalar.clone();
        if let serde_json::Value::Object(ref mut m) = ne_prop {
            m.insert("description".into(),
                serde_json::Value::String(format!("Filter by {} (not equal)", field.name)));
        }
        properties.insert(format!("{}__{}", field.name, "ne"), ne_prop);
        // __in: array of same scalar type
        properties.insert(format!("{}__{}", field.name, "in"), serde_json::json!({
            "type": "array",
            "items": scalar,
            "description": format!("Filter by {} (any of)", field.name),
        }));
    }
    // __gt/__gte/__lt/__lte for is_range_filter_field fields (D-10)
    for field in service.fields.iter().filter(|f| is_range_filter_field(f)) {
        let scalar = data_type_to_json_schema(field.data_type);
        for op in &["gt", "gte", "lt", "lte"] {
            let mut prop = scalar.clone();
            if let serde_json::Value::Object(ref mut m) = prop {
                m.insert("description".into(),
                    serde_json::Value::String(
                        format!("Filter by {} ({})", field.name, op)));
            }
            properties.insert(format!("{}__{}", field.name, op), prop);
        }
    }
    // sort param (D-11)
    properties.insert("sort".into(), serde_json::json!({
        "type": "string",
        "description": "Sort field. Prefix with '-' for descending (e.g. 'created_at', '-total')",
    }));

    Ok(serde_json::json!({ "type": "object", "properties": properties }))
}
```

**Analog 4: `build_action_input_schema`** (lines 111–163) — structural template for the three new builders. The entire function shape (identifier injection → required_fields push → data field loop → json! return) is copied for create/update/delete:

```rust
// Exact lines 111-163 — the template every new builder mirrors:
pub fn build_action_input_schema(
    action: &ActionDef,
    service: &ServiceDef,
) -> crate::Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();
    let mut required_fields: Vec<String> = Vec::new();

    // Inject identifier (required) — lines 118-136
    if let Some(id_field) = service.fields.iter()
        .find(|f| matches!(f.meaning, FieldMeaning::Identifier))
    {
        let mut prop = data_type_to_json_schema(id_field.data_type);
        if let serde_json::Value::Object(ref mut m) = prop {
            m.insert("description".into(),
                serde_json::Value::String(format!(
                    "ID of the {} record to act on",
                    service.display_name.as_deref().unwrap_or(&service.name)
                )));
        }
        properties.insert(id_field.name.clone(), prop);
        required_fields.push(id_field.name.clone());
    }

    // Map inputs; exclude Sensitive — lines 138-156
    for input in &action.inputs {
        if matches!(input.meaning, FieldMeaning::Sensitive) { continue; }
        let mut prop = data_type_to_json_schema(input.data_type);
        if let serde_json::Value::Object(ref mut m) = prop {
            if let Some(ref desc) = input.description {
                m.insert("description".into(), serde_json::Value::String(desc.clone()));
            }
        }
        properties.insert(input.name.clone(), prop);
        if input.required { required_fields.push(input.name.clone()); }
    }

    Ok(serde_json::json!({           // lines 158-163
        "type": "object",
        "properties": properties,
        "required": required_fields,
    }))
}
```

**New `build_create_input_schema`** (mirrors the template above, using `is_write_excluded_field` instead of `is Sensitive` check):
```rust
pub fn build_create_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();
    let mut required_fields: Vec<String> = Vec::new();
    let exclude_sm_status = service.state_machine.is_some();

    for field in &service.fields {
        if service.is_write_excluded_field(field, exclude_sm_status) { continue; }
        let mut prop = data_type_to_json_schema(field.data_type);
        if let serde_json::Value::Object(ref mut m) = prop {
            m.insert("description".into(),
                serde_json::Value::String(format!("{}", field.name)));
        }
        properties.insert(field.name.clone(), prop);
        if field.required { required_fields.push(field.name.clone()); }
    }

    Ok(serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required_fields,
    }))
}
```

**New `build_update_input_schema`** (identifier injected first as required, data fields all optional — D-06; mirrors `build_action_input_schema` identifier injection exactly):
```rust
pub fn build_update_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();
    let mut required_fields: Vec<String> = Vec::new();

    // Inject identifier (required) — same as build_action_input_schema lines 118-136
    if let Some(id_field) = service.fields.iter()
        .find(|f| matches!(f.meaning, FieldMeaning::Identifier))
    {
        let mut prop = data_type_to_json_schema(id_field.data_type);
        if let serde_json::Value::Object(ref mut m) = prop {
            m.insert("description".into(),
                serde_json::Value::String(format!(
                    "ID of the {} record to update",
                    service.display_name.as_deref().unwrap_or(&service.name)
                )));
        }
        properties.insert(id_field.name.clone(), prop);
        required_fields.push(id_field.name.clone());
    }

    let exclude_sm_status = service.state_machine.is_some();
    // Data fields: same exclusion predicate as create; all optional (patch semantics D-06)
    for field in &service.fields {
        if service.is_write_excluded_field(field, exclude_sm_status) { continue; }
        let mut prop = data_type_to_json_schema(field.data_type);
        if let serde_json::Value::Object(ref mut m) = prop {
            m.insert("description".into(),
                serde_json::Value::String(format!("{}", field.name)));
        }
        properties.insert(field.name.clone(), prop);
        // NOT added to required_fields (patch semantics)
    }

    Ok(serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required_fields,
    }))
}
```

**New `build_delete_input_schema`** (identifier required + `confirmation_token` optional — D-08; mirrors `render_confirm_tool` minimal schema at renderer.rs lines 304–320):
```rust
pub fn build_delete_input_schema(service: &ServiceDef) -> crate::Result<serde_json::Value> {
    let mut properties = serde_json::Map::new();
    let mut required_fields: Vec<String> = Vec::new();

    if let Some(id_field) = service.fields.iter()
        .find(|f| matches!(f.meaning, FieldMeaning::Identifier))
    {
        let prop = data_type_to_json_schema(id_field.data_type);
        properties.insert(id_field.name.clone(), prop);
        required_fields.push(id_field.name.clone());
    }

    // Schema-only confirmation_token — execution and enforcement in Phase 241/242
    properties.insert(
        "confirmation_token".to_string(),
        serde_json::json!({
            "type": "string",
            "description": "Confirmation token from request_confirm_delete_<svc> (Phase 241)"
        }),
    );

    Ok(serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required_fields,
    }))
}
```

**Test pattern** (mirror existing `test_action_schema_*` table tests at lines 282–393):
```rust
// In schema.rs #[cfg(test)] mod tests — mirror test_action_schema_injects_identifier:
#[test]
fn test_create_schema_excludes_server_injected() { ... }
#[test]
fn test_create_schema_status_excluded_with_sm() { ... }
#[test]
fn test_create_schema_status_included_without_sm() { ... }
#[test]
fn test_update_schema_identifier_required_data_optional() { ... }
#[test]
fn test_range_params_emitted_for_numeric_datetime_fields() { ... }
#[test]
fn test_ne_in_params_emitted_for_filter_eligible_fields() { ... }
#[test]
fn test_sort_param_present() { ... }
```

---

### `ferro-mcp-server/src/renderer.rs` — extend `render_exposed_tools`

**Analog: `render_action_tool`** (lines 179–217) — the complete template for new CRUD verb helper functions:
```rust
fn render_action_tool(
    service: &ServiceDef,
    action: &ActionDef,
    ctx: &McpContext,
) -> std::result::Result<Option<Tool>, ProjError> {
    for precondition in &action.preconditions {
        if ctx.evaluated_guards.get(precondition) == Some(&false) {
            return Ok(None);
        }
    }

    let name = action.name.clone();
    let description = action.description.clone()
        .or_else(|| action.display_name.clone())
        .unwrap_or_else(|| format!("{} {}", action.name, service.name));

    let schema_value = crate::schema::build_action_input_schema(action, service)
        .map_err(|e| ProjError::Render(e.to_string()))?;
    let schema_map = match schema_value {
        serde_json::Value::Object(m) => m,
        _ => return Err(ProjError::Render("action inputSchema must be an object".into())),
    };

    // destructive_hint defaults to true when absent in rmcp — always set explicitly (D-04)
    let annotations = ToolAnnotations::new()
        .read_only(false)
        .destructive(action.transition_trigger.is_some());

    Ok(Some(Tool::new(name, description, Arc::new(schema_map)).annotate(annotations)))
}
```

**Analog: `render_confirm_tool`** (lines 279–327) — the `destructive(true)` + minimal-schema template for `render_delete_tool`:
```rust
// From lines 304-320: the minimal fixed-schema pattern for delete (identifier + token):
let mut schema = serde_json::Map::new();
schema.insert("type".to_string(), serde_json::json!("object"));
let mut props = serde_json::Map::new();
props.insert("confirmation_token".to_string(),
    serde_json::json!({ "type": "string", "description": "..." }));
props.insert("id".to_string(),
    serde_json::json!({ "type": "integer", "description": "..." }));
schema.insert("properties".to_string(), serde_json::Value::Object(props));
schema.insert("required".to_string(), serde_json::json!(["confirmation_token", "id"]));
let annotations = ToolAnnotations::new().read_only(false).destructive(true);
```

**Emission loop insertion point** in `render_exposed_tools` (after line 88, after the `for action in &service.actions` loop, before the disambiguation pass at line 93):
```rust
// After ActionDef loop (line 88-89), before disambiguation (line 93):
// CRUD verb emission — Phase 240: schema only; execution wired in Phase 241.
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

**Disambiguation pass compatibility:** `disambiguate_write_tool_collisions` (lines 144–171) skips tools starting with `list_` (line 148, 161). CRUD verb names (`create_order`, `update_order`, `delete_order`) include the service name and are globally unique; the disambiguation check will never collide them. No code change needed in that function. Verify in tests that CRUD tools are not renamed.

**New helper function signatures** (each follows `render_action_tool` shape):
- `fn render_create_tool(service: &ServiceDef) -> std::result::Result<Option<Tool>, ProjError>` — `read_only(false)`, `destructive(false)`, name = `create_{service.name}`, schema from `build_create_input_schema`
- `fn render_update_tool(service: &ServiceDef) -> std::result::Result<Option<Tool>, ProjError>` — `read_only(false)`, `destructive(false)`, name = `update_{service.name}`, schema from `build_update_input_schema`
- `fn render_delete_tool(service: &ServiceDef) -> std::result::Result<Option<Tool>, ProjError>` — `read_only(false)`, `destructive(true)`, name = `delete_{service.name}`, schema from `build_delete_input_schema`

**Test pattern** (mirror `test_one_write_tool_per_action` at line 485 and `test_mcp_exposed_filter` at line 387):
```rust
#[test]
fn test_crud_tools_emitted_when_flags_set() {
    // service with creatable+updatable+deletable=true and mcp_write_ability
    // assert create_order, update_order, delete_order all present
}
#[test]
fn test_crud_tools_not_emitted_when_flags_false() {
    // default service (creatable=false etc.) → no CRUD tools
}
#[test]
fn test_delete_tool_has_destructive_hint_true() { ... }
#[test]
fn test_create_update_tools_have_destructive_hint_false() { ... }
```

---

### `ferro-mcp-server/src/dispatch.rs` — extend for `__op` filter keys and `sort`

**Analog 1: equality filter loop** (lines 128–148) — the `__op` key loop runs in parallel / after this loop. Keep this loop byte-for-byte identical (back-compat D-02):
```rust
// Lines 128-148 — DO NOT MODIFY:
if let Some(obj) = filters.as_object() {
    for (key, val) in obj {
        match service.fields.iter().find(|f| &f.name == key) {
            Some(field) if is_filter_field(field) => {}
            _ => {
                return Err(crate::Error::InvalidFilter(format!(
                    "unknown or non-filterable filter field: {key}"
                )));
            }
        }
        where_clauses.push(format!("\"{}\" = {}", key, placeholder(backend, idx)));
        values.push(json_to_sea_value(val));
        idx += 1;
    }
}
```

**New `split_op_key` helper** (add as a module-level `fn`):
```rust
/// Split `"field__op"` on the LAST `__` separator.
///
/// Returns `Some(("field", "op"))` or `None` if no `__` present.
/// Using `rfind` (not `find`) so field names containing `__` split correctly.
fn split_op_key(key: &str) -> Option<(&str, &str)> {
    let pos = key.rfind("__")?;
    Some((&key[..pos], &key[pos + 2..]))
}
```

**New op-key loop** (replaces the single equality loop — restructure to handle equality vs op keys in the same `as_object()` iteration):

Strategy: iterate `filters.as_object()` once; for each key, first check if it splits as `base__op`. If yes → op path; if no → equality path. This keeps the single `mut idx` correctly sequenced:
```rust
if let Some(obj) = filters.as_object() {
    for (key, val) in obj {
        if let Some((base, op)) = split_op_key(key) {
            // Op path — validate base + op, then emit SQL
            let op_sql = match op {
                "gt"  => ">",
                "gte" => ">=",
                "lt"  => "<",
                "lte" => "<=",
                "ne"  => "!=",
                "in"  => "IN",
                _     => return Err(crate::Error::InvalidFilter(
                    format!("unknown op suffix '{}' in filter key '{}'", op, key))),
            };
            // Validate base field against appropriate allowlist (D-10/D-12)
            let base_field = match service.fields.iter().find(|f| f.name == base) {
                Some(f) if matches!(op, "gt"|"gte"|"lt"|"lte") && is_range_filter_field(f) => f,
                Some(f) if matches!(op, "ne"|"in") && is_filter_field(f) => f,
                _ => return Err(crate::Error::InvalidFilter(
                    format!("unknown or non-filterable filter field: {key}"))),
            };
            if op == "in" {
                let arr = val.as_array().ok_or_else(|| crate::Error::InvalidFilter(
                    format!("'__in' value for '{}' must be an array", base)))?;
                if arr.is_empty() {
                    return Err(crate::Error::InvalidFilter(
                        format!("'__in' array for '{}' must not be empty", base)));
                }
                let placeholders: Vec<String> = arr.iter().map(|_| {
                    let ph = placeholder(backend, idx);
                    idx += 1;
                    ph
                }).collect();
                where_clauses.push(format!("\"{}\" IN ({})", base, placeholders.join(", ")));
                for item in arr { values.push(json_to_sea_value(item)); }
            } else {
                where_clauses.push(format!("\"{}\" {} {}", base, op_sql, placeholder(backend, idx)));
                values.push(json_to_sea_value(val));
                idx += 1;
            }
            let _ = base_field; // used for allowlist check above
        } else {
            // Equality path — lines 136-147 unchanged
            match service.fields.iter().find(|f| &f.name == key) {
                Some(field) if is_filter_field(field) => {}
                _ => return Err(crate::Error::InvalidFilter(
                    format!("unknown or non-filterable filter field: {key}"))),
            }
            where_clauses.push(format!("\"{}\" = {}", key, placeholder(backend, idx)));
            values.push(json_to_sea_value(val));
            idx += 1;
        }
    }
}
```

**Sort key extraction** — must happen BEFORE the filter loop (Pitfall 4). Mirror the `limit`/`offset` removal in `handle_tools_call` (jsonrpc.rs lines 127–130):
```rust
// Add at the top of dispatch(), before the filter loop, after clamping limit/offset:
let sort_param = if let Some(obj) = filters.as_object_mut() {
    obj.remove("sort").and_then(|v| {
        v.as_str().map(|s| s.to_string())
    })
} else {
    None
};
// Parse sort into (column, direction) — validated against is_filter_field allowlist:
let parsed_sort: Option<(String, &'static str)> = match sort_param.as_deref() {
    None => None,
    Some(s) => {
        let (col, dir) = if let Some(bare) = s.strip_prefix('-') {
            (bare, "DESC")
        } else {
            (s, "ASC")
        };
        // Validate col against filterable fields (D-11)
        match service.fields.iter().find(|f| f.name == col) {
            Some(f) if is_filter_field(f) => Some((col.to_string(), dir)),
            _ => return Err(crate::Error::InvalidFilter(
                format!("unknown or non-sortable field: {col}"))),
        }
    }
};
```

**ORDER BY assembly** — extend the existing `order_str` block (lines 197–211) to place user sort before the deterministic tiebreaker:
```rust
// Existing (lines 197-211):
let order_col = service.fields.iter()
    .find(|f| matches!(f.meaning, FieldMeaning::Identifier))
    .or_else(|| service.fields.first())
    .map(|f| f.name.clone());

// Replace order_str construction:
let order_str = match (&parsed_sort, &order_col) {
    (Some((col, dir)), Some(tiebreaker)) if col != tiebreaker => {
        format!(" ORDER BY \"{}\" {}, \"{}\"", col, dir, tiebreaker)
    }
    (Some((col, dir)), _) => {
        format!(" ORDER BY \"{}\" {}", col, dir)
    }
    (None, Some(tiebreaker)) => format!(" ORDER BY \"{}\"", tiebreaker), // unchanged
    (None, None) => String::new(),
};
```

**LIMIT/OFFSET** (lines 213–221) — unchanged; `idx` is already correct after the filter loop:
```rust
// Lines 213-221 — DO NOT MODIFY:
let limit_str = format!(
    " LIMIT {} OFFSET {}",
    placeholder(backend, idx),
    placeholder(backend, idx + 1)
);
values.push(sea_orm::Value::BigInt(Some(limit as i64)));
values.push(sea_orm::Value::BigInt(Some(offset as i64)));
```

**`is_range_filter_field` import** — add to the `use` block at line 1 alongside `is_filter_field`:
```rust
use crate::schema::{is_filter_field, is_range_filter_field};
```

**Test pattern** (mirror `setup_orders_db` + `#[tokio::test]` async style at dispatch.rs lines 244–442):
```rust
// Extend setup_orders_db() total column already present (REAL NOT NULL).
// New tests follow the exact same pattern:
#[tokio::test]
async fn range_filter_gt_returns_correct_rows() {
    let db = setup_orders_db().await;
    let service = order_service_no_tenant();
    let result = dispatch(&service, serde_json::json!({"total__gt": 150.0}), 10, 0, &db, None)
        .await.expect("dispatch ok");
    // Alice=100 excluded, Bob=200 + Dave=250 included (or Carol=150 also excluded by gt)
    assert_eq!(result.rows.len(), 2);
}
#[tokio::test]
async fn in_filter_returns_correct_rows() {
    let db = setup_orders_db().await;
    let service = order_service_no_tenant();
    let result = dispatch(
        &service,
        serde_json::json!({"status__in": ["pending"]}),
        10, 0, &db, None
    ).await.expect("dispatch ok");
    assert_eq!(result.rows.len(), 2); // Alice + Carol
}
#[tokio::test]
async fn sort_asc_orders_rows() { ... }
#[tokio::test]
async fn sort_desc_orders_rows() { ... }
#[tokio::test]
async fn equality_filter_backcompat_after_extension() { ... }
```

---

### `ferro-mcp-server/src/write_dispatch.rs` — add CRUD verb NTI routing

**Analog: confirmation prefix routing** (lines 123–153) — CRUD verb detection inserts BEFORE the `find_action` call at line 165, using the same prefix-strip pattern:
```rust
// Lines 123-153 — the prefix routing pattern to mirror:
#[cfg(feature = "confirmation")]
if let Some(action_name) = tool_name.strip_prefix("request_confirm_") { ... return ...; }
#[cfg(feature = "confirmation")]
if let Some(action_name) = tool_name.strip_prefix("confirm_") { ... return ...; }
```

**New CRUD verb detection** (add after confirmation prefix routing, before `find_action` at line 165):
```rust
// Phase 240: CRUD verb tools listed but not yet executable (Phase 241 wires execution).
// Return a structured NTI envelope so Phase 205 regression guard stays green (D-01).
for prefix in &["create_", "update_", "delete_"] {
    if let Some(svc_name) = tool_name.strip_prefix(prefix) {
        if services.iter().any(|s| s.mcp_exposed && s.name == svc_name) {
            let tool_result = CallToolResult::structured(serde_json::json!({
                "error_kind": "not_yet_implemented",
                "message": format!("{} execution is not yet wired (Phase 241)", tool_name)
            }));
            return json!({ "result": tool_result });
        }
    }
}
```

**Analog: `CallToolResult::structured` success envelope** (lines 206–213) — used verbatim for the NTI response:
```rust
// Lines 206-213 — the exact envelope shape:
Ok(result) => {
    let payload = json!({
        "status": "ok",
        "action": action.name,
        "result": result
    });
    let tool_result = CallToolResult::structured(payload);
    json!({ "result": tool_result })
}
```

**Phase 205 regression guard extension** in `ferro-mcp-server/src/jsonrpc.rs`:

The existing guard test (`tools_call_result_parses_as_valid_mcp_content` at line 215) only calls `list_order`. Add a new test that calls `create_order` and verifies the NTI envelope parses as `CallToolResult` with `is_error: Some(false)`:
```rust
// New test in jsonrpc.rs #[cfg(test)] mod — follows the exact pattern of
// tools_call_result_parses_as_valid_mcp_content (lines 215-273):
#[tokio::test]
async fn crud_tool_call_nti_parses_as_valid_mcp_content() {
    let db = setup_orders_db().await;
    // Service with creatable=true and mcp_write_ability declared
    let service = ServiceDef::new("order")
        .mcp_exposed(true)
        .creatable(true)
        .mcp_write_ability("write-orders")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("status", DataType::String, FieldMeaning::Status);
    let call_params = serde_json::json!({
        "name": "create_order",
        "arguments": { "status": "pending" }
    });
    let noop_dispatcher = crate::WriteDispatcher::new(
        Box::new(|_, _, _, _| Box::pin(async { Ok(serde_json::json!({})) })),
        Box::new(|_, _, _, _| Box::pin(async { Ok(true) })),
    );
    let response = handle_tools_call(
        call_params, &[service], &db, Some(1),
        &McpContext { scope: Some("read_write".to_string()), ..Default::default() },
        &noop_dispatcher,
        // ...cfg feature args...
    ).await;

    // Load-bearing: NTI response must parse as CallToolResult (not a -32601 error)
    let parsed: CallToolResult = serde_json::from_value(response["result"].clone())
        .expect("NTI result must parse as CallToolResult");
    assert_eq!(parsed.is_error, Some(false));
    let sc = parsed.structured_content.expect("structuredContent present");
    assert_eq!(sc["error_kind"].as_str(), Some("not_yet_implemented"));
}
```

---

## Shared Patterns

### `placeholder(backend, idx)` — backend-portable SQL placeholders

**Source:** `ferro-mcp-server/src/dispatch.rs` (lines 28–33)
**Apply to:** all new `where_clauses.push(...)` call sites in the `__op` loop and the sort clause

```rust
fn placeholder(backend: DatabaseBackend, index: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${index}"),
        _ => "?".to_string(),
    }
}
```

Key rule: `idx` must be incremented exactly once per bound scalar value, and by `arr.len()` for `__in` array expansions, before the `LIMIT/OFFSET` placeholders at `placeholder(backend, idx)` / `placeholder(backend, idx+1)`.

### `json_to_sea_value` — parameterized value binding

**Source:** `ferro-mcp-server/src/dispatch.rs` (lines 36–50)
**Apply to:** every new bound value in the `__op` loop (scalar and each element of `__in` arrays)

```rust
fn json_to_sea_value(val: &serde_json::Value) -> sea_orm::Value {
    match val {
        serde_json::Value::Null       => sea_orm::Value::String(None),
        serde_json::Value::Bool(b)    => sea_orm::Value::Bool(Some(*b)),
        serde_json::Value::Number(n)  => {
            if let Some(i) = n.as_i64() { sea_orm::Value::BigInt(Some(i)) }
            else { sea_orm::Value::Double(n.as_f64()) }
        }
        serde_json::Value::String(s)  => sea_orm::Value::String(Some(Box::new(s.clone()))),
        other => sea_orm::Value::String(Some(Box::new(other.to_string()))),
    }
}
```

### `CallToolResult::structured` envelope

**Source:** `ferro-mcp-server/src/write_dispatch.rs` (lines 206–213) and `ferro-mcp-server/src/jsonrpc.rs` (lines 138–145)
**Apply to:** CRUD verb NTI responses in `write_dispatch.rs`; Phase 205 guard extension test in `jsonrpc.rs`

Shape: `json!({ "result": CallToolResult::structured(payload) })` where `payload` is a `serde_json::Value::Object`. The `structured()` constructor emits `content: [{ "type": "text", "text": ... }]`, `isError: false`, and `structuredContent: payload`. This is the only result constructor — no bare `content[]` arrays.

### SQLite in-memory test setup

**Source:** `ferro-mcp-server/src/dispatch.rs` (lines 244–279) — `setup_orders_db()` + `#[tokio::test]` pattern
**Apply to:** all new dispatch integration tests for range filters, sort, `__in`

The existing `setup_orders_db()` already seeds `total REAL NOT NULL` and `status TEXT NOT NULL` columns, making it directly usable for `__gt`/`__lte` (on `total`) and `__in` (on `status`) tests. Reference it from the new test functions rather than creating a separate fixture.

---

## No Analog Found

All files have close analogs. No files require falling back to RESEARCH.md patterns alone.

---

## Metadata

**Analog search scope:** `ferro-projections/src/`, `ferro-mcp-server/src/`
**Files scanned:** 6 source files read in full
**Key line number references for planner:**

| Symbol | File | Lines |
|--------|------|-------|
| `is_filter_field` | `ferro-mcp-server/src/schema.rs` | 14–38 |
| `data_type_to_json_schema` | `ferro-mcp-server/src/schema.rs` | 44–55 |
| `build_input_schema` | `ferro-mcp-server/src/schema.rs` | 67–102 |
| `build_action_input_schema` | `ferro-mcp-server/src/schema.rs` | 111–163 |
| `is_server_injected_field` | `ferro-projections/src/service.rs` | 236–244 |
| `render_exposed_tools` loop | `ferro-mcp-server/src/renderer.rs` | 77–89 |
| `render_action_tool` | `ferro-mcp-server/src/renderer.rs` | 179–217 |
| `render_confirm_tool` minimal schema | `ferro-mcp-server/src/renderer.rs` | 304–320 |
| `disambiguate_write_tool_collisions` | `ferro-mcp-server/src/renderer.rs` | 144–171 |
| equality filter loop | `ferro-mcp-server/src/dispatch.rs` | 128–148 |
| tenant predicate | `ferro-mcp-server/src/dispatch.rs` | 151–166 |
| soft-delete predicate | `ferro-mcp-server/src/dispatch.rs` | 168–178 |
| ORDER BY assembly | `ferro-mcp-server/src/dispatch.rs` | 197–211 |
| LIMIT/OFFSET | `ferro-mcp-server/src/dispatch.rs` | 213–221 |
| `handle_write_call` prefix routing | `ferro-mcp-server/src/write_dispatch.rs` | 123–153 |
| `find_action` call | `ferro-mcp-server/src/write_dispatch.rs` | 165–170 |
| `CallToolResult::structured` success | `ferro-mcp-server/src/write_dispatch.rs` | 206–213 |
| Phase 205 regression guard test | `ferro-mcp-server/src/jsonrpc.rs` | 215–273 |

**Pattern extraction date:** 2026-06-23
