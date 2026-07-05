---
phase: 254-props-contracts-touch-foundation-design-rules
reviewed: 2026-07-05T01:49:38Z
depth: standard
files_reviewed: 9
files_reviewed_list:
  - docs/src/design-system/patterns.md
  - ferro-json-ui/assets/ferro-base.css
  - ferro-json-ui/assets/input.css
  - ferro-json-ui/src/component.rs
  - ferro-json-ui/src/design/rules.rs
  - ferro-json-ui/src/render/atoms.rs
  - ferro-json-ui/src/render/classes.rs
  - ferro-json-ui/src/render/mod.rs
  - ferro-mcp/src/tools/json_ui_catalog.rs
findings:
  critical: 0
  warning: 2
  info: 5
  total: 7
status: issues_found
---

# Phase 254: Code Review Report

**Reviewed:** 2026-07-05T01:49:38Z
**Depth:** standard
**Files Reviewed:** 9
**Status:** issues_found

## Summary

Reviewed the Phase 254 contracts-only change set (commits `86541417..HEAD`): additive `ProductTileProps` fields + five unregistered POS Props structs + `GridProps.row_weights` in `component.rs`; six POS touch constants with drift guards in `render/classes.rs` and the `render_product_tile` migration in `render/atoms.rs`; four POS design-lint rules with 12 fixtures in `design/rules.rs`; the matching `patterns.md` sections and `RULE_COMPONENTS` entries in `json_ui_catalog.rs`; plus the `pos-tap-highlight` `@utility` in `input.css` and the regenerated `ferro-base.css`.

**All locked constraints verified:**

- **No raw palette classes** — `POS_PRESS_ACTIVE` uses `active:bg-border` (token); asserted by `pos_constants_are_full_literals_and_token_compliant`.
- **Full Tailwind literals** — all six constants are complete class literals; equality-asserted in tests.
- **Legacy ProductTile render byte-identical** — verified by inspection: `POS_TOUCH_ACTION` and `POS_HIT_TARGET_MIN` are string-equal to the prior inline literals, and `categories_attr` is the empty string when `categories` is empty, so the legacy output byte stream is unchanged (but see IN-01 on test strength).
- **RULE_COMPONENTS references only existing builtins** — new entries map to `Grid` (a builtin) or `&[]`; the bidirectional drift guard (`design_system_component_guidance_drift_guarded`) enforces registry↔mapping parity plus builtin-membership in both directions.
- **Component count stays 47** — `test_all_components_present` still asserts 47; no `BUILTIN_TYPES`/catalog change in the phase diff.
- **New lint rules are `Severity::Warning` with internal presence gates** — all four rules use `intents: &[]` with in-check gates (`POS_TRIGGER_TYPES` presence, `fill_viewport`, root-Grid, `ProductGrid` presence respectively) and emit `Severity::Warning`.
- **HTML-escaping on `data-product-categories`** — `html_escape` (escapes `&`, `<`, `>`, `"`, `'`) is applied to the joined value inside a double-quoted attribute; covered by `product_tile_escapes_categories`.
- **ferro-base.css (generated) anomaly check** — all nine new utilities present exactly once (`.touch-manipulation`, `.min-h-\[44px\]`, `.min-w-\[44px\]`, `.min-h-\[56px\]`, `.min-w-\[56px\]`, `.active\:scale-95`, `.active\:bg-border`, `.overscroll-contain`, `.pos-tap-highlight`); the unused-until-256 `min-h-[56px]` variants are correctly picked up from the `classes.rs` literal via the `@source "../../ferro-json-ui/src"` scan. No anomalies.

Two warnings concern the durability of the new data contract and a lint false-positive class this codebase has previously treated as a bug. The remaining findings are minor.

## Warnings

### WR-01: Space-separated `data-product-categories` contract silently corrupts category names containing spaces

**File:** `ferro-json-ui/src/component.rs:1360-1366` and `ferro-json-ui/src/render/atoms.rs:1372-1379`
**Issue:** `render_product_tile` emits categories as `props.categories.join(" ")` into `data-product-categories`, and the field doc declares the attribute "space-separated". A category name containing a space — highly plausible for the target consumer (Italian labels like `"Bevande calde"`) — is indistinguishable from two separate categories once joined. The Phase 255 `setupPosFilter` runtime that splits on spaces will silently mis-filter: `"Bevande calde"` becomes tokens `Bevande` and `calde`, and a `CategoryNav` tab labeled `"Bevande calde"` will never match. Nothing in the Props contract, spec validation, or the doc comment constrains category names to be space-free. Because Phase 254 locks this contract for Phases 255-257, the ambiguity ships into the wire format.
**Fix:** Either (a) document and enforce the constraint now — add "category names must not contain spaces" to the `categories` doc comment and normalize at render time, e.g.:
```rust
let categories_attr = /* ... */ format!(
    " data-product-categories=\"{}\"",
    html_escape(
        &props.categories.iter()
            .map(|c| c.replace(' ', "-"))
            .collect::<Vec<_>>()
            .join(" ")
    )
);
```
(with the same normalization specified for `CategoryNavProps.items` matching in the Phase 255 handoff), or (b) switch the attribute encoding to something collision-free (e.g. a JSON array string) before the contract is consumed. Option (a) is smaller and keeps the CSS-selector-friendly token list.

### WR-02: `pos-grid-fill` false-positives on a `$data`-bound `fill` prop

**File:** `ferro-json-ui/src/design/rules.rs:463-489`
**Issue:** `check_pos_grid_fill` accepts only a literal `true`: `props.get("fill").and_then(|v| v.as_bool()).unwrap_or(false)`. Lint runs on the pre-resolve spec, where any prop may legitimately be `{"$data": "/..."}`; for a data-bound `fill` the `as_bool()` returns `None` and the rule warns even when the runtime value is `true`. This is the exact false-positive class this registry has already fixed twice — the `list-empty-state` and `breadcrumb-on-subpages` regression guards (test comments "WR-01"/"WR-02") both moved to non-null acceptance for this reason. The phase's own `pos_grid_fill_data_bound_no_misfire` fixture covers a `$data`-bound *child* prop but not the gated `fill` prop itself, so the gap is untested.
**Fix:** Accept any non-null, non-`false` value, consistent with the sibling rules:
```rust
let fill_set = root
    .props
    .get("fill")
    .map(|v| !v.is_null() && v.as_bool() != Some(false))
    .unwrap_or(false);
```
and add a fixture with `"fill": {"$data": "/ui/fill"}` asserting 0 findings.

## Info

### IN-01: `product_tile_legacy_render_is_byte_identical` does not assert byte-identity

**File:** `ferro-json-ui/src/render/atoms.rs:2551-2570`
**Issue:** The test name (and the phase's locked constraint) promise byte-identity for the legacy render, but the assertions are `contains(POS_TOUCH_ACTION)`, a count of `POS_HIT_TARGET_MIN`, and absence of `data-product-categories`. Since the assertions reference the same constants the render uses, a drifted constant value would still pass. Byte-identity currently holds (verified by inspection), but the guard is weaker than its name.
**Fix:** Assert against a hard-coded golden string for the legacy tile HTML (or at minimum assert the class attribute contains the literal `"touch-manipulation"` / `"min-h-[44px] min-w-[44px]"` strings, not the constants), or rename the test to match what it checks.

### IN-02: POS literal drift guard has evasion gaps

**File:** `ferro-json-ui/src/render/classes.rs:83-108`
**Issue:** Three gaps in `pos_render_functions_use_constants_not_literals`: (1) `read_dir` is non-recursive, so a Phase 256 render file placed in a subdirectory (e.g. `src/render/pos/`) is silently skipped despite the comment claiming "Auto-covers Phase 256 render files"; (2) the guarded literal is the joint string `"min-h-[44px] min-w-[44px]"` — a file inlining `min-h-[44px]` alone or in reversed order evades it; (3) the `POS_PRESS_ACTIVE`, `POS_OVERSCROLL_CONTAIN`, and `POS_TAP_HIGHLIGHT` literals are not guarded at all.
**Fix:** Guard the individual tokens (`"min-h-[44px]"`, `"min-w-[44px]"`, `"min-h-[56px]"`, `"min-w-[56px]"`, `"overscroll-contain"`, `"pos-tap-highlight"`, `"active:scale-95"`) and walk the directory recursively (or assert `src/render` stays flat).

### IN-03: New POS Props types not re-exported at crate root; `render::classes` made public without docs

**File:** `ferro-json-ui/src/lib.rs:50-63` and `ferro-json-ui/src/render/mod.rs:25`
**Issue:** Every existing Props type (including `ProductTileProps`, `GridProps`) is flat re-exported from the crate root, but the five new POS Props structs and `NumpadMode` are reachable only via `ferro_json_ui::component::...` — an inconsistent public surface for agents following the established convention. Separately, `render::classes` was widened from `pub(crate)` to `pub` (exposing the six POS constants as public API) with no mention in `docs/src/`. Both may be deliberate deferrals to Phase 256 registration, but neither deferral is recorded.
**Fix:** Either add the re-exports and a one-line docs note now, or record the deferral explicitly in the Phase 256 handoff so the inconsistency cannot ship in a release between phases.

### IN-04: `GridProps.row_weights` doc says weights align with `children`, but they are per-row

**File:** `ferro-json-ui/src/component.rs:906-913`
**Issue:** The doc comment reads "Per-row height weights … Positional alignment with `children`". For a multi-column fill grid, rows and children are different sequences (rows = ceil(children / columns)), so "positional alignment with children" contradicts "per-row". Since the Phase 256 renderer will implement `grid-template-rows` from this contract, the ambiguity is exactly where a mis-implementation would start.
**Fix:** Reword to state alignment with grid rows, e.g.: "Positional alignment with grid rows (row i takes `row_weights[i]`; rows beyond the vec default to weight 1)."

### IN-05: patterns.md POS example has orphan elements; pre-existing missing `---` separator

**File:** `docs/src/design-system/patterns.md:626-634` and `docs/src/design-system/patterns.md:480`
**Issue:** (1) The `pos-cart-present` conforming example declares `grid` and `cart` elements that are never referenced as children of the root Grid — the spec passes lint (flat-map scan) but renders an empty Grid if an agent copies it verbatim; patterns.md examples are agent-facing generation context. (2) Pre-existing (Phase 253, out of this phase's diff but in-scope file): the `## prefer-components` section lacks the `---` separator every other section has.
**Fix:** (1) Add `"children": ["grid", "cart"]` to the root Grid in the conforming example. (2) Insert the missing `---` before `## prefer-components`.

---

_Reviewed: 2026-07-05T01:49:38Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
