# Feature Landscape: JSON-UI Visual Overhaul

**Domain:** SSR component system — visual quality uplift to professional grade
**Researched:** 2026-03-24
**Milestone:** v10.0 JSON-UI Visual Overhaul

---

## Context: Current State

Ferro JSON-UI has a functionally complete component catalog (~30 components). The rendering pipeline exists and emits Tailwind CSS classes against a semantic token vocabulary. The gap is visual quality — the output looks like "Tailwind defaults applied once" rather than "a considered design system."

Reference systems studied: shadcn/ui (Tailwind v4 + Radix), Vercel Geist, Radix Themes, Tailwind UI.

---

## Table Stakes

Features users expect from any professional component system. Absence makes the output feel amateur.

| Feature | Why Expected | Complexity | Current State |
|---------|--------------|------------|---------------|
| Typography scale with consistent line-height | Every professional system (shadcn, Geist, Tailwind UI) defines explicit type ramp with line-height and letter-spacing per level, not just font-size | Low | `text-3xl font-bold`, `text-2xl font-semibold` — sizes exist but no line-height, letter-spacing, or responsive scaling |
| Professional font | Inter (71% of professional apps), Geist Sans, or equivalent — system-ui alone reads as "unstyled" | Low | No font load; falls back to system-ui |
| Surface/card contrast | Cards must read as visually elevated from page background. Geist uses Background1/Background2 separation. shadcn has `--card` token distinct from `--background`. Missing = cards look flat | Low-Med | Card uses `bg-background` — same token as page body, so no visual elevation |
| Focus rings on all interactives | WCAG 2.4.7 (Level AA). Professional systems (Radix Themes, shadcn) use `focus-visible:outline` or `focus-visible:ring-2` with 2px offset. `focus:ring-1` on inputs only partially covers this | Med | Inputs have `focus:ring-1 focus:ring-primary`. Buttons have no focus style. Select, checkbox, switch inconsistent |
| Hover states on all interactives | Every interactive element (table rows, nav items, pagination, collapsible trigger) needs a hover state. Missing = UI feels unresponsive | Low | Partial: buttons have hover, table rows have none, pagination links have hover |
| Custom select arrow | Native `<select>` arrow is browser-rendered and looks inconsistent cross-browser. `appearance-none` without a replacement arrow looks broken — current state | Low | `appearance-none bg-background` on select but no replacement arrow SVG |
| Input error states | Red border + error message below input — current state is partially correct. Missing: red focus ring variant for error state | Low | `border-destructive` on error but `focus:ring-primary` still applies (wrong color for error state) |
| Transition-colors on all interactive elements | 150ms ease transitions on color changes. shadcn uses `transition-colors duration-150`. Current buttons have `transition-colors` but most other interactives do not | Low | Buttons: yes. Everything else: no |
| Consistent spacing scale | 8px grid discipline throughout. All padding and gap values should be multiples of 4 (0.25rem = 4px). Mixing `py-3`, `py-3.5`, `py-4` in adjacent components creates visual jitter | Med | Inconsistent: `py-2`, `py-3`, `p-6`, `p-4`, `py-3.5` mixed without clear system |
| Empty state design | Centered icon + title + description + optional CTA. EmptyState component exists but visual treatment needs review | Low | EmptyState component exists |
| Skeleton loading shimmer | `animate-pulse` is minimal. Professional systems use shimmer gradient animation | Low | `animate-pulse bg-card` only — no shimmer |
| Table row hover | Rows with actions need hover feedback. Essential for scannable data tables | Low | No row hover state |
| Dark mode token completeness | All 23 theme tokens must have correct dark variants. Missing a dark variant = components look wrong in dark mode | Med | Unknown — depends on theme CSS file correctness |

---

## Differentiators

Features that distinguish ferro-json-ui from a basic Tailwind component set. Professional apps have these; commodity UI generators do not.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Semantic color tokens on focus rings | Error state gets a red ring, not primary ring. Success state gets green ring. shadcn does this with `--ring` token that components can override. Radix Themes does this per component variant | Med | Requires per-variant ring color logic in render.rs. Currently all focus rings use primary regardless of error state |
| Table row hover highlight | `hover:bg-muted/50` on `<tr>` — Tailwind UI standard pattern. Makes tables feel interactive and scannable | Low | Single CSS class addition |
| Sticky table header | `sticky top-0 bg-background z-10` on `<thead>` — standard for data-heavy views. Tailwind UI and Flowbite both include this in their table patterns | Low-Med | Requires wrapper overflow and sticky positioning change |
| Breadcrumb chevron separator | Replace `/` text separator with an SVG chevron. Every professional system (shadcn, Geist, Tailwind UI) uses chevrons | Low | Currently uses raw `/` span |
| Consistent transition system | All interactives use `transition-colors duration-150 ease-in-out`. Systematic application, not per-component | Low | Currently only buttons have this |
| Select with visible dropdown arrow | SVG arrow via background-image or pseudo-element. `appearance-none` without arrow is a known UX failure | Low | CSS background-image approach works for SSR; no JS needed |
| Disabled state styling | `disabled:opacity-50 disabled:cursor-not-allowed` on all form elements (inputs, selects, checkboxes, switches). Currently only buttons handle disabled | Low | Pattern is clear; apply to all form elements |
| Input description text positioning | Description below label (for context) vs. below input (for hints). Shadcn places description below the input, above error. Current order: label → description → input → error. Should be: label → input → description → error | Low | One reorder change but affects all inputs/selects |
| Collapsible smooth animation | `max-height` transition with CSS for open/close. `<details>` default is instant. Radix and shadcn both animate panel height | Med | Requires inline style or Tailwind trick with `group-open` |
| Active page state in tabs | Active tab needs `font-semibold` in addition to border/color change. Current: only border color and text color change, weight stays constant | Low | One class addition |
| Avatar fallback background | Initials avatar should use `bg-muted` not `bg-card`. Card color as avatar background blends into card context | Low | Token swap |
| Tooltip-style helper text | Inputs with long descriptions can show `?` icon with tooltip on hover. Not in current feature set. Differentiating but Medium complexity | High | New feature; defer to separate phase |

---

## Anti-Features

Features to explicitly NOT build in this milestone.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| JavaScript-powered custom select dropdown | Replaces native `<select>` with a JS-driven listbox for visual control. High complexity, accessibility regression risk, adds JS weight to SSR system | CSS-only arrow background-image on the `<select>` element |
| CSS-in-JS or runtime theming | Requires JS execution, defeats SSR purpose | CSS custom properties defined at `:root` — works perfectly SSR |
| Animation library dependency | Framer Motion, Animate.css add weight. The micro-interactions needed here are all achievable with Tailwind's `transition-*` and `animate-*` utilities | Tailwind utility classes only |
| Dark mode toggle JS | Persisting user preference in localStorage requires JS. SSR renders one mode | Use `prefers-color-scheme` CSS media query for automatic dark mode; no JS toggle |
| Per-component theming props | `Card` with `color="blue"` variant — this is a component library model, not a design system model | Theme tokens set globally; components use semantic tokens |
| Loading spinner component | Spinners on page load are inferior UX to skeleton screens for content areas | Improve skeleton component shimmer |
| Responsive table collapse | Turning tables into card stacks on mobile is a complete rewrite of table rendering | Horizontal scroll wrapper on overflow (already present) |

---

## Feature Dependencies

```
Font load (Inter) → Typography scale (line-heights, tracking) can reference the loaded font
Token completeness (dark variants) → Dark mode surface contrast works
Surface elevation tokens (bg-surface vs bg-card vs bg-background) → Card-on-background contrast
Custom select arrow (CSS background-image) → appearance-none remains; arrow layered on top
Focus ring system → Error state ring variant (error ring requires ring tokens to exist first)
Transition system → Applied universally after other component classes are correct
```

---

## Component-by-Component Gap Analysis

This section maps what each component needs, ordered by impact.

### High Impact (visible on every view)

**Card**
- Gap: `bg-background shadow-sm` — same background as page body means zero contrast
- Fix: Change to `bg-card shadow-sm` — introduces card-as-surface-above-background pattern
- Complexity: Low (token swap)

**Table**
- Gap: No row hover, "Azioni" hardcoded (Italian), column header letter-spacing uses `tracking-wider` but no `text-xs font-semibold uppercase` contrast improvement
- Fix: Add `hover:bg-muted/50` to `<tr>`, externalize "Actions" label or remove hardcoded text, add sticky header option
- Complexity: Low–Med

**Button**
- Gap: No `focus-visible:ring-2 focus-visible:ring-offset-2` — keyboard users get no focus indicator. `transition-colors` present but no easing specification
- Fix: Add `focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-primary` to base classes
- Complexity: Low

**Input / Select / Textarea**
- Gap: Focus ring uses wrong color during error state (`focus:ring-primary` but border is `border-destructive`). No transition on border color. Select has no dropdown arrow. Description above input rather than below.
- Fix: Split focus classes into normal and error variants. Add `transition-colors`. Add SVG arrow to select via `bg-[url(arrow.svg)]` pattern or inline SVG via padding-right + background-position
- Complexity: Med (error variant logic)

**Typography (Text component)**
- Gap: `text-3xl font-bold` for H1 has no `leading-tight tracking-tight`. `text-base` for P has no `leading-relaxed`. Line-height defaults to browser 1.2 for headings, which is visually cramped.
- Fix: Add explicit `leading-*` and `tracking-*` to each text level
- Complexity: Low

### Medium Impact (visible in content views)

**Breadcrumb**
- Gap: `/` text separator. No `aria-current="page"` on last item.
- Fix: Replace with `›` or inline SVG chevron. Add `aria-current`.
- Complexity: Low

**Badge**
- Gap: `rounded-full` for badges is conventional but `rounded-md` (pill with flat sides) is the shadcn default — more versatile. Color treatment `bg-primary/10 text-primary` is correct pattern.
- Fix: Keep current pattern — it is correct. No change needed.
- Complexity: None

**Alert**
- Gap: No icon before alert content (info icon, check icon, warning triangle, error X). Professional alerts universally show an icon.
- Fix: Add inline SVG icons per variant (4 SVGs needed)
- Complexity: Low–Med

**Skeleton**
- Gap: `animate-pulse bg-card` — pulse opacity animation is minimal. Professional systems use shimmer (gradient sweep left-to-right).
- Fix: Use `animate-pulse bg-muted` with Tailwind's `bg-gradient-to-r` and `animate-shimmer` (custom @keyframes in theme CSS)
- Complexity: Med (requires CSS keyframe in theme file)

**Tabs**
- Gap: Active tab only changes color and border. No font-weight change.
- Fix: Add `font-semibold` to active tab classes
- Complexity: Low

**Pagination**
- Gap: `bg-background` on inactive pages — same as page background, no affordance. No border or ring on items.
- Fix: `border border-border bg-background hover:bg-muted` on page links. Active page already has `bg-primary text-primary-foreground`.
- Complexity: Low

**Collapsible**
- Gap: Instant open/close with no animation signal.
- Fix: Add rotating chevron indicator via `group-open:rotate-180 transition-transform` on the summary icon
- Complexity: Low

### Lower Impact (specialized views)

**StatCard** — Current state: unknown (not reviewed in detail). Likely needs same card elevation fix.

**Checklist** — Current state: unknown. Likely needs focus ring on checkbox items.

**Toast** — Current state: unknown. Verify has proper shadow elevation and is not `bg-background`.

**NotificationDropdown** — Current state: `bg-background rounded-lg shadow-lg` — shadow-lg is good but `bg-background` creates same flat card issue.

**Modal** — Current state: `bg-background rounded-lg shadow-lg` — same issue. Needs `bg-card`.

---

## MVP Recommendation for v10.0

The highest leverage changes that produce the most visible quality improvement:

**Phase 1: Foundation** (enables everything else)
1. Load Inter font via `@import url(...)` in theme CSS head injection
2. Add `bg-card` token definition distinct from `bg-background` in default theme
3. Add `bg-muted` token for table/surface subtle backgrounds

**Phase 2: Card/Surface elevation** (biggest visual win)
4. Change Card, Modal, NotificationDropdown from `bg-background` to `bg-card`
5. Add `hover:bg-muted/50` to table rows

**Phase 3: Typography** (fast, high polish)
6. Add `leading-tight tracking-tight` to H1, H2, H3
7. Add `leading-relaxed` to P element
8. Apply consistent font-size scale (no gaps between levels)

**Phase 4: Form polish**
9. Add custom select arrow (CSS background-image)
10. Fix focus ring error variant (red ring when error present)
11. Apply `transition-colors duration-150` to all form elements

**Phase 5: Interaction states**
12. Add `focus-visible:ring-2 focus-visible:ring-offset-2` to Button
13. Add `hover:bg-muted` to sidebar nav items (already partially there)
14. Add `transition-colors` to all interactive elements missing it

**Phase 6: Component polish** (lower priority but visible)
15. Breadcrumb chevron separator
16. Alert icons (4 inline SVGs)
17. Skeleton shimmer animation
18. Active tab font-weight
19. Pagination border treatment

**Defer:**
- Tooltip-style help text (new feature, not polish)
- Sticky table header (layout change, needs design decision)
- Responsive table collapse (out of scope)
- Dark mode toggle JS (out of scope)

---

## Complexity Tiers Reference

| Level | Definition | Examples |
|-------|------------|---------|
| Low | Single class change or token swap. < 30min | Row hover, tab font-weight, button focus ring |
| Low-Med | Multiple coordinated class changes across 1-2 functions. < 2h | Error ring variant, select arrow, description reorder |
| Med | New CSS (keyframes, custom properties) or logic branch. 2-4h | Skeleton shimmer, collapsible animation |
| High | New component feature or JS coordination. > 4h | Tooltip help text, JS-powered custom select |

---

## Sources

- [shadcn/ui Theming](https://ui.shadcn.com/docs/theming) — CSS variable token system, OKLCH color format (HIGH confidence)
- [shadcn/ui Tailwind v4](https://ui.shadcn.com/docs/tailwind-v4) — v4 @theme directive integration (HIGH confidence)
- [Vercel Geist Typography](https://vercel.com/geist/typography) — Type scale taxonomy (MEDIUM confidence — page renders but tokens are proprietary)
- [Vercel Geist Colors](https://vercel.com/geist/colors) — Background1/Background2 elevation tokens, 10-step color scale (HIGH confidence)
- [Radix Themes DeepWiki](https://deepwiki.com/radix-ui/themes/7.2-input-components) — focus-visible implementation, :has() selector patterns (MEDIUM confidence)
- [shadcn Design Principles Gist](https://gist.github.com/eonist/c1103bab5245b418fe008643c08fa272) — 150ms transition standard, shadow tiers (MEDIUM confidence — community source)
- [Modern CSS: Custom Select Styles](https://moderncss.dev/custom-select-styles-with-pure-css/) — CSS-only select arrow techniques (HIGH confidence)
- [Elevation Design Patterns](https://designsystems.surf/articles/depth-with-purpose-how-elevation-adds-realism-and-hierarchy) — 4-6 elevation level discipline (HIGH confidence)
- [Tailwind CSS Tables — Flowbite](https://flowbite.com/docs/components/tables/) — Row hover, sticky header patterns (MEDIUM confidence)
- [Skeleton Screens — NN/g](https://www.nngroup.com/articles/skeleton-screens/) — When/when not to use skeleton states (HIGH confidence)
- [Fonts — HTTP Archive Web Almanac 2025](https://almanac.httparchive.org/en/2025/fonts) — 71% self-hosted adoption (HIGH confidence)
