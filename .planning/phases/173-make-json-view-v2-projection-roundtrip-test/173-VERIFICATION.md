---
phase: 173-make-json-view-v2-projection-roundtrip-test
verified: 2026-06-09T18:00:00Z
status: passed
score: 5/5
overrides_applied: 1
overrides:
  - must_have: "ferro make:json-view uses catalog.component_schema() for per-component structured output"
    reason: "D-05 (CONTEXT.md locked decision): the deterministic builder selects components via FieldMeaning dispatch without any per-component LLM call, making component_schema() unnecessary. SC1 is satisfied vacuously — the write-gate catalog.json_schema() validation (SC2) covers the full spec shape. No LLM pass was invented to exercise component_schema()."
    accepted_by: "alberto"
    accepted_at: "2026-06-09T18:00:00Z"
human_verification:
  - test: "Run ferro make:json-view <name> --description '<NL description>' against a configured FERRO_AI_* provider"
    expected: "Generated src/views/{name}.json reflects the NL description, root element is DataTable for a list-oriented description, and a Money-typed field (if scaffold_core identifies one) carries format: currency"
    why_human: "Live NL->ServiceDef quality depends on a real AI provider key; not automatable in CI without secrets. Mirrors Phase 171 SC4/SC6 live-quality gate precedent (D-07)."
---

# Phase 173: make:json-view v2 + Projection-Roundtrip Test — Verification Report

**Phase Goal:** Upgrade `ferro make:json-view` to consume a `ServiceDef` and render it via the existing deterministic renderer (`Spec::from_service_def`), and ship the projection-roundtrip proof test (NL → ServiceDef → rendered JSON-UI spec → schema-validated) as the structural proof that AI is a first-class projection consumer. Integration + a test, NOT greenfield.

**Verified:** 2026-06-09
**Status:** passed
**Re-verification:** No — initial authoritative verification (prior file was an in-execution SC record, not a GSD-format verification)

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `ferro make:json-view` uses `catalog.component_schema()` for per-component structured output | PASSED (override) | D-05 (CONTEXT.md): deterministic builder selects components via FieldMeaning dispatch without any LLM call; component_schema() is the per-component schema for LLM-constrained generation and is unnecessary when no LLM selects components. The write-gate `catalog.json_schema()` validates the full spec shape. No LLM pass was invented to satisfy SC1. |
| 2 | Generated views are v2 flat specs validated against `catalog.json_schema()` before write | VERIFIED | `render_service_def` in `make_json_view.rs` calls `Spec::from_service_def` (which internally calls `global_catalog().validate` per builder.rs:111) then re-parses via `Spec::from_json` write-gate (D-02, line 253). Roundtrip test also calls `global_catalog().validate(&spec)` explicitly. |
| 3 | `make:json-view` consumes a ServiceDef; component selection driven by FieldMeaning/Intent, not LLM re-prompting | VERIFIED | `make_json_view.rs` routes generation through `derive_intents(&service)` → `Spec::from_service_def`. NL path uses `scaffold_core` only for ServiceDef construction; `--from-service-json` path uses no LLM at all. `grep -c "Spec::from_service_def" make_json_view.rs` = 3. |
| 4 | No v1 `JsonUiView` types appear in the generation pipeline | VERIFIED | `grep -c "JsonUiView" ferro-cli/src/commands/make_json_view.rs` = 0. The old `generate_with_ai`, `build_json_view_pass1`, `build_json_view_pass2` are deleted. `grep -c "generate_with_ai\|build_json_view_pass1\|build_json_view_pass2" make_json_view.rs` = 0. |
| 5 | Projection-roundtrip test at `ferro-ai/tests/projection_roundtrip.rs` passes via the ServiceDef-aware path (cannot pass via generic fallback) | VERIFIED | Test exists, contains `Spec::from_service_def` (2 occurrences), and asserts `has_currency` (5 occurrences of "currency"). The assertion that `FieldMeaning::Money` produces a DataTable column with `"format": "currency"` (via `component_map.rs:277: FieldMeaning::Money => Some(ColumnFormat::Currency)`) pins the ServiceDef-aware dispatch path — a generic schema-normalization fallback cannot produce this deterministic observable. |

**Score:** 5/5 truths verified (1 via planned override)

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-cli/src/commands/make_json_view.rs` | ServiceDef-driven generation path feeding `Spec::from_service_def`; old two-pass deleted; catalog write-gate preserved | VERIFIED | File exists (315 lines). Contains `Spec::from_service_def` (3x), `scaffold_core` (3x), `Spec::from_json` write-gate. Contains zero occurrences of `generate_with_ai`, `build_json_view_pass1`, `build_json_view_pass2`, `JsonUiView`. |
| `ferro-cli/src/main.rs` | `MakeJsonView` clap variant with `from_service_json` arg wired to `run()` | VERIFIED | `from_service_json` appears 3 times (declaration at line 167, destructure at line 615, pass-through at line 617). |
| `ferro-ai/tests/projection_roundtrip.rs` | Offline projection-roundtrip proof test; `Spec::from_service_def` + `"currency"` path-proof assertion | VERIFIED | File exists (71 lines). Contains `Spec::from_service_def`, `FieldMeaning::Money`, `global_catalog().validate`, `assert_eq!(root.type_name, "DataTable")`, and the `has_currency` assertion. Offline/deterministic: no network, no LLM key. |
| `ferro-ai/Cargo.toml` | `ferro-json-ui` dev-dependency with `features = ["projections"]` | VERIFIED | `[dev-dependencies]` contains `ferro-json-ui = { path = "../ferro-json-ui", version = "0.2", features = ["projections"] }`. |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| `make_json_view.rs` (NL path) | `ferro_mcp::tools::ai_scaffold::scaffold_core` | `rt.block_on(scaffold_core(&desc_owned, &cwd))` inside `run()` | WIRED | `grep -c "scaffold_core" make_json_view.rs` = 3; pattern present at the block_on call site |
| `make_json_view.rs` (render) | `ferro_json_ui::Spec::from_service_def` | `derive_intents(&service)` → `Spec::from_service_def(&service, &intents, &ctx)` in `render_service_def` | WIRED | `grep -c "Spec::from_service_def" make_json_view.rs` = 3; `derive_intents` also present |
| `ferro-ai/tests/projection_roundtrip.rs` | `ferro_json_ui::Spec::from_service_def` | `derive_intents(&fixture)` → `Spec::from_service_def(&service, &intents, &ctx)` → `global_catalog().validate` | WIRED | Both `Spec::from_service_def` and `global_catalog()` present in test; catalog validated inline |
| `ferro-ai/tests/projection_roundtrip.rs` | `ColumnFormat::Currency` observable | `cols.iter().any(|c| c.get("format") == Some("currency"))` | WIRED | `has_currency` assertion present; pins `component_map.rs:277` dispatch |

---

## Data-Flow Trace (Level 4)

This phase produces a CLI command and a test, not a data-rendering component. Level 4 data-flow trace applies to the `render_service_def` helper which renders a `ServiceDef` to a JSON-UI spec string.

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `render_service_def` helper | `spec` (from `Spec::from_service_def`) | `ServiceDef` (from `scaffold_core` or `--from-service-json`) → `derive_intents` → `Spec::from_service_def` | Yes — deterministic render from typed `ServiceDef` fields | FLOWING |
| `projection_roundtrip.rs` | `spec` | `invoice_fixture()` (in-process) → `derive_intents` → `Spec::from_service_def` | Yes — real `FieldMeaning::Money` → `ColumnFormat::Currency` dispatch | FLOWING |

---

## Behavioral Spot-Checks

| Behavior | Check | Result | Status |
|----------|-------|--------|--------|
| Old two-pass functions deleted | `grep -c "generate_with_ai\|build_json_view_pass1\|build_json_view_pass2" make_json_view.rs` | 0 | PASS |
| `Spec::from_service_def` present in CLI | `grep -c "Spec::from_service_def" make_json_view.rs` | 3 | PASS |
| No v1 `JsonUiView` types | `grep -c "JsonUiView" make_json_view.rs` | 0 | PASS |
| `--from-service-json` arg wired | `grep -c "from_service_json" ferro-cli/src/main.rs` | 3 | PASS |
| `ferro-json-ui` dev-dep in ferro-ai | `grep -c "ferro-json-ui" ferro-ai/Cargo.toml` | 1 | PASS |
| Roundtrip test contains path-proof assertion | `grep -c "currency" ferro-ai/tests/projection_roundtrip.rs` | 5 | PASS |
| Commits present in git log | `bdebfda1`, `dd484d13`, `7694932d`, `d511a0dd` | All found | PASS |

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| AICLI-04 | 173-01-PLAN.md | `make:json-view` v2 — ServiceDef-driven rendering path | SATISFIED | `make_json_view.rs` routes generation through `Spec::from_service_def`; old two-pass deleted; `--from-service-json` flag added |
| AICLI-06 | 173-02-PLAN.md | Projection-roundtrip proof test (v12.1 capstone) | SATISFIED | `ferro-ai/tests/projection_roundtrip.rs` exercises ServiceDef fixture → `Spec::from_service_def` → catalog validation → DataTable + currency path-proof |

---

## SC1 Resolution (Vacuous — D-05)

**Criterion (ROADMAP SC1):** `catalog.component_schema()` is used for per-component structured output.

**Resolution:** SC1 is satisfied by design. The deterministic builder (`Spec::from_service_def` in `ferro-json-ui/src/projection/builder.rs`) selects components via `FieldMeaning` → component dispatch (`lookup_meaning` in `component_map.rs`) and validates the complete produced spec against `catalog.json_schema()` (SC2, exercised internally at `builder.rs:111`). `catalog.component_schema()` is the per-component schema for LLM-constrained generation — it is used when an LLM is being prompted to produce props for a named component. Since the deterministic builder performs no per-component LLM call, no per-component schema prompt is required.

The write-gate catalog validation (`catalog.json_schema()` exercised by `Spec::from_service_def` internally and by `Spec::from_json` re-parse in `render_service_def`) covers the full spec shape, making the per-component `component_schema()` path redundant in the deterministic pipeline.

**Decision reference:** D-05 (CONTEXT.md) — "If the deterministic path needs no per-component LLM call, SC1's component_schema() clause is satisfied vacuously and the planner documents that in VERIFICATION.md rather than inventing an LLM pass to use it."

---

## SC5 Path-Proof (SC5 — Currency Assertion)

The test at `ferro-ai/tests/projection_roundtrip.rs` seeds a `ServiceDef` with a `FieldMeaning::Money` field (`total`, `DataType::Float`). `Spec::from_service_def` deterministically maps this to a DataTable column with `ColumnFormat::Currency` via `build_column_for_field` in `component_map.rs:277`:

```rust
FieldMeaning::Money => Some(ColumnFormat::Currency),
```

The test asserts:

```rust
let has_currency = cols.iter()
    .any(|c| c.get("format").and_then(|f| f.as_str()) == Some("currency"));
assert!(has_currency, "Money field must produce a currency-formatted column ...");
```

This assertion cannot pass via any path that does not execute the `FieldMeaning::Money → ColumnFormat::Currency` dispatch. A generic NL→spec LLM call would not reliably produce the `"currency"` format string.

**Automated gate:** `cargo test -p ferro-ai --test projection_roundtrip`

---

## D-07 Manual Gate — Live NL→ServiceDef Quality

**Disposition:** Open (not blocking the automated phase gate).

**Requirement:** Verify that `ferro make:json-view <name> --description "<NL>"` against a configured `FERRO_AI_*` provider produces a view that reflects the description and selects sensible components.

**Instructions:**

1. Configure an AI provider (e.g., `FERRO_AI_API_KEY=<key> FERRO_AI_MODEL=claude-3-5-haiku-20241022`).
2. Run: `ferro make:json-view orders --description "A list of customer orders showing order number, total amount, and delivery status"`
3. Inspect the generated `src/views/orders.json`:
   - The root element should be a `DataTable`.
   - Column names should reflect the description's fields (order number, total, status).
   - The `total` column, if typed as Money by `scaffold_core`, should carry `"format": "currency"`.
4. Sign off: `[ ] Confirmed — <date> <initials>`

**Precedent:** Mirrors Phase 171 SC4/SC6 live quality manual gate pattern. The automated roundtrip test exercises the deterministic ServiceDef → Spec path. The live quality gate confirms that the NL → ServiceDef (`scaffold_core`) stage produces semantically sensible `ServiceDef` values for real descriptions.

---

## Anti-Patterns Found

No blockers. No stubs. The static template fallback in `make_json_view.rs` is intentional behavior (produces a valid spec that can be manually edited), not a stub.

| File | Pattern | Severity | Assessment |
|------|---------|----------|------------|
| `make_json_view.rs` | `templates::json_view_template(...)` fallback on AI/render failure | Info | Intentional — documented fallback behavior, not a stub |

---

## Human Verification Required

### 1. Live NL→ServiceDef Quality (D-07)

**Test:** Run `ferro make:json-view orders --description "A list of customer orders showing order number, total amount, and delivery status"` with a configured `FERRO_AI_*` provider.

**Expected:** The generated `src/views/orders.json` root element is `DataTable`, columns reflect the description's fields, and a Money-typed `total` field carries `"format": "currency"`.

**Why human:** Live AI provider key required; CI is key-free by design. This gate mirrors Phase 171 SC4/SC6 precedent.

---

## Gaps Summary

No gaps. All automated success criteria pass. D-07 is an intentional, pre-planned manual gate (D-07 decision locked in CONTEXT.md) that does not block phase completion — it mirrors the identical pattern from Phase 171. SC1 deviation is resolved via the D-05 locked decision with an applied override.

---

| SC | Status | Gate |
|----|--------|------|
| SC1 (component_schema) | PASSED (override) — vacuous, no LLM pass; D-05 locked decision | Override applied |
| SC2 (catalog write-gate) | VERIFIED | Automated: `render_service_def` + `global_catalog().validate` in builder + roundtrip test |
| SC3 (deterministic component selection) | VERIFIED | `cargo test -p ferro-ai --test projection_roundtrip` |
| SC4 (no JsonUiView) | VERIFIED | `grep -c "JsonUiView" ferro-cli/src/commands/make_json_view.rs` == 0 |
| SC5 (path-proof: currency assertion) | VERIFIED | `cargo test -p ferro-ai --test projection_roundtrip` |
| D-07 (live NL quality) | Open — manual gate, non-blocking | See instructions above |

---

_Verified: 2026-06-09T18:00:00Z_
_Verifier: Claude (gsd-verifier)_
