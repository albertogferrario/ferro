# Phase 196: Dogfood Acceptance + Hardening - Research

**Researched:** 2026-06-10
**Domain:** Rust test infrastructure, checkpoint_projection internal API, app/ projection-model corpus analysis
**Confidence:** HIGH (code paths verified; runtime seam behavior on app/ requires a live run)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Poisoned projection as a committed acceptance test fixture using `project_with_projection(name, src)` + `add_model(tmp, name, src)`. One dangling field; assert `status == "fail"`, finding names exactly that field, finding count for that seam == 1. NOT poisoning `app/` sample.
- **D-02:** Live consumer is `app/` sample application. gestiscilo is not reachable. Run the checkpoint against each `app/` projection and record the aggregate. (Implementation note: cannot use `run_for` due to function-name collision; must call seam functions directly — see Pattern 3 below.)
- **D-03:** Acceptance recorded as (1) automated tests and (2) committed `196-ACCEPTANCE.md` with GO/NO-GO verdict + per-seam finding tally.
- **D-04:** Tally findings per wrapper seam (1, 3, 4, 5) across both dogfood inputs (poisoned fixture + `app/` live run). For each seam with zero findings: change default to `not_checked` with reason; document in `service.rs` + `docs/src/agents/checkpoint-projection.md`.
- **D-05:** Reduce cap from 10 to 5. Edit `aggregate_next_steps` line 763 (`if result.len() == 10` → `== 5`). Update all doc comments saying "cap 10" (4 locations — see D-05 edit sites). Optionally introduce `const MAX_NEXT_STEPS: usize = 5`.

### Claude's Discretion

- Exact test function names for poisoned-fixture and over-cap tests.
- Wording of `not_checked`-by-default reason strings and tool-description notes.
- Structure of `196-ACCEPTANCE.md` beyond required GO/NO-GO verdict + finding tally.
- Whether cap becomes a named `const` or inline literal.

### Deferred Ideas (OUT OF SCOPE)

- Running checkpoint against gestiscilo (external repo, unreachable).
- IN-02 from Phase 194 code review (DataType warn subject) — fold in only if this phase touches that path.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CHK-10 | Checkpoint run across synthetic app catalog (deliberately poisoned) and live consumer; acceptance requires at least one real seam defect; a checkpoint that finds nothing fails acceptance and design is revisited | D-01 poisoned fixture proves seam 2. D-02 live-consumer run on `app/`: CRITICAL RISK — model-name matching issue means seam 2 produces `not_checked` on all app/ projections (see SC-2 risk below). Seams 3/4/5 are the viable sources of findings. |
</phase_requirements>

---

## Summary

Phase 196 is a pure hardening and acceptance phase — no new seam logic. Four concrete deliverables: a poisoned fixture test (D-01), a live-consumer dogfood run (D-02), a `next_steps` cap reduction from 10 to 5 (D-05), and evidence-driven demotion of zero-finding seams to `not_checked`-by-default (D-04).

**The critical empirical question** is which seams produce findings against `app/` projections. Research has fully traced the execution path and discovered two structural issues with the `app/` sample that the implementor must resolve before the dogfood run:

1. **Function-name collision:** All 8 `app/` projection files export a function named `service_def`. The `run_for` entry point uses `inspect_projection → list_projections`, which matches by function name. Calling `run_for(app_root, "service_def", ...)` would find only one projection. **Solution:** call seam functions directly per file, bypassing `run_for`. [VERIFIED]

2. **Model-name mismatch (SC-2 risk):** All `app/` entity files define `pub struct Model` (SeaORM convention). `list_models::execute` extracts the Rust struct ident as `ModelDetails.name` — so it returns `"Model"` for all three entity models. The `field_to_column` seam matches `service_name.to_lowercase() == model.name.to_lowercase()` — i.e., `"api_key" == "model"`, which is `false`. **Seam 2 will produce `not_checked("source_model_unresolved")` on all `app/` projections.** [VERIFIED by tracing `list_models.rs:148–160`]

This means **SC-2 (at least one finding on the live consumer) depends entirely on seams 1, 3, 4, or 5 producing a finding** against `app/`. If all wrapper seams produce `not_checked` or `pass`, the verdict is `pass` with no findings — SC-2 fails, acceptance is NO-GO, and the design is revisited.

**Primary recommendation:** The dogfood test must call each seam function directly per `app/` projection file. Seam 3 (`action_to_route`) on `feedback_form` (has `submit_feedback` action) and `order` (has `submit`, `approve`, `ship` actions) is the most likely source of findings if those handlers are not registered. The implementor must check `app/src/routes.rs` before writing the acceptance test.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Poisoned fixture + assertion | Test module (`checkpoint_projection.rs`) | — | Follows existing test helper pattern; no new scaffolding |
| Live-consumer dogfood run | Test module (`checkpoint_projection.rs`) | Acceptance report (`.planning/`) | Test is the evidence; report is the artifact |
| Cap reduction (10→5) | `aggregate_next_steps` function + docstrings | Docs | Single function with one literal; 4 doc locations track the invariant |
| Zero-finding seam demotion | Per-seam dispatch functions in `checkpoint_projection.rs` | `service.rs` + `docs/src/agents/` | Default outcome change is in code; documentation reflects it |

## Standard Stack

### Core (in-repo, no new dependencies)

| Symbol | Location | Purpose | Why Standard |
|--------|----------|---------|--------------|
| `project_with_projection(name, src)` | `checkpoint_projection.rs:861` | Creates tempdir with `src/projections/{name}.rs` | Established test helper for this module |
| `add_model(tmp, name, src)` | `checkpoint_projection.rs:871` | Adds `src/models/{name}.rs` to existing tempdir | Established test helper for this module |
| `field_to_column_seam(root, svc, display, content)` | `checkpoint_projection.rs:241` | Direct seam 2 invocation — bypass `run_for` | Only viable path for dogfood run (function-name collision) |
| `projection_well_formed_seam(root, name)` | `checkpoint_projection.rs:355` | Direct seam 1 invocation | `pub(crate)`, callable from tests |
| `action_to_route_seam(service, routes)` | `checkpoint_projection.rs:409` | Direct seam 3 invocation | `pub(crate)`, callable from tests |
| `rendered_view_seam(root, name)` | `checkpoint_projection.rs:492` | Direct seam 4 invocation | `pub(crate)`, callable from tests |
| `props_to_contract_seam(root, svc_name)` | `checkpoint_projection.rs:580` | Direct seam 5 invocation | `pub(crate)`, callable from tests |
| `reconstruct_service_def(svc, display, content)` | `render_projection.rs:113` | Extract ServiceDef from source text for seam 3 input | Already used by seam 2 internally |
| `list_routes::execute(root)` | `list_routes.rs` | Load routes for seam 3 (async) | Already used by `run_for` |
| `aggregate_next_steps(seams)` | `checkpoint_projection.rs:738` | Build ranked next_steps | Already used by `run_for` |
| `model_src_with_fields(struct_name, fields)` | `checkpoint_projection.rs:992` | Build SeaORM model source text for test fixtures | Private helper; reuse in poisoned fixture |

**No new crate dependencies.** Everything is already imported in `checkpoint_projection.rs`.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Direct seam fn calls for dogfood | `run_for` per projection | `run_for` requires unique function names; all `app/` projections export `service_def` — `inspect_projection` would find only one. Direct calls are the only viable approach. |
| `const MAX_NEXT_STEPS: usize = 5` | inline `5` literal | `const` is self-documenting and prevents accidental divergence between cap guard, docstring, and test. Mildly preferred per CONTEXT.md. |

## Architecture Patterns

### System Architecture Diagram

```
Poisoned fixture (test-only tempdir)           app/src/projections/*.rs
         |                                              |
         v                                              v
  project_with_projection               [read file content per file]
  + add_model (tempdir)                 [extract service_name from ServiceDef::new("...")]
         |                                              |
         v                                              v
  field_to_column_seam              field_to_column_seam (not_checked — Model name mismatch)
  → Fail (planted dangling field)   projection_well_formed_seam
  SC-1 assertion                    action_to_route_seam  ← primary finding source
                                    rendered_view_seam
                                    props_to_contract_seam
                                              |
                                              v
                                    per-seam SeamResult list
                                              |
                                              v
                                    aggregate_next_steps / per-seam tally
                                              |
                                              v
                                    D-04 demotion decision + 196-ACCEPTANCE.md GO/NO-GO
```

### Recommended Project Structure

No new files except the acceptance report. All code changes go into existing files:

```
ferro-mcp/src/tools/
└── checkpoint_projection.rs     # poisoned fixture test, over-cap test,
                                 # dogfood test, cap reduction, seam demotion

ferro-mcp/src/service.rs         # D-04: tool description updates for not_checked seams

docs/src/agents/
└── checkpoint-projection.md     # D-04: seam status docs + cap 5 update

.planning/phases/196-dogfood-acceptance-hardening/
└── 196-ACCEPTANCE.md            # D-03: GO/NO-GO verdict + per-seam tally
```

### Pattern 1: Poisoned Fixture Test (SC-1 / D-01)

**What:** Commit a test that builds exactly one dangling field and asserts the finding set is exactly `{dangling_field}`.

```rust
// Source: checkpoint_projection.rs existing test infrastructure
#[test]
fn poisoned_projection_dangling_field_acceptance() {
    // SC-1: exactly one planted dangling field → exactly one finding naming it.
    // Model has "id" only. Projection adds "phantom_col" — no backing column.
    let proj_src = r#"
use ferro::{ServiceDef, DataType, FieldMeaning};
pub fn poisoned_acceptance() -> ServiceDef {
    ServiceDef::new("poisoned_acceptance")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("phantom_col", DataType::Integer, FieldMeaning::FreeText)
}
"#;
    // model_src_with_fields("PoisonedAcceptance", ...) produces table name
    // "poisoned_acceptances"; list_models returns name="PoisonedAcceptance".
    // Matching: "poisoned_acceptance" == "poisonedacceptance" — case-insensitive.
    // But underscore stripping: "poisoned_acceptance".to_lowercase() ==
    // "poisonedacceptance".to_lowercase() → false!
    // Use a single-word struct name to avoid underscore mismatch.
    // e.g. struct name "Dangling" → service_name "dangling"
    let proj_src = r#"
use ferro::{ServiceDef, DataType, FieldMeaning};
pub fn dangling_service() -> ServiceDef {
    ServiceDef::new("dangling")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("phantom_col", DataType::Integer, FieldMeaning::FreeText)
}
"#;
    let model_src = model_src_with_fields("Dangling", &["id"]);
    let tmp = project_with_projection("dangling_service", proj_src);
    add_model(&tmp, "dangling", &model_src);

    let result = field_to_column_seam(tmp.path(), "dangling", &None, proj_src);

    assert_eq!(result.status, SeamStatus::Fail);
    // SC-1: exactly one finding, names exactly the planted field
    assert_eq!(result.findings.len(), 1, "exactly one finding for the one dangling field");
    assert_eq!(result.findings[0].subject, "phantom_col", "subject must name the planted field");
    // SC-1: no other field flagged (id must NOT appear)
    assert!(!result.findings.iter().any(|f| f.subject == "id"), "id must not be flagged");
}
```

[VERIFIED: matches existing test patterns at lines 1010–1045]

**CRITICAL: name-matching rule for fixtures.** `field_to_column_seam` matches `service_name.to_lowercase() == model.name.to_lowercase()`. `model_src_with_fields("Dangling", ...)` uses struct ident `"Dangling"` → `list_models` returns `ModelDetails { name: "Dangling", ... }`. `"dangling" == "dangling"` → match. Use a struct name whose lowercase equals the service_name exactly (no underscores, since the struct ident comparison is char-exact, not token-exact). [VERIFIED by tracing `list_models.rs:150`]

### Pattern 2: Over-Cap Test (SC-3 / D-05)

**What:** Build 7+ distinct findings and assert `next_steps.len() == 5` after the cap reduction.

```rust
// Source: adapts next_steps_cap_at_10 at checkpoint_projection.rs:1361
#[test]
fn next_steps_cap_at_five() {
    // SC-3: 7 distinct findings → exactly 5 next_steps entries (cap is 5).
    let findings: Vec<Finding> = (0..7)
        .map(|i| make_finding(&format!("field_{i}"), &format!("fix field_{i}")))
        .collect();
    let seams = vec![make_seam("field_to_column", SeamStatus::Fail, findings)];
    let steps = aggregate_next_steps(&seams);
    assert_eq!(steps.len(), 5, "next_steps must be capped at 5");
}
```

**The existing `next_steps_cap_at_10` test (line 1361) must be updated** from `assert_eq!(steps.len(), 10)` to `assert_eq!(steps.len(), 5)` in the same commit as the cap change. [VERIFIED: line 1368]

### Pattern 3: Dogfood Live-Consumer Test (SC-2 / D-02 / D-04)

**What:** Iterate `app/src/projections/*.rs` directly, call seam functions per file, tally findings per seam. The test must assert `total_findings > 0` to enforce SC-2.

**Key constraint:** Seam 2 will produce `not_checked` for all `app/` projections due to the Model-name mismatch (see SC-2 risk above). The test must find findings from seams 1, 3, 4, or 5. Seam 3 on `feedback_form`/`order` is the primary candidate.

```rust
#[tokio::test]
async fn dogfood_app_projections() {
    // SC-2 enforcement: live consumer must produce at least one finding.
    // D-04 evidence: tally per-seam findings across all app/ projections.
    //
    // Cannot use run_for: all app/ projections export `service_def` (name collision).
    // Call seam functions directly per file.

    let app_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("ferro-mcp has a parent")
        .join("app");

    assert!(app_root.exists(), "app/ must exist at {}", app_root.display());

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

    // Pre-load routes once (async I/O) for seam 3.
    let routes = list_routes::execute(&app_root)
        .await
        .map(|info| info.routes)
        .ok();

    let mut tally: std::collections::HashMap<&str, usize> = [
        ("projection_well_formed", 0),
        ("field_to_column", 0),
        ("action_to_route", 0),
        ("rendered_view", 0),
        ("props_to_contract", 0),
    ].into_iter().collect();

    for entry in &entries {
        let content = std::fs::read_to_string(entry.path()).unwrap();
        // Extract service_name from ServiceDef::new("...")
        let service_name_re = regex::Regex::new(r#"ServiceDef::new\("([^"]+)"\)"#).unwrap();
        let service_name = service_name_re.captures(&content)
            .map(|c| c[1].to_string())
            .unwrap_or_default();
        let display_name_re = regex::Regex::new(r#"\.display_name\("([^"]+)"\)"#).unwrap();
        let display_name = display_name_re.captures(&content).map(|c| c[1].to_string());
        let file_stem = entry.path().file_stem().unwrap().to_string_lossy().to_string();

        let seam1 = projection_well_formed_seam(&app_root, &file_stem);
        let seam2 = field_to_column_seam(&app_root, &service_name, &display_name, &content);
        let service_def = reconstruct_service_def(&service_name, &display_name, &content).ok();
        let seam3 = action_to_route_seam(service_def.as_ref(), routes.as_deref());
        let seam4 = rendered_view_seam(&app_root, &file_stem);
        let seam5 = props_to_contract_seam(&app_root, &service_name);

        for seam in &[&seam1, &seam2, &seam3, &seam4, &seam5] {
            let count = seam.findings.len()
                + if matches!(seam.status, SeamStatus::Fail | SeamStatus::Warn) { 0 } else { 0 };
            // Count findings only (not_checked has no findings)
            *tally.get_mut(seam.seam.as_str()).unwrap_or(&mut 0) += seam.findings.len();
        }
    }

    let total_findings: usize = tally.values().sum();

    // SC-2: at least one finding required. If zero, acceptance fails — revisit design.
    assert!(
        total_findings > 0,
        "SC-2 FAIL: checkpoint found zero findings across all app/ projections.\n\
         Per-seam tally: {:?}\n\
         Acceptance is NO-GO — design must be revisited before shipping.",
        tally
    );

    // D-04 evidence: print tally for ACCEPTANCE.md
    println!("Per-seam finding tally across app/ projections:");
    for (seam, count) in &tally {
        println!("  {}: {} findings", seam, count);
    }
}
```

[ASSUMED: exact `projection_well_formed_seam` and `rendered_view_seam` call sites may need the correct `name` parameter. `projection_well_formed_seam(root, name)` takes the projection function name or file stem — needs verification that `validate_projection::execute_single` accepts file-stem vs function-name. If it does path-based lookup, `file_stem` works. If it does list-based lookup, this needs adjustment.]

### Anti-Patterns to Avoid

- **Calling `run_for(app_root, "service_def", ...)` in the dogfood test:** `list_projections` finds all files that define `pub fn service_def() -> ServiceDef`, then `inspect_projection` does `.find(|p| p.name == name)` which returns the first alphabetical match only. The dogfood test would silently exercise only one of 8 projections. [VERIFIED: `list_projections.rs:76–89`, `inspect_projection.rs:56`]

- **Assuming seam 2 produces findings on `app/` projections:** Due to the `struct Model` naming convention in SeaORM entity files, `list_models` returns `name: "Model"` for all three `app/` entities. `"api_key" != "model"` → all seam 2 checks are `not_checked`. Seam 2 findings come ONLY from the poisoned fixture. [VERIFIED: `list_models.rs:148–160`]

- **Blanket-demoting all wrapper seams without evidence:** D-04 is explicitly evidence-driven — only seams that actually found nothing across both dogfood inputs are demoted. Run the tally first, write ACCEPTANCE.md, then make demotion changes.

- **Not updating the existing `next_steps_cap_at_10` test:** After changing `if result.len() == 10` to `== 5`, the existing test at line 1361–1368 asserts `steps.len() == 10` which will fail. Update it in the same commit.

- **Leaving doc comments at "cap 10" after the code changes:** Four locations must be updated atomically (see D-05 edit sites).

- **Multi-word service_name in poisoned fixture struct:** If the fixture uses `ServiceDef::new("poisoned_acceptance")` and `model_src_with_fields("PoisonedAcceptance", ...)`, the match check is `"poisoned_acceptance" == "poisonedacceptance"` — which is `false` because `"PoisonedAcceptance".to_lowercase()` is `"poisonedacceptance"` but `"poisoned_acceptance"` still has the underscore. Use single-word or use a struct name that exactly matches: `ServiceDef::new("dangling")` + `model_src_with_fields("Dangling", ...)` → `"dangling" == "dangling"`. [VERIFIED by tracing `list_models.rs:150` and `field_to_column_seam:296–309`]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Temp projection with model | Custom tempdir setup | `project_with_projection` + `add_model` | Already vetted, handles file structure correctly |
| SeaORM model source | Custom struct text | `model_src_with_fields(struct_name, fields)` | Private helper at line 992, already correct for `list_models::execute` parsing; outputs `DeriveEntityModel` derive |
| Finding assertions | Custom field-by-field checks | Assert `findings.len()` + `findings[0].subject` | SC-1 requires both size assertion AND exact subject name |

## Runtime State Inventory

Not applicable — greenfield test additions and code changes within existing Rust module. No stored data, live service config, OS-registered state, secrets, or build artifacts are affected. `.ferro/checkpoints/` cache entries from tests are transient (tempdir-scoped).

## Common Pitfalls

### Pitfall 1: `app/` projection name collision via `run_for`

**What goes wrong:** All 8 `app/` projection files export a function named `service_def`. Calling `run_for(app_root, "service_def", now)` invokes `inspect_projection → list_projections`, which collects all functions matching the `ServiceDef`-returning pattern. All 8 entries have `name == "service_def"`. `.find(|p| p.name == name)` returns the first entry (filesystem-order). Only one of 8 projections is actually checked.

**Why it happens:** The `app/` projection module pattern uses per-module file namespacing, while the checkpoint tool uses the function name as the key.

**How to avoid:** In the dogfood test, call seam functions directly per file (not through `run_for`).

**Warning signs:** Dogfood test produces results for only one projection.

### Pitfall 2: Model-name mismatch causing all-`not_checked` seam 2 on `app/`

**What goes wrong:** `list_models::execute` uses `node.ident.to_string()` to extract model names (`list_models.rs:150`). SeaORM entity files in `app/src/models/entities/` all define `pub struct Model` — so `list_models` returns `ModelDetails { name: "Model", ... }` for all three app models. `field_to_column_seam` matches `service_name.to_lowercase() == model.name.to_lowercase()` — `"api_key" == "model"` is `false`. Seam 2 produces `not_checked("source_model_unresolved")` for every `app/` projection.

**Why it happens:** SeaORM's entity-generation convention uses `struct Model` as a generic name, unlike the checkpoint's assumption that struct names are entity-specific.

**How to avoid:** Accept that seam 2 provides no findings on `app/` — rely on seams 1/3/4/5 for SC-2 compliance. The poisoned fixture (D-01) proves seam 2 independently.

**Warning signs:** All dogfood seam 2 results have `status: "not_checked"`, `reason: "source_model_unresolved"`.

### Pitfall 3: SC-1 under-specification

**What goes wrong:** SC-1 requires the test to assert that **exactly one** field appears as a finding, and that the finding names **exactly the planted field**. A test that only checks `!findings.is_empty()` passes even if the seam finds additional fields spuriously.

**How to avoid:** Assert both `result.findings.len() == 1` AND `result.findings[0].subject == "phantom_col"`. Both assertions are required for SC-1 compliance.

### Pitfall 4: Not updating the existing cap-at-10 test

**What goes wrong:** The existing test at line 1361–1368 asserts `steps.len() == 10` with 12 input findings. After changing `if result.len() == 10` to `== 5`, that test assertion fails.

**How to avoid:** In the same commit that changes the cap literal, update the existing test to assert `== 5`. Update or rename the function.

**Warning signs:** `cargo test` fails on `next_steps_cap_at_10` after the cap change.

### Pitfall 5: Demoting seams before running the dogfood tally

**What goes wrong:** D-04 is evidence-driven. Demoting before observing the tally may incorrectly demote a seam that produces findings (or fail to demote one that doesn't).

**How to avoid:** Run the dogfood test, record the per-seam tally in `196-ACCEPTANCE.md`, then make demotion changes. The report precedes the demotion.

### Pitfall 6: Missing doc-comment locations for cap 10→5

**What goes wrong:** Four locations say "cap 10". Missing any one leaves inconsistent documentation.

**How to avoid:** Update all four atomically:
1. `checkpoint_projection.rs:71` — `Verdict.next_steps` doc comment: `"cap 10"`
2. `checkpoint_projection.rs:737` — `aggregate_next_steps` doc: `"Cap at 10"` [VERIFIED: line 737]
3. `docs/src/agents/checkpoint-projection.md:61` — table row: `"capped at 10"` [VERIFIED]
4. `docs/src/agents/checkpoint-projection.md:125` — bullet: `"The list is capped at 10 entries"` [VERIFIED]

## Code Examples

### D-05: Cap reduction edit sites

```rust
// checkpoint_projection.rs:763 — before
if result.len() == 10 {

// after (with optional const)
const MAX_NEXT_STEPS: usize = 5;
// ...
if result.len() == MAX_NEXT_STEPS {
```

Docstring at line 737 (the `aggregate_next_steps` fn):
```
// before: "Dedup by `(subject, fix)`. Cap at 10."
// after:  "Dedup by `(subject, fix)`. Cap at 5."
```

Verdict doc at line 71:
```
// before: "Ranked, deduplicated actionable strings (failures before warnings; cap 10)."
// after:  "Ranked, deduplicated actionable strings (failures before warnings; cap 5)."
```

[VERIFIED: all line numbers confirmed by grep and direct read]

### D-04: Wrapper seam demotion pattern

If a wrapper seam produces zero findings across both dogfood inputs, replace its dispatch body:

```rust
// Before (example for projection_well_formed_seam):
fn projection_well_formed_seam(project_root: &Path, name: &str) -> SeamResult {
    match validate_projection::execute_single(project_root, name) {
        // ... dispatch ...
    }
}

// After (only if zero findings observed in tally):
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

Documentation string in `service.rs` (D-04 SC-4):
```
// Add to checkpoint_projection tool description:
"Seams 1 (projection_well_formed), 3 (action_to_route), 4 (rendered_view), 5 (props_to_contract)
are wrapper seams. Any seam marked not_checked-by-default was unproven across dogfood inputs
and is documented as such rather than silently passing."
```

### Test helper signatures (exact, verified)

```rust
// checkpoint_projection.rs:861
fn project_with_projection(name: &str, projection_src: &str) -> tempfile::TempDir
// Creates: {tempdir}/src/projections/{name}.rs with content projection_src

// checkpoint_projection.rs:871
fn add_model(tmp: &tempfile::TempDir, name: &str, model_src: &str)
// Creates: {tempdir}/src/models/{name}.rs with content model_src

// checkpoint_projection.rs:992 (private, in test module)
fn model_src_with_fields(struct_name: &str, fields: &[&str]) -> String
// struct_name: PascalCase (e.g. "Dangling"); table = struct_name.to_lowercase() + "s"
// fields: just the field names (all typed i64 in the generated source)
// GOTCHA: struct_name.to_lowercase() must exactly match the service_name
//         (no underscore vs. no-underscore mismatch)
```

[VERIFIED: read directly at those lines]

### Seam function signatures (exact, verified)

```rust
// checkpoint_projection.rs:241
fn field_to_column_seam(
    project_root: &Path,
    service_name: &str,
    display_name: &Option<String>,
    content: &str,
) -> SeamResult

// checkpoint_projection.rs:355
fn projection_well_formed_seam(project_root: &Path, name: &str) -> SeamResult

// checkpoint_projection.rs:409
fn action_to_route_seam(
    service: Option<&ferro_projections::ServiceDef>,
    routes: Option<&[list_routes::RouteInfo]>,
) -> SeamResult

// checkpoint_projection.rs:492
fn rendered_view_seam(project_root: &Path, name: &str) -> SeamResult

// checkpoint_projection.rs:580
fn props_to_contract_seam(project_root: &Path, service_name: &str) -> SeamResult
```

[VERIFIED: read directly at those lines]

## Empirical Analysis: `app/` Projection → Model Mapping

### `list_models` name extraction — VERIFIED

`list_models::execute` [VERIFIED: `list_models.rs:148–160`]:
- Uses `syn::Visit` to walk parsed Rust files
- In `visit_item_struct`: `let name = node.ident.to_string()` — the Rust struct ident
- Checks `has_model_derive` for `DeriveEntityModel` | `Model` | `Entity`
- Scans `src/models/` recursively (WalkDir) and `src/entities/` recursively

`app/src/models/entities/api_keys.rs` defines `pub struct Model` with `DeriveEntityModel` → `list_models` returns `ModelDetails { name: "Model", ... }`.

`app/src/models/api_key.rs` defines only `pub use super::entities::api_keys::*` and `type ApiKey = Model` — no struct with a derive macro. `list_models` finds `"Model"` from the entity file, not `"ApiKey"`.

**Conclusion: `list_models` returns `name: "Model"` for all three `app/` entity models.** The seam 2 matcher `service_name.to_lowercase() == model.name.to_lowercase()` evaluates `"api_key" == "model"` → `false`. Seam 2 is `not_checked("source_model_unresolved")` for all `app/` projections. [VERIFIED]

### All 9 `app/` projections

| File | `service_name` | Function name | Has actions | Model match |
|------|---------------|---------------|-------------|-------------|
| `api_key.rs` | `"api_key"` | `service_def` | None | No — `"api_key" != "model"` |
| `feedback_form.rs` | `"feedback_form"` | `service_def` | `submit_feedback` | No |
| `order.rs` | `"order"` | `service_def` | `submit`, `approve`, `ship` | No |
| `product.rs` | `"product"` | `service_def` | None | No |
| `revenue_dashboard.rs` | `"revenue_dashboard"` | `service_def` | None | No |
| `sales_analytics.rs` | `"sales_analytics"` | `service_def` | None | No |
| `todo.rs` | `"todo"` | `service_def` | None | No |
| `user.rs` | `"user"` | `service_def` | None | No |

Note: CONTEXT.md says "9 projections" but only 8 files found. The mod.rs lists 8 modules.

**Seam 2 (`field_to_column`):** `not_checked("source_model_unresolved")` for all 8. [VERIFIED]

**Seam 1 (`projection_well_formed`):** Expected mostly `pass` for well-formed projections; `order.rs` has a state machine + guards, which may trigger structural checks. Must run to confirm. [MEDIUM confidence]

**Seam 3 (`action_to_route`):** `feedback_form` (1 action) and `order` (3 actions) will be checked. `not_checked("route_list_unavailable")` if `list_routes` fails; `pass`/`fail` depending on whether handlers are registered. This is the primary potential finding source. [LOW confidence — requires checking `app/src/routes.rs`]

**Seam 4 (`rendered_view`):** `rendered_view_seam(root, name)` takes a projection name. For the dogfood test, the name to pass is the file stem (e.g. `"api_key"`) NOT `"service_def"` — `render_projection::execute` likely does its own projection lookup. [ASSUMED — needs verification]

**Seam 5 (`props_to_contract`):** `props_to_contract_seam(root, service_name)` — if `app/src/routes.rs` absent or has no matching Inertia routes, returns `not_checked("routes_file_missing")`. [MEDIUM confidence]

### SC-2 Risk Assessment

If the dogfood test produces zero findings (all seams `pass` or `not_checked`), SC-2 fails and acceptance is NO-GO. The design is revisited per CHK-10. This is a **real gate, not a formality.**

Viable mitigation paths if zero-findings result:
1. **Check `app/src/routes.rs`** — if `feedback_form`/`order` actions are not registered, seam 3 will produce `fail` findings. This is the most likely source.
2. **Run seam 4 on projections that have a rendered view** — if `render_projection` can find projections by file stem, `api_key.rs` may produce a spec-validation finding.
3. **Accept NO-GO and add a model to `app/`** — add an `order` model with intentionally different columns to generate a real seam 2 finding. But this changes the live consumer (D-02 says use the existing `app/`).

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Seam names: `schema_load`, `field_type_compat`, `action_binding`, `render_target` | Canonical: `projection_well_formed`, `field_to_column`, `action_to_route`, `rendered_view`, `props_to_contract` | Phase 195 (D-01) | Doc example in `checkpoint-projection.md:46` still shows old stub-era reason string `"not_implemented_phase_195"` — Phase 196 updates this |
| `next_steps` cap at 10 | Cap at 5 (Phase 196 D-05) | Phase 196 | 4 locations to update |
| Wrapper seams emit vacuous `pass` when unproven | Zero-finding seams demoted to `not_checked`-by-default (D-04) | Phase 196 | Coverage-honesty invariant enforced in production path |

**Deprecated artifacts in docs (Phase 196 must clean up):**
- `docs/src/agents/checkpoint-projection.md:46`: reason string `"not_implemented_phase_195"` — stale stub era; remove/replace with current actual not_checked reasons

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A2 | Seam 1 produces all-pass on `app/` projections (no structural errors) | Seam 1 analysis | If seam 1 finds errors → D-04 cannot demote seam 1; SC-2 trivially satisfied |
| A3 | Seam 3 produces findings on `feedback_form`/`order` if handlers not registered in `app/src/routes.rs` | Seam 3 analysis | If routes ARE registered, seam 3 passes → D-04 may demote seam 3; SC-2 still needs another source |
| A4 | Seam 5 produces `not_checked` on `app/` projections | Seam 5 analysis | If routes.rs exists with Inertia routes, seam 5 runs and may find mismatches |
| A5 | `rendered_view_seam(root, name)` accepts file stem (e.g. "api_key") as `name`, not function name | Seam 4 analysis | If render_projection uses function name lookup, file stem won't resolve; seam 4 → fail ("not found") |

**Confirmed (not assumed):**
- A1 (CONFIRMED): `list_models` returns `name: "Model"` for all `app/` entity models → seam 2 is `not_checked` on all `app/` projections. [VERIFIED: `list_models.rs:148–160`, `entities/api_keys.rs:11`]
- Function-name collision (CONFIRMED): all `app/` projections export `service_def` — `run_for` cannot be used in dogfood test. [VERIFIED: grep of all projection files]

**If A2, A3, A4 all produce zero findings simultaneously, `app/` run finds nothing — SC-2 fails.** This is the primary acceptance risk and must be tested before writing ACCEPTANCE.md.

## Open Questions

1. **Do `app/src/projections/*.rs` actions have registered routes?**
   - What we know: `feedback_form` has `submit_feedback`; `order` has `submit`, `approve`, `ship`
   - What's unclear: Whether `app/src/routes.rs` registers handlers matching these names (file not read in this session)
   - Recommendation: Read `app/src/routes.rs` as the first task in Wave 1 of the plan; the seam 3 finding probability determines SC-2 outcome before building tests.

2. **Does `rendered_view_seam` accept file stem or function name?**
   - What we know: `rendered_view_seam(root, name)` calls `render_projection::execute(root, name, None, None)`
   - What's unclear: Whether `render_projection::execute` resolves `name` as a file path stem or as the projection function name (via `list_projections`)
   - Recommendation: Check `render_projection::execute` signature and lookup logic before deciding what to pass in the dogfood test.

3. **Should the dogfood test assert SC-2 (`total_findings > 0`) or merely record findings?**
   - What we know: CONTEXT.md D-03 says ACCEPTANCE.md must have an explicit GO/NO-GO; SC-2 says a zero-finding run fails acceptance
   - What's unclear: Whether the test should `assert!` (compile-time enforcement) or `eprintln!` + manual review
   - Recommendation: Use `assert!` in the dogfood test — makes the acceptance gate machine-checkable and prevents accidental shipping of a vacuous checker.

## Environment Availability

Step 2.6: SKIPPED — this phase modifies existing Rust code only; no external tools, services, runtimes, databases, or CLIs beyond the standard Cargo toolchain are required.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `#[tokio::test]` (tokio in dev-dependencies) |
| Config file | `ferro-mcp/Cargo.toml` (existing; no new config needed) |
| Quick run command | `cargo test -p ferro-mcp checkpoint_projection -- --nocapture` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CHK-10 / SC-1 | Poisoned projection → `status: "fail"`, finding.subject == planted field, findings.len() == 1 | unit | `cargo test -p ferro-mcp poisoned_projection_dangling_field_acceptance` | ❌ Wave 0 |
| CHK-10 / SC-2 | Live `app/` consumer produces at least one finding (assert total > 0) | integration | `cargo test -p ferro-mcp dogfood_app_projections` | ❌ Wave 0 |
| CHK-10 / SC-3 | 7+ findings → `next_steps.len() == 5` (cap enforced) | unit | `cargo test -p ferro-mcp next_steps_cap_at_five` | ❌ Wave 0 |
| CHK-10 / SC-4 | Demoted seams documented as `not_checked`-by-default | manual code review | review `service.rs` + `checkpoint-projection.md` | N/A |

**Existing test to update:** `next_steps_cap_at_10` (line 1361) must be modified to assert `== 5` in the same commit as the cap change.

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-mcp checkpoint_projection -- --nocapture`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `poisoned_projection_dangling_field_acceptance` — new test in existing `mod tests` block
- [ ] `dogfood_app_projections` — new `#[tokio::test]` in existing `mod tests` block; needs `app/` path resolution via `CARGO_MANIFEST_DIR`
- [ ] `next_steps_cap_at_five` — new test OR update of existing `next_steps_cap_at_10`
- [ ] `196-ACCEPTANCE.md` — written after dogfood test run; records per-seam tally and GO/NO-GO verdict

## Security Domain

This phase adds no authentication, session management, access control, cryptography, or network-facing code. Security domain analysis not applicable.

## Sources

### Primary (HIGH confidence)

- `ferro-mcp/src/tools/checkpoint_projection.rs` (lines 1–1970) — full read; all edit sites, test helpers, seam function signatures verified
- `ferro-mcp/src/tools/list_models.rs` (lines 29–216) — `visit_item_struct` logic verified; `ModelDetails.name = node.ident.to_string()` confirmed
- `ferro-mcp/src/tools/list_projections.rs` (lines 40–99) — function-name extraction regex verified; name-collision mechanism confirmed
- `ferro-mcp/src/tools/inspect_projection.rs` (lines 48–96) — `.find(|p| p.name == name)` verified
- `app/src/projections/*.rs` (all 8 files) — field inventories, service_names, action names exact
- `app/src/models/entities/*.rs` (api_keys.rs, todos.rs, users.rs) — struct names (`Model`), column sets exact
- `.planning/phases/196-dogfood-acceptance-hardening/196-CONTEXT.md` — all decisions locked

### Secondary (MEDIUM confidence)

- `ferro-mcp/src/service.rs:1598–1619` — checkpoint_projection tool description (D-04 doc edit target)
- `docs/src/agents/checkpoint-projection.md` (full read) — cap-10 locations and stale stub artifacts identified

### Tertiary (LOW confidence)

- A3: `app/src/routes.rs` action registration — file not read in this session
- A5: `render_projection::execute` name resolution semantics — not traced to implementation

## Metadata

**Confidence breakdown:**
- Edit sites (cap, seam dispatch, doc comments): HIGH — exact line numbers verified
- Test helper signatures: HIGH — verified at exact lines
- `list_models` name extraction causing seam 2 not_checked on `app/`: HIGH — code path verified
- Function-name collision blocking `run_for` on `app/`: HIGH — code path verified
- Seam 3/4/5 behavior on `app/` (runtime findings): LOW–MEDIUM — projection files verified, route registration not traced

**Research date:** 2026-06-10
**Valid until:** Until `checkpoint_projection.rs`, `list_models.rs`, or `app/` projection/model files change
