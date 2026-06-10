# Phase 196: Dogfood Acceptance + Hardening - Pattern Map

**Mapped:** 2026-06-10
**Files analyzed:** 4 (3 modified, 1 new artifact)
**Analogs found:** 4 / 4

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-mcp/src/tools/checkpoint_projection.rs` (test block additions + cap + demotion) | test + utility | batch / transform | same file — existing `seam2_dangling_field`, `next_steps_cap_at_10`, `cache_write` tests | exact |
| `ferro-mcp/src/service.rs` (tool description update) | config / doc | request-response | same file — existing `checkpoint_projection` tool description at lines 1598–1610 | exact |
| `docs/src/agents/checkpoint-projection.md` (cap + seam status docs) | config / doc | — | same file — lines 61, 125 | exact |
| `.planning/phases/196-dogfood-acceptance-hardening/196-ACCEPTANCE.md` | report artifact | — | no code analog (free-form report) | none |

---

## Pattern Assignments

### New test: `poisoned_projection_dangling_field_acceptance` (SC-1 / D-01)

**Analog:** `seam2_dangling_field` test, `checkpoint_projection.rs` lines 1009–1045

**Fixture construction pattern** (lines 1022–1026):
```rust
let model_src = model_src_with_fields("Booking", &["id"]);
let tmp = project_with_projection("booking_service", proj_src);
add_model(&tmp, "booking", &model_src);

let result = field_to_column_seam(tmp.path(), "booking", &None, proj_src);
```

**Assertion pattern** (lines 1028–1044) — the poisoned test must copy ALL four assertion shapes, not just the non-empty check:
```rust
assert_eq!(result.status, SeamStatus::Fail, "dangling field must fail");
assert_eq!(
    result.findings.len(),
    1,
    "exactly one finding for the phantom field"
);
assert_eq!(result.findings[0].subject, "phantom");
assert!(
    result.findings[0].fix.contains("add column"),
    "fix must contain 'add column': {}",
    result.findings[0].fix
);
assert!(
    result.findings[0].fix.contains("migration"),
    "fix must reference migration: {}",
    result.findings[0].fix
);
```

**SC-1 extension** — the existing analog does not assert "no other field is flagged". The new test adds a negative assertion after the positive ones:
```rust
// SC-1: no other field flagged (id must NOT appear as a finding)
assert!(!result.findings.iter().any(|f| f.subject == "id"), "id must not be flagged");
```

**Name-matching rule** (from RESEARCH.md, verified at `list_models.rs:150`):
`model_src_with_fields("Dangling", ...)` → `list_models` returns `ModelDetails { name: "Dangling" }`. Match: `"dangling".to_lowercase() == "dangling".to_lowercase()` → true. Use a single-word struct name with no underscores so `.to_lowercase()` is an exact char match of the service_name.

**Projection source pattern** — use `DataType::String` (not `DataType::Text`) for fields that must reconstruct correctly to avoid the D-06 warn path firing before the column check:
```rust
let proj_src = r#"
use ferro::{ServiceDef, DataType, FieldMeaning};
pub fn dangling_service() -> ServiceDef {
    ServiceDef::new("dangling")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("phantom_col", DataType::String, FieldMeaning::FreeText)
}
"#;
```

---

### New test: `next_steps_cap_at_five` (SC-3 / D-05)

**Analog:** `next_steps_cap_at_10` test, `checkpoint_projection.rs` lines 1360–1369

**Exact analog to copy from** (lines 1360–1369):
```rust
#[test]
fn next_steps_cap_at_10() {
    // D-10: 12 distinct findings → exactly 10 next_steps entries.
    let findings: Vec<Finding> = (0..12)
        .map(|i| make_finding(&format!("field_{i}"), &format!("fix field_{i}")))
        .collect();
    let seams = vec![make_seam("field_to_column", SeamStatus::Fail, findings)];
    let steps = aggregate_next_steps(&seams);
    assert_eq!(steps.len(), 10, "next_steps must be capped at 10");
}
```

**New test** copies this structure verbatim, changes the count to 7 (>5) and the assertion to 5. The existing `next_steps_cap_at_10` test must also be updated in the same commit (change `0..12` to `0..7` and `== 10` to `== 5`, and rename to `next_steps_cap_at_five` or update in place).

**Helper signatures used** (lines 1238–1256):
```rust
fn make_seam(seam: &str, status: SeamStatus, findings: Vec<Finding>) -> SeamResult {
    SeamResult {
        seam: seam.to_string(),
        status,
        source: "test".to_string(),
        findings,
        reason: None,
    }
}

fn make_finding(subject: &str, fix: &str) -> Finding {
    Finding {
        subject: subject.to_string(),
        detail: "detail".to_string(),
        fix: fix.to_string(),
    }
}
```

---

### New test: `dogfood_app_projections` (SC-2 / D-02 / D-04)

**Analog:** `cache_write` test (`#[tokio::test]`, lines 1382–1432) — provides the `#[tokio::test]` + `project_with_projection` + `run_for` + tempdir pattern. The dogfood test replaces `run_for` with direct seam calls but follows the same async test structure.

**`app/` path resolution pattern** (from RESEARCH.md, Pattern 3):
```rust
let app_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    .parent()
    .expect("ferro-mcp has a parent")
    .join("app");

assert!(app_root.exists(), "app/ must exist at {}", app_root.display());
```

**Directory iteration pattern** — copy from Pattern 3 in RESEARCH.md; no existing in-file analog. Filter `.rs` files, exclude `mod.rs`, sort for determinism:
```rust
let proj_dir = app_root.join("src/projections");
let mut entries: Vec<_> = std::fs::read_dir(&proj_dir)
    .unwrap()
    .filter_map(|e| e.ok())
    .filter(|e| {
        e.path().extension().map(|x| x == "rs").unwrap_or(false)
            && e.path().file_name().map(|n| n != "mod.rs").unwrap_or(false)
    })
    .collect();
entries.sort_by_key(|e| e.path());
```

**Routes pre-load pattern** — mirrors lines 196–199 in `run_for`:
```rust
let routes = list_routes::execute(&app_root)
    .await
    .map(|info| info.routes)
    .ok();
```

**SC-2 assertion pattern** — the assert! form (not eprintln!) enforces the acceptance gate at test time:
```rust
assert!(
    total_findings > 0,
    "SC-2 FAIL: checkpoint found zero findings across all app/ projections.\n\
     Per-seam tally: {:?}\n\
     Acceptance is NO-GO — design must be revisited before shipping.",
    tally
);
```

**D-04 tally output** — use `println!` (visible with `--nocapture`):
```rust
println!("Per-seam finding tally across app/ projections:");
for (seam, count) in &tally {
    println!("  {}: {} findings", seam, count);
}
```

---

### Cap reduction: `aggregate_next_steps` (D-05)

**Edit site:** `checkpoint_projection.rs` line 763

**Exact line to change:**
```rust
// Before (line 763):
if result.len() == 10 {

// After (introduce const above aggregate_next_steps, change guard):
const MAX_NEXT_STEPS: usize = 5;

// ... inside aggregate_next_steps:
if result.len() == MAX_NEXT_STEPS {
```

**Docstring edit site** (line 737):
```rust
// Before:
/// Dedup by `(subject, fix)`. Cap at 10.

// After:
/// Dedup by `(subject, fix)`. Cap at 5.
```

**Verdict doc comment edit site** (line 71):
```rust
// Before:
/// Ranked, deduplicated actionable strings (failures before warnings; cap 10).

// After:
/// Ranked, deduplicated actionable strings (failures before warnings; cap 5).
```

**Docs edit sites** (`docs/src/agents/checkpoint-projection.md`):
- Line 61: `"capped at 10"` → `"capped at 5"`
- Line 125: `"The list is capped at 10 entries"` → `"The list is capped at 5 entries"`

---

### Zero-finding seam demotion: wrapper seam dispatch (D-04)

**Edit site:** the body of whichever wrapper seam function(s) produced zero findings across both dogfood inputs. The demotion pattern replaces the dispatch body with a `not_checked` literal return. **Do not demote before recording the tally.**

**Analog for `not_checked` literal construction** — lines 250–258 (inside `field_to_column_seam`):
```rust
return SeamResult {
    seam: "field_to_column".to_string(),
    status: SeamStatus::NotChecked,
    source: "checkpoint".to_string(),
    findings: vec![],
    reason: Some(format!("reconstruction_failed: {e}")),
};
```

**Demotion pattern** — replace the full function body; use `_` prefix on unused params to suppress clippy:
```rust
fn projection_well_formed_seam(_project_root: &Path, _name: &str) -> SeamResult {
    // Demoted: produced zero findings across all dogfood inputs (poisoned fixture + app/).
    // Reported as not_checked to preserve coverage-honesty invariant (CHK-03).
    SeamResult {
        seam: "projection_well_formed".to_string(),
        status: SeamStatus::NotChecked,
        source: "validate_projection".to_string(),
        findings: vec![],
        reason: Some("unproven_against_real_inputs".to_string()),
    }
}
```

Source field must still name the delegating validator (not `"checkpoint"`), consistent with SC-4 (lines 354–355 comment: `"source" is always "validate_projection"` for wrapper seams).

---

### `ferro-mcp/src/service.rs` tool description update (D-04 SC-4)

**Analog:** existing `checkpoint_projection` tool description, lines 1598–1610:
```rust
#[tool(
    name = "checkpoint_projection",
    description = "Run a checkpoint on a service projection and return a single structured verdict.\n\n\
        **When to use:** after generating or editing a projection; ...\n\n\
        **Returns:** top-level status (pass/warn/fail), a per-seam result list ...\n\n\
        **Read-only:** ...\n\n\
        **Combine with:** ..."
)]
```

**Pattern:** append a `**Seam coverage:**` paragraph to the existing description string using `\n\n`. Do not restructure the existing paragraphs. Follow the same `**Bold label:** sentence.` style used by the three existing paragraphs.

---

### `docs/src/agents/checkpoint-projection.md` update (D-04 + D-05)

**Edit 1 — stale stub artifact** (line 46): the example JSON shows `"reason": "not_implemented_phase_195"`. Replace with a current, real `not_checked` reason string (e.g. `"unproven_against_real_inputs"` if the seam is demoted, or an actual reason from a live run).

**Edit 2 — cap references** (lines 61 and 125): change `10` → `5` in both locations as described in the cap reduction section above.

**Edit 3 — seam coverage note**: add a brief paragraph or table row after the existing `not_checked` section documenting which wrapper seams are `not_checked`-by-default and why, citing the dogfood evidence.

---

## Shared Patterns

### Test module header and helper availability

**Source:** `checkpoint_projection.rs` lines 854–875

All new tests go inside the existing `#[cfg(test)] mod tests { ... }` block. The helpers `project_with_projection`, `add_model`, `model_src_with_fields`, `make_seam`, and `make_finding` are already defined there — no re-declaration.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn project_with_projection(name: &str, projection_src: &str) -> tempfile::TempDir { ... }
    fn add_model(tmp: &tempfile::TempDir, name: &str, model_src: &str) { ... }
    fn model_src_with_fields(struct_name: &str, fields: &[&str]) -> String { ... }
    fn make_seam(seam: &str, status: SeamStatus, findings: Vec<Finding>) -> SeamResult { ... }
    fn make_finding(subject: &str, fix: &str) -> Finding { ... }
}
```

### `not_checked` SeamResult construction

**Source:** multiple sites in `checkpoint_projection.rs` (e.g. lines 250–258, 285–292, 300–308)

Consistent shape used everywhere:
```rust
SeamResult {
    seam: "<seam_name>".to_string(),
    status: SeamStatus::NotChecked,
    source: "<delegating_validator>".to_string(),
    findings: vec![],
    reason: Some("<reason_string>".to_string()),
}
```

`reason` is always `Some(...)` for `NotChecked` (never `None`). `source` names the delegating validator, not `"checkpoint"` (SC-4). `findings` is always `vec![]`.

### `#[tokio::test]` async test structure

**Source:** `cache_write` test, lines 1382–1432

```rust
#[tokio::test]
async fn cache_write() {
    // setup
    let tmp = project_with_projection("booking_service", proj_src);
    add_model(&tmp, "booking", &model_src);
    let now = fixed_now();
    let result = run_for(tmp.path(), "booking_service", now).await;
    if let Ok(verdict) = result {
        // assertions
    }
}
```

The `dogfood_app_projections` test uses the same `#[tokio::test]` attribute (required for the `list_routes::execute` await call).

### SeamStatus assertion style

**Source:** throughout the test block

Always use the named variant, never a string:
```rust
assert_eq!(result.status, SeamStatus::Fail, "message");
assert_eq!(result.status, SeamStatus::NotChecked, "message");
assert_eq!(result.reason.as_deref(), Some("reason_string"), "message");
```

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `196-ACCEPTANCE.md` | report artifact | — | Free-form report; no existing acceptance report in the repo. Structure per CONTEXT.md D-03: GO/NO-GO verdict + per-seam finding tally. |

---

## Critical Implementation Constraints

These are verified facts from RESEARCH.md that affect which patterns apply:

1. **Function-name collision on `app/`:** All 8 `app/` projection files export `service_def`. The dogfood test cannot use `run_for` — call seam functions directly per file (see Pattern 3 above).

2. **Seam 2 always `not_checked` on `app/`:** SeaORM entity files define `pub struct Model`, so `list_models` returns `name: "Model"` for all three app models. `"api_key" == "model"` is false → seam 2 is `not_checked("source_model_unresolved")` for every `app/` projection. SC-2 compliance depends on seams 1, 3, 4, or 5 finding something.

3. **D-04 is evidence-driven:** Run the dogfood test first, record the per-seam tally in `196-ACCEPTANCE.md`, then demote only the seams that actually found nothing. Do not pre-emptively demote.

4. **Four doc-comment locations must be updated atomically with the cap change** (lines 71, 737 in `checkpoint_projection.rs`; lines 61, 125 in `docs/src/agents/checkpoint-projection.md`).

5. **Existing `next_steps_cap_at_10` test** (line 1361) must be updated or renamed in the same commit as the cap literal change — otherwise `cargo test` fails.

## Metadata

**Analog search scope:** `ferro-mcp/src/tools/checkpoint_projection.rs`, `ferro-mcp/src/service.rs`, `docs/src/agents/checkpoint-projection.md`
**Files scanned:** 4 primary source files read in full
**Pattern extraction date:** 2026-06-10
