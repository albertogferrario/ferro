# Phase 254: Props Contracts + Touch Foundation + Design Rules - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 254-CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-05
**Phase:** 254-props-contracts-touch-foundation-design-rules
**Mode:** `--auto` (all gray areas selected; recommended option chosen per question)
**Areas discussed:** ProductTile contract scope, RULE_COMPONENTS sequencing, Touch constants + CSS regen, Lint rule semantics, Props contract depth

---

## ProductTile contract scope (POS-02)

| Option | Description | Selected |
|--------|-------------|----------|
| `categories: Vec<String>` (plural) | Matches the plural `data-product-categories` attribute fixed in 254+255 SCs; multi-category products filter under each membership; one-element vec covers the singular case | ✓ |
| `category: Option<String>` (singular) | Literal reading of the REQUIREMENTS/ROADMAP prose; simpler but multi-category products cannot filter under both, and the plural attribute name becomes incoherent | |

**Notes:** Asymmetric risk drove the pick: renaming plural→singular later is trivial; discovering multi-category products after locking singular breaks the 255 filter contract mid-milestone. Deviation from the requirement literal is explicitly flagged in D-01.

| Option | Description | Selected |
|--------|-------------|----------|
| Criteria-exact renderer touch | 254 emits only `data-product-categories`; visual rendering of image_url/color/stock_badge is a named 256 handoff (tile visuals designed with ProductGrid) | ✓ |
| Full visual rendering in 254 | Delivers all POS-02 visuals now, but designs tile imagery/badges without their ProductGrid context — the exact thrash this contract-first phase exists to prevent | |

---

## RULE_COMPONENTS sequencing (POS-11 × ferro-mcp guard)

| Option | Description | Selected |
|--------|-------------|----------|
| Interim `&["Grid"]` associations, extend in 256 | Guard Direction 2 forces mapping entries in 254; Direction 3 forces existing-builtin names; Grid is the register-root all four rules structurally concern; 256 extends in the registration commit | ✓ |
| Empty-slice associations | Structurally allowed but weaker (no per-component guidance surfaced); accepted as fallback for `fill-viewport-layout-unknown` only if no guidance test breaks | |
| Relax the Direction 3 assertion | Never — weakening a drift guard to sequence work is the anti-pattern the guard exists to catch | |
| Register the components in BUILTIN_TYPES early | Breaks the lockstep (no dispatch arms/renderers exist until 256) | |

---

## Touch constants + CSS regen timing (POS-07)

| Option | Description | Selected |
|--------|-------------|----------|
| Five SC-named constants; migrate render_product_tile now; regen at phase end | Constants get a first real consumer with zero visual change; constant literals are scanner-visible immediately so the generated CSS changes this phase regardless | ✓ |
| Defer regen to 256 | Leaves in-tree ferro-base.css stale against source for a full phase; nothing publishes before 258 but deterministic-fresh is the established posture | |
| Constants only, no ProductTile migration | Drift guard would assert over zero consumers until 256 | |

---

## Lint rule semantics (POS-11)

| Option | Description | Selected |
|--------|-------------|----------|
| `intents: &[]` + internal presence gates; all four Warning | POS presence (type names / fill_viewport flag) is the trigger; intent-keying would silently skip specs where inference predates POS components; warnings must trip consumer `--deny` CI | ✓ |
| `intents: &["collect"]` per research sketch | Matches the research note but creates a silent-skip hole for undeclared-intent agent specs | |
| Mixed severities (pos-cart-present as Info) | Softer, but an incomplete register composition is a real defect gestiscilo CI should gate | |

**Notes:** Research directive recorded in D-10: verify the ferro-fill chain's actual supported-layout set (registry ships `default`/`app`/`auth` at layout.rs:669-671; 252 D-14 claimed `dashboard` — world-state claim needs one verification pass).

---

## Props contract depth (substrate for 256)

| Option | Description | Selected |
|--------|-------------|----------|
| Declare all five structs, lock behavioral anchors, leave field-level shapes to planning | Contracts locked where thrash is expensive ($each iteration, hidden-input compat, no CartRuntime hooks, standalone CategoryNav); field naming stays researcher/planner work against INVENTORY-PRIMITIVES.md | ✓ |
| Fully spec every field in CONTEXT | Over-locks ahead of the research evidence; CONTEXT is not a schema document | |
| Declare empty/stub structs | Fails the phase goal — a stub contract locks nothing and 256 renegotiates everything | |

---

## Claude's Discretion

- Exact class strings in POS_PRESS_ACTIVE / POS_TAP_HIGHLIGHT
- Drift-guard mechanism (source-scan vs composition equality) under the auto-coverage constraint
- Lint predicate details, element_id attribution, suggestion text
- POS_HIT_TARGET_NUMPAD (56px) now vs 256
- Field-level naming/types within D-17 anchors
- infer.rs ProductGrid → collect branch

## Deferred Ideas

- ProductTile visual rendering (image_url/color/stock_badge) → Phase 256
- RULE_COMPONENTS extension to new component names → Phase 256 registration commit
- setupNumpad/setupPosFilter runtimes + data-disable-on-submit → Phase 255
- CartRuntime, barcode wedge, layout-independent ferro-fill chain → Future Requirements (already recorded)
