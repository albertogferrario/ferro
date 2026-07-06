---
phase: 255-pos-runtime-modules-double-submit-protection
reviewed: 2026-07-05T13:47:38Z
depth: standard
files_reviewed: 16
files_reviewed_list:
  - app/src/views/cassa.json
  - docs/src/features/write-kernel.md
  - docs/src/json-ui/components.md
  - ferro-json-ui/src/catalog.rs
  - ferro-json-ui/src/component.rs
  - ferro-json-ui/src/lib.rs
  - ferro-json-ui/src/projection/component_map.rs
  - ferro-json-ui/src/render/atoms.rs
  - ferro-json-ui/src/render/classes.rs
  - ferro-json-ui/src/render/mod.rs
  - ferro-json-ui/src/runtime/filters.rs
  - ferro-json-ui/src/runtime/form_guards.rs
  - ferro-json-ui/src/runtime/mod.rs
  - ferro-json-ui/src/runtime/numpad.rs
  - ferro-json-ui/src/runtime/tiles.rs
  - ferro-mcp/src/tools/json_ui_catalog.rs
findings:
  critical: 0
  warning: 3
  info: 4
  total: 7
status: issues_found
---

# Phase 255: Code Review Report

**Reviewed:** 2026-07-05T13:47:38Z
**Depth:** standard
**Files Reviewed:** 16
**Status:** issues_found

## Summary

Reviewed the Phase 255 diff (`8bb6dc46^..HEAD`): the domain-neutral vocabulary rename (ProductTile→Tile, `data-product-*`→`data-filter-*`, `pos-*`→`register-*` lint ids, `POS_*` constants de-prefixed), the two new ES5 runtime modules (`setupNumpad`, `setupFilters`), the double-submit guard (`data-disable-on-submit` in `setupFormGuards`), the `ButtonProps.disable_on_submit` emission, and the write-kernel double-submit documentation.

**Security checks pass.** `data-filter-text` and `data-filter-tokens` both pass through `html_escape` (which covers `&`, `<`, `>`, `"`, `'` — sufficient for double-quoted attribute context), with XSS regression tests `tile_escapes_categories` (T-255-01) and `tile_escapes_filter_text` (T-255-07) in `render/atoms.rs`. No raw palette classes in the new JS literals (semantic tokens only: `border-primary`, `text-text-muted`, etc., mirroring `tabs.rs`). All new runtime code is ES5-syntax (`var`/`function`, no arrows/let/const/template literals). The client guard is correctly framed as a UX affordance with the `dispatch_write` idempotency hook as the authoritative dedupe layer.

**Rename is complete and consistent** across the reviewed surface: no leftover `ProductTile`, `data-product-*`, `pos-fill-viewport`/`pos-grid-fill`/`pos-cart-present`, `POS_HIT_TARGET*`, or `setupPosFilter` references in `ferro-json-ui`, `ferro-mcp`, `ferro-cli`, `app`, or `docs/src`. The renamed lint rule IDs in `ferro-mcp/src/tools/json_ui_catalog.rs` (`register-fill-viewport`, `register-grid-fill`, `register-selection-present`) exactly match `ferro-json-ui/src/design/rules.rs:85-99`. Bundle assembly and the `ferroRuntime()` dispatcher both include `setupNumpad`/`setupFilters`, with drift-guard tests.

**Key concern:** the phase's flagship demo does not actually exercise the double-submit guard — the `cassa.json` confirm button is a standalone Button (not inside a Form), which renders as a bare `<button>` with no enclosing form, so `initDisableOnSubmit` early-returns and the guard never binds (WR-01).

## Warnings

### WR-01: `disable_on_submit` demo in cassa.json cannot bind — button has no enclosing form

**File:** `app/src/views/cassa.json:57-65` (also `docs/src/features/write-kernel.md:158-170`, `ferro-json-ui/src/render/atoms.rs:259-263`, `ferro-json-ui/src/runtime/form_guards.rs:30-34`)
**Issue:** `btn_confirm` is a standalone `Button` child of a `Card` — it is not inside any `Form` element, and no Form wraps it anywhere in the spec. For non-GET actions, `render_button` returns the bare `<button>` with no form wrapper (`atoms.rs:259-263`: "the inner button is returned as-is"). Consequently:
1. `initDisableOnSubmit` does `btn.closest('form')` → `null`, then checks `btn.getAttribute('form')` → `null`, and returns at `form_guards.rs:34` — **the double-submit guard shipped by this phase never binds on the one page that demonstrates it.**
2. The button has no submit mechanism at all through the render path — clicking "Conferma ordine" does not POST to `/cassa/conferma` (no wrapping `<form>`, no `type="submit"` inside a form, no runtime click handler for element actions). This wiring gap pre-dates the phase (the 254 version of cassa.json had the same structure), but commit `bd21cefb` ("255-05: /cassa disable_on_submit demo") shipped the demo on top of it.
3. The `write-kernel.md` layer-1 example (lines 158-170) reproduces this exact JSON shape without showing the required enclosing Form, while the prose says the runtime "binds a `submit` event listener on the enclosing form" — a reader copying the snippet gets a guard that silently never engages.

**Fix:** Wrap the confirm button in a `Form` element in cassa.json so both the POST and the guard work:
```json
"confirm_form": {
  "type": "Form",
  "props": { "action": { "handler": "/cassa/conferma", "method": "POST" } },
  "children": ["btn_confirm"]
},
"btn_confirm": {
  "type": "Button",
  "props": {
    "label": "Conferma ordine",
    "variant": "primary",
    "button_type": "submit",
    "disable_on_submit": true
  }
}
```
(and reference `confirm_form` from `cart_pane.children`). Update the write-kernel.md example to show the enclosing Form, or add an explicit sentence: "`disable_on_submit` requires the button to be inside a `<form>` (a `Form` element or a form-wrapped context like ActionGroup); a standalone Button with an element-level POST action is not form-wrapped." Alternatively — the structural fix — make `render_button` wrap standalone non-GET action buttons in `<form action method="post">`, which the `docs/src/json-ui/actions.md:10` contract ("Non-GET actions render as form submissions") already implies.

### WR-02: Double-submit latch fires even when the first submission was cancelled

**File:** `ferro-json-ui/src/runtime/form_guards.rs:36-41`
**Issue:** The submit listener latches unconditionally:
```js
form.addEventListener('submit', function(e) {
    if (btn._submitted) { e.preventDefault(); return; }
    btn._submitted = true;
    btn.setAttribute('disabled', 'disabled');
    ...
});
```
If any earlier handler cancels the submission — e.g. an inline `onsubmit="return confirm(...)"` (parse-time inline handlers run before this DOMContentLoaded-registered listener) or any future validation hook calling `e.preventDefault()` — the guard still sets `btn._submitted = true` and disables the button. The user cancelled the dialog, no request was sent, and the form is now permanently unsubmittable until a full reload (the `pageshow` reset only fires on bfcache restore, not on staying on the page). Native HTML5 constraint validation is safe (an invalid form never fires `submit`), but cancellable-submit patterns are not.
**Fix:** Skip the latch when the submission was already cancelled:
```js
form.addEventListener('submit', function(e) {
    if (e.defaultPrevented) return;
    if (btn._submitted) { e.preventDefault(); return; }
    btn._submitted = true;
    ...
});
```

### WR-03: Unescaped attribute value in selector string can throw and abort the entire runtime

**File:** `ferro-json-ui/src/runtime/numpad.rs:34` (same pattern: `ferro-json-ui/src/runtime/tiles.rs:18-19`; blast radius: `ferro-json-ui/src/runtime/mod.rs:48-66`)
**Issue:** `initNumpad` builds a selector by string concatenation:
```js
var input = document.querySelector('input[data-numpad-input="' + field + '"]');
```
`field` comes from a server-emitted attribute that is `$data`-bound in specs (e.g. cassa.json binds Tile `field` from `/p/field`, DB-derived). HTML-escaping at render time protects the attribute context, but `getAttribute` returns the decoded raw string — a `field` containing `"` or `]` produces an invalid selector and `querySelector` throws a `SyntaxError` DOMException. Because all `setup*` calls run sequentially inside the single `ferroRuntime()` DOMContentLoaded handler (`runtime/mod.rs:48-66`), one throw aborts every subsequent setup (`setupFilters`, `setupModals`, `setupToasts`, `setupLazyHeroes` never run) — a single malformed field name kills the whole page runtime. Likelihood is low (field names are conventionally `qty_<id>`), but the failure mode is total. The `tiles.rs` occurrence is pre-existing; `numpad.rs` newly extends the pattern.
**Fix (structural):** isolate failures per concern in the dispatcher so one module cannot take down the rest:
```js
function ferroRuntime() {
    var setups = [setupScrollPreserve, setupSSE, /* ... */ setupNumpad, setupFilters, /* ... */];
    for (var i = 0; i < setups.length; i++) {
        try { setups[i](); } catch (err) { /* swallow; one concern must not kill the rest */ }
    }
}
```
Optionally also sanitize the interpolated value (`field.replace(/["\\\]]/g, '')`) at the two selector sites.

## Info

### IN-01: `numpadQtyKey` and `numpadPriceKey` are byte-identical

**File:** `ferro-json-ui/src/runtime/numpad.rs:58-82`
**Issue:** The two key-handling functions have identical bodies (clear/backspace/leading-zero-collapse/append with `MAX_LEN = 9`); only the comments differ. The mode split currently lives entirely in the display formatting (`numpadPriceDisplay`).
**Fix:** Collapse to a single `numpadKey(current, key)` and keep the mode branch only for display formatting — or add a comment stating the duplication is deliberate headroom for divergence (e.g. a future price-mode "00" key), so the next reader does not "helpfully" merge or drift them.

### IN-02: Residual product/POS vocabulary in doc comments after the rename

**File:** `ferro-json-ui/src/component.rs:1354, 1395, 1437, 1465`; `ferro-json-ui/src/render/classes.rs:57-58`
**Issue:** The rename left doc-comment residue in the surfaces this phase touched:
- `component.rs:1354` — `TileProps` doc still opens with "Props for a touch-friendly **product tile**" (the body was neutralized but the first line was not).
- `component.rs:1395` — `TileGridProps.data_path` doc: "JSON pointer to the **product** array".
- `component.rs:1437, 1465` — `QuantityStepperProps` / `NumpadProps` docs still say "**POS** builtin".
- `classes.rs:58` — `TAP_HIGHLIGHT` still carries the class string `"pos-tap-highlight"`. This one is deliberate (backed by `@utility pos-tap-highlight` in input.css; renaming requires regenerating ferro-base.css) and documented in the comment — noting it so a future pass renames the CSS utility and constant value together.
**Fix:** s/product tile/tile/, s/product array/item array/, s/POS builtin/builtin/ in the three doc comments. Leave `pos-tap-highlight` for a coordinated CSS-regen change.

### IN-03: Numpad accepts arbitrary `data-numpad-key` values into the hidden field

**File:** `ferro-json-ui/src/runtime/numpad.rs:40-52, 58-82`
**Issue:** The key handler appends any key value that is not `clear`/`backspace` verbatim — a spec-authored `data-numpad-key="."` or `"abc"` would corrupt the integer-cents contract in the hidden input. The contract explicitly delegates re-validation to the server (numpad.rs:1-4), and the key emitter (Phase 256 renderer) is framework-controlled, so this is authoring robustness rather than a security issue.
**Fix:** Guard the append path with a digit check: `if (!/^[0-9]+$/.test(key)) return current;`.

### IN-04: Number guard can visually re-enable a latched double-submit button

**File:** `ferro-json-ui/src/runtime/form_guards.rs:107-122` (vs. `36-41`)
**Issue:** When a form combines `data-form-guard="number-gt-0"` with a `data-disable-on-submit` submit button, an `input` event after the first submission re-runs `check()`, which calls `removeAttribute('disabled')` and strips the opacity classes — while `btn._submitted` stays `true`. The button looks enabled but every submit is `preventDefault`-ed: a confusing dead state (until bfcache restore or reload).
**Fix:** Have `check()` respect the latch: `if (submitBtn._submitted) return;` at the top, or share a single enable/disable helper that consults `_submitted`.

---

_Reviewed: 2026-07-05T13:47:38Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
