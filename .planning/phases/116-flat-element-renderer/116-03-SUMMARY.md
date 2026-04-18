---
phase: 116-flat-element-renderer
plan: 03
subsystem: ui
tags: [json-ui, rendering, html, xss, atom-renderers, v12.0]

# Dependency graph
requires:
  - phase: 116-flat-element-renderer-02
    provides: walker scaffolding with 23 atom stubs, dispatch match, html_escape helper
provides:
  - 23 real atom renderer bodies ported verbatim from v1 render.rs
  - decode_props helper (null→{} bridge for Phase 115 serialization convention)
  - Button GET-action <a> wrap (v1 render_node semantics preserved)
  - D-16 url=None fallback with href="#" + diagnostic comment
  - Pagination current_page clamp to [1, total_pages]
  - XSS regression coverage (Image src, Text content, attribute breakout)
affects:
  - 116-06 (framework integration: real atom HTML available for golden tests)
  - 117-catalog (catalog validation: now knows every atom type_name round-trips)
  - 121-field-test (gestiscilo: UIs that rely on these atom classes render)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Atom renderer signature: (el: &Element, _spec: &Spec, _data: &Value, _depth: usize) -> String"
    - "Prop decode via decode_props(&el.props) — treats Null as empty object"
    - "D-12 diagnostic fallback: <!-- ferro-json-ui: failed to decode TYPE props: MSG -->"
    - "D-15/D-16 action URL resolution: html_escape(url) or href=\"#\" + diagnostic"
    - "html_escape applied to every interpolated user string (XSS discipline)"

key-files:
  created:
    - .planning/phases/116-flat-element-renderer/116-03-SUMMARY.md
  modified:
    - ferro-json-ui/src/render/atoms.rs  # 39 LOC stubs → 1849 LOC real bodies

key-decisions:
  - "decode_props helper bridges Null→{} so all-optional Props structs (Separator, Skeleton) decode when el.props is omitted"
  - "Button non-GET actions return bare <button> (v1 behavior — no data-action-* attributes added)"
  - "Sidebar/Header/NotificationDropdown nested items rendered inline from typed Props (not via render_element — items are not Spec elements)"
  - "Icon fields (StatCard.icon, SidebarNavItem.icon, ActionCard.icon) emitted raw per v1 — treated as trusted server-authored SVG"
  - "render_sidebar_nav_item helper retained as private fn (matches v1 structure)"

patterns-established:
  - "Leaf renderers never call render_element — atoms are leaves per RESEARCH §Atoms"
  - "Button's <a> wrap lives inside render_button (v1 put it in render_node; v2 has no render_node)"
  - "Pagination clamp applied after total_pages compute — defensive for unsanitized query input"

requirements-completed:
  - RENDER-01
  - RENDER-03

# Metrics
duration: ~25min
completed: 2026-04-18
---

# Phase 116 Plan 03: Flat Element Renderer - Atom Bodies Summary

**23 atom renderers ported verbatim from v1 render.rs into the new (el, spec, data, depth) walker signature, with D-12 prop-decode diagnostics, D-15/D-16 action URL handling, and XSS regression coverage.**

## Performance

- **Duration:** ~25 min (worktree build cache cold; atom port + 38 tests)
- **Completed:** 2026-04-18
- **Tasks:** 2 (merged into single commit — both tasks modified the same file and all atoms share the same decode_props helper)
- **Files modified:** 1 (`ferro-json-ui/src/render/atoms.rs`)

## Accomplishments

- All 23 atom renderers produce non-empty v1-byte-compatible HTML
- Button GET-action wrap-in-anchor semantics preserved (v1 render_node lines 251-286)
- Button URL=None falls back to `href="#"` + visible diagnostic comment (D-16)
- Pagination current_page clamps to [1, total_pages] — tested with 99→5
- XSS regression tests for Image (classic `"` break-out AND `javascript:alert('xss')` quote escape)
- XSS test for Text (`<script>` → `&lt;script&gt;`)
- 38 inline tests; full ferro-json-ui lib suite 250 passed (up from 212)
- Zero `stub_renderer!` invocations remaining in atoms.rs
- `grep -c "pub(crate) fn render_"` returns 23

## Task Commits

Both Task 1 (11 primitives + Pagination) and Task 2 (12 composites) landed in one commit because they live in the same file, share the `decode_props` helper, and share the test module helpers. Splitting into two commits would have created an intermediate state with 11 real renderers + 12 stubs that the tests couldn't assert against holistically.

1. **Task 1+2 (combined): Port 23 atom renderers from v1 render.rs** — `4ad293bf` (feat)

This commit contains:
- Text (v1 lines 1715-1733), Button (1734-1798, with render_node wrap 251-286), Badge (1799-1844)
- Alert (1845-1874), Separator (1875-1882), Progress (1883-1904), Avatar (1905-1935)
- Image (1936-1977), Skeleton (1978-1997), Breadcrumb (1998-2025), Pagination (2026-2106)
- DescriptionList (2107-2122), EmptyState (2185-2213), StatCard (2263-2297)
- Checklist (2298-2360), Toast (2361-2404), NotificationDropdown (2405-2471)
- Sidebar (2472-2535), Header (2536-2590), DropdownMenu (591-693)
- CalendarCell (355-418), ActionCard (419-470), ProductTile (471-497)

Plus 38 inline tests covering: per-type smoke (23), button action variants (4), pagination clamp/edge cases (3), XSS regressions (3), decode diagnostic (1), avatar fallback branch (2), text element variants (2).

## Files Created/Modified

- `ferro-json-ui/src/render/atoms.rs` (39 → 1849 LOC) — 23 real renderer bodies + helpers + tests

## Decisions Made

- **decode_props helper** (new): Phase 115 Spec serialization uses `skip_serializing_if = "Value::is_null"` on `Element.props`, so an element with no user-provided props arrives here with `props = Null`. serde_json can't decode `null` into all-optional structs like `SeparatorProps`. The helper treats `Null` as `{}` for decode, preserving v1's zero-prop default behavior.
- **Button non-GET: no data-action-* attributes**. The plan mentioned potential data-action-handler/method/url attributes for non-GET buttons. v1 render_node does NOT add those (it only wraps GET actions). Per D-21 (verbatim port), non-GET buttons render as bare `<button>` — client-side form dispatch lives elsewhere (inside the parent Form).
- **Icon fields emitted raw**: StatCard.icon, SidebarNavItem.icon, ActionCard.icon, NotificationItem.icon (where applicable) are SVG strings from server-side authoring. v1 emits them raw (unescaped) since they must contain `<svg>` markup. Preserving v1 behavior — these are trusted server content per the Phase 115 security domain.
- **render_sidebar_nav_item as private helper**: Matches v1's structure (lines 2516-2535). Sidebar is a single renderer but sidebar-nav-item is reused across `fixed_top`, `groups`, and `fixed_bottom` — keeping it as a helper avoids triplication.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Null-props decode failure for all-optional Props**

- **Found during:** Running initial test suite after first write
- **Issue:** `Element::new("Separator")` and `Element::new("Skeleton")` with no `.prop()` calls produce `props: Value::Null` (per Phase 115 SpecBuilder), and serde `from_value::<SeparatorProps>(Null)` returns `invalid type: null, expected struct SeparatorProps` — even though all the struct's fields are `Option<T>` and would decode fine from `{}`. This would have made every no-prop element emit a diagnostic comment instead of rendering.
- **Fix:** Added `decode_props<T>(&Value) -> Result<T, serde_json::Error>` helper that returns `from_value(Value::Object(empty))` when the input is `Null`, otherwise `from_value(input.clone())`. Applied via `replace_all` across all 23 renderers.
- **Files modified:** `ferro-json-ui/src/render/atoms.rs`
- **Verification:** `separator_default_is_horizontal_hr` and `skeleton_emits_shimmer_div` tests now pass; `props_decode_failure_emits_diagnostic` still passes for genuine type mismatches (e.g., `props: 42`).
- **Committed in:** `4ad293bf`

**2. [Rule 1 - Bug] Clippy uninlined_format_args in pagination prev/next/page links**

- **Found during:** `cargo clippy -p ferro-json-ui --lib --all-features -- -D warnings`
- **Issue:** Three `format!` calls in `render_pagination` used positional `{}` with explicit args; clippy's `uninlined_format_args` lint (enforced via `-D warnings`) requires inline `{var}` capture.
- **Fix:** Rewrote three format strings to use `{base_url_escaped}`, `{page}`, `{prev}`, `{next}` captures; hoisted `current - 1` and `current + 1` into `let` bindings.
- **Files modified:** `ferro-json-ui/src/render/atoms.rs`
- **Verification:** `cargo clippy -p ferro-json-ui --lib --all-features -- -D warnings` clean; pagination tests still pass.
- **Committed in:** `4ad293bf`

---

**Total deviations:** 2 auto-fixed (Rule 1 bugs — both caught by automated verification)
**Impact on plan:** The decode_props fix is load-bearing: without it, every default-constructed atom would emit a diagnostic. The clippy fix is a workspace-wide convention (pre-commit CI). Neither changed the v1 HTML contract — both preserve byte-for-byte emission.

## Issues Encountered

None outside the deviations above.

## Gates

- `cargo test -p ferro-json-ui --lib render::atoms::`: **38 passed / 0 failed**
- `cargo test -p ferro-json-ui --lib`: **250 passed / 0 failed** (was 212 after Plan 02; +38 atom tests)
- `cargo clippy -p ferro-json-ui --lib --all-features -- -D warnings`: clean
- `cargo fmt -p ferro-json-ui -- --check`: clean
- `grep -c "stub_renderer!" ferro-json-ui/src/render/atoms.rs`: **0**
- `grep -c "pub(crate) fn render_" ferro-json-ui/src/render/atoms.rs`: **23**

## Success Criteria

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 1 | All 23 atoms emit v1-compatible HTML | PASS | 23 functions, zero stubs, 38 tests pass |
| 2 | ~25 inline tests pass | PASS | 38 tests (exceeded target) |
| 3 | RENDER-01 advanced (SC-1 atoms) | PASS | All atom dispatch arms route to real renderers |
| 4 | RENDER-03 advanced (SC-3 action URL) | PASS | button_get_action_wraps_in_anchor + button_action_url_none_uses_href_hash_with_diagnostic + dropdown_menu_get_action_renders_anchor |
| 5 | No new external dependencies | PASS | No Cargo.toml changes |
| 6 | File size ~950 LOC | NOTE | 1849 LOC — larger than target because of per-renderer D-12 decode boilerplate + 38 tests. Still <2000 target per RESEARCH §"Module Layout Proposal". |

## Next Phase Readiness

- Plans 04 (containers) and 05 (form/data) can continue independently — atoms.rs is now feature-complete.
- Plan 06 (framework integration tests) now has real HTML output to assert against — test `text_html_escaping_in_content`, `image_xss_src_escaped`, `button_get_action_wraps_in_anchor`, `pagination_clamps_current_page`, and `button_action_url_none_uses_href_hash_with_diagnostic` establish the byte-level contract.
- `decode_props` helper is a candidate for promotion to `render/mod.rs` if Plans 04/05 want the same Null→{} behavior; for Phase 116 scope, it remains in atoms.rs as a module-local helper (containers have required slot fields so they'd diagnose genuinely-missing props anyway).

## Self-Check: PASSED

- `ferro-json-ui/src/render/atoms.rs`: FOUND (1849 LOC)
- Commit `4ad293bf`: FOUND (verified via `git log --oneline`)
- Zero `stub_renderer!` invocations: CONFIRMED
- 23 `pub(crate) fn render_*` functions: CONFIRMED
- 38 tests in `render::atoms::tests`: CONFIRMED (test output)

---
*Phase: 116-flat-element-renderer*
*Plan: 03*
*Completed: 2026-04-18*
