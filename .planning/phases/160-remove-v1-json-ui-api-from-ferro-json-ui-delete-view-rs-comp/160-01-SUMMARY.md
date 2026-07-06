---
phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp
plan: 01
subsystem: ui

tags: [ferro-json-ui, render, doc-comments, neutral-voice, json-ui]

# Dependency graph
requires:
  - phase: 164-json-ui-improvements-batch-3-documenti-field-test-findings-m
    provides: V1-DELETION-AUDIT, zero v1 production-source surface confirmation
provides:
  - Neutral, agent-readable doc comments across the ferro-json-ui render pipeline
  - No v1-framing prose (Port of, Differences from, Replaces v1, Matches v1, render.rs line ranges)
  - Removed dead `_plan_02_reserved` placeholder from projection/builder.rs
  - Updated v1-schema test fixture in layout.rs to the current wire identifier
affects:
  - 160-02 — ferro-mcp code_templates deletion (D-04)
  - 160-03+ — remaining ferro-mcp scanner / fixture / public-doc rewrites
  - 161 — v12.0 → master merge (consumes a doc-clean ferro-json-ui)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Pattern 1 (Neutral Doc-Comment Style): module-level + per-function doc comments describe what the function emits, what props it reads, and what edge cases it handles in present-tense voice — no historical / phase / decision provenance, no contrast against a removed implementation."

key-files:
  created: []
  modified:
    - ferro-json-ui/src/render/mod.rs
    - ferro-json-ui/src/render/atoms.rs
    - ferro-json-ui/src/render/containers.rs
    - ferro-json-ui/src/render/form.rs
    - ferro-json-ui/src/render/data.rs
    - ferro-json-ui/src/projection/builder.rs
    - ferro-json-ui/src/layout.rs

key-decisions:
  - "Sweep was broader than the two CONTEXT D-02 named sites. Every section header in atoms.rs that carried `(v1 render.rs lines XXX-XXX)` provenance was rewritten because D-03 mandates removal of any v1 framing and the artifact spec for atoms.rs explicitly says `no v1`."
  - "Deleted `_plan_02_reserved` function (not just the stale comment). Grep confirmed zero callers in ferro-json-ui/; the function existed solely to suppress unused-import warnings during a prior plan transition that has long shipped."
  - "Updated `view_json: \"{\\\"schema\\\":\\\"v1\\\"}\"` test fixture to `\"ferro-json-ui/v2\"`. Test does not assert on the literal value; rewrite is cosmetic but aligns with the no-v1-framing rule."
  - "Preserved every documented behavior. Doc comments that previously said `Differences from v1: ...` had their substance lifted into a present-tense behavior description (e.g. form.rs `render_form` retains the HTTP method spoofing and `action.url = None → action=\"#\"` fallback notes — just stated as current behavior, not as a contrast)."

patterns-established:
  - "Pattern 1: Module-level doc + per-function doc + inline section comments in render/ describe present-tense behavior only. No phase numbers, no decision IDs (`D-XX`), no historical contrast against an earlier shape, no render.rs line-range provenance. Future renderer additions follow the same voice."

requirements-completed: [D-01, D-02, D-03]

# Metrics
duration: 42min
completed: 2026-05-17
---

# Phase 160 Plan 01: Sweep v1-framing prose from ferro-json-ui/src/render/ Summary

**Rewrote 30+ doc-comment sites across `render/{mod,atoms,containers,form,data}.rs`, `projection/builder.rs`, and `layout.rs` in present-tense voice, eliminating every `v1`, `Port of`, `Differences from v1`, `Phase 116`, `Per CONTEXT D-XX`, and `render.rs line-range` framing while preserving all documented behavior; deleted dead `_plan_02_reserved` placeholder.**

## Performance

- **Duration:** ~42 min
- **Started:** 2026-05-17 (post-context-load)
- **Completed:** 2026-05-17
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- ferro-json-ui/src/render/ subtree now reads as neutral, agent-readable documentation
- Every `Port of v1 ...`, `Differences from v1 ...`, `Matches v1`, `(v1 verbatim)`, `(v1 fallback)`, `Legacy {row_key}`, `Replaces v1`, `Phase 116`, `Per CONTEXT D-XX`, `render.rs L###-###`, `render.rs:NNN-NNN` framing removed from public-facing doc surface
- Dead `_plan_02_reserved` function deleted (was suppressing unused-import warnings since Plan 03 long ago)
- `layout.rs` test fixture schema literal updated from `"v1"` to `"ferro-json-ui/v2"`
- All ferro-json-ui tests pass; `cargo fmt --all -- --check` and `cargo clippy -p ferro-json-ui --all-targets -- -D warnings` exit 0

## Task Commits

Each task was committed atomically:

1. **Task 1: Rewrite render/mod.rs, atoms.rs, projection/builder.rs and layout.rs test fixture** - `c25e52a2` (docs)
2. **Task 2: Rewrite render/containers.rs and render/form.rs v1-framing comments** - `bfb8fe1b` (docs)
3. **Task 3: Rewrite render/data.rs v1-framing comments** - `0d67e9ca` (docs)

## Files Created/Modified

- `ferro-json-ui/src/render/mod.rs` — Module doc, `BUILTIN_TYPES` doc, `render_spec_to_html` doc, `render_element` inline pipeline comments, `collect_plugin_types`, `html_escape`, `render_css_tags`, `render_js_tags` docstrings rewritten; Phase 115 / Plan 02 / Plan 03 / D-XX references purged from tests block.
- `ferro-json-ui/src/render/atoms.rs` — Module doc, `decode_diagnostic` section comment, SVG-icon section comment, all 23 numbered component-section headers rewritten; per-function commentary on `render_button` and the inline XSS / shimmer notes neutralized.
- `ferro-json-ui/src/render/containers.rs` — Module doc plus doc comments for `render_card`, `render_modal`, `render_tabs`, `render_kanban_board`, `render_page_header`, `render_grid`, `render_collapsible`, `render_form_section`, `render_button_group` rewritten; inline body-wrapper / single-tab / data_path-precedence / decode-check comments and the Collapsible SVG chevron header neutralized; test-block headers and Card / PageHeader / Tabs / KanbanBoard test comments cleaned.
- `ferro-json-ui/src/render/form.rs` — Module doc plus doc comments for `render_form`, `render_input`, `render_select`, `render_checkbox`, `render_switch`, `resolve_checked` rewritten; inline action-URL / hidden-input / datalist / max-width / form-fields comments and the `default_value > data_path` precedence test comment neutralized.
- `ferro-json-ui/src/render/data.rs` — Module doc plus doc comments for `render_table`, `render_data_table`, `cell_string`, `resolve_row_key`, `template_actions` rewritten; inline URL-template / column-key substitution / handler-fallback / default-empty-message comments and the `{row_key}` / `{id}` regression-test comments neutralized.
- `ferro-json-ui/src/projection/builder.rs` — Deleted `_plan_02_reserved` placeholder function and its stale "Silence ... until Plan 03 rewires the legacy renderer" comment (verified no callers).
- `ferro-json-ui/src/layout.rs` — Updated test fixture `view_json: "{\"schema\":\"v1\"}"` → `"{\"schema\":\"ferro-json-ui/v2\"}"`.

## Decisions Made

- Adopted the broader D-03 reading — every `(v1 render.rs lines XXX-XXX)` section header in atoms.rs was rewritten, not just the two CONTEXT D-02 named sites. Rationale: the plan's artifact spec for atoms.rs says `no v1` and the user-facing naming-discipline rule treats every committed doc comment as potentially public.
- Deleted `_plan_02_reserved` outright rather than just removing the comment. Grep confirmed zero callers (`grep -rn '_plan_02_reserved' ferro-json-ui/`); leaving an `#[allow(dead_code)]` placeholder named after a long-shipped plan would have been a new form of historical narrative.
- Preserved every line of documented behavior. Where a doc comment had real substance (e.g. `render_form`'s HTTP method spoofing + `action.url = None → action="#"` fallback, `render_tabs` single-tab auto-hide and server-driven fallback, `data.rs` URL templating precedence), the rewrite carried that substance forward in present-tense voice rather than dropping it.

## Deviations from Plan

None - plan executed exactly as written. The acceptance-criteria grep gates and the workspace-level `! grep -rnE 'Port of v1|...' ferro-json-ui/src/render/` sweep gate from the plan's `<verification>` block all pass.

The plan's Task 1 action also called for the broader D-03 sweep of atoms.rs section headers; this was framed as part of the artifact spec ("no `v1`") and was executed accordingly. Treating that as in-scope rather than as a deviation matches the plan's intent.

## Issues Encountered

None.

## User Setup Required

None — pure documentation rewrites with no behavior change. No external service configuration required.

## Next Phase Readiness

- **160-02 (Containers/Form D-02 explicit sites)**: already covered by Task 2 of this plan; that follow-on plan can verify and close.
- **160-03+ (ferro-mcp `code_templates`, `application_info`, `json_ui_inspect` cleanups)**: source-side render/ subtree is now clean; ferro-mcp work in subsequent plans is independent.
- **Phase 161 (v12.0 → master merge)**: ferro-json-ui doc surface is one of the inputs to the merge; this plan reduces the "agent-facing narrative" debt that would have appeared in the v12.0 changelog.

## Self-Check: PASSED

- File `ferro-json-ui/src/render/mod.rs` — modified, present
- File `ferro-json-ui/src/render/atoms.rs` — modified, present
- File `ferro-json-ui/src/render/containers.rs` — modified, present
- File `ferro-json-ui/src/render/form.rs` — modified, present
- File `ferro-json-ui/src/render/data.rs` — modified, present
- File `ferro-json-ui/src/projection/builder.rs` — modified, present
- File `ferro-json-ui/src/layout.rs` — modified, present
- Commit `c25e52a2` (Task 1) — verified in `git log --oneline`
- Commit `bfb8fe1b` (Task 2) — verified in `git log --oneline`
- Commit `0d67e9ca` (Task 3) — verified in `git log --oneline`
- `cargo fmt --all -- --check` — exits 0
- `cargo clippy -p ferro-json-ui --all-targets -- -D warnings` — exits 0
- `cargo test -p ferro-json-ui --all-features` — passes (5 doc tests + all unit/integration tests)
- `grep -rnE 'Port of v1|Differences from v1|ported verbatim from v1|ported from v1|Replaces v1' ferro-json-ui/src/render/` — 0 matches

---
*Phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp*
*Completed: 2026-05-17*
