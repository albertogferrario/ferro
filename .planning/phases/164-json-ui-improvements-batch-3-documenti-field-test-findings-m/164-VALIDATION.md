---
phase: 164
slug: json-ui-improvements-batch-3-documenti-field-test-findings-m
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-17
---

# Phase 164 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Test-coverage requirements per decision are specified in `164-RESEARCH.md` § Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust 2021 edition, workspace tests) |
| **Config file** | `Cargo.toml` (workspace) + per-crate `Cargo.toml` |
| **Quick run command** | `cargo test -p ferro-json-ui --lib` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | quick ~15s, full ~3-5min (release-mode tests) |

CI parity command (per `feedback_ci_clippy_command_match`): the full suite above matches `.github/workflows/test.yml`.

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p {crate-touched} --lib` (the quick command scoped to the touched crate)
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test -p ferro-json-ui -p ferro-mcp --lib`
- **Before `/gsd-verify-work`:** Full suite (fmt + clippy + test --all-features) must be green
- **Max feedback latency:** 60 seconds (per-crate quick run) / 5 minutes (full suite)

---

## Per-Decision Verification Map

Plans land per the planner's slicing — IDs (164-NN) will be assigned at planning time. Verification map below is keyed by decision; the planner fills `Task ID` + `Plan` + `Wave` columns when generating plans.

| Decision | Source | Test Type | Required Test Coverage | Automated Command |
|----------|--------|-----------|------------------------|-------------------|
| D-12 (Spec.title `$data` binding) | F1 / 164-CONTEXT D-12 | unit + integration | (a) Serde round-trip: literal String + binding-map both deserialise; (b) Renderer test: title resolves from data; (c) Render test: literal title passes through unchanged | `cargo test -p ferro-json-ui --lib spec::title` |
| D-13a (KanbanBoard.data_path) | F3 / 164-CONTEXT D-13 | unit + render | (a) Props serde with `data_path: Some` and inlined `columns` both valid; (b) Render with `data_path` resolves columns from data array; (c) Validation: `data_path` and `columns` are mutually exclusive (or one wins, planner decides) | `cargo test -p ferro-json-ui --lib render::kanban` |
| D-13b ($each kanban doc) | F3 / 164-CONTEXT D-13 | docs + example | (a) Worked example in `docs/src/json-ui/expressions.md`; (b) End-to-end integration test of `$each` over KanbanColumn template | `cargo test -p ferro-json-ui --test directives_kanban_each` |
| D-14 (MAX_NESTING_DEPTH 3→5) | F4 / 164-CONTEXT D-14 | unit | (a) Depth-4 spec passes; (b) Depth-5 spec passes; (c) Depth-6 spec rejected with clear error; (d) Update existing depth-3 test at `spec.rs:1705` to depth-5; (e) Audit any hardcoded "3" in docs / error messages | `cargo test -p ferro-json-ui --lib spec::nesting_depth` |
| D-15 (Image / DescriptionList data_path) | F7 / 164-CONTEXT D-15 | unit + render | (a) ImageProps serde with `data_path` resolves `src` from data; (b) DescriptionListProps serde with `data_path` resolves `items` from data; (c) Static src/items still work; (d) Validation rejects both being set | `cargo test -p ferro-json-ui --lib component::image component::description_list` |
| D-16 (Validation pipeline reorder, two-stage) | F8 / 164-CONTEXT D-16 | unit + integration | (a) Startup loader runs structural validation only (footer-IDs, depth, element-refs) + semantic warnings logged; (b) Per-request `JsonUi::resolve` enforces semantic validation AFTER `expand_directives` + visibility resolution; (c) Spec with `visible:false` Alert.variant=`""` loads with warning AND renders cleanly (alert hidden); (d) Spec with `visible:true` Alert.variant=`""` rejected at request time | `cargo test -p ferro-json-ui --lib spec::validate --test validation_pipeline` |
| D-17a (Component::RawHtml) | F9 / 164-CONTEXT D-17 | unit + render + catalog | (a) Catalog count assertion updated in 3 sites (render/mod.rs:530, catalog.rs:1052, ferro-mcp/src/tools/json_ui_catalog.rs:290+expected-names); (b) RawHtmlProps with literal HTML renders verbatim (sanitisation policy TBD by planner); (c) `data_path: Some` resolves HTML from data; (d) Null data path renders empty (no container); (e) MCP catalog tool surfaces the new component | `cargo test -p ferro-json-ui --lib render::raw_html catalog && cargo test -p ferro-mcp --lib tools::json_ui_catalog` |
| D-18 (CardVariant: Bordered + Elevated) | F10 / 164-CONTEXT D-18 | unit + render | (a) `CardVariant::default() == Bordered`; (b) Serde round-trip both variants; (c) `render_card` emits `shadow-sm`+`p-4`+border for Bordered, `shadow-md`+`p-8`+no-border for Elevated; (d) Cards without explicit variant keep current dashboard look (default Bordered); (e) Codemod tweak (or explicit omit) doesn't regress 163-emitted specs | `cargo test -p ferro-json-ui --lib component::card render::card` |
| D-19/F2 (Codemod uppercase methods) | V7-RUNTIME F2 | unit | (a) Audit `ferro-cli/src/commands/json_ui_migrate_v1.rs:521` — already emits uppercase per research; (b) Add regression test that codemod input `method: "get"` produces output `method: "GET"` for all HTTP verbs; (c) If audit shows codemod broken, fix in same plan | `cargo test -p ferro-cli --test json_ui_migrate_v1 method_uppercase` |
| D-19/F5 (Visibility error message) | V7-RUNTIME F5 | unit | (a) Visibility deserializer custom error includes accepted variant shapes and the rejected JSON shape; (b) Error message snapshot test ensures stability | `cargo test -p ferro-json-ui --lib component::visibility::error_message` |
| D-19/F6 (PageHeader.actions accepts None) | V7-RUNTIME F6 | unit | (a) PageHeaderProps with `actions: null` deserialises to empty vec; (b) PageHeaderProps with `actions: []` deserialises identically; (c) PageHeaderProps with non-empty array deserialises as before | `cargo test -p ferro-json-ui --lib component::page_header::actions_optional` |
| D-04 (MCP validator surface) | 164-CONTEXT D-04 | unit + integration | (a) New `json_ui_validate_spec` tool (or extension) returns same errors as runtime startup; (b) MCP tool returns warnings from D-16 stage-1; (c) Snapshot test for typical bad spec | `cargo test -p ferro-mcp --test json_ui_validate_spec` |
| D-05 (Directive validation gates) | 164-CONTEXT D-05 | unit | (a) `$each.path` must resolve to JSON array — non-array rejected; (b) `$if.path` must resolve to bool/coercible — rejected otherwise; (c) Circular ref in templated element rejected; (d) `children` ref to absent element WITHOUT $if rejected, WITH $if accepted | `cargo test -p ferro-json-ui --lib spec::validate::directives` |
| D-01..D-03 (v1-deletion audit) | 164-CONTEXT D-01..D-03 | manual + grep | (a) Audit table in V1-DELETION-AUDIT.md covers every v1 surface element; (b) Each row resolved to MIGRATED / INTENTIONAL_DROP / BLOCKER; (c) Zero BLOCKER rows; (d) Grep proof of v1 absence — no `view.rs`, no `JsonUiView`, no `Component` enum, no `ComponentNode` in v12.0/json-ui-v2 HEAD | `grep -rE '\\b(JsonUiView\|ComponentNode)\\b' ferro-json-ui/src framework/src` (must return 0 matches) |
| D-06..D-07 (Plugin surface audit) | 164-CONTEXT D-06..D-07 | manual + paper exercise | (a) Audit doc reviews Phase 162 D-19 plugin guide against (i) Stripe widget, (ii) WhatsApp connection, (iii) chart renderer; (b) Any gap → BLOCKER row in D-01 audit + fix in this phase | n/a (paper audit) |
| D-08..D-09 (Documentation pass) | 164-CONTEXT D-08..D-09 | docs build + grep | (a) `mdbook build` clean; (b) Every catalog component has a doc section (grep proof); (c) Cheat-sheet table present in `migration-v1-to-v2.md` with 10 rows | `(cd docs && mdbook build) && cargo test -p ferro-json-ui --test docs_coverage` |
| D-10..D-11 (COMPLETED.md) | 164-CONTEXT D-10..D-11 | content review | (a) COMPLETED.md sections present: Shipped, Runtime frictions resolved, Intentional gaps, Deferred, v1→v2 surface migration table; (b) Every D-* from 162/163/163.1/164 cited with ship status; (c) Every F1..F10 listed with resolution; (d) Phase 160 unblocked statement | `grep -c '^### ' .planning/phases/164-*/164-COMPLETED.md` (must hit all required sections) |

*Status legend (per task in plans): ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Wave 0 is **empty for this phase** — all required test infrastructure already exists in the workspace:
- `cargo test` infrastructure ships with the workspace
- `ferro-json-ui` has existing unit + integration test scaffolding
- `ferro-mcp` has tool-test infrastructure
- `ferro-cli` has codemod integration-test fixtures (Phase 163.1 added the multi-root fixtures)

If the planner identifies a missing test harness or fixture file during planning, surface as a Wave 0 task at that time.

---

## Manual-Only Verifications

| Behavior | Decision | Why Manual | Test Instructions |
|----------|----------|------------|-------------------|
| Visual diff of Card Bordered vs Elevated | D-18 | Visual chrome rendered to HTML — unit tests assert class strings but a human eye on the rendered output catches subtle visual regressions | After Wave shipping D-18: render two test specs (one Bordered, one Elevated), open in browser, compare against V7-RUNTIME-FRICTION.md `login-prod.png` reference. Approve or surface a follow-up. |
| Cross-repo: gestiscilo specs pass against new validator | D-04, D-13a, D-15, D-16, D-18 | Real-world validation against the consumer that drove the friction | After Wave shipping D-04/D-16: in gestiscilo, run `cargo run --bin {whatever loads specs} --path src/views` and confirm zero hard errors on the 23+1 dashboard pages from V7-RUNTIME-FRICTION.md table |
| Plugin surface paper-audit | D-06 | The audit is a thought experiment — author plugin docs from scratch in head, list gaps | Read `docs/src/json-ui/plugins.md`, walk through Stripe payment widget / WhatsApp link / chart renderer scenarios, document any "I don't know how to do this from the docs" moments as BLOCKER rows |

---

## Validation Sign-Off

- [ ] All decision rows have an automated command OR a manual-only justification
- [ ] Sampling continuity: full suite gates every wave boundary
- [ ] Wave 0 confirmed empty (or filled by planner)
- [ ] No watch-mode flags in commands
- [ ] Feedback latency < 60s (quick) / < 5min (full)
- [ ] `nyquist_compliant: true` set in frontmatter after planner assigns Task IDs to every row

**Approval:** pending
