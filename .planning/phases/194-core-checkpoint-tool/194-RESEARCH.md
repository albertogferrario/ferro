# Phase 194: Core Checkpoint Tool - Research

**Researched:** 2026-06-10
**Domain:** ferro-mcp tool authorship, ServiceDef introspection, field→column seam, MCP tool registration
**Confidence:** HIGH

## Summary

Phase 194 adds `checkpoint_projection { name }` as a new ferro-mcp tool. All source
primitives it needs already exist in the codebase: `reconstruct_service_def` parses
projection source into a `ServiceDef`, `projection_coverage` owns the
name-match predicate, and `list_models::execute` delivers the column set without a
running database. The implementation is additive: one new file
(`ferro-mcp/src/tools/checkpoint_projection.rs`), three additions to `service.rs`
(params struct, `#[tool(...)]` handler, `use tools::checkpoint_projection`), and one
`pub mod` line in `mod.rs`.

The research flag in CONTEXT.md D-05 is now resolved: `FieldDef` has **no**
computed/virtual flag and `FieldMeaning` has no computed variant. The exemption
relies entirely on D-04 (relationships live in `ServiceDef.relationships`, not
`.fields`) and on the vocabulary of column-backed builder methods (D-05 — see
below). No new marker is needed for Phase 194 provided the D-05 exemption set is
explicitly documented; the coherence question about growing `FieldDef` with an
explicit `computed` flag is deferred (noted in Assumptions Log).

The status-cache target `.ferro/checkpoints/` does not yet exist anywhere in the
codebase. It is a new convention introduced by this phase. The pattern for creating
it is `std::fs::create_dir_all` — already used in multiple tools — followed by
`fs::write(path, serde_json::to_string_pretty(&cache_entry)?)`.

**Primary recommendation:** implement in one module; reuse existing primitives
exactly; establish `SeamStatus` and `Finding` as the public contract that Phase 195
plugs into without modification.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01** Source-model resolution reuses `projection_coverage`'s existing predicate:
  `service_name.to_lowercase() == model_name.to_lowercase()`. Column set comes from
  `list_models::execute` (no running DB).
- **D-02** Field-name comparison is exact (case-sensitive snake_case). A missing
  column produces `Finding { subject: "<field>", detail: "no column `<field>` on
  entity `<entity>`", fix: "add column `<field>` to `<entity>` migration, or remove
  the field from the projection" }`.
- **D-03** No source-model match → seam 2 is `not_checked` with
  `reason: "source_model_unresolved"`. Never `pass`, never escalates to `fail`.
- **D-04** Relationship fields are exempt by construction — they live in
  `ServiceDef.relationships`, never in `.fields`.
- **D-05** Computed/virtual exemption is by vocabulary: only fields added by
  `.field(`, `.optional_field(`, `.read_only_field(`, `.write_only_field(` builders
  are subject to the column check. (RESEARCH FLAG resolved — see §D-05 Findings.)
- **D-06** Completeness detection: count column-backed builder invocations in source
  via regex; compare to `ServiceDef.fields.len()`. If source count > reconstructed
  count, report `warn` with `reason: "reconstruction_incomplete"`.
- **D-07** Output types: `Finding`, `SeamStatus`, `SeamResult`, `Verdict` — all
  public, all in `checkpoint_projection.rs`, reused verbatim by Phase 195.
  `SeamStatus` uses `#[serde(rename_all = "snake_case")]`.
- **D-08** Per-seam normalization functions at module boundary. In Phase 194 only
  seam 2 produces findings.
- **D-09** Verdict aggregation: `fail` if any seam fails; `warn` if any warning; else
  `pass`. `not_checked` seams listed but never raise to `fail`.
- **D-10** `next_steps` ranking: failures before warnings; within rank, earlier seam
  first. Dedup by `(subject, fix)`. Format: `"<fix> (seam: <seam_name>)"`. Cap at 10.
- **D-11** Cache write: `project_root/.ferro/checkpoints/{name}.json`. Create
  directory if absent. Include timestamp passed in (not read from wall-clock in
  logic). Derived ambient field: `clean` = pass, `failing` = warn/fail.

### Claude's Discretion

- Exact regex for builder-invocation counting (adapt from `render_projection`'s patterns).
- Internal helper-function split within `checkpoint_projection.rs`.
- Test fixture layout under `ferro-mcp`'s inline `#[cfg(test)]` module.

### Deferred Ideas (OUT OF SCOPE)

- Wrapper seams 1/3/4/5 (Phase 195).
- Inline generator hook (Phase 195).
- Ambient status read-surfacing in `application_info` / `projection_coverage` (Phase 195).
- Growing `FieldDef`/`FieldMeaning` with an explicit computed/virtual marker.
- Model-anchored fan-out.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CHK-01 | Single structured verdict (`status`, per-seam result list, ranked deduplicated `next_steps`, `source` provenance) | Output types `Verdict`, `SeamResult`, `Finding` established in D-07; aggregation in D-09/D-10. |
| CHK-02 | Field→column seam: resolve projection→model via `projection_coverage` predicate, reconstruct via `reconstruct_service_def`, compare against `list_models::execute` columns (no running DB) | Predicate confirmed at `projection_coverage.rs:75-79`; `list_models::execute` confirmed at lines 165-188; reuse path clear. |
| CHK-03 | Four-variant `SeamStatus`; `not_checked` never coerced to `pass`; unchecked seams listed but don't raise overall `status` to `fail`; dedicated test required | `SeamStatus` enum shape determined; cascade rule documented; test case list in Validation Architecture. |
| CHK-04 | No false positives on relationships or computed/virtual fields | Relationships in `ServiceDef.relationships` (separate `Vec<RelationshipDef>`), never in `.fields` — exemption is by construction. Computed/virtual: no marker exists; exemption via builder vocabulary (D-05). |
| CHK-05 | Reconstruction-completeness check: if builder-invocation count > `fields.len()`, report `warn` not silent `pass` | Four column-backed builders enumerated; counting regex approach viable from `parse_and_add_fields` patterns. |
| CHK-06 | Ranked, deduplicated, actionable `next_steps` | Ranking rule (failures first, seam-order within rank) and dedup key `(subject, fix)` confirmed in D-09/D-10. |
</phase_requirements>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Tool surface + wire protocol | ferro-mcp (MCP server) | — | All MCP tools live in `ferro-mcp/src/tools/`; dispatched via `service.rs` |
| Field→column seam logic | ferro-mcp tool module | ferro-projections (types only) | Source analysis is MCP-layer; `ServiceDef`/`FieldDef` types come from ferro-projections |
| Model-column resolution | ferro-mcp (`list_models::execute`) | — | Static AST scan, no DB; owned entirely by existing MCP tool |
| ServiceDef reconstruction | ferro-mcp (`render_projection::reconstruct_service_def`) | ferro-projections (`ServiceDef` builders) | Regex parser lives in MCP crate; builder methods used to populate |
| Aggregation + verdict | ferro-mcp tool module | — | Pure logic, no external deps |
| Status cache I/O | ferro-mcp tool module | filesystem | `std::fs::create_dir_all` + `fs::write`; new `.ferro/checkpoints/` convention |
| Output contract types | ferro-mcp (public in `checkpoint_projection.rs`) | Phase 195 (consumer) | D-07 mandates public types for Phase 195 reuse |

---

## Standard Stack

### Core (all already in `ferro-mcp/Cargo.toml`)

| Library | Version (workspace) | Purpose | Why Standard |
|---------|---------------------|---------|--------------|
| `serde` + `serde_json` | 1.x | Serialize output types and write cache JSON | Used by every existing MCP tool |
| `schemars` | 1.x | `JsonSchema` derive on params struct (required by `rmcp` tool machinery) | All tool params use it |
| `regex` | 1.x | Builder-invocation counting (reuse `parse_and_add_fields` patterns) | Already a dependency |
| `chrono` | 0.4 | Cache entry timestamp | Already a dependency |
| `thiserror` | 2 | Error type for internal functions | Project convention |

[VERIFIED: ferro-mcp/Cargo.toml] No new dependencies required. The zero-new-dependency
constraint from REQUIREMENTS.md is satisfied by construction.

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `std::fs` | stdlib | `create_dir_all`, `write` for cache | Cache write in D-11 |
| `tempfile` | 3 | Test fixtures requiring temp directories | Already in `[dev-dependencies]` |

---

## Architecture Patterns

### System Architecture Diagram

```
checkpoint_projection { name: "Booking" }
        |
        v
[checkpoint_projection.rs: run_for(project_root, name, now)]
        |
        +---> inspect_projection::execute()
        |         -> locate projection file in src/projections/
        |         -> return ProjectionDetail { file, service_name, display_name }
        |
        +---> fs::read_to_string(file)
        |
        +---> [seam 2: field_to_column_seam(content, service_name, display_name, project_root)]
        |         |
        |         +---> reconstruct_service_def(service_name, display_name, content)
        |         |         -> returns ServiceDef with .fields, .relationships
        |         |
        |         +---> count_column_backed_builders(content)  [D-06 completeness]
        |         |         -> regex scan for .field( .optional_field( .read_only_field( .write_only_field(
        |         |
        |         +---> list_models::execute(project_root)
        |         |         -> returns Vec<ModelDetails> (static AST, no DB)
        |         |
        |         +---> resolve model: service_name.to_lowercase() == model.name.to_lowercase()
        |         |         -> None => SeamResult { status: NotChecked, reason: "source_model_unresolved" }
        |         |
        |         +---> for each FieldDef in service.fields:
        |                   field.name not in model.fields[*].name => Finding { subject, detail, fix }
        |         -> SeamResult { seam: "field_to_column", status, findings }
        |
        +---> [seams 3/4/5: NotChecked stubs — Phase 195 fills these]
        |
        +---> aggregate_verdict(seam_results)  [D-09]
        |         -> status = fail|warn|pass
        |         -> next_steps from findings, ranked + deduped  [D-10]
        |
        +---> write_cache(project_root, name, verdict, now)  [D-11]
        |         -> .ferro/checkpoints/{name}.json
        |
        v
Verdict { status, projection, seams, next_steps }
```

### Recommended Project Structure

```
ferro-mcp/src/tools/
├── checkpoint_projection.rs    # NEW — seam 2, aggregation, cache write, public output types
├── mod.rs                      # ADD: pub mod checkpoint_projection;
├── render_projection.rs        # REUSE: reconstruct_service_def (pub(crate))
├── projection_coverage.rs      # REUSE: name-match predicate (pattern)
├── list_models.rs              # REUSE: execute() -> Vec<ModelDetails>
└── inspect_projection.rs       # REUSE: execute() to locate projection file

ferro-mcp/src/
└── service.rs                  # ADD: CheckpointProjectionParams, handler method
```

### Pattern 1: MCP Tool Registration

The `service.rs` wiring pattern is:

1. Define a params struct (derives `Debug, Clone, Deserialize, Serialize, JsonSchema`):

```rust
// Source: ferro-mcp/src/service.rs (every existing params struct follows this)
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CheckpointProjectionParams {
    /// Projection function name (e.g., "user_service") or service name (e.g., "User")
    pub name: String,
}
```

2. Add a `#[tool(...)]` method on `FerroMcpService` inside the `#[tool_router]` impl block:

```rust
// Source: ferro-mcp/src/service.rs (render_projection handler, lines ~1541-1553)
#[tool(
    name = "checkpoint_projection",
    description = "..."
)]
pub async fn checkpoint_projection(
    &self,
    params: Parameters<CheckpointProjectionParams>,
) -> String {
    match tools::checkpoint_projection::execute(&self.project_root, &params.0.name) {
        Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        Err(e) => format!("{{\"error\": \"{}\"}}", e.replace('"', "\\\"")),
    }
}
```

3. Add `pub mod checkpoint_projection;` to `mod.rs`.

No other registration steps: `#[tool_router(router = tool_router)]` macro scans the
impl block automatically.

[VERIFIED: ferro-mcp/src/service.rs, ferro-mcp/src/tools/mod.rs]

### Pattern 2: Output Types Shape

```rust
// Source: CONTEXT.md D-07 (locked), mirrors serde conventions from ferro-projections/src/field.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Finding {
    pub subject: String,
    pub detail: String,
    pub fix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SeamStatus {
    Pass,
    Warn,
    Fail,
    NotChecked,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SeamResult {
    pub seam: String,
    pub status: SeamStatus,
    pub source: String,
    pub findings: Vec<Finding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Verdict {
    pub status: SeamStatus,
    pub projection: String,
    pub seams: Vec<SeamResult>,
    pub next_steps: Vec<String>,
}
```

The `serde(rename_all = "snake_case")` on `SeamStatus` produces the wire values
`pass`, `warn`, `fail`, `not_checked` required by the design spec.

[VERIFIED: consistent with `FieldMeaning` and `DataType` patterns in
`ferro-projections/src/field.rs`]

### Pattern 3: Field-Builder Vocabulary (D-05 Findings)

`parse_and_add_fields` in `render_projection.rs` parses exactly **four**
column-backed builder methods:

| Builder method | Regex used | `FieldDef` effect |
|---------------|-----------|-------------------|
| `.field(` | `\.field\("([^"]+)",\s*DataType::\w+,\s*FieldMeaning::\w+\)` | `required=true, readable=true, writable=true` |
| `.optional_field(` | `\.optional_field\(...\)` | `required=false, readable=true, writable=true` |
| `.read_only_field(` | `\.read_only_field\(...\)` | `readable=true, writable=false` |
| `.write_only_field(` | `\.write_only_field\(...\)` | `readable=false, writable=true` |

[VERIFIED: ferro-mcp/src/tools/render_projection.rs lines 157-197]

**Finding for D-05:** `FieldDef` has no `computed` or `virtual` flag. `FieldMeaning`
has no `Computed` variant — only `Custom(String)` as a catchall. There is no
builder method for computed/virtual fields. This means: every field reachable via
`.fields` on a reconstructed `ServiceDef` was placed there by one of the four
column-backed builders above. There is no path by which a non-column-backed
"display-only computed field" enters `.fields` through the standard builder API.

The CHK-04 exemption therefore holds by the combination of D-04 and D-05: the seam
iterates only `ServiceDef.fields`, which the current builder vocabulary populates
only from column-backed builders. See Assumptions Log A1 for the residual risk.

[VERIFIED: ferro-projections/src/field.rs lines 35-72, ferro-projections/src/service.rs
lines 130-210]

### Pattern 4: `reconstruct_service_def` Call Convention

```rust
// Source: ferro-mcp/src/tools/validate_projection.rs lines 50-70
use super::render_projection::reconstruct_service_def;
use super::inspect_projection::InspectResult;

let inspect = super::inspect_projection::execute(project_root, name);
let detail = match inspect {
    InspectResult::Found(d) => d,
    InspectResult::NotFound(nf) => return Err(format!("not found: {:?}", nf.available)),
};
let file_path = project_root.join(&detail.file);
let content = fs::read_to_string(&file_path)?;
let service = reconstruct_service_def(&detail.service_name, &detail.display_name, &content)?;
```

The `inspect_projection::execute` handles the `src/projections/` walk and returns
`ProjectionDetail { file, service_name, display_name, ... }`. Seam 2 uses the same
call chain.

[VERIFIED: ferro-mcp/src/tools/validate_projection.rs lines 34-51;
ferro-mcp/src/tools/inspect_projection.rs lines 47-76]

### Pattern 5: Model-Resolution Predicate (D-01)

```rust
// Source: ferro-mcp/src/tools/projection_coverage.rs lines 72-79
let model_lower = service.name.to_lowercase();   // service.name == service_name from ServiceDef
let matched = all_models.iter().find(|m| {
    m.name.to_lowercase() == model_lower
});
```

`list_models::execute(project_root)` returns `Result<Vec<ModelDetails>, McpError>`.
Each `ModelDetails` has `fields: Vec<FieldInfo>` where each `FieldInfo.name` is the
column/field name. The column set for presence-checking is `model.fields.iter().map(|f| &f.name)`.

[VERIFIED: ferro-mcp/src/tools/projection_coverage.rs lines 50-80;
ferro-mcp/src/tools/list_models.rs lines 12-27, 165-188]

### Pattern 6: Status Cache Write (D-11)

No `.ferro/` directory exists anywhere in the codebase yet.

[VERIFIED: grep of entire codebase for `.ferro/` string returned only CSS class
strings like `.ferro-kanban-scroll` — the directory convention is new.]

The write pattern follows `generate_types.rs` and `whatsapp.rs`:

```rust
// Pattern from ferro-mcp/src/tools/generate_types.rs lines 110-116
use std::fs;
let cache_dir = project_root.join(".ferro").join("checkpoints");
fs::create_dir_all(&cache_dir)
    .map_err(|e| format!("failed to create cache dir: {e}"))?;
let path = cache_dir.join(format!("{name}.json"));
let json = serde_json::to_string_pretty(&cache_entry)
    .map_err(|e| format!("failed to serialize cache: {e}"))?;
fs::write(&path, json)
    .map_err(|e| format!("failed to write cache: {e}"))?;
```

The cache entry shape (per D-11) includes the full `Verdict` plus a derived
`ambient_status: "clean" | "failing"` field and a `checked_at` timestamp.

### Pattern 7: Seam-Cascade Rule

From STATE.md (locked):
- Seam 1 fail → seams 4 and 5 become `not_checked` with `reason: "seam_1_failed"`.
- Seam 4 fail → seam 5 becomes `not_checked` with `reason: "seam_4_failed"`.
- Seams 2 and 3 run independently of seam 1.

In Phase 194, seams 1/3/4/5 are stub `not_checked` entries (their logic is Phase 195).
The cascade rule needs to be documented in code comments but does not have real
execution paths yet.

### Anti-Patterns to Avoid

- **Collapsing `not_checked` to `pass`:** Any early-return or default path that
  returns `SeamStatus::Pass` without actually running the check violates CHK-03.
  Every prerequisite-absent path must return `SeamStatus::NotChecked`.
- **Re-implementing model loading:** Do not parse `src/models/` directly. Always use
  `list_models::execute` — it handles both `src/models/` and `src/entities/`.
- **Re-implementing projection file location:** Use `inspect_projection::execute` to
  locate the file, not a hand-rolled `src/projections/` scan.
- **Reading wall-clock in pure functions:** Pass `now: DateTime<Utc>` into `run_for`
  as a parameter so unit tests can control time. Do not call `Utc::now()` inside the
  seam logic.
- **Producing duplicate findings:** Dedup at aggregation time, not per-seam.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Projection file location | Custom `src/projections/` walk | `inspect_projection::execute(project_root, name)` | Already handles all edge cases, returns `InspectResult` with `NotFound` |
| ServiceDef from source | New regex parser | `render_projection::reconstruct_service_def` | Covers all 4 field builders + relationships + actions + state machine |
| Model column list | Parse `src/models/` manually | `list_models::execute(project_root)` | Handles both `src/models/` and `src/entities/` with syn AST |
| Name-match predicate | New lowercase comparison | `to_lowercase()` comparison as in `projection_coverage.rs:75-79` | Already battle-tested against real projections |

**Key insight:** Phase 194 is a pure orchestrator over existing primitives. Every
function it calls already exists. The only new logic is: the field-vs-column loop,
the completeness count, the aggregation function, and the cache write.

---

## D-05 Research Finding: Computed/Virtual Field Exemption

**Resolved:** `FieldDef` (ferro-projections/src/field.rs:59-72) has fields:
`name`, `data_type`, `meaning`, `required`, `is_list`, `readable`, `writable`.
No `computed`, `virtual`, or `derived` flag.

`FieldMeaning` (lines 35-56) has 18 known semantic variants plus `Custom(String)`.
No `Computed` or `Virtual` variant.

The four builder methods in `ServiceDef` that add to `.fields` are:
`.field()`, `.optional_field()`, `.read_only_field()`, `.write_only_field()`.
All four are column-backed by convention: they map a projection field to an entity
attribute.

**Conclusion for D-05:** The exemption holds structurally — there is no path in the
current API to add a non-column-backed field to `ServiceDef.fields`. The column
check is safe to apply to every entry in `service.fields` without a runtime
computed-field guard.

**Coherence note for deferred scope:** If a future ferro-projections version adds a
`.computed_field()` builder that adds non-column-backed entries to `.fields`, CHK-04
will need an explicit marker. This is surfaced as A1 in the Assumptions Log.

---

## Common Pitfalls

### Pitfall 1: `not_checked` vs `pass` at prerequisite-absent paths
**What goes wrong:** A default branch or early-return produces `SeamStatus::Pass`
when the actual check did not run (e.g. `list_models::execute` returns `Err`,
`reconstruct_service_def` returns `Err`, or no name-match). The agent treats the
tool's output as verified clean.
**Why it happens:** Natural Rust patterns — `unwrap_or_default`, implicit fallback.
**How to avoid:** Every `Err` or `None` path that aborts seam 2 must return
`SeamStatus::NotChecked` with a specific `reason` string.
**Warning signs:** Test for `not_checked` cases (CHK-03 dedicated test).

### Pitfall 2: Counting builder invocations in comment strings
**What goes wrong:** The regex for D-06 completeness matches `.field("name",` inside
a comment like `// .field("id", DataType::Integer, ...)`, inflating the invocation
count and producing spurious `warn` results.
**Why it happens:** The existing `parse_and_add_fields` regexes also run against
the full file content including comments. For the `find` / insert path this is
harmless (double-adding is idempotent via ServiceDef builder). For a count-based
completeness assertion it is not.
**How to avoid:** Strip line comments (`//`) before counting, or use the
`reconstruct_service_def` result directly (its `.fields.len()` already accounts for
only parseable builder calls). The simplest approach: if `reconstructed_count == counted_invocations` after stripping comments, trust the reconstruction; the mismatch signal remains meaningful.

### Pitfall 3: `write_only_field` not counted in completeness check
**What goes wrong:** The completeness check regex covers `.field(`, `.optional_field(`,
`.read_only_field(` but misses `.write_only_field(`. The reconstructed
`ServiceDef.fields` includes write-only fields (they set `readable=false, writable=true`),
so a projection that uses `.write_only_field(` would appear as reconstruction-incomplete
when it is not.
**Why it happens:** The fourth builder was added later and may be overlooked when
adapting the count logic.
**How to avoid:** The invocation-count regex must include all four builders from the
D-05 vocabulary table. Verify with a fixture that uses `.write_only_field(`.

### Pitfall 4: ServiceDef `name` vs projection function name
**What goes wrong:** The projection function name (e.g. `user_service`) differs from
`ServiceDef.name` (e.g. `"user"`). Using the function name for model resolution
mismatches (e.g. `user_service` != `user`).
**Why it happens:** `inspect_projection` returns both `ProjectionDetail.name`
(function name) and `ProjectionDetail.service_name` (the string passed to
`ServiceDef::new()`). D-01 matches on `service_name`.
**How to avoid:** Use `detail.service_name` (from `ProjectionDetail`) for the
model-resolution comparison, not `detail.name`.
**Warning signs:** Test fixture with non-trivial function name (e.g. `booking_service`
vs `service_name = "booking"`).

### Pitfall 5: Model `FieldInfo.name` vs SeaORM column name
**What goes wrong:** `list_models::execute` parses Rust struct field names from the
model source. SeaORM entities can use `#[sea_orm(column_name = "...")]` to alias
the column. The Rust field name and SQL column name would then differ, and the
field→column check would produce a false positive.
**Why it happens:** `list_models::FieldInfo.name` is the Rust field name, not the
SQL column name. Most SeaORM projects do not use column aliases for standard fields.
**How to avoid:** For Phase 194, accept that the check operates on Rust field names
(which match projection `FieldDef.name` by convention). If column aliases become a
source of false positives in Phase 196 dogfood, add `column_name` attribute
extraction to `list_models`. Document this as a known limitation.

---

## Code Examples

### Seam 2 Core Logic Sketch

```rust
// Source: pattern derived from validate_projection.rs + projection_coverage.rs
fn field_to_column_seam(
    project_root: &Path,
    service_name: &str,
    display_name: &Option<String>,
    content: &str,
) -> SeamResult {
    let service = match reconstruct_service_def(service_name, display_name, content) {
        Ok(s) => s,
        Err(e) => return SeamResult {
            seam: "field_to_column".to_string(),
            status: SeamStatus::NotChecked,
            source: "checkpoint".to_string(),
            findings: vec![],
            reason: Some(format!("reconstruction_failed: {e}")),
        },
    };

    // D-06: completeness check
    let invocation_count = count_column_backed_builders(content);
    if invocation_count > service.fields.len() {
        return SeamResult {
            seam: "field_to_column".to_string(),
            status: SeamStatus::Warn,
            source: "checkpoint".to_string(),
            findings: vec![Finding {
                subject: service_name.to_string(),
                detail: format!(
                    "reconstruction may be incomplete: {} builder calls in source, {} fields parsed",
                    invocation_count, service.fields.len()
                ),
                fix: "check for unsupported builder patterns in the projection source".to_string(),
            }],
            reason: Some("reconstruction_incomplete".to_string()),
        };
    }

    // D-01 model resolution
    let models = match list_models::execute(project_root) {
        Ok(m) => m,
        Err(_) => return SeamResult {
            seam: "field_to_column".to_string(),
            status: SeamStatus::NotChecked,
            source: "checkpoint".to_string(),
            findings: vec![],
            reason: Some("source_model_unresolved".to_string()),
        },
    };

    let model = match models.iter().find(|m| m.name.to_lowercase() == service_name.to_lowercase()) {
        Some(m) => m,
        None => return SeamResult {
            seam: "field_to_column".to_string(),
            status: SeamStatus::NotChecked,
            source: "checkpoint".to_string(),
            findings: vec![],
            reason: Some("source_model_unresolved".to_string()),
        },
    };

    let column_names: std::collections::HashSet<&str> =
        model.fields.iter().map(|f| f.name.as_str()).collect();

    // D-04: service.fields excludes relationships (they are in service.relationships)
    let mut findings = Vec::new();
    for field in &service.fields {
        if !column_names.contains(field.name.as_str()) {
            findings.push(Finding {
                subject: field.name.clone(),
                detail: format!("no column `{}` on entity `{}`", field.name, service_name.to_lowercase()),
                fix: format!(
                    "add column `{}` to `{}` migration, or remove the field from the projection",
                    field.name, service_name.to_lowercase()
                ),
            });
        }
    }

    SeamResult {
        seam: "field_to_column".to_string(),
        status: if findings.is_empty() { SeamStatus::Pass } else { SeamStatus::Fail },
        source: "checkpoint".to_string(),
        findings,
        reason: None,
    }
}
```

### Counting Column-Backed Builder Invocations (D-06)

```rust
// Adapted from parse_and_add_fields in render_projection.rs
fn count_column_backed_builders(content: &str) -> usize {
    // Strip line comments before counting to avoid false matches
    let no_comments: String = content.lines()
        .map(|line| {
            if let Some(pos) = line.find("//") { &line[..pos] } else { line }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let patterns = [
        r#"\.field\("#,
        r#"\.optional_field\("#,
        r#"\.read_only_field\("#,
        r#"\.write_only_field\("#,
    ];
    patterns.iter()
        .map(|p| regex::Regex::new(p).unwrap().find_iter(&no_comments).count())
        .sum()
}
```

### `next_steps` Aggregation (D-09, D-10)

```rust
fn aggregate_next_steps(seams: &[SeamResult]) -> Vec<String> {
    // Collect (rank, seam_order, subject, fix, formatted_string)
    let mut items: Vec<(u8, usize, String, String, String)> = Vec::new();
    for (idx, seam) in seams.iter().enumerate() {
        let rank: u8 = match seam.status {
            SeamStatus::Fail => 0,
            SeamStatus::Warn => 1,
            _ => continue,
        };
        for finding in &seam.findings {
            let entry = format!("{} (seam: {})", finding.fix, seam.seam);
            items.push((rank, idx, finding.subject.clone(), finding.fix.clone(), entry));
        }
    }
    // Sort by rank asc, then seam order asc
    items.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    // Dedup by (subject, fix)
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for (_, _, subject, fix, entry) in items {
        if seen.insert((subject, fix)) {
            result.push(entry);
            if result.len() == 10 { break; }  // D-10 cap
        }
    }
    result
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Agent calls validators individually in sequence | Single `checkpoint_projection` call returns unified verdict | v12.5 (this phase) | Eliminates agent-side sequencing burden |
| No field→column static check | Seam 2 presence check | v12.5 (this phase) | Silent F11-class seam becomes detectable before runtime |
| `not_checked` conflated with `pass` in some validators | Explicit four-variant `SeamStatus` enum | v12.5 (this phase) | Coverage honesty guaranteed by type |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | No current builder API can add a non-column-backed field to `ServiceDef.fields` — the CHK-04 computed/virtual exemption holds by vocabulary | D-05 Finding | If a consumer uses a hand-rolled `FieldDef` struct literal pushed directly onto `service.fields` (bypassing builders), the column check would produce a false positive. This bypasses the API entirely; treat as out-of-scope for now. |
| A2 | `list_models::FieldInfo.name` is the Rust struct field name, which by SeaORM convention matches the snake_case SQL column name. Custom `column_name` attribute overrides are rare. | Pitfall 5 | If a project uses `#[sea_orm(column_name = "...")]` aliases that differ from Rust field names, Phase 194 would false-positive on those fields. |
| A3 | The `.ferro/` directory is safe to create at project root alongside `.cargo/`, `.git/`, and Cargo.toml. No `.gitignore` or convention conflicts. | D-11 / cache write | If an existing project has a conflicting `.ferro` file (not directory), `create_dir_all` would fail. Negligible real-world risk. |

---

## Open Questions (RESOLVED)

1. **`write_only_field` builder in `parse_and_add_fields`**
   - **RESOLVED:** treat all four builders as column-backed for D-05/D-06 purposes (Plan 02 confirms; `count_includes_write_only` test is the regression guard).
   - What we know: `render_projection.rs` lines 186-196 include a `write_only_field`
     regex. `ServiceDef` in `service.rs` line 191 has a `write_only_field` builder.
   - What's unclear: the CONTEXT.md D-05 text lists only three builders in
     "column-backed builders" (`.field(`, `.optional_field(`, `.read_only_field(`).
     The `write_only_field` is in the existing parser but not mentioned in the
     exemption text.
   - Recommendation: treat all four as column-backed for D-05/D-06 purposes
     (they all map to real entity attributes). The plan must confirm this explicitly.

2. **`SeamStatus::NotChecked` serde wire value**
   - **RESOLVED:** snake_case rename of `NotChecked` produces `"not_checked"` exactly — no custom rename attribute needed.
   - What we know: `#[serde(rename_all = "snake_case")]` on `SeamStatus` with
     variant `NotChecked` would serialize to `"not_checked"`.
   - What's unclear: the design spec output example shows `"not_checked"` as the
     wire value; snake_case rename of `NotChecked` produces exactly that.
   - Recommendation: verified — no custom `serde(rename)` attribute needed.

---

## Environment Availability

Step 2.6: SKIPPED — Phase 194 is a pure code addition to ferro-mcp. No external
tools, services, or runtimes are required beyond the existing Rust toolchain and
Cargo workspace already in use.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`#[test]`) |
| Config file | none — inline `#[cfg(test)]` modules |
| Quick run command | `cargo test -p ferro-mcp checkpoint_projection` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CHK-01 | `Verdict` contains `status`, `seams`, `next_steps` with provenance | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::verdict_shape` | ❌ Wave 0 |
| CHK-02 | Dangling field detected (field in projection, not in model) | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam2_dangling_field` | ❌ Wave 0 |
| CHK-02 | Clean projection (all fields match columns) passes seam 2 | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam2_all_pass` | ❌ Wave 0 |
| CHK-03 | `not_checked` when source model unresolved — not `pass` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::not_checked_no_model` | ❌ Wave 0 |
| CHK-03 | `not_checked` when `reconstruct_service_def` fails | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::not_checked_bad_source` | ❌ Wave 0 |
| CHK-04 | Relationship fields (in `ServiceDef.relationships`) never flagged | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::relationships_not_flagged` | ❌ Wave 0 |
| CHK-05 | Reconstruction-incomplete yields `warn` not `pass` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::reconstruction_incomplete_warn` | ❌ Wave 0 |
| CHK-06 | Mixed findings produce correctly ranked, deduped `next_steps` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::next_steps_ranked_deduped` | ❌ Wave 0 |
| D-11 | Cache file written to `.ferro/checkpoints/{name}.json` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::cache_write` | ❌ Wave 0 |
| D-10 | `next_steps` capped at 10 | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::next_steps_cap_at_10` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-mcp checkpoint_projection`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-mcp/src/tools/checkpoint_projection.rs` — the module itself (all tests are inline)
- [ ] Test fixtures: inline `&str` constants representing:
  - A projection source with a dangling field (field not in model)
  - A projection source with a relationship + clean fields
  - A projection source that has more builder calls than parseable fields
  - A fully coherent minimal projection (all fields match model)
  - A "no model" scenario (service_name with no matching model)

No new test infrastructure files needed — all tests are inline `#[cfg(test)]`
modules within `checkpoint_projection.rs`.

---

## Security Domain

This is a read-only introspective tool operating on the local filesystem within
`project_root`. It:
- Reads source files (`.rs` files under `src/projections/` and `src/models/`)
- Writes to `.ferro/checkpoints/` under the same project root
- Has no HTTP surface, no user input beyond the `name` parameter, no SQL execution

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | minimal | `name` parameter is used as a filesystem path component via `project_root.join(".ferro/checkpoints/").join(format!("{name}.json"))` — must sanitize: reject `/`, `..`, null bytes |
| V4 Access Control | no | Tool runs with the same privileges as the `ferro mcp` process; no additional access control surface |
| V2 Authentication | no | MCP server-level concern, not per-tool |
| V3 Session Management | no | Stateless tool |
| V6 Cryptography | no | Cache is plaintext JSON; no secrets stored |

**Path traversal prevention (required):** The `name` parameter is used as the stem
of a filename in `.ferro/checkpoints/{name}.json`. Before constructing the path,
validate that `name` contains only alphanumeric characters, underscores, and hyphens
(or use `Path::file_name()` + assertion). This prevents a crafted `name` value like
`../../etc/passwd` from escaping the `.ferro/checkpoints/` directory.

---

## Sources

### Primary (HIGH confidence)

- `ferro-mcp/src/tools/render_projection.rs:113-197` — `reconstruct_service_def` and `parse_and_add_fields` (all four builder regexes confirmed)
- `ferro-mcp/src/tools/projection_coverage.rs:50-79` — name-match predicate, `list_models::execute` call pattern
- `ferro-mcp/src/tools/list_models.rs:12-27, 165-188` — `ModelDetails`, `FieldInfo`, `execute()` signature
- `ferro-projections/src/field.rs:35-72` — `FieldMeaning` enum (no `Computed` variant confirmed), `FieldDef` struct (no `computed` flag confirmed)
- `ferro-projections/src/service.rs:63-96, 130-210` — `ServiceDef` struct layout, builder methods
- `ferro-mcp/src/service.rs:391-410, 1541-1553` — `#[tool_router]` pattern, `#[tool]` attribute, params struct conventions
- `ferro-mcp/src/tools/mod.rs` — `pub mod` registration list
- `ferro-mcp/src/tools/validate_projection.rs:34-71` — end-to-end usage of `inspect_projection` + `reconstruct_service_def` pattern
- `ferro-mcp/Cargo.toml` — confirmed: `serde_json`, `regex`, `chrono`, `thiserror`, `tempfile` (dev) all present; no new dependencies required

### Secondary (MEDIUM confidence)

- `.planning/phases/194-core-checkpoint-tool/194-CONTEXT.md` — locked implementation decisions
- `.planning/REQUIREMENTS.md` CHK-01..CHK-06 — requirement text
- `.planning/STATE.md` — seam cascade rule, fix-string normalization, ambient freshness decisions

---

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — zero new dependencies, all reused from existing Cargo.toml
- Architecture: HIGH — all integration points verified in source code
- Pitfalls: HIGH (pitfalls 1-4) / MEDIUM (pitfall 5 — column_name alias edge case)
- D-05 resolution: HIGH — `FieldDef` and `FieldMeaning` confirmed by direct source read

**Research date:** 2026-06-10
**Valid until:** 2026-07-10 (stable — no fast-moving dependencies)
