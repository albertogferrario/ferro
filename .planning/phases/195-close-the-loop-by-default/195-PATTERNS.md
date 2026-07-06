# Phase 195: Close the Loop by Default — Pattern Map

**Mapped:** 2026-06-10
**Files analyzed:** 7 modified files
**Analogs found:** 7 / 7

---

## File Classification

| Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------|------|-----------|----------------|---------------|
| `ferro-mcp/src/tools/checkpoint_projection.rs` | service + orchestrator | request-response + CRUD | self (Phase 194 output) | exact — extend in place |
| `ferro-mcp/src/tools/generate_projection.rs` | service | request-response | `generate_projection.rs` itself + `application_info.rs` (nested sub-struct) | exact |
| `ferro-mcp/src/tools/json_ui_generate.rs` | service | request-response | `json_ui_generate.rs` itself + `generate_projection.rs` (Optional field pattern) | exact |
| `ferro-mcp/src/tools/projection_coverage.rs` | service | CRUD + file-I/O | `projection_coverage.rs` itself + `application_info.rs` (sub-summary struct) | exact |
| `ferro-mcp/src/tools/application_info.rs` | service | CRUD + file-I/O | `application_info.rs` itself (BroadcastingStatus / ClaudeCodeSkillsStatus patterns) | exact |
| `ferro-mcp/src/service.rs` | middleware / wiring | request-response | `service.rs` itself (existing async handler patterns) | exact |
| `docs/src/agents/checkpoint-projection.md` | docs | — | the seam example block in the same file | exact |

---

## Pattern Assignments

### `ferro-mcp/src/tools/checkpoint_projection.rs` — wrapper seams + cascade + VerdictSummary + read_ambient_status

**Analog:** The file itself (Phase 194 output, lines 1–437).

#### Stub block to replace (lines 139–171)

The four stubs that Phase 195 replaces are at lines 144–171 of the current file. The surrounding structure to mirror when replacing them:

```rust
// Lines 131–188 — current run_for body structure
// Step 3 calls field_to_column_seam (sync, self-contained seam function)
let seam2 = field_to_column_seam(
    project_root,
    &detail.service_name,
    &detail.display_name,
    &content,
);

// Step 4 — stubs that become real dispatch (lines 144–171 to replace)
let seam1 = SeamResult {
    seam: "schema_load".to_string(),          // → "projection_well_formed"
    status: SeamStatus::NotChecked,
    source: "checkpoint".to_string(),          // → "validate_projection"
    findings: vec![],
    reason: Some("not_implemented_phase_195".to_string()),
};
// ... three more stubs for seam3 / seam4 / seam5 with wrong names

// Step 5 — aggregation (lines 173–188) stays unchanged
let seams = vec![seam1, seam2, seam3, seam4, seam5];
let next_steps = aggregate_next_steps(&seams);
let status = aggregate_status(&seams);
let verdict = Verdict { status, projection: name.to_string(), seams, next_steps };

// Step 6 — cache write (line 186) stays unchanged
write_cache(project_root, name, &verdict, now)?;
```

**`run_for` signature change (async):**

Current (line 107):
```rust
pub(crate) fn run_for(
    project_root: &Path,
    name: &str,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<Verdict, String>
```

After Phase 195:
```rust
pub(crate) async fn run_for(
    project_root: &Path,
    name: &str,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<Verdict, String>
```

**`execute` signature change (async):**

Current (line 101):
```rust
pub fn execute(project_root: &Path, name: &str) -> Result<Verdict, String> {
    run_for(project_root, name, chrono::Utc::now())
}
```

After Phase 195:
```rust
pub async fn execute(project_root: &Path, name: &str) -> Result<Verdict, String> {
    run_for(project_root, name, chrono::Utc::now()).await
}
```

#### `SeamResult` / `SeamStatus` types to reuse verbatim (lines 35–56)

Every wrapper seam function returns `SeamResult`. Copy the constructor pattern from the existing `field_to_column_seam` function (lines 195–300). Specifically:

- Not-checked return (lines 203–211):
```rust
return SeamResult {
    seam: "field_to_column".to_string(),
    status: SeamStatus::NotChecked,
    source: "checkpoint".to_string(),
    findings: vec![],
    reason: Some(format!("reconstruction_failed: {e}")),
};
```

- Fail return (lines 288–297):
```rust
SeamResult {
    seam: "field_to_column".to_string(),
    status: if findings.is_empty() {
        SeamStatus::Pass
    } else {
        SeamStatus::Fail
    },
    source: "checkpoint".to_string(),
    findings,
    reason: None,
}
```

Each new wrapper seam (1, 3, 4, 5) uses the identical constructor shape; only `seam`, `source`, and `findings` differ.

#### `write_cache` / `CacheEntry` analog for `read_ambient_status` (lines 407–437)

The cache write shows the exact file path convention and `ambient_status` field the read helper consumes:

```rust
// Lines 415–437 — write_cache
fn write_cache(
    project_root: &Path,
    name: &str,
    verdict: &Verdict,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<(), String> {
    let ambient_status = match verdict.status {
        SeamStatus::Pass => "clean",
        _ => "failing",
    };
    let entry = CacheEntry { verdict, ambient_status, checked_at: now };
    let cache_dir = project_root.join(".ferro").join("checkpoints");
    fs::create_dir_all(&cache_dir).map_err(|e| format!("failed to create cache dir: {e}"))?;
    let path = cache_dir.join(format!("{name}.json"));
    // ...
}
```

`read_ambient_status` mirrors this path construction (`project_root.join(".ferro").join("checkpoints").join(format!("{name}.json"))`) and reads `ambient_status` from the JSON value at that path. Visibility must be `pub(crate)` to be callable from `projection_coverage.rs` and `application_info.rs`.

#### `VerdictSummary` type placement

Declare `VerdictSummary` in the public output-contract block (after line 69, where `Verdict` is defined). It derives `Debug, Clone, Serialize, JsonSchema` — the same set used by `Verdict` (lines 59–69) and `SeamResult` (lines 45–56). Implement `Verdict::summary() -> VerdictSummary` as an `impl Verdict` method block after the struct.

`pub` visibility (not `pub(crate)`) because `generate_projection.rs` and `json_ui_generate.rs` use it in their result structs.

#### Test pattern for async conversion

Phase 194 tests that call `run_for` directly (lines 966–1096) are sync `#[test]`. After making `run_for` async these must become `#[tokio::test]`:

```rust
// Current pattern (line 981):
let result = run_for(tmp.path(), "booking_service", now);

// After async:
#[tokio::test]
async fn cache_write() {
    // ...
    let result = run_for(tmp.path(), "booking_service", now).await;
    // ...
}
```

The `tokio` dev-dep is already present (evidence: `json_ui_verify_action.rs` line 146 uses `#[tokio::test]`).

Tests that call only pure helpers (`aggregate_status`, `aggregate_next_steps`, `write_cache`, `count_column_backed_builders`, `field_to_column_seam`) remain sync `#[test]` — no change.

New Phase 195 tests that call `run_for` directly must be `#[tokio::test]`. New tests that call only pure helpers (normalization functions, `read_ambient_status` with a tempdir) remain `#[test]`.

---

### `ferro-mcp/src/tools/generate_projection.rs` — add `checkpoint: Option<VerdictSummary>` to result

**Analog:** The file itself + `application_info.rs` (nested sub-struct pattern).

#### Current `GenerateProjectionResult` struct (lines 19–26)

```rust
#[derive(Debug, Serialize)]
pub struct GenerateProjectionResult {
    pub model_name: String,
    pub service_def: serde_json::Value,
    pub intents: Vec<IntentInfo>,
    pub inferred_field_count: usize,
    pub manual_enrichment_needed: Vec<String>,
}
```

Add one field:
```rust
/// Checkpoint verdict summary run against the generated projection name.
/// `None` when the projection was not yet found in the project (first run).
#[serde(skip_serializing_if = "Option::is_none")]
pub checkpoint: Option<crate::tools::checkpoint_projection::VerdictSummary>,
```

#### Current `execute` signature (line 33)

```rust
pub fn execute(project_root: &Path, model_name: &str) -> Result<GenerateProjectionResult, String>
```

After Phase 195 — make async, call `run_for.await` at the end:
```rust
pub async fn execute(project_root: &Path, model_name: &str) -> Result<GenerateProjectionResult, String>
```

#### Checkpoint call pattern (after line 94)

The anchor name is `format!("{}_service", model_name.to_lowercase())`. Call pattern:

```rust
let anchor = format!("{}_service", model_name.to_lowercase());
let checkpoint = crate::tools::checkpoint_projection::run_for(
    project_root,
    &anchor,
    chrono::Utc::now(),
)
.await
.ok()
.map(|v| v.summary());
```

Then include `checkpoint` in the `Ok(GenerateProjectionResult { ..., checkpoint })` struct literal (line 85).

#### service.rs handler (lines 1645–1654) — no change needed

The service handler is already `pub async fn generate_projection`. It calls `tools::generate_projection::execute(...)` — after making `execute` async, add `.await`:

```rust
// Current (line 1649):
match tools::generate_projection::execute(&self.project_root, &params.0.model_name) {

// After:
match tools::generate_projection::execute(&self.project_root, &params.0.model_name).await {
```

---

### `ferro-mcp/src/tools/json_ui_generate.rs` — add `checkpoint: Option<VerdictSummary>` to context

**Analog:** `generate_projection.rs` (same `Option<VerdictSummary>` pattern); the existing `description` optional field in `JsonUiGenerationContext` (lines 27–30) for the `skip_serializing_if` pattern.

#### Current `description` optional field pattern (lines 27–30)

```rust
/// Optional view description passed through from input
#[serde(skip_serializing_if = "Option::is_none")]
pub description: Option<String>,
```

The `checkpoint` field uses the identical attribute:
```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub checkpoint: Option<crate::tools::checkpoint_projection::VerdictSummary>,
```

#### Current `execute` signature (line 104)

```rust
pub fn execute(
    project_root: &Path,
    model: Option<&str>,
    description: Option<&str>,
) -> JsonUiGenerationContext
```

After Phase 195 — make async:
```rust
pub async fn execute(
    project_root: &Path,
    model: Option<&str>,
    description: Option<&str>,
) -> JsonUiGenerationContext
```

#### Speculative anchor logic (lines 112–126 region)

When `model` is `Some(m)`, try `{m.to_lowercase()}_service` as checkpoint anchor. When `model` is `None`, skip entirely (embed `None` — SC-1 forbids vacuous all-`not_checked` summaries):

```rust
let checkpoint = match model {
    Some(m) => {
        let anchor = format!("{}_service", m.to_lowercase());
        crate::tools::checkpoint_projection::run_for(
            project_root,
            &anchor,
            chrono::Utc::now(),
        )
        .await
        .ok()
        .map(|v| v.summary())
    }
    None => None,
};
```

Include `checkpoint` in the `JsonUiGenerationContext { ..., checkpoint }` struct literal (line 113 region).

#### service.rs handler (lines 1356–1363) — add `.await`

```rust
// Current (line 1357):
let result = tools::json_ui_generate::execute(
    &self.project_root,
    params.0.model.as_deref(),
    params.0.description.as_deref(),
);

// After:
let result = tools::json_ui_generate::execute(
    &self.project_root,
    params.0.model.as_deref(),
    params.0.description.as_deref(),
)
.await;
```

---

### `ferro-mcp/src/tools/projection_coverage.rs` — add `checkpoint_status` to `ModelCoverage`

**Analog:** The file itself; `CoverageSummary` and `BroadcastingStatus` (application_info.rs lines 43–47) for the `String`-field-on-sub-struct pattern.

#### Current `ModelCoverage` struct (lines 22–37)

```rust
#[derive(Debug, Serialize)]
pub struct ModelCoverage {
    pub model_name: String,
    pub has_projection: bool,
    pub projection_name: Option<String>,
    pub projection_file: Option<String>,
    pub primary_intent: Option<String>,
    pub intent_confidence: Option<f64>,
    pub suggestion: Option<String>,
}
```

Add one field at the end:
```rust
/// Checkpoint status read from the cache file, stale-ok.
/// `"clean"` | `"failing"` | `"unverified"` (file absent or projection not found).
pub checkpoint_status: String,
```

#### Where to populate `checkpoint_status` in `execute` (lines 82–113)

In the `if let Some(proj) = matched` branch (line 82), after `projection_name: Some(proj.name.clone())`, add:

```rust
checkpoint_status: crate::tools::checkpoint_projection::read_ambient_status(
    project_root,
    &proj.name,    // projection function name, e.g. "booking_service"
),
.to_string(),
```

In the `else` branch (line 102), the projection does not exist, so:
```rust
checkpoint_status: "unverified".to_string(),
```

The call keying convention: `proj.name` is the projection **function** name (`"booking_service"`), which matches the cache file `{name}.json` written by `write_cache`. This is the same key used in `cache_write` test (checkpoint_projection.rs lines 988: `".ferro/checkpoints/booking_service.json"`). Do not use `model.name` (PascalCase) as the key.

---

### `ferro-mcp/src/tools/application_info.rs` — add `projection_checkpoint` to `ApplicationInfo`

**Analog:** The file itself. `BroadcastingStatus` (lines 43–47), `ClaudeCodeSkillsStatus` (lines 49–54), and `JsonUiSpecsStatus` (lines 56–62) all show the pattern for adding a new nested sub-struct to `ApplicationInfo`.

#### Pattern to copy for the new sub-struct — `ClaudeCodeSkillsStatus` (lines 49–54)

```rust
#[derive(Debug, Serialize)]
pub struct ClaudeCodeSkillsStatus {
    pub installed: bool,
    pub skill_count: usize,
    pub install_hint: Option<String>,
}
```

New sub-struct follows the same derive and field pattern:

```rust
#[derive(Debug, Serialize)]
pub struct ProjectionCheckpointSummary {
    pub total_projections: usize,
    pub clean: usize,
    pub failing: usize,
    pub unverified: usize,
}
```

#### Add field to `ApplicationInfo` (line 23 region)

Current (lines 11–23):
```rust
pub struct ApplicationInfo {
    pub framework_version: String,
    // ...
    pub claude_code_skills: ClaudeCodeSkillsStatus,
}
```

Add at end:
```rust
pub projection_checkpoint: ProjectionCheckpointSummary,
```

#### `execute` function — population pattern (lines 77–119)

Follow the same pattern as `check_claude_code_skills()` (line 106) — a private helper function:

```rust
fn check_projection_checkpoint(project_root: &Path) -> ProjectionCheckpointSummary {
    let list = super::list_projections::execute(project_root, None);
    let mut clean = 0usize;
    let mut failing = 0usize;
    let mut unverified = 0usize;
    for proj in &list.projections {
        match crate::tools::checkpoint_projection::read_ambient_status(project_root, &proj.name) {
            "clean" => clean += 1,
            "failing" => failing += 1,
            _ => unverified += 1,
        }
    }
    ProjectionCheckpointSummary {
        total_projections: list.total,
        clean,
        failing,
        unverified,
    }
}
```

Call it in `execute` alongside the other helper calls (line 106 pattern):
```rust
let projection_checkpoint = check_projection_checkpoint(project_root);
```

And include in the `Ok(ApplicationInfo { ..., projection_checkpoint })` struct literal.

---

### `ferro-mcp/src/service.rs` — update handler call sites + tool descriptions

**Analog:** The file itself. Three handler patterns to update:

#### `checkpoint_projection` handler (lines 1604–1613) — add `.await` to `execute` call

```rust
// Current (line 1608):
match tools::checkpoint_projection::execute(&self.project_root, &params.0.name) {

// After:
match tools::checkpoint_projection::execute(&self.project_root, &params.0.name).await {
```

The handler signature (`pub async fn checkpoint_projection`) does not change.

#### `generate_projection` handler (lines 1645–1654) — add `.await`

```rust
// Current (line 1649):
match tools::generate_projection::execute(&self.project_root, &params.0.model_name) {

// After:
match tools::generate_projection::execute(&self.project_root, &params.0.model_name).await {
```

#### `json_ui_generate` handler (lines 1356–1362) — add `.await`

```rust
// Current (line 1357):
let result = tools::json_ui_generate::execute(
    &self.project_root,
    params.0.model.as_deref(),
    params.0.description.as_deref(),
);

// After:
let result = tools::json_ui_generate::execute(
    &self.project_root,
    params.0.model.as_deref(),
    params.0.description.as_deref(),
)
.await;
```

#### Tool description updates

For `checkpoint_projection` (line 1594 region): update `source` description — remove caveat "always `checkpoint` in this version"; mention that seam 1/3/4/5 now delegate to their respective validators.

For `generate_projection` (line 1636 region): add to **Returns:** that the result includes a `checkpoint` field (summary only, `null` when projection not yet in project).

For `projection_coverage` (line 1618 region): add to **Returns:** that each model entry now includes `checkpoint_status: "clean" | "failing" | "unverified"`.

For `application_info` (line 403 region): add to **Returns:** that the result includes a `projection_checkpoint: { total_projections, clean, failing, unverified }` summary.

---

## Shared Patterns

### SeamResult construction (not-checked path)
**Source:** `checkpoint_projection.rs` lines 203–211 (`field_to_column_seam` not-checked returns)
**Apply to:** All four wrapper seam functions (seam 1, 3, 4, 5)

```rust
return SeamResult {
    seam: "<canonical_seam_name>".to_string(),
    status: SeamStatus::NotChecked,
    source: "<validator_name>".to_string(),
    findings: vec![],
    reason: Some("<reason_string>".to_string()),
};
```

### SeamResult construction (pass/fail path)
**Source:** `checkpoint_projection.rs` lines 288–299
**Apply to:** All four wrapper seam functions

```rust
SeamResult {
    seam: "<canonical_seam_name>".to_string(),
    status: if findings.is_empty() { SeamStatus::Pass } else { SeamStatus::Fail },
    source: "<validator_name>".to_string(),
    findings,
    reason: None,
}
```

### Finding construction
**Source:** `checkpoint_projection.rs` lines 272–284 (inside `field_to_column_seam`)
**Apply to:** All normalization functions

```rust
findings.push(Finding {
    subject: <subject_string>,
    detail: format!("<human-readable problem description>"),
    fix: format!("<concrete remediation step>"),
});
```

### Optional sub-struct field with skip_serializing_if
**Source:** `json_ui_generate.rs` lines 27–30 (`description` field)
**Apply to:** `VerdictSummary` fields in `VerdictSummary` struct; `checkpoint` fields on result structs

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub checkpoint: Option<SomeType>,
```

### `pub(crate)` helper consumed by sibling tool modules
**Source:** `json_ui_verify_action.rs` line 52 (`find_handler` is `pub(crate)`)
**Apply to:** `read_ambient_status` in `checkpoint_projection.rs`

`pub(crate)` visibility lets `projection_coverage.rs` and `application_info.rs` call `super::checkpoint_projection::read_ambient_status(...)` directly without making the helper part of the public MCP API.

### `#[tokio::test]` for async unit tests
**Source:** `json_ui_verify_action.rs` line 146

```rust
#[tokio::test]
async fn verify_action_rejects_oversized_handler_input() {
    let result = execute(std::path::Path::new("."), &huge, None).await;
    assert!(result.is_err());
}
```

Apply to: any Phase 195 test that calls `run_for` or `execute` from `checkpoint_projection.rs`.

---

## Seam-Name Reconciliation — Edit Sites (D-01)

The four wrong names appear in three locations. All three must be updated in the same commit:

| Wrong name (Phase 194) | Canonical name (Phase 195) |
|------------------------|---------------------------|
| `"schema_load"` | `"projection_well_formed"` |
| `"field_type_compat"` | `"action_to_route"` |
| `"action_binding"` | `"rendered_view"` |
| `"render_target"` | `"props_to_contract"` |

**Location 1:** `checkpoint_projection.rs` lines 145, 152, 159, 166 — the four `seam: "..."` string literals in the stub block.

**Location 2:** `checkpoint_projection.rs` test module lines ~849–933 — `make_seam("schema_load", ...)` and `make_seam("action_binding", ...)` calls in six test functions:
- `aggregate_status_fail_wins_over_not_checked` (line ~849)
- `aggregate_status_warn_not_checked` (line ~857)
- `aggregate_status_pass_not_checked` (line ~866)
- `aggregate_status_all_not_checked_is_pass` (line ~875) — uses both `"schema_load"` and `"action_binding"`
- `next_steps_ranked_deduped` (line ~887) — uses `"schema_load"` and `"field_to_column"`
- `next_steps_dedup` (line ~924) — uses `"action_binding"`

**Location 3:** `docs/src/agents/checkpoint-projection.md` — the seam example JSON block (~line 42) and the SeamResult table caveat (~line 78). The `source` shown for previously-stub seams must change from `"checkpoint"` to the respective validator name.

**Verification command:** `grep -rn "schema_load\|field_type_compat\|action_binding\|render_target" ferro-mcp/src/` must return zero results after the edit.

---

## No Analog Found

No new files are created in Phase 195. All work is modification of existing files. No files without analogs.

---

## Metadata

**Analog search scope:** `ferro-mcp/src/tools/`, `ferro-mcp/src/service.rs`
**Files scanned:** 10 source files read directly
**Pattern extraction date:** 2026-06-10
