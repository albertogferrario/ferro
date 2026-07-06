# Stack Research: v16.6 POS Component Suite

**Domain:** Touch-first sale-screen components in a server-rendered, no-JS-framework design system
**Researched:** 2026-07-04
**Confidence:** HIGH (CSS platform properties verified via MDN/caniuse; JS patterns verified against existing runtime; WCAG numbers from W3C primary docs)

---

## Zero-New-Dependency Verdict: CONFIRMED

No new Rust crates, no new JS libraries, no new CSS tooling. Every requirement below is met by CSS
platform properties + vanilla JS (following the existing `runtime/*.rs` pattern) + Tailwind v4
utility class additions. The `input.css` safelist requires extension for new dynamically-generated
classes; `gen-ferro-base-css.sh` remains the only build step.

---

## CSS Touch Ergonomics

### `touch-action: manipulation` — Double-Tap Zoom + Click Delay

**What it does:** Enables pan and pinch-zoom; disables the double-tap-to-zoom gesture. Removing
double-tap zoom eliminates the 300ms click-event delay browsers impose to wait for a potential
second tap — a critical latency win for POS tile grids where operators tap rapidly.

**iOS Safari support:** `manipulation` is supported from iOS Safari 9.3 and all current versions.
`none`, `pan-x`, `pan-y` are NOT supported on iOS Safari — only `auto` and `manipulation` are safe
to use cross-platform. (Confirmed via caniuse mdn-css_properties_touch-action_manipulation.)

**Current state:** The existing `ProductTile` component already applies `touch-manipulation` on its
outer container (`rounded-lg border … touch-manipulation`). This is correct for the container.
Extend to buttons inside all new POS components (numpad keys, category tabs, cart action buttons) —
the container class alone is not sufficient if child buttons trigger their own event handling.

**Do NOT use** `user-scalable=no` / `maximum-scale=1` in the viewport meta tag. This violates
WCAG 1.4.4 (Resize Text, AA) by preventing users from zooming the page.

**Integration:** Add `touch-manipulation` to every interactive POS component's button/tile class
string in the Rust renderer. Tailwind already knows this utility — no safelist entry needed.

---

### `-webkit-tap-highlight-color: transparent` — Tap Flash Suppression

**What it does:** Suppresses the default blue/gray flash that WebKit/Chrome mobile show when an
element is tapped. The flash is distracting in rapid-tap POS interactions (product grid, numpad).
Non-standard but universally supported in WebKit and Chrome Android.

**Pattern:** Apply globally at the `ferro-base.css` level with a base layer rule, or inline on
each POS interactive element. The global approach is cleaner for app-like workspaces:

```css
/* Add to input.css @layer base or as a utility: */
[data-ferro-pos-interactive] {
  -webkit-tap-highlight-color: transparent;
}
```

Then implement the press state via `:active` (see below) as the explicit, designed feedback
instead of the browser default flash.

**Integration:** Add a base rule to `input.css` scoped to POS interactive elements. Tailwind does
not generate this as a utility class — it must be a raw CSS rule in `input.css`.

---

### `:active` Press States — Touch Press Feedback

**Why `:hover` is wrong on touch:** `:hover` on touch devices fires after tap completion and
persists until the next tap elsewhere — it does not represent "finger is down." On a POS tile
grid, `:hover` causes a persistent highlight that looks like a stuck state. Use `:active` for
press feedback; it fires on `pointerdown`/`touchstart` and releases on lift.

**Correct press feedback for POS tiles:**

```
active:scale-[0.97] active:brightness-95
```

Or, using the existing token vocabulary:

```
active:bg-primary/80   (for primary-colored buttons)
active:bg-border       (for surface-colored tiles)
```

The `scale` approach provides immediate kinetic feedback; the `bg-*` approach matches the
hover pattern already used in `ProductTile` (`hover:bg-border`).

**Media query consideration:** For non-touch contexts (mouse+keyboard admin use), `:hover`
states remain useful. The correct approach is NOT to remove `:hover`, but to ensure `:active`
is always defined alongside it and provides at least equal visual weight.

**Integration:** Add `active:*` variants alongside every `hover:*` class in new POS components.
Tailwind generates `active:` variants from literal class strings — ensure they appear in Rust source
so the scanner picks them up. If dynamically constructed via `format!()`, add to the `@source
inline()` safelist in `input.css`.

---

### `user-select: none` — Accidental Text Selection Prevention

**What it does:** Prevents long-press text selection on buttons and tiles. On a POS numpad or
product grid, a prolonged touch triggers browser text selection rather than a button press —
especially on Android Chrome. `user-select: none` disables this.

**Tailwind utility:** `select-none`

**Apply to:** All numpad keys, product tiles, cart quantity controls, category navigation tabs.
Do NOT apply globally — form inputs and text display areas need selection.

**Integration:** Add `select-none` to the class strings of all new POS interactive components
in the Rust renderer. Already in Tailwind's core utilities — no safelist needed.

---

### `overscroll-behavior: contain` — Scroll Chain Prevention

**What it does:** Prevents scroll events in a child scrollable pane from propagating to the
parent viewport. Without this, scrolling the product grid on a fill_viewport POS page triggers
pull-to-refresh or viewport scroll, breaking the kiosk feel.

**Value choice:**
- `contain`: prevents chaining but preserves bounce/rubber-band within the element. Correct for
  a pane with genuine content to scroll.
- `none`: prevents chaining AND removes bounce. Use if rubber-band feels wrong in the POS context
  (e.g., the product pane never scrolls far enough to expose whitespace).

**Tailwind utilities:** `overscroll-contain` / `overscroll-none`

**Apply to:** The scrollable product grid pane and the cart line-item list within the
`fill_viewport` layout. Both are `overflow-y-auto` containers.

**Browser support:** Broadly supported including Safari (confirmed via MDN).

**Integration:** Add `overscroll-contain` to the Grid pane containers in the POS layout. This
is a new Tailwind utility in Rust source — picked up by the Tailwind scanner automatically.

---

### Input `font-size` ≥ 16px — iOS Auto-Zoom Prevention

**What it does:** iOS Safari auto-zooms into any `<input>` with `font-size < 16px` on focus.
On a POS numpad, this zoom is unexpected and disrupts the fill_viewport layout.

**Rule:** Every `<input>` element rendered inside POS components must use `font-size: 16px` or
larger. The numpad writes to a `<input type="hidden">` (no focus issue), but any visible
search-box or quantity-edit input must respect this floor.

**Tailwind utility:** `text-base` (which maps to `font-size: 1rem = 16px` by default in
Tailwind v4). Use `text-base` as the minimum on any visible input in a POS context.

**Design-lint candidate:** Encode as an Info-level rule in `design::lint`: any `Input`
component in a `fill_viewport: true` spec with font-size implied below 16px → warning.

---

## Hit Target Standards

| Standard | Requirement | Level |
|----------|-------------|-------|
| WCAG 2.5.8 (WCAG 2.2) | 24×24 CSS px minimum, or 24px spacing from adjacent targets | AA (required) |
| WCAG 2.5.5 (WCAG 2.1) | 44×44 CSS px | AAA (best practice for POS) |
| Apple Human Interface Guidelines | 44×44 points | Platform recommendation |
| Material Design 3 | 48×48 dp | Platform recommendation |

**POS recommendation:** 48×48 CSS px minimum for all tap targets (product tiles, numpad keys,
category tabs, cart controls). The existing `ProductTile` uses `min-h-[44px] min-w-[44px]`
(WCAG 2.5.5 AAA / Apple HIG). New components should meet or exceed this baseline; numpad keys
(large single-character buttons) should target 56×56 or larger since they are the primary input
surface.

**Rationale:** Research from the University of Maryland (cited in WCAG 2.5.8 understanding docs)
shows error rates 3× higher for targets below 44px. POS environments add stress factors (gloved
hands, split attention, counter vibration) that push this toward the 48-56px range.

**Design-lint encoding:** Add a POS-context lint rule: any interactive component in a spec that
includes a `ProductGrid`, `CartPanel`, or `Numpad` should warn if any button-class element has an
inferred height/width below 44px (checking `size: sm` in the component schema, which maps to
`h-8` = 32px in the existing size enum).

---

## Vanilla-JS Patterns

### Numpad Module (`runtime/numpad.rs`)

**Architecture:** Follow the existing `product_tiles.rs` / `kanban.rs` module pattern — a single
`setupNumpad()` function initialized in `ferroRuntime()`.

**Data model:**

```html
<!-- Emitted by render_numpad() -->
<div data-numpad data-numpad-target="qty_field">
  <div data-numpad-display class="...">0</div>
  <button type="button" data-numpad-key="1">1</button>
  ...
  <button type="button" data-numpad-key="backspace">⌫</button>
  <button type="button" data-numpad-key="clear">C</button>
</div>
<input type="hidden" name="qty_field" data-numpad-input="qty_field" value="0">
```

**Event delegation:** One `click` listener on each `[data-numpad]` container. `event.target.
closest('[data-numpad-key]')` handles taps on child elements (e.g., icon spans inside buttons).

```javascript
function setupNumpad() {
    var pads = document.querySelectorAll('[data-numpad]');
    for (var i = 0; i < pads.length; i++) {
        initNumpad(pads[i]);
    }
}

function initNumpad(pad) {
    var target = pad.getAttribute('data-numpad-target');
    var display = pad.querySelector('[data-numpad-display]');
    var input = document.querySelector('[data-numpad-input="' + target + '"]');
    var current = '';

    pad.addEventListener('click', function(e) {
        var btn = e.target.closest('[data-numpad-key]');
        if (!btn) return;
        var key = btn.getAttribute('data-numpad-key');
        if (key === 'clear') { current = ''; }
        else if (key === 'backspace') { current = current.slice(0, -1); }
        else if (/^\d$/.test(key)) { current = current === '0' ? key : current + key; }
        display.textContent = current || '0';
        if (input) { input.value = current || '0'; input.dispatchEvent(new Event('input', {bubbles:true})); }
    });
}
```

**Server round-trip decision:** No optimistic fetch. The numpad writes to a hidden form field.
The cart total updates in JS (DOM arithmetic). The POST happens once, when the operator taps
the submit/confirm button. This is the correct pattern for a ferro server-rendered app: the
form is the contract; intermediate cart state is client-transient. Round-tripping each quantity
change would add server latency to what should be instant tactile feedback.

---

### Cart Runtime Module (`runtime/cart_runtime.rs`)

**Responsibility:** Recompute line totals and cart total whenever any qty input changes.

**Data model:**

```html
<!-- Emitted per cart line -->
<tr data-cart-line data-unit-price="1250">  <!-- price in cents, integer -->
  <td>Product Name</td>
  <td data-qty-display="line_1">1</td>
  <td data-line-total="line_1">€12,50</td>
  <input type="hidden" name="line_1_qty" data-qty-input="line_1" value="1">
</tr>
<!-- Cart footer -->
<tfoot data-cart-total></tfoot>
```

**Arithmetic:** Use integer cents throughout — multiply `qty × unit_price_cents`, sum, then
format on display. Never use floating-point arithmetic for money.

**Event delegation:** Attach one `input` listener to the cart container. Triggered by the
`input` event dispatched in `product_tiles.rs` or `numpad.rs` after each qty change.

---

### Barcode Scanner (Keyboard Wedge) Module (`runtime/barcode_scanner.rs`)

**How wedge scanners work:** The scanner emulates a USB HID keyboard. Each scan produces a rapid
burst of `keydown` events — typically < 20ms between characters — followed by a `keydown` for
`Enter` (keyCode 13). Human typing is slower (> 80ms between keystrokes); this timing gap is the
detection heuristic.

**No library.** onscan.js (the most-cited library) adds a JS dependency and ~10KB for ~40 lines
of logic. The pattern is implemented inline.

**Pattern:**

```javascript
function setupBarcodeScanner() {
    var scanEl = document.querySelector('[data-barcode-input]');
    if (!scanEl) return;  // page doesn't include the component

    var buffer = '';
    var lastKeyTime = 0;
    var SCAN_TIMEOUT_MS = 100;
    var MAX_BETWEEN_CHARS_MS = 50;

    document.addEventListener('keydown', function(e) {
        var now = Date.now();
        var gap = now - lastKeyTime;
        lastKeyTime = now;

        // Long gap = human typing or new scan start; reset buffer
        if (gap > SCAN_TIMEOUT_MS && buffer.length > 0) { buffer = ''; }

        if (e.key === 'Enter' && buffer.length >= 4) {
            // Minimum 4 chars avoids triggering on keyboard Enter presses
            var code = buffer;
            buffer = '';
            scanEl.value = code;
            scanEl.dispatchEvent(new Event('input', {bubbles: true}));
            scanEl.dispatchEvent(new Event('change', {bubbles: true}));
            e.preventDefault();
        } else if (e.key.length === 1 && gap < MAX_BETWEEN_CHARS_MS) {
            // Only buffer if arrival is fast (scanner, not human)
            buffer += e.key;
        }
    });
}
```

**Integration with the POS page:** The `data-barcode-input` attribute is on a hidden (or
visually-prominent search) input. The `change` event triggers a form submission or a fetch
lookup for the product. The component emits `<input data-barcode-input name="barcode" …>`; the
runtime wires the scanner.

**Caveat:** The `MAX_BETWEEN_CHARS_MS = 50` threshold works for USB HID scanners. Bluetooth
scanners may have higher latency (~80-100ms between chars) and may misdetect. The threshold
should be a `data-barcode-max-gap="80"` attribute so per-instance tuning is possible without
code changes.

---

## Integration with the Existing Pipeline

### `input.css` Safelist Additions

New POS classes that are dynamically constructed (via `format!()` in Rust renderers) and will not
be detected by the Tailwind literal scanner:

```css
/* Add to @source inline() in input.css */
@source inline("active:scale-[0.97] active:brightness-95 active:bg-border active:bg-primary/80
                overscroll-contain overscroll-none select-none touch-manipulation
                grid-rows-1 grid-rows-2 grid-rows-3
                min-h-[48px] min-w-[48px] min-h-[56px] min-w-[56px]");
```

If `active:scale-[0.97]` uses an arbitrary value, verify it generates correctly; the literal
must appear in source or be in the safelist. Prefer `active:scale-95` (built-in Tailwind step)
over arbitrary values if the difference is imperceptible.

### `runtime/mod.rs` Pattern

Each new module (`numpad.rs`, `cart_runtime.rs`, `barcode_scanner.rs`) follows the exact existing
pattern:
1. File: `pub(super) const SOURCE: &str = r#"..."#;`
2. `mod.rs`: `mod numpad; mod cart_runtime; mod barcode_scanner;`
3. `FERRO_RUNTIME_JS`: push the three new sources into the assembled string
4. `ferroRuntime()` dispatcher: add `setupNumpad();`, `setupCartRuntime();`, `setupBarcodeScanner();`
5. Tests: add `bundle_contains_all_setup_functions` assertions for the new names

### Drift Guard Count

Every new builtin component added to the catalog requires updating the integer count in three
locations (per the existing lockstep):
1. `ferro-json-ui/src/catalog.rs` — `BUILTIN_TYPES` constant or equivalent drift guard
2. `ferro-mcp/src/tools/json_ui_catalog.rs` — the mirror count in the MCP tool
3. `ferro-json-ui/assets/ferro-base.css` — regenerated by `gen-ferro-base-css.sh`

Current builtin count is 47 (post v16.5). Every new POS component increments this.

---

## What NOT to Add

| Avoid | Why | What to Use Instead |
|-------|-----|---------------------|
| Gesture library (Hammer.js, ZingTouch) | No swipe gestures needed; tap, press, scroll handled by CSS + click events | CSS `touch-action` + `click` events |
| Virtual keyboard library (mobiscroll, onscreen-keyboard.js) | 40-300KB for functionality replaceable in ~80 lines of vanilla JS | `runtime/numpad.rs` module |
| onscan.js or barcode-scan-js | Library dependency for ~40 lines of timing logic | `runtime/barcode_scanner.rs` module |
| Pointer Events API (`pointerdown`/`pointerup`) for tap detection | Overkill — `click` fires correctly on touch when `touch-action: manipulation` eliminates the 300ms delay | `click` event + `touch-action: manipulation` |
| `user-scalable=no` / `maximum-scale=1` in `<meta name="viewport">` | WCAG 1.4.4 violation; breaks accessibility zooming | `touch-action: manipulation` per element |
| `touchstart` / `touchend` event listeners for basic tap | Superseded by `click` + pointer-events; adds complexity for no gain | `click` with `touch-action: manipulation` |
| CSS scroll snap on the product grid | Adds positional constraints that fight free-scroll; product grids are not paginated | Standard `overflow-y-auto` + `overscroll-contain` |
| Optimistic fetch on every cart quantity change | Server round-trip per tap adds latency; cart is transient until final POST | Client-side DOM arithmetic; POST only on submit |
| Arbitrary `@font-display` or font loading for the numpad display | Token vocabulary already includes `--font-display`; use `font-display` utility | Existing `font-display` token |

---

## Standards Numbers for Design-Lint Encoding

These numbers should be encoded as `design::lint` rules or component schema defaults in the
milestone phases:

| Concern | Number | Source | Severity |
|---------|--------|--------|----------|
| Minimum hit target | 44×44 CSS px | WCAG 2.5.5 AAA / Apple HIG | Warn if POS component size enum resolves to `h-8` (32px) |
| Preferred POS hit target | 48×48 CSS px | Material Design 3 / WCAG 2.5.8 guidance | Info if POS tile is exactly 44px but context is POS |
| Input font-size floor | 16px (= `text-base`) | iOS Safari auto-zoom behavior | Warn on any visible Input in a fill_viewport POS spec |
| Adjacent target spacing | 24 CSS px | WCAG 2.5.8 AA | Structural — enforce via padding in component CSS, not lint |
| `touch-action: manipulation` | Required on all interactive POS elements | Eliminates 300ms delay | Structural — enforce in component Rust renderer, not lint |

---

## Alternatives Considered

| Recommended | Alternative | Why Not |
|-------------|-------------|---------|
| Client-side cart arithmetic (DOM) | Fetch-based optimistic cart update | Server latency visible on rapid qty changes; complexity for no UX gain |
| Inline `setupBarcodeScanner()` (40 lines) | onscan.js library | External dependency; timing constants are tunable via `data-` attributes |
| `touch-action: manipulation` | `touch-action: none` | `none` not supported on iOS Safari |
| `:active` press states | `:hover` for touch feedback | `:hover` is sticky on touch; `:active` fires on `pointerdown` |
| `overscroll-contain` on panes | `overscroll-none` | `contain` preserves the within-pane bounce affordance; `none` feels abrupt for long product lists |

---

## Sources

- MDN `touch-action` — property values, `manipulation` behavior, double-tap zoom removal (HIGH)
- caniuse `mdn-css_properties_touch-action_manipulation` — iOS Safari support from 9.3 (HIGH)
- W3C WCAG 2.5.8 Understanding doc — 24×24px AA requirement, five exceptions (HIGH)
- W3C WCAG 2.5.5 Understanding doc — 44×44px AAA requirement (HIGH)
- MDN `-webkit-tap-highlight-color` — suppression via `transparent` (MEDIUM — non-standard)
- MDN `overscroll-behavior` — `contain` vs `none`, scroll chaining prevention (HIGH)
- defensivecss.dev — iOS input font-size 16px zoom prevention (MEDIUM — community, matches Apple behavior)
- `ferro-json-ui/src/render/atoms.rs` line 1373 — existing `touch-manipulation` on ProductTile (HIGH — in-codebase)
- `ferro-json-ui/src/runtime/product_tiles.rs` — existing qty-inc/dec/display/input data-attribute pattern (HIGH — in-codebase)
- `ferro-json-ui/src/runtime/mod.rs` — IIFE module assembly pattern (HIGH — in-codebase)
- `ferro-json-ui/assets/input.css` — `@source inline()` safelist pattern, `fill_viewport` chain (HIGH — in-codebase)
- `ferro-json-ui/src/render/classes.rs` — `INTERACTIVE_BASE`, `MOTION_FAST`, `FOCUS_RING` fragments (HIGH — in-codebase)
- 253-FRICTION.md — concrete gestiscilo cassa requirements driving this milestone (HIGH — first-party)
- GitHub axenox/onscan.js — barcode scanner detection timing heuristics (MEDIUM — third-party library used as reference only)

---

*Stack research for: ferro v16.6 POS Component Suite*
*Researched: 2026-07-04*
