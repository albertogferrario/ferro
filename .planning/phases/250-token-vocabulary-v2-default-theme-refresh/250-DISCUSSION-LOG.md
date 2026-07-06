# Phase 250: Token vocabulary v2 + default theme refresh - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-03
**Phase:** 250-token-vocabulary-v2-default-theme-refresh
**Mode:** `--auto` (Claude selected the recommended option for every question; no interactive prompts)
**Areas discussed:** Default-value delivery mechanism, Reduced-motion collapse semantics, Default theme refresh depth & accent policy, New-slot application scope

---

## Default-value delivery mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| `var(--slot, <default>)` fallbacks in `@theme inline` | Structural guarantee that v1 themes resolve values; no injection-order dependency; plus explicit values in default.css per spec | ✓ |
| `:root` defaults block inside generated ferro-base.css | Works but duplicates a value layer inside a generated artifact | |
| Rely on default.css always injecting before theme CSS | Order-dependent; fragile for per-tenant themes | |

**Choice:** `var()` fallbacks in the `@theme inline` mapping (recommended default).
**Notes:** Research flag recorded (D-06): `--spacing` is a slot Tailwind v4 itself owns — verify the self-referential mapping mechanism.

---

## Reduced-motion collapse semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Near-zero (`0.01ms`) | Keeps `transitionend`/`animationend` firing; standard practice | ✓ |
| Exactly `0ms` / `transition: none` | Can silently break completion-event listeners | |

**Choice:** Near-zero collapse via a global `prefers-reduced-motion: reduce` block in the generated CSS (recommended default).

---

## Default theme refresh depth & accent policy

| Option | Description | Selected |
|--------|-------------|----------|
| Cool-tint neutrals + harmonize accent into primary family | Matches spec's "cool-tinted neutrals, single accent used sparingly"; existing cyan accent (hue 200) is a second focal color today | ✓ |
| Colors-only minimal diff, keep cyan accent | Leaves the theme reading as two-accent; conflicts with the design language | |
| Full redesign incl. radii/shadows | Radii/shadows already satisfy "small, consistent, one elevation" — churn without payoff | |

**Choice:** Cool-tinted neutral ramp (low chroma, primary hue family ~250) in both modes; `--color-accent` default value harmonized toward the primary family; radii/shadows kept (recommended default).
**Notes:** Exact oklch values left to Claude's discretion with visual verification (Chrome MCP, light + dark).

---

## New-slot application scope (this phase)

| Option | Description | Selected |
|--------|-------------|----------|
| Expose slots + utilities only | Purely additive; component adoption is Phase 251's variant/state pass — avoids touching 47 components twice | ✓ |
| Apply `--font-display`/motion to headings & components now | Duplicates Phase 251 work; risks breaking SC1's render-identical invariant | |

**Choice:** Expose only; `--spacing` defaults to `0.25rem` so default rendering stays pixel-identical (recommended default).

---

## Claude's Discretion

- Exact oklch values for refreshed neutrals and harmonized accent (within the locked direction).
- `--color-ring` default value (visible contrast; primary-family ring is the natural pick).
- themes.md prose structure for the v2 reference and type-scaling recipe.

## Deferred Ideas

None new — component application (251), variant enums (251), lint (252), MCP/docs/publish (253) are already scoped to their phases.

## Session notes

- Milestone pointer drift found and fixed before discussion: STATE.md pointed at v16.3
  while phases 250–253 live under v16.5 — `phase_found: false` until STATE.md
  `milestone:` moved to v16.5 and the ROADMAP overview gained the 🚧 v16.5 bullet
  (v16.3 flipped to ✅, complete since 2026-06-24).
- `gsd-tools state update milestone` does not update the milestone field — it
  regenerated frontmatter and regressed status/progress fields (reverted via git);
  the pointer was fixed by direct edit instead.
- REQUIREMENTS.md had no v16.5 section (milestone-plan commit ff18e8d5 only touched
  ROADMAP/STATE); DS-01..DS-08 added, derived from the approved anchor spec.
