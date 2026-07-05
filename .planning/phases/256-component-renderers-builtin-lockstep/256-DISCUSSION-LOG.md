# Phase 256: Component Renderers + BUILTIN Lockstep - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-06
**Phase:** 256-component-renderers-builtin-lockstep
**Mode:** `--auto` (recommended defaults selected without interactive questioning)
**Areas discussed:** Tile tap-to-add redesign, SelectionPanel live-view runtime, TileGrid/FilterTabs composition, QuantityStepper/Numpad render contracts, BUILTIN lockstep mechanics, Grid row_weights render, Locale-neutral defaults audit

---

## Tile tap-to-add redesign

| Option | Description | Selected |
|--------|-------------|----------|
| Tile root carries `data-qty-inc` (reuse tiles runtime) | Whole tile is one tap surface; the shipped `initQtyButton` binds it with zero new add-path runtime | ✓ |
| New `data-tile-add` attribute + new runtime handler | Dedicated add runtime; duplicates the existing qty mechanism | |
| Keep 255 stepper markup, add tap zone | Contradicts the operator's "NO on-tile steppers or qty display" lock | |

**Notes:** `price_cents: Option<u64>` additive prop → `data-unit-price` chosen over
parsing the `price` display string (unparseable) or panel-side price data
(duplicated catalog). `color` renders via exhaustive Tone match (full literals)
over inline-style raw colors (token bypass) or dynamic class construction
(SC-3 violation). Legacy byte-identical test superseded and deleted — this IS
the designed redesign it was guarding against accidental versions of.

## SelectionPanel live-view runtime (CartRuntime slice)

| Option | Description | Selected |
|--------|-------------|----------|
| Input-event-driven reconciliation | Panel reconciles off bubbling `input` events from `data-qty-input` fields; one code path for tap/stepper/numpad; panel = pure view | ✓ |
| Direct coupling (tile tap calls panel API) | Couples tiles runtime to panel; second state source; misses numpad writes | |
| Server round-trip per change | Rejected by STACK.md posture (form is the contract; intermediate state client-transient) | |

**Notes:** Lines cloned from a server-rendered `<template>` (classes stay in
Rust source, scanner-visible) over imperative JS `createElement`. Per-line
controls use delegated `data-selection-*` attributes, NOT `data-qty-inc/dec`
(load-time per-element binding vs post-load cloned lines → double/no-binding
races). `form_id` = HTML id of an ancestor `Form` (FormProps.id exists);
components never render `<form>`s. Confirm slot = panel children (author
supplies the Button with `disable_on_submit` + `form`), no button-config props.

## TileGrid / FilterTabs composition

| Option | Description | Selected |
|--------|-------------|----------|
| TileGrid root emits `data-filter-scope`; shared tab-strip helper; standalone FilterTabs = nearest ancestor scope | Matches shipped filters.rs semantics; zero runtime change; SC-5 path is the integrated strip | ✓ |
| Value-paired scoping (`data-filter-for`) now | Additive runtime extension for sibling placement — no current consumer needs it; deferred | |
| Document-level fallback scope | Conflicts with TileGrid-emitted scopes on the same page | |

**Notes:** "Uncategorized" sentinel tab stays deferred (needs a reserved
non-empty token). Search input gets `text-base` (16px, iOS zoom pitfall).

## QuantityStepper / Numpad render contracts

| Option | Description | Selected |
|--------|-------------|----------|
| Stepper self-contained (own hidden input) + `data-qty-min/max/step` honored via small tiles.rs extension | Declared props are honored, not decorative; one-input-per-field invariant kept | ✓ |
| Emit bounds attributes, runtime ignores | Contract lies — props declared but dead | |
| No bounds this phase | Props already shipped in 254/255 declarations | |

**Notes:** Numpad emits the exact 255 contract incl. `data-numpad-mode` and its
own adjacent hidden input; keys ≥56px via HIT_TARGET_NUMPAD; never a visible
native input.

## BUILTIN lockstep mechanics

| Option | Description | Selected |
|--------|-------------|----------|
| One commit per component: entry + dispatch + spec + BOTH count bumps + History comment (48→52); RULE_COMPONENTS extended in the registering commits | Roadmap SC-1 letter and the 254 D-14 same-commit rule | ✓ |
| Single batch commit 47→52 | Loses the per-addition audit trail SC-1 requires | |

## Grid row_weights render

| Option | Description | Selected |
|--------|-------------|----------|
| Inline `style="grid-template-rows: Nfr …"`, fill-mode only, emit-as-given | Exactly SC-4; no validation this phase (lint candidate later) | ✓ |
| Generated CSS classes per weight combo | Unbounded class space — scanner contract violation | |

## Locale-neutral defaults audit

| Option | Description | Selected |
|--------|-------------|----------|
| Neutral English defaults ("All", English aria-labels, neutral empty message); consumers override via props | Project-agnostic crates principle; corrects the 254 "Tutte" rustdoc claim and the Italian aria-labels in render_tile | ✓ |
| Keep "Tutte"/Italian labels | Locale leak in a `ferro-*` crate | |

## Claude's Discretion

- Exact tile markup structure (valid-HTML placement of the hidden input around a button root), image aspect, text-only layout
- `data-selection-*` attribute names, template internals, money-format helper, panel display-format prop names
- Registration order, BUILTIN_SPECS example content, TileGrid column ladder
- Aria wording, backspace/clear glyphs
- `infer.rs` TileGrid → collect branch (carried discretionary from 254)

## Deferred Ideas

- Per-line extra columns generic mechanism (255 handoff — evidence base removed by the tap-to-add redesign; revisit on gestiscilo friction)
- Sibling FilterTabs↔TileGrid pairing (`data-filter-for`)
- Uncategorized sentinel tab; qty badge / picked ring on tiles
- `row_weights` validation lint
