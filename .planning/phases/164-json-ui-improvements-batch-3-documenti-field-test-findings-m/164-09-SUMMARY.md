---
phase: 164
plan: "09"
subsystem: ferro-mcp, ferro-json-ui
tags: [mcp-tool, validation, directive-audit, D-04, D-05]
dependency_graph:
  requires: [164-07]
  provides: [json_ui_validate_spec MCP tool, D-05-case-4-verified]
  affects: [ferro-mcp, ferro-json-ui]
tech_stack:
  added: []
  patterns: [two-stage MCP validation tool, structural-vs-catalog error separation]
key_files:
  created:
    - ferro-mcp/src/tools/json_ui_validate_spec.rs
  modified:
    - ferro-mcp/src/tools/mod.rs
    - ferro-mcp/src/service.rs
    - ferro-json-ui/src/spec.rs
decisions:
  - "D-05 case 4 already covered: validate_no_dangling checks key presence in elements map; $if-gated elements are present at parse time so no fix needed"
  - "Used $schema:ferro-json-ui/v2 (not schema_version) in test fixtures per actual spec format"
  - "Alert.variant='' reliably triggers CatalogError::PropsInvalid for the catalog-error test"
metrics:
  duration: "~12 minutes"
  completed: "2026-05-17"
  tasks: 3
  files: 4
---

# Phase 164 Plan 09: MCP validate-spec tool + D-05 directive audit Summary

**One-liner:** MCP `json_ui_validate_spec` tool surfaces structural vs catalog errors separately; D-05 case 4 confirmed covered with regression test.

## What Was Built

### D-04: `json_ui_validate_spec` MCP Tool

New tool at `ferro-mcp/src/tools/json_ui_validate_spec.rs`. Wraps `Spec::from_json` and `global_catalog().validate` in a two-stage pipeline:

1. **Structural validation** (`Spec::from_json`) — catches missing root, dangling child refs, depth violations, directive errors, footer ID gaps. On failure: populates `structural_errors`, returns early (catalog step skipped).
2. **Catalog validation** (`global_catalog().validate`) — catches per-component prop schema violations (bad enum variant, missing required field, bad type). On failure: populates `catalog_errors`.

**MCP surface:**

| Field | Type | Notes |
|-------|------|-------|
| Tool name | `json_ui_validate_spec` | Registered in service.rs |
| Input param | `spec_json: String` | Full JSON-UI v2 spec as string |
| Response | `ValidateResponse` | See below |

**`ValidateResponse` shape:**

```rust
pub struct ValidateResponse {
    pub valid: bool,                    // true iff both error vecs empty
    pub structural_errors: Vec<String>, // from Spec::from_json
    pub catalog_errors: Vec<String>,    // from Catalog::validate
    pub warnings: Vec<String>,          // reserved; currently always empty
}
```

Four tests shipped:
- `accepts_valid_spec` — minimal valid spec returns `valid: true`
- `reports_structural_error_on_missing_root` — `DanglingRootMissing` in structural_errors
- `reports_catalog_error_on_bad_variant` — `Alert.variant=""` in catalog_errors, structural_errors empty
- `reports_both_vecs_addressable_on_any_spec` — confirms response shape is always well-formed

### D-05 Case 4: `validate_no_dangling` Audit

**Audit conclusion: Case 4 already covered (no code change needed).**

The four D-05 directive validation cases:

| Case | Validator | Error | Status |
|------|-----------|-------|--------|
| 1. `$each.path` resolves to array | `validate_directives` | `EachPathNotArray` | Existing |
| 2. `$if.path` resolves cleanly | `validate_directives` | `IfPathMissing` | Existing |
| 3. No circular refs in templated elements | `validate_directives` | `NestedEach` / `MismatchedEach` | Existing |
| 4. Children refs to absent elements allowed when target has `$if` | `validate_no_dangling` | — | **Already covered** |

**Why case 4 is already covered:** `validate_no_dangling` checks `!elements.contains_key(child)`. Elements with `$if` ARE present in the elements map at parse time — `$if` only removes them at resolve time (per-request). The structural validator cannot observe per-request data, so it correctly accepts children references to `$if`-gated elements.

**Regression test added** (`validate_allows_children_ref_to_if_gated_element` in `ferro-json-ui/src/spec.rs`): constructs a spec where `parent.children = ["child"]` and `child` has `$if: {path, operator, value}`. Asserts `Spec::from_json` succeeds.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| Task 1 | `74200d93` | feat(164-09): add json_ui_validate_spec MCP tool (D-04) |
| Task 2 | `ab5edcb1` | test(164-09): D-05 case 4 regression — children ref to $if-gated element |

## Deviations from Plan

**1. [Rule 1 - Bug] Test fixture format correction**
- **Found during:** Task 1 (writing tests)
- **Issue:** Plan's example test fixtures used `"schema_version": "ferro-json-ui/v1"` — the actual spec field is `"$schema": "ferro-json-ui/v2"` (renamed in Phase 162)
- **Fix:** All four test fixtures use `"$schema": "ferro-json-ui/v2"` per the current `Spec` serde shape
- **Files modified:** `ferro-mcp/src/tools/json_ui_validate_spec.rs`
- **Commit:** `74200d93`

**2. [Rule 1 - Bug] Test 3 fixture uses reliable catalog-error trigger**
- **Found during:** Task 1 (test 3)
- **Issue:** Plan's test 3 used `Alert.variant=""` which IS a reliable catalog error trigger (empty string not in AlertVariant enum). Kept as-is.
- **Fix:** No change needed — confirmed working.

**3. Test 4 renamed for clarity**
- **Found during:** Task 1
- **Issue:** Plan named test 4 `reports_both_when_both_present` but a single spec cannot simultaneously fail structural AND catalog validation (structural failure skips catalog). Renamed to `reports_both_vecs_addressable_on_any_spec` with a comment explaining the constraint.
- **Files modified:** `ferro-mcp/src/tools/json_ui_validate_spec.rs`

## Note for Plan 11 (audit) and Plan 12 (COMPLETED.md)

The `json_ui_validate_spec` MCP tool is now available as a self-verification mechanism during the v1-deletion-readiness sweep (Plan 11). Agents can call it with any spec candidate to get the same diagnostics the framework produces at server startup, distinguishing structural failures from catalog-validation failures.

D-05 audit conclusion for COMPLETED.md: all four directive validation cases are covered in the shipped validator; case 4 confirmed via regression test `validate_allows_children_ref_to_if_gated_element`.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. The `json_ui_validate_spec` tool is read-only — it takes a string parameter and returns diagnostic text. Error messages use `SpecError::Display` and `CatalogError::Display` which produce diagnostic text containing element IDs and brief messages, consistent with the existing loader logs.

## Self-Check: PASSED

- `ferro-mcp/src/tools/json_ui_validate_spec.rs` — FOUND
- commit `74200d93` (feat D-04 MCP tool) — FOUND
- commit `ab5edcb1` (test D-05 case 4 regression) — FOUND
- `cargo fmt --all -- --check` — PASSED
- `cargo clippy --all --all-targets -- -D warnings` — PASSED
- `cargo test --all-features` — PASSED (zero failures)
