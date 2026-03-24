# Project Research Summary

**Project:** v10.0 JSON-UI Visual Overhaul
**Domain:** Server-side HTML rendering with CSS design tokens — professional visual quality uplift
**Researched:** 2026-03-24
**Confidence:** HIGH

## Executive Summary

Ferro JSON-UI has a functionally complete component catalog of ~30 components but the rendered output reads as "Tailwind defaults applied once" rather than a considered design system. The research identifies this as a solvable surface problem: no new crates, no new npm packages, and no architectural changes are required. All quality improvements target four existing files — `default.css`, `layout.rs`, `render.rs`, and `runtime.rs` — through targeted CSS class substitutions, semantic surface token corrections, and improved interactive state handling.

The recommended approach is a six-phase visual uplift ordered by dependency and impact. Font loading and token fixes must come first because they underpin everything else. Surface elevation corrections (Card/Modal/Notification backgrounds) deliver the single highest visible quality increase. Focus rings, transitions, and form polish follow as systematic sweeps across all interactive components. The final phase handles lower-impact component details (Alert icons, Skeleton shimmer, Breadcrumb chevron). This ordering ensures each phase builds cleanly on the previous and that test suite impacts are managed upfront.

The key risk is the test suite: 157 tests assert on exact Tailwind class strings, meaning visual changes will cause cascading failures that obscure regressions. This must be addressed in the first phase by establishing a structural vs. cosmetic assertion separation rule. A secondary risk is dark mode contrast: oklch lightness adjustments that look trivial can reduce WCAG contrast ratios meaningfully, requiring systematic verification with an oklch-native contrast checker after any token change.

---

## Key Findings

### Recommended Stack

The existing stack is correct and requires no additions. `@tailwindcss/browser@4` CDN with `<style type="text/tailwindcss">` injection handles all CSS generation. The only change is font loading: **Bunny Fonts** (not Google Fonts) for Inter Variable via `@import` inside the `<style type="text/tailwindcss">` block.

There is one critical pre-existing bug: `ferro-theme/assets/default.css` uses `--font-family-sans` and `--font-family-mono` which generate no Tailwind utilities. The correct namespace for Tailwind v4 is `--font-sans` and `--font-mono`. This must be fixed before any font work has effect.

**Core technologies:**
- `@tailwindcss/browser@4` (CDN): Tailwind v4 in-browser compiler — already working, no changes needed
- Bunny Fonts CDN: GDPR-compliant Inter Variable delivery — replaces Google Fonts recommendation in ARCHITECTURE.md due to EU legal risk
- oklch color space: Perceptually uniform semantic tokens — already correct in `default.css`, no color science changes needed
- Inline SVG strings (Rust): Icon delivery — no external icon library needed; all icons are hardcoded SVG strings in Rust source

**Critical version fix required:**
- `--font-family-sans` → `--font-sans` in `default.css` and `token.rs` constants

### Expected Features

The research classifies all improvements into three tiers based on visual impact and reference system precedent (shadcn/ui, Vercel Geist, Radix Themes, Tailwind UI).

**Must have (table stakes — absence makes output look amateur):**
- Professional font (Inter Variable) with correct `--font-sans` token wiring
- Surface/card contrast: `bg-card` distinct from `bg-background` for elevated surfaces
- Focus rings on all interactive elements using `focus-visible:` (not `focus:`)
- Custom select dropdown arrow (CSS background-image or absolute SVG span)
- Transition-colors on all interactive elements with `motion-reduce:transition-none`
- Typography scale with explicit `leading-*` and `tracking-*` per heading level
- Hover states on all interactive elements (table rows, pagination, nav items)
- Input error states with red focus ring variant (currently uses primary ring on error)
- Consistent 8px spacing scale across components

**Should have (differentiators — visible in production apps):**
- Semantic focus ring variants per state (error gets red ring, success gets green)
- Sticky table headers for data-heavy views
- Breadcrumb SVG chevron separator (replaces raw `/` text)
- Skeleton shimmer animation (replaces `animate-pulse`)
- Alert icons per variant (4 inline SVGs)
- Active tab `font-semibold` weight
- Pagination border treatment for inactive pages
- Collapsible rotating chevron indicator
- Emoji-to-SVG replacement (bell icon in notification dropdown)

**Defer (v2+):**
- Tooltip-style help text on inputs (new feature, not polish)
- Sticky table headers (layout change needing design decision)
- Responsive table collapse (complete rewrite)
- Dark mode toggle with localStorage persistence (requires JS)
- JavaScript-powered custom select dropdown

### Architecture Approach

All changes are within four existing files with no new cross-crate dependencies. The render.rs ↔ ferro-theme boundary remains: render.rs stays token-agnostic and emits Tailwind class strings; the theme CSS injects token values that those classes resolve to. No ThemeContext should be passed into render.rs — theme variation is expressed purely in CSS custom properties.

**Major components and change targets:**

1. `ferro-theme/assets/default.css` — Fix `--font-family-*` → `--font-*` namespace, add Inter `@import`, refine token values
2. `ferro-json-ui/src/layout.rs` — Font `<link>` injection in `base_document()` using Bunny Fonts (not Google Fonts)
3. `ferro-json-ui/src/render.rs` — PRIMARY TARGET: surface bg corrections, focus rings, transitions, select arrow, hover states, typography classes, SVG icons (~1700 lines, all visual changes concentrated here)
4. `ferro-json-ui/src/runtime.rs` — Replace hardcoded `bg-blue-500`/`border-blue-600` with semantic class names (`bg-primary`, `border-primary`, `text-text-muted`)

**Key patterns to enforce:**
- Surface elevation hierarchy: `background` (page) → `surface` (panels, sidebars) → `card` (cards, modals, dropdowns)
- Focus ring standard: `focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2`
- Transition standard: `transition-colors duration-150 motion-reduce:transition-none` for color changes; `transition-transform duration-200` for rotations
- Shadow mapping: cards → `shadow-sm`, modals/popovers → `shadow-md`
- Use module-level `const` for repeated class combinations, not a ClassBuilder abstraction

### Critical Pitfalls

1. **Test avalanche from class string changes** — 157 tests assert on exact Tailwind class strings. One visual pass can trigger 40+ cascade failures, making regressions invisible. Prevention: before any class change, separate tests into structural (must pass) vs cosmetic (update to match intent). Address in Phase 1.

2. **Dark mode oklch contrast failure** — Adjusting card lightness values by 2-3% can reduce WCAG contrast ratios below 4.5:1 without being visually obvious. Check all 8 critical token pairs with OddContrast (oklch-native) after every token change in both light and dark mode.

3. **Font loading causing CLS and FOUT** — `@import url(google fonts)` inside theme CSS creates a CDN processing dependency chain. Font loading belongs in layout `<head>` as a `<link>` tag with `rel="preload"` and `font-display: swap`. Theme CSS defines only the `--font-sans` family stack.

4. **Select arrow missing on Windows/Firefox** — `appearance-none` removes the native select arrow but the current renderer provides no replacement. The field appears as a plain rectangle with no affordance. Fix: wrap in `div.relative` with an absolutely-positioned SVG chevron span.

5. **Modal fixed overlay trapped in stacking context** — Adding `transition-transform` to the sidebar for mobile slide-in creates a CSS containing block that traps `position: fixed` modals inside it. Before adding any transform-based animation to layout containers, verify modals render outside them or use `translate` CSS property instead (which does not create a containing block in modern browsers).

---

## Implications for Roadmap

Based on combined research, the following phase structure is recommended. The ordering is driven by CSS dependency chains: fonts and tokens must exist before components can reference them correctly; surface elevation must be correct before component polish has its full effect; focus rings and transitions should sweep all components in a single pass to avoid incomplete coverage.

### Phase 1: Foundation — Token Fix and Font Loading

**Rationale:** Two pre-existing bugs block all subsequent work. The `--font-family-sans` namespace mismatch means no font token has ever had effect. The test avalanche pitfall must be addressed before any class strings change.
**Delivers:** Working Inter Variable font, correct Tailwind font token wiring, test separation rule established
**Addresses:** "Professional font" table stake, font token namespace bug
**Avoids:** Test avalanche (establish rule first), font CLS pitfall (correct `<link>` in `<head>` pattern, not theme CSS `@import`)
**Files:** `default.css`, `token.rs`, `layout.rs`
**Research flag:** Standard patterns — skip phase research

### Phase 2: Surface Elevation and Dark Mode Tokens

**Rationale:** The single highest-impact visual change is correcting `bg-background` → `bg-card` for cards, modals, and dropdowns. Flat surfaces (cards with same color as page body) are the most visible marker of amateur rendering. Dark mode token completeness must be verified here before any further token changes compound the contrast problem.
**Delivers:** Visual depth hierarchy — cards, modals, and dropdowns visually elevated above page background; all dark mode tokens verified
**Addresses:** "Surface/card contrast" table stake, dark mode token completeness
**Avoids:** oklch contrast failure in dark mode (verify all 8 critical token pairs with OddContrast)
**Files:** `render.rs` (card, modal, stat card, notification dropdown, sidebar), `default.css`
**Research flag:** Standard patterns — well-documented CSS elevation; skip phase research

### Phase 3: Typography Scale

**Rationale:** Typography improvements are pure class additions (`leading-tight tracking-tight` to headings, `leading-relaxed` to body text) with no dependencies other than the font being loaded (Phase 1). Fast execution, high polish perception.
**Delivers:** Professional type ramp with correct line-height and letter-spacing at every heading level
**Addresses:** "Typography scale" table stake
**Files:** `render.rs` (Text component renderer)
**Research flag:** Standard patterns — Tailwind typography utilities are well-documented; skip phase research

### Phase 4: Form Component Polish

**Rationale:** Form components have the most accumulated visual debt: broken select arrow (cross-browser regression), wrong focus ring on error state, missing transitions, and inconsistent spacing. This phase sweeps all form components in a single focused pass.
**Delivers:** Custom select arrow, error-state focus ring variant, `transition-colors` on all form elements, `disabled:opacity-50 disabled:cursor-not-allowed` on all form elements, description positioning fix (label → input → description → error order)
**Addresses:** "Custom select arrow", "Input error states", "Transition-colors" table stakes; "Semantic focus ring variants" differentiator
**Avoids:** Select arrow cross-browser failure (test on Windows Chrome + Firefox), peer modifier breakage in Safari (Switch component)
**Files:** `render.rs` (Input, Select, Textarea, Checkbox, Switch renderers)
**Research flag:** Standard patterns — skip phase research

### Phase 5: Interactive State Consistency

**Rationale:** Focus rings and hover states must be applied as a complete sweep across all interactive elements — partial coverage (e.g., buttons have focus rings but pagination does not) is worse than no coverage because it fails WCAG and creates inconsistent keyboard navigation experience.
**Delivers:** Unified `focus-visible:ring-2 focus-visible:ring-offset-2` on all interactive elements (Button, Tabs, Breadcrumb links, Pagination); `hover:bg-muted/50` on table rows, sidebar nav, pagination; `transition-colors duration-150 motion-reduce:transition-none` universally applied
**Addresses:** "Focus rings on all interactives" and "Hover states on all interactives" table stakes; "Consistent transition system" differentiator
**Avoids:** Focus rings removed for aesthetics pitfall; hover-only without focus equivalent UX pitfall; reduced motion accessibility requirement
**Files:** `render.rs` (Button, Tabs, Breadcrumb, Pagination, Table, Sidebar nav); `runtime.rs` (tab switcher JS using hardcoded `border-blue-600`)
**Research flag:** Standard patterns — skip phase research

### Phase 6: Component Details and Consistency Pass

**Rationale:** Lower-impact but visible polish: Alert icons, Skeleton shimmer, Breadcrumb chevron, active tab weight, pagination borders, emoji-to-SVG. These are independent of each other and of earlier phases, making them safe to batch.
**Delivers:** Alert icons per variant (4 SVGs), Skeleton shimmer keyframe animation, Breadcrumb SVG chevron, active tab `font-semibold`, pagination border treatment, notification bell emoji → SVG, CollapsibleRotating chevron indicator
**Addresses:** Alert icons, Skeleton shimmer, Breadcrumb separator, tab active weight differentiators
**Avoids:** Emoji rendering inconsistency across OS (replace with SVG), large inline SVG performance trap (keep icons small and hardcoded, not dynamic)
**Files:** `render.rs` (Alert, Skeleton, Breadcrumb, Tabs, Pagination, Collapsible); `layout.rs` (notification bell); `default.css` (shimmer keyframe)
**Research flag:** Standard patterns — skip phase research

### Phase Ordering Rationale

- **Token and font fixes precede all component work** — the `--font-family-*` bug means font changes have zero effect until the namespace is corrected; there is no point polishing component typography before the font exists in the page
- **Surface elevation before interactive states** — establishing the correct background hierarchy makes focus ring offsets (`ring-offset-2`) render correctly; on a flat surface the offset creates confusion about the background it is separating from
- **Form components isolated from interaction states** — form components have variant-specific class logic (error state, disabled state) that is more complex than the simple sweep applied to buttons and navigation; isolating the work prevents scope creep
- **Component details last** — the shimmer animation requires a CSS keyframe in `default.css`; adding it last avoids the risk of the keyframe name colliding with any class introduced in earlier phases

### Research Flags

Phases needing deeper research during planning:
- None identified. All patterns in this milestone are well-documented in Tailwind v4, CSS specifications, and the reference systems studied (shadcn/ui, Geist, Radix).

Phases with standard patterns (skip `/gsd:research-phase`):
- **All 6 phases** — source confidence is HIGH across the board; implementation patterns are explicit in ARCHITECTURE.md with code examples ready to apply

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All technology choices verified against official Tailwind v4 docs; Bunny Fonts drop-in confirmed; no inferred patterns |
| Features | HIGH | Gap analysis from direct codebase inspection of render.rs; reference systems studied at shadcn/ui, Vercel Geist, Radix Themes; complexity estimates grounded in actual function scope |
| Architecture | HIGH | All findings from direct codebase inspection of the 4 target files; data flow traced through actual call chains; no inference about internal behavior |
| Pitfalls | HIGH | 5 of 6 critical pitfalls grounded in code inspection (157 tests counted, modal `<details>` pattern confirmed, `appearance-none` without arrow confirmed); oklch/WCAG math sourced from W3C discussion |

**Overall confidence:** HIGH

### Gaps to Address

- **Tailwind v4 arbitrary-value syntax `rounded-[--radius-md]`:** ARCHITECTURE.md notes MEDIUM confidence on whether this generates classes in CDN mode. The recommendation is to avoid this pattern for v10.0 and use standard Tailwind scale classes (`rounded-md`) since the default token values match. Validate with a quick browser test before using arbitrary custom property references.
- **Bunny Fonts `@import` inside `type="text/tailwindcss"`:** STACK.md recommends this placement, but PITFALLS.md warns that `@import` inside the theme `<style>` block can create a CDN processing dependency chain. Resolve by placing the font `@import` inside the injected `<style type="text/tailwindcss">` block but before the `@theme` block — OR move font loading entirely to a separate `<link>` in `base_document()`. The `<link>` approach in the layout `<head>` is recommended for Phase 1 to eliminate ambiguity.
- **Dark mode token completeness:** The current state of dark mode token coverage is marked "Unknown" in FEATURES.md. A verification pass using OddContrast is required at the start of Phase 2 before any further token changes are made.
- **`details`/`summary` modal Safari keyboard trap:** The existing modal uses a `<details>` element which has known Safari limitations for keyboard focus order. This is documented as a known limitation but not fixed in v10.0. Flag for a future modal-to-`<dialog>` migration.

---

## Sources

### Primary (HIGH confidence)
- [Tailwind v4 theme variables documentation](https://tailwindcss.com/docs/theme) — `@theme` syntax, token namespaces, `--font-sans` vs `--font-family-sans`
- [Tailwind v4 Play CDN documentation](https://tailwindcss.com/docs/installation/play-cdn) — CDN development-only requirement; `type="text/tailwindcss"` pattern
- [Tailwind v4 hover/focus states](https://tailwindcss.com/docs/hover-focus-and-other-states) — `focus-visible:` pattern
- [CSS accent-color MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/accent-color) — checkbox/radio accent-color
- [prefers-reduced-motion MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@media/prefers-reduced-motion) — motion accessibility
- [Bunny Fonts](https://fonts.bunny.net/) — GDPR-compliant font CDN, Inter available
- [shadcn/ui Theming](https://ui.shadcn.com/docs/theming) — CSS variable token system, OKLCH color format
- [shadcn/ui Tailwind v4](https://ui.shadcn.com/docs/tailwind-v4) — v4 @theme directive integration
- [Vercel Geist Colors](https://vercel.com/geist/colors) — Background1/Background2 elevation tokens
- [Modern CSS: Custom Select Styles](https://moderncss.dev/custom-select-styles-with-pure-css/) — appearance-none + SVG arrow pattern
- [OddContrast](https://www.oddcontrast.com/) — oklch-native contrast checker
- Direct codebase inspection: `ferro-json-ui/src/render.rs`, `layout.rs`, `runtime.rs`, `config.rs`, `ferro-theme/src/token.rs`, `ferro-theme/assets/default.css`, `framework/src/json_ui/mod.rs`

### Secondary (MEDIUM confidence)
- [Vercel Geist Typography](https://vercel.com/geist/typography) — Type scale taxonomy (page renders but tokens are proprietary)
- [Radix Themes DeepWiki](https://deepwiki.com/radix-ui/themes/7.2-input-components) — focus-visible implementation patterns
- [shadcn Design Principles Gist](https://gist.github.com/eonist/c1103bab5245b418fe008643c08fa272) — 150ms transition standard
- [Tailwind v4 @theme theming discussion](https://github.com/tailwindlabs/tailwindcss/discussions/18471) — CDN @theme behavior edge cases

### Tertiary (LOW confidence)
- None — all findings have at least MEDIUM confidence backing

---
*Research completed: 2026-03-24*
*Ready for roadmap: yes*
