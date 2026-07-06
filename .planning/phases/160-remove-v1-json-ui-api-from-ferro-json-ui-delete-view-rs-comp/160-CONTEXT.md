# Phase 160: Remove v1 JSON-UI API — Context

**Gathered:** 2026-05-17
**Status:** Ready for planning
**Mode:** `--auto` (single pass; all gray areas resolved with recommended defaults)

<domain>
## Phase Boundary

Permanently delete every remaining v1 JSON-UI artifact from the workspace and verify the three consumer repos (`ferro`, `ferro-code`, `gestiscilo`) still build and test green.

**In scope**

1. Verify the v1 type surface is absent from production source (`JsonUiView`, `Component` enum, `ComponentNode`, `PluginProps`, `view.rs`, `SCHEMA_VERSION = "ferro-json-ui/v1"`).
2. Delete stale v1 references that survived the bulk migration:
   - Internal port-comments in `ferro-json-ui/src/render/containers.rs` and `ferro-json-ui/src/render/form.rs`.
   - `ferro-mcp/src/tools/code_templates.rs::migration_v1_to_v2_templates()` and its registration.
   - `ferro-mcp/src/tools/application_info.rs::scan_json_ui_specs` legacy v1 scanner.
   - Doc-comment in `ferro-mcp/src/tools/json_ui_inspect.rs` test fixture if it surfaces "v1" framing.
3. Reframe public docs that still narrate v1: `docs/protocol/src/{terminology,architecture,rendering}.md`, `docs/src/features/projections.md`. JSON-UI is the only version that exists in agent-readable surface.
4. Verify cross-repo: ferro builds + tests + clippy; gestiscilo and ferro-code consume ferro via local-path and pass their own suites.

**Out of scope**

- The v12.0 merge to master (Phase 161 owns the merge PR).
- Any new feature surface — this phase only removes.
- The on-the-wire literal `"ferro-json-ui/v2"` in `SCHEMA_VERSION` (it is the wire identifier, not a version label; leave it alone).
- `crates.io` publishing (Phase 161 / single end-of-loop publish per cadence).

</domain>

<decisions>
## Implementation Decisions

### Cleanup scope

- **D-01:** Deepest scope. Verification + stale-reference cleanup + public-doc reframing — not verification-only. Phase mandate is "permanently delete v1 surface"; that includes stale prose and dead MCP plumbing, not just the Rust types.

### Internal port-comments (`ferro-json-ui/src/render/{containers,form}.rs`)

- **D-02:** Delete all v1-framing prose from doc comments. Two confirmed sites:
  - `containers.rs:631-635` — `/// Port of v1 render_button_group ... Note: v1 iterated props.buttons: Vec<ComponentNode>; v2 takes children from Element.children ...`
  - `form.rs:33-39` — `/// Port of v1 render_form ... Differences from v1: ...`
  Replace each with a neutral description of what the function does today. Provenance lives in `git log`/`git blame`, not in the doc comment.
- **D-03:** Sweep `ferro-json-ui/src/` for any remaining `v1`, `legacy`, `removed`, `Port of`, `Differences from v1` framing introduced during the v2 cutover. Treat any match as a cleanup candidate; leave only mentions that are themselves the wire literal `"ferro-json-ui/v2"`.

### MCP migration code-templates (`ferro-mcp/src/tools/code_templates.rs`)

- **D-04:** Delete `fn migration_v1_to_v2_templates()` and the `templates.extend(migration_v1_to_v2_templates());` registration at the top of the registry. Delete the corresponding integration test asserting "at least 7 migration_v1_to_v2 templates". Rationale: per the Phase 164 audit, the sole consumer (gestiscilo) is fully migrated; there are no v1 codebases the MCP needs to help migrate. Per the user-feedback constraint on naming, no migration story belongs in agent-readable surface.

### `application_info` JSON-UI scanner (`ferro-mcp/src/tools/application_info.rs::scan_json_ui_specs`)

- **D-05:** Replace, do not delete. The introspection value (telling agents how many JSON-UI views a project ships) is real. Rewrite to scan v2-shaped surface:
  - Count `*.json` spec files under `src/views/` (current shape).
  - Optionally also count controller call sites of `JsonUi::render_file(...)` and `Spec::builder()`/`Spec::from_json`.
  Remove the `Scans for legacy v1 patterns. TODO(Phase 120): ...` doc comment; replace with a neutral description. The status struct field names should describe what is there, not what was there.

### `json_ui_inspect.rs` test fixture

- **D-06:** Audit `ferro-mcp/src/tools/json_ui_inspect.rs:307` (`write_file(&views_dir, "old_view.rs", "// old v1 file");`). If the test still exercises a meaningful behavior, rename the fixture (`stale_view.rs` + neutral comment) so no test asserts on the literal `"v1"`. If the test was specifically validating "we ignore old v1 view.rs files" and that scenario no longer applies after Phase 160, delete the test.

### Public documentation reframe

- **D-07:** Reframe — do not just substitute the version string.
  - `docs/protocol/src/terminology.md:98` — `JsonUiRenderer produces ferro-json-ui/v1 component trees, but ...` → describe what the renderer produces (a `Spec` conforming to the `"ferro-json-ui/v2"` schema URI) without contrasting against an earlier shape.
  - `docs/protocol/src/architecture.md:172` — same treatment for the `JsonUiRenderer` paragraph.
  - `docs/protocol/src/rendering.md:136` — rewrite the sentence about "ferro-json-ui/v1 schema with envelope ..." to describe the v2 wire shape (`schema`, `root`, `elements`) as-it-is.
  - `docs/src/features/projections.md:42` — update inline code comment `// json["$schema"] == "ferro-json-ui/v1"` to match the actual current assertion (`spec.schema == "ferro-json-ui/v2"` or the example in `ferro-json-ui/src/spec.rs:1009`).
- **D-08:** Sweep `docs/src/` and `docs/protocol/src/` for any remaining narrative `v1`, `legacy`, `Migrating from`, `was removed`, `in v2`, `since v2`. Treat each as a rewrite target; describe what JSON-UI is, not what it was.

### Cross-repo verification gate

- **D-09:** Verify all three repos in this phase (do not defer to Phase 161). Phase 161 is the merge; verification is the deletion gate.
  - **ferro:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` on `v12.0/json-ui-v2`.
  - **gestiscilo:** point `ferro = { path = "../ferro" }`, run the gestiscilo suite (the consumer that drove the friction loop). Per memory `project_v12_merge_task.md`, gestiscilo already consumes local-path ferro during friction phases 138-143.
  - **ferro-code:** local-path consume + that repo's standard test command.
  Each must be green. Any compile error in a consumer means a missing v2 equivalent and re-opens the audit.

### Post-deletion grep gate

- **D-10:** Final gate must show zero matches for `\b(JsonUiView|ComponentNode|PluginProps)\b` across `ferro-json-ui/`, `framework/`, `ferro-mcp/` source trees, and zero matches for `ferro-json-ui/v1` workspace-wide except inside `.planning/` (planning files are historical and stay).

### Cadence

- **D-11:** No publish in Phase 160. The single end-of-loop publish happens at Phase 161 (v12.0 merge to master). Per memory `feedback_friction_loop_release_cadence.md`, mid-loop publishes freeze the API before later batches revise it.

### Claude's Discretion

- Exact rewording of each doc comment and prose passage — the constraint is "neutral, present-tense, no v1 framing"; specific phrasing is a planning/execution detail.
- Whether `application_info::scan_json_ui_specs` also walks controllers for `JsonUi::render_file` call sites or stays file-count-only — value is similar; pick the simpler of the two at plan time.
- Test re-organization in `json_ui_inspect.rs` if D-06 forces a rename — keep coverage equivalent.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 160 inputs

- `.planning/ROADMAP.md` §"Phase 160" (line 1566) — goal, requirements, depends-on.
- `.planning/ROADMAP.md` §"Phase 161" (line 1576) — downstream merge phase; informs cadence.
- `.planning/phases/164-json-ui-improvements-batch-3-documenti-field-test-findings-m/V1-DELETION-AUDIT.md` — full v1→v2 surface audit; BLOCKER count = 0; lists every v1 element and its v2 equivalent.
- `.planning/phases/164-json-ui-improvements-batch-3-documenti-field-test-findings-m/COMPLETED.md` — §5 is the migration table; §1 catalogues every v2 capability shipped during the friction loop.
- `.planning/phases/164-json-ui-improvements-batch-3-documenti-field-test-findings-m/PLUGIN-SURFACE-AUDIT.md` — D-06 plugin paper-audit (both gaps already closed).
- `.planning/phases/159-v12-0-end-to-end-browser-verification-and-docs-build-check/159-VERIFICATION.md` — browser/docs gate (root-cause path bug fixed in commit `6601c015`; verification must be re-run after rebase if Phase 159's gap-found status has not been re-cleared).

### Current v2 surface (sanity-check targets)

- `ferro-json-ui/src/spec.rs:31` — `pub const SCHEMA_VERSION: &str = "ferro-json-ui/v2";` (wire literal; LEAVE).
- `ferro-json-ui/src/spec.rs` — `Spec`, `Element`, `SpecBuilder`, `ElementBuilder`, `TitleBinding`, `DataRef` — the post-Phase-160 public surface.
- `ferro-json-ui/src/component.rs` — typed `*Props` structs reused by the v2 renderer (these stay; only v1-only types are gone).
- `ferro-json-ui/src/render/mod.rs` — `BUILTIN_TYPES` (41 entries) is the live catalog assertion.

### Public-doc rewrite targets

- `docs/protocol/src/terminology.md:98`
- `docs/protocol/src/architecture.md:172`
- `docs/protocol/src/rendering.md:136`
- `docs/src/features/projections.md:42`
- `docs/src/json-ui/*.md` — already free of v1 framing (verified 2026-05-17); no rewrites expected here, but sweep with the D-08 grep.

### Source cleanup targets

- `ferro-json-ui/src/render/containers.rs:631-635`
- `ferro-json-ui/src/render/form.rs:33-39`
- `ferro-mcp/src/tools/code_templates.rs:78-79, 1504-1820, 1820-1827` (template fn + extend call + test)
- `ferro-mcp/src/tools/application_info.rs:244-258`
- `ferro-mcp/src/tools/json_ui_inspect.rs:307`

### Constraints (load-bearing for downstream agents)

- Memory `feedback_json_ui_naming.md` (user's private memory; not a repo file): the naming/framing constraint is mirrored as decisions D-02/D-03/D-07/D-08 above so the planner can execute without needing to read memory.
- Memory `feedback_friction_loop_release_cadence.md` (private memory): mirrored as D-11 (no mid-loop publish).
- CLAUDE.md "Repository documents must read as neutral" — reinforces D-07/D-08 framing rule.

</canonical_refs>

<code_context>
## Existing Code Insights

### Verified-absent v1 surface (audit complete)

- `ferro-json-ui/src/view.rs` — deleted in commit `dbe5adaf`.
- `JsonUiView`, `ComponentNode`, `PluginProps`, `Component::` — zero production matches (2 doc-comment historical mentions remain; D-02 deletes them).
- `framework/src/lib.rs` v1 re-exports — zero matches.
- `ferro-json-ui/src/lib.rs` v1 re-exports — zero matches.
- `ferro-json-ui/v1` schema string in live code — zero matches (`SCHEMA_VERSION = "ferro-json-ui/v2"`).

### Surviving v2 public surface (untouched by Phase 160)

- `Spec`, `Element`, `SpecBuilder`, `ElementBuilder`, `NestedElement`, `TitleBinding`, `DataRef`
- The component `*Props` structs reused by the v2 renderer (`CardProps`, `FormProps`, `GridProps`, `KanbanBoardProps`, `ImageProps`, `DescriptionListProps`, `PageHeaderProps`, etc.)
- Plugin trait surface: `JsonUiPlugin`, `register_plugin`, `register_built_in_plugins`, `RawHtml` primitive
- Expression/render pipeline: `expand_directives`, `apply_visibility`, `render_element`
- Loader/render: `JsonUi::render_file`, `Spec::from_json`

### Established patterns to follow

- **Neutral doc-comment style:** Files written for v2-only (no port history) describe what the function does in the present tense. See `ferro-json-ui/src/render/atoms.rs` or `ferro-json-ui/src/render/mod.rs` as exemplars.
- **Schema URI literal stays:** `"ferro-json-ui/v2"` is the on-the-wire identifier — keep it everywhere it appears as a literal string. The cleanup targets prose/comments, not the literal.

### Integration points

- `ferro-mcp` ships in-process with `ferro mcp` subcommand — deleting `migration_v1_to_v2_templates` shrinks the catalog returned by `code_templates`; the registration tests need a matching update.
- Consumer repos (gestiscilo, ferro-code) consume ferro via local-path during the friction loop; verification needs both repos on disk and pointing at the v12.0/json-ui-v2 worktree.

</code_context>

<specifics>
## Specific Ideas

- The audit's spot-check `grep` commands (`V1-DELETION-AUDIT.md` lines 80-105) are the right shape for the final gate; re-use them rather than inventing new ones.
- For the `render/{containers,form}.rs` rewrite: the closest exemplar style is the neutral prose in `ferro-json-ui/src/render/mod.rs` and `atoms.rs` — describe what the function emits and what props it reads, no provenance narrative.
- The MCP template-deletion test should be a true removal (delete the assertion + the parent function) rather than a count update — leaving "expected at least 0 migration templates" as a green-test artifact is itself noise.

</specifics>

<deferred>
## Deferred Ideas

- **`LoadError::Catalog` variant cleanup** — Already deferred by `COMPLETED.md` §4. Stays deferred past Phase 160.
- **Unified `$if` + `visible` directive** — `COMPLETED.md` §4 candidate for v12.1+. Out of scope.
- **`Modal` chrome variant** — `COMPLETED.md` §3 (intentional gap). Stays deferred.
- **Granular `Card` props (`padding`, `elevation`)** — `COMPLETED.md` §3 (intentional gap). Stays deferred.
- **Codemod directory-recursive mode** — `COMPLETED.md` §3 (intentionally rejected at 163 D-10). Stays out.
- **v12.0 CHANGELOG drafting** — Phase 161 owns; uses `COMPLETED.md` §1 as input.
- **crates.io publish** — Phase 161 owns; per release-cadence memory, single publish at end of friction loop.

</deferred>

---

*Phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp*
*Context gathered: 2026-05-17*
