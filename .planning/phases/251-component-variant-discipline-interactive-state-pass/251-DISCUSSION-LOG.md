# Phase 251: Component variant discipline + interactive-state pass - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-03
**Phase:** 251-component-variant-discipline-interactive-state-pass
**Mode:** `--auto` — all gray areas selected, recommended option chosen for each question
**Areas discussed:** Enum architecture, Tone adoption, Size normalization, Interactive-state/motion strategy, Migration table location, Drift-guard shape

---

## Enum architecture

| Option | Description | Selected |
|--------|-------------|----------|
| Shared canonical enums | One `Variant`, one `Tone`, one `Size` in component.rs; per-component copies deleted | ✓ |
| Per-component enums, same values | Keep `ButtonVariant` etc. but normalize values | |

**User's choice:** Shared enums (auto — recommended: single source of truth; catalog schemas converge automatically; drift guard checks one definition; per-component copies allow silent re-divergence)

| Option | Description | Selected |
|--------|-------------|----------|
| Remove `Link`, migrate to `ghost` | Canonical 5-value set stays fixed; migration table entry | ✓ |
| Keep `Link` as Button-only extension | Preserves style, breaks "one vocabulary" invariant | |

**User's choice:** Remove `Link` (auto — recommended: the spec fixes the canonical set at 5 values; a Button-only extra value would make the drift guard need exceptions)

| Option | Description | Selected |
|--------|-------------|----------|
| Rename Card `variant` → `appearance` | `variant` reserved framework-wide for the weight enum | ✓ |
| Leave Card `variant` (bordered/elevated) | Card isn't interactive, spec's variant is for interactive elements | |

**User's choice:** Rename to `appearance` (auto — recommended: keeps the invariant absolute — any prop named `variant` is the canonical enum — which is what makes the drift guard simple and the vocabulary learnable)

---

## Tone adoption (status components)

| Option | Description | Selected |
|--------|-------------|----------|
| Rename prop `variant` → `tone` | Weight and status become distinct axes with distinct names | ✓ |
| Keep prop name `variant`, change values | Less churn but weight/status stay conflated | |

**User's choice:** Rename to `tone` (auto — recommended: the spec defines tone as its own axis; Alert/Toast/Badge/StatCard/CalendarCell + audit-found components; `info`→`neutral`, `error`→`destructive`)

| Option | Description | Selected |
|--------|-------------|----------|
| Badge: `tone` only | `default`/`secondary`/`outline` all collapse to `neutral` | ✓ |
| Badge: both `variant` and `tone` | Preserves outline/secondary looks as weight axis | |

**User's choice:** Tone only (auto — recommended: spec lists Badge under tone components without a weight axis; neutral visual treatment left to Claude's discretion)

---

## Size normalization

| Option | Description | Selected |
|--------|-------------|----------|
| Hard rename, no aliases | `xs`→`sm`, `default`→`md`; serde rejects old values | ✓ |
| Serde aliases for old values | Old specs keep parsing; two names for one thing | |

**User's choice:** Hard rename (auto — recommended: pre-1.0 clean break per D-02; catalog schema + serde rejection is the enforcement; aliases would be a duplicate control surface)

---

## Interactive-state / motion strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Shared interactive-base class constants | One definition composed into each component | ✓ |
| Per-component class strings | Status quo; 47 hand-copied strings | |

**User's choice:** Shared constants (auto — recommended: structural guarantee over one-off fixes; atoms.rs:137 base string is the consolidation seed)

| Option | Description | Selected |
|--------|-------------|----------|
| `ring-ring` + token duration utilities, drop `motion-reduce:transition-none` where replaced | Single reduced-motion mechanism (0.01ms collapse, events fire) | ✓ |
| Keep both reduced-motion mechanisms | `transition-none` swallows transitionend — contradicts Phase 250 D-07 | |

**User's choice:** Token utilities as the single mechanism (auto — recommended: Phase 250 chose 0.01ms deliberately so `transitionend` keeps firing; duplicating the control surface would undo that)

---

## Migration table location

| Option | Description | Selected |
|--------|-------------|----------|
| Public docs section + phase-notes reference | Consumers (gestiscilo 232) read docs; docs must reflect current features | ✓ |
| Phase notes only | Spec's literal wording, but invisible to consumers | |

**User's choice:** Public docs (auto — recommended: ferro is published; the migration table is consumer-facing by definition)

---

## Drift-guard shape

| Option | Description | Selected |
|--------|-------------|----------|
| Schema-walking test over catalog props | Assert every `variant`/`tone`/`size` prop schema equals the canonical enum set | ✓ |
| Count-style assertions per enum | Weaker; misses a new component adding a deviant prop | |

**User's choice:** Schema-walking test (auto — recommended: mirrors the existing 47-count guard; makes future divergence a test failure by construction)

---

## Claude's Discretion

- Neutral badge visual treatment (filled vs outline).
- Ring width/offset on compact controls.
- Per-family hover classes (surface hover direction, no new colors).
- Audit-driven tone component list beyond the spec's named set.
- Module placement of shared interactive-base constants.

## Deferred Ideas

- Design lint module + CLI (Phase 252), MCP surface + docs + publish (Phase 253).
- gestiscilo consumer migration (their Phase 232, separate repo).
