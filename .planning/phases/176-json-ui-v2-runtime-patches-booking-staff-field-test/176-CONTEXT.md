# Phase 176: JSON-UI v2 runtime patches — booking↔staff binding field test findings (F7–F9) — Context

**Gathered:** 2026-05-20
**Status:** Ready for planning
**Source:** Consumer chrome-mcp UAT of v12.0.1 JSON-UI v2 against the gestiscilo-it v6.9 booking↔staff binding dashboard (per-staff filter chip strip, PendingEmail kanban column with countdown badges, staff-member detail widget on booking detail page). Three runtime gaps surfaced; in all three cases the server emits the correct spec + data block, but the renderer silently drops the prop or conditional.

Full triage doc lives in the consumer repo at `.planning/phases/152-booking-staff-binding/152-UI-FINDINGS.md` (Bugs R2/R3/R4). This CONTEXT.md restates the same material in ferro-internal terms and adds the planning shape.

## Phase Boundary

Phase 176 is a v12.0.2 follow-up batch (the next consumer-driven runtime patch series after Phase 175 / v12.0.1). Scope is exclusively the three findings F7–F9 below; no new component surface beyond what the findings explicitly require, and no architectural moves.

Three responsibilities:
1. **Land the three fixes** as individual plans (or two plans if F7+F8 are coupled at the Card template) so they can be shipped, reviewed, and reverted independently.
2. **Re-run the consumer chrome-mcp UAT** against the patched runtime to confirm closure.
3. **Update v12.0 COMPLETED.md** with the patches.

The decision boundary against "ship a new component" is reproduced from Phase 175: F7+F8 extend the existing `Card` component template with two new optional prop slots. F9 fixes the existing `Grid` component's `visible` evaluator (or adds the conditional if it's currently absent on Grid). No new component is registered.

## Findings

### F7 — `Card.badge` prop silently dropped

**Symptom:** Specs declaring `Card { props: { title: "T", description: "D", badge: "B" } }` render DOM containing only `<h3>T</h3>` + `<p>D</p>`. The `badge` slot is silently absent — no warning, no error, no fallback markup.

**Evidence:** Consumer kanban card for a `pending_email` booking. Server spec at `data-view` shows `Card.badge: "Scade tra 9m"`. Rendered DOM has no badge element. Card template at `ferro-json-ui/src/components/card.rs` (or equivalent) presumably has no `{{ badge }}` slot in its render template.

**Consumer use case:** Per-card countdown badges ("Scade tra Nm") for time-sensitive states.

**Decision required:** Add a `badge: Option<String>` slot to `CardProps` + render it inside the card chrome (likely as a small Badge component styled inline, right-aligned with the title or beneath it depending on Card layout convention). Update catalog JSON schema. Add doctest.

### F8 — `Card.subtitle` prop silently dropped

**Symptom:** Specs declaring `Card { props: { title: "T", description: "D", subtitle: "S" } }` render DOM containing only `<h3>T</h3>` + `<p>D</p>`. The `subtitle` slot is silently absent.

**Evidence:** Consumer kanban card showing `staff_name_snapshot` as a secondary identifier beneath the customer name. Server spec has `Card.subtitle: "Marco Rossi"`. Rendered DOM has no subtitle element.

**Consumer use case:** Secondary identifier — staff name, category, or any "second line" beneath the title.

**Decision required:** Add a `subtitle: Option<String>` slot to `CardProps` + render it inside the card chrome (likely as a smaller, muted text below the title, above the description). Update catalog JSON schema. Add doctest.

**Coupling with F7:** F7 and F8 both extend the same `Card` template + catalog entry + JSON schema. Plan-time decision: ship as one combined plan (shared `Card.tpl` edits) or two split plans (separate review surfaces). Default: one combined plan; the two prop slots are visually distinct but render-pipeline-coupled.

### F9 — `Grid.visible` conditional drops entire subtree

**Symptom:** Specs declaring `Grid { children: [...], visible: { path: "/has_staff", operator: "eq", value: true } }` with `data.has_staff = true` render NO Grid element at all — the entire subtree is absent from the DOM. The expected behavior is that `visible: true` renders the Grid + children, and `visible: false` hides it (renders nothing).

**Evidence:** Consumer per-staff filter chip strip — 4 chips (Tutti / Marco / Giulia / Senza staff). Server spec has `staff_chips_row: Grid { children: [chip-0..3], visible: { path: "/has_staff", operator: "eq", value: true } }` and `data.has_staff: true`. Rendered DOM has no Grid element. Other `visible`-bearing elements in the same spec (e.g. `summary_badge: Badge { visible: { path: "/active_count_gt0", ... } }`) render correctly when their path evaluates to true.

**Two possible root causes (planner verifies at plan time):**
- (a) `Grid` component does NOT parse `visible` at all — the field is silently ignored, but the spec validation succeeds, and the Grid is then rendered with `display: none` or omitted from the output entirely depending on the renderer's default behavior. Fix: add `visible` parsing to `GridProps` and honor it.
- (b) `Grid` DOES parse `visible` but the predicate evaluates against the wrong scope (e.g. local Grid scope vs. global data root). Fix: align Grid's `visible` evaluator with the same scope all other components use.

**Consumer use case:** Conditionally rendered chip strips, banners, callouts, or any grouped block that should appear only when a data flag is true.

**Decision required:** Audit which v2 components currently support `visible` and document the union. Grid joining the supported set (if absent) or fixing the evaluator scope (if Grid is supposed to support it). Either way, the test from criterion 3 below is the load-bearing acceptance.

## Code Insights (Reusable Assets)

- `ferro-json-ui/src/components/card.rs` (or equivalent) — Card template; F7+F8 extend it with `badge` and `subtitle` slots.
- `ferro-json-ui/src/components/grid.rs` (or equivalent) — Grid template; F9 audits/extends its `visible` handling.
- `ferro-json-ui/src/spec.rs` — JSON schema authority. CardProps + GridProps both updated here.
- `ferro-json-ui/src/render/` — render pipeline. The "visible" evaluator lives in the conditional-rendering path; F9 traces this for Grid specifically.
- Phase 175 (v12.0.1) precedent — same consumer field-test loop pattern; one plan per finding; default to bidirectional adaptation.

## Established Patterns

- v2 `visible` clause shape: `{ path: "/some/data/path", operator: "eq" | "not_empty" | ..., value: <comparison> }`. Already used by `Badge.visible`, `Card.visible`, `Button.visible` and others. Grid joining (or fixing) follows the existing semantics — no new operator surface.
- Phase 175's "register or document substitution" decision: F7+F8 are PURE extensions (no substitution path makes sense — there's no v2-native way to render a badge inside a Card today). F9 is a fix (visible should work consistently across containers).
- Doctest discipline: every new prop slot ships a passing doctest at the component module + a catalog JSON-schema assertion.

## Specific Ideas

- **F7 visual semantics:** Card.badge rendered as a Badge component instance inside the card chrome, top-right of the title or to the right of the description (depending on card padding). Variant defaults to `secondary` to be visually distinct from the title.
- **F8 visual semantics:** Card.subtitle rendered immediately below the title as `<p class="text-sm text-text-muted">`. Distinct from description (which sits below subtitle). Title → subtitle → description vertical order.
- **F9 verification harness:** ship a small integration test in ferro-json-ui that renders a Grid with `visible: {path: "/flag", operator: "eq", value: true}` against `data: {flag: true}` and asserts the Grid + children are in the rendered output; flip flag to false, assert absent.

## Deferred Ideas

- **F7/F8 click-through on the Card chrome** — neither finding requires Card.badge or Card.subtitle to be interactive. If a future consumer use case needs a clickable badge (e.g., status filter), file as F-future.
- **Visibility on every container component** — F9 only fixes Grid because that's the failing case. A future audit phase could systematically test `visible` on Card, Form, Tabs, Wave, etc. and document the supported union.
- **Catalog JSON-schema migration** — F7+F8 add `badge` and `subtitle` to `Card.props`. Both are optional, so existing specs are unaffected. No `v12.0.3` migration needed.

## Canonical References

- Consumer field test: `gestiscilo-it/.planning/phases/152-booking-staff-binding/152-UI-FINDINGS.md` (Bugs R2, R3, R4)
- Phase 175 precedent: `.planning/phases/175-json-ui-v2-runtime-patches-staff-domain-field-test/175-CONTEXT.md`
- Catalog schema authority: `ferro-json-ui/src/spec.rs` (CardProps, GridProps)
- v2 visible-clause precedent: `ferro-json-ui/src/render/visibility.rs` (or equivalent)

## Folded Todos

None.
