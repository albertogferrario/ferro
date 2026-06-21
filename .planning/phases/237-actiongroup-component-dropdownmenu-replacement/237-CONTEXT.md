# Phase 237: ActionGroup Action Primitive + DropdownMenu Replacement - Context

**Gathered:** 2026-06-22
**Status:** Ready for planning
**Mode:** `--auto` (gray areas auto-selected; recommended defaults locked — review before planning)

<domain>
## Phase Boundary

ferro-json-ui gains a first-class `ActionGroup` component that takes **one ordered
action list** and renders inline buttons + a trailing overflow kebab, enforcing the
dashboard action conventions structurally (primary first / destructive in the kebab /
kebab last / `max_inline` cap / non-GET inline wrapped in `<form>`). `ActionGroup`
**replaces** the public `DropdownMenu` component (replace, not wrap). The phase ends
with internal migration (`emit_actions_placeholder`, example/test specs, docs) and an
operator-gated `0.2.71 → 0.2.72` workspace bump + crates.io publish of `ferro-json-ui`
(+ `ferro-rs` re-export).

**In scope:** the component, its props, render path, registration surface (both drift
guards), `DropdownMenu` public removal, projection-codegen migration, internal spec +
docs migration, version bump + publish.

**Out of scope:** any new action *semantics* beyond DropdownMenu parity (no new
confirm-dialog variants, no new Action kinds, no async/optimistic UI); gestiscilo
consumer adoption (separate consumer-repo phase, blocked on published 0.2.72).
</domain>

<decisions>
## Implementation Decisions

### Partition / layout rules
- **D-01:** `destructive: true` items are **always** forced into the overflow kebab and
  rendered **last**, regardless of input order. They do **not** count toward `max_inline`.
- **D-02:** `max_inline` (default `2`) caps the number of **non-destructive** inline
  buttons. Non-destructive items beyond the cap overflow into the kebab in input order.
- **D-03:** The overflow kebab is **hidden entirely** when nothing overflows (≤ `max_inline`
  non-destructive items and zero destructive items). No empty kebab glyph.
- **D-04:** "Primary first" is expressed by **input order** — the first item in `items` is
  the primary/navigational action and renders first inline. No separate `primary: bool`
  flag (avoid a second control surface; order already carries the intent). `variant` on an
  item still controls button styling.

### Prop shape (finalizes the research-seed sketch)
- **D-05:** `ActionItem` fields: `label`, `action: Action`, `destructive: bool` (default
  false), `variant: Option<ButtonVariant>`, `icon: Option<String>`, `visible_if:
  Option<String>` — same `visible_if` fail-closed row semantics as `DropdownMenuAction`
  (absent/falsy field hides the item). Reuse the documented behavior verbatim.
- **D-06:** `ActionGroupProps` fields: `items: Vec<ActionItem>` (or `{"$data":"/path"}`
  binding), `menu_id: String` (**required** — pairs the overflow popover; mirrors
  `DropdownMenuProps.menu_id`, no auto-generation), `max_inline: Option<u8>` (default 2),
  `overflow_label: Option<String>` (aria-label, default `"Azioni"`), `row_key:
  Option<String>`. All optionals `#[serde(default, skip_serializing_if = "Option::is_none")]`.
- **D-07:** Both structs derive `Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema`.
- **D-08:** `items` accepts a literal array **or** `{"$data":"/path"}` binding with
  `{row_key}` substitution and `visible_if` row gates — full parity with today's
  DropdownMenu so server-built kanban/DataTable action arrays flow in unchanged.

### DropdownMenu retention boundary (replace, not delete-everything)
- **D-09:** Remove `DropdownMenu` from the **public** surface only: `BUILTIN_TYPES`,
  `BUILTIN_SPECS`, the `pub use component::{…}` export (`lib.rs`), the dispatch arm, and
  the catalog. No consumer authors a `DropdownMenu` spec after this phase.
- **D-10:** **Keep** `render_menu_item` and the kebab building blocks (kebab glyph SVG +
  popover panel + destructive styling, `render/atoms.rs:1073,1166-1193`) as **internal**
  (`pub(crate)`) render helpers — `ActionGroup`'s overflow path reuses `render_menu_item`'s
  non-GET `<form>` branch and reproduces the kebab trigger/panel HTML from these blocks.
  **Refined (per RESEARCH/checker):** `render_dropdown_menu` (`atoms.rs:1154`) is **NOT**
  reused and becomes **dead code** once the `DropdownMenu` dispatch arm is removed — it is an
  `Element`-based wrapper that `render_action_group` does not call, and `DataTable`/`Kanban`
  rows already render via `render_inline_dropdown` (`data.rs:520`), not `render_dropdown_menu`.
  Per "delete old code completely" + CI `-D warnings`, `render_dropdown_menu` and its atoms.rs
  tests are **deleted** in this phase (Plan 03). The original D-10 intent — keep the kebab
  rendering in one place, not re-invent it — holds; only the specific helper retained changes
  from `render_dropdown_menu` to `render_menu_item` + the building blocks.
- **D-11:** `DropdownMenuAction` / `DropdownMenuProps` structs: `DropdownMenuProps` (the
  public component props) is removed with the public component. `DropdownMenuAction` is
  **retained** as the internal row-action carrier used by `DataTableProps.row_actions` and
  Kanban — renaming it cascades across data.rs/builder.rs for no behavioral gain, so keep
  the name. (Planner may reassess if a clean `ActionItem` reuse is trivial, but default is
  retain.)

### Migration scope (within ferro)
- **D-12:** `emit_actions_placeholder` (`projection/builder.rs:672`) emits an `ActionGroup`
  element instead of `DropdownMenu`. Its existing unit test (`builder.rs:1221+`, currently
  decoding `DropdownMenuProps`) is updated to decode `ActionGroupProps`.
- **D-13:** Migrate **all** ferro-internal / example / test specs that author a
  `DropdownMenu` element to `ActionGroup`. Migrate json-ui docs (Phase 121 doc set) to
  document `ActionGroup` and drop `DropdownMenu`.

### Builtin-count handling
- **D-14:** This is a **one-for-one swap**: add `ActionGroup` (+1), remove public
  `DropdownMenu` (−1) → `BUILTIN_TYPES.len()` **stays 47**. Update the canonical
  `builtin_types_count_drift_guard` (`catalog.rs:1093`) history comment to record the swap
  but keep the asserted number at **47**; the ferro-mcp mirror (`json_ui_catalog.rs:~295`)
  stays 47. The relational guards (`BUILTIN_SPECS.len() == BUILTIN_TYPES.len()`) hold
  automatically. **Verify** the name list in both the catalog `expected` array and the mcp
  `expected` array swaps `DropdownMenu` → `ActionGroup`.

### Form-wrapping non-GET inline actions
- **D-15:** A non-GET inline action auto-wraps in `<form>` (method POST, CSRF token
  included via the existing Button-in-form path) — no bare POST button. A GET action
  renders as a plain link/button. This removes the hand-built `form_toggle_active` sibling
  workaround. Reuse the existing Button form-rendering code (`render/atoms.rs:203` area);
  do not invent a new form emitter.

### Claude's Discretion
- Exact placement of `render_action_group` (research seed suggests `render/containers.rs`
  since ActionGroup resolves slots; planner confirms).
- Whether `ActionGroup` is registered in the containers section vs atoms section of
  `BUILTIN_TYPES` — seed suggests containers; planner decides.
- Internal helper signatures / how `render_action_group` shares the kebab helper.
- Whether `DropdownMenuAction` gets a type alias to a future `ActionItem` (cosmetic).
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase research seed (primary)
- `.planning/research/actiongroup-component.md` — the grounding doc: replace-not-wrap
  decision, prop-shape sketch, full registration-surface checklist (8 touch points),
  migration list, release step. Line numbers are against workspace `0.2.71` — verify
  before editing.

### ferro-json-ui source (touch points)
- `ferro-json-ui/src/component.rs` — props structs live here; `DropdownMenuAction`
  `:1058`, `DropdownMenuProps` `:1074`, `ButtonProps`/`ButtonVariant` `:262`/`:55`,
  `DataTableProps.row_actions` `~:1085`, `PageHeaderProps.actions` `:917`,
  `DetailPageProps` `:1040`, schema-nonempty test pattern `:1510-1515`.
- `ferro-json-ui/src/lib.rs:49-65` — `pub use component::{…}` export block.
- `ferro-json-ui/src/render/mod.rs` — `BUILTIN_TYPES` `:43` (container area `:70`),
  dispatch `match el.type_name` `:176-230`.
- `ferro-json-ui/src/render/atoms.rs` — kebab helpers to reuse: `render_menu_item`
  `:1073`, `render_dropdown_menu` `:1154`, kebab SVG `:1166`, destructive styling
  `:1184-1190`; Button render `:203`.
- `ferro-json-ui/src/render/containers.rs` — `PageHeader` actions slot `:597,609-613,653`,
  `DetailPage` slot `:685`, `ButtonGroup` `:946` (likely home of `render_action_group`).
- `ferro-json-ui/src/render/data.rs` — DataTable/Kanban inline row-action variant `:520`,
  per-row gate `:445/468`.
- `ferro-json-ui/src/catalog.rs` — `BUILTIN_SPECS` `:124`, runtime length drift check
  `:576`, `builtin_types_count_drift_guard` (count = 47) `:1093`.
- `ferro-json-ui/src/action.rs` — `Action` `:153`, `ConfirmDialog` `:46`, `ActionHandler`
  `:90`.
- `ferro-json-ui/src/projection/builder.rs` — `emit_actions_placeholder` `:672`, its test
  `:1221+`, DataTable/Kanban row-action emit `:300/463`.

### ferro-mcp mirror
- `ferro-mcp/src/tools/json_ui_catalog.rs:~288-300` — cross-crate count mirror (47) +
  `expected` name array; swap `DropdownMenu` → `ActionGroup`, keep count 47.

### Docs
- json-ui component docs (Phase 121 doc set, under `docs/src/`) — document `ActionGroup`,
  drop `DropdownMenu`. Planner to locate exact files (`grep -rl DropdownMenu docs/`).

### Process / convention refs
- `CLAUDE.md` — "delete old code completely / no deprecation"; the no-duplicate-control-
  surface principle (informs D-04: order over a `primary` flag); the json-ui-component
  add/remove checklist (BUILTIN_TYPES + dispatch + catalog spec + drift-guard count +
  ferro-mcp count + regenerate `ferro-base.css` via `scripts/gen-ferro-base-css.sh`
  **after** the component code lands).
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `render_dropdown_menu` / `render_menu_item` (`atoms.rs:1073,1154`): the entire kebab
  popover (glyph, anchoring, destructive styling) — ActionGroup's overflow reuses these as
  internal helpers (D-10). No re-implementation of the kebab.
- Button form-rendering path (`atoms.rs:203` area): already wraps actions in `<form>` with
  CSRF for non-GET — D-15 reuses it for inline non-GET items.
- `visible_if` row-gate logic (already implemented for `DropdownMenuAction` in DataTable
  context): ActionItem inherits the identical fail-closed semantics (D-05).
- `{"$data":"/path"}` binding + `{row_key}` substitution machinery already drives
  DataTable/Kanban row actions (`data.rs`) — ActionGroup `items` binding rides the same
  path (D-08).

### Established Patterns
- Adding a json-ui component requires touching **eight** sites in lockstep (props →
  export → BUILTIN_TYPES → dispatch arm → render impl → BUILTIN_SPECS → both drift guards
  → schema-nonempty test), plus the ferro-mcp count mirror and a `ferro-base.css`
  regen. Removing the public DropdownMenu touches the same sites in reverse.
- The absolute builtin count is asserted in exactly **one** place
  (`builtin_types_count_drift_guard`); every other count test is relational. The mcp side
  carries a documented mirror because `BUILTIN_TYPES` is `pub(crate)`.

### Integration Points
- Projection codegen (`emit_actions_placeholder`) is the only generator that emits the
  action menu — switching it to `ActionGroup` is what makes projection-rendered views use
  the new primitive end-to-end (D-12).
- `DataTableProps.row_actions` and Kanban continue to carry `DropdownMenuAction` and render
  via the internal kebab helper — they are *not* re-typed to ActionGroup in this phase
  (D-11); they already render an inline-kebab, which is ActionGroup's overflow behavior.
</code_context>

<specifics>
## Specific Ideas

- Replace, **not** wrap — `ActionGroup` is the sole public action primitive; the kebab
  survives only as an internal helper. (Locked in the research seed.)
- Default `overflow_label` is Italian (`"Azioni"`) per the seed, matching the consumer
  (gestiscilo) dashboard locale. Keep as a default the consumer can override per spec.
- The phase deliberately keeps DropdownMenu's *behavioral parity* exact so the
  gestiscilo server-built kanban action arrays migrate with zero data-shape change.
</specifics>

<deferred>
## Deferred Ideas

- Renaming `DropdownMenuAction` → `ActionItem` across DataTable/Kanban internals — cosmetic,
  cascades widely, no behavioral gain. Out of scope (D-11); revisit only if a future phase
  unifies row-action and ActionGroup item types.
- New action semantics (async/optimistic actions, new confirm-dialog variants, new `Action`
  kinds) — out of scope; this phase is parity + structural enforcement only.
- gestiscilo consumer adoption of `ActionGroup` — separate consumer-repo phase, blocked on
  published `0.2.72` (per cross-repo phase-split convention: ferro publishes, consumer
  adopts in its own tree).

### Reviewed Todos (not folded)
None — no pending todos matched this phase.
</deferred>

---

*Phase: 237-actiongroup-component-dropdownmenu-replacement*
*Context gathered: 2026-06-22*
