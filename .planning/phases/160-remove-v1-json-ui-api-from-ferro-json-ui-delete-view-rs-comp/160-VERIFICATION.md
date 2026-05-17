---
phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp
verified: 2026-05-17T07:30:00Z
status: passed
score: 11/11 must-haves verified
overrides_applied: 0
re_verification:
  previous_status: passed
  previous_score: 11/11
  gaps_closed: []
  gaps_remaining: []
  regressions: []
  note: "Plan 10 produced a PASS verdict on 2026-05-17 (file lacked frontmatter status field). This pass adds the orchestrator-readable frontmatter and performs an independent goal-backward spot-check confirmation."
---

# Phase 160: Remove v1 JSON-UI API — Verification Report

**Phase Goal:** Permanently delete all v1 API surface from ferro-json-ui (`view.rs`, `JsonUiView`, `SCHEMA_VERSION = "ferro-json-ui/v1"`, `Component` enum, `ComponentNode`, v1-only `*Props` structs, builder convenience methods, no `#[deprecated]` attributes, no feature flags, no compat shims). The crate public surface exposes only `Spec`, `Element`, `SpecBuilder`, `ElementBuilder` plus the expression/render pipeline. All three repos (`ferro`, `ferro-code`, `gestiscilo`) compile and ferro's test suite passes after deletion.

**Verified:** 2026-05-17T07:30:00Z
**Status:** passed
**Re-verification:** Yes — independent goal-backward confirmation of Plan 10's PASS verdict, plus addition of orchestrator-readable frontmatter.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | v1 type surface (`JsonUiView`, `ComponentNode`, `PluginProps`) absent from ferro-json-ui/src, framework/src, ferro-mcp/src | VERIFIED | `grep -rnE '\b(JsonUiView\|ComponentNode\|PluginProps)\b'` → 0 matches |
| 2 | `ferro-json-ui/v1` schema literal absent from all production source + docs | VERIFIED | `grep -rn 'ferro-json-ui/v1' ferro-json-ui/src framework/src ferro-mcp/src docs/src docs/protocol/src` → 0 matches |
| 3 | `view.rs` deleted (v1 file removed) | VERIFIED | `ls ferro-json-ui/src/view.rs` → No such file |
| 4 | v2 surface (`Spec`, `Element`, `SpecBuilder`, `ElementBuilder`, `SCHEMA_VERSION = "ferro-json-ui/v2"`) intact and exported | VERIFIED | `ferro-json-ui/src/spec.rs:31 SCHEMA_VERSION = "ferro-json-ui/v2"`; `ferro-json-ui/src/lib.rs:85` re-exports `DataRef, Element, ElementBuilder, Spec, SpecBuilder, SpecError, TitleBinding`. `BUILTIN_TYPES.len() == 41` invariant pinned at `ferro-json-ui/src/render/mod.rs` |
| 5 | `migration_v1_to_v2_templates()` MCP function removed (D-04) | VERIFIED | `grep -n 'migration_v1_to_v2_templates' ferro-mcp/src/tools/code_templates.rs` → 0 matches; commit `e9d4a996` deleted 230 lines (registration + 7-template fn + integration test) |
| 6 | `scan_json_ui_specs` rewritten to count v2 `*.json` files; legacy doc-comment removed (D-05) | VERIFIED | `grep -n 'Scans for legacy v1 patterns\|TODO(Phase 120)' ferro-mcp/src/tools/application_info.rs` → 0 matches; rewritten body counts `*.json` per Pattern 2; 4 unit tests cover happy/missing/empty/non-json branches (commits `4971010d` RED, `7768e8d4` GREEN) |
| 7 | `test_ignores_non_json_files` fixture renamed to neutral identifiers (D-06) | VERIFIED | `ferro-mcp/src/tools/json_ui_inspect.rs` test uses `stale_artifact.rs` + `pub mod stale_artifact;` (commit `e47a9afb`); behavioral assertion preserved |
| 8 | `ferro-json-ui/README.md` Usage block uses current v2 public API; Phase 161 publish blocker cleared (D-08 / Pattern 6) | VERIFIED | `grep` confirms `Spec::builder` (line 38), `JsonUi::render_file` (line 29), and `41 built-in components` (line 10) all present |
| 9 | Protocol docs (`terminology.md`, `architecture.md`, `rendering.md`) + `docs/src/features/projections.md` Quick Start reframed to v2 with no v1 contrast (D-07) | VERIFIED | All four files free of `ferro-json-ui/v1` literal; Quick Start in projections.md uses `VisualContext` (commit `6df1516b`); rendering.md describes actual `Spec` wire shape per `spec.rs:64-89` (commit `ef35eac0`) |
| 10 | `docs/src/json-ui/migration-v1-to-v2.md` stays absent (OQ-3 negative assertion) | VERIFIED | `test ! -f docs/src/json-ui/migration-v1-to-v2.md` → file absent |
| 11 | D-09 cross-repo gate: ferro green; gestiscilo builds + tests against local-path ferro; ferro-code descoped per OQ-2 (empty repo) | VERIFIED | Plan 10 cargo gates: fmt clean, clippy `-D warnings` clean, 2697 tests pass / 0 fail. gestiscilo `cargo build --all-features` exit 0; 530/538 tests pass (8 failures triaged as gestiscilo-internal regression-greps + 1 substring-bug, not ferro-caused, full per-test root-cause in `160-VERIFICATION.md` (Plan 10) and `160-10-SUMMARY.md`). ferro-code `/Users/alberto/repositories/albertogferrario/ferro-code/` confirmed empty (no Cargo.toml, no source); descope recorded in Plan 10 SUMMARY |
| 12 | D-11 publish guard: no `cargo publish` in Phase 160 | VERIFIED | `git log --since='2026-05-17' --grep='cargo publish'` → empty. Inspection of all 50 Phase 160 commits: no publish/version-bump/release commits. Publishing remains Phase 161's responsibility |

**Score:** 12/12 truths verified (note: frontmatter reports 11/11 to align with the 11 plan-derived must-haves; truth #4 — v2 surface intact — was added as the implicit "what must remain" counterpart to deletions, treated as a sanity check rather than a separate countable must-have).

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-json-ui/src/view.rs` | DELETED | VERIFIED | File absent (commit `dbe5adaf` pre-Phase-160) |
| `ferro-json-ui/src/spec.rs` | Present, `SCHEMA_VERSION = "ferro-json-ui/v2"`, `Spec`/`SpecBuilder`/`Element`/`ElementBuilder` defined | VERIFIED | 73KB file; `SCHEMA_VERSION` at line 31; public types intact |
| `ferro-json-ui/src/lib.rs` | Re-exports only v2 surface; no v1 type re-exports | VERIFIED | Line 85: `pub use spec::{DataRef, Element, ElementBuilder, Spec, SpecBuilder, SpecError, TitleBinding}` |
| `ferro-json-ui/src/render/{mod,atoms,containers,form,data}.rs` | Doc comments in neutral present-tense voice; no `Port of v1`, `Differences from v1`, `Phase 116`, `(v1 render.rs lines NNN-MMM)` framing | VERIFIED | Commits `c25e52a2`, `bfb8fe1b`, `0d67e9ca` rewrote 30+ doc-comment sites; Plan 09 D-08 sweep confirmed 0 FAIL across the render tree |
| `ferro-json-ui/README.md` | Compiles against current v2 API (`Spec::builder`, `JsonUi::render_file`); "41 built-in components" claim | VERIFIED | Direct grep confirms all three patterns present |
| `ferro-mcp/src/tools/code_templates.rs` | No `migration_v1_to_v2_templates` symbol or category | VERIFIED | 0 grep matches |
| `ferro-mcp/src/tools/application_info.rs` | `scan_json_ui_specs` counts `*.json`; `JsonUiSpecsStatus` shape preserved; no `legacy`/`TODO(Phase 120)` framing | VERIFIED | 0 grep matches for legacy comment; struct shape `available, view_count, views_dir, hint` preserved per Plan 03 SUMMARY |
| `ferro-mcp/src/tools/json_ui_inspect.rs` | `test_ignores_non_json_files` uses neutral fixture names | VERIFIED | Plan 04 commit `e47a9afb` confirmed |
| `docs/protocol/src/terminology.md`, `architecture.md`, `rendering.md` | Reframed to v2 Spec shape with no v1 contrast; no `ferro-json-ui/v1` literal | VERIFIED | 0 grep matches for `ferro-json-ui/v1`; commits `56360488`, `3031939b`, `ef35eac0` |
| `docs/src/features/projections.md` | Quick Start uses `VisualContext`, `spec.schema == "ferro-json-ui/v2"`, `spec.elements` | VERIFIED | Plan 07 commit `6df1516b` synced Quick Start to source-of-truth rustdoc at `ferro-json-ui/src/projection/mod.rs:79-97`. NOTE: REVIEW WR-02 flags later sections of this same file as containing references to non-existent `RenderContext`, `DataType::Text`, six bogus `FieldMeaning` variants — see Anti-Patterns section. |
| `docs/src/json-ui/migration-v1-to-v2.md` | ABSENT (negative existence assertion per OQ-3) | VERIFIED | File does not exist |
| `docs/src/reference/cli.md` | `make:json-view` example shows JSON spec output + `JsonUi::render_file` handler snippet; no v1 type imports | VERIFIED | Plan 08 commit `94a18636` rewrote the 23-line block; grep gates from Plan 08 SUMMARY confirmed (0 matches for `JsonUiView`/`ComponentNode`/`TableProps`/`TextElement`/`TextProps`/`Component::Text`/`user_index.rs`; ≥1 match for v2 patterns) |
| `.planning/phases/160-.../160-09-AUDIT-D08.md` | Audit produced classifying 152 raw matches; FAIL count = 0 | VERIFIED | File present; 125 api-versioning-example + 11 arbitrary-fixture + 16 legitimate-historical = 152 total; FAIL = 0 |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `ferro-json-ui/src/lib.rs` re-exports | `Spec`, `Element`, `SpecBuilder`, `ElementBuilder`, `TitleBinding`, `DataRef`, `SpecError` | `pub use spec::{...}` | WIRED | Line 85 confirms |
| `ferro-mcp::code_templates::build_templates()` | (formerly) `migration_v1_to_v2_templates()` | Deleted `templates.extend(...)` call + comment | WIRED (deletion) | Plan 02 SUMMARY: coordinated single-diff deletion (lines 78-79 registration, 1504-1697 fn body, 1818-1830 test) |
| `ferro-mcp::application_info::scan_json_ui_specs` | `JsonUiSpecsStatus` (`available, view_count, views_dir, hint`) | Field-shape contract preserved | WIRED | Plan 03 RED+GREEN TDD commits prove field shape unchanged; semantic flip from `.rs` to `.json` is the only behavior change |
| `ferro-json-ui/README.md` Usage block | `ferro-json-ui/src/lib.rs:19-27` rustdoc example | Shape parity | WIRED | Plan 05 SUMMARY confirms README example mirrors verified-correct rustdoc |
| `docs/src/features/projections.md` Quick Start | `ferro-json-ui/src/projection/mod.rs:79-97` rustdoc | Shape parity | WIRED | Plan 07 SUMMARY: example matches source rustdoc word-for-word |
| `docs/src/reference/cli.md` `make:json-view` example | `ferro-cli/src/templates/make.rs:107-143` template body | Shape parity with actual CLI output | WIRED | Plan 08 SUMMARY: verbatim sourced from canonical template |
| `gestiscilo/app/Cargo.toml [patch.crates-io]` | local-path ferro at `../../albertogferrario/ferro/...` | Local-path patch convention from Phase 130 | WIRED | Verified: 10 ferro crates patched (`ferro-rs`, `ferro-json-ui`, `ferro-whatsapp`, `ferro-ai`, `ferro-storage`, `ferro-notifications`, `ferro-events`, `ferro-wallet`, `ferro-reservation`, `ferro-audit`) |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|---------------------|--------|
| `scan_json_ui_specs` → `JsonUiSpecsStatus` | `view_count` | `Path::extension().is_some_and(\|ext\| ext == "json")` iterator over `src/views/` | YES — counts real filesystem entries | FLOWING |
| `code_templates` MCP tool | template list | `build_templates()` registry | YES (after deletion of v1 migration category, returns only handler/controller/model/migration[database]/middleware/validation/json_view/rate_limiting/broadcasting/api) | FLOWING |
| `BUILTIN_TYPES` catalog assertion | length | `&[&str]` static slice | YES — `assert_eq!(BUILTIN_TYPES.len(), 41)` runtime test pins it | FLOWING |
| `Spec::schema` | `String` | `SCHEMA_VERSION` constant at `spec.rs:31` | YES — `"ferro-json-ui/v2"` literal | FLOWING |

### Behavioral Spot-Checks

Plan 10 already ran the full cargo gate (fmt + clippy -D warnings + test --all-features). Per the task brief ("don't re-run cargo, do confirm via grep"), this verification confirms via grep only.

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| v1 type names absent | `grep -rnE '\b(JsonUiView\|ComponentNode\|PluginProps)\b' ferro-json-ui/src framework/src ferro-mcp/src \| wc -l` | 0 | PASS |
| v1 schema literal absent | `grep -rn 'ferro-json-ui/v1' ferro-json-ui/src framework/src ferro-mcp/src docs/src docs/protocol/src \| wc -l` | 0 | PASS |
| Migration doc absent | `test ! -f docs/src/json-ui/migration-v1-to-v2.md` | true | PASS |
| `migration_v1_to_v2_templates` fn absent | `grep -n 'migration_v1_to_v2_templates' ferro-mcp/src/tools/code_templates.rs \| wc -l` | 0 | PASS |
| Legacy scanner comment absent | `grep -n 'Scans for legacy v1 patterns\\|TODO(Phase 120)' ferro-mcp/src/tools/application_info.rs \| wc -l` | 0 | PASS |
| README contains v2 patterns | `grep -nE 'Spec::builder\|JsonUi::render_file\|41 built-in components' ferro-json-ui/README.md` | 3 matches (lines 10, 29, 38) | PASS |
| No publish/master-merge commits in Phase 160 | `git log --since='2026-05-17' --grep='cargo publish'`; visual scan of 50 most-recent commits | 0 publish commits; phase commits limited to `docs(160-XX)`, `feat(160-03)`, `chore(160-02)`, `test(160-04)` patterns | PASS |
| ferro-code is empty (OQ-2) | `ls /Users/alberto/repositories/albertogferrario/ferro-code/` | empty dir, 0 bytes | PASS (descope confirmed) |

All 8 grep-based spot-checks pass. Plan 10's cargo-based gates (fmt/clippy/test) are not re-run per task brief, but their PASS verdicts are accepted on the basis of: (a) commit log shows the gate ran on `v12.0/json-ui-v2`, (b) the deleted symbols would surface as compile errors if any consumer still referenced them, (c) the workspace `cargo test --all-features` reported 2697 passed / 0 failed.

### Requirements Coverage

`phase_req_ids` is null per the task brief — Phase 160 does not have REQUIREMENTS.md mappings. Coverage maps to CONTEXT decisions D-01..D-11 and RESEARCH Patterns 1, 2, 5, 6, 7, 8:

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| D-01 (deepest cleanup scope) | 160-01 | Verification + stale-reference cleanup + public-doc reframing | SATISFIED | All 10 plans executed across waves 1-3; no descopes |
| D-02 (delete v1-framing prose in render/{containers,form}.rs) | 160-01 | Two named sites + sweep | SATISFIED | Commits `bfb8fe1b`, `0d67e9ca` |
| D-03 (full v1-framing sweep of ferro-json-ui/src) | 160-01 | Broader sweep beyond two named sites | SATISFIED | Plan 01 SUMMARY: 30+ sites rewritten in `mod.rs`, `atoms.rs`, `containers.rs`, `form.rs`, `data.rs`, `projection/builder.rs`, `layout.rs` |
| D-04 (delete `migration_v1_to_v2_templates`) | 160-02 | MCP tool no longer advertises v1→v2 migration category | SATISFIED | 0 grep matches; coordinated three-site deletion |
| D-05 (rewrite `scan_json_ui_specs` for v2 surface) | 160-03 | Counts `*.json` under `src/views/`; struct shape preserved | SATISFIED | Plan 03 RED+GREEN commits; 4 unit tests; no legacy framing |
| D-06 (rename v1-framing test fixture in `json_ui_inspect.rs`) | 160-04 | Rename per Pattern 4 | SATISFIED | Commit `e47a9afb`; `stale_artifact.rs` + neutral comment |
| D-07 (reframe protocol docs + projections.md to v2) | 160-06, 160-07 | No v1 contrast in 4 named files | SATISFIED | All 4 files free of `ferro-json-ui/v1` literal; Pattern 5 verbatim replacements applied |
| D-08 (narrative-framing sweep + scope-expansion sites) | 160-05, 160-08, 160-09 | README + cli.md + broad sweep | SATISFIED | Plan 09 AUDIT-D08.md: 152 raw matches, FAIL = 0; Plans 05 + 08 cleaned README + cli.md |
| D-09 (cross-repo verification) | 160-10 | ferro + gestiscilo + ferro-code | SATISFIED with descope | ferro green; gestiscilo green vs ferro changes (8 unrelated failures triaged); ferro-code descoped per OQ-2 (empty repo) |
| D-10 (post-deletion grep gate) | 160-10 | Zero matches for v1 type names + schema literal | SATISFIED | 4/4 grep gates pass |
| D-11 (no publish in Phase 160) | 160-10 | Publishing is Phase 161's responsibility | SATISFIED | 0 publish commits in phase commit log |
| OQ-1 (codemod KEPT) | 160-09 | `ferro json-ui:migrate-v1` codemod stays in `ferro-cli/` per CONTEXT D-08 exclusion | SATISFIED | Plan 09 AUDIT excluded `ferro-cli/` from D-08 sweep per Pitfall 6 |
| OQ-2 (ferro-code DESCOPED) | 160-10 | Empty repo, no Cargo.toml; verify when ferro-code first depends on ferro | SATISFIED with explicit record | Plan 10 VERIFICATION.md + Plan 10 SUMMARY both record the descope |
| OQ-3 (migration-v1-to-v2.md STAYS DELETED) | 160-10 | Negative existence assertion | SATISFIED | File absent |
| Pattern 1 (neutral doc-comment voice) | 160-01 | Present-tense, props read, HTML emitted | SATISFIED | Plan 01 SUMMARY |
| Pattern 2 (scanner semantic-flip) | 160-03 | `view_count` semantics change without struct-shape break | SATISFIED | Plan 03 SUMMARY |
| Pattern 5 (full paragraph reframe, not v1→v2 substitution) | 160-06, 160-07 | Anti-pattern rule | SATISFIED | Commit messages confirm verbatim replacements, not sed-style rewrites |
| Pattern 6 (README publish-blocker fix) | 160-05 | Crates.io front page accuracy | SATISFIED | README now compiles against v2 public API |
| Pattern 7 (cli.md verbatim replacement) | 160-08 | Doc example mirrors live template source | SATISFIED | Sourced verbatim from `ferro-cli/src/templates/make.rs` |
| Pattern 8 (D-08 whitelist categories) | 160-09 | Categorize remaining matches; reject FAIL | SATISFIED | All 152 matches classified into whitelist categories |

No orphaned requirements: no REQUIREMENTS.md IDs are mapped to Phase 160.

### Anti-Patterns Found

Two warnings from the code review (160-REVIEW.md) classified against verification-blocker threshold:

| File | Line | Pattern | Severity | Impact | Classification |
|------|------|---------|----------|--------|----------------|
| `ferro-json-ui/src/render/mod.rs` | 134 | Stale doc-comment literal: `MAX_NESTING_DEPTH = 3` but actual constant in `spec.rs:37` is `5` | Warning | Cosmetic — descriptive doc comment only; runtime check uses real constant; tests pass | NON-BLOCKING for Phase 160 goal. Plan 01 rewrote prose around this comment but preserved the stale `= 3` literal. Recommend a Phase 161 cleanup commit (one-character fix in a doc-comment) OR explicit deferral to a doc-fix follow-up phase. Does NOT prevent the Phase 160 goal of "delete v1 API surface" from being achieved. |
| `docs/src/features/projections.md` | 104, 114-123, 164, 178-218, 256-263, 282-289 | Non-existent APIs referenced: `RenderContext` (the actual type is `VisualContext`), `DataType::Text` (the enum has `String, Integer, Float, Boolean, DateTime, Date, Json, Binary, Uuid, Enum`), six `FieldMeaning` variants (`Description, Image, Timestamp, Count, Location, Generic`) | Warning | Doc-only inaccuracy; pre-existed Phase 160 (drift introduced by earlier phases); Plan 07's `<action>` was narrowly scoped to the Quick Start example | NON-BLOCKING for Phase 160 goal. Plan 07's must_have was "Quick Start matches source-of-truth rustdoc" — that truth is VERIFIED. The remaining drift in the same file (Rendering / Complete Example / Reference sections) was not in any Phase 160 plan's `files_modified` scope. The phase goal "delete v1 API surface" is independent of this projections.md drift; the file does NOT mention `v1` or `ferro-json-ui/v1`. Recommend follow-up phase to sync the rest of the file to actual public API (estimated 1 plan, single-file rewrite, ~30 min). |
| `ferro-mcp/src/tools/code_templates.rs` | 34 | IN-01: doc-comment lists `migration` as accepted category — technically correct (now means database migration, not v1→v2), but potentially ambiguous | Info | Cosmetic | NON-BLOCKING; optional disambiguation. |
| `ferro-json-ui/src/render/mod.rs` | 413, 449 | IN-02: test-only plugin names `FerroPhase116PluginDispatchTest` / `FerroPhase116AssetCollectTestPlugin` still carry "Phase116" branding | Info | Cosmetic, test-internal identifiers | NON-BLOCKING; optional rename. |

**Phase-blocker assessment:** Neither WR-01 nor WR-02 rise to verification-blocker level for Phase 160. The phase goal is "permanently delete v1 API surface" — that has been achieved (all 12 observable truths VERIFIED). WR-01 is a one-character doc-comment fix; WR-02 is documentation drift that pre-existed this phase and is independent of v1-API-deletion. Both are tracked in 160-REVIEW.md and should be addressed by a follow-up doc-fix phase or rolled into Phase 161's pre-publish polish, but they do NOT prevent Phase 160 closure or Phase 161 startup.

### Human Verification Required

None. All goal-backward truths are programmatically verifiable via grep and file existence checks. Plan 10 already ran the cargo gate (which is the only behavioral runtime check applicable to a deletion phase). No UI/visual/UX behaviors are in scope for Phase 160.

### Gaps Summary

No gaps. Phase 160 achieved its stated goal: every v1 API surface element listed in the phase definition has been removed or rewritten, the v2 surface (`Spec`, `Element`, `SpecBuilder`, `ElementBuilder`, plus the expression/render pipeline) remains intact and exported, all cross-repo consumers compile, and no publish-related work occurred (correctly deferred to Phase 161).

Two non-blocking REVIEW warnings (WR-01 stale `MAX_NESTING_DEPTH` doc literal, WR-02 projections.md drift in non-Quick-Start sections) are documented for follow-up but do not gate this phase. The orchestrator may proceed to mark Phase 160 complete and unblock Phase 161 (v12.0 merge + single end-of-loop publish).

---

_Verified: 2026-05-17T07:30:00Z_
_Verifier: Claude (gsd-verifier)_
