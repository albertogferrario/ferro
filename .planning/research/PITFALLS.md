# Pitfalls Research — v16.6 POS Component Suite

**Domain:** Touch-first sale-screen components in a server-rendered, no-JS-framework design system (ferro-json-ui builtin catalog)
**Researched:** 2026-07-04
**Confidence:** HIGH — grounded in the gestiscilo cassa friction (253-FRICTION.md, first-hand audit of the ~1500 lines of RawHtml escape hatches), the Phase 253 code review (WR-01 col-span safelist finding), Phase 252 (destructive-confirmation lint rule false-positive), the fill_viewport implementation in input.css, the BUILTIN_TYPES drift-guard in catalog.rs, and established knowledge of touch ergonomics and server-side form idempotency.

---

## Critical Pitfalls

### Pitfall 1: SUB-44PX TOUCH TARGETS — product tiles and cart rows sized for desktop, not fingers

**What goes wrong:** A 40×40px tile looks correct on a monitor but is consistently missed on a tablet at a register; wrong products land in the cart under customer pressure. Acute for quantity-stepper +/− buttons, category tabs, cart row action icons. Apple HIG requires 44pt; WCAG 2.5.5 AAA requires 44px; WCAG 2.5.8 AA minimum 24px.

**Why:** Dashboard components size hit targets by visual element (Tailwind padding, e.g. `py-2` on `text-sm` ≈ 32px). No catalog component enforces a minimum touch height at render time except ProductTile.

**Prevention:** Every POS-family component (ProductTile, QuantityStepper, NumpadKey, CategoryTab, CartLineAction) emits `min-h-[44px]` (or larger) at the Rust render layer, not as a theme override. Touch padding via container `min-h` + button `padding: 0` expands hit region without affecting layout. Encode as a render-time guarantee (see Architecture §5 — not expressible at spec/lint level).

**Warning signs:** `size: "sm"` on interactive POS elements; tests asserting visual heights but not minimum touch heights; gestiscilo cassa uses `h-16` (64px) tiles — a shorter builtin fails at the reference consumer.

**Phase:** first POS component implementation phase, from the first commit.

---

### Pitfall 2: DOUBLE-TAP ZOOM AND GHOST CLICKS — a product tap becomes a zoom or two cart adds

**What goes wrong:** iOS Safari synthesizes a delayed `click` after `touchend` (~300ms); fast taps submit a form twice, adding two line items. Double-tapping text-bearing buttons triggers browser double-tap zoom.

**Prevention (three mechanisms in combination):**
1. `touch-action: manipulation` on all interactive POS elements — eliminates double-tap zoom and the 300ms delay with zero JS. Emit in every POS component's base class string (behavior property, not a theme token); consider a shared constant in `render/classes.rs`.
2. Server-side idempotency key in every cart-mutation form (hidden field generated at render, checked with a short TTL). The `framework::write` kernel already has an idempotency hook (Phase 231/232) — attach there, no new mechanism.
3. Disable-submit-on-first-click via a `data-disable-on-submit` attribute in the existing `FERRO_RUNTIME_JS`.

**Warning signs:** POS buttons without `touch-action: manipulation`; cart forms without idempotency key; rapid double-tap on a real iPad producing two mutations.

**Phase:** component implementation (touch-action); handler/adoption phase (idempotency key) — must exist before gestiscilo adoption.

---

### Pitfall 3: WHOLE-PAGE SCROLL DISPLACEMENT — cart pane scrolls with the page instead of pinning

**What goes wrong:** The classic gestiscilo cassa bug (200px cart panel scrolling off screen). The Phase 253 fix (`fill_viewport` + Grid `fill`) exists but is fragile: the CSS selector chain (`body.ferro-fill > div.flex > main > div > #ferro-json-ui > *`) only activates for dashboard-family layout shells. A POS spec with a custom/unknown layout name silently falls back to scrolling behavior — no error (253-FRICTION.md Gap 1: silent layout fallback).

**Prevention:**
1. Lint rule `fill-viewport-layout-unknown` → Warning when `fill_viewport: true` and `layout` is not a registered layout name.
2. Document the selector-chain dependency in the POS authoring guide + `generation_context`.
3. A layout-name-independent `ferro-fill` chain is a deeper refactor — likely post-v16.6.

**Warning signs:** cassa spec with `fill_viewport: true` and `layout: "cassa"`; cart scrolls away on a real tablet; no lint finding on such a spec.

**Phase:** component catalog phase (rule must exist before agents author cassa specs); adoption phase depends on it.

---

### Pitfall 4: KEYBOARD POPUP DISPLACEMENT — numpad or search input pushes critical cart UI off screen

**What goes wrong:** On a portrait tablet, focusing a text/number input summons the software keyboard (40–60% of viewport), obscuring or displacing the cart/total in a `fill_viewport` layout. `dvh` accounts for browser chrome, NOT the software keyboard (iOS never resizes the viewport; `visualViewport` requires JS).

**Prevention:** Numpad is a custom tap surface (NumpadKey grid), never a native `<input type="number">` — no keyboard triggered at all. Required text inputs (search, customer name) anchor to the TOP of the layout or open in a modal. Document in `generation_context`; lint candidate `pos-text-input-position` (Warning for text Input/Textarea inside a fill-viewport spec outside the top panel).

**Phase:** component design phase (Numpad shape decided up front).

---

### Pitfall 5: SERVER ROUND-TRIP PERCEIVED LATENCY — every product tap takes 300–800ms before the cart updates

**What goes wrong:** PRG (POST→redirect→GET→full re-render) per cart action costs 200–800ms per tap. POS operators tolerate <500ms; cashiers tapping 5 items in 3 seconds perceive a stuck UI, tap again, and double-submit.

**Prevention (latency minimization, not client state):**
1. Session-backed draft order (not a DB write per tap) if per-tap POSTs are used; or client-accumulate-then-single-commit (the current cassa.json hidden-input pattern / a cart runtime).
2. CSS `:active` press states (v16.5 interactive quality bar) give <16ms visual feedback regardless of round-trip.
3. POST-to-same-page can return 200 with inline re-render instead of PRG redirect (route-level decision), saving one round-trip.
4. Cart-mutation handlers must not re-query the full product catalog.

**Warning signs:** full catalog query in the add-item handler; no `active:` classes on ProductTile/NumpadKey; replacing gestiscilo's JS-driven local cart with per-tap round-trips (slower than what it replaces).

**Phase:** architecture decided at requirements/planning; `:active` in component phase; latency analysis before adoption.

---

### Pitfall 6: BUILTIN-COMPONENT COUNT LOCKSTEP — one addition breaks two drift guards

**What goes wrong:** New builtin added, `catalog.rs` drift guard (asserts absolute count 47) fails, AND the ferro-mcp mirrored count fails separately. Known repo failure mode (three hardcoded sites before consolidation; now one canonical + one documented mirror).

**Prevention:** execute the CLAUDE.md checklist per component, in order: props struct → render fn → BUILTIN_TYPES → dispatch arm → BUILTIN_SPECS → canonical count bump (catalog.rs:1219 + History comment) → ferro-mcp mirror bump (json_ui_catalog.rs:396 + expected array) → `gen-ferro-base-css.sh` regen → facade re-exports. If components land in different phases, each phase bumps counts in its own commit — never batch.

**Phase:** every component-addition phase; the checklist is an acceptance criterion in each plan.

---

### Pitfall 7: CSS SAFELIST DRIFT — runtime-concatenated class names missing from ferro-base.css

**What goes wrong:** `format!("grid-cols-{}", n)` produces classes the Tailwind v4 scanner never sees; they're absent from the pre-built `ferro-base.css`, so layout silently breaks in production. This is exactly Phase 253 WR-01 (`col-span-{2,3,4}` missing until added to `@source inline(...)`), invisible to tests that didn't assert on the class.

**Prevention:** the Phase 251 rule — every emitted Tailwind class appears as a complete string literal (exhaustive `match` arms) or in `@source inline(...)`. Product-grid column counts, cart-panel widths, numpad key sizes: all via bounded enums → full literals. Every POS render function gets a test asserting the expected class strings in output. Review check: `grep -rn 'format!(".*-{}' ferro-json-ui/src/` → zero unaccounted matches.

**Phase:** every component implementation phase (review criterion).

---

### Pitfall 8: DESIGN-LINT RULE MISFIRES ON DATA-BOUND PROPS

**What goes wrong:** A POS lint rule checks a static field that authors legitimately supply via `$data.*` binding; the rule false-positives on every valid data-bound spec, forcing `allow`-list pollution. Real v16.5 bug class (Phase 252 WR-01/WR-02: rules misfiring on `$data`-bound `empty_message`/`breadcrumb`; destructive-confirmation checking the wrong level).

**Prevention:** every new POS rule treats a `$data.*` reference in a checked field position as satisfying the presence check; every rule ships with BOTH a static fixture and a data-bound fixture in its tests.

**Phase:** the lint-rule authoring phase.

---

### Pitfall 9: TOKEN BYPASS — raw Tailwind palette values instead of semantic tokens

**What goes wrong:** `bg-orange-500` instead of `bg-primary`, `text-zinc-800` instead of `text-text` — correct with the default theme, silently broken under any consumer theme. gestiscilo's RawHtml picker already suffers this.

**Prevention:** review blocker on any raw palette class (`red-`, `blue-`, `orange-`, `zinc-`, `gray-`, `slate-`…) in POS render functions; extend the Phase 251 runtime drift-guard pattern (`variant_classes_use_semantic_tokens`) to new runtime JS modules.

**Phase:** every component implementation phase (review criterion, alongside the v16.5 interactive-state bar).

---

### Pitfall 10: RAWHTML ESCAPE HATCH BLINDSPOT — lint cannot see inside RawHtml

**What goes wrong:** `prefer-components` (Phase 253, rule 11) surfaces RawHtml as Info — never fails `--deny`. If the new components don't fully cover the picker's behaviors, adoption leaves RawHtml islands invisible to the design system.

**Prevention:** after the POS components ship, escalate RawHtml-in-POS-spec findings to Warning (scope decision: automatic for POS-intent specs vs an explicit rule update). Gate the gestiscilo adoption phase on `design:lint --deny` over cassa specs with any remaining RawHtml explicitly `allow`-justified.

**Phase:** components phase ships; adoption phase runs the sweep + severity escalation.

---

### Pitfall 11: INTENT VOCABULARY BLOAT — adding "register"/"kiosk" as new intents

**What goes wrong:** The sale screen doesn't feel like any of the seven intents, tempting an eighth `Intent::Register`. This breaks the KNOWN_INTENTS drift guard, `infer_intent`, the MCP generation context, and the v16.5 decision (archetypes ARE the seven intents).

**Prevention:** express POS within the seven — Collect (the recommended mapping; cassa.json already declares it) or Process. Key POS lint rules on component composition/layout, never on a new intent value. Flag at planning time, before code.

**Phase:** planning/roadmap (this document is the flag).

---

### Pitfall 12: PLUGIN REGISTRY CONFUSION — POS components placed in plugins instead of builtins

**What goes wrong:** The plugin registry looks like the extension path for "POS-specific" components, but it has known global-mutable-state debt, isn't counted by the drift guard, and is invisible to `design:lint`/`json_ui_catalog`/`checkpoint_projection`.

**Prevention:** explicit scope constraint — all v16.6 components are builtins. Review blocker on any `register_component` call.

**Phase:** scope definition (this milestone's REQUIREMENTS.md should state it).

---

### Pitfall 13: SCOPE CREEP INTO PAYMENT, RECEIPT, AND HARDWARE

**What goes wrong:** The consumer's cassa naturally extends into payment method selection, change calculation, fiscal receipts, shift close, scanners/drawers. Each addition redefines the milestone. gestiscilo's `helpers.rs` already has payment inline with the cart builder — the temptation is immediate.

**Prevention:** the PROJECT.md out-of-scope boundary (payment/receipt/shift close) is reviewed first in any plan touching cassa. `PaymentPanel`, `ReceiptPreview`, `ShiftClose`, hardware triggers → cut before implementation; defer to a later milestone. Each phase plan carries a one-line out-of-scope reminder.

**Phase:** every phase planning review.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Raw Tailwind palette classes in POS components | Faster authoring | Custom themes break silently | Never — semantic tokens |
| `format!("util-{}", n)` runtime classes | DRY code | Absent from ferro-base.css; production-only breakage | Never — exhaustive full-literal match arms |
| Plugin registry for POS components | Avoids BUILTIN_TYPES churn | Invisible to catalog/lint/MCP; count guard wrong | Never for internal catalog components |
| Native `<input type="number">` numpad | Simpler | Software keyboard; small targets; displacement | Never for register screens |
| No idempotency key on cart forms | Fewer fields | Duplicate line items on rapid taps | Never on touch input |
| No `touch-action: manipulation` | One less class | 300ms delay; double-tap zoom; ghost clicks | Never on POS targets |
| No fill_viewport | No CSS-chain dependency | Cart scrolls off screen | Never for cassa screens |
| `Intent::Register` | Clear naming | Breaks seven-intent invariant + drift guards | Never |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `fill_viewport` + custom layout | Unknown layout name → silent scroll fallback | dashboard/app layout; `fill-viewport-layout-unknown` lint rule before agents author specs |
| Tailwind v4 scanner + POS classes | Runtime concatenation invisible to scanner | Full-literal match arms; `@source inline(...)` for anything dynamic |
| Cart POST + PRG | Back button re-submits; rapid taps duplicate | Idempotency key (hidden field) + PRG; `framework::write` idempotency hook |
| BUILTIN_TYPES + ferro-mcp mirror | Bumping one count, not the other | Canonical guard in ferro-json-ui; mirror updated in the same commit |
| `design::lint` + data binding | Static field check misses `$data.*` forms | Rules accept `$data.*` as presence; dual fixtures per rule |
| CSS `:active` + touch | `:active` unreliable on some Android | `:active` + optional `data-pressed` JS toggle via the existing runtime |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| DB write per cart tap | 400–800ms per tap; queue forms | Session draft or accumulate-then-commit | First real register usage |
| Full catalog query per mutation | Slow qty increments | Query once at page load | Catalogs with 50+ products |
| No `:active` feedback | Taps feel dead → re-tap → double submit | `active:` classes; paint <16ms | First cashier usage |
| fill_viewport chain break | Cart collapses / page scrolls | Lint rule + documented chain | Any layout-name change |

## "Looks Done But Isn't" Checklist

- [ ] ProductTile/NumpadKey/stepper minimum `min-h-[44px]` in rendered HTML — verify computed style on a real iPad.
- [ ] `touch-action: manipulation` in every POS interactive class string — no 300ms delay on real iOS.
- [ ] Hidden `idempotency_key` in every cart-mutation form — rapid double-submit creates no duplicate.
- [ ] After `gen-ferro-base-css.sh`, every POS class present in `ferro-base.css` (grep the output).
- [ ] Both drift-guard counts match `BUILTIN_TYPES.len()` after EACH component addition.
- [ ] No raw palette class in any POS render function.
- [ ] cassa spec uses a registered layout; `body.ferro-fill` chain pins on a real tablet.
- [ ] Every new POS lint rule has a `$data.*`-bound fixture.
- [ ] `ferro-projections/src/intent.rs` still has exactly seven variants.
- [ ] No `register_component` call for any POS component.
- [ ] No PaymentPanel/ReceiptPreview/ShiftClose/hardware element in any v16.6 phase.

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Sub-44px targets | First component phase | `min-h-[44px]` in HTML; visual UAT on iPad |
| Double-tap/ghost clicks | First component phase | `touch-action: manipulation`; no double-submit |
| Scroll displacement | Catalog phase (lint rule) | `fill-viewport-layout-unknown` exists |
| Keyboard displacement | Component design phase | Numpad is a custom surface, no native input |
| Round-trip latency | Requirements/architecture | <500ms cart mutation; `:active` feedback |
| Count lockstep | Every component phase | Both guards pass per addition |
| Safelist drift | Every component phase | grep zero unaccounted `format!` classes |
| Lint data-bound misfires | Lint authoring phase | Dual fixtures per rule |
| Token bypass | Every component phase | No raw palette classes (review) |
| RawHtml blindspot | Adoption phase | `design:lint --deny` on cassa specs |
| Intent bloat | Planning | Seven variants unchanged |
| Plugin confusion | Scope definition | No `register_component` |
| Payment scope creep | Every plan review | Out-of-scope line in each plan |

## Sources

- `.planning/phases/253-mcp-surface-docs-publish/253-FRICTION.md` — gestiscilo cassa audit (~1500 lines RawHtml), fill_viewport motivation, Gap 1 silent layout fallback.
- `.planning/phases/253-mcp-surface-docs-publish/253-REVIEW.md` — WR-01 col-span safelist; IN-01 test blindspot.
- `.planning/phases/251-component-variant-discipline-interactive-state-pass/251-PATTERNS.md` — full-literal match-arm rule; shared interactive constants.
- `.planning/phases/252-design-module-lint-cli/252-PATTERNS.md` — lint purity (no `$data` resolution); rule false-positive class.
- `ferro-json-ui/assets/input.css` — fill_viewport selector chain; `@source inline()` safelist.
- CLAUDE.md / MEMORY.md — component-addition checklist; plugin-registry debt; seven-intent decision.
- Web: Apple HIG 44pt; WCAG 2.5.5/2.5.8; `touch-action: manipulation` behavior; PRG + idempotency-key patterns.

---

*Pitfalls research for: v16.6 POS Component Suite — touch-first sale-screen components in ferro-json-ui builtin catalog*
*Researched: 2026-07-04*
