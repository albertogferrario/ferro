# Pitfalls Research

**Domain:** Visual polish overhaul — SSR component system with Tailwind CSS
**Researched:** 2026-03-24
**Confidence:** HIGH

---

## Critical Pitfalls

### Pitfall 1: Test Avalanche — 157 Snapshot-Style Tests Assert on Exact Class Strings

**What goes wrong:**
Every visual change to a component triggers cascading test failures because 157 tests in `render.rs` assert on exact Tailwind class strings (e.g., `assert!(html.contains("bg-primary text-primary-foreground hover:bg-primary/90"))`). Changing `rounded-md` to `rounded-lg` on a button breaks the button variant tests, button size tests, and any container tests that embed buttons. A single focused spacing pass touching 10 components can produce 40+ failures simultaneously — making it impossible to distinguish regressions from intentional changes.

**Why it happens:**
The tests were written to verify functional correctness of the renderer. They use `contains()` on class substrings, which is appropriate for verifying structure (e.g., "button has a disabled attribute") but was also used for visual classes that are now the subject of change. There is no distinction between structural/behavioral assertions and cosmetic ones.

**How to avoid:**
Before any visual pass, audit the test suite and separate assertions into two categories:
1. **Structural** — HTML element type, attributes (id, name, type, disabled, required, checked), data attributes (data-tabs, data-tab, role), and semantic tokens (bg-primary, text-destructive). These must keep passing.
2. **Cosmetic** — Spacing (px-4, py-2), radius (rounded-md), shadow classes. These will change and must be updated to match intent, not locked.

For the cosmetic category, either update the tests as part of the same PR as the visual change, or convert them to assert on semantic meaning instead of literal strings. Do not batch a visual overhaul with a test refactor in the same commit — keep them separate so failures are attributable.

**Warning signs:**
- A diff touching one function in `render.rs` causes 20+ test failures
- Tests fail in components you did not touch (cascade from shared class string)
- PR description says "visual polish" but shows 40 test file changes

**Phase to address:**
The first visual phase (typography or surface/elevation). Establish the separation rule before touching any class strings.

---

### Pitfall 2: oklch Contrast Failure in Dark Mode — Light Values Break Accessible Ratios

**What goes wrong:**
The default theme uses oklch values that are visually balanced in light mode but can fail WCAG AA (4.5:1) in dark mode if tweaked without checking contrast. Specifically: `--color-text-muted: oklch(60% 0 0)` on a `--color-card: oklch(20% 0 0)` background produces a contrast ratio near 3:1 — acceptable for large text only. Adding surface elevation (slightly lighter cards) narrows this gap further. A developer adjusting `--color-card` from `oklch(20%)` to `oklch(22%)` to add depth will not notice the 0.3:1 contrast reduction until it is flagged by an accessibility audit.

**Why it happens:**
oklch lightness (L) is perceptual but WCAG 2.x contrast ratio is calculated in sRGB luminance space. The relationship is non-linear. Moving L from 20% to 22% feels trivial but changes sRGB luminance by a meaningful percentage. Standard color pickers do not show WCAG ratios in real time. The W3C has confirmed WCAG 2.x calculations apply to oklch after conversion to sRGB — so the math works, but you must measure it.

**How to avoid:**
After any theme token change, run the following critical pairs through a contrast checker (OddContrast accepts oklch natively):
- `--color-text` on `--color-background` (must be >= 7:1 for body text)
- `--color-text-muted` on `--color-background` (must be >= 4.5:1)
- `--color-text-muted` on `--color-surface` (must be >= 4.5:1)
- `--color-text-muted` on `--color-card` (must be >= 4.5:1)
- `--color-primary-foreground` on `--color-primary` (must be >= 4.5:1)
- `--color-destructive` text on white/light background (must be >= 4.5:1)

Check both light AND dark mode values. Do this before writing a single line of Rust — token changes are CSS-only and can be verified in isolation.

**Warning signs:**
- Card background moves lighter (higher L) without a corresponding adjustment to text-muted
- "Elevation" pass increases card L values by more than 2-3% without contrast check
- Badge or alert colors use `text-warning` directly on `bg-background` without a background tint

**Phase to address:**
Surface/elevation system phase. Also applies to any theme token refinement phase.

---

### Pitfall 3: Tailwind CDN WASM Blocks First Paint — Semantic Classes Not Emitted at Build Time

**What goes wrong:**
The framework currently uses `<script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4"></script>` for development (the `tailwind_cdn: true` config). The browser CDN downloads a ~300KB WASM bundle, scans the page HTML, and generates CSS in-browser before first paint. This means: (1) pages appear unstyled for 200-800ms on first load depending on connection, (2) the WASM bundle must re-scan on every page navigation, and (3) arbitrary dynamic classes emitted from plugins or third-party code may not be detected if they are not present in the initial HTML scan. During the polish phase, adding new Tailwind utility classes is safe because they will be picked up on the next scan — but it creates a false sense that the system scales to production.

Additionally, Tailwind's official documentation states the CDN is development-only and must not be used in production. Any deployment of ferro apps to production requires a proper Tailwind build step.

**Why it happens:**
The CDN was chosen for zero-config developer experience, which is correct. The pitfall is not in the choice but in: (a) adding focus styles or transitions that rely on classes not present in the initial scan window, and (b) shipping CDN config to production apps.

**How to avoid:**
For the visual polish milestone: add new classes freely — the CDN will pick them up. But document clearly in `JsonUiConfig` that `tailwind_cdn: false` requires a build step and provide guidance. When testing the polish output, do a hard reload (not soft navigation) between changes to ensure the CDN re-scans. Watch for brief FOUT between CDN load and CSS injection — this is expected in dev but a signal for production gap.

**Warning signs:**
- Components look correctly styled after a soft navigation but unstyled on first hard load
- A class added in render.rs doesn't appear to work until you reload twice
- Production app shows unstyled flash for 300-500ms

**Phase to address:**
Font loading phase (typography). Add a `PRODUCTION NOTE` to JsonUiConfig documentation at the start. A separate production-build phase may be needed post-overhaul.

---

### Pitfall 4: Custom `<select>` Arrow Disappears on Windows Chrome — `appearance-none` Removes Native Indicator Without Replacement

**What goes wrong:**
The current `render_select` produces `appearance-none bg-background rounded-md border ...`. `appearance-none` removes the native dropdown arrow across all browsers, but the replacement arrow (typically a CSS background-image SVG) is never added. On macOS Chrome, this is less noticeable because the OS renders a subtle indicator. On Windows Chrome and Firefox, the select field appears as a plain rectangle with no affordance that it is interactive. Users do not know it is a dropdown.

**Why it happens:**
`appearance-none` is the correct first step to custom styling, but it must be paired with a custom arrow indicator. The current renderer removed the default arrow for styling consistency but never replaced it. This is a well-known trap: the class name (`appearance-none`) does not hint at what it removes.

**How to avoid:**
Wrap the `<select>` in a `relative` container and add an SVG chevron as an absolutely-positioned `pointer-events-none` element. The standard pattern:
```html
<div class="relative">
  <select class="block w-full appearance-none bg-background rounded-md border ... pr-8">...</select>
  <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2 text-text-muted">
    <svg class="h-4 w-4" ...chevron-down...</svg>
  </div>
</div>
```
The `pr-8` on the select prevents text from overlapping the arrow.

**Warning signs:**
- `appearance-none` present in select HTML without a wrapper `relative` container
- No SVG or background-image arrow visible when inspecting the select element
- Testing only on macOS (where it looks acceptable) but not on Windows or Firefox

**Phase to address:**
Component polish phase — specifically form field components (Input, Select, Checkbox, Switch).

---

### Pitfall 5: Modal `fixed` Overlay Trapped in Parent Stacking Context

**What goes wrong:**
The current modal renders as a `<details>` element with a child `<div class="fixed inset-0 z-50 ...">`. This `position: fixed` + `z-index: 50` should overlay the entire viewport. However, if the modal's ancestor has `transform`, `will-change: transform`, `opacity < 1`, `filter`, or `isolation: isolate` applied, a new stacking context is created and the `fixed` div is positioned relative to that ancestor — not the viewport. The `DashboardLayout` sidebar is a candidate: any transition or transform added to the sidebar during the polish phase (e.g., slide-in animation) will trap any modal rendered inside it.

**Why it happens:**
CSS `position: fixed` is defined to be relative to the initial containing block (viewport), but CSS transforms create a new containing block. This is a spec-compliant behavior that surprises nearly everyone. Adding `transition-transform` to the sidebar for a mobile slide-in effect creates exactly this trap.

**How to avoid:**
Before adding any `transform` or `transition` to container elements in layouts, verify that no modal components are rendered inside them. The `<details>`-based modal approach is vulnerable to this. If sidebar animations are added: either (1) render modal HTML outside the sidebar DOM tree (requires layout restructuring), or (2) avoid transform-based animations on layout containers, using `translate` CSS property instead (which does NOT create a containing block in modern browsers). Alternatively, migrate to `<dialog>` element which exists outside the normal stacking context.

**Warning signs:**
- Adding `transition-transform` to the sidebar causes modal overlay to not cover the full viewport
- Modal backdrop stops at the sidebar edge
- z-index: 9999 doesn't fix the modal from being trapped

**Phase to address:**
Layout fixes phase (DashboardLayout). Must be checked before adding any CSS transitions to layout containers.

---

### Pitfall 6: Font Loading Causes CLS and FOUT — Wrong `font-display` Strategy

**What goes wrong:**
When a professional font (e.g., Inter from Google Fonts) is loaded to replace the system font stack, two failure modes exist:
1. **FOUT (Flash of Unstyled Text):** The browser renders text in the fallback font, then swaps to the loaded font. The visual jump causes Cumulative Layout Shift (CLS) if the fonts have different metrics (different line-height, cap-height, x-height, character width).
2. **Render blocking:** If the font `<link>` is placed without `rel="preload"` or without `font-display: swap`, the browser blocks rendering until the font downloads.

With the CDN approach, both the Tailwind WASM bundle and the font file compete for bandwidth on first load. If a Google Fonts `@import` is placed inside the `<style type="text/tailwindcss">` tag (i.e., inside the theme CSS), the CDN must process it — adding additional latency before CSS is available.

**Why it happens:**
Font loading is treated as a CSS concern and placed in the theme file. The `@import url("https://fonts.googleapis.com/...")` inside `@theme` causes the CDN to attempt to process external CSS, which either fails silently or adds a dependency chain.

**How to avoid:**
- Load the font via a `<link rel="preload" as="font">` in the HTML `<head>`, not inside the theme CSS.
- Use `font-display: swap` in the `@font-face` declaration to avoid render blocking.
- Specify fallback font metrics that match the target font using `size-adjust`, `ascent-override`, and `descent-override` to minimize CLS.
- In the framework: add the font `<link>` tags to the layout's `<head>` section, not to the theme CSS injected via `<style type="text/tailwindcss">`.
- The theme CSS should only define `--font-family-sans: 'Inter', ui-sans-serif, system-ui, sans-serif` — not load the font.

**Warning signs:**
- Font `@import` placed inside the theme CSS file (`assets/default.css`)
- Text visibly reflows when the page finishes loading
- Chrome Lighthouse reports CLS > 0.1 after font swap
- No `rel="preload"` for font files in the layout `<head>`

**Phase to address:**
Typography foundation phase (first phase of the overhaul).

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Hardcode Tailwind spacing in render.rs instead of consuming theme tokens | Faster to implement | Themes cannot override spacing; visual inconsistency between themed and default rendering | Never — theme tokens exist for this |
| Use CDN in "production" docs without noting build requirement | Zero-config for demos | Users ship 300KB WASM bundle to real users; FOUT on every page | Never — document clearly |
| Assert on exact class strings in new tests | Easy to write | Polish phases break tests; developers lose confidence in the test suite | Acceptable only for structural/behavioral assertions, not cosmetic classes |
| Add emoji icons in components during polish | Fast placeholder | Emoji rendering varies by OS (Apple vs Android vs Windows); breaks visual consistency | Never — use SVG |
| Apply `transform` to layout containers for animations | Smooth slide-in effect | Traps fixed modals; breaks stacking context | Only when no modals are children of that container |
| Skip dark mode verification for "cosmetic" changes | Faster iteration | Dark mode contrast failures; light-mode-only visual testing | Never — always check both modes |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Tailwind CDN + theme CSS injection | Placing `@import url(google fonts)` inside `<style type="text/tailwindcss">` | Load fonts via separate `<link>` in `<head>`; theme CSS defines font-family token only |
| Tailwind CDN + dynamic classes | Adding a class in Rust that never appears in static HTML | Class is not in CDN scan window; must ensure it appears in rendered HTML or use safelist mechanism |
| `peer` modifier + Switch component | Assuming `peer` works across arbitrary DOM nesting | `peer` requires strict sibling relationship; the `<input>` must be the immediate previous sibling |
| `appearance-none` + Select | Removing native arrow without replacement | Always pair with a wrapper `div.relative` and an absolutely-positioned SVG chevron |
| oklch colors + WCAG tools | Using only sRGB-based contrast checkers | Use OddContrast or Atmos which accept oklch natively; avoid converting manually |
| `@theme` inside `@media` | Tailwind v4 `@theme` inside `@media (prefers-color-scheme: dark)` | v4 supports this but it overrides at the theme level, not via CSS cascade; confirm behavior matches intent in the CDN browser build |
| `details`/`summary` modal + Safari | Assuming `group-open:block` shows the modal overlay | Safari does not include details content in keyboard focus order; test tab navigation explicitly |
| Plugin CSS assets + theme | Plugin CSS loads after theme CSS | Plugin may use hardcoded colors that conflict with theme tokens; plugins should use semantic token classes, not literal values |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| CDN re-scan on every page load | 200-800ms FOUC on first visit | Use CDN only in dev; build CSS for production | Every production page load |
| Large inline SVG icons | Slow HTML parse, no caching | Use `<use>` with external SVG sprite or data-URI for small icons | When >5 components have inline SVGs |
| Inline `style=` overrides instead of Tailwind | Blocks CSS batching in browser | Use Tailwind utilities or CSS variables; avoid `style=` for visual properties | Not a scale issue — a maintainability issue from day 1 |
| Font loaded synchronously without preload | Render-blocking first paint | Add `<link rel="preload" as="font">` before other resources | Every page for users on slow connections |
| CLS from font swap with mismatched metrics | Visible text reflow after ~200ms | Use `size-adjust` / `ascent-override` / `descent-override` fallback | Users on slower connections who see fallback font briefly |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Injecting theme CSS without escaping | XSS if theme is loaded from filesystem and tampered | Current implementation uses `include_str!` (safe); `from_path()` reads untrusted CSS — validate it does not contain `</style>` sequences |
| Adding SVG icons with `innerHTML`-style injection | XSS if icon content comes from user data | Icons must be hardcoded SVG strings in Rust source, not data from API or user input |
| Raw HTML in icon fields passed to `data-icon` attribute | Attribute injection if icon name contains `"` | Current `html_escape()` handles this correctly; maintain this for any new icon attributes |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Focus rings removed for aesthetics | Keyboard-only users cannot see focus position; WCAG 2.4.7 violation | Use `:focus-visible` ring (visible only for keyboard, not mouse); never remove focus indicators |
| Hover-only state changes with no focus equivalent | Keyboard users never see hover states | Every `hover:` class should have a matching `focus-visible:` counterpart |
| Transition on every interactive element | Motion sickness for users with vestibular disorders | Wrap all `transition-*` classes in `motion-safe:` modifier; use `@media (prefers-reduced-motion: reduce)` |
| Select field with no visible arrow indicator | Users don't know it is interactive; high error rates | Always provide visual affordance for all interactive elements |
| Empty state components without actionable next step | Users feel lost with no guidance | EmptyState should always include at least one primary action button |
| Modal without keyboard close (Escape) | Trapped keyboard users | The `<details>`-based modal has no Escape key support; document this limitation |

---

## "Looks Done But Isn't" Checklist

- [ ] **Dark mode tokens:** Verify every new or changed token passes contrast checks in BOTH light and dark mode — not just light mode
- [ ] **Select arrow:** After polishing select styling, confirm a visible dropdown indicator exists on Windows Chrome and Firefox, not just macOS
- [ ] **Focus rings:** After adding focus ring styles, verify they appear on keyboard navigation in Safari (`:focus-visible` landed in Safari 15.4; test explicitly)
- [ ] **Font loading:** After adding a professional font, verify CLS score in Chrome DevTools Performance panel — a zero CLS score is the target
- [ ] **Modal overlay:** After adding any CSS transition to layout container elements, open a modal and verify the overlay covers the full viewport
- [ ] **Tests updated:** After any cosmetic class change, verify the test suite passes — and verify the failures you fixed are the ones you expected to change
- [ ] **Reduced motion:** After adding CSS transitions, confirm `motion-safe:` modifier is applied or `@media (prefers-reduced-motion)` is respected
- [ ] **Emoji-to-SVG:** After replacing emoji icons, verify the SVG renders at the correct size and color in all target browsers
- [ ] **Plugin compatibility:** After changing base component classes, verify that Leaflet map plugin (and any other registered plugins) still renders correctly in context

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Test avalanche from class changes | LOW | Run `cargo test 2>&1 \| grep FAILED` to list failures; update cosmetic assertions to match new classes; keep structural assertions unchanged |
| Dark mode contrast failure | LOW | Adjust only the L value of the failing token; use OddContrast to verify; re-run contrast check on all pairs |
| CDN shipped to production | MEDIUM | Add Tailwind build step to deployment; update JsonUiConfig docs; bump minor version |
| Select arrow missing | LOW | Add `relative` wrapper + SVG chevron to `render_select` in render.rs; update the 3-4 related tests |
| Modal trapped in stacking context | MEDIUM | Move modal rendering outside transformed container; or replace `<details>` modal with `<dialog>` element |
| Font causing CLS | MEDIUM | Move font loading to layout `<head>` with preload; add fallback font metric overrides; measure with Lighthouse |
| Peer modifier broken in Safari | LOW | Verify Safari 16.4+ requirement is met; test with actual Safari; peer modifier requires strict sibling DOM structure |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Test avalanche from class changes | Phase 1 (Typography / first visual pass) — establish test separation rule first | `cargo test` passes with zero unexpected failures after each component change |
| oklch contrast failure in dark mode | Surface/elevation phase (adds depth to card tokens) | All 8 critical token pairs checked with OddContrast in both light and dark mode |
| CDN FOUT / production gap | Typography phase (introduces font loading) | Lighthouse CLS score; hard reload test; documentation for production build |
| Select arrow missing | Component polish phase (form fields) | Manual test in Windows Chrome + Firefox; visual screenshot |
| Modal trapped in stacking context | Layout fixes phase (DashboardLayout animations) | Open modal inside dashboard layout; verify overlay covers full viewport |
| Font CLS | Typography foundation phase | Chrome DevTools Performance > Layout Shift; Lighthouse CLS < 0.1 |
| Focus ring browser inconsistency | Component polish phase (focus rings) | Keyboard tab navigation test in Safari 17, Chrome, Firefox |
| Peer modifier CSS-only breakage | Component polish phase (Switch component) | Test Switch in Safari 16.4+; verify checked/unchecked toggle visual |
| Emoji to SVG rendering inconsistency | Consistency pass phase | Screenshot comparison across macOS, Windows, and Linux |
| Plugin compatibility after base class changes | Component polish phase | Run full test suite including plugin tests; verify Leaflet map renders |

---

## Sources

- Tailwind v4 compatibility and browser requirements: [Tailwind CSS Compatibility Docs](https://tailwindcss.com/docs/compatibility)
- Tailwind v4 CDN development-only note: [Play CDN docs](https://tailwindcss.com/docs/installation/play-cdn)
- WCAG OKLCH contrast calculation clarification: [w3c/wcag Discussion #4559](https://github.com/w3c/wcag/discussions/4559)
- OKLCH accessibility and contrast: [LogRocket — OKLCH in CSS](https://blog.logrocket.com/oklch-css-consistent-accessible-color-palettes)
- OddContrast tool (accepts oklch natively): [OddContrast](https://www.oddcontrast.com/)
- Custom select styling cross-browser: [Modern CSS Custom Select Styles](https://moderncss.dev/custom-select-styles-with-pure-css/)
- CSS select appearance-none cross-browser: [LogRocket Custom Select Dropdown](https://blog.logrocket.com/creating-custom-select-dropdown-css/)
- Stacking context and fixed position / transform trap: [Smashing Magazine — Unstacking CSS Stacking Contexts](https://www.smashingmagazine.com/2026/01/unstacking-css-stacking-contexts/)
- Z-index stacking context reference: [MDN Stacking context](https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Positioned_layout/Stacking_context)
- Focus ring / :focus-visible browser interop: [WebKit blog :focus-visible](https://webkit.org/blog/12179/the-focus-indicated-pseudo-class-focus-visible/)
- details/summary Safari limitations: [Can I Use — details](https://caniuse.com/?search=details)
- details/summary modal vs dialog: [web.dev Details and Summary](https://web.dev/learn/html/details)
- Font loading CLS and FOUT: [Ramotion — Optimizing Web Fonts](https://www.ramotion.com/blog/optimizing-web-fonts-for-performance/)
- Tailwind v4 @theme theming discussion: [GitHub Discussion #18471](https://github.com/tailwindlabs/tailwindcss/discussions/18471)
- Brittle test strategy: [When to Use Jest Snapshots](https://selleo.com/blog/when-to-use-jest-snapshots)
- Personal analysis of ferro-json-ui render.rs (157 test functions), default.css (oklch tokens), layout.rs (details-based modal, stacking context exposure), and framework/src/json_ui/mod.rs (CDN injection path)

---
*Pitfalls research for: JSON-UI visual overhaul — SSR component system with Tailwind CSS*
*Researched: 2026-03-24*
