# Phase 255: POS Runtime Modules + Double-Submit Protection - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-05
**Phase:** 255-pos-runtime-modules-double-submit-protection
**Mode:** `--auto` (recommended defaults selected without interactive prompts)
**Areas discussed:** Numpad contract & entry semantics, PosFilter matching & attribute contract, Double-submit guard placement & emission, Idempotency-key documentation, Module organization & tests

---

## Numpad contract & entry semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Adopt research STACK.md contract as-is | `data-numpad`/`data-numpad-target`/`data-numpad-display`/`data-numpad-key`/`data-numpad-input`; event delegation via `closest()` | ✓ |
| Design a new attribute contract | Fresh naming, diverges from the milestone research anchor Phase 256 renderers were scoped against | |

| Option | Description | Selected |
|--------|-------------|----------|
| Price mode = cents-shift entry, hidden field carries integer cents | Real-POS convention (no decimal key); integer-cents money rule from PITFALLS.md | ✓ |
| Price mode = decimal-point key, hidden field carries decimal string | Extra key, float-adjacent server parsing, no POS-terminal precedent | |

| Option | Description | Selected |
|--------|-------------|----------|
| Keep `data-numpad-input`, extend form_guards number-guard selector to include it | Contract stays self-describing; number-gt-0 guard sees numpad values (SC-3 form-guard compatible) | ✓ |
| Reuse `data-qty-input` on the numpad hidden field | Zero guard change but overloads the ProductTile display-pairing contract | |

**Notes:** Leading-zero collapse, backspace/clear keys, `input` event dispatch locked by SC-3 + research sketch. Max-length cap and display separator → Claude's discretion.

---

## PosFilter matching & attribute contract

| Option | Description | Selected |
|--------|-------------|----------|
| Scoped contract: `data-pos-filter` container, `data-category-tab` (empty = All), `data-pos-search`, tiles = `[data-product-name]` | Multi-scope safe; every attribute finalized for Phase 256 renderers | ✓ |
| Global page-level filtering, no scope container | Simpler JS but breaks with two grids on one page | |

| Option | Description | Selected |
|--------|-------------|----------|
| Add `data-product-name` to `render_product_tile` this phase | Search source + universal tile marker (uncategorized tiles have no categories attribute); render touch is contract-level, matching the phase goal | ✓ |
| Search tile textContent | Couples JS to DOM structure; matches against price text (false positives) | |

| Option | Description | Selected |
|--------|-------------|----------|
| Category AND search intersection | Both filters must match — standard POS/product-picker behavior | ✓ |
| OR / last-filter-wins | Surprising results when both are active | |

| Option | Description | Selected |
|--------|-------------|----------|
| Hide via inline `style.display` | Immune to Tailwind display-utility ordering; no CSS dependency | ✓ |
| `hidden` attribute | UA rule loses to author display utilities on the tile | |
| Tailwind `hidden` class | Stylesheet-order gamble vs grid/flex utilities | |

**Notes:** Case-insensitive matching in JS; space→hyphen token normalization inherited from the 254 render contract. Uncategorized-sentinel tab deferred to Phase 256.

---

## Double-submit guard placement & emission

| Option | Description | Selected |
|--------|-------------|----------|
| Extend `form_guards.rs`, init from `setupFormGuards()` | SC-1/SC-2 name exactly two new setup functions; conceptually a form guard | ✓ |
| New `setupSubmitGuards()` module + dispatcher entry | Third setup function drifts from the phase's own success-criteria list | |

| Option | Description | Selected |
|--------|-------------|----------|
| Disable in the form's `submit` event + `pageshow` bfcache re-enable | Avoids the click-time disable race; back-nav doesn't strand a dead button | ✓ |
| Disable on button `click` | Early disabling can cancel the submission in some engines | |

| Option | Description | Selected |
|--------|-------------|----------|
| Additive `ButtonProps.disable_on_submit` → `render_button` emits the attribute; /cassa confirm button carries it | Framework-level, HTML-assertable, consumed by CartPanel's confirm slot in 256 | ✓ |
| App-level only (RawHtml attribute in /cassa) | No framework contract for Phase 256 to consume | |
| Automatic on every submit button | Changes all existing button HTML; too aggressive | |

---

## Idempotency-key documentation

| Option | Description | Selected |
|--------|-------------|----------|
| New section in `docs/src/features/write-kernel.md`; documentation only | PITFALLS.md: attach to the existing `dispatch_write` hook, no new mechanism; the hook's docs are the natural home | ✓ |
| New standalone docs page | Fragments the write-kernel story | |
| Ship a code helper (form idempotency middleware) | New mechanism explicitly ruled out by research | |

---

## Module organization & tests

| Option | Description | Selected |
|--------|-------------|----------|
| `runtime/numpad.rs` + `runtime/pos_filter.rs`, ES5 house style, extend both existing drift-list tests | Matches ARCHITECTURE.md placement and the SC-named test mechanism | ✓ |
| One combined `runtime/pos.rs` | Diverges from one-module-per-concern house pattern | |

**Notes:** Inline-source inspection + HTML attribute assertions per SC-3/SC-4; semantic-token-only JS class strings (bundle scan); CI-exact gate incl. `--all-features` clippy/test + docs build.

---

## Claude's Discretion

- Price-mode display separator; max-length entry cap
- Exact active-tab class strings (token-compliant, full literals)
- Whether /cassa demo handler demonstrates the idempotency hidden field
- Internal JS helper factoring; diacritic handling in search (lowercase-only acceptable)

## Deferred Ideas

- CartRuntime (operator-deferred, Future Requirements)
- Barcode keyboard-wedge module (operator-deferred)
- `pos-text-input-position` lint rule candidate (PITFALLS.md Pitfall 4)
- "Uncategorized" virtual sentinel tab (Phase 256 render decision)
