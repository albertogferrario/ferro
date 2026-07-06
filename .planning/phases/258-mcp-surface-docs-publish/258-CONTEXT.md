# Phase 258: MCP Surface + Docs + Publish - Context

**Gathered:** 2026-07-06 (auto mode — recommended defaults selected, logged in 258-DISCUSSION-LOG.md)
**Status:** Ready for planning

<domain>
## Phase Boundary

Close the v16.6 POS Component Suite milestone: make the register composition
discoverable and authorable through `ferro-mcp` without consulting source code,
document the five new builtins and the register projection surface in
`docs/src`, run the full CI-exact gate, and perform the single crates.io
publish that unblocks gestiscilo's register phase. Requirements: POS-12,
POS-13.

**Killer feature framing:** this phase completes the agent-authoring loop for
the register — an agent discovers `TileGrid`/`SelectionPanel`/`FilterTabs`/
`QuantityStepper`/`Numpad` through `json_ui_catalog`, learns WHEN and HOW to
compose a sale screen through `generation_context` (including the one-call
`register_template()` Collect→Register projection path — one `ServiceDef` →
working sale screen), and validates with `design_lint` — all before human
review. The publish makes it pinnable for the consumer.

Out of scope: new components or renderer changes (256 closed the catalog at
52), new lint rules (254/255 own them), projection/builder changes (257
shipped them; only bug-fix level touches), payment flow / receipts / shift
close (standing milestone deferrals), the gestiscilo adoption work itself
(consumer repo, brief-only handoff).

**World-state corrections found during scouting
(feedback_validate_scope_premises):**
1. **SC-1 is already satisfied in-tree.** Phase 256 bumped both drift-guard
   counts to 52 in the same commits per component addition; the ferro-mcp
   mirror (`ferro-mcp/src/tools/json_ui_catalog.rs:405`) already asserts 52
   AND all five component names. This phase re-runs the tests and records
   the evidence — verification must not claim the count/name work as this
   phase's output.
2. **SC-4's version bar is stale.** crates.io already carries ferro-rs
   **0.2.88** (no-op auto-bumps 0.2.87/0.2.88 shipped from remote master on
   ferro-a2ui changes). The publish bar is "> 0.2.88", i.e. 0.2.89+ — not
   the roadmap's "> 0.2.86".
3. **POS-13's `/cassa` flip already shipped in Phase 257** (projection-derived
   spec, `cassa.json` deleted, UAT passed). This phase verifies it stands;
   the substantive POS-13 work here is the gate + publish.
4. **`generation_context` has zero register content today** (498 lines, no
   "register" mention) and `docs/src/json-ui/components.md` documents only
   `Tile` of the six touch-first components — SC-2 and SC-3 are the real
   scope of this phase, alongside the publish.

</domain>

<decisions>
## Implementation Decisions

### json_ui_catalog surface (POS-12) — verification-first
- **D-01:** SC-1 is pre-satisfied (world-state correction 1). Work = run
  `test_all_components_present` + both count assertions
  (`catalog.rs:1296` canonical, `json_ui_catalog.rs:405` mirror) and record
  them as pre-existing evidence. No re-implementation, no count churn.
- **D-02:** Per-component design guidance for the five new components rides
  the 253 D-05 derived mapping (`design::rules()` registry / `RULE_COMPONENTS`
  — the `register-*` rules already reference the new component names). Audit
  the `json_ui_catalog` output for the five components; fix only additive
  gaps found. Any static supplement carries a drift test tying it to the
  registry/enum set (single-registry philosophy). All changes additive —
  existing output shape stays backward-compatible.

### generation_context register guidance (POS-12)
- **D-03:** Content contract (SC-2, all six items): (a) when to use the
  Register layout template vs. a form-only Collect spec; (b) the form-state
  selection contract — hidden-input quantity accumulation (`data-qty-input`),
  ONE confirm POST, SelectionPanel as a live client-side VIEW of form state,
  never a second source of truth; (c) the filter/numpad data attributes
  (`data-filter-tokens`, `data-filter-text`, numpad target-field wiring);
  (d) the `fill_viewport` dependency for register layouts + the supported
  shell layouts (app/dashboard) per the ferro-fill CSS chain; (e) the four
  `register-*` lint rule ids agents should check via `design_lint`;
  (f) a pointer to `register_template()`
  (`ferro-json-ui/src/projection/intent_layout.rs:50`) as the one-call
  Collect→Register override and the projection-derived `/cassa` sample as
  the composition reference.
- **D-04:** Style mirrors 253 D-06: compact — ids and one-liners with a
  pointer to `docs/src` for depth. `generation_context` is inline agent
  context, not a manual.
- **D-05:** Derive what is derivable (rule ids/rationale from
  `design::rules()`, component names from the builtin registry); hand-written
  register prose is drift-guarded: a test asserts every component name, rule
  id, and data attribute mentioned in the register guidance exists in its
  authoritative source (BUILTIN registry, rule registry, runtime attribute
  contract) — the 253 D-09 pattern applied to generation_context.
- **D-06:** Numpad guidance documents it as an author-composable addition —
  it is NOT part of the v1 register template (257 D-07 handoff discharged
  here). Same for adding a standalone FilterTabs outside the TileGrid
  integrated strip.

### docs/src updates (POS-12)
- **D-07:** Five new component sections in `docs/src/json-ui/components.md`
  — TileGrid, SelectionPanel, FilterTabs, QuantityStepper, Numpad — each with
  a props table and at minimum one usage example, following the existing
  per-component format (the `Tile` section at components.md:1411 is the
  format anchor). Domain-neutral voice throughout; commerce naming only in
  examples explicitly framed as samples.
- **D-08:** The register projection surface is documented per the 257
  verification handoff: the `layout: "Register"` template arm, the
  `register_template()` helper, `ElementBuilder.each(path, as_)`, and
  `SpecBuilder.fill_viewport(bool)`. Placement is planner's call within this
  constraint: extend existing pages first (`json-ui/layouts.md` for the
  Register layout + fill_viewport chain; `json-ui/spec-construction.md` for
  the builder API; components.md cross-links) over inventing new pages; if a
  dedicated register/composition page is warranted, wire it into
  `docs/src/SUMMARY.md`. Cross-link, never duplicate (253 D-08 practice).
- **D-09:** Docs also cover the interaction model (tap-to-add tiles — one tap
  adds one unit; ALL quantity editing in the SelectionPanel), the
  `disable_on_submit` double-submit guard + idempotency-key pattern pointer
  (255 D-16/D-18), and the `Form` common-ancestor scoping requirement for the
  hidden-input contract.
- **D-10:** mdBook docs build exits 0 (SC-3 gate). Neutral product
  documentation voice; no internal-strategy framing; no version-vs-version
  comparison framing (feedback_json_ui_naming).

### Publish + gate (POS-13)
- **D-11:** ONE final workspace bump **0.2.88 → 0.2.89** as the publish
  commit (world-state correction 2), manual bump so CI publishes directly
  with no double-bump (established 0.2.75/0.2.85 practice). No mid-phase
  publishes.
- **D-12:** Branch topology: all Phase 256/257 work sits on
  `feat/billable-return-url-seam` — 140 commits ahead of remote master;
  remote master IS an ancestor of HEAD (clean fast-forward push); local
  master is 74 commits behind HEAD and holds nothing unique. Land 258 work on
  this branch, then fast-forward local master to the branch head **from the
  main repo root with `HEAD`=master asserted first**
  (feedback_worktree_merge_cwd_trap), then push master via the gh HTTPS
  credential helper (SSH is denied for this repo).
- **D-13:** The branch base carries ferro-payments **0.1.5 → 0.1.6**
  (defaulted `Billable::success_url`/`cancel_url` return-URL seam,
  backward-compatible; the bump is already committed in `4477e394`).
  crates.io ferro-payments is at 0.1.5, so this push also publishes
  ferro-payments 0.1.6. Publish verification must confirm BOTH ferro-rs
  0.2.89 AND ferro-payments 0.1.6 on crates.io.
- **D-14:** `ferro-a2ui` stays `publish = false` and out of `publish.yml`
  (experimental crate, gated). No new crates in v16.6 → no publish.yml wave
  changes, no publish-new token bootstrap.
- **D-15:** CI-exact gate before the publish push:
  `cargo fmt --all -- --check`,
  `cargo clippy --all --all-targets --all-features -- -D warnings`,
  `cargo test --all-features`, plus `cargo doc` with `-D warnings` and
  cargo-deny awareness — CI's matrix is wider than the three-command gate
  (feedback_ci_matrix_wider_than_local_gate). Re-run fmt after ANY hand-edit.
  Serialize CPU-heavy runs (one at a time); check disk space and clean
  `target/` before the full `--all-features` test gate (recurrent ENOSPC).
  Schema-export churn is discarded unless a real diff appears.
- **D-16:** The publish step is operator-gated (236/253 practice): present a
  pre-publish checklist at the gate (gate results, version bumps, what ships
  — including the ferro-payments 0.1.6 rider). Post-publish: verify via
  crates.io / gh API, never local `origin/*` refs (recurrent local-refs lie);
  run `git update-ref refs/remotes/origin/master HEAD` after a verified push.
- **D-17:** gestiscilo handoff is a brief only
  (feedback_cross_repo_phase_split): their register phase pins ferro-rs
  0.2.89. Never edit the consumer tree or its planning from this session.
- **D-18:** Stray untracked planning artifacts from other phases (209, 212,
  214, 231, 232, 238, 251, 252, 253 files and `app/tmp/`) and the phantom
  `planning/phases/158-…` deletion stay OUT of Phase 258 commits — stage
  specific files only.

### Claude's Discretion
- Exact docs placement within the D-08 constraint (extend existing pages vs.
  one new register page) and section ordering inside components.md.
- Exact `generation_context` section naming/structure for the register
  guidance and how much rule rationale is embedded verbatim vs. trimmed.
- Whether any catalog-guidance gap found under D-02 is fixed in ferro-json-ui
  or ferro-mcp (whichever owns the derivation point).
- Pre-publish checklist composition details at the D-16 gate.
- Test organization for the D-05 drift guards.

### Folded Todos
None — `todo match-phase 258` returned 0 matches.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase definition + requirements
- `.planning/ROADMAP.md` — v16.6 Phase 258 section (goal + SC 1–4, with the
  world-state corrections above applied) and the milestone scope constraints
  (structural vocabulary, single publish, no new crates).
- `.planning/REQUIREMENTS.md` — POS-12, POS-13 text.

### Prior phase contracts this phase documents and publishes
- `.planning/phases/257-projection-builder-register-layout-template/257-CONTEXT.md`
  — D-02 (`register_template()` helper), D-07 (Numpad NOT in v1 template —
  the guidance obligation discharged here), D-09/D-10 (meaning-driven Tile
  mapping + per-row data contract that generation_context describes), D-19
  (docs deferred to 258).
- `.planning/phases/257-projection-builder-register-layout-template/257-VERIFICATION.md`
  — the explicit docs handoff (`register_template()`, `each()`,
  `fill_viewport()` documented in docs/src → Phase 258).
- `.planning/phases/256-component-renderers-builtin-lockstep/256-CONTEXT.md`
  — interaction model (tap-to-add, SelectionPanel live view), D-04
  (`price_cents`/`data-unit-price` contract), component render contracts the
  docs sections describe.
- `.planning/phases/255-pos-runtime-modules-double-submit-protection/255-CONTEXT.md`
  — final attribute vocabulary (V-01..V-05: `data-filter-tokens`,
  `data-filter-text`, `data-qty-*`), D-16 (`disable_on_submit`), D-18
  (idempotency pattern).
- `.planning/phases/253-mcp-surface-docs-publish/253-CONTEXT.md` — the
  sibling publish-phase pattern this phase mirrors: D-05/D-06 (derived
  catalog/generation_context content), D-09 (docs drift guards), D-11..D-16
  (publish mechanics, operator gate, consumer handoff).
- `.planning/phases/253-mcp-surface-docs-publish/253-FRICTION.md` — the
  gestiscilo picker audit (the consumer need the register guidance answers).

### Milestone research (2026-07-04)
- `.planning/research/FEATURES.md`, `.planning/research/PITFALLS.md` —
  register composition evidence; integer-cents and fill-viewport pitfalls
  the docs must state.

### Source anchors (current on `feat/billable-return-url-seam`)
- `ferro-mcp/src/tools/generation_context.rs` — 498 lines, zero register
  content today; the SC-2 target. Follow the existing section style from the
  253 design-system summary.
- `ferro-mcp/src/tools/json_ui_catalog.rs` :405 — mirror count assertion,
  already 52 with all five names (D-01 evidence).
- `ferro-json-ui/src/catalog.rs` :1296 — canonical count assertion (52).
- `ferro-json-ui/src/projection/intent_layout.rs` :50 —
  `pub fn register_template() -> ThemeTemplates` (the helper D-03(f) points
  agents at).
- `ferro-json-ui/src/design/rules.rs` — the four `register-*` rules +
  `RULE_COMPONENTS` (D-02 derivation source; rationale text for D-05).
- `docs/src/json-ui/components.md` :1411 (`### Tile` — format anchor;
  migration table with the tap-to-add note), `docs/src/json-ui/layouts.md`,
  `docs/src/json-ui/spec-construction.md`, `docs/src/SUMMARY.md` (:63-79
  json-ui + design-system sections).
- `app/src/controllers/cassa.rs` — the projection-derived sample the docs
  and generation_context reference.
- `.github/workflows/publish.yml` — publish waves (verify no changes needed;
  ferro-a2ui absence is intentional).
- `Cargo.toml` :47 — workspace version 0.2.88 (D-11 bump target);
  `ferro-payments/Cargo.toml` :3 — 0.1.6 already bumped (D-13).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- The 253 `generation_context` design-system summary + its drift-guard tests
  — the exact structural pattern (compact ids + one-liners + docs pointer +
  registry-tied tests) the register guidance extends.
- `design::rules()` registry with `rationale` fields — machine source for
  lint-rule prose in both generation_context and docs.
- The `Tile` section in components.md — the per-component docs format
  (description, props table, data-attribute notes, Form-pairing note).
- The projection-derived `/cassa` controller — a working, lint-clean,
  UAT-passed register composition to lift examples from.
- The 253-05 publish plan artifacts
  (`.planning/phases/253-mcp-surface-docs-publish/253-05-PLAN.md` +
  SUMMARY) — the operator-gated publish choreography executed successfully
  once already.

### Established Patterns
- Verification-first for pre-satisfied SCs: record evidence, never re-do or
  claim credit (257 world-state practice).
- Drift-guard every hand-written surface that mirrors a registry.
- CI-exact gate incl. `--all-features` + docs build; fmt after any
  hand-edit; serialize CPU-heavy runs; disk check before full test runs.
- Operator-gated publish with post-publish crates.io/gh-API verification.

### Integration Points
- `ferro-mcp/src/tools/generation_context.rs` — register guidance section.
- `ferro-mcp/src/tools/json_ui_catalog.rs` + `ferro-json-ui` catalog/rules —
  D-02 audit surface.
- `docs/src/json-ui/*` (+ `SUMMARY.md` if a page is added) — SC-3.
- `Cargo.toml` workspace version — the single publish commit.
- Consumer: gestiscilo register phase pins the published 0.2.89 (brief-only
  handoff).

</code_context>

<specifics>
## Specific Ideas

- The phase's one-sentence user story: "an agent that has never seen the
  ferro source composes a working register screen from MCP context alone."
  generation_context quality is where the polish budget goes — the catalog
  count and the publish choreography are commodity.
- The compressive claim to preserve in all prose: one `ServiceDef` → a
  working tablet sale screen via `register_template()`; the register needed
  zero new intents, zero new crates, five new builtins.
- Mirror 253's structure deliberately — same phase shape, same publish
  gate, one milestone later. Where 253 wrote the design-system docs chapter,
  258's docs load is component sections + the register projection surface.

</specifics>

<deferred>
## Deferred Ideas

- **Numpad in the register template** — standing 257 deferral; 258 only
  documents manual composition; revisit on gestiscilo friction.
- **Register template knobs** (pane ratios, order, search toggle) — v1 stays
  opinionated defaults; parameterize only on consumer friction.
- **Sibling FilterTabs↔TileGrid pairing** (`data-filter-for`) — still
  deferred from 256 D-18.
- **Category strip derivation hint** — still deferred from 257 D-07; no
  ServiceDef hint without evidence.
- **Barcode wedge, payment flow, receipts, shift close** — standing
  milestone deferrals.
- **v16.6 milestone archival** (`/gsd-complete-milestone`) — after this
  phase; the archive backlog also holds v16.0/16.1/16.2/16.3/16.5.

### Reviewed Todos (not folded)
None — no pending todos matched this phase.

</deferred>

---

*Phase: 258-mcp-surface-docs-publish*
*Context gathered: 2026-07-06*
