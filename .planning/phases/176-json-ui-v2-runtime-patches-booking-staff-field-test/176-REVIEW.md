---
phase: 176-json-ui-v2-runtime-patches-booking-staff-field-test
reviewed: 2026-05-21T00:00:00Z
depth: standard
files_reviewed: 4
files_reviewed_list:
  - ferro-json-ui/src/component.rs
  - ferro-json-ui/src/render/containers.rs
  - ferro-json-ui/src/catalog.rs
  - docs/src/json-ui/components.md
findings:
  critical: 0
  warning: 0
  info: 2
  total: 2
status: clean
---

# Phase 176: Code Review Report

**Reviewed:** 2026-05-21
**Depth:** standard
**Files Reviewed:** 4
**Status:** clean (2 Info items — non-blocking observations)

## Summary

Phase 176 adds two `Option<String>` slots (`subtitle`, `badge`) to `CardProps`, extends `render_card` to emit them, refreshes the catalog description, documents the slots in `components.md`, and adds three Grid visibility regression tests against the no-repro outcome from F9.

All four review axes are clean:

- **XSS:** every new render slot routes through `html_escape` (containers.rs:74, 86). The escape function (render/mod.rs:254-260) covers `&`, `<`, `>`, `"`, `'` — sufficient for HTML text-content and double-quoted attribute contexts. New tests do not include hostile inputs but `html_escape` is the same function the title/description slots have used since v9; the new slots inherit that contract.
- **Optional handling:** `subtitle` and `badge` use `#[serde(default, skip_serializing_if = "Option::is_none")]` matching the existing `description` pattern. Round-trip tests confirm Some/None deserialize and reserialize symmetrically (`card_props_round_trips_badge`, `card_props_round_trips_subtitle`, `card_props_omits_empty_badge_in_json`, `card_props_omits_empty_subtitle_in_json`). Schema-presence tests confirm both fields surface in `schema_for!(CardProps)` output.
- **Test coverage:** both Some and None paths are exercised at the render layer (`render_card_emits_badge_when_present` / `..._omits_badge_when_absent`, `render_card_emits_subtitle_when_present` / `..._omits_subtitle_when_absent`) and at the serde layer. The combined `render_card_emits_title_subtitle_description_badge_together` test additionally asserts the title → subtitle → description ordering, which is the load-bearing visual claim in the spec.
- **Doc accuracy:** the `Card` section's slot list and the prose line *"Vertical stacking: title → subtitle → description"* match the actual render order in containers.rs:65-94. The table column ordering (title, description, subtitle, badge) matches the struct field order in `CardProps`, which is the conventional reading order; the rendering order is documented separately and correctly in prose.
- **Backward compatibility:** `CardProps` gained two `Option<String>` fields with `#[serde(default)]`. Existing controllers emitting `CardProps` JSON without these keys continue to deserialize into `None`, and existing render output for cards lacking these slots is byte-identical to the pre-Phase 176 output (the `if let Some` guards leave the no-badge / no-subtitle paths untouched).

The Grid visibility regression tests (`grid_renders_when_visible_true`, `grid_hidden_when_visible_false`, `grid_visible_consumer_reproduction`) lock in the no-repro Outcome A from F9. The consumer-reproduction test mirrors the chip-strip topology from the gestiscilo `calendar_day.json` spec and asserts both the visible and hidden branches against `/has_staff`.

## Info

### IN-01: Card badge pill duplicates Badge atom chrome inline

**File:** `ferro-json-ui/src/render/containers.rs:73`
**Issue:** The Card's badge pill emits the full Badge atom chrome inline:
```
inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium bg-secondary/10 text-secondary-foreground shrink-0
```
This duplicates the class string built by `render_badge` (atoms.rs:254-258) for `BadgeVariant::Secondary`. If the Badge atom's chrome ever evolves (e.g. spacing tweak, new token), the Card's inline pill silently diverges.

**Fix (optional, out of Phase 176 scope):** Consider extracting a `badge_classes(BadgeVariant)` helper that both `render_badge` and `render_card` consume, or have `render_card` synthesize and recursively render a `Badge` child element. Neither is required for v1; the current duplication is intentional per the Phase 176 PLAN ("Badge-styled pill rendered inline within Card chrome"). Recorded only so a future refactor knows where the second copy lives.

### IN-02: Pre-existing doc inconsistency on Grid gap_size

**File:** `docs/src/json-ui/components.md:182`
**Issue:** The Grid section lists `gap` valid values as `"none", "xs", "sm", "md", "lg", "xl"` but the shared `gap_size` enum at line 68 omits `"xs"`. This predates Phase 176 (verified against `d135caf9` baseline) and is not in this review's scope. Flagged here only because the file was edited in Phase 176 and the inconsistency may surface in adjacent doc passes.

**Fix:** Either add `"xs"` to the shared enum row at line 68, or drop `"xs"` from the Grid section at line 182. The `GapSize` enum in code (`containers.rs:697-703`) only has `None`, `Sm`, `Md`, `Lg`, `Xl` — so the Grid section's `"xs"` claim is the inaccurate one.

---

_Reviewed: 2026-05-21_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
