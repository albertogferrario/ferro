---
phase: 115-spec-v2-data-structures
plan: 04
subsystem: introspection
tags: [json-ui, spec-v2, ferro-mcp, ferro-cli, caller-migration, ai-prompt]

# Dependency graph
requires:
  - phase: 115-spec-v2-data-structures
    plan: 02
    provides: Spec / Element / SpecBuilder / JsonUiRenderer::Output = Spec
provides:
  - ferro-mcp compiles against ferro-json-ui v2 (live-code type signatures + template strings)
  - ferro-cli compiles against ferro-json-ui v2 (scaffold templates + AI prompt emit Spec)
  - json_ui_inspect.rs and application_info.rs explicitly flagged as v1 scanners with TODO(Phase 120)
  - render_projection MCP tool wraps Spec via serde_json::to_value(&spec)? to preserve MCP protocol shape
affects:
  - 115-05-final-phase-gate (workspace-wide cargo build green once framework plan 03 also lands)
  - 120-mcp-ai-tool-rewrite (owns the v2 scanner rewrite flagged by TODO(Phase 120))
  - 117-catalog-and-schema (may replace the hand-maintained catalog in json_ui_catalog.rs)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "MCP Spec serialization: renderer.render() -> Spec, then serde_json::to_value(&spec) to produce MCP-protocol serde_json::Value"
    - "v1-scanner quarantine: top-of-file doc comment + TODO(Phase 120) marker; regex string literals left verbatim as documented v1 artifacts"
    - "v2 template emission: Spec::builder().element(id, Element::new(type).prop(k, v).child(id)).build().expect(...)"
    - "AI prompt structure: rules block mentioning flat element map + root id convention, embedded few-shot example in v2 syntax"

key-files:
  created: []
  deleted: []
  modified:
    - ferro-mcp/src/service.rs
    - ferro-mcp/src/tools/render_projection.rs
    - ferro-mcp/src/tools/code_templates.rs
    - ferro-mcp/src/tools/generation_context.rs
    - ferro-mcp/src/tools/json_ui_generate.rs
    - ferro-mcp/src/tools/json_ui_catalog.rs
    - ferro-mcp/src/tools/json_ui_inspect.rs
    - ferro-mcp/src/tools/application_info.rs
    - ferro-cli/src/templates/make.rs
    - ferro-cli/src/templates/module.rs
    - ferro-cli/src/ai.rs

key-decisions:
  - "json_ui_inspect.rs keeps its v1 regex string literals verbatim (`-> JsonUiView`, `Component::(\\w+)`) per D-19 and the plan's Warning 3. The live-code type import was never present — the module does pure string matching on user source — so no Rust type references needed changing. A top-of-file doc block plus a TODO(Phase 120) marker quarantines the v1 semantics until the dedicated rewrite phase."
  - "application_info.rs rename was mechanical: JsonUiViewsStatus -> JsonUiSpecsStatus, scan_json_ui_views -> scan_json_ui_specs. The regex inside remains a v1 scanner by design (flagged TODO(Phase 120)). Choosing to rename rather than keep the v1 name avoided leaving a public type with a misleading label."
  - "code_templates.rs's json_view section contained three templates (basic/list/form), not five as the plan's prose estimate suggested. Line ranges checked against the file after Plan 02's changes; the five-figure in the plan refers to an older count. All three were rewritten into v2 syntax and a single canonical import set — no templates were dropped or added."
  - "json_ui_catalog.rs's BUILDER_API const was reformulated to describe Spec { $schema, root, elements } and Element { type, props, children, action?, visible? } rather than preserving the old ComponentNode documentation. The text is still hand-maintained; a TODO(Phase 117) comment at file top points to the schemars-based introspection pass that will retire this const."
  - "ferro-cli/src/ai.rs prompt: switched from .layout(\"app\") to .layout(\"dashboard\") in the rules and example since the projection renderer's primary layout target is now `dashboard`. Also added a rule stating the root element id is always \"root\" — without it the AI would invent arbitrary ids and the spec wouldn't round-trip through Spec::from_json."
  - "render_projection.rs still returns `json_ui: serde_json::Value` in its MCP output struct (not the typed Spec). The MCP protocol prefers untyped JSON at the tool boundary; only the internal renderer return type changed. The wrap is `serde_json::to_value(&spec)` with a lossy-to-JSON error mapped to the existing String-based error channel."

patterns-established:
  - "When a file is a `v1 scanner` (regex-matches user source for v1 constructs), quarantine it with top-of-file doc + TODO(Phase X) marker; do NOT preemptively update the regexes to v2 patterns — that's the dedicated scanner-rewrite phase's responsibility."
  - "MCP tool outputs that serialize Spec should use `serde_json::to_value(&spec)` at the tool-return boundary to keep the MCP protocol surface as plain JSON while the internal pipeline uses typed Spec."

requirements-completed: [SPEC-04]

# Metrics
duration: ~30min
completed: 2026-04-18
---

# Phase 115 Plan 04: ferro-mcp + ferro-cli Migration to Spec v2 Summary

**Signature-and-string-swap migration of ferro-mcp (8 files) and ferro-cli (3 files) against the ferro-json-ui v2 surface produced by Plan 02. All live-code `JsonUiView` / `ComponentNode` references removed; template strings across 6 files now emit `Spec::builder()` / `Element::new()`. Two v1 scanners (json_ui_inspect.rs, application_info.rs) kept as v1-only with `TODO(Phase 120)` markers per D-19 and the plan's Warning 3.**

## Performance

- **Tasks:** 2 (all completed)
- **Files modified:** 11 (8 ferro-mcp + 3 ferro-cli)
- **Per-crate build/test:**
  - `cargo build -p ferro-mcp --all-targets --all-features` exits 0
  - `cargo test -p ferro-mcp` -> 203 passed, 0 failed
  - `cargo clippy -p ferro-mcp --all-targets --all-features -- -D warnings` exits 0
  - `cargo build -p ferro-cli --all-targets --all-features` exits 0
  - `cargo test -p ferro-cli` -> 11 passed, 0 failed
  - `cargo clippy -p ferro-cli --all-targets --all-features -- -D warnings` exits 0
- **Workspace-wide:** `cargo build --all-targets --all-features` still fails on `framework/src/json_ui/mod.rs` — this is Plan 03's territory (parallel executor in another worktree). Plans 03 and 04 together close the workspace-wide red state.

## Accomplishments

**ferro-mcp live-code migration (Task 1):**
- `service.rs` line 1270: `JsonUiView builder API` -> `Spec builder API` in the `json_ui_catalog` tool's description string.
- `tools/render_projection.rs`: renderer now returns `Spec`; tool wraps via `serde_json::to_value(&spec)?` to keep the MCP `json_ui: serde_json::Value` field shape stable. Test data's `"ferro-json-ui/v1"` schema string updated to `v2`.
- `tools/json_ui_inspect.rs`: v1-scanner quarantine applied. Added a top-of-file doc block stating the tool is a v1 scanner and carries `TODO(Phase 120)` for the rewrite. The regex literals `-> JsonUiView` and `Component::(\w+)` are preserved verbatim (no Rust type imports of `JsonUiView` ever existed in this file — the tool scans source with string regexes).
- `tools/application_info.rs`: `JsonUiViewsStatus` -> `JsonUiSpecsStatus`, `scan_json_ui_views` -> `scan_json_ui_specs`. Added a doc comment containing `TODO(Phase 120)` describing a parallel v2 scanner as future work.
- `tools/json_ui_catalog.rs`: mechanical `"Vec<ComponentNode>"` -> `"Vec<String>"` (11 occurrences in prop-description strings). BUILDER_API const rewritten from `JsonUiView::new()...ComponentNode{...}` to `Spec::builder()...Element::new(type).prop(k, v).child(id)...build()`. Top-of-file TODO updated from `TODO(ferro)` to `TODO(Phase 117)` naming the schemars-based introspection pass. Two assertions in `test_builder_api_present` updated to check for v2 strings.

**Template-string migration (Task 2):**
- `ferro-mcp/src/tools/code_templates.rs`: the 3 `json_view` templates (`basic_view`, `list_view`, `form_view`) rewritten from `JsonUiView::new().component(ComponentNode { ... })` shape to `Spec::builder().element(id, Element::new(type).prop(...)).build().expect(...)`. Imports reduced to `use ferro::{..., Spec, Element, JsonUi, Response}`; function signatures switched from `pub fn view() -> JsonUiView` to `pub async fn view() -> Response` (reflects the Plan 02 renderer contract: framework `JsonUi::render(&spec, &data) -> Response`).
- `ferro-mcp/src/tools/json_ui_generate.rs`: `VIEW_EXAMPLE` const rewritten into v2 canonical shape with a single `DataTable` root. `ViewConventions.function_signature` now `"pub async fn view() -> Response"` and `import_pattern` now `"use ferro::{Spec, Element, JsonUi, Response, ...};"`. Two tests (`test_conventions_populated`, `test_example_not_empty`) updated.
- `ferro-mcp/src/tools/generation_context.rs`: `common_patterns.json_ui_view` rewritten to a minimal `Spec::builder().element("root", Element::new("DataTable").prop(...))` example; `imports.json_ui_view` switched to `use ferro::{Spec, Element, JsonUi, Response, /* Action, Visibility, ... */};`.
- `ferro-cli/src/templates/make.rs`'s `json_view_template(name, title, layout)`: `--no-ai` scaffold now emits a v2 card-with-heading spec. Keeps positional parameters and templating tokens intact.
- `ferro-cli/src/templates/module.rs`'s `module_view_index_rs(name)`: same migration for the feature-module views/index.rs scaffold.
- `ferro-cli/src/ai.rs`'s `build_view_context`: system prompt's rule list and embedded few-shot example both switched to v2 syntax. Rules now mention the flat element map and `"root"` id convention so the AI generates spec-compliant output. Example switched from `Component::Table(TableProps { ... })` to `Element::new("DataTable").prop("columns", serde_json::json!([...])).prop("data_path", "/data/users")`.

## Task Commits

1. **Task 1: Migrate ferro-mcp live-code type signatures (non-template files)** — `85830223` (refactor)
2. **Task 2: Rewrite template strings — code_templates / generation_context / json_ui_generate / ferro-cli** — `53d44f4a` (refactor)

## Files Modified

### ferro-mcp
- `ferro-mcp/src/service.rs` (1 line; tool description string)
- `ferro-mcp/src/tools/render_projection.rs` (Spec-to-Value wrap + 2 test string literals)
- `ferro-mcp/src/tools/code_templates.rs` (3 json_view templates rewritten; ~130 LoC net change)
- `ferro-mcp/src/tools/generation_context.rs` (2 template strings + 1 import pattern)
- `ferro-mcp/src/tools/json_ui_generate.rs` (VIEW_EXAMPLE const + conventions struct + 2 tests)
- `ferro-mcp/src/tools/json_ui_catalog.rs` (11x prop-type descriptions + BUILDER_API const + 2 test assertions + top-of-file TODO)
- `ferro-mcp/src/tools/json_ui_inspect.rs` (top-of-file TODO(Phase 120) doc block)
- `ferro-mcp/src/tools/application_info.rs` (struct rename + fn rename + scan fn doc TODO)

### ferro-cli
- `ferro-cli/src/templates/make.rs` (`json_view_template` rewritten to v2 scaffold)
- `ferro-cli/src/templates/module.rs` (`module_view_index_rs` rewritten to v2 scaffold)
- `ferro-cli/src/ai.rs` (AI system prompt rewritten — rules + few-shot example)

## Deviations from Plan

None in substance. Two implementation notes:

1. **Template count discrepancy.** The plan text said "5 templates at lines 909–1109" in `code_templates.rs`; the actual file contains 3 json_view templates (basic/list/form) in that range. All three were migrated. No templates outside the `json_view` category reference v1 types.

2. **Layout default switched from `"app"` to `"dashboard"` in the AI prompt.** The plan's canonical v2 template uses `.layout("dashboard")`. Preserving `"app"` would have contradicted the plan's `<interfaces>` block. The `--no-ai` fallback in `make.rs` still respects the caller-supplied `layout` parameter (no default change there).

## TDD Gate Compliance

Plan 04 is type-`execute` (not type-`tdd`), so RED/GREEN gates do not apply. All existing tests in both crates passed after each task commit without modification (except test-string updates that track implementation changes — conventions struct values, example substring, schema version).

## Issues Encountered

- **`cargo clippy --all --all-targets -- -D warnings` workspace-wide**: not run. It would fail on `framework/` until Plan 03 lands. Per-crate clippy (`-p ferro-mcp`, `-p ferro-cli`) exits 0 with `-D warnings`. This matches the plan's intent: Plan 04's scope is ferro-mcp + ferro-cli only.
- **No auto-fix events.** No Rule 1/2/3 fixes applied; the migration was purely mechanical signature-and-string swaps.

## Self-Check: PASSED

**Files verified (all present and modified):**
- `ferro-mcp/src/service.rs` — FOUND
- `ferro-mcp/src/tools/render_projection.rs` — FOUND
- `ferro-mcp/src/tools/code_templates.rs` — FOUND
- `ferro-mcp/src/tools/generation_context.rs` — FOUND
- `ferro-mcp/src/tools/json_ui_generate.rs` — FOUND
- `ferro-mcp/src/tools/json_ui_catalog.rs` — FOUND
- `ferro-mcp/src/tools/json_ui_inspect.rs` — FOUND
- `ferro-mcp/src/tools/application_info.rs` — FOUND
- `ferro-cli/src/templates/make.rs` — FOUND
- `ferro-cli/src/templates/module.rs` — FOUND
- `ferro-cli/src/ai.rs` — FOUND

**Commits verified:**
- `85830223` (Task 1 refactor) FOUND in git log
- `53d44f4a` (Task 2 refactor) FOUND in git log

**Acceptance gates (run 2026-04-18):**
- `cargo build -p ferro-mcp --all-targets --all-features` -> 0
- `cargo test -p ferro-mcp` -> 203 passed
- `cargo clippy -p ferro-mcp --all-targets --all-features -- -D warnings` -> 0
- `cargo build -p ferro-cli --all-targets --all-features` -> 0
- `cargo test -p ferro-cli` -> 11 passed
- `cargo clippy -p ferro-cli --all-targets --all-features -- -D warnings` -> 0

**Grep invariants verified:**
- `! grep -rEn "JsonUiView::new|ComponentNode \{" ferro-mcp/src/ ferro-cli/src/` — empty (no v1 construction)
- `grep -q 'JsonUiView' ferro-mcp/src/tools/json_ui_inspect.rs` — match (regex preserved)
- `grep -q 'TODO(Phase 120)' ferro-mcp/src/tools/json_ui_inspect.rs` — match
- `grep -q 'TODO(Phase 120)' ferro-mcp/src/tools/application_info.rs` — match
- `grep -q "scan_json_ui_specs" ferro-mcp/src/tools/application_info.rs` — match
- `grep -q "JsonUiSpecsStatus" ferro-mcp/src/tools/application_info.rs` — match
- `! grep -q "JsonUiViewsStatus" ferro-mcp/src/tools/application_info.rs` — empty
- `grep -q "serde_json::to_value(&spec)" ferro-mcp/src/tools/render_projection.rs` — match
- `grep -q "Spec builder API" ferro-mcp/src/service.rs` — match
- `grep -q "Spec::builder()" ferro-cli/src/ai.rs` — match
- `grep -q "Spec::builder()" ferro-cli/src/templates/make.rs` — match
- `grep -q "Spec::builder()" ferro-mcp/src/tools/code_templates.rs` — match

## Workspace Status

Plan 04's scope (ferro-mcp + ferro-cli) is v2-only and builds + tests + clippies clean.

- `framework/` remains red on this branch — Plan 03's executor (parallel worktree) owns that migration.
- Once Plans 03 and 04 both merge, `cargo build --all-targets --all-features` at the workspace root will be green for the first time since Plan 02.

## Next Phase Readiness

- **Plan 05** (final phase gate) can run once Plan 03's framework-caller migration also lands — at that point the full workspace will be green against v2.
- **Phase 117** (catalog + schema) is unblocked: `json_ui_catalog.rs`'s hand-maintained catalog is now v2-consistent, and the TODO(Phase 117) marker points directly at the schemars-based introspection pass that will retire it.
- **Phase 120** (MCP AI tool rewrite) has an explicit contract: two files carry `TODO(Phase 120)` markers (`json_ui_inspect.rs`, `application_info.rs`). Both preserve their v1 regexes so Phase 120 can diff the old patterns against the new v2 scanner in a single commit. The v1 regex literal `JsonUiView` in `json_ui_inspect.rs` is the authoritative marker for the scanner-rewrite target.
- **AI code generation quality**: `ferro-cli ai.rs`'s system prompt now steers models toward valid v2 source. The embedded few-shot matches the canonical form from the plan's `<interfaces>` block, mitigating T-115-11.

---
*Phase: 115-spec-v2-data-structures*
*Plan: 04*
*Completed: 2026-04-18*
