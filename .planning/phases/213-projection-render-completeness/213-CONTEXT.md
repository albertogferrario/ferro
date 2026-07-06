# Phase 213: Projection Render Completeness - Context

**Gathered:** 2026-06-12
**Status:** Ready for planning
**Source:** Scoped directly from Phase 209 (COMP-01 Slice A) findings by the orchestrator at the user's direction ("you decide"). Auto-selected sound defaults below; revisit at plan time.

<domain>
## Phase Boundary

Make `JsonUiRenderer`'s projection render **content-complete**. Phase 209 proved the render is layout-complete but content-incomplete: intent derivation and layout selection work, but the content emitters in `ferro-json-ui/src/projection/builder.rs` are partial — several are deliberate placeholders that ignore data already present on the `ServiceDef`. This phase wires that existing `ServiceDef` data (state machine, actions, stat fields, image fields) into the components the projection already selects, so migrating a real view produces a usable page instead of a skeleton.

**Key framing:** the data is already on the `ServiceDef` (`state_machine: Option<StateMachine>`, `actions: Vec<ActionDef>`, the typed fields). The builder's emit functions simply don't read it yet. This is wiring, not new abstraction — it stays inside the existing `Spec`/`Element`/component model and must not touch `intent.rs`/`derive.rs` (intent classification is correct and is frozen by the Phase 207 catalog).

**In scope:** the five gaps from Phase 209's WEAKNESS-NOTE (A–E). **Out of scope:** any change to intent derivation (`derive.rs`/`intent.rs`); new component types beyond what the catalog already defines (unless a gap genuinely requires one); the gestiscilo migration itself (re-verified here, executed in the gestiscilo repo).

**Re-verification target:** the two preserved, unmerged gestiscilo probe branches — `feat/207-orders-projection-migration` (Orders/Process, currently a placeholder kanban) and `feat/208-staff-projection-migration` (Staff/Browse, currently data-bound but action-less). After this phase, Orders should render real columns + cards and Staff should regain row actions.
</domain>

<decisions>
## Implementation Decisions (auto-selected; revisit at plan time)

### Gap A — Process kanban: derive columns from the state machine + bind cards
- **D-01:** `emit_kanban_root` reads `service.state_machine` and emits one `KanbanColumnProps` per state (column `id`/`title` from the state name/display). It sets `KanbanBoardProps.data_path` so cards bind to runtime data grouped by the status field, instead of the current single hard-coded placeholder column with `data_path: None`. When `state_machine` is `None`, keep the existing single-column fallback. `VisualContext.current_state` may mark the active column. (Replaces the "state-machine awareness is a deferred idea" stub.)

### Gap C — Summarize StatCard: bind values
- **D-02:** `emit_statcard_root` binds each stat to runtime data instead of `value: String::new()` — one data-bound `StatCard` per Money/Quantity read-only field (using the JSON-UI `$data` binding convention the components already support), rather than one empty card for the whole service. Confirm `StatCardProps.value` accepts a data-bound expression; if not, the smallest extension to support it is in scope.

### Gap B — actions slot: wire ServiceDef actions (highest leverage — affects every intent)
- **D-03:** `emit_actions_placeholder` emits real action elements from `service.actions` (`Vec<ActionDef>`): page-level actions (e.g. create) as `PageHeader`/`Button` elements, row/card-level actions as the catalog's `DropdownMenu`/`Button`. This single fix lifts every migrated list/detail/kanban out of read-only mode — it is why Browse (Staff) fell short of parity despite rendering data. Prioritized first.

### Gap D — ImageUrl columns
- **D-04:** the Browse `DataTable` column emit renders `FieldMeaning::ImageUrl` fields as an image column (image cell), rather than excluding them. Restores avatar/thumbnail rendering.

### Gap E — app-shell / layout context
- **D-05:** lowest priority, possibly deferred. The projection currently emits a standalone spec with no surrounding chrome. Decide between (a) a layout/slot context on `VisualContext` so a projection renders inside an app shell, or (b) a documented composition pattern where the consumer embeds the projection spec into their layout via `merge`. Default: document the composition pattern now; defer a first-class layout context unless a gap forces it.

### Sequencing & split
- **D-06:** implement in leverage order — **B (actions) → A (kanban) → C (statcard) → D (imageurl) → E (layout)**. The phase MAY split into per-gap sub-phases at plan time; each gap is independently testable. B+A+C together are the substance (they turn Process/Summarize/action-bearing views usable).

### Verification
- **D-07:** every gap gets (1) a `ferro-projections`/`ferro-json-ui` render test asserting the emitted `Spec` contains the bound component (columns from a state machine, non-empty StatCard binding, action elements from `ServiceDef.actions`, image column), and (2) re-verification against the gestiscilo probe branches via the same dev-server + Chrome MCP harness used in Phase 209.
- **D-08:** the Phase 207 catalog `derive_intents` invariants MUST stay green — this phase changes rendering only, never classification.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### The gaps (requirements source)
- `.planning/phases/209-comp-01-slice-a-gestiscilo-migration/WEAKNESS-NOTE.md` — Gaps A–E with root cause; the requirements source for this phase.
- `.planning/phases/209-comp-01-slice-a-gestiscilo-migration/EQUIV-orders-process.md` / `EQUIV-staff-browse.md` — the live evidence + screenshots.

### Code to change (all in ferro-json-ui projection builder)
- `ferro-json-ui/src/projection/builder.rs` — `emit_kanban_root` (Gap A), `emit_statcard_root` (Gap C), `emit_actions_placeholder` (Gap B), `emit_datatable_root` + the column emit (Gap D). **The functions to wire.**
- `ferro-json-ui/src/projection/intent_layout.rs` — the intent→layout slot templates (title/body/actions/stats); read-only reference for which slots feed which emit.
- `ferro-json-ui/src/component.rs` — `KanbanBoardProps`/`KanbanColumnProps`, `StatCardProps`, `DropdownMenu`/`Button`, `Column` shapes the emits must produce.

### Data already on the ServiceDef (read, do not change)
- `ferro-projections/src/service.rs` — `ServiceDef.state_machine: Option<StateMachine>`, `ServiceDef.actions: Vec<ActionDef>`, the typed `fields`.
- `ferro-projections/src/state.rs` — `StateMachine`/`StateDef`/`Transition` (states → columns).
- `ferro-projections/src/render/mod.rs` — `VisualContext` (`intent_index`, `current_state`) the renderer receives.

### Frozen — must not change
- `ferro-projections/src/intent.rs`, `ferro-projections/src/derive.rs`, and `ferro-projections/tests/catalog.rs` invariants (Phase 207 baseline). Rendering changes must not alter intent classification.
</canonical_refs>

<code_context>
## Existing Code Insights

- The render entry the migration used (`JsonUiRenderer.render(&service, &intents, &VisualContext::default())`) routes through the `builder.rs` emit functions per the intent's slot template (`intent_layout.rs`). The fix is local to those emit functions.
- `emit_datatable_root` already proves the pattern: it sets `data_path: /data/{service.name}` and derives columns from `fields`. Gaps A/C/B should follow the same "read ServiceDef → bind to component props" shape that Browse already demonstrates.
- The gestiscilo probe branches already contain working handler migrations; they are the integration test bed — no new consumer code is needed to verify the fix, just re-running them against a rebuilt ferro.
</code_context>

<specifics>
## Specific Ideas

- Gap B (actions) is the highest-leverage single change: it is the difference between "Browse renders data" and "Browse is a usable management page," and it benefits Process/Summarize too. Do it first.
- Re-use the Phase 209 verification harness verbatim: gestiscilo dev server (`ferro serve --backend-only`, port 8080), magic-link dev auto-login (tenant `jetskiadriatic@gestiscilo.it`), Chrome DevTools MCP (`chrome-devtools-3` profile), insert probe rows to test binding, delete after.
</specifics>

<deferred>
## Deferred Ideas

- Gap E (first-class app-shell/layout context) may defer to a follow-up if the composition-pattern documentation (D-05a) suffices.
- A chart/visualization `FieldMeaning` (the gestiscilo Statistics SVG-chart gap, forecast Gap 1) — out of scope; the stat *values* (Gap C) are the target, not server-rendered charts.
- Resuming/merging the gestiscilo Slice A migrations — happens in the gestiscilo repo after this phase ships and re-verification passes.

*Phase: 213-projection-render-completeness*
*Context gathered: 2026-06-12*
</deferred>
