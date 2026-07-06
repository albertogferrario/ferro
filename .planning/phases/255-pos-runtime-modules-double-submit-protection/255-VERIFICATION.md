---
phase: 255-pos-runtime-modules-double-submit-protection
verified: 2026-07-05T14:30:00Z
status: human_needed
score: 5/5
overrides_applied: 0
re_verification: false
human_verification:
  - test: "Open /cassa in a browser, click Conferma ordine once, verify the button becomes disabled (opacity-50 cursor-not-allowed). Attempt a second click and confirm the submit is prevented (no second POST to /cassa/conferma). Navigate back and forward to trigger bfcache restore; verify the button is re-enabled."
    expected: "First click disables the button; second click is a no-op (preventDefault); back/forward navigation re-enables via pageshow+persisted guard."
    why_human: "The setupFormGuards double-submit guard is JS runtime code executing in a browser; grep + source inspection confirm the code is correct (WR-01 fix verified) but the actual bind-and-fire sequence requires a live browser to confirm no DOM timing or rendering quirk breaks it. WR-01 was a real guard-binding failure found by code review; a live smoke-test closes the loop."
---

# Phase 255: POS Runtime Modules + Double-Submit Protection — Verification Report

**Phase Goal:** The catalog vocabulary is domain-neutral (operator decision 2026-07-05) and the POS runtime modules (numpad input, token/text tile filtering) are in the bundle with a stable data-attribute contract before any render function targets it; the double-submit guard is in place for selection-mutation forms. Scope boundary: NO cart-state JS — quantities accumulate in Tile hidden inputs and submit as a single confirm POST; the live CartRuntime is deferred.

**Verified:** 2026-07-05T14:30:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| SC-0 | Vocabulary neutralization is complete: ProductTile→Tile, POS_ prefix dropped, pos-* lint ids renamed to register-*, data-product- renamed to data-filter-, show_staff/show_people removed; global grep gate returns ZERO hits; component count stays 47 | VERIFIED | `grep -rn 'ProductTile\|product_tile\|setupProductTiles\|data-product-\|CartPanel\|CategoryNav\|ProductGrid' ferro-json-ui/src ferro-mcp/src app/src docs/src` returned 0 hits; `assert_eq!(BUILTIN_TYPES.len(), 47)` at catalog.rs:1219; all 5 struct renames confirmed in component.rs |
| SC-1 | `bundle_contains_all_setup_functions` test passes for setupNumpad and setupFilters | VERIFIED | Both "setupNumpad" and "setupFilters" present in the drift-list array in runtime/mod.rs:205-206; test passes (783-test suite green per REVIEW-FIX.md) |
| SC-2 | `dispatcher_invokes_every_setup` passes — ferroRuntime() calls setupNumpad() and setupFilters() exactly once; both are no-ops when elements absent | VERIFIED | runtime/mod.rs:62-64 shows setupNumpad and setupFilters in the try/catch setups array; numpad.rs: `if (pads.length === 0) return;`; filters.rs: `if (scopes.length === 0) return;`; drift test at mod.rs:229 passes |
| SC-3 | Numpad key taps write to the target hidden field and dispatch a bubbling input event; token/text filtering toggles tile visibility via data-filter-tokens + data-filter-text with no server round-trip; render_tile always emits HTML-escaped data-filter-text | VERIFIED | numpad.rs:57 `input.dispatchEvent(new Event('input', { bubbles: true }))` present; filters.rs reads data-filter-tokens + data-filter-text and sets `el.style.display`; atoms.rs render_tile unconditionally emits `data-filter-text="{name}"` (html_escape applied); SC-3 inline test `runtime_exposes_numpad_and_filter_contract` at mod.rs:288 asserts all key strings in bundle |
| SC-4 | The selection-mutation confirm button emits data-disable-on-submit; the runtime guard disables it after the first submission (submit-event bound, bfcache-safe); the idempotency-key pattern is documented with the framework::write hook | VERIFIED | component.rs: `pub disable_on_submit: Option<bool>`; atoms.rs: emits `" data-disable-on-submit"` when Some(true); form_guards.rs: `form.addEventListener('submit', ...)` with `e.defaultPrevented` guard (WR-02 fix) + `window.addEventListener('pageshow', ...)` bfcache reset; cassa.json: btn_confirm inside confirm_form (WR-01 fix); write-kernel.md: "Double-submit protection for forms" section with `idempotency_key` references |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-json-ui/src/component.rs` | TileProps (item_id), TileGridProps, SelectionPanelProps (no show_staff/show_people), FilterTabsProps, ButtonProps.disable_on_submit | VERIFIED | All 5 struct renames confirmed; 1 occurrence each; show_staff/show_people grep returns 0; disable_on_submit Option<bool> present |
| `ferro-json-ui/src/catalog.rs` | BUILTIN_SPECS entry Tile with schema_for!(TileProps); count 47 | VERIFIED | `schema_for!(TileProps)` and `"Tile"` present; count assertion at line 1219 stays 47 |
| `ferro-json-ui/src/render/atoms.rs` | render_tile emits data-filter-tokens (conditional) + data-filter-text (always, HTML-escaped); render_button emits data-disable-on-submit when Some(true) | VERIFIED | Both attributes confirmed; html_escape applied; tile_escapes_categories + tile_escapes_filter_text regression tests present; render_button_emits_disable_on_submit + omits test at lines 1692/1707 |
| `ferro-json-ui/src/render/classes.rs` | 6 neutral touch constants without POS_ prefix; value strings byte-identical | VERIFIED | 6 constants (TOUCH_ACTION, HIT_TARGET_MIN, HIT_TARGET_NUMPAD, PRESS_ACTIVE, OVERSCROLL_CONTAIN, TAP_HIGHLIGHT); no POS_ prefix grep hits; "touch-manipulation" value preserved |
| `ferro-json-ui/src/render/mod.rs` | Dispatch arm "Tile" => atoms::render_tile; BUILTIN_TYPES "Tile" | VERIFIED | `"Tile" => atoms::render_tile(el, spec, data, depth)` confirmed |
| `ferro-json-ui/src/runtime/tiles.rs` | setupTiles function (product_tiles.rs removed) | VERIFIED | tiles.rs exists with `function setupTiles()`; product_tiles.rs absent |
| `ferro-json-ui/src/runtime/numpad.rs` | ES5 setupNumpad (97 lines) — quantity + price modes, event delegation, bubbling input event, no-op guard | VERIFIED | 97 lines; `function setupNumpad`; data-numpad-key/target/input/display attrs; `bubbles: true`; `pads.length === 0` no-op; ES5 compliance confirmed (no arrows/let/const/template literals in JS content) |
| `ferro-json-ui/src/runtime/filters.rs` | ES5 setupFilters (119 lines) — AND matching, style.display toggle, semantic active-tab classes | VERIFIED | 119 lines; `function setupFilters`; data-filter-tokens + data-filter-text + data-filter-search; `el.style.display`; `border-primary` semantic classes; `scopes.length === 0` no-op |
| `ferro-json-ui/src/runtime/form_guards.rs` | Double-submit guard inside setupFormGuards; data-numpad-input in number guard; e.defaultPrevented check (WR-02); no new dispatcher entry | VERIFIED | `button[data-disable-on-submit]` querySelectorAll; `addEventListener('submit', ...)` at line 36; `e.defaultPrevented` check at line 40 (WR-02 fix present); `var numpadInputs = form.querySelectorAll('input[data-numpad-input]')` (D-05); `setupDisableOnSubmit` NOT in drift lists (D-13 preserved) |
| `ferro-json-ui/src/runtime/mod.rs` | mod numpad/filters; concat; dispatcher (try/catch setups array); both drift lists; SC-3 + SC-4 inline tests | VERIFIED | mod numpad + filters at lines 10/16; push_str at lines 42-43; try/catch setups array at line 62-70; drift lists at 205-207 + 243-245; SC-3 test at 288; SC-4 test at 278 |
| `ferro-json-ui/src/design/rules.rs` | register-fill-viewport / register-grid-fill / register-selection-present; REGISTER_TRIGGER_TYPES = ["TileGrid", "SelectionPanel", "Numpad"] | VERIFIED | 24 hits for register-* ids; REGISTER_TRIGGER_TYPES constant confirmed |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | Tile in expected list; register-* rule ids; count 47; no old tokens | VERIFIED | "Tile" at line 450; 3 hits for register-* ids; no ProductTile/pos-* hits |
| `app/src/views/cassa.json` | type "Tile" / item_id; confirm_form Form wrapping btn_confirm with disable_on_submit:true (WR-01 fix) | VERIFIED | "type": "Tile" at line 79; "item_id" at 82; confirm_form Form at line 57-68; btn_confirm with disable_on_submit:true at line 70 |
| `docs/src/json-ui/components.md` | Tile section; v16.6 migration note (no retired compound tokens) | VERIFIED | `### Tile` present; migration table uses "formerly the product-prefixed..." wording without literal ProductTile or data-product- strings |
| `docs/src/design-system/patterns.md` | register-fill-viewport / register-grid-fill / register-selection-present sections | VERIFIED | 8 hits for register-* ids; no old pos-* hits |
| `docs/src/features/write-kernel.md` | "Double-submit protection for forms" section with idempotency_key and enclosing Form guidance | VERIFIED | Section heading at line 156; idempotency_key at multiple lines; Form requirement documented ("The button must participate in a form...") |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| render/mod.rs dispatch | render/atoms.rs render_tile | "Tile" => atoms::render_tile | WIRED | Confirmed: `"Tile" => atoms::render_tile(el, spec, data, depth)` |
| catalog.rs | component.rs TileProps | schema_for!(TileProps) | WIRED | `|| to_value(schema_for!(TileProps)).unwrap()` confirmed |
| runtime/mod.rs | numpad.rs | s.push_str(numpad::SOURCE) | WIRED | `s.push_str(numpad::SOURCE)` at line 42 |
| runtime/mod.rs | filters.rs | s.push_str(filters::SOURCE) | WIRED | `s.push_str(filters::SOURCE)` at line 43 |
| numpad.rs | target hidden input | input.dispatchEvent(new Event('input', { bubbles: true })) | WIRED | Line 57 confirmed |
| filters.rs | rendered tiles | reads data-filter-tokens + data-filter-text, sets style.display | WIRED | All three attributes present in filters.rs source; style.display toggle confirmed |
| form_guards.rs | form submit event | form.addEventListener('submit', ...) | WIRED | Line 36 confirmed; e.defaultPrevented guard at line 40 (WR-02) |
| form_guards.rs | bfcache restore | window.addEventListener('pageshow', ...) with event.persisted | WIRED | Lines 20-27 confirmed |
| docs/write-kernel.md | framework::write dispatch_write idempotency hook | idempotency_key dedupe on (tenant_id, key) | WIRED | Multiple idempotency_key references confirmed |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| render_tile (atoms.rs) | props.name → data-filter-text | TileProps struct, field `name: String` | Yes — server-rendered from spec data | FLOWING |
| render_tile (atoms.rs) | props.categories → data-filter-tokens | TileProps struct, field `categories: Option<Vec<String>>` | Yes — conditional, HTML-escaped | FLOWING |
| render_button (atoms.rs) | props.disable_on_submit → data-disable-on-submit | ButtonProps field Option<bool> | Yes — emitted only when Some(true) | FLOWING |
| numpad.rs JS | field from data-numpad-target → writes hidden input | data-numpad-target attribute set by Phase 256 Numpad renderer | Deferred — renderer ships Phase 256; runtime is ready | WIRED (renderer deferred) |
| filters.rs JS | data-filter-text / data-filter-tokens from rendered tiles | Server-rendered via render_tile | Yes — render_tile always emits data-filter-text; data-filter-tokens conditional | FLOWING |

### Behavioral Spot-Checks

Step 7b: SKIPPED (runtime JS executes in a browser; inline-source tests in the Rust test suite are the designated verification method per ROADMAP SC-3/SC-4; the cargo test suite confirms the bundle contains the required strings and the dispatch pattern is correct).

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| SC-0 | 255-01, 255-02, 255-03 | Vocabulary neutralization — ProductTile→Tile, register-* lint ids, REGISTER_TRIGGER_TYPES, POS_ prefix dropped | SATISFIED | Global grep gate: 0 hits; all renamed artifacts confirmed |
| POS-08 | 255-04, 255-05 | POS forms double-submit protected — data-disable-on-submit runtime guard + documented idempotency-key pattern | SATISFIED | ButtonProps.disable_on_submit field; render_button emission; setupFormGuards guard block; write-kernel.md documentation |

**No orphaned requirements.** REQUIREMENTS.md maps POS-08 and SC-0 to Phase 255; both are satisfied.

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| `ferro-json-ui/src/component.rs` | Doc comments at lines ~1354, ~1395, ~1437, ~1465 still carry "product tile", "product array", "POS builtin" phrasing (IN-02 from REVIEW — info level, not fixed in scope) | Info | Cosmetic only; no compiler or runtime impact; does not affect any grep gate (these are doc comment strings, not identifiers) |
| `ferro-json-ui/src/runtime/numpad.rs` | `numpadQtyKey` and `numpadPriceKey` functions have identical bodies (IN-01 from REVIEW — info level, not fixed in scope) | Info | Readability; no correctness impact; comment should note deliberate duplication |
| `ferro-json-ui/src/runtime/form_guards.rs` | Number guard can visually re-enable a latched double-submit button on input event (IN-04 from REVIEW — info level, not fixed in scope) | Info | Edge case UX glitch (button looks clickable but every submit is preventDefault'd); not a data-loss or security issue; fix is add `if (submitBtn._submitted) return;` at top of check() |
| `ferro-json-ui/src/runtime/numpad.rs` | Key handler appends any non-clear/non-backspace key verbatim (IN-03 from REVIEW — info level) | Info | Authoring robustness; server re-validates; Phase 256 Numpad renderer controls key emission |

All anti-patterns are Info severity (from the REVIEW's IN-01..IN-04 findings). None are blockers. The three Warning-level findings (WR-01, WR-02, WR-03) were fixed in commit 31299b56, 68a8958d, and b27e4a78 respectively.

### Human Verification Required

#### 1. Form guard live-fires on /cassa confirm

**Test:** Open /cassa in a browser. Add items (or with any present). Click "Conferma ordine". Verify: the button transitions to disabled state (visually grayed, cursor-not-allowed). Click a second time. Verify: no second POST request is sent to /cassa/conferma (check browser DevTools Network tab). Navigate back via browser back button and then forward; verify the button is re-enabled (bfcache pageshow reset).

**Expected:** Single submission accepted; second blocked; bfcache restore re-enables the button.

**Why human:** The JS runtime guard (`initDisableOnSubmit`) binds dynamically at DOMContentLoaded via the form's submit event. The code is correct (WR-01 fixed — btn_confirm now inside confirm_form; WR-02 fixed — defaultPrevented check; WR-03 fixed — try/catch isolation), but the full DOM lifecycle (form present, button found via closest('form'), event listener fires, bfcache triggered) is browser-runtime behavior that can only be confirmed in a live browser. The code review found a real binding failure (WR-01) that code analysis alone initially missed; a live smoke-test closes the loop.

### Gaps Summary

No gaps. All five ROADMAP success criteria (SC-0 through SC-4) are satisfied by the codebase. The global SC-0 grep gate returns zero hits. The 783-test suite (confirmed by REVIEW-FIX.md after all three warning fixes were applied) includes the drift-guard tests, the XSS escaping regression tests, the disable_on_submit render tests, and the SC-3/SC-4 inline-source tests. One human verification item remains: live browser confirmation that the form guard fires correctly on /cassa after the WR-01 fix.

---

_Verified: 2026-07-05T14:30:00Z_
_Verifier: Claude (gsd-verifier)_
