# Phase 175: JSON-UI v2 runtime patches — staff-domain field test findings (F1–F6) — Context

**Gathered:** 2026-05-20
**Status:** Ready for planning
**Source:** Consumer field test of v12.0 JSON-UI v2 against a tenant-scoped staff CRUD surface (per-day weekly hours editor with copy-source-to-N-targets shortcut, two-month calendar overlay, multipart avatar upload). Six runtime gaps surfaced; none blocked by spec authoring, all blocked by runtime behavior.

Full triage doc lives in the consumer repo at `.planning/phases/151-staff-domain/UI-FRAMEWORK-FINDINGS.md`. This CONTEXT.md captures the same material restated in ferro-internal terms and adds the planning shape.

## Phase Boundary

Phase 175 is a v12.0 follow-up batch (informally "batch 4" after the Phases 162/163/164 v12.0 friction loop). Scope is exclusively the six runtime findings F1–F6 below; no new component surface beyond what the findings explicitly require, and no architectural moves.

Three responsibilities:
1. **Land the six fixes** as individual plans (one plan per finding) so they can be shipped, reviewed, and reverted independently.
2. **Re-run the consumer field test** against the patched runtime to confirm closure and surface any second-order gaps.
3. **Update v12.0 COMPLETED.md** with the patches and any intentional gaps retained.

The decision boundary against "ship a new component" (Phase 164's bidirectional-adaptation principle) is reproduced here: F2 (`CheckboxGroup`) and F4 (`Switch`) are existing v1 components whose v2 absence was an intentional drop. Each plan must decide between (a) re-introducing the component, (b) documenting a v2-native substitution path the consumer should use instead, or (c) doing both. Default to (b) where a sibling v2 component covers the intent.

## Findings

### F1 — Spec depth limit rejects depth-8 tree-shaped specs

**Symptom:** The runtime parse/render trips at depth 7 for tree-shaped specs that contain no cycles, with the diagnostic comment `<!-- ferro-json-ui: cycle guard tripped at depth 7 — spec should have been rejected at parse time -->` injected at every child node past the limit.

**Evidence:** The consumer's staff detail view reaches depth 8 (`dashboard → root → DetailPage → tab → card → form → row → switch`). The wrapper at depth 7 renders (the row's `<div class="grid">` is present); its children at depth 8 are stripped.

**Two issues conflated:** The diagnostic message names "cycle" but the actual condition is depth-limit. Distinguishing cycle detection from depth-limit makes future failures legible.

**Reference for prior work:** Phase 164 raised `MAX_NESTING_DEPTH` from 3 to 5 (file `ferro-json-ui/src/spec.rs`, constant + `validate_depth`). At least 8 is needed by the current consumer evidence; headroom past 8 is a planning decision.

### F2 — `CheckboxGroup` component not registered in v2 catalog

**Symptom:** Specs declaring components of type `CheckboxGroup` render with the component silently dropped. No warning, no error, no fallback markup. Sibling elements in the same parent render normally; only the missing-component child is absent.

**Evidence:** The consumer's "copy source to N targets" picker declares a `CheckboxGroup` whose `field` is `copy_to[]` and whose `options` come from `{$data: /copy_from_options}`. The `<form>` element renders with 2 children (the source `Select` + the submit `Button`); the `CheckboxGroup` is silently absent from the DOM.

**Decision required:**
- (a) Register `CheckboxGroup` as a first-class v2 component with the same semantics as v1
- (b) Document the v2 substitution: `Form` with N repeated `Checkbox` children whose `field` ends in `[]`, sharing a `name="copy_to[]"` for array submission semantics. This is more verbose but compositionally simpler.
- (c) Both: register the component AND document the alternative for ad-hoc use

### F3 — Tabbed pages render every panel concurrently

**Symptom:** `DetailPage`/`Tabs` specs emit `<div role="tabpanel">` elements for every tab with `hidden=false` and `aria-hidden=null` regardless of the "selected" tab. The tab-strip visually highlights the selected tab (via the URL `?tab=` query or initial-tab prop) but the panel content is not toggled — the page renders all tab contents stacked vertically.

**Knock-on effect:** Forms from different tabs render in the same DOM tree. The consumer's staff detail page has an Orari-tab form and an Assenze-tab form both visible at once. Direct DOM probes from any URL show all three tabs' fields present, which both confuses the operator and creates cross-form field-name collision risk.

**Two implementation paths:**
- **Server-side:** The server already knows the active tab from the request (`?tab=`). Render only the selected panel; emit the others as empty stubs or skip them. Simplest, no client-side runtime needed.
- **Client-side:** A small IIFE in the runtime (similar to the existing tab-strip click handler that toggles the URL query) sets `hidden=true` on inactive panels at boot. Preserves the multi-panel DOM for SPA-style instant tab switching without a server roundtrip.

Default to client-side for the no-roundtrip ergonomics, but server-side is the cleaner cut if no consumer needs instant tab switching today.

### F4 — `Switch` component does not render

**Symptom:** Specs declaring components of type `Switch` produce no DOM element of any kind. Direct query for `[role=switch]`, `input[type=checkbox]`, or any switch-flavored element returns empty in the affected region.

**Evidence:** The consumer's per-day open/closed toggle in the Orari grid declares `switch_day_N` of type `Switch` with `field: day_N_is_open`. Combined with F1 (which strips the row children entirely), the toggle is doubly absent.

**Decision required:** Same shape as F2 — register natively, document substitution, or both. The natural v2 substitution is a `Checkbox` with a `variant: "switch"` styling prop; the natural ferro-direction substitution is a dedicated `Switch` component since the semantic is distinct from a checkbox (toggle state, not a binary choice).

### F5 — `Input[type=file]` not rendered + `Form.enctype` not propagated

**Symptom:** Two related runtime gaps with the same blast radius:
- `Input` components declaring `input_type: "file"` produce no `<input type=file>` in the DOM. The element is absent rather than mis-rendered.
- `Form` components declaring `enctype: "multipart/form-data"` render with the default browser encoding (`application/x-www-form-urlencoded`). The `enctype` attribute is not emitted on the rendered `<form>` tag.

**Combined effect:** Spec-authored file-upload forms are impossible. A consumer can author the spec, the server can implement the multipart-parsing controller, but the browser will never send a multipart body because there's no file input and the form encoding is wrong.

**Evidence:** The consumer's staff create form declares `field_avatar` of type `Input[input_type=file]` with `accept: "image/jpeg,image/png,image/webp"` and a 5 MB hint. The form declares `enctype: "multipart/form-data"`. The rendered DOM has neither.

**Consumer workaround currently in place:** Branch the controller on `Content-Type` — accept urlencoded for text-only submits, accept multipart for file submits. This works but is a band-aid that pushes complexity into every file-upload controller across every consumer.

**Plan responsibility:** Both pieces (file input + enctype propagation) are landed together because shipping one without the other doesn't unblock anything.

### F6 — DataTable `{row.X}` placeholders not interpolated

**Symptom:** `DataTable` rows declared with per-row form actions of the form `action: "{row.delete_url}"` (the "Approach A" pattern documented in the v12.0 catalog) render the placeholder literally. The browser URL-encodes the curly braces: `/dashboard/staff/%7Brow.delete_url%7D`. Clicking the button POSTs to a non-existent endpoint.

**Evidence:** The consumer's Assenze tab renders a DataTable of time-off rows with per-row delete forms. The form's `action` attribute is the literal string `{row.delete_url}` instead of the row's `delete_url` field value (e.g. `/dashboard/staff/1/assenze/3/elimina`). Direct POST to the correct endpoint confirms the backend delete works; only the UI link is broken.

**Likely cause:** The DataTable component substitutes `{row.X}` placeholders inside cell rendering (column values) but not inside per-row action URLs. The interpolation pass needs to extend to action-URL templates.

**Affected surface:** Every DataTable consumer using Approach A row actions. The Approach B alternative (per-row inline buttons with explicit handlers) works but is verbose and inconsistent across the catalog.

## Slice from Consumer Field Test

Source artifacts in the consumer repo:
- `.planning/phases/151-staff-domain/UI-FRAMEWORK-FINDINGS.md` — full F1–F6 triage with reproductions, workarounds attempted, and recommended real-fixes
- `.planning/phases/151-staff-domain/151-UAT.md` — operator-driven UAT sign-off; UAT-1, UAT-3, UAT-4, UAT-5 PASS; UAT-2 (the load-bearing copy-source-to-N-targets ≤30s gate) is UI-BLOCKED by F1+F2+F3+F4
- `.planning/phases/151-staff-domain/151-06-SUMMARY.md` — closure record for the consumer phase; documents the in-tree mid-execution fixes (`req.param()` Result→Option coercion, urlencoded fallback in the multipart controller)

The consumer phase is mechanically complete. The operator-perceived UI experience is the limit of what the current v12.0 runtime can demonstrate. The findings above are necessary and sufficient to close that limit for the staff-domain surface.

## Decisions Already Made

1. **Scope is exclusively F1–F6.** No new components beyond what those findings require. No architectural moves. No additions to the v2 catalog without an explicit blocked-consumer use case.
2. **One plan per finding.** Each finding is small enough to ship and revert independently. The phase aggregates them for milestone clarity; the plans are independent units.
3. **Land in master, do not push.** Per the consumer-repo convention. Once each plan lands, the user opens a PR (or the GH Actions auto-publish handles it).
4. **No version bump in the patch itself.** ferro-rs version bump happens after merge as part of the publish workflow.

## Decisions Locked (auto-discuss 2026-05-20)

Defaults from the original "Decisions Required at Planning Time" section, resolved with recommended choices so plans can be written without re-asking. Each entry restates the choice space and the locked default.

- **D-F1-depth — `MAX_NESTING_DEPTH = 16`.** Consumer evidence requires at least 8. Phase 164's "5 is enough" underestimated; pick a deliberately generous ceiling so the next field-test doesn't re-trip. 16 leaves room for nested cards inside nested tabs inside a layout shell with margin past today's worst-case spec. Reject any plan that proposes a value below 12 without new consumer evidence.

- **D-F1-diagnostic — split depth-limit from cycle detection.** Two separate diagnostics: `depth limit exceeded at depth N (max=M)` for the depth-limit trip, and `cycle detected: <path>` for actual cycles. The current "cycle guard tripped at depth N" comment is removed. Cycle detection remains in the validator and emits the cycle diagnostic only when it observes a real revisit.

- **D-F2-CheckboxGroup — option (c) both.** Register `CheckboxGroup` as a first-class v2 component with the same semantics as v1 AND document the v2-native substitution path (`Form` with N repeated `Checkbox` children whose `field` ends in `[]`, sharing `name="copy_to[]"` for array submission). Reintroduction unblocks the immediate consumer; the documented substitution preserves compositional simplicity for ad-hoc use.

- **D-F3-tabs — client-side IIFE.** A small IIFE in the runtime (paired with the existing tab-strip click handler that toggles the URL query) sets `hidden=true` on inactive panels at boot and toggles them on tab change without a server roundtrip. Preserves the multi-panel DOM for SPA-style instant tab switching. Server-side conditional render is rejected as the default because consumer ergonomics (no flash, instant switching) outweigh the simpler-cut argument.

- **D-F4-Switch — option (c) both.** Register `Switch` natively as a first-class v2 component with toggle semantics distinct from `Checkbox` (state-flip, not multi-choice) AND document `Checkbox` with `variant: "switch"` as the substitution path. The semantic distinction justifies the dedicated component; the documented alternative covers consumers who do not need the toggle semantic.

- **D-F5 — file input + `enctype` ship together.** `Input[input_type=file]` rendering and `Form.enctype` propagation land in the same plan. Shipping either alone unblocks nothing: a `<form>` without `enctype=multipart/form-data` can't carry a file, and a file input without the enctype on its surrounding form is encoded as `application/x-www-form-urlencoded`. Single plan, two emitter changes, one self-check that submits a multipart body end-to-end.

- **D-F6 — extend interpolation pass to action-URL templates.** The `{row.X}` substitution path is already wired for column-cell rendering inside `DataTable`. Extend the same interpolation pass to per-row action URL templates so `action: "{row.delete_url}"` resolves to the row's `delete_url` field value. Keep the existing column-cell behavior identical; this is an additive pass-extension, not a rewrite.

These locks bind the planner to a single concrete choice per finding. If a plan-time investigation surfaces a reason to deviate, the plan must say so explicitly in its `<decisions>` block and re-validate against the consumer evidence in `.planning/phases/151-staff-domain/UI-FRAMEWORK-FINDINGS.md`.

## Dependencies

- **Phase 161** (v12.0 merge to master) — provides the surface being patched.
- **Phase 164** (batch-3 documenti field-test) — established the precedent for raising `MAX_NESTING_DEPTH` and the bidirectional-adaptation principle for component reintroduction.

## Out of Scope

- The consumer-side urlencoded fallback in the staff create controller. That stays as a defensive layer in the consumer repo even after F5 lands; removing it is a separate consumer-side cleanup.
- v1 catalog re-imports beyond F2 and F4. If a future consumer surfaces a missing v1 component, file it as its own finding rather than expanding Phase 175's scope.
- Plugin model parity (a different Phase 174-era research question).
- HXML / non-WebView protocol direction (Phase 174 research).

## Planning Note

Run `/gsd-plan-phase 175` to break this CONTEXT into six plans (175-01 through 175-06, one per finding). Each plan is small enough to ship as a single commit chain.

Suggested wave order by consumer-blast-radius (high → low):
- **Wave 1 — F1** (depth limit) — unblocks every nested-form consumer
- **Wave 2 — F3** (tabs) — high-traffic across detail pages
- **Wave 3 — F6** (DataTable interpolation) — every per-row delete UX
- **Wave 4 — F2** (CheckboxGroup) — multi-select primitive
- **Wave 4 — F4** (Switch) — boolean-toggle primitive
- **Wave 5 — F5** (file input + enctype) — file-upload surfaces

F2 and F4 can run in the same wave (both are component-registry additions with no overlap in files).

## Acceptance Posture

Each plan in this phase carries its own acceptance criteria. Phase-level acceptance is met when:
- All six findings have a landed fix with a passing self-check
- The consumer staff-domain UAT (`151-UAT.md`) can be re-run end-to-end with all five scenarios PASS at the UI layer, including UAT-2's ≤30s copy-source-to-N-targets gate
- v12.0 COMPLETED.md lists the six patches under a "v12.0.1 — staff-domain field-test runtime patches" section
