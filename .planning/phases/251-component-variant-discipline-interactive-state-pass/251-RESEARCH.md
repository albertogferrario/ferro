# Phase 251: Component variant discipline + interactive-state pass - Research

**Researched:** 2026-07-03
**Domain:** ferro-json-ui component vocabulary normalization + interactive-state audit (codebase-audit phase, no external stack research needed)
**Confidence:** HIGH (all findings verified by direct codebase inspection this session)

## Summary

This is a codebase-audit phase; every claim below was verified by reading the tree at the current master. The audit found **7 component-level enums** carrying weight/status/size semantics (`Size`, `ButtonVariant`, `AlertVariant`, `BadgeVariant`, `ToastVariant`, `CardVariant`, `ActionCardVariant`) plus **2 action-level enums** (`DialogVariant`, `NotifyVariant`) whose props are also literally named `variant` and which are transitively reachable from component props schemas (ActionGroup → ActionItem → Action) — a scoping decision the planner must make explicit (Open Question 1). The interactive-state inventory found ~30 interactive render sites across `render/atoms.rs`, `render/containers.rs`, `render/data.rs`, `render/form.rs`, **and `layout.rs`** (a file the CONTEXT's audit-surface list omits but which duplicates the interactive base string and has a test pinning `focus-visible:ring-primary` + `duration-150`). The button style table is duplicated verbatim in `containers.rs::button_variant_classes` — the second D-13 consolidation seed alongside `atoms.rs:137`.

Two JS/SSR lockstep contracts constrain the rename: `runtime/toasts.rs` reads `data-toast-variant` and keys `VARIANT_CLASSES` by `info|success|warning|error`, and `runtime/tabs.rs` toggles the exact class strings the SSR emits for tabs. Both must change in the same commit as their render counterparts. On enforcement: schemars 1.x emits enum fields as `$ref: #/$defs/EnumName` (evidence: `docs/protocol/schemas/field-def.json`), and enums with **per-variant doc comments** degrade to `anyOf`-of-`const` (evidence: `intent.json`) — `BadgeVariant` has one today. The D-19 guard must therefore resolve `$ref`s and should prefer canonical enums with container-level docs only, so `$defs/Variant|Tone|Size` stay plain `{"type":"string","enum":[...]}`.

**Primary recommendation:** Structure the work as (1) enum consolidation in `component.rs` + compiler-driven rename ripple (render dispatch, projection emitters, lib.rs/framework re-exports), (2) shared interactive-base constants + per-file class pass including `layout.rs` and the two JS runtimes, (3) D-19 schema-walking guard + test updates + catalog/mcp prose, (4) docs migration table + `ferro-base.css` regen + workspace gate + Chrome MCP visual pass.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Locked by the anchor spec (do not re-derive):**
- **D-01:** Canonical enum values are fixed: `variant` = `primary | secondary | outline | ghost | destructive` (visual weight of interactive elements); `tone` = `neutral | success | warning | destructive` (semantic status color of stateful display components); `size` = `sm | md | lg`.
- **D-02:** Pre-1.0 breaking renames are acceptable; no compatibility shims. A migration table lists every renamed prop/value for consumers.
- **D-03:** Interactive-state pass criteria per component: hover treatment present and consistent; `focus-visible` ring from `--color-ring`; disabled state (opacity + `pointer-events`) consistent; transitions use the motion tokens at frequency-appropriate tiers (fast = hover/toggles/controls/nav, base = dropdowns/modals/toasts, slow = drawers/page-level reveals). No decorative animation; enter ease-out / leave ease-in stays component-internal.
- **D-04:** `ferro-base.css` regenerated (scripts/gen-ferro-base-css.sh) AFTER all class changes are in tree; catalog drift guards extend to the canonical enum sets.

**Enum architecture:**
- **D-05:** Three **shared** enums in `ferro-json-ui/src/component.rs` — `Variant`, `Tone`, `Size` — replace the per-component copies (`ButtonVariant`, `AlertVariant`, `BadgeVariant`, `ToastVariant`, today's 4-value `Size`). One definition point means catalog schemas converge automatically and the drift guard checks one source of truth. Per-component enums remain only for genuinely component-specific axes that are NOT weight/status/size (e.g. `InputType`, `IconPosition`, `Orientation`, `ColumnFormat`).
- **D-06:** `variant` is reserved framework-wide for the canonical weight enum. A prop named `variant` whose values are structural rather than weight is **renamed**: `CardProps.variant` (`bordered | elevated`) becomes `CardProps.appearance` (same values, same enum type renamed `CardAppearance`). The invariant the drift guard enforces: *any* prop named `variant`/`tone`/`size` in the catalog schema carries exactly the canonical value set — no exceptions.
- **D-07:** `ButtonVariant::Default` → `Variant::Primary` (the serialized value `"default"` → `"primary"`; `primary` is the enum default). `ButtonVariant::Link` is **removed** — the canonical set has no `link`; consumers migrate `link` → `ghost` (migration-table entry). No underline-link button style survives; inline links are the Text/anchor components' job.

**Tone adoption (status components):**
- **D-08:** Stateful display components rename their status prop `variant` → `tone` with the canonical values: Alert, Toast (`info`→`neutral`, `error`→`destructive`, `success`/`warning` unchanged), Badge, StatCard, CalendarCell, and any other status-colored display component the audit surfaces (spec's list is open-ended). Weight (`variant`) and status (`tone`) never share a prop again.
- **D-09:** Badge's mixed set collapses to `tone` only: `default`→`neutral`, `secondary`→`neutral`, `outline`→`neutral`, `warning`→`warning`, `destructive`→`destructive`. The neutral badge's visual treatment (filled vs outlined) is Claude's discretion — pick one and apply it consistently.
- **D-10:** Data-driven variant plumbing follows the rename: DataTable's badge column format (row data `{"variant": ...}`), MediaCardGrid's `badge_variant_key`, and any similar pass-through key are renamed to `tone`/`badge_tone_key` and their accepted values normalized to the canonical tone set. The audit must grep for `variant` in compound-component prop plumbing, not just top-level props.

**Size normalization:**
- **D-11:** `Size` becomes exactly `Sm | Md | Lg` with `Md` the default. Value migration: `xs` → `sm`, `default` → `md`. Applies to Button, Avatar, SegmentedControl, and any other size-bearing component found in the audit.
- **D-12:** No serde aliases for old values — clean break. Enforcement is structural: serde rejects unknown values at spec-parse time and the catalog schema advertises only canonical values to agents.

**Interactive-state + motion pass:**
- **D-13:** Shared class constants for the interactive base (focus-visible ring + transition + disabled treatment), composed into each component's class string — structural guarantee over 47 hand-copied strings. Today's partially-duplicated base strings in `render/atoms.rs` are the consolidation seed.
- **D-14:** Focus ring migrates `focus-visible:ring-primary` → `focus-visible:ring-ring` (the Phase 250 `--color-ring` utility, already safelisted in `input.css`). Ring width/offset treatment stays uniform (ring-2 + offset-2 baseline; Claude's discretion for compact controls).
- **D-15:** Hardcoded durations (`duration-150`, `duration-300`) are replaced by the token utilities `duration-fast` / `duration-base` / `duration-slow` + `ease-base` per the D-03 frequency tiers. Where a token utility takes over, remove the redundant `motion-reduce:transition-none` — Phase 250 deliberately collapses durations to 0.01ms (not `none`) so `transitionend` keeps firing; keeping both would reintroduce the event-swallowing behavior via a duplicate control surface.
- **D-16:** Disabled treatment is uniform: `disabled:opacity-50 disabled:pointer-events-none` (aria-disabled equivalents where the element is not a native control). Hover states must exist on every interactive component; where missing today, add the component-appropriate surface hover (`hover:bg-surface` family) rather than inventing new colors.

**Migration table + audit surface:**
- **D-17:** The migration table lives in **public docs** (a "Component vocabulary migration" section in the json-ui docs under `docs/src/`), listing every renamed prop and value with old → new mapping; the phase summary references it. Consumers (gestiscilo Phase 232 reference-case adoption) depend on this table.
- **D-18:** The rename audit must cover every surface that emits or consumes component JSON, not just `component.rs`: `render/*.rs`, catalog descriptions (`catalog.rs` prose strings mentioning old values), `ferro-projections` builder output, `app/` sample specs and tests, `ferro-cli` scaffold templates, and `ferro-mcp` `code_templates`/`generation_context` text. Stale old-value mentions in agent-facing strings are bugs, held to the same bar as code.

**Drift guard shape:**
- **D-19:** Extend the catalog drift guard with a schema-walking test: iterate the catalog's oneOf component schemas; for every property named `variant`, `tone`, or `size`, assert its enum value set equals the canonical set exactly. This makes future divergence (a new component with `size: xs`) a compile-visible test failure, mirroring the existing 47-count guard at `catalog.rs:1101`.

### Claude's Discretion
- Neutral badge visual treatment (filled vs outline look for `tone: neutral`).
- Ring width/offset on compact controls where ring-2/offset-2 is visually heavy.
- Exact hover classes per component family (within the "surface hover, no new colors" direction).
- Which components beyond the spec's named list receive `tone` (audit-driven).
- Whether shared interactive-base constants live in a new module or in `render/mod.rs` — planner's call.

### Deferred Ideas (OUT OF SCOPE)
- `Spec.design` field + `design::lint` rule engine + CLI — Phase 252 (by design).
- `design_lint` MCP tool, catalog/generation-context extensions, docs chapter, publish — Phase 253. Publish is a single event at Phase 253; do not publish mid-milestone (friction-loop release cadence).
- Per-field AX `description` work — Future Direction B (unrelated to this milestone).
- gestiscilo reference-case adoption (consumer-side migration using the D-17 table) — gestiscilo Phase 232, separate repo.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DS-03 | All 47 builtin components use the canonical `variant`/`tone`/`size` enums; catalog prop schemas enforce them; drift guards extend to the enum sets; migration table lists every rename | §Enum/Prop Audit (complete inventory + migration mapping), §Enforcement Mechanics (schemars shapes, D-19 guard design), §Emit-Side Consumers (every surface to update), §Migration Table Skeleton |
| DS-04 | Every interactive component has hover, `focus-visible` (ring from `--color-ring`), and disabled states; transitions use the motion tokens at frequency-appropriate tiers; `ferro-base.css` regenerated after class changes | §Interactive-State Inventory (per-site table with gaps), §Don't Hand-Roll (shared base constants), §Regen + Safelist Mechanics, §JS/SSR Lockstep Contracts |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Workspace gate before every commit (CI-exact): `cargo fmt --all -- --check`, `cargo clippy --all --all-targets --all-features -- -D warnings`, `cargo test --all-features`. CI runs clippy/test with `--all-features`; local runs must match.
- **Update documentation in `docs/src/`** for any framework change (required — and D-17 mandates the migration table there).
- **Update ferro-mcp** when introspectable surfaces change — the mirrored 47-count stays 47, but agent-facing strings (`code_templates`, `json_ui_catalog` ACTION_API prose) mention old values.
- Project-agnostic crates: no hardcoded app identity in `ferro-*` (not triggered by this phase, but reviewers check).
- No co-author lines in commits; prefer editing existing files; keep changes focused.
- `cargo test` regenerates `docs/protocol/schemas/*.json` (Phase 94 export test) — unrelated churn; `git checkout` it, don't fold into phase commits (memory: project_schema_export_test_dirties_tree).
- `cargo test --all-features` recurrently disk-full-fails; check `df` / clean `target/` before the full gate (memory: project_ferro_disk_full_test_gate).
- Serialize CPU-intensive cargo operations; one at a time (memory: feedback_one_cpu_op_at_a_time).
- UI changes: verify with Chrome MCP proactively, before/after screenshots light + dark (CONTEXT Specifics; established Phase 250 practice).

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Canonical enum definitions (`Variant`/`Tone`/`Size`) | ferro-json-ui `component.rs` | re-exports in `ferro-json-ui/src/lib.rs` + `framework/src/lib.rs` | D-05: one definition point; schemas derive from these structs |
| Class-string emission (hover/ring/motion/disabled) | ferro-json-ui `render/*.rs` + `layout.rs` | `runtime/*.rs` (JS class strings) | SSR renderer owns markup; JS runtime must toggle the same literals |
| Schema enforcement | ferro-json-ui `catalog.rs` (schemars-generated) | ferro-mcp `json_ui_catalog` (auto-derives via `global_catalog()`) | Changing the Rust enums IS the schema change; no separate schema editing |
| Drift guards | ferro-json-ui `catalog.rs` tests | ferro-mcp mirrored count test (documented mirror, stays 47) | Single source of truth in ferro-json-ui per established convention |
| Projection-emitted component JSON | ferro-json-ui `src/projection/` (builder.rs, component_map.rs) | — | Projection render builder lives in ferro-json-ui, NOT ferro-projections (verified: ferro-projections has zero variant usage) |
| CSS utilities | `ferro-json-ui/assets/input.css` → generated `ferro-base.css` | `scripts/gen-ferro-base-css.sh` | Phase 250 already shipped all needed utilities; this phase only consumes them |
| Consumer-facing migration docs | `docs/src/json-ui/` | — | D-17: public docs, gestiscilo Phase 232 depends on it |

## Enum/Prop Audit (complete inventory)

### Enums carrying weight/status/size semantics — `ferro-json-ui/src/component.rs`

All verified by direct read. [VERIFIED: codebase grep + read]

| Enum | Line | Values (wire format) | strum AsRefStr | Consumed by (prop sites) | Canonical disposition |
|------|------|----------------------|----------------|--------------------------|----------------------|
| `Size` | :14 | `xs, sm, default, lg` | no | `ButtonProps.size` :289, `AvatarProps.size: Option<Size>` :654, `SegmentedControlProps.size` :1042 | → shared `Size { Sm, Md(default), Lg }` (D-11); `xs`→`sm`, `default`→`md` |
| `ButtonVariant` | :55 | `default, secondary, destructive, outline, ghost, link` | yes | `ButtonProps.variant` :287, `ActionItem.variant: Option<ButtonVariant>` :979 (ActionGroup items) | → shared `Variant { Primary(default), Secondary, Outline, Ghost, Destructive }`; `default`→`primary`, `link` REMOVED→`ghost` (D-07) |
| `AlertVariant` | :90 | `info, success, warning, error` | yes | `AlertProps.variant` :397 | → prop renamed `tone`, shared `Tone`; `info`→`neutral`, `error`→`destructive` (D-08) |
| `BadgeVariant` | :104 | `default, secondary, destructive, warning, outline` | yes | `BadgeProps.variant` :407; `data.rs` DataTable `BadgeCell` :361 | → prop renamed `tone`; `default`/`secondary`/`outline`→`neutral` (D-09) |
| `ToastVariant` | :716 | `info, success, warning, error` | yes | `ToastProps.variant` :818 | → prop renamed `tone`; same mapping as Alert (D-08) |
| `CardVariant` | :188 | `bordered, elevated` | no | `CardProps.variant` :216 | → prop renamed `appearance`, enum renamed `CardAppearance`, values unchanged (D-06) |
| `ActionCardVariant` | :1303 | `default, setup, danger` | no | `ActionCardProps.variant` :1321 | → status-colored ("variant-colored left border") — recommend prop→`tone`: `default`→`neutral`, `setup`→`warning`, `danger`→`destructive`; add a `success` render arm (`border-l-success`). Render evidence: atoms.rs:1271-1273 maps Default→`border-l-primary`, Setup→`border-l-warning`, Danger→`border-l-destructive` |

**Action-level enums with props literally named `variant`** — `ferro-json-ui/src/action.rs`: [VERIFIED: read]

| Enum | Line | Values | Prop site | Note |
|------|------|--------|-----------|------|
| `DialogVariant` | :27 | `default, danger` | `ConfirmDialog.variant` :52 | Set by `Action::confirm()` / `confirm_danger()` builders (:216, :226). Render currently emits only `data-confirm-title`/`data-confirm-message` (atoms.rs:1102-1106) — the variant does NOT reach markup today |
| `NotifyVariant` | :61 | `success(default), info, warning, error` | `ActionOutcome::Notify.variant` :80 | Reachable in the catalog schema via `$defs/Action` | 

These are transitively inside component props schemas: `ActionGroupProps.items: Vec<ActionItem>` → `ActionItem.action: Action` → `ConfirmDialog`/`ActionOutcome`. See Open Question 1.

**Enums verified OUT of scope** (props not named `variant`/`tone`/`size` per D-06's invariant): `GapSize { none, sm, md, lg, xl }` (prop `gap` on Grid :886 / ButtonGroup :957), `FormMaxWidth` (prop `max_width`), `InputType`, `IconPosition`, `Orientation`, `ColumnFormat`, `ColumnAlign`, `TextElement`, `ButtonType`, `FormSectionLayout`, `SortDirection`. [VERIFIED: grep of all `pub enum` + prop fields in component.rs]

**D-08-named components with NO status prop today** (operation is ADD, not rename): `StatCardProps` (:774) has no variant/tone — not interactive, plain metric card. `CalendarCellProps` (:1281) has no variant/tone — its status axes are `closed: bool` and `dot_colors: Vec<String>` (raw Tailwind classes like `"bg-blue-500"` — a pre-existing semantic-token violation, see Open Question 3). See Open Question 2.

### Migration Table Skeleton (old → new, for D-17 docs)

| Component | Old prop | Old value | New prop | New value |
|-----------|----------|-----------|----------|-----------|
| Button, ActionGroup item | `variant` | `default` | `variant` | `primary` |
| Button, ActionGroup item | `variant` | `link` | `variant` | `ghost` (link style removed) |
| Button, ActionGroup item | `variant` | `secondary`/`outline`/`ghost`/`destructive` | `variant` | unchanged |
| Button, Avatar, SegmentedControl | `size` | `xs` | `size` | `sm` |
| Button, Avatar, SegmentedControl | `size` | `default` | `size` | `md` |
| Alert, Toast | `variant` | `info` | `tone` | `neutral` |
| Alert, Toast | `variant` | `error` | `tone` | `destructive` |
| Alert, Toast | `variant` | `success`/`warning` | `tone` | unchanged |
| Badge | `variant` | `default`/`secondary`/`outline` | `tone` | `neutral` |
| Badge | `variant` | `warning`/`destructive` | `tone` | unchanged |
| Card | `variant` | `bordered`/`elevated` | `appearance` | unchanged values |
| ActionCard | `variant` | `default`/`setup`/`danger` | `tone` (recommended) | `neutral`/`warning`/`destructive` |
| DataTable badge column | row data `{"variant": ..}` | BadgeVariant values | row data `{"tone": ..}` | canonical tone values |
| MediaCardGrid | `badge_variant_key` | key naming + `outline`/`destructive`/`default` row values | `badge_tone_key` | canonical tone values |
| ConfirmDialog (if normalized, OQ-1) | `variant` | `default`/`danger` | `tone` | `neutral`/`destructive` |
| Notify outcome (if normalized, OQ-1) | `variant` | `info`/`error` (+`success`/`warning`) | `tone` | `neutral`/`destructive` (+unchanged) |

## Data-Driven Variant Plumbing (D-10 sites)

All sites where `variant` flows through row/item data rather than typed props: [VERIFIED: grep + read]

1. **DataTable badge cells** — `ferro-json-ui/src/render/data.rs:353-374`: `render_cell` deserializes the row value into a local `struct BadgeCell { variant: BadgeVariant, label: String }` and calls `badge_inline_html`. The diagnostic string at :376 also says `expected object {variant, label}`. Doc comment on `ColumnFormat` (component.rs:131-132) documents the `{variant, label}` shape. Test: `data.rs` (`data_table_badge_column_format_renders_pill`) uses `{"status": {"variant": "destructive", "label": "Mancante"}}`.
2. **MediaCardGrid** — `MediaCardGridProps.badge_variant_key` (component.rs:1198, doc comment says `"outline" | "destructive" | "default"`); read at `data.rs:683-688` with `.unwrap_or("outline")` default; string-matched at `data.rs:746-748` (`"destructive"` → destructive classes, everything else → outline look). Test at `data.rs:1422-1423` (`"badge_variant_key": "variant"`). Rename → `badge_tone_key`, values → canonical tone set, default → `neutral`.
3. **KanbanBoard cards** — NO badge-variant plumbing exists. Kanban cards render via `card_title_key`/`card_description_key`; the Card `badge` prop is a plain string styled with fixed "Secondary chrome" (test `render_card_emits_badge_when_present`, containers.rs:1761-1776 — asserts secondary chrome classes, which change when the neutral treatment is picked). No rename needed, but the chrome test updates.
4. **Toast → JS data attribute** — atoms.rs:825-828 maps ToastVariant → the literal strings `info|success|warning|error` emitted as `data-toast-variant`; `runtime/toasts.rs:84` queries `[data-toast-variant]` and `:16-18` keys `VARIANT_CLASSES` by the same strings (`toast.variant || 'info'`), plus a hardcoded `variant: 'success'` at :72. Attribute name, key set, and default must move to `tone`/`neutral` in lockstep with the SSR side.

## Emit-Side Consumers (D-18 audit surface — every file:line)

[VERIFIED: workspace-wide grep]

| Surface | File | Sites |
|---------|------|-------|
| Projection builder | `ferro-json-ui/src/projection/component_map.rs` | `badge_variant_for` :164-169 (FieldMeaning Status→`Default`, Category→`Secondary`, Boolean→`Outline` — **all three collapse to `Tone::Neutral` under D-09**; function likely simplifies to a constant or stays for future tones); `link_button` :340-346 uses **`ButtonVariant::Link` which D-07 removes** → migrate to `Variant::Ghost` (visible behavior change: Link-relationship buttons lose the underline-link look); `Size::default()` :346 |
| Projection builder | `ferro-json-ui/src/projection/builder.rs` | :370 `CardVariant::Bordered` → `CardAppearance::Bordered`; :688 `ActionItem { variant: None }` |
| ferro-projections | `ferro-projections/src/` | **ZERO variant usage — no changes needed** (the render builder lives in ferro-json-ui, contrary to what the phase description assumes) |
| App sample specs | `app/src/views/login.json` (:12 Card `"variant": "elevated"` → `appearance`; :41 Button `"variant": "default"` → `"primary"`), `app/src/views/login_confirm.json` (:12 Card elevated → `appearance`; :26 Button `"variant": "outline"` — prop name unchanged, value unchanged). `pagamenti.json` clean. No app Rust code uses json-ui variant vocabulary (verified: app/src grep hits are unrelated "variant" prose/relation names) |
| ferro-cli | `ferro-cli/src/commands/make_json_view.rs` — clean (no variant/size emission). The only template hits are React/shadcn `.tsx.tpl` files (`Settings.tsx.tpl:208`, `AuthLayout.tsx.tpl`) — that is shadcn/ui's own React `variant` prop for the Inertia frontend scaffold, a **different vocabulary; out of scope** (document as explicitly skipped) |
| ferro-mcp | `code_templates.rs:1095` — json-ui spec template `"variant": "default"` → `"primary"`; `json_ui_validate_spec.rs:107-137` — tests using `Alert props {"variant": "", ...}` and `{"variant": "info"}` → `tone`; `json_ui_catalog.rs:277-279` — ACTION_API prose mentions `variant: NotifyVariant` and `DialogVariant (default|danger)` (update only if OQ-1 normalizes actions); `json_ui_catalog.rs:294-295` — mirrored 47-count + expected-names test (count unchanged, no edit needed); component schemas/descriptions auto-derive from `ferro_json_ui::global_catalog()` (`json_ui_catalog.rs:71-72`) so enum values in schemas update automatically; `generation_context.rs` — **zero variant/tone/size mentions, clean** |
| Catalog prose | `ferro-json-ui/src/catalog.rs` BUILTIN_SPECS descriptions: :134 Button "variant, size" (generic, likely fine); :140 Badge "Small variant-styled label" → tone; :146 Alert "info / success / warning / error variants" → canonical tone wording; :164 Avatar "size variants"; :248 ActionCard "variant-colored border" → tone-colored; :321 FormSection "layout variant" and :358 Input "type variants" use "variant" in the generic sense — reword optional |
| Loader test | `ferro-json-ui/src/loader.rs:314-324` — visibility-gated `Alert {"variant": "", ...}` test spec |
| Re-export lists | `ferro-json-ui/src/lib.rs:47` (action exports: `DialogVariant`, `NotifyVariant`), :49-.. (component exports: `ButtonVariant`, `BadgeVariant`, `AlertVariant`, `ToastVariant`, `CardVariant`, `Size`, ...); `framework/src/lib.rs:87-97` (same names re-exported through the `ferro::` facade) |
| Docs | `docs/src/json-ui/components.md` (1600 lines; hits at :44 size doc `"xs"|"sm"|"default"|"lg"`, :169, :504, :518 Avatar size, :955, :977-979, :1055, :1077, :1290, :1544 + prose); `docs/src/json-ui/actions.md` (:21, :58, :65, :77, :135 — action-level variants, OQ-1 dependent); `docs/src/json-ui/forms.md:138` (`.prop("variant", "error")` builder example); `docs/src/features/json-ui.md` — clean; other json-ui/*.md pages — clean. Note components.md:182 documents GapSize with an `"xs"` value that doesn't exist in the enum (pre-existing doc drift; fix opportunistically) |
| ElementBuilder call sites (stringly-typed `.prop("variant", ...)`) | `render/containers.rs:1718` (`"elevated"` test), `render/atoms.rs:1596` (`"info"` test), `render/atoms.rs:1945` (`"success"` test), `docs/src/json-ui/forms.md:138` |

## Interactive-State Inventory (DS-04)

Per-site audit of every interactive render site. "ring" = `focus-visible:ring-2 focus-visible:ring-primary ring-offset-2` today; all such sites migrate to `ring-ring` (D-14). [VERIFIED: grep + read of all four render files, layout.rs, runtime/, plugins/]

### `render/atoms.rs`

| Site | Line | hover | focus-visible | disabled | motion today | Target tier |
|------|------|-------|---------------|----------|--------------|-------------|
| Button base | :137 | ✓ (per-variant) | ✓ ring-primary | conditional literals ` opacity-50 cursor-not-allowed` + `disabled` attr (:157-166) — NOT D-16's variant classes | `duration-150` + `motion-reduce:transition-none` | fast |
| Breadcrumb links | :493 | ✓ | ✓ | n/a | duration-150 + motion-reduce | fast |
| Pagination links | :542, :559, :569 | ✓ | ✓ | n/a | duration-150 + motion-reduce | fast |
| EmptyState CTA `<a>` | :672-678 | ✓ | **✗ missing** | n/a | `transition-colors` no duration | fast |
| Checklist dismiss button | :753 | ✓ (text only) | **✗ missing** | n/a | **none** | fast |
| Toast container | :834 | n/a | n/a | n/a | `transition-opacity duration-300` | base (`duration-base`) |
| NotificationDropdown trigger | :857 | ✓ | **✗ missing** | n/a | **none** | fast |
| Notification item links | :888 | hover:underline | **✗ missing** | n/a | none | fast |
| Sidebar nav links (active/inactive) | :978, :983 | ✓ | ✓ | n/a | duration-150 + motion-reduce | fast |
| Header logout link | :1053 | ✓ | **✗ missing** | n/a | none | fast |
| CalendarCell (current-month) | :1173 | ✓ | **✗ missing** | n/a | transition-colors no duration | fast |
| ActionCard `<a>`/`<div>` | :1279, :1289 | ✓ | **✗ missing** | n/a | duration-150 (no motion-reduce) | fast |
| ProductTile +/- buttons | :1340, :1344 | ✓ | ✓ | n/a | transition-colors **no duration** | fast |

### `render/containers.rs`

| Site | Line | hover | focus-visible | disabled | motion today | Target tier |
|------|------|-------|---------------|----------|--------------|-------------|
| Modal close button | :180 | ✓ | **✗ missing** | n/a | duration-150 | fast (control) |
| Tabs (active/inactive) | :254-275 | ✓ | ✓ | n/a | duration-150 + motion-reduce | fast |
| Collapsible `<summary>` | :871 | ✓ | **✗ missing** | n/a | chevron `transition-transform` no duration | base |
| ActionGroup inline buttons (`button_variant_classes`) | :949-999 | ✓ | ✓ ring-primary | **✗ none** | duration-150 (no motion-reduce) | fast — **verbatim duplicate of the Button style table; D-13 consolidation target** |
| ActionGroup kebab trigger | :1150-1152 | ✓ | ✓ | n/a | duration-150 | fast |
| ActionGroup menu items | :1174-1175 | ✓ | **✗ missing** | n/a | duration-150 | fast |
| SegmentedControl segments | :1266, :1282 | ✓ | **✗ missing** | n/a | transition-colors no duration | fast |
| SidebarLayout nav links | :1357 | ✓ | **✗ missing** | n/a | transition-colors no duration | fast |

### `render/data.rs`

| Site | Line | hover | focus-visible | disabled | motion today | Target tier |
|------|------|-------|---------------|----------|--------------|-------------|
| Table rows | :82 | ✓ | n/a | n/a | none | fast |
| Table cell links | :97 | ✓ | **✗ missing** | n/a | none | fast |
| DataTable rows (clickable) | :224 | ✓ | n/a (row-level click attrs) | n/a | duration-150 | fast |
| DataTable mobile cards `<a>` | :261 | ✓ | **✗ missing** | n/a | none | fast |
| Row-action kebab trigger | :585 | ✓ | **✗ missing** | n/a | none | fast |
| Dropdown menu items | :593-594 | ✓ | **✗ missing** | n/a | none | fast |

### `render/form.rs`

| Site | Line | focus treatment | disabled | motion today | Notes |
|------|------|-----------------|----------|--------------|-------|
| Input/Textarea | :181-183, :198, :264 | ✓ `focus-visible:ring-primary`; **`focus-visible:ring-destructive` when has_error** | `disabled:opacity-50 disabled:cursor-not-allowed` (cursor, not D-16's pointer-events-none) | duration-150 + motion-reduce | Error ring is semantic (destructive), NOT the ring token — only the non-error ring migrates to `ring-ring`. Tests pin this: :941-942, :1121-1122 |
| Select | :362-364, :376 | ✓ same dual-ring pattern | ✓ same | duration-150 + motion-reduce | |
| Checkbox | :466-468, :480 | ✓ same | ✓ same | duration-150 + motion-reduce | |
| Switch | :661-663, :763 | `peer-focus:ring-primary/30` (peer-focus, not focus-visible; /30 opacity) | — | `after:transition-all` no duration | Tests :1075-1082 pin `peer-focus:ring-destructive/30` on error |
| File input | :228 | error `ring-1 ring-destructive` only | — | none | `hover:file:bg-surface/80` |

### `ferro-json-ui/src/layout.rs` — NOT in the CONTEXT audit list; must be added

| Site | Line | State |
|------|------|-------|
| Layout sidebar nav items (active/inactive) | :154, :159 | Full interactive base duplicated: hover + `focus-visible:ring-primary` + duration-150 + motion-reduce — third copy of the D-13 seed string |
| Sidebar toggle | :242 | hover only, no ring |
| Notification toggles | :259, :268 | hover only, no ring |
| Logout link | :304 | hover only, no ring |
| Nav text states | :459, :492 | hover only |
| **Test pinning old classes** | :1290-1296 | `layout_sidebar_nav_item` asserts `focus-visible:ring-primary` AND `duration-150` (labeled INT-07) — must flip to `ring-ring` + `duration-fast` |

### JS runtimes and plugins

- `runtime/toasts.rs` — `VARIANT_CLASSES` map :4-9 (semantic classes, keys `info|success|warning|error`), `transition-opacity duration-300` :22, dismiss `setTimeout(..., 300)` :~57 coupled to the 300ms class, `data-toast-variant` selector :84, hardcoded `variant: 'success'` :72. `runtime/mod.rs:70` test `variant_classes_use_semantic_tokens`. Migrating the class to `duration-base` (220ms default, theme-overridable) leaves the 300ms JS cleanup timer as a safe upper bound, or switch dismissal to `transitionend` (which the Phase 250 0.01ms mechanism keeps firing). Planner's call; keeping the timer is lower-risk.
- `runtime/tabs.rs:65-70` — JS `classList.add/remove('border-transparent', 'text-text-muted', 'hover:text-text')` toggles the exact literals the SSR emits (containers.rs:254-275, :545). Any change to tab state classes must update both sides in the same task.
- `plugins/map.rs`, `plugins/rich_text_editor.rs` — **zero** hover/transition/ring classes; out of the interactive pass. [VERIFIED: grep]
- `runtime/modals.rs`, `runtime/dropdowns.rs`, `runtime/kanban.rs` — **no transition/opacity manipulation at all**; modals and dropdown popovers currently open/close with no animation. See Open Question 4.

### Hardcoded-class occurrence summary (D-14/D-15 migration set)

- `focus-visible:ring-primary` sites: atoms.rs ×7 (:137, :493, :542, :559, :569, :978, :983), containers.rs ×10 (:261, :275, :974, :979, :984, :989, :994, :999, :1151), form.rs ×3 non-error (:183, :364, :468) + Switch `peer-focus:ring-primary/30` :663, layout.rs ×2 (:154, :159). data.rs ×0.
- `duration-150` sites: atoms.rs ×9, containers.rs ×10, data.rs ×1 (:224), form.rs ×4 (:198, :264, :376, :480), layout.rs ×2. All → `duration-fast` (all are hover/control/nav interactions) + `ease-base`.
- `duration-300` sites: atoms.rs :834 (Toast) and runtime/toasts.rs :22 → `duration-base`.
- `motion-reduce:transition-none` sites to REMOVE where a token utility takes over (D-15): atoms.rs :137, :493, :542, :559, :569, :978, :983; containers.rs :261, :275; form.rs :198, :264, :376, :480; layout.rs :154, :159. [VERIFIED: grep — complete list]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Interactive base classes ×47 | Per-component hand-copied strings (today's state: 3+ duplicates — atoms.rs:137, containers.rs `button_variant_classes` :949-999, layout.rs:154/159) | Shared `const` class fragments (e.g. `INTERACTIVE_BASE`, `FOCUS_RING`, `DISABLED_BASE`, per-tier `MOTION_FAST/BASE`) composed via `concat!` or `format!` at each site, defined once (new `render/classes.rs` module or `render/mod.rs` — Claude's discretion per CONTEXT) | Structural guarantee (D-13); Tailwind scanner still sees literals because `concat!` of literals is a literal; if `format!` is used the fragments themselves remain scannable literals in the same crate source |
| Enum→class mapping | String concatenation `format!("bg-{tone}/10")` | Exhaustive `match` arms returning full literal class strings (today's `badge_inline_html` pattern) | Dynamic concatenation is invisible to the Tailwind `@source` scanner → classes purged from ferro-base.css; match arms also give compile-time exhaustiveness when `Tone` gains a value |
| Enum wire-format sync | Manual string tables | `#[serde(rename_all = "snake_case")]` + `strum::AsRefStr` `#[strum(serialize_all = "snake_case")]` + the existing `variant_enums_strum_matches_serde_wire_format` test pattern (component.rs:1845) extended to `Variant`/`Tone`/`Size` | Established crate convention; test already exists to copy |
| Schema enforcement | Hand-edited JSON schemas | schemars derive on the props structs — changing the Rust enums IS the schema change | Catalog schemas are generated (`schema_for!` per BUILTIN_SPECS entry, catalog.rs:129+) |

## Enforcement Mechanics (D-19 guard design)

### How catalog schemas are generated

- `BUILTIN_SPECS`: static table of `(name, description, schema_fn = schema_for!(XxxProps), slot_fields)` (catalog.rs:129+). `sanitize_schema` (:423) rewrites `definitions`→`$defs` and `#/definitions/X` refs. `assemble_full_schema` (:479) hoists all component-local `$defs` to the root and builds `$defs/Element` as a `oneOf` where each variant is `allOf: [{properties.type.const = name}, {properties: {props: <props schema>, children, action: $ref Action, visible: $ref Visibility}}]`. `per_component_schemas` (used by the markdown prop docs) retain their own local `$defs`. [VERIFIED: read catalog.rs:440-560]
- schemars is v1 (`ferro-json-ui/Cargo.toml:19`, `schemars = { version = "1", features = ["derive"] }`). [VERIFIED]

### schemars 1.x enum schema shapes (critical for the guard walker)

Evidence from committed schemars-1.x output in `docs/protocol/schemas/`: [VERIFIED: read field-def.json, intent.json]

1. Struct field of enum type → `{"$ref": "#/$defs/EnumName"}`; `Option<Enum>` → `{"anyOf": [{"$ref": ...}, {"type": "null"}]}`.
2. Unit-variant enum with **no per-variant doc comments** → `$defs` entry `{"type": "string", "enum": ["a", "b", ...]}` (see `DataType` in field-def.json).
3. Unit-variant enum **with per-variant doc comments** → `anyOf` of `{"type": "string", "const": "x", "description": ...}` entries — NO `enum` array (see intent.json). **`BadgeVariant` has a doc comment on `Warning` today** (component.rs:109-110), so its current schema is likely this shape.

**Recommendation:** give the canonical `Variant`/`Tone`/`Size` enums container-level doc comments only (no per-variant docs) so `$defs/Variant|Tone|Size` are plain `enum` arrays — simpler guard, and `render_field_type`'s inline-enum detection (catalog.rs:923-927) can work. Make the guard's enum-value extractor handle both shapes anyway (`enum` array OR `anyOf[].const`) so it cannot be silently defeated.

### Concrete guard shape

Mirror `builtin_types_count_drift_guard` (catalog.rs:~1090, the 47-count guard — CONTEXT cites :1101; actual test found at the `#[cfg(test)] mod tests` block, count assertion `assert_eq!(crate::render::BUILTIN_TYPES.len(), 47)`):

```rust
// Sketch — Catalog::build_builtins_only() then walk full_schema.
// 1. Assert the three canonical $defs directly:
//    $defs/Variant.enum == ["primary","secondary","outline","ghost","destructive"] (serde decl order)
//    $defs/Tone.enum    == ["neutral","success","warning","destructive"]
//    $defs/Size.enum    == ["sm","md","lg"]
// 2. Walk $defs/Element/oneOf; for each variant take allOf[1].properties.props;
//    recursively walk that subtree RESOLVING $ref against the root $defs
//    (with a visited-set to break cycles — $defs/Element is self-referential
//    via Action? No, but ActionItem→Action→ActionOutcome chains are deep).
//    For every object property named "variant" | "tone" | "size":
//    extract its value set (enum array, or anyOf[].const, following $ref /
//    Option anyOf-null wrappers) and assert equality with the canonical set.
```

**Walker gotchas verified in the schema structure:** `ActionItem` lives in `$defs` (referenced from `ActionGroupProps.items`), so a top-level-properties-only walk MISSES `ActionItem.variant` — the walk must be transitive through `$ref`. Transitive resolution ALSO reaches `$defs/Action` → `ConfirmDialog.variant` + `Notify.variant` (Open Question 1 decides whether those are normalized or the walker skips the `Action`/`Visibility` refs). `Option<Size>` (Avatar) appears as `anyOf [$ref, null]`.

- `render_enum_inline` (catalog.rs:987) inlines enum values into prop docs only when the property schema carries an inline `enum` — `$ref`'d enums currently fall through to `<see schema>` (render_field_type fallback #5, :962). The CONTEXT's claim that "canonical sets will surface there automatically" is only true if `render_field_type` learns to resolve `$ref` against the component schema's local `$defs` — a small, worthwhile addition for the agent surface (flag as recommended task, not required by DS-03 letter).
- ferro-mcp mirror: `json_ui_catalog.rs:294-295` pins the 47 count + expected-names list (unchanged this phase — no component added/removed); component schemas auto-derive from `global_catalog()` so no mirrored enum assertions exist there. [VERIFIED: grep]
- Serde-level enforcement (D-12): render `decode_props` fails on unknown values → HTML-comment diagnostic (renderer is infallible); `Catalog::validate` rejects at spec-load/startup via the assembled JSON Schema; loader startup validation tolerates visibility-gated invalid elements (loader.rs:149).

## Regen + Safelist Mechanics (D-04, pitfall RF-6)

Phase 250 state, verified in tree: [VERIFIED: read input.css, token.rs, ferro-base.css grep]

- `ferro-theme/src/token.rs` has all 7 new tokens (`TOKEN_SPACING` :65, `TOKEN_MOTION_DURATION_FAST/BASE/SLOW` :69-73, `TOKEN_MOTION_EASE` :75, `TOKEN_COLOR_RING` :79, `TOKEN_FONT_DISPLAY` :83); `ALL_TOKENS` = 30.
- `ferro-json-ui/assets/input.css`: `@theme inline` maps `--color-ring: var(--color-ring, var(--color-primary))` and `--ease-base: var(--motion-ease, ...)`; `duration-fast/base/slow` are **custom `@utility` definitions** (Tailwind v4 doesn't resolve `duration-*` from a theme namespace) with built-in fallbacks; reduced-motion collapses durations to `0.01ms !important`. Safelist: `@source inline("duration-fast duration-base duration-slow ease-base font-display ring-ring")` plus grid-cols and font-sans/mono.
- Scanner sources: `@source "../../ferro-json-ui/src"` and `@source "../../framework/src"`. **`layout.rs` and `runtime/*.rs` are inside ferro-json-ui/src → scanned.** Class literals inside Rust string constants (including the JS source strings in runtime/) are picked up.
- `focus-visible:ring-ring` is NOT in the safelist (only bare `ring-ring`) — it will be generated because the literal string appears in render source once the class pass lands. No safelist change needed **as long as every emitted class remains a complete literal in Rust source** (the match-arm convention). If any task introduces `format!("...{}", tone_str)` class construction, that class must be added to `@source inline(...)` or it will be purged — make this an explicit review criterion.
- Regen: `scripts/gen-ferro-base-css.sh` (pinned Tailwind CLI, auto-installs into `.tooling/bin/`, minified output). Run ONCE after all class changes (D-04). Current `ferro-base.css` already contains `ring-ring`/`duration-fast` (Phase 250 regen).
- New classes this phase likely introduces that must appear as literals: `focus-visible:ring-ring`, `duration-fast`, `duration-base`, `ease-base` (safelisted anyway), `disabled:opacity-50`, `disabled:pointer-events-none`, `border-l-success` (new ActionCard success arm), neutral badge/alert/toast tone classes (from existing token families — e.g. `bg-surface`, `text-text`, `border-border` already generated).

## Common Pitfalls

### Pitfall 1: JS/SSR class-string drift
**What goes wrong:** Rename `data-toast-variant`→`data-toast-tone` or retab classes in SSR but not in `runtime/toasts.rs` / `runtime/tabs.rs` → toasts render unstyled/never dismiss, tab switching visually breaks. Tests won't catch it (runtime JS is string constants; only `variant_classes_use_semantic_tokens` checks tokens).
**How to avoid:** Pair each SSR change with its runtime counterpart in the same task; grep `data-toast-variant`, `VARIANT_CLASSES`, and the tab classList literals before closing the task.

### Pitfall 2: Removing `motion-reduce:transition-none` where NO token utility takes over
**What goes wrong:** D-15 says remove `motion-reduce:transition-none` where a token utility takes over. If a site keeps a raw `transition-colors` without a `duration-*` token (browser default 0s — fine) that's harmless, but if a site keeps `duration-150` AND loses `motion-reduce:transition-none`, reduced-motion users get full motion.
**How to avoid:** The invariant is per-site: token duration ⟹ no motion-reduce override (the 0.01ms collapse handles it); raw numeric duration ⟹ must not survive the phase at all. A final `grep -rn "duration-150\|duration-300\|motion-reduce" ferro-json-ui/src framework/src` should return zero hits in render/layout/runtime code.

### Pitfall 3: `disabled:pointer-events-none` vs the anchor-wrapped Button
**What goes wrong:** Button renders inside an `<a>` wrapper for GET actions (atoms.rs:203-246). `disabled:` variants only fire on form controls; a disabled Button inside an anchor still navigates. Today's conditional literal `opacity-50 cursor-not-allowed` + `disabled` attr has the same hole.
**How to avoid:** D-16's aria-equivalent clause: for non-native-control interactive elements, emit `aria-disabled="true"` + literal `pointer-events-none opacity-50` classes conditionally (and skip the anchor wrap when disabled). Planner should make the disabled contract explicit for: Button (native + anchored), ActionGroup items, SegmentedControl segments (anchors).

### Pitfall 4: BadgeVariant's per-variant doc comment shape
**What goes wrong:** Adding per-variant doc comments to the new enums silently changes the schema shape from `enum` array to `anyOf`-of-`const`, breaking a naive D-19 walker and keeping prop docs at `<see schema>`.
**How to avoid:** Container-level docs only on `Variant`/`Tone`/`Size`; walker handles both shapes defensively (see Enforcement Mechanics).

### Pitfall 5: Default-value semantics changes hiding in the rename
**What goes wrong:** Three defaults change meaning: Badge default was primary-tinted (`bg-primary/10 text-primary`), becomes `neutral` (visibly different for every projection-emitted status badge); Toast/Notify default vs Alert default (`info`→`neutral`); MediaCardGrid badge default `"outline"` string → `neutral`. Screenshots will differ; that is intended, but tests asserting the old chrome (e.g. kanban badge "Secondary chrome" test containers.rs:1776) fail for the right reason and must be updated, not worked around.
**How to avoid:** Fold the expected visual deltas into the Chrome MCP before/after review checklist; list them in the migration table so gestiscilo expects the change.

### Pitfall 6: Publish discipline + doc-churn hygiene
**What goes wrong:** Mid-milestone publish (frozen API before Phase 252/253 feedback) or folding `docs/protocol/schemas/*.json` regeneration churn into phase commits.
**How to avoid:** No version bump/publish this phase (Phase 253 owns it). `git checkout docs/protocol/schemas/` after full test runs.

### Pitfall 7: Full-gate environment issues
**What goes wrong:** `cargo test --all-features` disk-full failures (ENOSPC link/fingerprint errors) mistaken for real regressions; local clippy without `--all-features` missing CI failures.
**How to avoid:** `df` check / clean `target/` first; run the CI-exact commands (memory-documented, listed in Project Constraints).

## Test Surface (sizing for the planner)

Sites asserting old enum values / class strings, by file (counts from targeted greps, mechanical updates): [VERIFIED: grep counts]

| File | ~Sites | Nature |
|------|--------|--------|
| `ferro-json-ui/src/component.rs` | 38 | Enum unit tests: strum/serde wire-format test :1845 (extend to new enums), `alert_variant_as_ref_str_matches_wire_format` :1898, `card_variant_tests` module :1907-1966, props default tests |
| `ferro-json-ui/src/render/atoms.rs` | 36 | Class + variant assertions (badge/alert/toast/button), `.prop("variant", "info")` :1596, `.prop("variant", "success")` :1945 |
| `ferro-json-ui/src/render/containers.rs` | 12 | `.prop("variant", "elevated")` :1718, kanban badge secondary-chrome :1761-1776, ActionGroup/tab assertions |
| `ferro-json-ui/src/render/data.rs` | 3 | Badge cell `{"variant": "destructive"}` test, MCG `badge_variant_key` :1422-1423 |
| `ferro-json-ui/src/render/form.rs` | 4 | Ring assertions :941, :1075-1082, :1121 — destructive-ring assertions STAY (semantic error ring); only primary→ring-ring sites flip |
| `ferro-json-ui/src/layout.rs` | 2 | INT-07 test :1290-1296 (`ring-primary` + `duration-150` → `ring-ring` + `duration-fast`) |
| `ferro-json-ui/src/loader.rs` | 1 | `{"variant": "", ...}` gated-Alert test :314-324 |
| `ferro-json-ui/src/projection/*` | 6 | `badge_variant_for` + `link_button` + builder tests |
| `ferro-json-ui/src/action.rs` | ~8 | Only if OQ-1 normalizes action-level fields (DialogVariant/NotifyVariant tests :216-:507) |
| `ferro-json-ui/src/runtime/mod.rs` | 1 | `variant_classes_use_semantic_tokens` :70 |
| `ferro-json-ui/src/catalog.rs` | prose + budget | BUILTIN_SPECS descriptions; check `prompt_under_size_budget` :1716 still passes after prose edits |
| `ferro-mcp` | 4 | `json_ui_validate_spec.rs` :107-137, `code_templates.rs` :1095; 47-count mirror unchanged |
| `app/` | 4 JSON lines | login.json, login_confirm.json |
| `docs/src/json-ui/` | ~20 | components.md, actions.md, forms.md |

Ballpark total: **~120-140 discrete mechanical sites**, compiler-driven for the Rust enum paths (delete old enums → fix every compile error), grep-driven for JSON/prose/docs.

## Architecture Patterns

### System flow this phase touches

```
Spec JSON (agent/app authored)
  └─ serde decode → Props structs (component.rs enums = wire contract)   ← rename here
       └─ render dispatch (render/mod.rs BUILTIN_TYPES → per-file renderers)
            └─ class strings (atoms/containers/data/form/layout)          ← state/motion pass here
                 └─ Tailwind scanner (input.css @source) → ferro-base.css ← regen last
       └─ schemars schema_for!(Props) → catalog full_schema               ← enforcement, auto
            └─ Catalog::validate (spec-load) + json_ui_catalog MCP tool
            └─ D-19 drift-guard test (new)                                ← guard here
  ProjectionBuilder (projection/builder.rs + component_map.rs) ──emits──▶ Spec JSON  ← emitters here
  JS runtime (runtime/*.rs string constants) ◀──lockstep──▶ SSR class/attr contracts
```

### Recommended task ordering (dependency-driven)

1. **Enum consolidation (compiler-driven):** new `Variant`/`Tone`/`Size`/`CardAppearance` in component.rs; delete old enums; fix every compile error across render/, projection/, layout.rs, lib.rs, framework/src/lib.rs, ferro-mcp tests, loader; runtime JS + data-attribute lockstep; app JSON specs. Decide OQ-1 (action-level) here — it changes the same files.
2. **Interactive base constants + class pass (grep-driven):** shared constants; per-file sweep filling the ✗ gaps in the inventory table; ring-ring + duration-token migration; motion-reduce removals; disabled uniformity.
3. **Guards + agent surface:** D-19 schema-walking test; extend strum wire-format test; catalog prose; mcp `code_templates`; optional `render_field_type` `$ref` resolution.
4. **Docs + regen + verification:** migration table in docs/src/json-ui; components.md sweep; `gen-ferro-base-css.sh`; workspace gate; Chrome MCP before/after (light + dark) on the sample app.

Steps 1 and 2 touch the same lines in render files — planner may merge them per-file to avoid double-editing, but the enum rename must land before or with the class pass (class strings for tone arms depend on the new enum names).

## Runtime State Inventory

This is a rename/refactor phase — categories answered explicitly:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — json-ui specs are files in-repo (`app/src/views/*.json`); no DB stores spec JSON in this workspace. Consumer apps (gestiscilo) may store/author specs but that is the separate gestiscilo Phase 232, gated on the Phase 253 publish. Verified: no spec persistence in framework/app beyond view files | none (consumer migration via D-17 table) |
| Live service config | None — no external service holds json-ui vocabulary. Verified: no n8n/dashboards/etc. in this project | none |
| OS-registered state | None — library crates; no scheduled tasks/services carry the vocabulary | none |
| Secrets/env vars | None — no env var names or secret keys reference variant/tone/size. Verified: rename is pure code/docs | none |
| Build artifacts | `ferro-json-ui/assets/ferro-base.css` — generated artifact that goes stale after class changes; regenerated by `scripts/gen-ferro-base-css.sh` (D-04, in-phase). `.tooling/bin/tailwindcss` pinned binary auto-installs. No other stale artifacts | regen ferro-base.css after class pass |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain (cargo/fmt/clippy) | workspace gate | ✓ (project builds daily) | workspace-pinned | — |
| Tailwind v4 CLI | ferro-base.css regen | ✓ auto-installed by `scripts/install-tailwind.sh` into `.tooling/bin/` | pinned by script | — |
| Chrome MCP | visual before/after verification | ✓ (3 instances configured in ~/.claude.json) | — | manual browser check |
| Disk space in `target/` | `cargo test --all-features` | ⚠ recurrent ENOSPC (memory-documented) | — | clean `target/` before full gate |

**Missing dependencies with no fallback:** none.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness via cargo (workspace) |
| Config file | Cargo.toml (workspace); no separate test config |
| Quick run command | `cargo test -p ferro-json-ui` (crate-scoped; covers ~95% of this phase's assertions) |
| Full suite command | `cargo test --all-features` (CI-exact; plus `cargo fmt --all -- --check` and `cargo clippy --all --all-targets --all-features -- -D warnings`) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DS-03 | Canonical enums serialize/deserialize; old values rejected | unit | `cargo test -p ferro-json-ui component::` | ✅ (existing enum test modules in component.rs — updated in place) |
| DS-03 | Every `variant`/`tone`/`size` prop in catalog schema = canonical set | unit (new drift guard) | `cargo test -p ferro-json-ui catalog::tests::` | ❌ Wave 0/1 — new test alongside `builtin_types_count_drift_guard` |
| DS-03 | strum↔serde wire-format agreement for new enums | unit | `cargo test -p ferro-json-ui variant_enums_strum` | ✅ pattern at component.rs:1845 — extend |
| DS-03 | Projection builder emits canonical values | unit | `cargo test -p ferro-json-ui projection::` | ✅ existing tests updated |
| DS-04 | Interactive class assertions (ring-ring, duration tokens, disabled) | unit (render string assertions) | `cargo test -p ferro-json-ui render::` + `layout` | ✅ existing (e.g. layout INT-07, form ring tests) — flip expected strings; add assertions for previously-missing states where practical (CONTEXT: "concrete class assertions where practical") |
| DS-04 | No stale `duration-150`/`duration-300`/`ring-primary`/`motion-reduce:transition-none` in render code | structural grep (verification step, optionally a test) | `grep -rn "duration-150\|duration-300\|focus-visible:ring-primary\|motion-reduce:transition-none" ferro-json-ui/src framework/src` → 0 hits | ❌ verification-step gap |
| DS-04 | ferro-base.css contains the new utilities post-regen | smoke | `grep -c "ring-ring\|duration-fast" ferro-json-ui/assets/ferro-base.css` | ✅ trivially checkable |
| DS-03/04 | Visual parity + intended deltas, light + dark | manual (Chrome MCP) | screenshot sample app before/after | HUMAN/agent-visual — per Phase 250 practice |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-json-ui` (+ `cargo fmt --all -- --check`)
- **Per wave merge:** `cargo clippy --all --all-targets --all-features -- -D warnings` + `cargo test --all-features` (disk check first)
- **Phase gate:** full CI-exact triple green + ferro-base.css regenerated + Chrome MCP visual pass

### Wave 0 Gaps
- [ ] D-19 schema-walking drift-guard test in `ferro-json-ui/src/catalog.rs` tests module (new; model: `builtin_types_count_drift_guard`)
- [ ] Decision on OQ-1 scope (action-level variants) — determines the walker's ref-resolution boundary; must be settled before the guard is written

*(No framework install needed; all other coverage exists and is updated in place.)*

## Security Domain

`security_enforcement` not configured; this phase is a pure rendering/vocabulary refactor with no auth/session/crypto surface. Applicable considerations:

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes (unchanged posture) | serde strict enum deserialization + JSON-Schema catalog validation — D-12 strengthens it (old values now rejected) |
| V2/V3/V4/V6 | no | no auth/session/access/crypto surface in this phase |

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| XSS via rendered labels | Tampering | existing `html_escape` on all user content (e.g. `badge_inline_html` escapes label; wrapper markup is server-controlled) — preserve this property in every touched render site |
| Action leakage via visibility | Elevation | existing fail-closed `visible_if` gates (ActionItem) — untouched; do not weaken while editing ActionItem |

## State of the Art

| Old Approach (in-tree today) | Current Approach (this phase) | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Per-component variant enums (7 copies) | 3 shared canonical enums | Phase 251 | One vocabulary; schema convergence automatic |
| `focus-visible:ring-primary` hardcoded ×22 | `focus-visible:ring-ring` (`--color-ring` token) | Phase 250 token / 251 application | Theme-controllable focus ring |
| `duration-150`/`duration-300` + `motion-reduce:transition-none` | `duration-fast/base` + `ease-base`; reduced-motion via 0.01ms collapse | Phase 250 token / 251 application | Frequency-tiered, theme-controllable, `transitionend`-safe |
| Interactive base string duplicated ×3+ | Shared class constants | Phase 251 (D-13) | Divergence architecturally impossible |
| Count-only drift guard (47) | + schema-walking enum-set guard | Phase 251 (D-19) | `size: xs` regression = test failure |

**Deprecated/outdated by this phase:** `ButtonVariant::Link` (removed; ghost replaces it — projection `link_button` is the only in-tree emitter); the `data-toast-variant` attribute name (if renamed with the prop — lockstep with runtime JS).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | schemars 1.x emits `$ref`/`$defs` + the two enum shapes exactly as observed in the ferro-projections protocol exports (same schemars major version, different crate) | Enforcement Mechanics | Guard walker mis-parses; mitigated by writing the walker against the actual built catalog in a test (self-correcting — the test fails loudly, not silently) |
| A2 | Tailwind v4 `@source inline()` bare `ring-ring` does not generate `focus-visible:ring-ring`; the variant form must appear as a source literal | Regen + Safelist | If wrong (variants auto-generated), harmless — literal-in-source covers it either way |
| A3 | No consumer inside this workspace persists spec JSON in a database | Runtime State Inventory | A missed store would keep old vocabulary at runtime; verified for framework/app by grep, gestiscilo is explicitly out of scope |

All other claims are [VERIFIED] against the tree this session.

## Open Questions

1. **Action-level `variant` props (`ConfirmDialog.variant: DialogVariant`, `Notify.variant: NotifyVariant`) — in or out?**
   - What we know: D-06's invariant says "*any* prop named `variant`/`tone`/`size` in the catalog schema — no exceptions"; D-19's guard walks "component schemas", but ActionGroup's props schema transitively contains the Action schema (`ActionItem.action`), so a correct transitive walker WILL reach both fields. The variants barely reach markup (ConfirmDialog emits only title/message attrs today).
   - What's unclear: whether the phase boundary ("47 builtin components") excludes action schema fields.
   - Recommendation: **normalize them** — `ConfirmDialog.variant`→`tone` (`default`→`neutral`, `danger`→`destructive`), `Notify.variant`→`tone` (`info`→`neutral`, `error`→`destructive`; default stays `success`), reusing the shared `Tone`. This lets the D-19 guard walk the entire `full_schema` with zero exclusions (strongest "one word, one meaning"), costs ~8 extra test-site updates in action.rs + the ACTION_API prose in ferro-mcp + actions.md, and keeps `confirm_danger()` ergonomics unchanged. The weaker alternative (walker skips `$defs/Action`/`$defs/Visibility` refs) leaves two vocabularies and an asterisk in the migration table.

2. **StatCard / CalendarCell `tone`: add new API or document as no-op?**
   - What we know: D-08 and the anchor spec name both under `tone`, but neither has a status prop today (verified) — the operation would be ADD, not rename. StatCard is non-interactive (plain metric card); CalendarCell's status axes are `closed: bool` + `dot_colors: Vec<String>`.
   - Recommendation: add `#[serde(default)] tone: Tone` to StatCard (colors the value text/icon accent; `neutral` = today's look, so zero visual change by default) — cheap and satisfies the spec's naming. For CalendarCell, a cell-level `tone` has no obvious render semantics distinct from `closed`; recommend documenting it as audit-assessed-and-skipped in the plan (Claude's-discretion clause covers this), rather than inventing an unused prop.

3. **CalendarCell `dot_colors` raw Tailwind classes** (`"bg-blue-500"` examples in the doc comment) — a pre-existing violation of "components emit semantic classes exclusively". Out of this phase's scope (not a variant/tone/size prop), but worth a one-line note in the migration docs or a backlog entry; Phase 252's lint could flag it later.

4. **Modal/dropdown enter-leave animation: none exists today.** D-03 assigns "base" tier to dropdowns/modals/toasts, but `runtime/modals.rs`/`dropdowns.rs` do zero animation — there is no existing transition to migrate. Adding new open/close animation is real JS+CSS work and borders on decorative ("no decorative animation"). Recommendation: interpret DS-04 as "existing transitions use tokens + every component meets hover/focus/disabled"; migrate Toast (has a real fade) to `duration-base`; leave modal/dropdown animation as an explicitly-documented non-addition (or a small discretionary popover fade if the planner budgets it). The plan should state the choice either way so verify-work doesn't flag it.

5. **Toast JS dismiss timer vs token duration** — `dismissToast` uses `setTimeout(300)` matched to `duration-300`. With `duration-base` themable (could exceed 300ms), recommendation: switch removal to a `transitionend` listener with a generous fallback timeout (the Phase 250 0.01ms mechanism guarantees the event fires even under reduced motion) — or keep a 500ms fallback timer. Planner's call; either is small.

## Sources

### Primary (HIGH confidence — all verified this session)
- Direct reads/greps of: `ferro-json-ui/src/component.rs`, `action.rs`, `catalog.rs`, `layout.rs`, `loader.rs`, `lib.rs`, `render/{mod,atoms,containers,data,form}.rs`, `runtime/{toasts,tabs,mod}.rs`, `plugins/`, `projection/{builder,component_map}.rs`, `assets/input.css`, `assets/ferro-base.css`
- `ferro-theme/src/token.rs` (Phase 250 tokens in tree), `scripts/gen-ferro-base-css.sh`
- `ferro-mcp/src/tools/{json_ui_catalog,json_ui_validate_spec,code_templates,generation_context}.rs`
- `app/src/views/*.json`, `framework/src/lib.rs`, `ferro-cli/src/{commands,templates}`
- `docs/src/json-ui/*.md`, `docs/src/SUMMARY.md`
- `docs/protocol/schemas/{field-def,intent}.json` — empirical schemars-1.x output shapes
- `.planning/phases/251-.../251-CONTEXT.md`, `250-CONTEXT.md`, `docs/superpowers/specs/2026-07-03-json-ui-design-system-design.md`, `.planning/REQUIREMENTS.md`

### Secondary (MEDIUM confidence)
- Project memory: disk-full test gate, schema-export churn, CI-exact command discipline (documented operational history)

### Tertiary (LOW confidence)
- A2 (Tailwind `@source inline` variant generation behavior) — from training knowledge of Tailwind v4; consequence-free either way (see Assumptions Log)

## Metadata

**Confidence breakdown:**
- Enum/prop audit: HIGH — every enum and prop site read directly with line numbers
- Interactive-state inventory: HIGH — exhaustive grep of hover/focus/disabled/motion classes across all render surfaces, per-site table
- Emit-side consumers: HIGH — workspace-wide grep; ferro-projections confirmed clean; ferro-cli confirmed json-ui-clean
- Enforcement mechanics: HIGH for structure (read), MEDIUM for exact schemars per-enum output (inferred from same-version committed exports; self-correcting via the guard test itself)
- Pitfalls: HIGH — each grounded in a specific in-tree coupling

**Research date:** 2026-07-03
**Valid until:** master moves under this phase's own commits — re-grep line numbers if other phases land in ferro-json-ui first (v16.4 shares no code surface, so drift risk is low)
