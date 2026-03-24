# Architecture Research

**Domain:** Server-side HTML renderer with CSS design tokens
**Researched:** 2026-03-24
**Confidence:** HIGH (all findings from direct codebase inspection)

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    HTTP Request Layer                         │
│  JsonUi::render() → JsonUi::build_response()                 │
├─────────────────────────────────────────────────────────────┤
│                    Theme Injection (framework)                │
│  current_theme() → theme.css → <style type="text/tailwindcss">│
├─────────────────────────────────────────────────────────────┤
│                    Render Pipeline (ferro-json-ui)            │
│  ┌──────────────┐  ┌────────────┐  ┌──────────────────────┐  │
│  │  render.rs   │  │  layout.rs │  │     runtime.rs        │  │
│  │ component→   │  │ base_doc + │  │ FERRO_RUNTIME_JS      │  │
│  │ HTML string  │  │ DashLayout │  │ tabs/toast/sidebar JS │  │
│  └──────┬───────┘  └─────┬──────┘  └──────────────────────┘  │
│         │                │                                    │
├─────────┴────────────────┴────────────────────────────────────┤
│                    Token Layer (ferro-theme)                   │
│  default.css: @theme { --radius-md, --shadow-sm, ... }        │
│  token.rs: const names (NOT consumed by render.rs today)      │
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Current State |
|-----------|----------------|---------------|
| `ferro-json-ui/src/render.rs` | Component→HTML with Tailwind classes | Hardcodes `rounded-md`, `shadow-sm`, etc. — does not use CSS custom properties |
| `ferro-json-ui/src/layout.rs` | Page shell (base_document, DashboardLayout, AuthLayout) | Font link has no home; `base_document()` is the natural injection point |
| `ferro-json-ui/src/runtime.rs` | Inline JS for tabs, toasts, sidebar toggle | Hardcodes `bg-blue-500`, `border-blue-600` — no semantic token awareness |
| `ferro-json-ui/src/config.rs` | `JsonUiConfig` (tailwind_cdn flag, body_class, custom_head) | `custom_head` can carry font `<link>` tags but no first-class font field |
| `ferro-theme/src/token.rs` | Constant names for 23 semantic token slots | Defined but not consumed by render.rs |
| `ferro-theme/assets/default.css` | `@theme` CSS with light+dark values | Has `--radius-md`, `--shadow-sm`, `--font-family-sans` defined |
| `framework/src/json_ui/mod.rs` | Bridge: resolves actions, injects theme CSS into head | Injects `theme.css` as `<style type="text/tailwindcss">` — Tailwind CDN processes it |
| `framework/src/theme/context.rs` | task-local `current_theme()` | Works correctly; available to render pipeline |

## Recommended Project Structure

No new crates or modules are needed. All changes are within existing files:

```
ferro-json-ui/src/
├── render.rs           # PRIMARY TARGET: replace hardcoded classes with token-aware classes
├── layout.rs           # SECONDARY: base_document gets font <link>; surface bg fixes
├── runtime.rs          # TERTIARY: replace hardcoded color names in JS toast/tab code
└── config.rs           # OPTIONAL: add font_url field to JsonUiConfig

ferro-theme/assets/
└── default.css         # Add Inter font @import + refined token values
```

### Structure Rationale

- **render.rs:** All 30 component renderers are here — it is the single file to change for visual quality. No new abstraction layer is needed.
- **layout.rs:** `base_document()` is where font loading belongs. It is called by all three built-in layouts.
- **runtime.rs:** The JS toast code has hardcoded `bg-blue-500` etc. These need to match the semantic token colors.

## Architectural Patterns

### Pattern 1: CSS Custom Property References in Tailwind v4

**What:** Tailwind v4 supports `rounded-[--radius-md]` syntax (CSS arbitrary-value referencing a custom property). When a `@theme` block defines `--radius-md: 0.375rem`, Tailwind v4 generates the `.rounded-\[--radius-md\]` class at the CDN processing step.

**When to use:** Every place render.rs currently hardcodes a shape, shadow, or font class.

**Confidence:** HIGH — Tailwind v4 arbitrary-value syntax is documented, and the project already uses `@tailwindcss/browser@4` CDN. The theme CSS is injected as `<style type="text/tailwindcss">` before rendering, so the CDN processes all `@theme` directives before the page displays.

**Trade-offs:**
- Pro: render.rs becomes theme-overridable without changing Rust code
- Pro: No new Rust API — just different class strings
- Con: Class strings like `rounded-[--radius-md]` are less readable than `rounded-md`
- Con: Themes must always define the custom properties (they already must — token.rs enforces this)

**Example (before):**
```rust
"<div class=\"rounded-lg border border-border bg-background shadow-sm\">"
```

**Example (after):**
```rust
"<div class=\"rounded-[--radius-lg] border border-border bg-card shadow-[--shadow-sm]\">"
```

**Note on `rounded-md` vs `rounded-[--radius-md]`:** Tailwind v4's `@theme` directive makes `--radius-md` available as `rounded-md` only if the theme names it to map to Tailwind's own scale. Since ferro uses custom names (`--radius-md` mapped to `0.375rem`, same as Tailwind's `rounded-md` default), the simplest approach is to keep using Tailwind utility names (`rounded-md`) for shape tokens, because the default theme values match. Only use `rounded-[--radius-md]` if the theme overrides are expected. For this visual overhaul milestone, use Tailwind's existing scale classes (`rounded-md`, `rounded-lg`) but fix the semantic surface token mismatches (`bg-background` → `bg-card` for Card).

### Pattern 2: Semantic Surface Layering

**What:** The token vocabulary defines a three-level elevation hierarchy: `background` (page) → `surface` (panels, sidebars) → `card` (cards, modals). Components must use the correct level for their visual layer.

**Current violations found in render.rs:**
- `render_card()` uses `bg-background` — should be `bg-card`
- `render_stat_card()` uses `bg-background` — should be `bg-card`
- `render_modal()` inner panel uses `bg-background` — should be `bg-card`
- `DashboardLayout` body uses `bg-surface` (correct) but sidebar uses `bg-background` (wrong — sidebar is a panel, should be `bg-surface`)
- `layout_header_html()` uses `bg-background` — should be `bg-surface` or keep `bg-background` (header is top-level)

**When to use:** Any container component that renders as a visually raised surface.

**Rule:** Card-level components (`Card`, `StatCard`, `Modal` dialog panel, `NotificationDropdown` panel) → `bg-card`. Panel-level structures (sidebar, collapsible, table header) → `bg-surface`. Page background → `bg-background`.

### Pattern 3: CSS-Only Select Arrow

**What:** Replacing the native `<select>` arrow with a custom SVG chevron using a CSS `background-image`. Requires `appearance-none` (already present) plus a wrapper div that provides the arrow via `::after` pseudo-element or inline SVG background.

**Current state:** `render_select()` uses `appearance-none bg-background` but provides no custom arrow — the select element is unstyled on most browsers.

**Recommended approach:** Wrap the `<select>` in a `relative` div and add an absolute-positioned SVG span:

```rust
"<div class=\"relative\">\
  <select class=\"block w-full appearance-none bg-background rounded-md border border-border \
    px-3 py-2 pr-8 text-sm shadow-sm focus:border-primary focus:ring-1 focus:ring-primary\">\
  </select>\
  <span class=\"pointer-events-none absolute inset-y-0 right-2 flex items-center\">\
    <svg class=\"h-4 w-4 text-text-muted\" ...chevron-down SVG...</svg>\
  </span>\
</div>"
```

**Why not CSS background-image:** The inline SVG approach works without external assets and supports semantic token colors (SVG `fill` can use `currentColor`).

### Pattern 4: Consistent Focus Ring Pattern

**What:** A standardized focus ring applied uniformly across all interactive elements (inputs, select, textarea, buttons, checkboxes).

**Current state:** Inputs/selects use `focus:border-primary focus:ring-1 focus:ring-primary`. Buttons use `transition-colors` only. Checkboxes use `focus:ring-primary`. No outline-offset pattern.

**Recommended uniform pattern:** `focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2`

- `focus-visible` (not `focus`) — only shows ring on keyboard navigation, not mouse clicks
- `ring-offset-2` — small gap between element border and ring for clarity

This replaces the current split `focus:border-primary focus:ring-1 focus:ring-primary` approach.

### Pattern 5: CSS Transition Classes

**What:** Consistent animation timing across interactive components.

**Current state:** Buttons have `transition-colors`. Switch toggle has `after:transition-all`. Collapsible arrow uses `transition-transform`. No unified timing or easing.

**Recommended approach:** Define a standard transition class set:
- Interactive elements (buttons, links): `transition-colors duration-150`
- Position/size changes (switch thumb): `transition-all duration-200`
- Rotation (collapsible arrow, accordion): `transition-transform duration-200`

Keep these as inline Tailwind classes — no abstraction needed.

### Pattern 6: Font Loading via base_document

**What:** Inter (or another professional sans-serif) should be loaded once for all layouts via the `base_document()` function in layout.rs.

**Where exactly:** In `base_document()`, inject a Google Fonts `<link>` preconnect + stylesheet before the `{head}` placeholder. This ensures it appears for all three built-in layouts (DefaultLayout, AppLayout, AuthLayout) and DashboardLayout (which delegates to `base_document_ext`).

**Example addition to base_document:**
```rust
let font_link = r#"<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">"#;
```

Then update `default.css`:
```css
--font-family-sans: 'Inter', ui-sans-serif, system-ui, sans-serif;
```

**Alternative:** Expose a `font_url: Option<String>` field on `JsonUiConfig` that gets injected in `JsonUi::build_response()`. This is more flexible but adds API surface. The `base_document` approach is simpler and achieves the same result for all layouts.

**Recommendation:** Hard-code Inter in `base_document` for v10.0. The font is universally appropriate and CDN-available. If customization is needed later, add a `JsonUiConfig` field.

### Pattern 7: Class Builder vs Inline Strings

**What:** Whether to introduce a helper like `ClassBuilder` / `cx!()` macro to compose Tailwind class strings, versus keeping inline string literals.

**Assessment:** Do not introduce a class builder for this milestone.

**Rationale:**
- render.rs already has ~1700 lines of working inline string concatenation
- A class builder adds a new abstraction that agents must learn to generate
- The visual changes are largely one-for-one substitutions (`bg-background` → `bg-card`)
- Tailwind v4 with CDN has no purging — all classes are available regardless of how they appear in source

**What to do instead:** Define small `const` strings for frequently reused class combinations (e.g., the focus ring pattern, the form field wrapper pattern). Keep these local to the function or as module-level `const` if reused by 3+ functions.

```rust
// Module-level const for reused patterns
const FOCUS_RING: &str = "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2";
const FIELD_LABEL: &str = "block text-sm font-medium text-text";
```

This reduces repetition without adding abstraction overhead.

## Data Flow

### Theme → Visual Output

```
App Startup: ThemeMiddleware::new().resolver(...)
    ↓
HTTP Request arrives
    ↓
ThemeMiddleware → current_theme() task-local set
    ↓
Handler calls JsonUi::render(view, data)
    ↓
JsonUi::build_response()
    ├── current_theme() → theme.css injected as <style type="text/tailwindcss">
    ├── render_to_html_with_plugins(view, data) → HTML fragment with Tailwind class strings
    └── render_layout(layout_name, ctx)
        ├── base_document() → <!DOCTYPE html> + <head> + font links
        └── DashboardLayout::render() → sidebar + header + main
            ↓
Full HTML page with:
  - Tailwind CDN script (processes @theme + @import)
  - <style> containing theme tokens (e.g., --radius-md: 0.5rem)
  - Component HTML using classes like bg-card, rounded-[--radius-md]
```

### Visual Quality Change Points

For the v10.0 overhaul, changes touch three files and flow in this order:

1. `ferro-theme/assets/default.css` — Token value refinements + Inter font family
2. `ferro-json-ui/src/layout.rs` — Font `<link>` injection in `base_document()`
3. `ferro-json-ui/src/render.rs` — Surface bg corrections + focus rings + transitions + select arrow + SVG icons
4. `ferro-json-ui/src/runtime.rs` — Replace hardcoded `bg-blue-500` etc. with semantic CSS variables

## Integration Points

### Existing Boundary: render.rs ↔ ferro-theme

**Current:** render.rs has **zero dependency** on ferro-theme. It uses hardcoded Tailwind class strings. The theme system only affects the CSS injected into `<head>` — it does not influence which classes the renderer emits.

**Implication for this milestone:** The visual improvements require no new cross-crate dependencies. render.rs changes Tailwind class strings (e.g., `bg-background` → `bg-card` for Card) — these changes are visible because those class names map to the token values defined in the theme CSS.

**Future consideration (not this milestone):** If we ever need render.rs to *programmatically* read token values (e.g., to choose between rendering strategies), we would need to pass a `Theme` reference into the render pipeline. That is not needed for v10.0.

### Existing Boundary: framework ↔ ferro-json-ui

`framework/src/json_ui/mod.rs` is the only place where `current_theme()` is called and theme CSS is injected. This file is the correct place for any new theme-aware head content (font `<link>` tags could also go here via config).

### Internal Boundary: layout.rs base_document ↔ all layouts

All three built-in layouts and `DashboardLayout` call `base_document()` or `base_document_ext()`. Adding font loading to `base_document()` affects all layouts uniformly — this is the desired behavior.

### runtime.js hardcoded colors

`FERRO_RUNTIME_JS` in `runtime.rs` uses JS-side hardcoded Tailwind class names (`bg-blue-500`, `border-blue-600`) for dynamic DOM manipulation during tab switching and toast creation. These do not respond to the theme system.

**Resolution:** Replace with semantic CSS custom property references using inline styles or replace the Tailwind class names with semantic token names:
- Toast creation: use CSS variables directly in inline style (`background: oklch(var(--color-primary))` or use semantic Tailwind classes like `bg-primary`)
- Tab switching: replace `border-blue-600` with `border-primary`, `text-blue-600` with `text-primary`, `text-gray-500` with `text-text-muted`

The JS can use semantic class names because Tailwind CDN generates all classes eagerly — `border-primary` is always available.

## Anti-Patterns

### Anti-Pattern 1: ThemeContext in render.rs

**What people do:** Pass `current_theme()` into the render functions so the Rust code can read token values and make rendering decisions (e.g., `if theme.radius == "full" { "rounded-full" } else { "rounded-md" }`).

**Why it's wrong:** Creates coupling between ferro-json-ui (pure renderer) and ferro-theme via the framework context. Also forces token values to be parsed as Rust strings rather than expressed as CSS. Makes the renderer stateful.

**Do this instead:** Keep render.rs token-agnostic. Express theme variation purely in CSS via custom properties. The Tailwind class `rounded-[--radius-md]` automatically reflects whatever `--radius-md` is set to in the theme's CSS, without render.rs knowing the value.

### Anti-Pattern 2: ClassBuilder Abstraction

**What people do:** Introduce a `cx!()` macro or `ClassBuilder` struct to compose Tailwind class strings with conditional logic.

**Why it's wrong:** Adds an abstraction that agents need to learn and generate. The render.rs functions are already well-understood as simple string builders. The conditional class logic in render.rs (e.g., `border_class` for error states) is simple enough to keep inline.

**Do this instead:** Use module-level `const` strings for repeated patterns (focus ring, label style) and inline `match` arms for variant-specific classes.

### Anti-Pattern 3: Per-Component Shadow Values

**What people do:** Each component picks its own shadow from Tailwind's scale (`shadow-sm`, `shadow-md`, `shadow-lg`) based on intuition.

**Why it's wrong:** Inconsistency. The theme token vocabulary (`--shadow-sm`, `--shadow-md`, `--shadow-lg`) exists to let themes control elevation uniformly.

**Do this instead:** Define a mapping rule:
- Inline elements, form fields: no shadow
- Cards, stat cards: `shadow-[--shadow-sm]`
- Modals, popovers, notification dropdowns: `shadow-[--shadow-md]`
- Full-screen overlays (if any): `shadow-[--shadow-lg]`

Apply this mapping consistently to all components rather than case-by-case.

### Anti-Pattern 4: Custom Head for Font Loading

**What people do:** Users inject font `<link>` via `JsonUiConfig::custom_head()` as a workaround.

**Why it's wrong:** Requires every app to manually configure font loading. It is boilerplate that should be a framework default.

**Do this instead:** Embed the Inter font `<link>` in `base_document()`. Inter is universally appropriate for professional UIs and is the font used by Linear, Vercel, and most modern SaaS products. Apps that need a different font can override `--font-family-sans` in their theme CSS.

### Anti-Pattern 5: Emoji Icons in Notification Bell

**What people do:** Use emoji characters (`&#x1F514;` bell emoji) in the notification dropdown button.

**Why it's wrong:** Emoji rendering is platform-dependent (size, color, metrics vary per OS). Inconsistent with SVG icons used elsewhere.

**Do this instead:** Replace with an inline SVG bell icon (same pattern as the hamburger button in `layout_header_html()`). Keep all icons as SVG strings.

## Migration Path for Existing Apps

No breaking API changes are required. All changes are:
1. Visual improvements to HTML output (class names change)
2. Font loading added to `base_document()` (additive)
3. theme CSS token value refinements (additive — values may shift slightly)

**Apps using `ThemeMiddleware`:** Will pick up better default token values from the updated `default.css`. No code changes needed.

**Apps using `DashboardLayout`:** The sidebar background fix (`bg-background` → `bg-surface`) may cause a subtle visual change. This is intentional.

**Apps overriding `body_class` via `JsonUiConfig`:** Unaffected.

**Apps with custom theme CSS files:** Unaffected — they provide their own token values.

## Build Order for v10.0

The following order minimizes re-work and ensures visual changes can be reviewed incrementally:

1. **default.css token refinements** — Adjust color values for better contrast, add Inter to font stack. No code changes. Visual baseline improves immediately.

2. **base_document() font loading** — Add Inter Google Fonts `<link>` to `base_document()`. One-line addition. All layouts pick it up.

3. **Surface bg corrections in render.rs** — Fix `bg-background` → `bg-card` for Card, StatCard, Modal dialog, NotificationDropdown panel. These are the highest-impact visual changes.

4. **DashboardLayout sidebar bg fix** — `layout_sidebar_html()` sidebar `<aside>` uses `bg-background`, should use `bg-surface`. Header already uses `bg-background` which is correct for a top chrome element.

5. **Focus ring standardization in render.rs** — Replace all `focus:border-primary focus:ring-1 focus:ring-primary` with `focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2`. Apply to all form fields (Input, Select, Textarea, Checkbox, Switch).

6. **Select custom arrow in render.rs** — Wrap `<select>` in a relative `<div>` and add inline SVG chevron span.

7. **Transition consistency in render.rs** — Add `duration-150` to button transitions; standardize on `duration-200` for switch/collapsible.

8. **Shadow token mapping in render.rs** — Apply the elevation mapping (card→shadow-sm, modal→shadow-md) consistently.

9. **runtime.js semantic class names** — Replace `bg-blue-500`/`border-blue-600` with `bg-primary`/`border-primary`/`text-text-muted` in `FERRO_RUNTIME_JS`.

10. **Emoji→SVG in render.rs and layout.rs** — Replace `&#x1F514;` bell emoji with SVG; audit for other emoji usage.

## Sources

- Direct codebase inspection: `ferro-json-ui/src/render.rs`, `layout.rs`, `runtime.rs`, `config.rs`
- `ferro-theme/src/token.rs`, `ferro-theme/assets/default.css`
- `framework/src/json_ui/mod.rs`, `framework/src/theme/context.rs`
- Token vocabulary is HIGH confidence (inspected directly)
- Tailwind v4 `rounded-[--custom-prop]` syntax — MEDIUM confidence (based on Tailwind v4 arbitrary value docs knowledge, aligns with how the CDN is already being used)

---
*Architecture research for: ferro-json-ui visual overhaul (v10.0)*
*Researched: 2026-03-24*
