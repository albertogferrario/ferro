---
phase: 251-component-variant-discipline-interactive-state-pass
reviewed: 2026-07-03T14:35:41Z
depth: standard
files_reviewed: 27
files_reviewed_list:
  - app/src/views/login.json
  - app/src/views/login_confirm.json
  - docs/src/json-ui/actions.md
  - docs/src/json-ui/components.md
  - docs/src/json-ui/forms.md
  - ferro-json-ui/assets/ferro-base.css
  - ferro-json-ui/src/action.rs
  - ferro-json-ui/src/catalog.rs
  - ferro-json-ui/src/component.rs
  - ferro-json-ui/src/layout.rs
  - ferro-json-ui/src/lib.rs
  - ferro-json-ui/src/loader.rs
  - ferro-json-ui/src/projection/builder.rs
  - ferro-json-ui/src/projection/component_map.rs
  - ferro-json-ui/src/render/atoms.rs
  - ferro-json-ui/src/render/classes.rs
  - ferro-json-ui/src/render/containers.rs
  - ferro-json-ui/src/render/data.rs
  - ferro-json-ui/src/render/form.rs
  - ferro-json-ui/src/render/mod.rs
  - ferro-json-ui/src/runtime/mod.rs
  - ferro-json-ui/src/runtime/toasts.rs
  - ferro-json-ui/src/runtime/tabs.rs
  - ferro-mcp/src/tools/code_templates.rs
  - ferro-mcp/src/tools/json_ui_catalog.rs
  - ferro-mcp/src/tools/json_ui_validate_spec.rs
  - framework/src/lib.rs
findings:
  critical: 0
  warning: 5
  info: 6
  total: 11
status: issues_found
---

# Phase 251: Code Review Report

**Reviewed:** 2026-07-03T14:35:41Z
**Depth:** standard
**Files Reviewed:** 27
**Status:** issues_found

## Summary

Phase 251 normalizes 47 builtin components onto canonical `Variant`/`Tone`/`Size` enums, renames Card `variant`→`appearance` and status props →`tone`, and applies an interactive-state pass (shared `INTERACTIVE_BASE`/`FOCUS_RING`/`MOTION_*`/`DISABLED_BASE` fragments in `render/classes.rs`). The core work is well-executed:

- The strum↔serde wire-format guard (`component.rs::strum_tests`) and the D-19 schema-walking drift guard (`catalog.rs::variant_tone_size_enum_sets_drift_guard`) are structurally sound — the walker resolves `$ref`s, covers action-level defs, and asserts non-vacuity (`checked >= 10`).
- Tailwind scanner contract holds: every emitted utility (including `focus-visible:ring-ring`, `focus-visible:ring-inset`, `duration-fast/base/slow`, `ease-base`, `bg-*/70`, `peer-focus:ring-ring/30`) is present in the regenerated `ferro-base.css`; the dynamically built `grid-cols-{n}` / `md:`/`lg:` variants are all safelisted (1–12 verified). Reduced-motion is correctly handled at the token layer (`--motion-duration-*: .01ms` under `prefers-reduced-motion`), justifying the removal of per-class `motion-reduce:transition-none`.
- JS↔SSR lockstep for the toast `data-toast-tone` attribute and `duration-base`/`transitionend` dismissal is verified by runtime tests; `framework/src/lib.rs` re-exports are consistent with the retired enum removals; `ferro-mcp` mirrors (47-count, canonical vocabulary in catalog text) are in sync.

Two substantive gaps remain: (1) renamed props (`Card.variant`, `Badge.variant`, `ConfirmDialog.variant`, …) are **silently ignored** rather than rejected — contradicting the migration doc's "retired values fail at spec parse" and producing silent visual downgrades; (2) the SSR and JS toast background treatments still diverge despite the phase's lockstep goal, and several interactive elements missed the focus-ring/motion pass.

## Warnings

### WR-01: Renamed props are silently ignored on old specs — migration is not fail-at-parse

**File:** `ferro-json-ui/src/component.rs:230` (CardProps.appearance), `:421` (BadgeProps.tone), `:411` (AlertProps.tone), `:822` (ToastProps.tone), `:1315` (ActionCardProps.tone), `:1202` (MediaCardGridProps.badge_tone_key); `ferro-json-ui/src/action.rs:43` (ConfirmDialog.tone); `docs/src/json-ui/components.md:74`
**Issue:** No Props struct uses `#[serde(deny_unknown_fields)]` (verified crate-wide), and the schemars-generated per-component schemas do not set `additionalProperties: false` — so the catalog validator accepts unknown keys too. Retired **values** on surviving prop names correctly fail (`Button.variant: "link"` errors), but retired **prop names** are silently dropped: `{"type": "Badge", "props": {"label": "Paid", "variant": "success"}}` decodes cleanly and renders a *neutral* badge; `Card.variant: "elevated"` silently renders bordered; `confirm: {"title": "...", "variant": "danger"}` silently loses its destructive styling. This contradicts the migration doc's claim "Retired values fail at spec parse — there are no aliases" and turns a compile-visible migration into a silent visual downgrade for every consumer spec using the old prop names.
**Fix:** Add a retired-prop-name lint to `Catalog::validate` Stage 2 (or a dedicated stage), flagging `variant` on Card/Badge/Alert/Toast/ActionCard, `badge_variant_key` on MediaCardGrid, and `variant` inside `confirm`/`on_success.notify` objects:

```rust
// Stage 2b: retired prop names (251 migration) — hard error, not silent ignore.
const RETIRED_PROPS: &[(&str, &str, &str)] = &[
    ("Card", "variant", "appearance"),
    ("Badge", "variant", "tone"),
    ("Alert", "variant", "tone"),
    ("Toast", "variant", "tone"),
    ("ActionCard", "variant", "tone"),
    ("MediaCardGrid", "badge_variant_key", "badge_tone_key"),
];
for (ty, old, new) in RETIRED_PROPS {
    if el.type_name == *ty && el.props.get(old).is_some() {
        errors.push(CatalogError::PropsInvalid { /* "`{old}` was renamed to `{new}`" */ });
    }
}
```

Alternatively add `#[serde(deny_unknown_fields)]` to the renamed structs (heavier: rejects all future-unknown keys), or — minimum viable — correct the docs claim at `components.md:74` to state that renamed prop names are ignored, not rejected.

### WR-02: SSR toast and JS runtime toast tone classes diverge; in-code comment claims they match

**File:** `ferro-json-ui/src/render/atoms.rs:828-838`; `ferro-json-ui/src/runtime/toasts.rs:4-9`
**Issue:** The phase brief calls out JS↔SSR class-literal lockstep for toasts. The `data-toast-tone` attribute and `duration-base`/`transitionend` dismissal were synced, but the background treatment was not: SSR `render_toast` emits translucent `bg-primary/70 … backdrop-blur-md` per tone, while the runtime's `VARIANT_CLASSES` emits solid `bg-primary` / `bg-success` / `bg-warning` / `bg-destructive` with no blur. A `?toast=` URL toast and an SSR `Toast` element with the same tone render visibly differently. The comment block at `atoms.rs:828-830` is self-contradictory — it first claims "Solid, opaque variant backgrounds … Matches the JS showToast() palette so server-rendered and JS-created toasts look identical", then immediately describes the translucent 70%-alpha treatment.
**Fix:** Pick one treatment and share the mapping. E.g. define the four tone-class strings as `pub(crate)` consts next to `MOTION_BASE` in `render/classes.rs`, use them in `render_toast`, and assert lockstep in `runtime/mod.rs` tests (`FERRO_RUNTIME_JS.contains("bg-success/70")` after updating `VARIANT_CLASSES`). At minimum delete the stale "Solid, opaque … look identical" paragraph.

### WR-03: Interactive-state pass stragglers — focusable elements without the token ring / motion tier

**File:** multiple (all sites listed)
**Issue:** The uniform focus-visible ring + motion pass (D-14/D-03/D-16) missed several interactive elements that sit next to elements that did receive it:

- `ferro-json-ui/src/render/containers.rs:163` — Modal trigger button: no focus ring, no motion, and no hover state (also duplicates the primary-button look without `hover:bg-primary/90`). The modal *close* button two lines down got `INTERACTIVE_BASE`.
- `ferro-json-ui/src/render/containers.rs:549` — KanbanBoard mobile tab buttons: no `INTERACTIVE_BASE`, while the equivalent `Tabs` triggers (`containers.rs:262`) were migrated.
- `ferro-json-ui/src/render/containers.rs:627` and `:708` — PageHeader and DetailPage breadcrumb `<a>` links: hover-only, no focus ring; the standalone `Breadcrumb` atom's links carry `INTERACTIVE_BASE` (`atoms.rs:501`).
- `ferro-json-ui/src/render/form.rs:616` — CheckboxList/CheckboxGroup `<input type="checkbox">` lacks `FOCUS_RING`, `MOTION_FAST`, and `DISABLED_BASE`, while the single `Checkbox` input has all three (`form.rs:487`). Disabled CheckboxList options therefore miss the uniform D-16 treatment.
- `ferro-json-ui/src/render/atoms.rs:790-807` — Checklist checkbox inputs and item links: no focus ring.
- `ferro-json-ui/src/render/form.rs:772` — Switch knob uses raw `after:transition-all` with Tailwind's default 150ms duration instead of the `duration-fast ease-base` tier (untokenized motion).

**Fix:** Compose the shared fragments at each site, e.g. for the modal trigger:

```rust
"<button type=\"button\" class=\"inline-flex items-center justify-center rounded-md bg-primary text-primary-foreground px-4 py-2 text-sm font-medium hover:bg-primary/90 {INTERACTIVE_BASE}\" data-modal-open=\"{}\">"
```

and `after:transition-all after:duration-fast after:ease-base` for the Switch knob (add the `after:duration-fast`/`after:ease-base` literals to the safelist/regen CSS if not already emitted).

### WR-04: SSR Toast ignores `dismissible`; `timeout: 0` produces a permanent, undismissable toast

**File:** `ferro-json-ui/src/render/atoms.rs:843-851`; `ferro-json-ui/src/runtime/toasts.rs:89-103`; `ferro-json-ui/src/component.rs:827-828`; `docs/src/json-ui/components.md:1104`
**Issue:** `ToastProps.dismissible` (default `true`, documented as "Allow manual dismiss") is never read by `render_toast` — the SSR path emits no close button and the comment states the `data-toast-timeout` timer "is the only way out". `setupServerToasts` skips any toast whose timeout is `<= 0` or non-numeric (`if (!isFinite(timeout) || timeout <= 0) continue;`). Consequence: a spec that sets `timeout: 0` (a plausible "persistent but dismissible" intent, since `dismissible` defaults true) yields a fixed-position overlay that can never be removed and covers the top-right of the page. Schema/docs advertise behavior the renderer does not implement.
**Fix:** Either honor `dismissible` by emitting the same `[data-toast-close]` button the JS path creates (dismissal wiring already exists in `dismissToast`), or remove `dismissible` from `ToastProps`/docs and clamp `timeout` to a minimum of 1 in `render_toast`.

### WR-05: Component docs describe props that do not exist — documented examples fail decode at render

**File:** `docs/src/json-ui/components.md` (multiple), `docs/src/json-ui/actions.md:110-122`
**Issue:** Several reference entries drifted from the Props structs. Because these docs are the agent-facing authoring surface (and part of the framework quality bar per project instructions), a spec copied from them either fails decode (diagnostic comment, nothing renders) or silently no-ops:

- `components.md:332-350` — Collapsible documents `trigger`/`open`; code requires `title` (+ `expanded`). The example fails decode (required `title` missing).
- `components.md:451-474` — Table documents a `rows` prop that does not exist; required `data_path` is absent from the docs and the example — decode failure.
- `components.md:747` (and example at `:164`) — Form `max_width` documented as `"sm"|"md"|"lg"|"xl"|"full"`; code accepts `default|narrow|wide`. `"max_width": "md"` fails decode and kills the entire `<form>`.
- `components.md:441-444` — DataTable `row_actions` example uses bare Action objects; code requires `DropdownMenuAction` (`{label, action}`) — decode failure.
- `components.md:296` — Modal footer documented as `footer_children`; code slot is `footer` (silently ignored → footer never renders).
- `components.md:1375-1401` — ProductTile documents `title`/`description`/`image_url`/`badge`/`action_label`; code requires `product_id`/`name`/`price`/`field` — decode failure.
- `components.md:1467-1489` — KanbanColumn documents `data_path`/`empty_message` (not in `KanbanColumnProps`) and omits required `id`.
- `components.md:982` — `button_type: "reset"` is documented; `ButtonType` only has `button|submit` — decode failure.
- `components.md:60` — `input_type` list omits `"file"` (supported since the file-input work).
- `actions.md:110-122` — outcome `"type": "reload"` documented; the serde tag is `refresh` — decode failure.

Most of these predate this phase, but the phase touched both files and the migration table promises parse-level strictness the surrounding examples then violate.
**Fix:** Correct the listed entries to match the Props structs. Structural option: add a doc test that extracts each ```json block from `docs/src/json-ui/*.md` and runs element props through `Catalog::validate` so doc drift becomes a test failure (mirrors the D-19 guard philosophy).

## Info

### IN-01: Stale `#[allow(dead_code)]` on `render_action_group`

**File:** `ferro-json-ui/src/render/containers.rs:1032-1034`
**Issue:** Comment says "Dead-code suppressed until plan 02 wires this into the dispatch table", but the dispatch arm exists (`render/mod.rs:216`). The allow is now masking nothing and the comment is misleading.
**Fix:** Remove the attribute and comment.

### IN-02: Stale default in `SegmentedControlProps.size` doc comment

**File:** `ferro-json-ui/src/component.rs:1045`
**Issue:** "Visual size — defaults to `default`." The retired `default` value no longer exists; `Size` defaults to `md`.
**Fix:** Change to "defaults to `md`".

### IN-03: Test still sends retired `variant` prop to Alert — passes only via silent-ignore

**File:** `ferro-json-ui/src/render/atoms.rs:1663-1668`
**Issue:** `alert_emits_message_and_role` builds `Element::new("Alert").prop("message", "OK").prop("variant", "info")`. The prop is silently dropped (see WR-01) so the test passes, but it perpetuates retired vocabulary in-tree and would break if WR-01's strictness fix lands.
**Fix:** Replace with `.prop("tone", "neutral")` or drop the prop.

### IN-04: Stale references to the removed `DropdownMenu` component

**File:** `ferro-json-ui/src/render/atoms.rs:1077-1079,1098`; `ferro-json-ui/src/component.rs:1151`
**Issue:** `DropdownMenu` was replaced by `ActionGroup` (catalog history note, `catalog.rs:1135`), but the atoms.rs section header, `render_menu_item` doc, and `DropdownMenuAction.visible_if` doc still describe a "standalone `DropdownMenu` element".
**Fix:** Reword to reference `ActionGroup` / the inline row-action dropdown.

### IN-05: Button variant→color map duplicated across two files with no lockstep guard

**File:** `ferro-json-ui/src/render/atoms.rs:141-147`; `ferro-json-ui/src/render/containers.rs:974-980`
**Issue:** `render_button_inner` and `button_variant_classes` carry byte-identical five-arm `Variant` color maps. The structural base was deduplicated into `classes.rs` this phase, but the color fragments were not, and no test asserts they stay identical — a future edit to one silently forks Button vs ActionGroup-inline-button styling.
**Fix:** Move a `variant_color_classes(Variant) -> &'static str` into `render/classes.rs` (class strings remain source literals) and call it from both sites.

### IN-06: Docs missing sections for four shipped components; dead anchor

**File:** `docs/src/json-ui/components.md:23-36,1418`
**Issue:** `SegmentedControl`, `SidebarLayout`, `DetailPage`, and `MediaCardGrid` are in `BUILTIN_TYPES` and the MCP catalog but have no section in components.md and are absent from the Component Overview table (StreamText is also missing from the overview). Line 1418 links to `#mediacardgrid`, an anchor that does not exist.
**Fix:** Add the four sections (props tables + examples) and update the overview table.

---

_Reviewed: 2026-07-03T14:35:41Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
