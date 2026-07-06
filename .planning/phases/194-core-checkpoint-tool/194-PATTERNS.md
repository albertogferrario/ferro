# Phase 194: Core Checkpoint Tool - Pattern Map

**Mapped:** 2026-06-10
**Files analyzed:** 3 new/modified files
**Analogs found:** 3 / 3

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-mcp/src/tools/checkpoint_projection.rs` | tool / service | request-response + file-I/O | `ferro-mcp/src/tools/validate_projection.rs` | exact |
| `ferro-mcp/src/tools/mod.rs` | config / registry | — | `ferro-mcp/src/tools/mod.rs` (self) | exact — one `pub mod` line added |
| `ferro-mcp/src/service.rs` | service / router | request-response | `ferro-mcp/src/service.rs` (self, render_projection handler) | exact — params struct + `#[tool]` method |

---

## Pattern Assignments

### `ferro-mcp/src/tools/checkpoint_projection.rs` (tool, request-response + file-I/O)

**Primary analog:** `ferro-mcp/src/tools/validate_projection.rs`
**Secondary analog:** `ferro-mcp/src/tools/projection_coverage.rs` (name-match predicate)
**File-write analog:** `ferro-mcp/src/tools/generate_types.rs` (dir creation + JSON write)

---

#### Imports pattern

Copy from `validate_projection.rs` lines 1-12 and extend:

```rust
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::fs;
use std::path::Path;

use super::inspect_projection::InspectResult;
use super::render_projection::reconstruct_service_def;
use super::list_models;
```

Note: `validate_projection.rs` does not use `JsonSchema` on its result structs — it uses only `Serialize`. Phase 194 output types must derive both `Serialize` and `JsonSchema` (locked by D-07; these types are public and consumed by the MCP tool machinery). Add `use schemars::JsonSchema;` alongside `use serde::{Deserialize, Serialize};`.

---

#### Output types pattern

All four output types must be `pub`, derive `Serialize + Deserialize + JsonSchema`, and live at the top of the file. The `SeamStatus` enum uses `#[serde(rename_all = "snake_case")]` to produce wire values `pass`, `warn`, `fail`, `not_checked`.

Mirror the derive convention from `ferro-projections/src/field.rs` lines 35-56 (`FieldMeaning` enum) and lines 59-72 (`FieldDef` struct):

```rust
// Output types — public contract reused verbatim by Phase 195
// Derive order: Debug, Clone, Serialize, Deserialize, JsonSchema
// PartialEq + Eq on SeamStatus for aggregation comparisons

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

---

#### `execute` function shape

Copy the `execute_single` shape from `validate_projection.rs` lines 34-71. The public entry point is:

```rust
pub fn execute(project_root: &Path, name: &str) -> Result<Verdict, String>
```

The internal `run_for` that accepts a timestamp is called by `execute` with `chrono::Utc::now()`. Keep `run_for` package-private (`pub(crate)` or just `fn`) so tests can inject time:

```rust
pub fn execute(project_root: &Path, name: &str) -> Result<Verdict, String> {
    run_for(project_root, name, chrono::Utc::now())
}

pub(crate) fn run_for(
    project_root: &Path,
    name: &str,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<Verdict, String> {
    // 1. locate projection via inspect_projection
    // 2. read source
    // 3. run seam 2 (field_to_column_seam)
    // 4. stub seams 1/3/4/5 as NotChecked
    // 5. aggregate_verdict
    // 6. write_cache
    // 7. return Verdict
}
```

---

#### Projection-location + reconstruction pattern

Copy verbatim from `validate_projection.rs` lines 34-50:

```rust
// ferro-mcp/src/tools/validate_projection.rs:34-50
let inspect = super::inspect_projection::execute(project_root, name);
let detail = match inspect {
    InspectResult::Found(d) => d,
    InspectResult::NotFound(nf) => {
        return Err(format!(
            "projection '{}' not found. Available: {:?}",
            nf.name, nf.available
        ))
    }
};

let file_path = project_root.join(&detail.file);
let content = fs::read_to_string(&file_path)
    .map_err(|e| format!("failed to read {}: {e}", detail.file))?;

let service = reconstruct_service_def(&detail.service_name, &detail.display_name, &content)?;
```

For checkpoint seam 2 specifically, a failed `reconstruct_service_def` must return `SeamStatus::NotChecked` (not propagate `Err`). Wrap the call differently from `validate_projection` — use the `Ok`/`Err` arms to build a `SeamResult` rather than propagating the error up.

---

#### Model name-match predicate pattern

Copy from `projection_coverage.rs` lines 72-79:

```rust
// ferro-mcp/src/tools/projection_coverage.rs:72-79
let model_lower = detail.service_name.to_lowercase();
let matched = all_models.iter().find(|m| {
    m.name.to_lowercase() == model_lower
});
```

Use `detail.service_name` (from `ProjectionDetail`), not `detail.name` (the function name). See RESEARCH.md Pitfall 4.

`list_models::execute` returns `Result<Vec<ModelDetails>, McpError>`. Column presence is checked against `model.fields.iter().map(|f| f.name.as_str())`.

---

#### Field-builder regex pattern (D-05/D-06)

Copy the four regex patterns from `render_projection.rs` lines 159-196. For the invocation-count function, adapt to plain `r"\.field\("` match-count form (not capture groups). Strip line-comments before counting (RESEARCH.md Pitfall 2):

```rust
// ferro-mcp/src/tools/render_projection.rs:159-196 — adapt for counting
// Four builder patterns (all column-backed — D-05 vocabulary):
// r#"\.field\("# — .field(
// r#"\.optional_field\("# — .optional_field(
// r#"\.read_only_field\("# — .read_only_field(
// r#"\.write_only_field\("# — .write_only_field(

fn count_column_backed_builders(content: &str) -> usize {
    // Strip // line comments to avoid matching commented-out builder calls
    let no_comments: String = content
        .lines()
        .map(|line| {
            if let Some(pos) = line.find("//") { &line[..pos] } else { line }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let patterns = [
        r"\.field\(",
        r"\.optional_field\(",
        r"\.read_only_field\(",
        r"\.write_only_field\(",
    ];
    patterns
        .iter()
        .map(|p| regex::Regex::new(p).unwrap().find_iter(&no_comments).count())
        .sum()
}
```

---

#### Status cache write pattern (D-11)

Copy from `generate_types.rs` lines 110-116:

```rust
// ferro-mcp/src/tools/generate_types.rs:110-116
// Adapted: use project_root/.ferro/checkpoints/{name}.json

let cache_dir = project_root.join(".ferro").join("checkpoints");
fs::create_dir_all(&cache_dir)
    .map_err(|e| format!("failed to create cache dir: {e}"))?;
let path = cache_dir.join(format!("{name}.json"));
let json = serde_json::to_string_pretty(&cache_entry)
    .map_err(|e| format!("failed to serialize cache: {e}"))?;
fs::write(&path, json)
    .map_err(|e| format!("failed to write cache: {e}"))?;
```

`generate_types.rs` uses `McpError::IoError` for `fs::create_dir_all` and `fs::write`. In `checkpoint_projection.rs` (which uses `Result<_, String>` not `Result<_, McpError>`), map the `io::Error` to a formatted string instead.

`cache_entry` is an anonymous struct (or a `CacheEntry` struct) containing the full `Verdict` plus `ambient_status: &str` (`"clean"` if pass, else `"failing"`) and `checked_at: DateTime<Utc>` (the `now` parameter — not wall-clock from inside logic).

**Path traversal guard** (RESEARCH.md Security section): before constructing the path, validate `name` contains only `[a-zA-Z0-9_-]`. Reject with `Err(...)` otherwise.

---

#### Test module pattern

Copy the `#[cfg(test)]` structure from `validate_projection.rs` lines 114-269. Each test:

1. Creates a `tempfile::tempdir()`.
2. Creates `tmp.path().join("src/projections")` and optionally `tmp.path().join("src/models")`.
3. Writes inline `r#"..."#` source strings via `std::fs::write`.
4. Calls the function under test directly.
5. Asserts on the result struct fields.

For `checkpoint_projection` tests that need a model, also populate `tmp.path().join("src/models")` with a minimal SeaORM entity source. Example fixture pattern from `validate_projection.rs` lines 162-184:

```rust
// ferro-mcp/src/tools/validate_projection.rs:162-184
let tmp = tempfile::tempdir().unwrap();
let proj_dir = tmp.path().join("src/projections");
std::fs::create_dir_all(&proj_dir).unwrap();

let content = r#"
use ferro::{ServiceDef, DataType, FieldMeaning};

pub fn order_service() -> ServiceDef {
    ServiceDef::new("order")
        .display_name("Order")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("total", DataType::Float, FieldMeaning::Money)
}
    "#;
std::fs::write(proj_dir.join("order.rs"), content).unwrap();
```

For seam 2 tests requiring a model, add a parallel `src/models/` fixture with a SeaORM-style struct (fields with `name`, `field_type`, and optionally `#[sea_orm(primary_key)]`). Pattern: any struct that `list_models::execute` can parse as a `ModelDetails`.

---

### `ferro-mcp/src/tools/mod.rs` (config/registry)

**Analog:** `ferro-mcp/src/tools/mod.rs` lines 1-65 (self — one line addition)

Add one line in alphabetical order in the `pub mod` list (between `cache_inspect` and `code_templates`):

```rust
// ferro-mcp/src/tools/mod.rs — insert alphabetically
pub mod checkpoint_projection;
```

The current list ends at line 65 (`pub mod whatsapp;`). Alphabetically, `checkpoint_projection` falls between `cache_inspect` (line 8) and `code_templates` (line 9).

---

### `ferro-mcp/src/service.rs` (service/router — params struct + handler)

**Analog:** `ferro-mcp/src/service.rs` lines 302-310 (RenderProjectionParams) and lines 1531-1553 (render_projection handler)

**Step 1 — params struct** (place after existing projection params structs near line 318):

```rust
// ferro-mcp/src/service.rs — after ValidateProjectionParams (line 318)
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CheckpointProjectionParams {
    /// Projection function name (e.g. "user_service") or service name (e.g. "User").
    pub name: String,
}
```

**Step 2 — handler method** (place after `validate_projection` handler, around line 1581):

```rust
// ferro-mcp/src/service.rs — after validate_projection handler (~line 1581)
/// Run the projection checkpoint and return a structured verdict
#[tool(
    name = "checkpoint_projection",
    description = "Run a checkpoint on a service projection and return a structured verdict.\n\n\
        **When to use:** Verifying projection–model coherence before deploying, \
        CI validation, debugging field→column mismatches.\n\n\
        **Returns:** Per-seam status (pass/warn/fail/not_checked), findings with actionable fix strings, \
        ranked next_steps. Also writes a status cache to .ferro/checkpoints/{name}.json.\n\n\
        **Combine with:** `validate_projection` for structural checks, `projection_coverage` for coverage gaps."
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

The error-formatting pattern (`e.replace('"', "\\\"")`) is copied from the `deploy_check` handler at lines 423-424, which escapes interior double-quotes correctly for JSON string embedding.

---

## Shared Patterns

### Reconstruct-then-branch pattern (not_checked on failure)

**Source:** `validate_projection.rs` lines 34-70 (contrast: propagates `Err`; checkpoint must not)
**Apply to:** seam 2 in `checkpoint_projection.rs`

`validate_projection` propagates `reconstruct_service_def` errors upward with `?`. Checkpoint must instead return `SeamStatus::NotChecked` on any sub-step failure (D-03). The pattern inversion is explicit:

```rust
// validate_projection.rs (propagates):
let service = reconstruct_service_def(...)?;

// checkpoint_projection.rs (must branch instead):
let service = match reconstruct_service_def(...) {
    Ok(s) => s,
    Err(e) => return SeamResult { status: SeamStatus::NotChecked, reason: Some(...), ... },
};
```

### Serde snake_case enum pattern

**Source:** `ferro-projections/src/field.rs` — `FieldMeaning` enum uses `#[serde(rename_all = "snake_case")]` (lines 35-56 show the pattern; actual rename attribute is confirmed there).
**Apply to:** `SeamStatus` enum in `checkpoint_projection.rs`

The `NotChecked` variant serializes to `"not_checked"` with `rename_all = "snake_case"`. No custom `#[serde(rename = "...")]` attribute is needed.

### `serde_json::to_string_pretty` + `unwrap_or_else` handler pattern

**Source:** `service.rs` lines 1548-1551 (render_projection handler)
**Apply to:** `checkpoint_projection` handler in `service.rs`

```rust
// service.rs:1548-1551
Ok(result) => {
    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
}
Err(e) => format!("{{\"error\": \"{e}\"}}"),
```

The `checkpoint_projection` error arm must use `e.replace('"', "\\\"")` rather than bare `{e}` interpolation, because the error string may contain paths with quotes. See `deploy_check` handler pattern at `service.rs` lines 423-424.

### `tempfile` + `create_dir_all` + inline `r#"..."#` test fixture pattern

**Source:** `validate_projection.rs` lines 162-184 (and 189-218 for a second example)
**Apply to:** All `#[cfg(test)]` test cases in `checkpoint_projection.rs`

Tests requiring a projection file:
1. `tempfile::tempdir()` for an isolated root
2. `std::fs::create_dir_all(tmp.path().join("src/projections"))` 
3. `std::fs::write(proj_dir.join("{name}.rs"), r#"..."#)` with inline source

Tests also requiring a model file:
4. `std::fs::create_dir_all(tmp.path().join("src/models"))`
5. `std::fs::write(models_dir.join("{name}.rs"), r#"..."#)` with a minimal SeaORM entity struct

---

## No Analog Found

All three files have close analogs. No gaps.

---

## Key Observations for Executor

1. **`reconstruct_service_def` visibility** — It is declared `pub(crate)` in `render_projection.rs` line 114. Since `checkpoint_projection.rs` lives in the same crate (`ferro-mcp`), `use super::render_projection::reconstruct_service_def;` is the correct import form — same as `validate_projection.rs` line 11.

2. **`list_models::execute` error type** — Returns `Result<Vec<ModelDetails>, McpError>`. `checkpoint_projection.rs` uses `Result<_, String>` as its external error type. The `McpError` must be mapped: `.map_err(|e| e.to_string())` or branch on `Err` to produce `SeamStatus::NotChecked` (per D-03 and Pitfall 1).

3. **`ProjectionDetail.service_name` is `String`, not `Option<String>`** — Confirmed in `validate_projection.rs` line 50 (`&detail.service_name` used directly). The model-resolution predicate in `projection_coverage.rs` line 77 uses `p.service_name.as_ref().is_some_and(...)` because `ProjectionInfo.service_name` is `Option<String>`. In `checkpoint_projection.rs`, use `detail.service_name` directly (it is a plain `String` from `ProjectionDetail`).

4. **Four-builder completeness regex must include `.write_only_field(`** — RESEARCH.md Pitfall 3 calls this out explicitly. All four patterns from `render_projection.rs` lines 159-196 are column-backed and must be counted.

5. **Params struct derives** — `Deserialize` is required (rmcp machinery deserializes incoming JSON) even though the tool returns `String`. Match the full derive set of the adjacent params structs: `Debug, Clone, Deserialize, Serialize, JsonSchema`.

---

## Metadata

**Analog search scope:** `ferro-mcp/src/tools/`, `ferro-mcp/src/service.rs`, `ferro-projections/src/`
**Files scanned:** 6 analog files read in full
**Pattern extraction date:** 2026-06-10
