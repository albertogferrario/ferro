# Phase 135: ServiceDef Derivation Bridge - Research

**Researched:** 2026-04-17
**Domain:** ferro-projections + ferro-mcp — ServiceDef construction from model metadata
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `ServiceDef::from_model()` takes an intermediate `ModelMetadata` struct, not a concrete SeaORM type. ferro-projections stays free of SeaORM dependencies.
- **D-02:** `ModelMetadata` contains: `name: String`, `display_name: Option<String>`, `table: Option<String>`, `fields: Vec<FieldMetadata>` where `FieldMetadata` has `name`, `column_type` (string), `is_primary_key`, `is_nullable`.
- **D-03:** ferro-mcp bridges by converting its parsed `ModelDetails` (from `list_models.rs` syn-based AST parsing) into `ModelMetadata`, then calls `ServiceDef::from_model()`.
- **D-04:** Add `DataType::from_column_type(type_str: &str) -> DataType` in ferro-projections `field.rs`. Pattern-matches common SeaORM/Rust types: `i32`/`i64`/`u32`/`u64` → Integer, `String` → String, `bool` → Boolean, `DateTime`/`chrono::` → DateTime, `f32`/`f64`/`Decimal` → Float, `Uuid` → Uuid, `Vec<u8>` → Binary, `serde_json::Value`/`Json` → Json.
- **D-05:** `infer_meaning()` (already exists) handles field name → `FieldMeaning` mapping. Combined with `DataType::from_column_type()`, a full `FieldDef` can be derived from name + type string + nullable flag.
- **D-06:** New `generate_projection` MCP tool returns serialized `ServiceDef` JSON, not Rust source code.
- **D-07:** Tool inputs: `model_name` (required). Finds the model via existing `list_models::execute()`, converts to `ModelMetadata`, calls `ServiceDef::from_model()`.
- **D-08:** Tool output includes: the `ServiceDef` JSON, the derived intent scores (via `derive_intents()`), and a note about what was inferred vs what needs manual enrichment.
- **D-09:** `from_model()` infers fields only. Actions, state machines, and explicit relationships are too domain-specific — they stay as manual builder additions.
- **D-10:** FK fields (detected by `_id` suffix via `infer_meaning()`) produce `FieldMeaning::ForeignKey` but do NOT auto-generate `RelationshipDef` entries.
- **D-11:** System fields (`id`, `created_at`, `updated_at`) are included in ServiceDef but marked with their semantic meanings so renderers can handle them appropriately.

### Claude's Discretion

- Whether `ModelMetadata` lives in a new `metadata.rs` module or alongside `ServiceDef` in `service.rs`
- Whether `from_model()` is an inherent method on `ServiceDef` or a standalone function
- Display name derivation heuristic (snake_case → Title Case, or just capitalize)
- Test structure for the round-trip demonstration

### Deferred Ideas (OUT OF SCOPE)

- Cross-model relationship inference (analyzing FK targets across models)
- Action inference from route handlers
- State machine inference from enum fields
- Crate consolidation audit (CONC-04 in v13.0)
</user_constraints>

---

## Summary

Phase 135 adds a derivation bridge that shortens the path from a SeaORM model to a rendered projection. The two deliverables are: (1) `ModelMetadata` + `ServiceDef::from_model()` in ferro-projections, and (2) a `generate_projection` MCP tool in ferro-mcp that wires model parsing to ServiceDef derivation.

All the raw materials already exist. `infer_meaning()` is in `field.rs` and covers 7 inference rules. `list_models::execute()` already parses SeaORM structs via syn AST and extracts `name`, `field_type` (string), `is_primary_key`, and `is_nullable` into `ModelDetails` / `FieldInfo`. `ServiceDef::from_model()` is the missing bridge function that chains these two together. The MCP tool then exposes this as an agent-callable operation.

The design is intentionally minimal: ferro-projections gets no new dependencies (ModelMetadata is a plain data struct), ferro-mcp calls `from_model()` after mapping its existing `FieldInfo` values, and the round-trip test uses an in-memory `ModelMetadata` — no file I/O required.

**Primary recommendation:** Implement `ModelMetadata` + `DataType::from_column_type()` + `ServiceDef::from_model()` as a single coherent addition to ferro-projections, then add the MCP tool as a thin bridge layer.

---

## Standard Stack

### Core (verified against codebase)

| Component | Location | Purpose |
|-----------|----------|---------|
| `ferro-projections::ServiceDef` | `ferro-projections/src/service.rs` | Target schema type; builder API already complete |
| `ferro-projections::field::infer_meaning` | `ferro-projections/src/field.rs:86` | Field name → FieldMeaning; reused directly in from_model() |
| `ferro-projections::DataType` | `ferro-projections/src/field.rs:10` | 10-variant enum; `from_column_type()` method to be added here |
| `ferro-projections::derive_intents` | `ferro-projections/src/derive.rs` | Called after from_model() to rank intents |
| `ferro-mcp::tools::list_models` | `ferro-mcp/src/tools/list_models.rs` | Provides `ModelDetails` + `FieldInfo`; feeds ModelMetadata |
| `ferro-mcp::service::FerroMcpService` | `ferro-mcp/src/service.rs` | Tool registration via `#[tool]` + `tool_router!` macros |
| `rmcp` | ferro-mcp Cargo.toml | MCP protocol primitives (`#[tool]`, `Parameters<T>`, `ToolRouter`) |
| `serde` / `schemars` | all crates | Serialization; every Params struct needs `JsonSchema` |

### No New Dependencies

ferro-projections must stay free of SeaORM. `ModelMetadata` is a plain Rust struct (`name: String`, `display_name: Option<String>`, `table: Option<String>`, `fields: Vec<FieldMetadata>`) — zero external deps.

---

## Architecture Patterns

### Recommended Module Layout

```
ferro-projections/src/
├── field.rs          — add DataType::from_column_type() here
├── service.rs        — add ModelMetadata, FieldMetadata, ServiceDef::from_model() here
                        (or split into metadata.rs — discretion decision)
└── lib.rs            — add ModelMetadata, FieldMetadata to pub use

ferro-mcp/src/tools/
├── generate_projection.rs   — new tool module
└── mod.rs                   — pub mod generate_projection;

ferro-mcp/src/
└── service.rs               — GenerateProjectionParams struct + #[tool] handler
```

### Pattern 1: DataType::from_column_type()

Add as an inherent method on `DataType`. Input is the raw type string from `FieldInfo::field_type` (already cleaned: `type_to_string()` removes spaces). The string may be wrapped in `Option<...>` — strip the wrapper before matching.

```rust
// In ferro-projections/src/field.rs
impl DataType {
    /// Infers a DataType from a Rust/SeaORM column type string.
    ///
    /// Strips Option<> wrappers before matching. Falls back to String.
    pub fn from_column_type(type_str: &str) -> Self {
        // Strip Option<...> wrapper
        let inner = if let Some(stripped) = type_str
            .strip_prefix("Option<")
            .and_then(|s| s.strip_suffix('>'))
        {
            stripped
        } else {
            type_str
        };

        match inner {
            "i32" | "i64" | "u32" | "u64" | "i8" | "i16" | "u8" | "u16" => Self::Integer,
            "f32" | "f64" => Self::Float,
            "bool" => Self::Boolean,
            "Uuid" | "uuid::Uuid" => Self::Uuid,
            s if s.contains("Decimal") => Self::Float,
            s if s.starts_with("DateTime") || s.contains("chrono::") => Self::DateTime,
            s if s.starts_with("NaiveDate") => Self::Date,
            "Vec<u8>" => Self::Binary,
            s if s.contains("Json") || s.contains("serde_json") => Self::Json,
            // String and all unknown types → String
            _ => Self::String,
        }
    }
}
```

**Confidence:** HIGH — type strings verified against `list_models.rs` `type_to_string()` output (strips spaces via `.replace(' ', "")`).

### Pattern 2: ModelMetadata and ServiceDef::from_model()

`ModelMetadata` is a plain data struct. `from_model()` is an inherent method on `ServiceDef` that chains `DataType::from_column_type()` + `infer_meaning()` + `is_nullable` flag to build `FieldDef`s, then uses system field detection to mark `id`, `created_at`, `updated_at` as read-only.

```rust
// In ferro-projections/src/service.rs (or metadata.rs)

/// Intermediate representation of a SeaORM model for ServiceDef derivation.
///
/// Decouples ferro-projections from SeaORM — callers (ferro-mcp) populate this
/// from their own model parsing and call ServiceDef::from_model().
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub display_name: Option<String>,
    pub table: Option<String>,
    pub fields: Vec<FieldMetadata>,
}

/// Metadata for a single model field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMetadata {
    pub name: String,
    /// Raw Rust/SeaORM type string, e.g. "String", "i32", "Option<Uuid>"
    pub column_type: String,
    pub is_primary_key: bool,
    pub is_nullable: bool,
}

impl ServiceDef {
    /// Derives a ServiceDef from model metadata.
    ///
    /// Infers DataType from column_type strings and FieldMeaning from field names.
    /// System fields (id, created_at, updated_at) are marked read-only.
    /// Actions, state machines, and relationships are not derived — add them
    /// via the builder API after calling from_model().
    pub fn from_model(meta: &ModelMetadata) -> Self {
        let display = meta
            .display_name
            .clone()
            .unwrap_or_else(|| snake_to_title(&meta.name));

        let mut def = Self::new(&meta.name).display_name(display);

        for field in &meta.fields {
            let data_type = DataType::from_column_type(&field.column_type);
            let meaning = infer_meaning(&field.name);

            // System fields: read-only, always present
            let is_system = matches!(
                field.name.as_str(),
                "id" | "created_at" | "updated_at"
            ) || field.is_primary_key;

            let field_def = FieldDef {
                name: field.name.clone(),
                data_type,
                meaning,
                required: !field.is_nullable,
                is_list: false,
                readable: true,
                writable: !is_system,
            };
            def.fields.push(field_def);
        }

        def
    }
}

/// Converts snake_case to Title Case ("order_item" → "Order Item").
fn snake_to_title(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
```

**Note:** `def.fields.push()` bypasses the consuming builder methods, but that's acceptable because `from_model()` constructs the entire struct. Alternatively, call `.field()` / `.optional_field()` / `.read_only_field()` — but that requires pattern-matching on required+writable combos. Direct field push is simpler and equally correct since `from_model()` owns the struct.

### Pattern 3: MCP Tool Registration

Follow the existing pattern from `render_projection` and `list_models`. Three parts:

1. **Params struct** in `service.rs` with `#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]`
2. **Tool module** at `ferro-mcp/src/tools/generate_projection.rs` with `pub fn execute(...) -> GenerateProjectionResult`
3. **Handler method** on `FerroMcpService` in `service.rs` with `#[tool(...)]` annotation

```rust
// service.rs — params struct
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GenerateProjectionParams {
    /// Model name (e.g., "User", "Order"). Case-sensitive, matches struct name.
    pub model_name: String,
}

// service.rs — handler method
#[tool(
    name = "generate_projection",
    description = "..."
)]
pub async fn generate_projection(
    &self,
    params: Parameters<GenerateProjectionParams>,
) -> String {
    match tools::generate_projection::execute(&self.project_root, &params.0.model_name) {
        Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_default(),
        Err(e) => format!("Error: {e}"),
    }
}
```

The `execute()` function in `generate_projection.rs`:

```rust
// ferro-mcp/src/tools/generate_projection.rs
use ferro_projections::{derive_intents, ModelMetadata, FieldMetadata, ServiceDef};
use crate::tools::list_models;

pub fn execute(project_root: &Path, model_name: &str) -> Result<GenerateProjectionResult, String> {
    // 1. Fetch model details via existing list_models
    let models = list_models::execute(project_root)
        .map_err(|e| format!("list_models failed: {e}"))?;

    let model = models
        .iter()
        .find(|m| m.name == model_name)
        .ok_or_else(|| format!("model '{model_name}' not found"))?;

    // 2. Convert ModelDetails → ModelMetadata
    let meta = ModelMetadata {
        name: model.name.clone(),
        display_name: None,
        table: model.table.clone(),
        fields: model.fields.iter().map(|f| FieldMetadata {
            name: f.name.clone(),
            column_type: f.field_type.clone(),
            is_primary_key: f.is_primary_key,
            is_nullable: f.is_nullable,
        }).collect(),
    };

    // 3. Derive ServiceDef
    let service_def = ServiceDef::from_model(&meta);

    // 4. Derive intents
    let intents = derive_intents(&service_def);

    // 5. Build result with inferred-vs-manual note
    Ok(GenerateProjectionResult { service_def, intents, ... })
}
```

### Anti-Patterns to Avoid

- **Don't add SeaORM as a dependency to ferro-projections.** `ModelMetadata` exists specifically to avoid this. If you find yourself importing `sea_orm::` in ferro-projections, stop — the design is wrong.
- **Don't call `list_models::execute()` from within ferro-projections.** The conversion from `FieldInfo` to `FieldMetadata` lives in ferro-mcp, not in ferro-projections.
- **Don't use `Option<_>` stripping inside `infer_meaning()`.** The type string stripping belongs in `DataType::from_column_type()`. `infer_meaning()` works on field names, not types — keep them separate.
- **Don't call `list_models::execute()` and panic on empty result.** Return a proper `Result<_, String>` and surface "model not found" to the MCP caller.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead |
|---------|-------------|-------------|
| Field name → semantic meaning | Custom heuristic | `infer_meaning()` at `field.rs:86` — already has 7 rules |
| Column type → DataType | Separate mapping table | `DataType::from_column_type()` — add as inherent method on existing enum |
| SeaORM AST parsing | Custom syn visitor | `list_models::execute()` — already parses models, extracts field_type as string |
| Intent ranking | Custom scoring | `derive_intents()` in `ferro-projections::derive` — 5 analyzers, fully tested |
| MCP serialization | Custom JSON | `serde_json::to_string_pretty()` + `serde_json::to_value()` — existing pattern |

**Key insight:** Every component of the derivation pipeline already exists. This phase wires them together, not builds them from scratch.

---

## Common Pitfalls

### Pitfall 1: Option<T> Wrapper in Type Strings

**What goes wrong:** `list_models.rs` `type_to_string()` produces `"Option<String>"` for nullable fields. A naive `match type_str { "String" => ... }` misses these.

**Why it happens:** `type_to_string()` calls `.replace(' ', "")` but does not strip `Option<>` — nullable detection is handled separately via `is_nullable: bool`.

**How to avoid:** Strip `Option<...>` wrapper first in `DataType::from_column_type()` before pattern-matching the inner type.

**Warning signs:** If `DataType::String` is never returned for nullable string fields, the Option stripping is missing.

### Pitfall 2: is_nullable vs required Inversion

**What goes wrong:** `FieldDef.required = true` means the field is mandatory. `FieldInfo.is_nullable = true` means the column accepts NULL. These are inverses. Mapping `is_nullable → required` directly without negation is wrong.

**How to avoid:** `required: !field.is_nullable` — the negation is load-bearing.

### Pitfall 3: Primary Key Writability

**What goes wrong:** The `id` field is detected as `is_primary_key = true` by list_models, but `infer_meaning("id")` returns `FieldMeaning::Identifier`. Without special-casing, `from_model()` would mark `id` as writable (defaulting to read-write like any other required field).

**How to avoid:** System field detection must check `is_primary_key` flag in addition to name patterns (`"id"`, `"created_at"`, `"updated_at"`). Set `writable: false` for all system fields.

### Pitfall 4: Tool Registration Order in tool_router!

**What goes wrong:** Adding a new tool handler method without also adding `pub mod generate_projection;` in `tools/mod.rs` and the `generate_projection` method to `tool_router!` macro in `service.rs` results in the tool not appearing in the MCP tool list.

**How to avoid:** Three-location update: `tools/mod.rs` (module), `tools/generate_projection.rs` (implementation), `service.rs` (params struct + handler method + tool_router! macro entry).

**Warning signs:** Tool compiles but doesn't appear when the MCP server lists tools.

### Pitfall 5: Circular Type Matching for Rust Qualified Paths

**What goes wrong:** SeaORM models sometimes use fully-qualified types (`chrono::DateTime<chrono::Utc>` or `sea_orm::entity::prelude::DateTime`). After space-stripping, these become `"chrono::DateTime<chrono::Utc>"` — the prefix-based matching (`starts_with("DateTime")`) misses the qualified form.

**How to avoid:** Match on `contains("DateTime")` or `starts_with("DateTime") || contains("chrono::")` as specified in D-04.

---

## Code Examples

### Existing FieldInfo → ModelMetadata Mapping (verified against list_models.rs)

```rust
// Source: ferro-mcp/src/tools/list_models.rs — ModelDetails and FieldInfo structs
// FieldInfo fields: name: String, field_type: String, is_primary_key: bool, is_nullable: bool

let meta = ModelMetadata {
    name: model.name.clone(),
    display_name: None,
    table: model.table.clone(),
    fields: model
        .fields
        .iter()
        .map(|f| FieldMetadata {
            name: f.name.clone(),
            column_type: f.field_type.clone(), // already space-stripped
            is_primary_key: f.is_primary_key,
            is_nullable: f.is_nullable,
        })
        .collect(),
};
```

### Round-Trip Test (in-memory, no file I/O)

```rust
// Source: design decision D-09 / CONTEXT.md specifics section
#[test]
fn round_trip_order_model() {
    let meta = ModelMetadata {
        name: "order".to_string(),
        display_name: None,
        table: Some("orders".to_string()),
        fields: vec![
            FieldMetadata { name: "id".to_string(), column_type: "i32".to_string(), is_primary_key: true, is_nullable: false },
            FieldMetadata { name: "total".to_string(), column_type: "f64".to_string(), is_primary_key: false, is_nullable: false },
            FieldMetadata { name: "status".to_string(), column_type: "String".to_string(), is_primary_key: false, is_nullable: false },
            FieldMetadata { name: "notes".to_string(), column_type: "Option<String>".to_string(), is_primary_key: false, is_nullable: true },
            FieldMetadata { name: "created_at".to_string(), column_type: "DateTime<Utc>".to_string(), is_primary_key: false, is_nullable: false },
        ],
    };

    let service = ServiceDef::from_model(&meta);
    assert_eq!(service.name, "order");
    assert_eq!(service.display_name.as_deref(), Some("Order"));

    let id_field = service.fields.iter().find(|f| f.name == "id").unwrap();
    assert_eq!(id_field.data_type, DataType::Integer);
    assert_eq!(id_field.meaning, FieldMeaning::Identifier);
    assert!(!id_field.writable); // system field, read-only

    let notes_field = service.fields.iter().find(|f| f.name == "notes").unwrap();
    assert!(!notes_field.required); // is_nullable → !required

    let intents = derive_intents(&service);
    assert!(!intents.is_empty()); // pipeline produces ranked intents
}
```

### Tool Output Structure

```rust
// In ferro-mcp/src/tools/generate_projection.rs
#[derive(Debug, Serialize)]
pub struct GenerateProjectionResult {
    pub model_name: String,
    pub service_def: serde_json::Value,    // ServiceDef serialized to JSON
    pub intents: Vec<IntentInfo>,           // ranked intent scores
    pub inferred_count: usize,              // fields successfully inferred
    pub manual_enrichment_needed: Vec<String>, // ["actions", "state_machine", "relationships"]
}
```

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test runner (cargo test) |
| Config file | none — standard cargo test |
| Quick run command | `cargo test -p ferro-projections` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Behavior | Test Type | Automated Command |
|----------|-----------|-------------------|
| `DataType::from_column_type()` maps all 9 declared type patterns | unit | `cargo test -p ferro-projections from_column_type` |
| Option<> wrapper stripping | unit | included in from_column_type tests |
| `ServiceDef::from_model()` round-trip (fields, readable/writable, DataType, FieldMeaning) | unit | `cargo test -p ferro-projections from_model` |
| System fields marked read-only (id, created_at, updated_at, PK) | unit | included in round-trip test |
| is_nullable → !required mapping | unit | included in round-trip test |
| `generate_projection` MCP tool returns valid JSON ServiceDef | integration | `cargo test -p ferro-mcp generate_projection` |
| Full pipeline: ModelMetadata → ServiceDef → derive_intents → non-empty result | integration | `cargo test -p ferro-projections round_trip` |

### Wave 0 Gaps

- [ ] `ferro-projections/src/field.rs` — add `DataType::from_column_type()` tests alongside existing DataType tests
- [ ] `ferro-projections/src/service.rs` (or `metadata.rs`) — add round-trip tests for `ServiceDef::from_model()`
- [ ] `ferro-mcp/src/tools/generate_projection.rs` — new module with `execute()` + tests
- [ ] `ferro-mcp/src/tools/mod.rs` — add `pub mod generate_projection;`
- [ ] `ferro-mcp/src/service.rs` — add `GenerateProjectionParams` + handler method

---

## Environment Availability

Step 2.6: SKIPPED (no external dependencies — all changes are Rust source in existing crates with no new external deps).

---

## Open Questions

1. **ModelMetadata module placement**
   - What we know: CLAUDE.md for ferro-projections lists `metadata.rs` as a future module; `service.rs` already has ServiceDef.
   - What's unclear: whether splitting into `metadata.rs` adds clarity or just file count.
   - Recommendation: Put `ModelMetadata` and `FieldMetadata` in `service.rs` initially. If the file grows past ~400 lines, move to `metadata.rs`. This is discretion territory per CONTEXT.md.

2. **`from_model()` as inherent method vs standalone function**
   - What we know: Builder pattern uses consuming inherent methods. `from_model()` is a constructor, not a transformer.
   - Recommendation: Inherent method `ServiceDef::from_model(&ModelMetadata) -> Self`. Consistent with `ServiceDef::new()` as the other constructor entry point.

3. **Enum column types**
   - What we know: SeaORM allows custom enum columns (user-defined Rust enums). `type_to_string()` would return the enum's type name (e.g., `"OrderStatus"`).
   - What's unclear: Whether to fall through to `DataType::String` or add a `DataType::Enum` case. `DataType::Enum` already exists in the enum.
   - Recommendation: Match enum types heuristically — if the type string is capitalized and not a known primitive, map to `DataType::Enum`. Or fall through to `DataType::String` and let manual enrichment override. Either is acceptable; the simple fallback to String is safer for phase scope.

---

## Sources

### Primary (HIGH confidence)

- `ferro-projections/src/field.rs` — verified: `DataType` enum (10 variants), `FieldMeaning` enum (18 known + Custom), `infer_meaning()` function (lines 86–116), all existing test cases
- `ferro-projections/src/service.rs` — verified: `ServiceDef` struct, full builder API (`new`, `field`, `optional_field`, `read_only_field`, `write_only_field`, `list_field`)
- `ferro-mcp/src/tools/list_models.rs` — verified: `ModelDetails` struct, `FieldInfo` struct, `type_to_string()` method (space-stripping), `is_nullable` detection (Option<> prefix check), `execute()` function signature
- `ferro-mcp/src/service.rs` — verified: `#[tool]` + `tool_handler` + `tool_router` macro pattern, `Parameters<T>` wrapper, existing Params structs as templates
- `ferro-mcp/src/tools/render_projection.rs` — verified: existing projection tool pattern (lines 1–100), `derive_intents()` call pattern, intent serialization

### Secondary (MEDIUM confidence)

- `ferro-projections/CLAUDE.md` — crate boundary rules; confirmed no rendering or SeaORM deps allowed
- `.planning/phases/135-servicedef-derivation-bridge/135-CONTEXT.md` — all locked decisions (D-01 through D-11) as authoritative scope

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified against actual source files
- Architecture patterns: HIGH — all patterns derived from existing working code in the same codebase
- Pitfalls: HIGH — all pitfalls derived from reading actual implementation details (Option<> stripping, is_nullable inversion, tool_router! registration)

**Research date:** 2026-04-17
**Valid until:** 2026-05-17 (stable codebase, no external dependencies)
