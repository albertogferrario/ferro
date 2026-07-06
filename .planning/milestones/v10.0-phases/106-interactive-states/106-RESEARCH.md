# Phase 106: Interactive States - Research

**Researched:** 2026-03-25
**Domain:** Tailwind v4 focus-visible / hover states in Rust HTML generation
**Confidence:** HIGH

## Summary

Phase 106 adds keyboard focus rings and hover highlights to every interactive element in the JSON-UI renderer. The work is a direct continuation of Phase 105 (Form Polish), which already established the canonical pattern: `focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2` and `transition-colors duration-150 motion-reduce:transition-none`. Phase 106 propagates this pattern to the six remaining interactive element types — buttons, tab buttons/links, pagination links, breadcrumb links, sidebar nav items — and adds `hover:bg-surface` to table rows.

The implementation is entirely within two files: `ferro-json-ui/src/render.rs` (buttons, tabs, pagination, breadcrumbs, sidebar standalone component, table rows) and `ferro-json-ui/src/layout.rs` (sidebar nav items in DashboardLayout shell). No new dependencies, no new Rust types, no CDN concerns. All required Tailwind v4 classes (`focus-visible:*`, `motion-reduce:*`) are already proven working from Phase 105.

The test infrastructure is already prepared: two comment stubs in `render.rs` explicitly flag Phase 106 changes — "Phase 106 adds hover:bg-surface to rows" (line 5445) and "Phase 106 adds focus-visible ring" (line 5852). Tests should follow the structural test pattern established in Phase 102: `has_class()` helper for resilient class assertions rather than full class-string matching.

**Primary recommendation:** Apply the exact focus-visible ring and transition-colors class strings from Phase 105 form elements to each of the six remaining interactive element categories. One plan, one file at a time (render.rs then layout.rs), with new structural tests per requirement.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| INT-01 | All buttons have `focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2` | `render_button()` at line 1094 — `base` string already has `transition-colors` but no focus ring. Pattern from Phase 105 `focus_ring_class` is directly applicable. |
| INT-02 | Tab buttons have focus-visible ring | `render_tabs()` at line 451 — button elements at line 490 and `<a>` server-driven tabs at line 501 both lack focus ring classes. |
| INT-03 | Pagination links have focus-visible ring | `render_pagination()` at line 1288 — three `<a>` class strings (prev, page numbers, next) at lines 1306, 1325, 1337 each need focus ring appended. |
| INT-04 | Breadcrumb links have focus-visible ring | `render_breadcrumb()` at line 1260 — non-last items with URL render `<a>` at line 1273 without focus ring. |
| INT-05 | Sidebar nav items have focus-visible ring | Two functions: `render_sidebar_nav_item()` in render.rs at line 1726 (standalone Sidebar component) and `layout_sidebar_nav_item()` in layout.rs at line 144 (DashboardLayout shell). Both return `<a>` elements without focus ring classes. |
| INT-06 | Table rows have `hover:bg-surface` for row highlighting | `render_table()` at line 590 — body row `<tr>` at line 627 renders `<tr>` with no class. Needs `class="hover:bg-surface"` added. |
| INT-07 | All interactive elements have `transition-colors duration-150 motion-reduce:transition-none` | Buttons already have `transition-colors` in base (line 1095) but not duration/motion-reduce. Tab buttons, pagination links, breadcrumb links, and sidebar nav items currently have no transition class. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tailwind v4 (CDN) | v4.x | Utility CSS | Already integrated via CDN in JSON-UI head |
| Rust `format!` / `concat!` | stdlib | HTML generation | Established pattern throughout render.rs |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `has_class()` helper | project-local | Resilient class assertions in tests | Every new test — avoids full class-string fragility |

**Installation:** No new dependencies required.

## Architecture Patterns

### Recommended Project Structure

No new files needed. All changes go into:
```
ferro-json-ui/src/
├── render.rs     # buttons, tabs, pagination, breadcrumbs, sidebar (component), table rows
└── layout.rs     # sidebar nav items in DashboardLayout shell
```

### Pattern 1: Focus Ring via Class Constant

**What:** The canonical focus ring class string is a `&str` literal applied directly in the element's class attribute.

**When to use:** Every non-form interactive element (buttons, tabs, pagination, breadcrumb links, sidebar links).

**Example:**
```rust
// Source: render.rs lines 696-699 (Phase 105 established pattern)
let focus_ring = "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2";

// Button (INT-01):
let base = "inline-flex items-center justify-center rounded-md font-medium \
    transition-colors duration-150 motion-reduce:transition-none \
    focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2";
```

### Pattern 2: Transition Classes

**What:** `transition-colors duration-150 motion-reduce:transition-none` is the full transition string — duration and reduced-motion selector must accompany `transition-colors`.

**When to use:** All interactive elements. Buttons already have `transition-colors` but are missing `duration-150 motion-reduce:transition-none` (INT-07). All other elements have neither.

**Example:**
```rust
// Source: render.rs line 1095 (current — MISSING duration and motion-reduce)
let base = "inline-flex items-center justify-center rounded-md font-medium transition-colors";
// Correct after Phase 106:
let base = "inline-flex items-center justify-center rounded-md font-medium \
    transition-colors duration-150 motion-reduce:transition-none \
    focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2";
```

### Pattern 3: Table Row Hover

**What:** `<tr>` elements get `class="hover:bg-surface"`. Tailwind v4 `hover:*` variant on a `<tr>` applies to the row's background on cursor hover.

**When to use:** Body rows only (not header rows — header uses `bg-surface` statically).

**Example:**
```rust
// Source: render.rs line 627 (current — no class)
html.push_str("<tr>");
// Correct after Phase 106:
html.push_str("<tr class=\"hover:bg-surface\">");
```

### Pattern 4: Pagination/Breadcrumb/Tab Link Focus Ring

**What:** Links (`<a>`) that act as interactive navigation need the focus ring appended to their class string. For pagination links that already have a class string, append focus ring classes.

**When to use:** `render_pagination()` prev/next/page links, `render_breadcrumb()` links, tab `<a>` elements.

**Example:**
```rust
// Source: render.rs line 1325 (current pagination page link)
"<a href=\"{}page={}\" class=\"px-3 py-1 rounded-md bg-background text-text hover:bg-surface\">{}</a>"
// Correct after Phase 106:
"<a href=\"{}page={}\" class=\"px-3 py-1 rounded-md bg-background text-text hover:bg-surface \
 transition-colors duration-150 motion-reduce:transition-none \
 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2\">{}</a>"
```

### Anti-Patterns to Avoid

- **`focus:ring-*` instead of `focus-visible:*`:** The requirement (INT success criteria) explicitly mandates `focus-visible:` so mouse clicks do not show rings. Line 926 in the existing checkbox uses `focus:ring-primary` — this is NOT the pattern to copy.
- **Missing `ring-offset-2`:** The ring-offset is needed so the ring appears outside the element background, not inset. Omitting it collapses the ring into the element.
- **Adding focus ring to `<tr>` header row:** Only body rows need `hover:bg-surface`. Header uses `bg-surface` statically — hover would have no visible effect but would be harmless.
- **Full class string in tests:** Use `has_class(&html, "focus-visible:ring-primary")` not `html.contains("class=\"...full string...\"")` — the structural test pattern from Phase 102 is more resilient.
- **Modifying active sidebar item:** The active sidebar nav item uses a different class set (`bg-card text-primary`) — it must NOT get `hover:bg-surface` because hovering the current page should not change its appearance. Focus ring should still be added to active items.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Focus ring calculation | Custom CSS utility | Tailwind `focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2` | Phase 105 already validated these class names work in Tailwind v4 CDN mode |
| Reduced-motion detection | JavaScript media query listener | CSS `motion-reduce:transition-none` | CDN mode, zero JS required |
| Row hover state | JavaScript mouseenter/mouseleave | CSS `hover:bg-surface` on `<tr>` | Pure CSS, works without JS, consistent with Tailwind v4 approach |

**Key insight:** All required interactive state classes are plain Tailwind v4 utilities already proven working in this project. No custom CSS, no JavaScript, no new tokens.

## Common Pitfalls

### Pitfall 1: Missing `ring-offset-2` on `<a>` elements
**What goes wrong:** Focus ring appears invisible or inset on anchor tags because `ring-offset-2` requires an explicit background behind it to contrast.
**Why it happens:** Pagination links have `bg-background` so ring-offset works. Breadcrumb links have no background — the page background provides contrast for the offset.
**How to avoid:** Always include `ring-offset-2` as part of the canonical focus ring class string. The existing form elements all use it.
**Warning signs:** Invisible focus ring during tab navigation in browser.

### Pitfall 2: `layout.rs` sidebar nav items missed
**What goes wrong:** The standalone `Sidebar` component (render.rs) gets updated but sidebar items in `DashboardLayout` (layout.rs) do not, leaving inconsistent behavior.
**Why it happens:** Two separate functions render sidebar nav items: `render_sidebar_nav_item()` in render.rs and `layout_sidebar_nav_item()` in layout.rs. They are structurally identical but live in different files.
**How to avoid:** Update both functions. The research confirms both at render.rs:1726 and layout.rs:144.
**Warning signs:** Sidebar works in isolated component tests but fails in full layout tests.

### Pitfall 3: Buttons already have `transition-colors` but not the full string
**What goes wrong:** The INT-07 requirement ("all interactive elements have `transition-colors duration-150 motion-reduce:transition-none`") is incorrectly marked as "done for buttons" because `transition-colors` exists in the base string.
**Why it happens:** Phase 105 added `duration-150 motion-reduce:transition-none` to form elements but buttons were not touched. The button `base` string (line 1095) has only `transition-colors`.
**How to avoid:** The full triple `transition-colors duration-150 motion-reduce:transition-none` must be in button base.
**Warning signs:** Test asserting `has_class(&html, "duration-150")` on a button fails.

### Pitfall 4: Tab active state class branching
**What goes wrong:** Tab buttons have two branches — active vs inactive — with different text/border classes. Focus ring must be added to ALL branches, not just the active branch.
**Why it happens:** `render_tabs()` builds class strings per-tab based on `is_active` flag, tempting per-branch rather than shared focus ring class.
**How to avoid:** Factor focus ring into a shared `const` or local variable applied outside the `is_active` branch.

## Code Examples

Verified patterns from existing codebase:

### Canonical Focus Ring String (from render.rs lines 696-699)
```rust
// Source: ferro-json-ui/src/render.rs (Phase 105, lines 695-699)
let focus_ring_class = if has_error {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-destructive focus-visible:ring-offset-2"
} else {
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
};
```
For non-form interactive elements there is no error state, so use the non-error variant directly as a constant.

### Button Base String (current, needs update)
```rust
// Source: ferro-json-ui/src/render.rs line 1095
let base = "inline-flex items-center justify-center rounded-md font-medium transition-colors";
// Phase 106 target:
let base = "inline-flex items-center justify-center rounded-md font-medium \
    transition-colors duration-150 motion-reduce:transition-none \
    focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2";
```

### Tab Button (current, needs update)
```rust
// Source: ferro-json-ui/src/render.rs lines 489-498 (client-side trigger)
html.push_str(&format!(
    "<button type=\"button\" role=\"tab\" data-tab=\"{}\" \
     class=\"border-b-2 {} {} px-3 py-2 text-sm font-medium cursor-pointer\" \
     aria-selected=\"{}\">{}</button>",
    ...
));
// Phase 106 target — add focus ring and transition to class:
// "border-b-2 {} {} px-3 py-2 text-sm font-medium cursor-pointer \
//  transition-colors duration-150 motion-reduce:transition-none \
//  focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
```

### Table Row (current — no class)
```rust
// Source: ferro-json-ui/src/render.rs line 627
html.push_str("<tr>");
// Phase 106 target:
html.push_str("<tr class=\"hover:bg-surface\">");
```

### Structural Test Pattern (has_class helper)
```rust
// Source: ferro-json-ui/src/render.rs lines 5223-5228
fn has_class(html: &str, class: &str) -> bool {
    html.contains(&format!("class=\"{class}\""))
        || html.contains(&format!("class=\"{class} "))
        || html.contains(&format!(" {class}\""))
        || html.contains(&format!(" {class} "))
}

// Usage in new tests:
assert!(has_class(&html, "focus-visible:ring-primary"), "button should have focus ring");
assert!(has_class(&html, "transition-colors"), "button should have transition");
assert!(html.contains("motion-reduce:transition-none"), "should suppress animation for reduced motion");
assert!(html.contains("hover:bg-surface"), "table row should highlight on hover");
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `focus:ring-*` | `focus-visible:ring-*` | Phase 105 | Mouse clicks no longer show focus ring; keyboard-only |
| `focus:ring-1` | `focus-visible:ring-2` | Phase 105 | Ring is now 2px, visible at WCAG level |
| `transition-colors` alone | `transition-colors duration-150 motion-reduce:transition-none` | Phase 105 | Duration explicit; reduced-motion users unaffected |

**Deprecated/outdated:**
- `focus:ring-primary` on checkbox (render.rs line 926): This is the pre-Phase-105 pattern. Phase 106 should also update this checkbox to `focus-visible:ring-primary` for consistency, though it is not formally an INT requirement.

## Open Questions

1. **Checkbox `focus:ring-primary` on line 926**
   - What we know: Checkbox uses the old `focus:` prefix, not `focus-visible:`. INT requirements don't explicitly cover checkbox (that's FRM territory, already complete).
   - What's unclear: Whether Phase 106 should clean this up or leave it for a separate pass.
   - Recommendation: Update it opportunistically since it's adjacent code and wrong by the established standard. Low risk.

2. **Active sidebar nav item focus ring**
   - What we know: Active items use a hardcoded class string `"flex items-center gap-2 px-3 py-2 rounded-md text-sm font-medium bg-card text-primary"` — no hover, no focus ring.
   - What's unclear: Should active items show a focus ring when tabbed to?
   - Recommendation: Yes — a user can tab to the current-page link; they need to know keyboard focus is there. Add focus ring to active class string too.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` via `cargo test` |
| Config file | none — cargo workspace |
| Quick run command | `cargo test -p ferro-json-ui 2>&1 \| tail -5` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| INT-01 | Button renders with `focus-visible:ring-primary` and `duration-150` | unit | `cargo test -p ferro-json-ui button_focus_ring -- --nocapture` | Wave 0 |
| INT-02 | Tab button/link renders with focus ring | unit | `cargo test -p ferro-json-ui tabs_focus_ring` | Wave 0 |
| INT-03 | Pagination links render with focus ring | unit | `cargo test -p ferro-json-ui pagination_focus_ring` | Wave 0 |
| INT-04 | Breadcrumb links render with focus ring | unit | `cargo test -p ferro-json-ui breadcrumb_focus_ring` | Wave 0 |
| INT-05 | Sidebar nav items render with focus ring | unit | `cargo test -p ferro-json-ui sidebar_nav_focus_ring` | Wave 0 |
| INT-06 | Table body rows have `hover:bg-surface` | unit | `cargo test -p ferro-json-ui table_row_hover` | Wave 0 |
| INT-07 | Interactive elements have full transition string | unit | covered by INT-01 through INT-05 tests | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-json-ui 2>&1 | tail -5`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `ferro-json-ui/src/render.rs` test block — 7 new structural tests (INT-01 through INT-07), named following the `{component}_focus_ring` / `table_row_hover` convention, placed in `mod structural_tests`

*(No new test files needed — all tests live inline in render.rs following the project pattern)*

## Sources

### Primary (HIGH confidence)
- Direct code read of `ferro-json-ui/src/render.rs` — all render function line numbers verified
- Direct code read of `ferro-json-ui/src/layout.rs` — layout_sidebar_nav_item verified
- `.planning/REQUIREMENTS.md` — INT-01 through INT-07 class strings specified verbatim
- `.planning/phases/105-form-polish/105-01-PLAN.md` — Phase 105 implementation patterns as canonical reference

### Secondary (MEDIUM confidence)
- `ferro-json-ui/src/render.rs` lines 5445, 5852 — Phase 106 comment stubs confirm planned changes

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all required classes already working in codebase from Phase 105
- Architecture: HIGH — file locations, function names, and line numbers verified by direct read
- Pitfalls: HIGH — identified from direct code inspection, not inference

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (stable codebase, no external dependencies changing)
