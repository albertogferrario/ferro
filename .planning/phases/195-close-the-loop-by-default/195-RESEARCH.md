# Phase 195: Close the Loop by Default — Research

**Researched:** 2026-06-10
**Domain:** ferro-mcp checkpoint pipeline wiring, async/sync dispatch, seam-name reconciliation
**Confidence:** HIGH — all findings from direct codebase reads; no external docs needed

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01** Seam-name reconciliation: correct four stubs to canonical names in `run_for`, tests (~lines 851-933), and docs.

| Seam | Canonical name | source (validator) | 194 wrong name |
|------|--------------|--------------------|----------------|
| 1 | `projection_well_formed` | `validate_projection` | `schema_load` |
| 2 | `field_to_column` | `checkpoint` (already correct) | — |
| 3 | `action_to_route` | `json_ui_verify_action` | `field_type_compat` |
| 4 | `rendered_view` | `render_projection` + `json_ui_validate_spec` | `action_binding` |
| 5 | `props_to_contract` | `validate_contracts` | `render_target` |

**D-02** Seam 1 calls `validate_projection::execute_single(project_root, name)` → `ValidationResult`; normalizes `errors/warnings` → `Finding`; `source: "validate_projection"`.

**D-03** Seam 3 dispatches to `json_ui_verify_action`; `source: "json_ui_verify_action"`; no route-matching logic duplicated.

**D-04** Seam 4 calls `render_projection::execute(...)` then feeds rendered spec JSON to `json_ui_validate_spec::execute(spec_json)`; `source: "render_projection"` for render failures, `source: "json_ui_validate_spec"` for spec validation findings.

**D-05** Seam 5 calls `validate_contracts::execute(project_root, route_filter)` scoped to this projection's routes; `source: "validate_contracts"`.

**D-06** Seam cascade: seam 1 fail → seams 4 and 5 `not_checked(reason: "seam_1_failed")`; seam 4 fail → seam 5 `not_checked(reason: "seam_4_failed")`; seams 2 and 3 run independently.

**D-07** Inline hook: `generate_projection` and `json_ui_generate` call `checkpoint_projection::run_for` after generating, embed result under `checkpoint` key; one-way dependency.

**D-08** Embedded value is a `VerdictSummary` — NOT the raw `Verdict.seams` array. Must contain top-level `status`. SC-1 forbids five `not_checked` entries. Exact layout is Claude's discretion within those constraints.

**D-09** `projection_coverage::ModelCoverage` gets a `checkpoint_status` field read from `.ferro/checkpoints/{projection_name}.json`; missing file → `"unverified"`.

**D-10** `application_info::ApplicationInfo` gets a `projection_checkpoint` summary `{ total_projections, clean, failing, unverified }`.

**D-11** Ambient consumers read cache only — never call `run_for`/recompute.

### Claude's Discretion

- Exact field layout of `VerdictSummary` (within D-08 constraints: top-level `status`, not raw seams array).
- Whether seam 3 gets a thin `execute` wrapper vs per-action `find_handler` calls (research-led; resolved below).
- Cache-read helper location: a shared `read_ambient_status(project_root, name) -> &'static str` in `checkpoint_projection.rs` consumed by both ambient tools is recommended.

### Deferred Ideas (OUT OF SCOPE)

Dogfood acceptance, poisoned synthetic fixture, zero-finding-seam demotion, `next_steps` cap 10→5 — all Phase 196.

IN-02 (surface unrecognized DataType in D-06 warn subject) — fold in only if Phase 195 touches that path.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CHK-07 | `generate_projection` and `json_ui_generate` return the checkpoint verdict inline after generating. | `generate_projection::execute` is sync, returns `GenerateProjectionResult` (line 85). `json_ui_generate::execute` is sync, returns `JsonUiGenerationContext`. Both are called from async service handlers. Adding a `checkpoint: Option<VerdictSummary>` field to each result struct is the lowest-friction path. |
| CHK-08 | `application_info` and `projection_coverage` surface per-projection checkpoint status as read-only cache consumers. | `ModelCoverage` (projection_coverage.rs:22) and `ApplicationInfo` (application_info.rs:12) are both sync structs. Cache file at `.ferro/checkpoints/{name}.json` contains `ambient_status: "clean"\|"failing"` plus full verdict. |
| CHK-09 | Seams 1, 3, 4, 5 dispatch to existing validators; each finding's `source` names the producing validator. | All four validators confirmed read. Seam 3 async concern resolved — use `find_handler` (sync) + pre-loaded route list. |
</phase_requirements>

---

## Summary

Phase 195 wires four stub seams in `checkpoint_projection::run_for` to their respective validators, adds inline checkpoint summaries to the two generator tools, and surfaces per-projection status in `projection_coverage` and `application_info`. All work is inside `ferro-mcp/src/tools/`; zero new crate dependencies are required.

The highest-risk decision is the async mismatch for seam 3: `json_ui_verify_action::execute` is async (it calls `list_routes::execute` which is async), but `run_for` is currently synchronous. The resolution is to use the existing sync `find_handler` helper after pre-loading routes via a `tokio::task::block_in_place` shim OR by making `run_for` async. The analysis below recommends making `run_for` and `execute` async — the service.rs handler is already `async fn`, so the change is mechanical and propagates cleanly.

The second key finding is on the json_ui_generate anchor: `json_ui_generate::execute` takes an `Option<&str> model` parameter but returns a generation *context* (component catalog, routes, conventions), not a named projection. There is no projection function name to anchor a checkpoint on. The recommended resolution is to skip the inline checkpoint when no `model` parameter is supplied and to attempt a name-derived anchor (`{model_lowercase}_service`) when `model` is Some — calling `checkpoint_projection::run_for` speculatively and embedding a `VerdictSummary` if it resolves, or emitting `None` if it does not.

**Primary recommendation:** Make `run_for` async. Implement seam 3 using `find_handler` (the existing sync pure helper) after loading routes in an async step at the top of `run_for`. This keeps the seam logic itself synchronous while satisfying the async requirement without a blocking thread.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Seam dispatch + normalization | `checkpoint_projection.rs` | target validators | Checkpoint owns orchestration; validators own logic |
| Route list loading (seam 3) | `checkpoint_projection::run_for` (async) | `list_routes::execute` | Route loading is a one-time async I/O step; `find_handler` is sync |
| Rendered spec extraction (seam 4) | `render_projection::execute` (sync) | `json_ui_validate_spec::execute` (sync) | Both sync; no new async requirement |
| Contract validation (seam 5) | `validate_contracts::execute` (sync) | — | Sync; scoped via `route_filter` param |
| Inline checkpoint summary | `generate_projection.rs` / `json_ui_generate.rs` | `checkpoint_projection::run_for` | Generators call checkpoint; not the reverse |
| Ambient status reads | `projection_coverage.rs` / `application_info.rs` | cache files only | Read-only, stale-ok; no call to `run_for` |
| `VerdictSummary` type | `checkpoint_projection.rs` | — | Shared type consumed by both generators |
| Cache read helper | `checkpoint_projection.rs` | — | One `pub(crate) fn read_ambient_status(...)` consumed by two callers |

---

## Research Flag Resolutions (HIGHEST PRIORITY)

### RF-1: Seam 3 dispatch — `action_to_route` via `json_ui_verify_action`

**Finding:** `json_ui_verify_action` exposes two entry points:

- `pub async fn execute(project_root, handler, method) -> Result<VerifyActionResult>` (line 37): loads routes via `list_routes::execute` (async), then delegates to `find_handler`.
- `pub(crate) fn find_handler(routes: &[RouteInfo], handler: &str, method: Option<&str>) -> VerifyActionResult` (line 52): pure sync lookup, takes a pre-loaded slice.

`run_for` is currently `pub(crate) fn run_for(...) -> Result<Verdict, String>` — synchronous. Calling `execute` directly from `run_for` requires either a runtime block (`tokio::task::block_in_place`) or making `run_for` async.

**Recommendation: make `run_for` and `execute` async.**

The service.rs handler `checkpoint_projection` is already `pub async fn`. Making `run_for` async is a mechanical change: add `async`, add `.await` at the route-loading call, keep everything else the same. The existing sync tests become `#[tokio::test]` (the test suite already has `#[tokio::test]` in `json_ui_verify_action.rs`, so tokio is a dev-dep). No new dependencies.

**Concrete seam 3 implementation:**

```rust
// At the top of run_for (once, not per-action):
let routes = match list_routes::execute(project_root).await {
    Ok(info) => info.routes,
    Err(_) => vec![],   // seam 3 falls back to not_checked per-action
};

// In the action_to_route seam:
fn action_to_route_seam(service: &ServiceDef, routes: &[RouteInfo]) -> SeamResult {
    let mut findings = vec![];
    for action in &service.actions {
        let result = json_ui_verify_action::find_handler(routes, &action.name, None);
        if !result.found {
            findings.push(Finding {
                subject: action.name.clone(),
                detail: format!("action '{}' has no registered route", action.name),
                fix: format!(
                    "register a route for handler '{}'{}",
                    action.name,
                    result.candidate.as_ref()
                        .map(|c| format!("; closest match: '{c}'"))
                        .unwrap_or_default()
                ),
            });
        }
    }
    SeamResult {
        seam: "action_to_route".to_string(),
        status: if findings.is_empty() { SeamStatus::Pass } else { SeamStatus::Fail },
        source: "json_ui_verify_action".to_string(),
        findings,
        reason: None,
    }
}
```

If `list_routes::execute` returns `Err` and we use an empty slice, every action will appear not-found. Better: on `Err`, return a single `not_checked` result with `reason: "route_list_unavailable"`.

**The `ServiceDef.actions` field** is `Vec<ActionDef>`; `ActionDef.name` is the handler name string (e.g., `"submit_order"`). This is exactly what `find_handler` matches against `RouteInfo.name` (the `Option<String>` named-route identifier set via `.name("...")`). If actions have no matching named routes, `found: false`. [VERIFIED: ferro-projections/src/action.rs, ferro-mcp/src/tools/list_routes.rs, ferro-mcp/src/tools/json_ui_verify_action.rs]

**Alternative (if async is unacceptable):** expose `pub(crate) fn find_handler` from `json_ui_verify_action` as `pub` and call it with a synchronously-loaded route slice via `tokio::task::block_in_place`. This is more complex and not recommended.

---

### RF-2: `json_ui_generate` inline checkpoint anchor

**Finding:** `json_ui_generate::execute(project_root, model: Option<&str>, description: Option<&str>) -> JsonUiGenerationContext` (line 104). The function does not generate or name a projection; it assembles a generation *context* (catalog, models, routes, conventions). There is no projection function name present in the return type or any intermediate step.

**Resolution:** Use a name-derived speculative anchor.

When `model` is `Some("Booking")`, the canonical projection function name (per ferro conventions) is `booking_service`. Attempt `checkpoint_projection::run_for(project_root, "booking_service", now).await`:
- If it returns `Ok(verdict)` → embed `VerdictSummary` under `checkpoint`.
- If it returns `Err` (projection not found or name invalid) → embed `None`; omit the field (using `#[serde(skip_serializing_if = "Option::is_none")]`).

When `model` is `None` → skip the checkpoint entirely; do not embed a vacuous all-`not_checked` summary (SC-1).

**Derived projection name convention:** `{model.to_lowercase()}_service`. This matches the naming convention used throughout the codebase (e.g., `Booking` → `booking_service`, `User` → `user_service`). [VERIFIED: checkpoint_projection.rs tests, inspection of projection fixture names]

**Why not skip always:** The design spec (D-07) says both generators embed the checkpoint. When a model is named, the speculative anchor is actionable. When no model is named (context-only call), there is nothing to anchor on.

---

## Seam 1: `projection_well_formed` via `validate_projection`

**Entry point:** `validate_projection::execute_single(project_root: &Path, name: &str) -> Result<ValidationResult, String>` (line 34). [VERIFIED: validate_projection.rs]

**`ValidationResult` shape:**
```rust
pub struct ValidationResult {
    pub service_name: String,
    pub file: String,
    pub warnings: Vec<String>,   // format!("{:?}", warning) strings
    pub errors: Vec<String>,     // error.to_string() strings
    pub valid: bool,
}
```
[VERIFIED: validate_projection.rs lines 15-21]

**Normalization to `Finding`:**
- Each `errors[i]` → `Finding { subject: service_name.clone(), detail: errors[i].clone(), fix: "fix the structural error in the projection source".to_string() }`
- Each `warnings[i]` → same but as a warning-severity finding
- `valid: false` → seam status `Fail`; `valid: true` with warnings → seam status `Warn`; `valid: true` no warnings → `Pass`
- `source: "validate_projection"`

**Seam 1 fail → cascade:** If seam 1 fails (i.e., `ValidationResult.valid == false`), both seam 4 and seam 5 must be `not_checked` with `reason: "seam_1_failed"`. Seam 1 fail does NOT block seams 2 or 3.

**Important:** `execute_single` calls `inspect_projection` and `reconstruct_service_def` internally. If the projection is found (the checkpoint already found it in step 1 of `run_for`), this will succeed. `execute_single` accepts the projection *function name* (same as `run_for`'s `name` param) — no adaptation needed.

---

## Seam 3: `action_to_route` via `json_ui_verify_action`

Covered in RF-1 above. Summary:
- Pre-load `routes` via `list_routes::execute(project_root).await` once at the top of async `run_for`.
- For each `service.actions` action, call `find_handler(routes, &action.name, None)`.
- `source: "json_ui_verify_action"` on every finding.
- Empty `service.actions` → `Pass` with no findings (correct: nothing to check).
- Route load failure → single `not_checked` with `reason: "route_list_unavailable"`.

---

## Seam 4: `rendered_view` via `render_projection` + `json_ui_validate_spec`

**render_projection entry:** `render_projection::execute(project_root, name, mode: Option<&str>, intent_index: Option<usize>) -> Result<RenderResult, String>` (line 36). [VERIFIED: render_projection.rs]

**`RenderResult` shape:**
```rust
pub struct RenderResult {
    pub service_name: String,
    pub intent: String,
    pub confidence: f64,
    pub mode: String,
    pub json_ui: serde_json::Value,   // the rendered spec
    pub all_intents: Vec<IntentInfo>,
}
```
[VERIFIED: render_projection.rs lines 16-25]

**json_ui_validate_spec entry:** `json_ui_validate_spec::execute(spec_json: &str) -> ValidateResponse` (line 43). [VERIFIED: json_ui_validate_spec.rs]

**`ValidateResponse` shape:**
```rust
pub struct ValidateResponse {
    pub valid: bool,
    pub structural_errors: Vec<String>,
    pub catalog_errors: Vec<String>,
    pub warnings: Vec<String>,
}
```
[VERIFIED: json_ui_validate_spec.rs lines 23-35]

**Seam 4 pipeline:**
1. Call `render_projection::execute(project_root, name, None, None)`.
   - On `Err(e)` → `SeamResult { seam: "rendered_view", status: Fail, source: "render_projection", findings: [Finding { subject: name, detail: e, fix: "fix the projection before rendering" }] }`.
   - On `Ok(render)` → serialize `render.json_ui` to string, pass to step 2.
2. Call `json_ui_validate_spec::execute(&spec_json_string)`.
   - `structural_errors` → `Finding`s with `source: "json_ui_validate_spec"`, higher severity.
   - `catalog_errors` → `Finding`s with `source: "json_ui_validate_spec"`.
   - `valid: true` → `Pass`.
   - Any errors → `Fail`.

**render_projection::execute is synchronous** — no async required. [VERIFIED: render_projection.rs line 36]

**Cascade:** If seam 1 failed, seam 4 must return `not_checked(reason: "seam_1_failed")` without calling `render_projection::execute`.

---

## Seam 5: `props_to_contract` via `validate_contracts`

**Entry point:** `validate_contracts::execute(project_root: &Path, route_filter: Option<&str>) -> Result<ContractValidationResult>` (line 80). [VERIFIED: validate_contracts.rs]

**`ContractValidationResult` shape (relevant fields):**
```rust
pub struct ContractValidationResult {
    pub total_routes: usize,
    pub validated: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub validations: Vec<RouteValidation>,
    pub summary: Vec<String>,
}
pub struct RouteValidation {
    pub route: String,
    pub component: String,
    pub status: ValidationStatus,   // Passed | Failed | Skipped
    pub rust_props: Option<PropsInfo>,
    pub typescript_props: Option<PropsInfo>,
    pub mismatches: Vec<Mismatch>,
}
pub struct Mismatch {
    pub kind: MismatchKind,
    pub field: String,
    pub details: String,
}
```
[VERIFIED: validate_contracts.rs lines 17-78]

**`route_filter` scoping:** `validate_contracts::execute` applies the filter as: `!route.contains(filter) && !component.contains(filter)` (line 94-97). To scope to a single projection's routes, the route filter should be the projection's service name (lowercased) or the route path prefix. The exact value to pass is the projection `service_name` — e.g., `"booking"` will match any route path containing `"booking"`. This is a substring match, not exact. The seam should pass the service name as the filter and accept that it may include adjacent routes with the same substring.

**Normalization:**
- For each `RouteValidation` with `status: Failed`, for each `mismatch`:
  - `Finding { subject: format!("{}.{}", validation.route, mismatch.field), detail: mismatch.details.clone(), fix: "align Rust InertiaProps struct with TypeScript interface" }`
  - `source: "validate_contracts"`
- `Err(McpError::FileNotFound("src/routes.rs"))` → `not_checked(reason: "routes_file_missing")`.

**validate_contracts::execute is synchronous.** [VERIFIED: validate_contracts.rs line 80]

**Cascade:** If seam 4 failed, seam 5 must return `not_checked(reason: "seam_4_failed")`. If seam 1 failed, seam 5 must also return `not_checked(reason: "seam_1_failed")`.

---

## Inline Hook Architecture (CHK-07)

### `generate_projection` path

**Current execute signature:** `fn execute(project_root: &Path, model_name: &str) -> Result<GenerateProjectionResult, String>` (line 33). [VERIFIED: generate_projection.rs]

**`GenerateProjectionResult` fields (line 20-26):**
```rust
pub struct GenerateProjectionResult {
    pub model_name: String,
    pub service_def: serde_json::Value,
    pub intents: Vec<IntentInfo>,
    pub inferred_field_count: usize,
    pub manual_enrichment_needed: Vec<String>,
    // NEW: pub checkpoint: Option<VerdictSummary>,
}
```

**Anchor:** `generate_projection` takes `model_name` (e.g., `"Booking"`). The projection function name is `{model_name.to_lowercase()}_service`. Call `checkpoint_projection::run_for(project_root, &anchor_name, Utc::now()).await`, embed `Some(verdict.summary())` on Ok, `None` on Err. The `execute` function must become async if `run_for` becomes async (or it calls the async `run_for` via block).

**Service.rs impact:** `generate_projection` service handler is already `pub async fn`, so making `generate_projection::execute` async is a clean propagation.

### `json_ui_generate` path

**Current execute signature:** `fn execute(project_root: &Path, model: Option<&str>, description: Option<&str>) -> JsonUiGenerationContext` (line 104). [VERIFIED: json_ui_generate.rs]

**`JsonUiGenerationContext` fields (line 14-30):**
- Existing: `component_catalog`, `models`, `routes`, `existing_views`, `example`, `conventions`, `description: Option<String>`
- New: `checkpoint: Option<VerdictSummary>` (with `#[serde(skip_serializing_if = "Option::is_none")]`)

**Anchor:** When `model = Some("Booking")`, try `booking_service`. On Ok embed summary; on Err/None emit `None`. When `model = None`, emit `None`.

**service.rs impact:** `json_ui_generate` handler already `pub async fn`; `execute` becomes async.

### `VerdictSummary` type (D-08)

Defined in `checkpoint_projection.rs`, `pub` so generators can use it. Must satisfy:
- Top-level `status` field
- NOT the raw `seams: Vec<SeamResult>` array (SC-1)
- Derive `Serialize`, `JsonSchema`

**Recommended shape (Claude's discretion):**
```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct VerdictSummary {
    pub status: SeamStatus,
    /// Names of seams with `Fail` status.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fail_seams: Vec<String>,
    /// Names of seams with `Warn` status.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warn_seams: Vec<String>,
    /// Actionable next steps (capped, ranked, same as Verdict.next_steps).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub next_steps: Vec<String>,
}

impl Verdict {
    pub fn summary(&self) -> VerdictSummary {
        let fail_seams = self.seams.iter()
            .filter(|s| s.status == SeamStatus::Fail)
            .map(|s| s.seam.clone())
            .collect();
        let warn_seams = self.seams.iter()
            .filter(|s| s.status == SeamStatus::Warn)
            .map(|s| s.seam.clone())
            .collect();
        VerdictSummary {
            status: self.status.clone(),
            fail_seams,
            warn_seams,
            next_steps: self.next_steps.clone(),
        }
    }
}
```
This is small (3-5 fields), directly actionable, and consistent with the F11 signal-to-noise lesson cited in the spec.

---

## Ambient Status Surfacing (CHK-08)

### Cache file structure

`CacheEntry` written by Phase 194 `write_cache` (checkpoint_projection.rs lines 407-423):
```json
{
  "status": "pass",
  "projection": "booking_service",
  "seams": [...],
  "next_steps": [...],
  "ambient_status": "clean",
  "checked_at": "2026-06-10T12:00:00Z"
}
```
Path: `.ferro/checkpoints/{projection_function_name}.json`. [VERIFIED: checkpoint_projection.rs]

`ambient_status` is `"clean"` when `status == Pass`, otherwise `"failing"`. The third value `"unverified"` is reserved (see D-09/D-11) for projections with no cache file.

### Cache read helper (recommended)

```rust
// In checkpoint_projection.rs — pub(crate) so both ambient consumers can import it.
pub(crate) fn read_ambient_status(project_root: &Path, name: &str) -> &'static str {
    let path = project_root
        .join(".ferro")
        .join("checkpoints")
        .join(format!("{name}.json"));
    if !path.exists() {
        return "unverified";
    }
    let Ok(content) = fs::read_to_string(&path) else {
        return "unverified";
    };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) else {
        return "unverified";
    };
    match val.get("ambient_status").and_then(|v| v.as_str()) {
        Some("clean") => "clean",
        Some("failing") => "failing",
        _ => "unverified",
    }
}
```

### `projection_coverage::ModelCoverage` (D-09)

Current struct (projection_coverage.rs:22):
```rust
pub struct ModelCoverage {
    pub model_name: String,
    pub has_projection: bool,
    pub projection_name: Option<String>,   // function name, e.g. "booking_service"
    pub projection_file: Option<String>,
    pub primary_intent: Option<String>,
    pub intent_confidence: Option<f64>,
    pub suggestion: Option<String>,
    // NEW: pub checkpoint_status: String,  // "clean" | "failing" | "unverified"
}
```

Population:
- When `has_projection: true`: call `read_ambient_status(project_root, projection_name.as_deref().unwrap_or(""))`.
- When `has_projection: false`: `"unverified"` (no projection exists to checkpoint).

The key is `projection_name` (function name), matching the cache file name `{projection_name}.json`. [VERIFIED: projection_coverage.rs lines 22-37, 98-101]

### `application_info::ApplicationInfo` (D-10)

Current struct (application_info.rs:12):
```rust
pub struct ApplicationInfo {
    pub framework_version: String,
    pub rust_version: String,
    pub database_engine: Option<String>,
    pub environment: String,
    pub installed_crates: Vec<CrateInfo>,
    pub models: Vec<ModelInfo>,
    pub json_ui_views: JsonUiSpecsStatus,
    pub features: FeatureSummary,
    pub broadcasting: BroadcastingStatus,
    pub claude_code_skills: ClaudeCodeSkillsStatus,
    // NEW: pub projection_checkpoint: ProjectionCheckpointSummary,
}
```

New type:
```rust
#[derive(Debug, Serialize)]
pub struct ProjectionCheckpointSummary {
    pub total_projections: usize,
    pub clean: usize,
    pub failing: usize,
    pub unverified: usize,
}
```

Population: iterate projections via `list_projections::execute(project_root, None)`, for each projection call `read_ambient_status(project_root, &proj.name)`, tally into `clean/failing/unverified`. `total_projections = clean + failing + unverified`.

---

## SC-4 Anti-Reimplementation Guard

**Invariant:** `source == "checkpoint"` must appear ONLY on `field_to_column` findings. Every wrapper-seam finding must name its delegating validator in `source`.

**Enforcement test:**
```rust
#[test]
fn sc4_no_checkpoint_source_on_wrapper_seams() {
    // Build a Verdict with findings from all five seams.
    // Assert that for any seam != "field_to_column",
    // no finding has source == "checkpoint".
    let allowed_checkpoint_seam = "field_to_column";
    for seam in &verdict.seams {
        if seam.seam != allowed_checkpoint_seam {
            for finding in &seam.findings {
                // (SeamResult.source is on the SeamResult, not per-Finding)
                assert_ne!(
                    seam.source, "checkpoint",
                    "seam '{}' must not use source 'checkpoint'; use the delegating validator name",
                    seam.seam
                );
            }
        }
    }
}
```

Note: `source` lives on `SeamResult`, not on `Finding`. The check is: for every `SeamResult` where `seam != "field_to_column"`, `source != "checkpoint"`.

---

## Seam-Name Reconciliation (D-01)

**Edit sites:**

1. **`checkpoint_projection.rs` lines ~144-171** — the four stubs: rename `seam` field strings.
   - `"schema_load"` → `"projection_well_formed"`
   - `"field_type_compat"` → `"action_to_route"`
   - `"action_binding"` → `"rendered_view"`
   - `"render_target"` → `"props_to_contract"`

2. **Tests lines ~851-933** — `make_seam("schema_load", ...)` etc: update string literals.
   Specific tests affected:
   - `aggregate_status_fail_wins_over_not_checked` (line ~849): uses `"schema_load"`
   - `aggregate_status_warn_not_checked` (line ~857): uses `"schema_load"`
   - `aggregate_status_pass_not_checked` (line ~866): uses `"schema_load"`
   - `aggregate_status_all_not_checked_is_pass` (line ~875): uses `"schema_load"`, `"action_binding"`
   - `next_steps_ranked_deduped` (line ~887): uses `"schema_load"`, `"field_to_column"`
   - `next_steps_dedup` (line ~924): uses `"field_to_column"`, `"action_binding"`

3. **`docs/src/agents/checkpoint-projection.md` line ~42** — the seam example block shows `"schema_load"` as a `not_checked` seam. Update to `"projection_well_formed"` and `source` from `"checkpoint"` to `"validate_projection"` (since Phase 195 wires it). Also update the `source` description in the SeamResult table (line ~78): "always `"checkpoint"` in this version" → remove that caveat.

---

## Async Architecture Decision: Making `run_for` Async

**Current:** `pub(crate) fn run_for(project_root: &Path, name: &str, now: DateTime<Utc>) -> Result<Verdict, String>`

**After Phase 195:** `pub(crate) async fn run_for(project_root: &Path, name: &str, now: DateTime<Utc>) -> Result<Verdict, String>`

**Propagation chain:**
- `execute` → `pub async fn execute(...)` (calls `run_for(...).await`)
- `service.rs` checkpoint handler already `pub async fn` — no change needed
- `generate_projection::execute` → becomes `pub async fn` (calls `run_for.await`)
- `service.rs` generate_projection handler already `pub async fn` — no change
- `json_ui_generate::execute` → becomes `pub async fn` (calls `run_for.await`)
- `service.rs` json_ui_generate handler already `pub async fn` — no change

**Test impact:** All existing `#[test]` in `checkpoint_projection.rs` that call `run_for` must become `#[tokio::test]`. Since `tokio` is already a dev-dep (evidenced by `json_ui_verify_action.rs` line 146 `#[tokio::test]`), this is a matter of adding the attribute.

Tests that call pure helper functions (`field_to_column_seam`, `count_column_backed_builders`, `aggregate_status`, `aggregate_next_steps`, `write_cache`) remain synchronous `#[test]` — no change needed for those.

---

## Common Pitfalls

### Pitfall 1: Calling async from sync with `block_on`
**What goes wrong:** Adding `tokio::runtime::Handle::current().block_on(...)` inside `run_for` to call `list_routes::execute` or `json_ui_verify_action::execute` — this panics in an async context because tokio prohibits nested blocking.
**Prevention:** Make `run_for` async. The propagation is mechanical and the service.rs handler is already async.

### Pitfall 2: Using `seam.source` instead of `seam.seam` for the SC-4 guard
**What goes wrong:** Writing the SC-4 test to assert on `Finding.source` — but `source` is a field on `SeamResult`, not on `Finding`.
**Prevention:** The test must assert `seam_result.source != "checkpoint"` for wrapper seams, not `finding.source`.

### Pitfall 3: Embedding vacuous all-`not_checked` summary in `json_ui_generate`
**What goes wrong:** When no `model` param is given, attempting `run_for` with a guessed name fails and embeds an empty/noise summary.
**Prevention:** Skip the inline checkpoint when `model = None`. Use `#[serde(skip_serializing_if = "Option::is_none")]` on `checkpoint: Option<VerdictSummary>`.

### Pitfall 4: Using the wrong key for ambient reads in `projection_coverage`
**What goes wrong:** Keying the cache read on `ModelCoverage.model_name` ("Booking") instead of `ModelCoverage.projection_name` ("booking_service").
**Prevention:** The cache file is `{projection_function_name}.json`, not `{model_name}.json`. Use `projection_name` field.

### Pitfall 5: Seam-name mismatch in test fixtures
**What goes wrong:** Updating stubs in `run_for` to canonical names but missing the test fixtures that hard-code the old names (`"schema_load"`, `"action_binding"`, `"render_target"`, `"field_type_compat"`).
**Prevention:** Grep for all four wrong names before committing: `grep -rn "schema_load\|field_type_compat\|action_binding\|render_target" ferro-mcp/src/`.

### Pitfall 6: `route_filter` in seam 5 substring-matching too broadly
**What goes wrong:** Passing `service_name = "order"` as filter catches every route containing "order" including `/reorder`, `/order_items`, etc., inflating findings.
**Prevention:** Document and test that the filter is a substring match, not an exact match. The seam should note this in finding provenance. For Phase 195 this is acceptable; exact scoping can be tightened in Phase 196.

### Pitfall 7: docs example block not updated
**What goes wrong:** `docs/src/agents/checkpoint-projection.md` still shows `"schema_load"` and `source: "checkpoint"` after Phase 195 wires the seams.
**Prevention:** Update the example block in the same commit that wires the seams. The docs line ~42 JSON snippet must use `"projection_well_formed"` with `source: "validate_projection"`.

---

## Architecture Patterns

### Normalization function pattern (from Phase 194 D-08)

Each wrapper seam has a private normalization function:
```
fn normalize_<seam_name>(result: ValidatorOutput) -> SeamResult { ... }
```
These live at module level in `checkpoint_projection.rs`, below the seam dispatch functions.

### Seam cascade pattern

```
seam1 = run_seam_1(...)
seam4 = if seam1.status == Fail {
    not_checked("seam_1_failed")
} else {
    run_seam_4(...)
}
seam5 = if seam1.status == Fail {
    not_checked("seam_1_failed")
} else if seam4.status == Fail {
    not_checked("seam_4_failed")
} else {
    run_seam_5(...)
}
```
Seams 2 and 3 always run (independent).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Route lookup | Custom route-file parser in checkpoint | `find_handler(routes, name, None)` from `json_ui_verify_action` | Already handles named-route lookup + Levenshtein suggestion |
| Spec validation | Inline spec structural checks | `json_ui_validate_spec::execute(spec_json)` | Wraps `Spec::from_json` + `Catalog::validate` — the exact server-startup pipeline |
| Contract mismatch detection | Custom Rust/TS prop comparison | `validate_contracts::execute(project_root, filter)` | Full props extraction + field-level mismatch categorization |
| ServiceDef validation | Repeat of validate_projection logic | `validate_projection::execute_single(root, name)` | Owns `service.validate()` round-trip |

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test + `tokio` for async tests |
| Config file | None — uses `cargo test -p ferro-mcp` |
| Quick run command | `cargo test -p ferro-mcp checkpoint_projection` |
| Full suite command | `cargo test -p ferro-mcp` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CHK-07 | `generate_projection` embeds `checkpoint` field in response | unit | `cargo test -p ferro-mcp generate_projection` | ❌ Wave 0 |
| CHK-07 | `json_ui_generate` embeds `checkpoint: None` when no model | unit | `cargo test -p ferro-mcp json_ui_generate` | ❌ Wave 0 |
| CHK-07 | `VerdictSummary` serializes without raw seam array | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::verdict_summary_shape` | ❌ Wave 0 |
| CHK-08 | `read_ambient_status` returns `"unverified"` for missing file | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::ambient_missing_unverified` | ❌ Wave 0 |
| CHK-08 | `read_ambient_status` returns `"clean"` / `"failing"` from cache | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::ambient_read_clean` | ❌ Wave 0 |
| CHK-08 | `projection_coverage` report includes `checkpoint_status` field | unit | `cargo test -p ferro-mcp projection_coverage` | ❌ Wave 0 |
| CHK-08 | `application_info` includes `projection_checkpoint` summary | unit | `cargo test -p ferro-mcp application_info` | ❌ Wave 0 |
| CHK-09 | Seam 1 `projection_well_formed` dispatches to `validate_projection`; `source == "validate_projection"` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam1_source_provenance` | ❌ Wave 0 |
| CHK-09 | Seam 3 `action_to_route` dispatches to `find_handler`; `source == "json_ui_verify_action"` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam3_source_provenance` | ❌ Wave 0 |
| CHK-09 | Seam 4 `rendered_view` dispatches to render + validate; correct `source` per stage | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam4_source_provenance` | ❌ Wave 0 |
| CHK-09 | Seam 5 `props_to_contract` dispatches to `validate_contracts`; `source == "validate_contracts"` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam5_source_provenance` | ❌ Wave 0 |
| CHK-09 | SC-4: `source == "checkpoint"` appears only on `field_to_column` seam | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::sc4_no_checkpoint_source_on_wrapper_seams` | ❌ Wave 0 |
| D-01 | Seam names in output use canonical names (not old Phase 194 names) | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::seam_names_canonical` | ❌ Wave 0 |
| D-06 | Seam cascade: seam 1 fail → seams 4 and 5 `not_checked` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::cascade_seam1_fail` | ❌ Wave 0 |
| D-06 | Seam cascade: seam 4 fail → seam 5 `not_checked` | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::cascade_seam4_fail` | ❌ Wave 0 |
| D-06 | Seams 2 and 3 run independently of seam 1 failure | unit | `cargo test -p ferro-mcp checkpoint_projection::tests::cascade_seams_2_3_independent` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-mcp checkpoint_projection`
- **Per wave merge:** `cargo test -p ferro-mcp`
- **Phase gate:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test -p ferro-mcp` (scoped per project thermal policy)

### Wave 0 Gaps

All test functions listed above are new. The existing test infrastructure in `checkpoint_projection.rs` covers Phase 194 behavior. Phase 195 adds a new test module section covering:

- [ ] `verdict_summary_shape` — `VerdictSummary` serializes to `{ status, fail_seams?, warn_seams?, next_steps? }` without `seams` array
- [ ] `ambient_missing_unverified` — `read_ambient_status` with no cache file → `"unverified"`
- [ ] `ambient_read_clean` / `ambient_read_failing` — `read_ambient_status` with a written cache → correct value
- [ ] `seam1_source_provenance` — seam 1 result has `source == "validate_projection"`
- [ ] `seam3_source_provenance` — seam 3 result has `source == "json_ui_verify_action"`
- [ ] `seam4_source_provenance` — seam 4 result has correct `source` per render vs spec error
- [ ] `seam5_source_provenance` — seam 5 result has `source == "validate_contracts"`
- [ ] `sc4_no_checkpoint_source_on_wrapper_seams` — SC-4 invariant, the mechanic anti-reimplementation guard
- [ ] `seam_names_canonical` — asserts no seam in a fresh run has old names
- [ ] `cascade_seam1_fail` — stub a failing seam 1, assert seams 4+5 are `not_checked`
- [ ] `cascade_seam4_fail` — stub a failing seam 4, assert seam 5 is `not_checked`
- [ ] `cascade_seams_2_3_independent` — seam 1 fails, seams 2 and 3 still run
- [ ] Tests for inline checkpoint in `generate_projection.rs` and `json_ui_generate.rs`
- [ ] Tests for `projection_coverage::ModelCoverage.checkpoint_status` field serialization
- [ ] Tests for `application_info::ProjectionCheckpointSummary` population

*(Existing Phase 194 tests remain unchanged and continue to pass.)*

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `{model_name.to_lowercase()}_service` is the canonical projection function name convention | RF-2, json_ui_generate anchor | Wrong anchor → `run_for` always returns Err → `checkpoint: None` always embedded (safe degradation, not broken) |
| A2 | `route_filter` substring match in `validate_contracts` is acceptable for Phase 195 | Seam 5 | Overly broad filter inflates findings; no false negatives, only false positives |

---

## Environment Availability

Step 2.6: SKIPPED — Phase 195 is code-only within `ferro-mcp`. Zero new external tool dependencies. All validators called are already compiled into the same crate.

---

## Sources

### Primary (HIGH confidence — verified from codebase reads)

- `ferro-mcp/src/tools/checkpoint_projection.rs` — output types, `run_for`, stubs, cache write, existing tests
- `ferro-mcp/src/tools/json_ui_verify_action.rs` — `execute` (async), `find_handler` (sync)
- `ferro-mcp/src/tools/validate_projection.rs` — `execute_single`, `ValidationResult` shape
- `ferro-mcp/src/tools/render_projection.rs` — `execute` (sync), `RenderResult` shape
- `ferro-mcp/src/tools/json_ui_validate_spec.rs` — `execute` (sync), `ValidateResponse` shape
- `ferro-mcp/src/tools/validate_contracts.rs` — `execute` (sync), `ContractValidationResult`/`RouteValidation` shapes
- `ferro-mcp/src/tools/generate_projection.rs` — `execute` (sync), `GenerateProjectionResult` shape
- `ferro-mcp/src/tools/json_ui_generate.rs` — `execute` (sync), `JsonUiGenerationContext` shape
- `ferro-mcp/src/tools/projection_coverage.rs` — `ModelCoverage` struct, projection name key
- `ferro-mcp/src/tools/application_info.rs` — `ApplicationInfo` struct
- `ferro-mcp/src/service.rs` — all handler signatures (all `pub async fn`)
- `ferro-projections/src/action.rs` — `ActionDef` shape (name field, no handler/route field)
- `.planning/phases/195-close-the-loop-by-default/195-CONTEXT.md` — locked decisions
- `docs/superpowers/specs/2026-06-09-projection-checkpoint-design.md` — design spec
- `docs/src/agents/checkpoint-projection.md` — current docs with old seam names

---

## Metadata

**Confidence breakdown:**
- Seam dispatch signatures: HIGH — all entry points read directly
- Async/sync architecture: HIGH — all function signatures verified
- RF-1 resolution: HIGH — verified `find_handler` is `pub(crate)` and sync; route list is async
- RF-2 resolution: MEDIUM — anchor derivation convention confirmed from test fixtures; `run_for` behavior on missing projection confirmed from existing tests
- Normalization shapes: HIGH — all result structs read directly
- Test coverage gaps: HIGH — existing test infrastructure read; new tests enumerated from requirements

**Research date:** 2026-06-10
**Valid until:** 2026-07-10 (stable internal codebase; all dependencies within the same workspace)
