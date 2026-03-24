# Stack Research

**Domain:** Server-side HTML rendering — professional UI quality for JSON-UI
**Researched:** 2026-03-24
**Confidence:** HIGH

## Context

This is a research file for the v10.0 JSON-UI Visual Overhaul milestone. The existing system
already has: `@tailwindcss/browser@4` CDN, `ferro-theme` with 23 semantic oklch tokens, and
a `<style type="text/tailwindcss">` injection pipeline. This file covers only what needs to
change or be added for professional visual quality.

---

## Critical Bug: Font Token Namespace Mismatch

**This must be fixed before any font work can have effect.**

The existing `ferro-theme/assets/default.css` uses `--font-family-sans` and `--font-family-mono`.
Tailwind v4's `@theme` namespace for font-family utilities is `--font-sans` and `--font-mono`
(not `--font-family-*`). The existing tokens define CSS custom properties but generate no
Tailwind utilities and do not affect `font-sans` base styles.

```css
/* WRONG — generates no Tailwind utility, affects nothing */
--font-family-sans: ui-sans-serif, system-ui, sans-serif;

/* CORRECT — generates font-sans utility and sets body default */
--font-sans: ui-sans-serif, system-ui, sans-serif;
```

Source: [Tailwind v4 font-family documentation](https://tailwindcss.com/docs/font-family) (verified)

---

## Recommended Stack

### Core Technologies (already in use — no changes)

| Technology | Version | Purpose | Status |
|------------|---------|---------|--------|
| `@tailwindcss/browser` | `@4` (pins to 4.x, current: 4.2.2) | Tailwind v4 CDN for development rendering | Working |
| oklch color space | CSS native | Perceptually uniform semantic tokens | Working in default.css |
| `<style type="text/tailwindcss">` | — | Injects theme CSS for CDN processing | Working |

### Font Loading

**Recommendation: Bunny Fonts CDN for Inter Variable.**

Do not use Google Fonts. German courts have ruled Google Fonts transmits user IPs to Google,
violating GDPR. Bunny Fonts is a 1:1 drop-in replacement with zero logging and EU-based
delivery.

Do not self-host fonts. Ferro generates HTML server-side; fonts would need to be static
assets in the user's application, adding setup friction. CDN is correct for development-mode
JSON-UI (same audience as `tailwind_cdn: true`).

| Font | CDN | URL | Format | Purpose |
|------|-----|-----|--------|---------|
| Inter Variable | Bunny Fonts | `https://fonts.bunny.net` | woff2 variable | Primary UI sans-serif |
| Geist (alternative) | jsDelivr npm | `https://cdn.jsdelivr.net/npm/geist` | woff2 variable | Vercel-style developer aesthetic |

**Inter is the correct default.** It is the most-copied professional UI typeface (used by
Linear, Vercel, Stripe, GitHub). It has the highest character coverage and a variable font
file covering weights 100–900 in one HTTP request.

Bunny Fonts CSS import for Inter Variable:

```css
@import url('https://fonts.bunny.net/css?family=inter:100,200,300,400,500,600,700,800,900&display=swap');
```

This import goes inside `<style type="text/tailwindcss">` before the `@theme` block. The
`@tailwindcss/browser` CDN processes `@import` statements within `type="text/tailwindcss"` blocks.

Then in `@theme`:

```css
@theme {
  --font-sans: 'Inter', ui-sans-serif, system-ui, sans-serif;
}
```

Note: `Inter Variable` as a font-family name requires quoting because of the space. Using
`'Inter'` (matching the Bunny Fonts declaration) is simpler and also matches the name served
by fonts.bunny.net.

**Geist as alternative** (for theme authors wanting the Vercel aesthetic):
```css
@import url('https://fonts.bunny.net/css?family=geist:100,200,300,400,500,600,700,800,900&display=swap');
@theme { --font-sans: 'Geist', ui-sans-serif, system-ui, sans-serif; }
```

Geist is available on Bunny Fonts. The Vercel CDN (`geistfont.vercel.app`) is also an option
but is a third-party unofficial host — Bunny Fonts is preferable for consistency.

### Supporting Libraries (CSS-only, no npm)

None. All visual quality improvements are pure CSS patterns injected via the existing
`<style type="text/tailwindcss">` pipeline. No new Rust crates or npm packages needed.

---

## CSS Patterns for Visual Quality

### 1. Focus Rings (`focus-visible:`)

Use `focus-visible:` not `focus:`. The distinction matters: `focus:` triggers for mouse
clicks; `focus-visible:` triggers only for keyboard navigation, which is the correct
accessibility behavior.

**Standard pattern:**
```html
<button class="... focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary">
```

Alternative using ring utilities (visually identical, slightly more Tailwind-idiomatic):
```html
<button class="... focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 focus-visible:outline-none">
```

The `outline-none` on the ring variant prevents double indicators. Both patterns are valid
with `@tailwindcss/browser@4`.

**In render.rs:** Every interactive element (Button, Input, Select, Checkbox, Switch, Tab
trigger, Breadcrumb link, Pagination button, Collapsible summary) needs one of these patterns.
Currently, inputs use `focus:ring-2 focus:ring-primary/20` — this should be
`focus-visible:ring-2 focus-visible:ring-primary/20 focus-visible:outline-none`.

### 2. Transitions

Use `transition-colors` for color/background changes. Use `transition-transform` for
geometric changes. Never use `transition-all` — it creates performance issues and causes
focus style transitions on properties that should appear instantly.

```html
<!-- Button hover -->
class="... transition-colors duration-150 hover:bg-primary/90"

<!-- Collapsible chevron rotation -->
class="... transition-transform duration-200 group-open:rotate-180"
```

Always pair transitions with `motion-reduce:transition-none` for accessibility:
```html
class="... transition-colors duration-150 motion-reduce:transition-none"
```

Tailwind v4 has `motion-reduce:` as a built-in variant. `@media (prefers-reduced-motion)`
automatically disables these when the user has enabled that OS setting.

**Duration rule:** 100–200ms for micro-interactions (hover, focus), 200–300ms for panel
reveals (collapsible, modal). No transition longer than 300ms on UI elements.

### 3. Custom Select Arrow

The native `<select>` arrow is browser-rendered and cannot be themed with color tokens.
Replace it with an SVG data URI via `appearance-none`.

```html
class="... appearance-none bg-no-repeat bg-right pr-10"
style="background-image: url(\"data:image/svg+xml,...\")"
```

The recommended SVG pattern (URL-encoded, works cross-browser, respects color tokens via
oklch in the data URI):

```
background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3E%3Cpath stroke='%236b7280' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M6 8l4 4 4-4'/%3E%3C/svg%3E")
```

The stroke color (`%236b7280` = `#6b7280`) is hardcoded in the URI but can be a Tailwind
class of `text-text-muted` applied to a wrapper — or just hardcode a neutral gray that works
in both light and dark modes (gray at ~50% lightness is safe).

**Note on `appearance: base-select`:** Chrome 135 (March 2025) introduced this but it's not
cross-browser (no Firefox/Safari support yet). Do not use it. Stay with `appearance-none` +
SVG for now.

### 4. Checkbox and Radio Styling

**Use `accent-color` first.** It is the simplest approach and is fully accessible because
the browser maintains all native semantics and keyboard interaction.

```html
class="accent-primary h-4 w-4"
```

Tailwind v4 has `accent-{color}` utilities built-in. `accent-primary` will use the
`--color-primary` token.

The `appearance-none` + full custom CSS approach (using `::before`/`::after` pseudo-elements)
gives more visual control but is more code and creates accessibility maintenance burden
(keyboard states, indeterminate state, RTL support). Use it only if `accent-color` doesn't
produce sufficient visual quality — which it won't for highly custom designs but is adequate
for professional utility UI.

**Decision:** Use `accent-primary` in render.rs. Reserve the custom approach for theme
authors who need full control via `custom_head`.

### 5. Oklahoma Color Science (oklch best practices)

The existing `default.css` already uses oklch correctly. The key rules to enforce when
updating tokens or designing new themes:

- **Lightness governs contrast.** For text on background: minimum Δ lightness of 45% (e.g.,
  text at L=15% on background at L=100%, or text at L=95% on background at L=12%).
- **Chroma drives vibrancy.** Semantic role colors (primary, destructive, success, warning)
  should have C ≥ 0.15 for clear visual differentiation. Neutral surfaces use C=0.
- **Hue consistency.** All primary-family colors use the same hue; shift lightness/chroma
  for variants (`primary/90` opacity works in Tailwind).
- **Dark mode:** Increase L on role colors by ~10 points in dark mode (the existing default
  does this: L=55 → L=65 for primary).
- **WCAG 2.2 target:** 4.5:1 contrast ratio for body text, 3:1 for large text and UI
  components. Check with the OKLCH lightness difference; for neutrals Δ L ≈ 50 ≈ 4.5:1.

The existing defaults pass these checks. No changes needed to the color science, only to
component application (render.rs class strings).

### 6. Dark Mode Pattern in @theme

The existing `default.css` uses `@media (prefers-color-scheme: dark)` with a nested `@theme`
block plus a `[data-theme="dark"]` selector for explicit toggling. This is the correct
pattern for the ferro-theme CDN use case.

The shadcn/ui pattern of `@theme inline` + CSS variables at `:root` / `.dark` is designed
for build pipelines, not the CDN. Do not switch to that pattern.

Current pattern works. No changes needed to the dark mode architecture.

---

## Alternatives Considered

| Recommended | Alternative | Why Not |
|-------------|-------------|---------|
| Bunny Fonts CDN | Google Fonts CDN | GDPR violation risk; Google collects user IPs |
| Bunny Fonts CDN | Self-hosted fonts | Adds static asset setup friction for users |
| `accent-color` for checkboxes | `appearance-none` custom CSS | More complexity, more code, equivalent quality for utility UI |
| `focus-visible:` | `focus:` | `focus:` shows rings on mouse clicks — incorrect UX |
| `motion-reduce:transition-none` | Skip accessibility flag | Required by WCAG 2.1 for users with vestibular disorders |
| SVG data URI for select arrow | CSS `::after` + clip-path wrapper | Wrapper requires extra HTML element; render.rs generates no wrapper divs around selects |
| `appearance: base-select` | Not yet — no cross-browser support | Chrome 135+ only as of March 2025 |

---

## What NOT to Add

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Animate.css / tw-animate-css | Ferro JSON-UI needs no keyframe animations; transitions suffice | `transition-*` Tailwind utilities |
| Custom JavaScript animation library | Zero-JS is a JSON-UI design goal | CSS `transition` + `motion-reduce:` |
| Alpine.js | Would compromise the zero-JS philosophy | Existing vanilla JS runtime in `runtime.rs` |
| New Rust CSS-in-Rust libraries (e.g., `grass`) | Overkill; CSS is already compiled by the browser CDN | Direct string generation in `render.rs` |
| `@tailwindcss/typography` plugin | CDN mode doesn't support plugins via npm; would need build pipeline | Handcrafted prose styles in `Text` component |
| Geist as default font | Inter has better coverage, more recognizable professional aesthetic | Use Inter; make Geist available as theme example |
| Google Fonts | GDPR violation risk in EU deployments | Bunny Fonts |

---

## Integration Points in Existing Code

| Change | File | What to Change |
|--------|------|---------------|
| Fix font token namespace | `ferro-theme/assets/default.css` | `--font-family-sans` → `--font-sans`; `--font-family-mono` → `--font-mono` |
| Add Inter font import | `ferro-theme/assets/default.css` | Add `@import url('https://fonts.bunny.net/...')` before `@theme` block |
| Fix font token names in Rust constants | `ferro-theme/src/token.rs` | Rename `TOKEN_FONT_FAMILY_SANS` → `TOKEN_FONT_SANS`, update value to `--font-sans` |
| Add `focus-visible:` to interactive components | `ferro-json-ui/src/render.rs` | All Button, Input, Select, Checkbox, Switch, Tab, Breadcrumb, Pagination renderers |
| Add `transition-colors motion-reduce:transition-none` | `ferro-json-ui/src/render.rs` | Button, Input, Select, Collapsible |
| Add `appearance-none` + SVG arrow to select | `ferro-json-ui/src/render.rs` | `render_select()` function |
| Replace `accent-{hardcoded}` with `accent-primary` | `ferro-json-ui/src/render.rs` | `render_checkbox()`, `render_switch()` |
| Fix hardcoded colors in runtime JS | `ferro-json-ui/src/runtime.rs` | Toast `VARIANT_CLASSES` uses `bg-blue-500` etc. — should use semantic tokens |
| Fix hardcoded colors in tab switching JS | `ferro-json-ui/src/runtime.rs` | Tab switcher uses `border-blue-600`, `text-blue-600` — should use semantic tokens |

---

## Version Compatibility

| Package | Pinned To | Notes |
|---------|-----------|-------|
| `@tailwindcss/browser` | `@4` (resolves to latest 4.x) | Current: 4.2.2. Pinning to `@4` rather than `@4.2.2` is intentional — patch updates are safe |
| Bunny Fonts Inter | `latest` via CDN URL | No versioning in Bunny Fonts URLs; always serves current |

No new Rust crates needed. No new npm packages needed.

---

## Sources

- [Tailwind v4 theme variables documentation](https://tailwindcss.com/docs/theme) — `@theme` syntax, token namespaces (HIGH confidence)
- [Tailwind v4 font-family documentation](https://tailwindcss.com/docs/font-family) — confirmed `--font-sans` not `--font-family-sans` (HIGH confidence)
- [Tailwind v4 Play CDN documentation](https://tailwindcss.com/docs/installation/play-cdn) — `type="text/tailwindcss"` pattern for `@theme` in CDN mode (HIGH confidence)
- [jsDelivr @tailwindcss/browser](https://www.jsdelivr.com/package/npm/@tailwindcss/browser) — current version 4.2.2 (HIGH confidence)
- [Bunny Fonts](https://fonts.bunny.net/) — GDPR-compliant font CDN, Inter available (HIGH confidence)
- [Fontsource Inter CDN](https://fontsource.org/fonts/inter/cdn) — jsDelivr variable font URL (MEDIUM confidence, jsDelivr alternative to Bunny)
- [Tailwind v4 hover/focus states](https://tailwindcss.com/docs/hover-focus-and-other-states) — `focus-visible:` pattern confirmed (HIGH confidence)
- [Modern CSS custom select styles](https://moderncss.dev/custom-select-styles-with-pure-css/) — `appearance-none` + SVG pattern (HIGH confidence)
- [CSS accent-color MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/accent-color) — checkbox/radio accent-color (HIGH confidence)
- [prefers-reduced-motion MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@media/prefers-reduced-motion) — motion accessibility (HIGH confidence)
- [OKLCH color contrast guide](https://medium.com/@vyakymenko/color-contrast-with-oklch-prefers-reduced-motion-and-motion-design-ethics-089c0c8897d0) — oklch contrast rules (MEDIUM confidence)
- [shadcn/ui Tailwind v4 migration](https://ui.shadcn.com/docs/tailwind-v4) — confirmed `@theme inline` pattern is for build pipelines (HIGH confidence)

---

*Stack research for: v10.0 JSON-UI Visual Overhaul*
*Researched: 2026-03-24*
