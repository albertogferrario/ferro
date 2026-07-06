---
phase: 213-projection-render-completeness
plan: "04"
subsystem: json-ui
tags: [rust, ferro-json-ui, datatable, image, column-format, projection, xss]

requires:
  - phase: 213-03
    provides: StatCard value_path binding and gap C infrastructure
provides:
  - ColumnFormat::Image variant in ferro-json-ui component.rs
  - lookup_meaning(ImageUrl).column = Some(()) — ImageUrl fields now appear as DataTable columns
  - Image-format cell renderer in data.rs — html-escaped <img> thumbnail
affects: [ferro-json-ui consumers, gestiscilo staff browse page]

tech-stack:
  added: []
  patterns:
    - "ColumnFormat dispatch in render_cell: Image branch follows the Badge special-case pattern — early return before the default html_escape(&cell_string) path"
    - "XSS defense for URL-in-attribute: html_escape() on the URL before interpolation into src='' prevents attribute injection"

key-files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/projection/component_map.rs
    - ferro-json-ui/src/projection/builder.rs
    - ferro-json-ui/src/render/data.rs

key-decisions:
  - "Only ImageUrl's column marker flipped from None to Some(()). Identifier/ForeignKey/Sensitive remain column: None — security filter unchanged."
  - "Image branch returns empty string for null/empty URL rather than a broken <img src=''> — consistent with Badge null handling."
  - "html_escape() applied to URL before src attribute interpolation (T-213-09 mitigation)."

patterns-established:
  - "ColumnFormat::Image: unit variant, serde snake_case serializes as 'image'. Cell value is an image URL string."

requirements-completed: [GAP-D]

duration: 15min
completed: 2026-06-12
---

# Phase 213 Plan 04: ImageUrl DataTable Column Summary

**`ColumnFormat::Image` added; ImageUrl fields render as html-escaped `<img>` thumbnails in DataTable columns instead of being excluded**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-06-12T21:00:00Z
- **Completed:** 2026-06-12T21:03:15Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added `ColumnFormat::Image` variant to the `ColumnFormat` enum (component.rs) — additive, serde-compatible, serializes as `"image"`.
- Flipped `lookup_meaning(FieldMeaning::ImageUrl).column` from `None` to `Some(())` and added `FieldMeaning::ImageUrl => Some(ColumnFormat::Image)` arm in `build_column_for_field` — ImageUrl fields now pass the `emit_datatable_root` column filter.
- Added Image branch to `render_cell` in data.rs: URL html-escaped before `src` interpolation, null/empty returns empty string (no broken `<img>`).
- All security invariants preserved: Identifier/ForeignKey/Sensitive meanings remain `column: None`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add ColumnFormat::Image, include ImageUrl column, map its format** - `7ed12bd5` (feat)
2. **Task 2: Render an `<img>` for Image-format cells in data.rs** - `c20046ca` (feat)

## Files Created/Modified

- `ferro-json-ui/src/component.rs` — Added `Image` variant to `ColumnFormat` enum
- `ferro-json-ui/src/projection/component_map.rs` — `ImageUrl.column: Some(())` + `build_column_for_field` arm for `ImageUrl => Some(ColumnFormat::Image)`
- `ferro-json-ui/src/projection/builder.rs` — TDD tests: `datatable_root_includes_image_url_column`, `image_column_has_image_format`
- `ferro-json-ui/src/render/data.rs` — Image branch in `render_cell` + TDD tests: `renders_img_tag`, `xss_escaped`, `null_renders_empty_cell`

## Decisions Made

- Only `ImageUrl` column marker changed. All other meanings with `column: None` (Identifier, ForeignKey, Sensitive) remain excluded — the security filter is unchanged.
- Null Image cell returns empty string (consistent with Badge null path), not a broken `<img>`.
- XSS mitigation via `html_escape()` on the URL before `src` attribute interpolation (T-213-09).

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None. Both TDD RED/GREEN cycles were clean.

## Threat Surface Scan

No new network endpoints, auth paths, or schema changes introduced beyond the plan's documented threat model.

| Threat | Disposition | Notes |
|--------|-------------|-------|
| T-213-08: column filter integrity | mitigated | Only ImageUrl flipped; Sensitive/ForeignKey/Identifier remain excluded |
| T-213-09: XSS via src attribute | mitigated | `html_escape()` applied to URL before interpolation |

## Known Stubs

None. ImageUrl columns now render a real `<img>` tag from the cell URL value.

## User Setup Required

None.

## Next Phase Readiness

- Gap D (ImageUrl DataTable column) complete. All four content gaps (B/A/C/D) are now implemented.
- Plan 05 (Gap E documentation or phase wrap-up) can proceed.
- Gestiscilo staff browse page (`/dashboard/staff`) avatar_url column will render as an image thumbnail once ferro is rebuilt.

---
*Phase: 213-projection-render-completeness*
*Completed: 2026-06-12*
