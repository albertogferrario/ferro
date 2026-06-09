# Technology Stack — v12.5 Projection Checkpoint

**Project:** ferro / checkpoint_projection MCP tool
**Researched:** 2026-06-09
**Scope:** Incremental — what the NEW tool needs beyond the existing codebase.
**Confidence:** HIGH (all findings from direct source reading; no training-data guesses)

---

## Verdict: Zero new dependencies required

Everything `checkpoint_projection` needs exists in the current `ferro-mcp` crate graph. The implementation is purely additive: a new tool file plus hooks into `generate_projection`, `json_ui_generate`, `application_info`, and `projection_coverage`.

---

## Field→Column Resolver — Reusable Primitives

This is the only genuinely new check. Two source paths already exist for column names; use both in priority order.

### Source 1 — Entity fields via `list_models` (static analysis, always available)

**Module:** `ferro-mcp/src/tools/list_models.rs`

`list_models::execute(project_root)` scans `src/models/` and `src/entities/` with `syn` + `WalkDir`, parses `#[derive(DeriveEntityModel)]` structs, and returns `Vec<ModelDetails>` where each entry carries `Vec<FieldInfo> { name, field_type, is_primary_key, is_nullable }`.

The `name` field on each `FieldInfo` is the entity column name (snake_case). This is exactly what the field→column check needs to assert that every `FieldDef.name` in the `ServiceDef` has a counterpart.

**How to use it in `checkpoint_projection`:**
1. Resolve the projection's source model name: `ServiceDef.name` (snake_case) matches the `service_name` extracted by `projection_coverage::execute` / `list_projections::execute`. The mapping is `projection.service_name.to_lowercase() == model.name.to_lowercase()` — the exact predicate already in `projection_coverage.rs:76-79`.
2. Call `list_models::execute(project_root)` (already imported in `projection_coverage.rs` at line 51 as `super::list_models::execute`).
3. Find the matching `ModelDetails` and build a `HashSet<&str>` of `FieldInfo.name` values.
4. For each `FieldDef` in the `ServiceDef`, assert membership. Missing → seam-2 finding.

No new dependency. `syn`, `walkdir`, and `quote` are already in `ferro-mcp/Cargo.toml`.

### Source 2 — Live DB schema via `database_schema` (runtime, requires DATABASE_URL)

**Module:** `ferro-mcp/src/tools/database_schema.rs`

`database_schema::execute(project_root, table_filter)` connects to the live DB (SQLite/Postgres/MySQL via SeaORM) and returns `SchemaInfo { tables: Vec<TableInfo { name, columns: Vec<ColumnInfo { name, ... }> } }`.

This gives the real applied-migration column set. Use it as a secondary check when the DB is reachable: if a column exists in the entity file but not in the live schema, that is a migration-pending finding (warn, not fail). If a projection field has no entity column AND no DB column, that is a fail.

Preference: entity-file scan (Source 1) first. It is synchronous and works without a running database, matching the checkpoint's read-only, no-runtime requirement. DB schema is a strengthening pass, not a prerequisite.

No new dependency. `sea-orm` is already in `ferro-mcp/Cargo.toml` with SQLite + Postgres features.

### Source 3 — Migration source (static, lower fidelity) — DO NOT USE

**Module:** `ferro-mcp/src/tools/list_migrations.rs`

`scan_migration_files` lists migration filenames; there is no column-level parsing of migration source. Do not add migration AST parsing for v12.5 — the entity file (Source 1) already contains the column set SeaORM materializes from migrations. Parsing migration source would duplicate information at lower fidelity (migration source is imperative Rust, harder to parse than entity struct definitions).

---

## ServiceDef Reconstruction — Reuse Exactly

**Module:** `ferro-mcp/src/tools/render_projection.rs` → `reconstruct_service_def(service_name, display_name, content)`

This is the canonical way all existing validators get their `ServiceDef` from source. It is already used by `validate_projection::execute_single`, `render_projection::execute`, and `projection_coverage::derive_primary_intent`. The checkpoint must use the same entry point to avoid divergence.

`FieldDef.name` on the reconstructed `ServiceDef` is the authoritative field-name list to check against the column set from Source 1.

---

## Aggregation and Verdict — Existing Primitives

**serde / serde_json** — already in `ferro-mcp/Cargo.toml` (`serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`). The verdict struct (`CheckpointVerdict`, `SeamResult`, `Finding`) is plain `#[derive(Serialize)]` data — no additional crate.

**HashSet** — standard library. No crate.

**SeamStatus enum** (`pass | fail | warn | not_checked`) — plain enum, serializes with `#[serde(rename_all = "snake_case")]` following the existing convention in `ferro-projections`.

---

## Dispatch to Existing Validators

All wrapper seams (1, 3, 4, 5) call existing synchronous or async functions in other tool modules and repackage their output as `SeamResult`. No new logic, no new deps:

| Seam | Existing entry point | Return type to repackage |
|------|---------------------|--------------------------|
| 1 — well-formed | `validate_projection::execute_single(root, name)` | `ValidationResult` |
| 3 — action→route | `json_ui_verify_action::execute(...)` | existing result |
| 4 — rendered view | `render_projection::execute(...)` + `json_ui_validate_spec::execute(...)` | `RenderResult` + spec validation |
| 5 — props→contract | `validate_contracts::execute(...)` | existing result |

---

## Existing Dependencies That Cover Each Concern

| Concern | Crate | Version in Cargo.toml |
|---------|-------|----------------------|
| Source parsing (entities / models) | `syn` | 2 |
| File traversal | `walkdir` | 2 |
| Regex-based projection parsing | `regex` | 1 |
| JSON serialization of verdict | `serde` + `serde_json` | 1 |
| ServiceDef type | `ferro-projections` | workspace (0.2.49) |
| Live DB schema | `sea-orm` | 1.0 |
| Async runtime | `tokio` | 1 |

All present. None need a version bump for this feature.

---

## What NOT to Add

| Item | Why not |
|------|---------|
| SeaORM entity-metadata reflection crate | Entity fields are already parsed statically by `list_models` via `syn`; no ORM-level reflection needed |
| Migration AST parser | Entity struct is the ground truth for column names SeaORM materializes; migration source is derivative and harder to parse |
| Any diff/comparison crate | Column membership check is a `HashSet::contains` — no library needed |
| `indexmap` or ordered map | Verdict `next_steps` is a sorted `Vec<String>` built inline; no ordered-map crate needed |
| `petgraph` or similar | Seam spine is a fixed five-element array, not a runtime graph |
| Any new async executor | `tokio` already present; DB-schema branch reuses same async path as `database_schema.rs` |
| New error-type crate | `thiserror` already in Cargo.toml; existing `McpError` is sufficient, or a local `CheckpointError` derives from it |

---

## Implementation Entry Points

```
ferro-mcp/src/tools/checkpoint_projection.rs  (new file)
  |
  +-- list_models::execute(root)               // entity column set (Source 1)
  +-- database_schema::execute(root, filter)   // live DB column set (Source 2, optional)
  +-- render_projection::reconstruct_service_def(...)  // ServiceDef reconstruction
  +-- validate_projection::execute_single(root, name)  // seam 1
  +-- json_ui_verify_action::execute(...)      // seam 3
  +-- render_projection::execute(...)          // seam 4 (render half)
  +-- json_ui_validate_spec::execute(...)      // seam 4 (validate half)
  +-- validate_contracts::execute(...)         // seam 5
```

Model→projection name resolution — copy the predicate from `projection_coverage.rs:76-79`:
```rust
p.service_name.as_ref().is_some_and(|sn| sn.to_lowercase() == model_lower)
```

Column set from entity fields (Source 1):
```rust
let columns: HashSet<&str> = model.fields.iter().map(|f| f.name.as_str()).collect();
```

Field→column seam check loop:
```rust
for field in &service.fields {
    if !columns.contains(field.name.as_str()) {
        // emit Finding { subject: field.name, detail: "...", fix: "..." }
    }
}
```

When `list_models` returns no match for the projection's `service_name`, seam 2 reports
`not_checked` (never `pass`).

---

## Sources

- `ferro-mcp/src/tools/list_models.rs` — direct read (entity field extraction via `syn`)
- `ferro-mcp/src/tools/database_schema.rs` — direct read (live DB column query)
- `ferro-mcp/src/tools/projection_coverage.rs` — direct read (model↔projection name matching)
- `ferro-mcp/src/tools/render_projection.rs` — direct read (`reconstruct_service_def` entry point)
- `ferro-mcp/src/tools/validate_projection.rs` — direct read (seam-1 dispatch)
- `ferro-mcp/Cargo.toml` — direct read (all dependency versions)
- `ferro-projections/src/service.rs` — direct read (`ServiceDef`, `FieldDef`, `validate()`)
- `docs/superpowers/specs/2026-06-09-projection-checkpoint-design.md` — direct read (design spec)
