---
phase: 251-component-variant-discipline-interactive-state-pass
fixed_at: 2026-07-03T15:00:26Z
review_path: .planning/phases/251-component-variant-discipline-interactive-state-pass/251-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 251: Code Review Fix Report

**Fixed at:** 2026-07-03T15:00:26Z
**Source review:** .planning/phases/251-component-variant-discipline-interactive-state-pass/251-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 5 (fix_scope: critical_warning — 0 Critical, 5 Warning; 6 Info excluded)
- Fixed: 5
- Skipped: 0

Verification per fix: `cargo fmt --all -- --check` + `cargo test -p ferro-json-ui` (635 passed after final code commit). Final gate: `cargo clippy -p ferro-json-ui --all-targets --all-features -- -D warnings` clean.

## Fixed Issues

### WR-01: Renamed props are silently ignored on old specs

**Files modified:** `ferro-json-ui/src/catalog.rs`, `docs/src/json-ui/components.md`
**Commit:** e1b5c520
**Applied fix:** Added Stage 2b to `Catalog::validate`: a table-driven lint (`RETIRED_PROPS`) rejecting `variant` on Card/Badge/Alert/Toast/ActionCard and `badge_variant_key` on MediaCardGrid as `PropsInvalid` errors naming the replacement prop, plus a recursive walk (`collect_retired_action_variants`) flagging `variant` inside props-embedded `confirm` dialogs and `on_success`/`on_error` notify outcomes (e.g. in `row_actions`). Kept minimal and additive per constraints — no `deny_unknown_fields`, so plugin forward-compat is unaffected. Element-level typed `action` fields (deserialized before validate sees them) remain out of reach — the walk covers all props-embedded actions, which is where row/button actions live. Three new tests (reject retired names, reject confirm/notify variant, accept canonical names). Updated the migration doc claim at `components.md` to state retired values fail at parse and retired prop names fail at catalog validation. Verified no existing views/templates trip the lint (all `"variant"` usages in `app/src/views` and `ferro-mcp` code templates are on Button, which legitimately keeps `variant`).

### WR-02: SSR toast and JS runtime toast tone classes diverge

**Files modified:** `ferro-json-ui/src/render/classes.rs`, `ferro-json-ui/src/render/atoms.rs`, `ferro-json-ui/src/runtime/toasts.rs`, `ferro-json-ui/src/runtime/mod.rs`
**Commit:** e74d2ba6
**Applied fix:** Adopted the translucent treatment (the phase's intended look) on both sides. Added `TOAST_TONE_{NEUTRAL,SUCCESS,WARNING,DESTRUCTIVE}` consts to `render/classes.rs` (single source of truth, full literals for the Tailwind scanner); `render_toast` now uses them and the self-contradictory "Solid, opaque … look identical" comment paragraph is deleted. The JS runtime's `VARIANT_CLASSES` switched to the same `bg-*/70 text-primary-foreground` literals and the JS toast shell gained `backdrop-blur-md` to match SSR. New lockstep test `toast_tone_classes_match_ssr` asserts `FERRO_RUNTIME_JS` contains each const verbatim plus the blur. No CSS regen needed — `bg-*/70` and `backdrop-blur-md` were already in `ferro-base.css`.

### WR-03: Interactive-state pass stragglers

**Files modified:** `ferro-json-ui/src/render/containers.rs`, `ferro-json-ui/src/render/form.rs`, `ferro-json-ui/src/render/atoms.rs`, `ferro-json-ui/assets/ferro-base.css`
**Commit:** 76529cb1
**Applied fix:** All six sites composed with the shared fragments:
- Modal trigger button: `hover:bg-primary/90 {INTERACTIVE_BASE}` added.
- KanbanBoard mobile tab buttons: `{INTERACTIVE_BASE}` added (matches Tabs triggers).
- PageHeader and DetailPage breadcrumb links: `{INTERACTIVE_BASE}` added (both sites, matches the Breadcrumb atom).
- CheckboxList/CheckboxGroup inputs: `{MOTION_FAST} {DISABLED_BASE} {FOCUS_RING}` added (matches the single Checkbox).
- Checklist checkbox inputs (`{MOTION_FAST} {FOCUS_RING}`) and item links (`{INTERACTIVE_BASE}`).
- Switch knob: `after:duration-fast after:ease-base` added to tokenize the knob motion.

**ferro-base.css regenerated** (`scripts/gen-ferro-base-css.sh`) in the same commit for the two new class literals `after:duration-fast` / `after:ease-base`; presence verified in the output. One test updated: `checkbox_list_disabled_propagates_to_each_input` counted `" disabled"` substrings, which now also match the legitimate `disabled:*` DISABLED_BASE class fragments — changed to match the bare ` disabled>` attribute.

### WR-04: SSR Toast ignores `dismissible`; `timeout: 0` produces a permanent toast

**Files modified:** `ferro-json-ui/src/render/atoms.rs`, `ferro-json-ui/src/runtime/toasts.rs`, `ferro-json-ui/src/component.rs`, `docs/src/json-ui/components.md`
**Commit:** 116447ce
**Applied fix:** `render_toast` now honors `dismissible` (default true) by emitting the same `[data-toast-close]` button the JS `showToast()` path creates (identical class literals — all already in `ferro-base.css` via the runtime source, verified). `setupServerToasts` wires the close-button click to `dismissToast` and its stale "the timer is the only way out" comment is rewritten. When `dismissible: false`, the renderer clamps `timeout` to a minimum of 1 second so the auto-dismiss timer is always scheduled — no toast can be permanent and undismissable. `timeout: 0` with `dismissible: true` is now a supported "persistent until manually closed" shape. `ToastProps` field docs and the components.md Toast table updated to state the exact behavior. New test `toast_dismissible_emits_close_button_and_clamps_timeout` covers both branches. Checked ferro-mcp for schema-description snapshots — none reference ToastProps text.

### WR-05: Component docs describe props that do not exist

**Files modified:** `docs/src/json-ui/components.md`, `docs/src/json-ui/actions.md`, `docs/src/json-ui/plugins.md`
**Commit:** 34938db2
**Applied fix:** Every listed entry corrected against the actual Props structs (each edited example cross-checked field-by-field for decode):
- Collapsible: `trigger`/`open` → `title`/`expanded`.
- Table: rewritten as the lightweight data-bound table it is — required `data_path`, no `rows` prop; example split into spec + handler data.
- Form `max_width`: `"default" | "narrow" | "wide"`; all four doc examples fixed (components.md ×3, plugins.md ×1).
- DataTable: props table now matches `DataTableProps` (removed nonexistent `sortable`/`sort_column`/`sort_direction`, added `row_key`/`row_href`); `row_actions` example converted to `DropdownMenuAction` shape (`{label, action}`), and the embedded `confirm` fixed to use required `title` (the old `{"message": ...}` also failed decode).
- Modal: `footer_children` → `footer`; also added the **required `id`** prop to the table and example (the documented example previously failed decode on missing `id`).
- ProductTile: rewritten to the real quantity-control component (`product_id`/`name`/`price`/`field`/`default_quantity`).
- KanbanColumn: reframed as a column object inside `KanbanBoard.columns` (it is not an element type — `{"type": "KanbanColumn"}` fails Stage 1); fields `id`/`title`/`count`/`children` with static-vs-data-bound semantics.
- Button `button_type`: removed `"reset"` (`ButtonType` is `button|submit`).
- `input_type` enum list: added `"file"`.
- actions.md: `"type": "reload"` → `"type": "refresh"` (serde tag of `ActionOutcome::Refresh`), heading renamed.

**Beyond the listed entries, same decode-failure class found while verifying:** `FormProps.action` is a *required props field* (`render_form` reads `props.action`; real views like `app/src/views/login.json` set it in props), but six doc examples placed the action on the element's `"action"` field — those examples failed decode entirely. Fixed all six (components.md ×3, actions.md ×2, plugins.md ×1) and corrected the prose claim "the submit action is set on the element's action field, not in props" to the opposite. The review's structural suggestion (a doc test extracting ```json blocks) was not implemented — noted as follow-up below.

## Observations / Follow-ups (not in scope)

- `docs/src/json-ui/layouts.md:95` has a Form example with a nonexistent `fields` prop and a string-valued `action` — same doc-drift class, but the file was not in the review's scope and needs a larger rewrite.
- The review's structural option for WR-05 (test that runs every docs ```json block through `Catalog::validate`) would make doc drift a test failure; worth a dedicated task.
- Info findings IN-01 through IN-06 were out of fix scope (`critical_warning`). IN-03's test (`alert_emits_message_and_role` sending retired `variant` to Alert) still passes because render-time serde decode remains lenient — the WR-01 lint lives in `Catalog::validate`, so no test breakage.

---

_Fixed: 2026-07-03T15:00:26Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
