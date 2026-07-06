# Phase 250: Token vocabulary v2 + default theme refresh - Context

**Gathered:** 2026-07-03 (auto mode — recommended defaults selected, logged in 250-DISCUSSION-LOG.md)
**Status:** Ready for planning

<domain>
## Phase Boundary

Grow the fixed ferro-theme vocabulary from 23 to 30 slots — one-knob density
(`--spacing`), frequency-tiered motion (`--motion-duration-fast/base/slow`,
`--motion-ease`), a uniform focus ring (`--color-ring`), and a display font slot
(`--font-display`) — every new slot with a default so existing v1 themes stay valid
unchanged; refresh `ferro-theme/assets/default.css` to the documented design language;
regenerate `ferro-base.css`; update `docs/src/features/themes.md`.

Component-level application of the new tokens (hover/focus/disabled/motion passes
across the 47 components) is **Phase 251**, not this phase. Requirements: DS-01, DS-02.

**Killer feature framing:** a scaffolded ferro app should read as *designed, not
templated* out of the box. This phase lays that foundation — the opinionated defaults
(calm motion tiers, cool-tinted neutrals, single focal color) are what Phase 251's
quality bar and Phase 252's lint subsequently enforce.

</domain>

<decisions>
## Implementation Decisions

### Locked by the anchor spec (do not re-derive)
- **D-01:** The 7 new slots and their defaults are fixed:
  `--spacing` (density knob), `--motion-duration-fast` `120ms`,
  `--motion-duration-base` `220ms`, `--motion-duration-slow` `320ms`,
  `--motion-ease` `cubic-bezier(0.2, 0, 0.38, 0.9)`, `--color-ring`,
  `--font-display` (defaults to `var(--font-sans)`).
- **D-02:** Deliberate exclusions stand: no per-size type tokens (root `font-size`
  in a theme's `tokens.css` is the documented type-scale mechanism), no font-weight
  tokens.
- **D-03:** Every valid v1 theme remains a valid v2 theme with zero changes — an
  unmodified v1 `tokens.css` must render identically before and after this phase.
- **D-04:** `ferro-theme/src/token.rs` doc header moves to `ferro-theme/v2`;
  `ALL_TOKENS` → 30. Update the "23 slots"/"~23" doc comments.

### Default-value delivery mechanism
- **D-05:** New-slot defaults are delivered as `var(--slot, <default>)` fallbacks in
  the `@theme inline` mapping in `ferro-json-ui/assets/input.css`, so the generated
  utilities resolve even when the active theme predates v2 — a structural guarantee
  with no dependency on CSS injection order. `default.css` additionally declares
  explicit values for all 7 slots (light + dark), per the spec.
- **D-06:** Research question for the researcher: `--spacing` is a token Tailwind v4
  itself defines (the base spacing unit). Verify the correct way to map it so themes
  can override at runtime (self-referential `var()` in `@theme inline` vs `@theme`,
  or a distinct internal name) — the requirement is that regenerated spacing utilities
  resolve through a runtime-overridable custom property with `0.25rem` behavior by
  default. Same verification applies to `--font-display` (new utility `font-display`).

### Reduced motion
- **D-07:** `prefers-reduced-motion: reduce` collapses all three durations to
  near-zero (`0.01ms`), not `0ms`/`transition: none` — `transitionend`/`animationend`
  listeners keep firing. Emitted into the generated `ferro-base.css` via `input.css`.

### Default theme refresh (design language)
- **D-08:** Neutral ramp (`background/surface/card/border/text/text-muted`) gains a
  subtle cool tint — low chroma in the hue family of the existing primary (~250) —
  replacing the current zero-chroma flat grey, in **both** light and dark. Dark stays
  dark, not gloomy: refresh tint and contrast, don't crush lightness.
- **D-09:** Single focal color: `--color-accent` (currently a separate cyan, hue 200)
  is harmonized toward the primary hue family so the default theme reads as one
  dominant neutral field with one focal color. The token itself stays (vocabulary is
  fixed) — only its default value changes.
- **D-10:** Radii and shadows are kept structurally as-is (already small, consistent,
  one elevation treatment). Exact oklch values are Claude's discretion, verified
  visually (Chrome MCP screenshots of the sample `app/` before/after, light + dark).
- **D-11:** `default.css` must remain plain CSS — it is injected verbatim into a
  `<style>` tag by `framework/src/json_ui/mod.rs`; no Tailwind at-rules there. All
  Tailwind-specific work happens in `input.css` → regenerated `ferro-base.css`.

### Scope of application in this phase
- **D-12:** This phase only *exposes* the new slots and utilities
  (`duration-fast/base/slow`, `ease-base`, ring color, `font-display`, spacing base).
  No component markup/class changes — applying motion, focus rings, and the display
  font across components is Phase 251's variant-discipline pass. `--spacing` defaults
  to `0.25rem` (Tailwind's native base) so default rendering is pixel-identical.

### Mechanical sequencing
- **D-13:** Regenerate `ferro-base.css` with `scripts/gen-ferro-base-css.sh` AFTER
  the `input.css` / token code changes are in tree (established convention). Check
  for token-count drift guards in tests (grep `ALL_TOKENS` consumers) and update in
  the same change.

### Claude's Discretion
- Exact oklch values for the refreshed neutral ramp and harmonized accent (within D-08/D-09 direction).
- `--color-ring` default value (spec direction: visible contrast; a primary-family ring is the natural pick).
- themes.md prose structure for the v2 reference and type-scaling recipe.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design spec (anchor — the source of truth for this milestone)
- `docs/superpowers/specs/2026-07-03-json-ui-design-system-design.md` §1 — token
  vocabulary v2 table, design-language defaults (motion tiers, default-theme
  direction, component quality bar), exact file-change list for this phase.

### Planning
- `.planning/ROADMAP.md` — v16.5 section, Phase 250 details (goal, success criteria 1–4).
- `.planning/REQUIREMENTS.md` — DS-01, DS-02 (v16.5 section).

### Docs to update
- `docs/src/features/themes.md` — v2 token reference, migration note (none required),
  root-font-size type-scaling recipe.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-theme/src/token.rs` — 23 `TOKEN_*` constants + `ALL_TOKENS` slice; new
  constants follow the same pattern (7 additions → 30).
- `ferro-json-ui/assets/input.css` — `@theme inline` block mapping every slot to
  `var()` references (values injected at runtime by `ThemeMiddleware`); new slots
  extend this block. Also carries the `@source` scan paths and a safelist.
- `ferro-theme/assets/default.css` — 77-line plain-CSS `:root` + dark-mode block; the
  refresh target. Header comment: injected verbatim, MUST NOT contain Tailwind at-rules.
- `scripts/gen-ferro-base-css.sh` — regenerates `ferro-json-ui/assets/ferro-base.css`.

### Established Patterns
- Current neutrals are zero-chroma grey (`oklch(N% 0 0)`); primary is blue
  (`oklch(55% 0.2 250)`), accent a separate cyan (hue 200) — the D-08/D-09 refresh
  baseline.
- Token docs describe the vocabulary as "fixed and versioned"; the version string in
  `token.rs` doc header is the vocabulary version marker (v1 → v2).
- Workspace gate: `cargo fmt --all -- --check`, `cargo clippy --all --all-targets
  --all-features -- -D warnings`, `cargo test --all-features` (CI-exact commands).

### Integration Points
- `framework/src/json_ui/mod.rs` injects `default.css` verbatim — plain-CSS constraint.
- `ThemeMiddleware` (ferro-theme) resolves per-request theme CSS — unchanged surface,
  but the reason defaults must not depend on injection order (D-05).
- `ferro-mcp` mirrors some ferro-json-ui counts (documented mirror) — token-count
  changes may have a mirrored assertion; grep before the gate.

</code_context>

<specifics>
## Specific Ideas

- "A framework default is still a design decision, and an unmade decision reads as a
  template" (spec) — the refresh should be opinionated, not a minimal diff.
- Motion discipline: the more often an interaction repeats, the less it may move;
  nothing may pop or reflow during a transition.
- Success criterion 1 is a hard invariant to test: an unmodified v1 `tokens.css`
  theme renders identically (consider a snapshot/regression check on rendered CSS
  resolution, not just "it compiles").

</specifics>

<deferred>
## Deferred Ideas

- Component-level application of motion/ring/display-font tokens — Phase 251 (by design).
- Canonical variant/tone/size enums — Phase 251.
- `design::lint` + `Spec.design` — Phase 252; MCP surface + docs chapter + publish — Phase 253.
- No new deferred ideas surfaced during discussion.

</deferred>

---

*Phase: 250-token-vocabulary-v2-default-theme-refresh*
*Context gathered: 2026-07-03*
