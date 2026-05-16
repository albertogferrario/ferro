---
phase: 163-json-ui-improvements-batch-2-cassa-and-calendario-field-test
verified: 2026-05-16T00:00:00Z
status: human_needed
score: 13/14
overrides_applied: 0
human_verification:
  - test: "Manually test 'ferro json-ui:migrate-v1 ferro-cli/tests/fixtures/migrate_v1/in_auth.rs --dry-run' and verify the proposed JSON spec places login-form and its children reachable from root."
    expected: "Either (a) root wraps page-title and login-form in a Group/Fragment container, or (b) multi-root handlers are rejected as Unsupported with a TODO marker. Currently root is 'page-title' and login-form/email/password/submit are orphaned (unreachable)."
    why_human: "WR-01 from code review: the codemod emits a structurally correct JSON file (it passes serde validation) but the output spec is semantically wrong — elements are unreachable from root. The integration test passes only because the fixture itself contains the bug. The correct fix (Option A or B from the review) requires a human decision about which repair to make."
---

# Phase 163: JSON-UI Improvements Batch 2 — Verification Report

**Phase Goal:** Ship the iteration-and-ergonomics slice of gestiscilo Phase 138 FRICTION.md. Adds two element-level directives (`$each` for homogeneous list iteration, `$if` for conditional emission), a validator gate for malformed directives, an ergonomic nested-tree `SpecBuilder` layer for truly heterogeneous Rust-side construction, an AST-based `ferro json-ui:migrate-v1` codemod, and MCP catalog reflection.
**Verified:** 2026-05-16T00:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | EachDirective struct exists in ferro-json-ui/src/spec.rs with path: String and as_: String (serde-renamed "as") | VERIFIED | `grep -c "pub struct EachDirective" ferro-json-ui/src/spec.rs` = 1; `each_directive_round_trips` test passes |
| 2 | Element struct gains optional each: Option<EachDirective> and if_: Option<Visibility> fields with serde renames | VERIFIED | Both fields confirmed in spec.rs; skip_serializing_if on both; 7 round-trip tests pass |
| 3 | expand_directives is a public function in ferro-json-ui/src/resolve.rs that mutates Spec in place | VERIFIED | `grep -c "pub fn expand_directives" ferro-json-ui/src/resolve.rs` = 1 |
| 4 | JsonUi::resolve runs expand_directives BEFORE resolve_actions and resolve_expressions | VERIFIED | framework/src/json_ui/mod.rs shows exact ordering: expand_directives → resolve_actions → resolve_expressions |
| 5 | $if-falsy elements deleted, $each-N elements expand with correlated children — all 12 expand_ unit tests pass | VERIFIED | `cargo test -p ferro-json-ui --lib expand_` = 12 passed |
| 6 | SpecError gains five new variants: EachPathNotArray, IfPathMissing, EachAsReservedName, NestedEach, MismatchedEach | VERIFIED | `grep -c "EachPathNotArray\|IfPathMissing\|EachAsReservedName\|NestedEach\|MismatchedEach" ferro-json-ui/src/spec.rs` = 25; all 17 validate_ tests pass |
| 7 | validate_directives called between validate_no_dangling and detect_cycle | VERIFIED | Confirmed in validate_structure body: validate_no_dangling → validate_directives → validate_footer_ids → detect_cycle → check_depth |
| 8 | NestedElement struct + SpecBuilder::element_nested + flatten_nested in spec.rs | VERIFIED | `grep -c "pub struct NestedElement"` = 1; `pub fn element_nested` = 1; `fn flatten_nested` = 2; all 14 nested_ tests pass |
| 9 | D-04 reuse mandate: Visibility::evaluate is the SOLE predicate evaluator for $if | VERIFIED | `grep -c "fn evaluate_if\|fn if_evaluate\|fn check_if_predicate" ferro-json-ui/src/resolve.rs` = 0; `.evaluate(` = 1 |
| 10 | JsonUiCatalog.directives field with DirectiveInfo carrying $each and $if entries | VERIFIED | DirectiveInfo struct confirmed in json_ui_catalog.rs; all 3 MCP directive tests pass (219 total) |
| 11 | ferro json-ui:migrate-v1 AST codemod registered; single-file; idempotent; dry-run; TODO markers | VERIFIED | Main.rs has `json-ui:migrate-v1`; syn::parse_file used (AST-based); idempotency check present; dry_run branch confirmed; TODO marker text matches D-09; 5 integration tests pass |
| 12 | Codemod correctly handles single-root handlers (login_form fixture produces valid reachable spec) | FAILED | out_auth_login_form.json has root="page-title" with login-form/email/password/submit unreachable from root. WR-01 from review: multi-root handler silently orphans all elements except the first. Test passes only because the fixture encodes the bug. |
| 13 | docs/src/json-ui/spec-construction.md exists with four-quadrant decision rubric | VERIFIED | File exists; `grep -c "Decision rubric\|Heterogeneous runtime construction\|Static spec\|Homogeneous iteration\|Conditional emission"` = 12 |
| 14 | docs/src/json-ui/expressions.md extended with $each and $if sections | VERIFIED | `grep -c "^## \$each\|^## \$if"` = 2; all five validator error names present |

**Score:** 13/14 truths verified

### Deferred Items

No items deferred to later phases. AISSE-02 (StreamText component) is mapped to Phase 163 in REQUIREMENTS.md but was not part of this phase's CONTEXT, PLAN, or scope. It is an orphaned requirement mapping — see Requirements Coverage section.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-json-ui/src/spec.rs` | EachDirective + Element fields + SpecError variants + NestedElement + validate_directives | VERIFIED | All present; 473 unit tests pass |
| `ferro-json-ui/src/resolve.rs` | expand_directives public function + 12 unit tests | VERIFIED | 12 expand_ tests pass |
| `ferro-json-ui/src/lib.rs` | Re-exports expand_directives | VERIFIED | `grep -c "expand_directives" ferro-json-ui/src/lib.rs` = 1 |
| `framework/src/json_ui/mod.rs` | expand_directives wired FIRST in resolve pipeline | VERIFIED | `grep -c "expand_directives" framework/src/json_ui/mod.rs` = 5 |
| `ferro-json-ui/tests/directives_e2e.rs` | 4 end-to-end directive tests | VERIFIED | All 4 e2e tests pass |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | DirectiveInfo struct + directives field + 3 tests | VERIFIED | All 3 MCP directive tests pass |
| `ferro-cli/src/commands/json_ui_migrate_v1.rs` | Codemod: pub fn run, AST-based, idempotent, dry-run | VERIFIED | File exists; syn used; all behaviors present |
| `ferro-cli/src/main.rs` | json-ui:migrate-v1 subcommand registered | VERIFIED | `grep -c "json-ui:migrate-v1" ferro-cli/src/main.rs` = 1 |
| `ferro-cli/tests/json_ui_migrate_v1.rs` | 5 fixture-driven integration tests | VERIFIED | All 5 codemod integration tests pass |
| `ferro-cli/tests/fixtures/migrate_v1/` | 4 fixture files (in_auth.rs, out_auth.rs, out_auth_login_form.json, in_with_runtime_branch.rs) | VERIFIED | All 4 fixture files exist |
| `docs/src/json-ui/spec-construction.md` | Four-quadrant decision rubric | VERIFIED | All rubric sections present |
| `docs/src/json-ui/expressions.md` | $each + $if sections appended | VERIFIED | Both sections present with all validator error names |
| `docs/src/SUMMARY.md` | spec-construction.md linked | VERIFIED | `grep -c "spec-construction.md" docs/src/SUMMARY.md` = 1 |
| `CHANGELOG.md` | Phase 163 Unreleased entry | VERIFIED | $each, $if, expand_directives, migrate-v1, EachPathNotArray all present |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| framework/src/json_ui/mod.rs JsonUi::resolve | ferro-json-ui/src/resolve.rs expand_directives | imported and called before resolve_actions | VERIFIED | Ordering confirmed: expand_directives → resolve_actions → resolve_expressions |
| ferro-json-ui/src/resolve.rs expand_if | ferro-json-ui/src/visibility.rs Visibility::evaluate | sole predicate evaluator (D-04) | VERIFIED | `.evaluate(data)` called once; no parallel evaluator |
| ferro-json-ui/src/spec.rs validate_structure | ferro-json-ui/src/spec.rs validate_directives | called between validate_no_dangling and detect_cycle | VERIFIED | Call sequence confirmed in validate_structure body |
| ferro-json-ui/src/spec.rs SpecBuilder::element_nested | ferro-json-ui/src/spec.rs flatten_nested | recursive walk that emits flat HashMap | VERIFIED | element_nested calls flatten_nested; confirmed in spec.rs |
| ferro-mcp/src/tools/json_ui_catalog.rs JsonUiCatalog | ferro-json-ui/src/spec.rs SpecError variants | DirectiveInfo.validation_errors names the variants | VERIFIED | EachPathNotArray, IfPathMissing, EachAsReservedName present in json_ui_catalog.rs |
| ferro-cli/src/main.rs Commands::JsonUiMigrateV1 | ferro-cli/src/commands/json_ui_migrate_v1.rs::run | match arm calls run(file, dry_run) | VERIFIED | json_ui_migrate_v1::run wired in match arm |

### Data-Flow Trace (Level 4)

The directives are resolve-time transforms, not render-time data sources. The `expand_directives` function reads `spec.data`, expands elements, then the existing render pipeline renders the expanded (static) elements. Verified via the 4 e2e tests that confirm ORD-1/ORD-2/ORD-3 and BADGE_ONE/BADGE_TWO appear in rendered HTML output. Data flow is real (not hardcoded) — the test data is in the spec JSON and must traverse the full expand → render pipeline to appear in output.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| 473 ferro-json-ui unit tests pass | `cargo test -p ferro-json-ui --all-features` | 473 passed | PASS |
| 12 expand_ tests (D-01 through D-03 resolve behaviors) | `cargo test -p ferro-json-ui --lib expand_` | 12 passed | PASS |
| 17 validate_ tests (D-12 validator gates) | `cargo test -p ferro-json-ui --lib validate_` | 17 passed | PASS |
| 14 nested_ tests (D-06/D-07 SpecBuilder layer) | `cargo test -p ferro-json-ui --lib nested_` | 14 passed | PASS |
| 4 e2e directive tests (full pipeline) | `cargo test -p ferro-json-ui --test directives_e2e` | 4 passed | PASS |
| 219 ferro-mcp tests including 3 DirectiveInfo tests | `cargo test -p ferro-mcp` | 219 passed | PASS |
| 5 codemod integration tests | `cargo test -p ferro-cli --test json_ui_migrate_v1` | 5 passed | PASS |
| cargo clippy clean | `cargo clippy --all --all-targets -- -D warnings` | No warnings | PASS |
| cargo fmt clean | `cargo fmt --all -- --check` | No formatting issues | PASS |
| ferro-rs framework builds | `cargo build -p ferro-rs` | Builds successfully | PASS |

### Requirements Coverage

| Requirement | Description | Status | Evidence |
|-------------|-------------|--------|----------|
| AISSE-02 (orphaned) | ferro-json-ui provides a `StreamText` component for SSE token streaming | ORPHANED | StreamText component does not exist in ferro-json-ui. REQUIREMENTS.md maps AISSE-02 to Phase 163 but no plan in this phase claims or mentions it. The phase CONTEXT (D-01 through D-13) covers iteration directives and ergonomics — StreamText/SSE is outside scope. AISSE-01 shows the same pattern (mapped to Phase 162, not in Phase 162 plans). This appears to be a speculative mapping in REQUIREMENTS.md that was never part of Phase 163's actual design. No later phase (164, 165) explicitly addresses StreamText either. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `ferro-cli/src/commands/json_ui_migrate_v1.rs` | ~342 | Multi-root handler silently sets root to `top_ids.first()`, orphaning all other top-level nodes and their subtrees | Warning (WR-01 from review) | The codemod emits a structurally valid JSON spec that passes `Spec::from_json` validation, but `login-form`, `email`, `password`, `submit` are unreachable from root `page-title`. Users running the codemod on multi-root v1 handlers will see a page with only the first element rendered. The integration test passes because the expected fixture also encodes the bug. |
| `ferro-cli/src/commands/json_ui_migrate_v1.rs` | ~682 | `src.find(format!("fn {handler_name}("))` substring match can hit wrong function if handler names share a prefix | Warning (WR-02 from review) | Low impact in practice (handler names are rarely prefixes of each other), but nondeterministic across HashMap iteration order. |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | 236-259 | `BUILDER_API` constant omits `element_nested` and `NestedElement`; also omits `$each`/`$if` from Element shape description | Warning (WR-03 from review) | Agents reading the MCP catalog have no way to discover `element_nested` or `NestedElement`. The D-13 mandate required MCP reflection of new surface; directives are reflected via the `directives` field, but the builder API documentation gap means agents can only discover 3 of 4 rubric quadrants via MCP. |

### Human Verification Required

#### 1. Codemod Multi-Root Handler Fix (WR-01)

**Test:** Run `cargo run -p ferro-cli -- json-ui:migrate-v1 ferro-cli/tests/fixtures/migrate_v1/in_auth.rs --dry-run` and inspect the proposed JSON spec.

**Expected:** The spec should either (a) wrap both top-level nodes (`page-title` and `login-form`) in a synthetic `Group` root container, making all elements reachable from root, OR (b) reject multi-root handlers as `Unsupported` and emit the TODO marker, producing no spec. Currently the codemod produces a spec where `login-form`, `email`, `password`, and `submit` are in `elements` but unreachable from root `"page-title"` — a user running this would see a page with only a bare PageHeader, no form.

**Why human:** The review identified two viable fixes (Option A: Group wrapper, Option B: reject as Unsupported). The choice affects the codemod's output contract and the expected fixture content. Either Option B is the safer choice per the review (since `Group`/`Fragment` was explicitly NOT added in Phase 163). After the human decision, the fixture `out_auth_login_form.json` and the `codemod_one_handler_emits_spec_and_rewrites_controller` test must be updated to match.

### Gaps Summary

One truth FAILED (Truth 12): the codemod's multi-root handling is semantically incorrect. The codemod produces a JSON spec where elements beyond the first top-level node are unreachable from root. This is a correctness issue, not just a style concern — users would see incomplete pages.

The REQUIREMENTS.md maps AISSE-02 (StreamText component) to Phase 163, but this requirement was never part of the phase's locked design decisions (D-01 through D-13). It is an orphaned mapping. No plan claimed it, no code implements it, and no later phase explicitly schedules it.

Two other issues from the review (WR-02: substring handler-name matching, WR-03: BUILDER_API omitting NestedElement) are present but advisory-level. They do not prevent the phase goal from being achieved.

---

_Verified: 2026-05-16T00:00:00Z_
_Verifier: Claude (gsd-verifier)_
