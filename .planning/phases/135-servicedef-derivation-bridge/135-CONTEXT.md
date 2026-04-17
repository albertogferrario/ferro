# Phase 135: ServiceDef Derivation Bridge - Context

**Gathered:** 2026-04-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Add `ServiceDef::from_model()` derivation that infers fields, data types, and field meanings from SeaORM model metadata. Expose this through ferro-mcp as a `generate_projection` tool that produces a ServiceDef from model introspection. Include a round-trip test: create model → derive ServiceDef → derive intents → render.

This is the time-to-working-projection bottleneck. An agent should go from a SeaORM model to a rendered projection without hand-writing ServiceDef builders.

</domain>

<decisions>
## Implementation Decisions

### Model Metadata Source
- **D-01:** `ServiceDef::from_model()` takes an intermediate `ModelMetadata` struct, not a concrete SeaORM type. ferro-projections stays free of SeaORM dependencies.
- **D-02:** `ModelMetadata` contains: `name: String`, `display_name: Option<String>`, `table: Option<String>`, `fields: Vec<FieldMetadata>` where `FieldMetadata` has `name`, `column_type` (string), `is_primary_key`, `is_nullable`.
- **D-03:** ferro-mcp bridges by converting its parsed `ModelDetails` (from `list_models.rs` syn-based AST parsing) into `ModelMetadata`, then calls `ServiceDef::from_model()`.

### Field Type Mapping
- **D-04:** Add `DataType::from_column_type(type_str: &str) -> DataType` in ferro-projections `field.rs`. Pattern-matches common SeaORM/Rust types: `i32`/`i64`/`u32`/`u64` → Integer, `String` → String, `bool` → Boolean, `DateTime`/`chrono::` → DateTime, `f32`/`f64`/`Decimal` → Float, `Uuid` → Uuid, `Vec<u8>` → Binary, `serde_json::Value`/`Json` → Json.
- **D-05:** `infer_meaning()` (already exists) handles field name → `FieldMeaning` mapping. Combined with `DataType::from_column_type()`, a full `FieldDef` can be derived from name + type string + nullable flag.

### MCP Tool Design
- **D-06:** New `generate_projection` MCP tool returns serialized `ServiceDef` JSON, not Rust source code. An agent can inspect, refine, and decide how to use the JSON.
- **D-07:** Tool inputs: `model_name` (required). Finds the model via existing `list_models::execute()`, converts to `ModelMetadata`, calls `ServiceDef::from_model()`.
- **D-08:** Tool output includes: the `ServiceDef` JSON, the derived intent scores (via `derive_intents()`), and a note about what was inferred vs what needs manual enrichment (actions, state machines, relationships beyond FK hints).

### Derivation Scope
- **D-09:** `from_model()` infers fields only (name, DataType, FieldMeaning, required/optional). Actions, state machines, and explicit relationships are too domain-specific — they stay as manual builder additions.
- **D-10:** FK fields (detected by `_id` suffix via `infer_meaning()`) produce `FieldMeaning::ForeignKey` but do NOT auto-generate `RelationshipDef` entries. Relationship inference would require cross-model analysis beyond this phase's scope.
- **D-11:** System fields (`id`, `created_at`, `updated_at`) are included in the ServiceDef but marked with their semantic meanings (Identifier, CreatedAt, UpdatedAt) so renderers can handle them appropriately (e.g., hide from forms, show as metadata).

### Claude's Discretion
- Whether `ModelMetadata` lives in a new `metadata.rs` module or alongside `ServiceDef` in `service.rs`
- Whether `from_model()` is an inherent method on `ServiceDef` or a standalone function
- Display name derivation heuristic (snake_case → Title Case, or just capitalize)
- Test structure for the round-trip demonstration

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Core implementation targets
- `ferro-projections/src/service.rs` — ServiceDef struct and builder API
- `ferro-projections/src/field.rs` — DataType, FieldMeaning, FieldDef, infer_meaning()
- `ferro-projections/src/lib.rs` — Re-exports
- `ferro-projections/CLAUDE.md` — Crate boundary rules (no runtime logic in ServiceDef, no rendering deps)

### MCP introspection (model parsing)
- `ferro-mcp/src/tools/list_models.rs` — ModelDetails, FieldInfo, syn-based SeaORM model parsing
- `ferro-mcp/src/tools/mod.rs` — Tool registration
- `ferro-mcp/src/tools/render_projection.rs` — Existing projection rendering tool (pattern for new tool)

### Existing projection pipeline
- `ferro-projections/src/derive.rs` — derive_intents() function
- `ferro-projections/src/render/mod.rs` — Renderer trait, BaseContext

### Architecture references
- `ferro-projections/CLAUDE.md` — Crate boundary rules
- `.planning/codebase/ARCHITECTURE.md` — Layer breakdown

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `infer_meaning()` at field.rs:86-116 — field name → FieldMeaning, directly reusable in from_model()
- `list_models::ModelDetails` + `FieldInfo` at ferro-mcp — already parses SeaORM entities via syn AST, extracts name, type string, pk flag, nullable flag
- `render_projection.rs` tool — pattern for how MCP tools reconstruct a ServiceDef and call derive_intents()

### Established Patterns
- Builder API on ServiceDef: `new().field().optional_field()` — from_model() can use the same builder internally
- MCP tools return serde_json::Value serialized structs — generate_projection follows this
- Tool registration in ferro-mcp/src/tools/mod.rs follows a consistent module + pub fn execute() pattern

### Integration Points
- `ferro-mcp/src/tools/render_projection.rs` currently reconstructs ServiceDef by parsing Rust source code with regex. `generate_projection` would offer a cleaner path: parse model → derive ServiceDef → optionally render.
- `ferro-mcp/src/tools/projection_coverage.rs:51` already cross-references models vs projections — could use generate_projection to identify coverage gaps.

</code_context>

<specifics>
## Specific Ideas

- The existing `infer_meaning()` function in field.rs already handles 7 inference rules. from_model() chains: column_type string → DataType, field_name → FieldMeaning, nullable flag → required/optional.
- ferro-mcp's `list_models` uses syn AST parsing and already extracts `field_type` as a string — this maps directly to the proposed `DataType::from_column_type()`.
- The round-trip test can use a synthetic model (in-memory ModelMetadata) rather than requiring actual source files.

</specifics>

<deferred>
## Deferred Ideas

- Cross-model relationship inference (analyzing FK targets across models) — future phase
- Action inference from route handlers — future phase
- State machine inference from enum fields — future phase
- Crate consolidation audit → CONC-04 in v13.0

</deferred>

---

*Phase: 135-servicedef-derivation-bridge*
*Context gathered: 2026-04-17*
