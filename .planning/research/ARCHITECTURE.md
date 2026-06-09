# Architecture Research: v12.5 Projection Checkpoint

**Domain:** ferro-mcp tool integration
**Researched:** 2026-06-09
**Confidence:** HIGH — based on direct source inspection of all relevant tools

---

## System Overview

The checkpoint lives entirely within `ferro-mcp`. No new crates are needed.

```
┌──────────────────────────────────────────────────────────────────────┐
│  MCP Tool Entry Points                                                │
│                                                                      │
│  generate_projection     json_ui_generate                            │
│       │                        │                                     │
│       └──────────┬─────────────┘                                     │
│                  ▼  (inline hook, P2)                                │
│        checkpoint_projection  ◄──── NEW TOOL (P1)                   │
│                  │                                                    │
│       ┌──────────┼────────────────────────────────┐                 │
│       ▼          ▼          ▼          ▼           ▼                 │
│  validate_  json_ui_   render_   json_ui_    validate_               │
│  projection verify_    projection validate_  contracts               │
│  (seam 1)  action     (seam 4)  spec        (seam 5)                │
│            (seam 3)             (seam 4b)                            │
│                  │                                                    │
│       field→column resolver ──── NEW LOGIC (seam 2, owned here)     │
│       (reads: list_models, db_schema introspection)                  │
│                                                                      │
├──────────────────────────────────────────────────────────────────────┤
│  Read-only status consumers                                          │
│                                                                      │
│  application_info    projection_coverage                             │
│  (surfaces per-projection checkpoint_status: unverified/failing/    │
│   clean as a new field — P2)                                         │
└──────────────────────────────────────────────────────────────────────┘
```

---

## Question 1: No-Duplication Dispatch Pattern

### How existing tools are structured (observed)

All tools in `ferro-mcp/src/tools/` follow the same pattern:

- Each tool has a public `execute(project_root, ...)` function that is the external entry point.
- Internal helpers are extracted as free functions or `pub(crate)` functions within the same module.
- Tools compose each other by calling their `execute` functions directly — they do not invoke the MCP dispatch layer. Examples:
  - `validate_projection::execute_single` calls `inspect_projection::execute` and `render_projection::reconstruct_service_def` directly.
  - `projection_coverage::execute` calls `list_models::execute` and `list_projections::execute` directly.
  - `application_info::execute` calls `list_resources::execute`, `list_policies::execute`, `list_rate_limiters::execute`, and `list_broadcast_channels::execute` directly.
- `json_ui_verify_action` exposes `find_handler` as `pub(crate)` — a pure, testable inner function that takes pre-fetched route data, plus the `async execute` wrapper that fetches the route list first. This is the cleanest form of the pattern.

### Recommended dispatch pattern for `checkpoint_projection`

Call the `execute` functions of existing tools directly. Do not extract logic upward into a shared module; do not re-implement any existing check.

The seam-dispatch table:

| Seam | Call | Signature to reuse |
|------|------|--------------------|
| 1 (well-formed) | `validate_projection::execute_single(project_root, name)` | returns `Result<ValidationResult, String>` |
| 2 (field→column) | owned by `checkpoint_projection` | new private fn `check_field_to_column(project_root, &service_def)` |
| 3 (action→route) | `json_ui_verify_action::find_handler(&routes, handler, method)` after fetching routes once with `list_routes::execute` | returns `VerifyActionResult` |
| 4 (rendered view) | `render_projection::execute(project_root, name, mode, intent_index)` | returns `Result<RenderResult, String>` |
| 4b (spec valid) | `json_ui_validate_spec::execute(spec_json)` on the spec from seam 4 | returns `ValidateResponse` |
| 5 (props→contract) | `validate_contracts::execute(project_root)` | returns `ContractValidationResult` |

Seam 3 optimization: call `list_routes::execute` once and reuse the `RouteInfo` slice across all `ActionDef` handlers via `find_handler`. This avoids re-reading routes per action and matches the existing `find_handler` design intent (`pub(crate)` pure function for exactly this case).

**Why call execute functions directly and not extract shared logic:**

Extracting validation logic into a shared module would split single-responsibility modules and risk accumulating unrelated checks in a utility layer. The design spec explicitly requires the checkpoint to "own only the field→column seam + aggregation." All other logic stays in the tools that already own it. The checkpoint is an orchestrator, not a validator.

**Seam 2 — field→column resolver:**

`projection_coverage::execute` already performs the `src/projections/` to `src/models/` name-match using `list_models::execute`. The field→column check reuses the same path: call `list_models::execute`, find the model whose name matches `service_def.name` (lowercased), then compare `service_def.fields[*].name` against the model's column list (`ModelDetails.fields[*].name` — same struct `list_models` already returns). When no model matches, report `not_checked` for seam 2, never `pass`.

`render_projection::reconstruct_service_def` is `pub(crate)` within `ferro-mcp` and can be called directly to get the `ServiceDef` from source before the seam walk begins.

---

## Question 2: Inline Hook Dependency Direction

The generators depend on the checkpoint; the checkpoint does not depend on the generators.

`generate_projection` and `json_ui_generate` are callers. After their generation logic completes, they call `checkpoint_projection::run_for(project_root, name)` and embed the returned `CheckpointVerdict` into their response under a `checkpoint` key.

This is exactly the pattern `application_info` uses for `list_resources`, `list_policies`, etc.: the aggregator calls the leaf tool, never the reverse.

**Concrete change in each generator:**

- `generate_projection::execute` — after building `GenerateProjectionResult`, call `checkpoint_projection::run_for(project_root, &result.model_name)` and add `checkpoint: Option<CheckpointVerdict>` to `GenerateProjectionResult`. Set to `None` if the projection file does not yet exist on disk when the tool runs.

- `json_ui_generate::execute` — same shape: after assembling `JsonUiGenerationContext`, call `run_for` if a `service_name` is provided as context, embed as `checkpoint: Option<CheckpointVerdict>`.

**Why not the reverse:**

The checkpoint is a read-only verifier. Having it call generators would introduce a side effect and create a generate-then-verify cycle where the checkpoint's output depends on generation having just occurred. Tools that produce artifacts are responsible for closing their own loop by running the check after producing.

---

## Question 3: Status Surfacing in `application_info` and `projection_coverage`

Both tools are read-only consumers. The pattern mirrors `application_info::scan_feature_counts`, which calls multiple tool execute functions and assembles a summary struct from their results.

### `projection_coverage`

`projection_coverage::ModelCoverage` gains one new field:

```rust
pub checkpoint_status: Option<CheckpointStatus>,
```

`CheckpointStatus` is a new type in `checkpoint_projection`:

```rust
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointStatus {
    Unverified,  // no checkpoint has ever run for this projection
    Failing,     // last run had at least one seam fail
    Clean,       // last run: all checked seams pass
}
```

`projection_coverage::execute` calls `checkpoint_projection::last_status(project_root, &proj.name)` for each covered projection. `last_status` reads from a per-projection status file at `{project_root}/.ferro/checkpoints/{name}.json` (runtime artifact, gitignored). If absent, returns `Unverified`.

### `application_info`

`ApplicationInfo` gains one new field:

```rust
pub projection_checkpoint: ProjectionCheckpointSummary,
```

```rust
#[derive(Debug, Serialize)]
pub struct ProjectionCheckpointSummary {
    pub total_projections: usize,
    pub clean: usize,
    pub failing: usize,
    pub unverified: usize,
}
```

Populated by iterating over `list_projections::execute` results and calling `checkpoint_projection::last_status` per projection. Direct parallel to how `scan_feature_counts` already aggregates counts from multiple list tools.

---

## Question 4: Build Order Across 3 Phases

### Phase 1 (P1): Tool + field→column seam + aggregation

**Goal:** `checkpoint_projection` callable standalone; seam 2 operational.

**New files:**
- `ferro-mcp/src/tools/checkpoint_projection.rs`

**Contents of the new module:**
- `pub struct CheckpointVerdict` — `status`, `projection`, `seams: Vec<SeamResult>`, `next_steps: Vec<String>`
- `pub struct SeamResult` — `seam: SeamName`, `status: SeamStatus`, `source: &'static str`, `findings: Vec<Finding>`
- `pub struct Finding` — `subject: String`, `detail: String`, `fix: String`
- `pub enum SeamStatus` — `Pass`, `Fail`, `Warn`, `NotChecked`
- `pub enum SeamName` — `WellFormed`, `FieldToColumn`, `ActionToRoute`, `RenderedView`, `PropsContract`
- `pub fn run_for(project_root: &Path, name: &str) -> CheckpointVerdict` — synchronous; see async boundary note below
- `pub enum CheckpointStatus` — `Unverified`, `Failing`, `Clean`
- `pub fn last_status(project_root: &Path, name: &str) -> CheckpointStatus`
- `fn check_field_to_column(project_root: &Path, service: &ServiceDef) -> SeamResult` (private)
- Aggregation logic: `status` = `Fail` if any seam fails, `Warn` if only warns, `Pass` if all checked seams pass. Unchecked seams do not raise status.
- `next_steps` ranking: failures from earlier seams first, warnings after.
- Status cache write: `.ferro/checkpoints/{name}.json` written at end of `run_for`.

**Modified files:**
- `ferro-mcp/src/tools/mod.rs` — add `pub mod checkpoint_projection;`
- MCP tool dispatcher (wherever tools are registered as callable tools) — register `checkpoint_projection`

**Tests in P1:**
- `check_field_to_column` unit tests with temp fixtures: dangling field (seam 2 fail), clean slice (pass), absent model (not_checked)
- Aggregation tests for mixed seam results producing correct overall `status`
- Coverage-honesty: absent rendered view means seams 4/5 are `not_checked`, verdict is not `fail`

### Phase 2 (P2): Inline hook + status surfacing

**Goal:** Generators close the loop automatically; `application_info` and `projection_coverage` show checkpoint status.

**Modified files:**
- `ferro-mcp/src/tools/generate_projection.rs` — add `checkpoint: Option<CheckpointVerdict>` to `GenerateProjectionResult`, call `checkpoint_projection::run_for` after generation
- `ferro-mcp/src/tools/json_ui_generate.rs` — same inline hook
- `ferro-mcp/src/tools/projection_coverage.rs` — add `checkpoint_status: Option<CheckpointStatus>` to `ModelCoverage`, call `checkpoint_projection::last_status` per covered projection
- `ferro-mcp/src/tools/application_info.rs` — add `projection_checkpoint: ProjectionCheckpointSummary` to `ApplicationInfo`, populate from `list_projections` + `last_status`

**Tests in P2:**
- `generate_projection` result contains a `checkpoint` key
- `projection_coverage` per-model coverage includes `checkpoint_status`
- `application_info` summary correctly counts clean/failing/unverified

### Phase 3 (P3): Wrapper seams + dogfood acceptance

**Goal:** Seams 1, 3, 4, 4b, 5 report real results; dogfood gate passes.

P3 activates the dispatch calls for seams 1, 3, 4, 4b, 5 inside `run_for`. Each seam earns its place against dogfood results. A seam that does not catch a real defect in any real projection across the synthetic catalog and one live consumer may ship as `not_checked` rather than being forced active.

**Dogfood gate:** Run `checkpoint_projection` against all projections in `app/src/projections/` and against at least one gestiscilo consumer projection. At least one real seam defect must surface. If none surfaces with all seams active, the design spec requires the design to be revisited rather than shipped.

**Modified files (conditional):**
- `ferro-mcp/src/tools/checkpoint_projection.rs` — activate wrapper seam dispatches as dogfood justifies

---

## Component List: New vs Modified

### New

| Component | Path | Phase |
|-----------|------|-------|
| `checkpoint_projection` module | `ferro-mcp/src/tools/checkpoint_projection.rs` | P1 |
| `CheckpointVerdict`, `SeamResult`, `SeamStatus`, `SeamName`, `Finding` | inside the new module | P1 |
| `check_field_to_column` private fn | inside the new module | P1 |
| `CheckpointStatus` enum | inside the new module | P1 |
| `last_status` public fn | inside the new module | P1 |
| `.ferro/checkpoints/{name}.json` status cache | runtime artifact, gitignored | P1 write |

### Modified

| Component | Path | Change | Phase |
|-----------|------|--------|-------|
| `mod.rs` | `ferro-mcp/src/tools/mod.rs` | add `pub mod checkpoint_projection;` | P1 |
| MCP tool dispatcher | `ferro-mcp/src/lib.rs` or equivalent | register `checkpoint_projection` as callable tool | P1 |
| `GenerateProjectionResult` | `ferro-mcp/src/tools/generate_projection.rs` | add `checkpoint: Option<CheckpointVerdict>` field | P2 |
| `generate_projection::execute` | same file | call `run_for` after generation | P2 |
| `JsonUiGenerationContext` | `ferro-mcp/src/tools/json_ui_generate.rs` | add `checkpoint: Option<CheckpointVerdict>` field | P2 |
| `json_ui_generate::execute` | same file | call `run_for` after generation | P2 |
| `ModelCoverage` | `ferro-mcp/src/tools/projection_coverage.rs` | add `checkpoint_status: Option<CheckpointStatus>` field | P2 |
| `projection_coverage::execute` | same file | call `last_status` per covered projection | P2 |
| `ApplicationInfo` | `ferro-mcp/src/tools/application_info.rs` | add `projection_checkpoint: ProjectionCheckpointSummary` field | P2 |
| `application_info::execute` | same file | populate summary from `list_projections` + `last_status` | P2 |
| `checkpoint_projection::run_for` | checkpoint module | activate wrapper seam dispatches as dogfood justifies | P3 |

---

## Data Flow: Checkpoint Verdict

```
checkpoint_projection::run_for(project_root, "Booking")
    │
    ├── validate_projection::execute_single(root, "Booking")
    │       → SeamResult { seam: WellFormed, ... }
    │
    ├── render_projection::reconstruct_service_def(name, display, content)
    │   then:
    ├── check_field_to_column(root, &service_def)
    │       uses list_models::execute + service_def.fields[*].name
    │       → SeamResult { seam: FieldToColumn, ... }
    │
    ├── list_routes::execute(root).await  [fetch once]
    │   for each ActionDef in service_def:
    │       json_ui_verify_action::find_handler(&routes, handler, method)
    │       → SeamResult { seam: ActionToRoute, ... }
    │
    ├── render_projection::execute(root, "Booking", None, None)
    │   if Ok → json_ui_validate_spec::execute(&serialized_spec)
    │       → SeamResult { seam: RenderedView, ... }
    │
    └── validate_contracts::execute(root)
            → SeamResult { seam: PropsContract, ... }
    │
    ▼
aggregate: status = worst of seam statuses (not_checked seams excluded)
           next_steps = failures first by seam order, then warnings
    │
    ├── write .ferro/checkpoints/Booking.json
    └── return CheckpointVerdict
```

---

## Async Boundary Note

`json_ui_verify_action::execute` is async (`async fn execute(...) -> Result<VerifyActionResult>`). All other tools in the projection stack are synchronous. Two options:

1. Make `run_for` synchronous, call `find_handler` with pre-fetched routes (routes fetched once by the MCP dispatcher before calling `run_for`). Signature becomes `run_for(project_root, name, routes: &[RouteInfo])` or `run_for_with_context(...)`. This keeps the module sync-consistent.

2. Make `run_for` async, matching the `json_ui_verify_action` pattern. Changes the calling signature in the generators and status consumers.

Option 1 is preferred because: (a) all other projection tools are sync, (b) `find_handler` is already `pub(crate)` specifically to allow reuse without the async wrapper, (c) the MCP dispatcher can own the single async fetch. If the dispatcher context makes option 2 unavoidable (e.g. the dispatcher itself is already async), option 2 is acceptable.

---

## Key Constraints

**No duplicate control surface.** The checkpoint owns exactly one piece of logic that exists nowhere else: `check_field_to_column`. All other seams delegate to the tool that already owns that check. This is a hard coherence rule.

**Dependency direction is fixed.** `checkpoint_projection` depends on: `validate_projection`, `json_ui_verify_action`, `render_projection`, `json_ui_validate_spec`, `validate_contracts`, `list_models`, `list_projections`, `list_routes`. None of those tools depend on `checkpoint_projection`. The generators and coverage tools depend on `checkpoint_projection` in P2. No reverse dependencies.

**Read-only and no-cargo.** The checkpoint does not invoke the compiler, does not write code, and does not modify any projection source. The `.ferro/checkpoints/` status cache is the only write side effect.

**Coverage honesty is structural.** `not_checked` must never be collapsed to `pass`. This is enforced by the `SeamStatus::NotChecked` variant being distinct from `Pass` in aggregation logic and in the output JSON.

---

*Architecture research for: ferro-mcp v12.5 Projection Checkpoint*
*Researched: 2026-06-09*
