# Phase 251: Component variant discipline + interactive-state pass - Context

**Gathered:** 2026-07-03 (auto mode — recommended defaults selected, logged in 251-DISCUSSION-LOG.md)
**Status:** Ready for planning

<domain>
## Phase Boundary

One variant vocabulary across the whole component set: audit all 47 builtin
ferro-json-ui components, normalize prop vocabulary to the canonical `variant`
(primary/secondary/outline/ghost/destructive), `tone`
(neutral/success/warning/destructive), and `size` (sm/md/lg) enums enforced by the
catalog's schemars-generated prop schemas; bring every interactive component to the
component quality bar (hover, `focus-visible` ring from `--color-ring`, disabled
treatment, frequency-tiered motion via the Phase 250 tokens); publish a migration
table of every rename; regenerate `ferro-base.css`. Requirements: DS-03, DS-04.

The `design` module / lint rules are **Phase 252**; MCP surface + docs chapter +
publish are **Phase 253**. No new components, no new token slots.

**Killer feature framing:** this is the phase where "designed, not templated"
becomes true at the component level — one vocabulary an agent can learn once and
apply to all 47 components, and a uniform interactive feel (ring, motion tiers,
disabled) that is architecturally consistent rather than per-component accidental.

</domain>

<decisions>
## Implementation Decisions

### Locked by the anchor spec (do not re-derive)
- **D-01:** Canonical enum values are fixed: `variant` =
  `primary | secondary | outline | ghost | destructive` (visual weight of
  interactive elements); `tone` = `neutral | success | warning | destructive`
  (semantic status color of stateful display components); `size` = `sm | md | lg`.
- **D-02:** Pre-1.0 breaking renames are acceptable; no compatibility shims. A
  migration table lists every renamed prop/value for consumers.
- **D-03:** Interactive-state pass criteria per component: hover treatment present
  and consistent; `focus-visible` ring from `--color-ring`; disabled state
  (opacity + `pointer-events`) consistent; transitions use the motion tokens at
  frequency-appropriate tiers (fast = hover/toggles/controls/nav, base =
  dropdowns/modals/toasts, slow = drawers/page-level reveals). No decorative
  animation; enter ease-out / leave ease-in stays component-internal.
- **D-04:** `ferro-base.css` regenerated (scripts/gen-ferro-base-css.sh) AFTER all
  class changes are in tree; catalog drift guards extend to the canonical enum sets.

### Enum architecture
- **D-05:** Three **shared** enums in `ferro-json-ui/src/component.rs` — `Variant`,
  `Tone`, `Size` — replace the per-component copies (`ButtonVariant`,
  `AlertVariant`, `BadgeVariant`, `ToastVariant`, today's 4-value `Size`). One
  definition point means catalog schemas converge automatically and the drift guard
  checks one source of truth. Per-component enums remain only for genuinely
  component-specific axes that are NOT weight/status/size (e.g. `InputType`,
  `IconPosition`, `Orientation`, `ColumnFormat`).
- **D-06:** `variant` is reserved framework-wide for the canonical weight enum. A
  prop named `variant` whose values are structural rather than weight is **renamed**:
  `CardProps.variant` (`bordered | elevated`) becomes `CardProps.appearance` (same
  values, same enum type renamed `CardAppearance`). The invariant the drift guard
  enforces: *any* prop named `variant`/`tone`/`size` in the catalog schema carries
  exactly the canonical value set — no exceptions.
- **D-07:** `ButtonVariant::Default` → `Variant::Primary` (the serialized value
  `"default"` → `"primary"`; `primary` is the enum default). `ButtonVariant::Link`
  is **removed** — the canonical set has no `link`; consumers migrate `link` →
  `ghost` (migration-table entry). No underline-link button style survives; inline
  links are the Text/anchor components' job.

### Tone adoption (status components)
- **D-08:** Stateful display components rename their status prop `variant` → `tone`
  with the canonical values: Alert, Toast (`info`→`neutral`, `error`→`destructive`,
  `success`/`warning` unchanged), Badge, StatCard, CalendarCell, and any other
  status-colored display component the audit surfaces (spec's list is open-ended).
  Weight (`variant`) and status (`tone`) never share a prop again.
- **D-09:** Badge's mixed set collapses to `tone` only: `default`→`neutral`,
  `secondary`→`neutral`, `outline`→`neutral`, `warning`→`warning`,
  `destructive`→`destructive`. The neutral badge's visual treatment (filled vs
  outlined) is Claude's discretion — pick one and apply it consistently.
- **D-10:** Data-driven variant plumbing follows the rename: DataTable's badge
  column format (row data `{"variant": ...}`), MediaCardGrid's `badge_variant_key`,
  and any similar pass-through key are renamed to `tone`/`badge_tone_key` and their
  accepted values normalized to the canonical tone set. The audit must grep for
  `variant` in compound-component prop plumbing, not just top-level props.

### Size normalization
- **D-11:** `Size` becomes exactly `Sm | Md | Lg` with `Md` the default. Value
  migration: `xs` → `sm`, `default` → `md`. Applies to Button, Avatar,
  SegmentedControl, and any other size-bearing component found in the audit.
- **D-12:** No serde aliases for old values — clean break. Enforcement is
  structural: serde rejects unknown values at spec-parse time and the catalog
  schema advertises only canonical values to agents.

### Interactive-state + motion pass
- **D-13:** Shared class constants for the interactive base (focus-visible ring +
  transition + disabled treatment), composed into each component's class string —
  structural guarantee over 47 hand-copied strings. Today's partially-duplicated
  base strings in `render/atoms.rs` are the consolidation seed.
- **D-14:** Focus ring migrates `focus-visible:ring-primary` →
  `focus-visible:ring-ring` (the Phase 250 `--color-ring` utility, already
  safelisted in `input.css`). Ring width/offset treatment stays uniform (ring-2 +
  offset-2 baseline; Claude's discretion for compact controls).
- **D-15:** Hardcoded durations (`duration-150`, `duration-300`) are replaced by
  the token utilities `duration-fast` / `duration-base` / `duration-slow` +
  `ease-base` per the D-03 frequency tiers. Where a token utility takes over,
  remove the redundant `motion-reduce:transition-none` — Phase 250 deliberately
  collapses durations to 0.01ms (not `none`) so `transitionend` keeps firing;
  keeping both would reintroduce the event-swallowing behavior via a duplicate
  control surface.
- **D-16:** Disabled treatment is uniform: `disabled:opacity-50
  disabled:pointer-events-none` (aria-disabled equivalents where the element is not
  a native control). Hover states must exist on every interactive component; where
  missing today, add the component-appropriate surface hover (`hover:bg-surface`
  family) rather than inventing new colors.

### Migration table + audit surface
- **D-17:** The migration table lives in **public docs** (a "Component vocabulary
  migration" section in the json-ui docs under `docs/src/`), listing every renamed
  prop and value with old → new mapping; the phase summary references it. Consumers
  (gestiscilo Phase 232 reference-case adoption) depend on this table.
- **D-18:** The rename audit must cover every surface that emits or consumes
  component JSON, not just `component.rs`: `render/*.rs`, catalog descriptions
  (`catalog.rs` prose strings mentioning old values), `ferro-projections`
  builder output, `app/` sample specs and tests, `ferro-cli` scaffold templates,
  and `ferro-mcp` `code_templates`/`generation_context` text. Stale old-value
  mentions in agent-facing strings are bugs, held to the same bar as code.

### Drift guard shape
- **D-19:** Extend the catalog drift guard with a schema-walking test: iterate the
  catalog's oneOf component schemas; for every property named `variant`, `tone`, or
  `size`, assert its enum value set equals the canonical set exactly. This makes
  future divergence (a new component with `size: xs`) a compile-visible test
  failure, mirroring the existing 47-count guard at `catalog.rs:1101`.

### Claude's Discretion
- Neutral badge visual treatment (filled vs outline look for `tone: neutral`).
- Ring width/offset on compact controls where ring-2/offset-2 is visually heavy.
- Exact hover classes per component family (within the "surface hover, no new
  colors" direction).
- Which components beyond the spec's named list receive `tone` (audit-driven).
- Whether shared interactive-base constants live in a new module or in
  `render/mod.rs` — planner's call.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design spec (anchor — source of truth for this milestone)
- `docs/superpowers/specs/2026-07-03-json-ui-design-system-design.md` §2 —
  component variant discipline: canonical enums, interactive-state pass, drift
  guards. Also §1 "Design language defaults" — motion tiers table and the
  component quality bar this phase verifies per component.

### Prior phase decisions (tokens this phase consumes)
- `.planning/phases/250-token-vocabulary-v2-default-theme-refresh/250-CONTEXT.md` —
  D-07 (reduced-motion collapses to 0.01ms, informs D-15 here), D-12 (Phase 250
  exposed tokens only; component application is this phase).

### Planning
- `.planning/ROADMAP.md` — v16.5 section, Phase 251 details (goal, success criteria 1–4).
- `.planning/REQUIREMENTS.md` — DS-03, DS-04 (v16.5 section).

### Docs to update
- `docs/src/features/json-ui.md` (or the components reference page the audit
  identifies as canonical) — canonical enum documentation + migration table (D-17).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-json-ui/src/component.rs` — all props structs + today's enums:
  `Size { Xs, Sm, Default, Lg }`, `ButtonVariant { Default, Secondary, Destructive,
  Outline, Ghost, Link }`, `AlertVariant`/`ToastVariant { Info, Success, Warning,
  Error }`, `BadgeVariant { Default, Secondary, Destructive, Warning, Outline }`,
  `CardVariant { Bordered, Elevated }`. No `Tone` enum exists yet.
- `ferro-json-ui/src/render/atoms.rs:137` — existing interactive base string
  (`transition-colors duration-150 … focus-visible:ring-2 focus-visible:ring-primary
  ring-offset-2`) duplicated across buttons/pagination/links/tabs — the
  consolidation seed for D-13.
- `ferro-json-ui/assets/input.css` — Phase 250 utilities live: `duration-fast/base/
  slow`, `ease-base`, `font-display`, `ring-ring` already safelisted via
  `@source inline(...)`; `--color-ring` falls back to `--color-primary` for v1 themes.
- `scripts/gen-ferro-base-css.sh` — regenerates `ferro-base.css` (run after class
  changes, D-04).
- `ferro-json-ui/src/catalog.rs:1101` — 47-count drift guard; the model for the
  D-19 enum-set guard. `render_enum_inline` (catalog.rs:987) renders enum values
  into agent-facing prop docs — canonical sets will surface there automatically.

### Established Patterns
- Enums: `#[serde(rename_all = "snake_case")]` + `strum::AsRefStr` with
  `#[strum(serialize_all = "snake_case")]` where render code needs the string;
  `#[default]` marks the default variant. Follow for `Variant`/`Tone`/`Size`.
- Catalog prop schemas are schemars-generated from the props structs — changing
  the Rust enums IS the schema enforcement; no separate schema editing.
- ferro-mcp mirrors the builtin component count (documented mirror) — count stays
  47 this phase, but grep for mirrored enum/value assertions before the gate.
- Workspace gate (CI-exact): `cargo fmt --all -- --check`, `cargo clippy --all
  --all-targets --all-features -- -D warnings`, `cargo test --all-features`.

### Integration Points
- Renderer dispatch in `ferro-json-ui/src/render/` (atoms/containers/data/form) —
  every match on the old enum variants changes.
- Data-driven variant keys: `render/data.rs` DataTable badge column format
  (row `{"variant": …}`), MediaCardGrid `badge_variant_key` (D-10).
- `ferro-projections` render builder emits component props (Badge variants for
  Process/Browse) — must emit canonical values after the rename.
- `app/` sample application specs/tests and `ferro-cli` scaffold templates consume
  the old vocabulary (D-18 audit surface).
- `ferro-base.css` regen: dynamic class concatenation must be covered by the
  `@source` safelist — new tier utilities are already listed; verify any newly
  emitted classes (e.g. tone classes) survive the Tailwind scan.

</code_context>

<specifics>
## Specific Ideas

- "One word, one meaning": after this phase, `variant` always means weight, `tone`
  always means status, `size` always means sm/md/lg — for every one of the 47
  components, with a drift guard making divergence impossible (structural
  guarantees over one-off fixes).
- The quality bar is checkable, not adjectival: per component, verify hover /
  focus-visible / disabled / motion tier as concrete class assertions where
  practical.
- Motion discipline: the more often an interaction repeats, the less it may move;
  nothing may pop or reflow during a transition (spec §1).
- Visual verification: Chrome MCP screenshots of the sample `app/` before/after
  (light + dark) for the interactive-state pass, per established Phase 250 practice.

</specifics>

<deferred>
## Deferred Ideas

- `Spec.design` field + `design::lint` rule engine + CLI — Phase 252 (by design).
- `design_lint` MCP tool, catalog/generation-context extensions, docs chapter,
  publish — Phase 253. Publish is a single event at Phase 253; do not publish
  mid-milestone (friction-loop release cadence).
- Per-field AX `description` work — Future Direction B (unrelated to this milestone).
- gestiscilo reference-case adoption (consumer-side migration using the D-17
  table) — gestiscilo Phase 232, separate repo.
- No new deferred ideas surfaced during discussion.

</deferred>

---

*Phase: 251-component-variant-discipline-interactive-state-pass*
*Context gathered: 2026-07-03*
