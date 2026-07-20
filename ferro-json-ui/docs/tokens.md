# Token Contract v2 — Reference

ferro-json-ui v2 defines a fixed set of 40 semantic CSS custom properties that all
themes must provide. Components reference only these tokens — no raw color or size
literals appear in skin rules. This document covers:

1. [Schema reference](#1-schema-reference) — every token: name, type, role, light default, dark default
2. [Theming guide](#2-theming-guide) — how to rebrand via `tokens.css` alone
3. [Annotated starter template](#3-annotated-starter-template) — copy-paste `:root {}` + `.dark {}` block

---

## 1. Schema Reference

All 40 token slots. Dark column shows the `.dark {}` override; "same" means the token
does not change between modes (it is set once in `:root`).

### Surface tokens

Background hierarchy from page → panel → card.

| Token | Type | Role | Light default | Dark default |
|-------|------|------|---------------|--------------|
| `--color-background` | color | Page background (60% of viewport) | `oklch(100% 0 0)` | `oklch(16% 0.01 270)` |
| `--color-surface` | color | Raised surface: sidebar, hover states, input backgrounds (30%) | `oklch(97% 0.005 270)` | `oklch(19% 0.01 270)` |
| `--color-card` | color | Card / panel background (one step above surface) | `oklch(100% 0 0)` | `oklch(22% 0.01 270)` |
| `--color-border` | color | Hairline separators, card borders, input borders | `oklch(88% 0.008 270)` | `oklch(30% 0 0)` |
| `--color-text` | color | Primary body text | `oklch(18% 0.01 270)` | `oklch(95% 0 0)` |
| `--color-text-muted` | color | Secondary labels, meta text, empty-state body | `oklch(52% 0.01 270)` | `oklch(60% 0 0)` |

### Role tokens

Semantic color roles for actions and status.

| Token | Type | Role | Light default | Dark default |
|-------|------|------|---------------|--------------|
| `--color-primary` | color | Primary button background; active nav indicator | `oklch(18% 0 0)` | `oklch(95% 0 0)` |
| `--color-primary-foreground` | color | Text rendered on `--color-primary` backgrounds | `oklch(100% 0 0)` | `oklch(18% 0 0)` |
| `--color-secondary` | color | Secondary button background | `oklch(94% 0 0)` | `oklch(25% 0 0)` |
| `--color-secondary-foreground` | color | Text rendered on `--color-secondary` backgrounds | `oklch(18% 0 0)` | `oklch(95% 0 0)` |
| `--color-accent` | color | Decorative accent (same role as primary by default) | `oklch(18% 0 0)` | `oklch(95% 0 0)` |
| `--color-destructive` | color | Destructive / danger actions and badges | `oklch(52% 0.22 25)` | `oklch(60% 0.22 25)` |
| `--color-success` | color | Success / confirmation badges and status dots | `oklch(52% 0.18 145)` | `oklch(60% 0.18 145)` |
| `--color-warning` | color | Warning / caution badges and alerts | `oklch(68% 0.18 75)` | `oklch(65% 0.18 75)` |

### Focus token

| Token | Type | Role | Light default | Dark default |
|-------|------|------|---------------|--------------|
| `--color-ring` | color | `outline` color for `focus-visible` on every interactive element | `oklch(60% 0.01 270)` | `oklch(72% 0.01 270)` |

### Shape tokens

Border radius scale. Elevation rule: flat surfaces use `--radius-sm`; overlays use
`--radius-md`; pill elements use `--radius-full`.

| Token | Type | Role | Light default | Dark default |
|-------|------|------|---------------|--------------|
| `--radius-sm` | radius | Controls and cards (6 px) | `0.375rem` | same |
| `--radius-md` | radius | Overlays: dropdowns, modals, toasts (10 px) | `0.625rem` | same |
| `--radius-lg` | radius | Reserved for large panels (12 px) | `0.75rem` | same |
| `--radius-full` | radius | Pill / badge / status-dot (full round) | `9999px` | same |

### Shadow tokens

Elevation scale. Flat surfaces use no shadow; overlays use `--shadow-md` or
`--shadow-lg`.

| Token | Type | Role | Light default | Dark default |
|-------|------|------|---------------|--------------|
| `--shadow-sm` | shadow | Subtle lift: inputs, small floating elements | `0 1px 2px 0 rgb(0 0 0 / 0.05)` | same |
| `--shadow-md` | shadow | Floating panels: dropdowns, tooltips | `0 4px 6px -1px rgb(0 0 0 / 0.08)` | same |
| `--shadow-lg` | shadow | High-lift overlays: modals, toasts | `0 10px 15px -3px rgb(0 0 0 / 0.08)` | same |

### Typography tokens — font families

| Token | Type | Role | Default | Dark default |
|-------|------|------|---------|--------------|
| `--font-sans` | font-family | Default UI sans-serif stack | `'Geist', ui-sans-serif, system-ui, -apple-system, sans-serif` | same |
| `--font-mono` | font-family | Code, editors, numeric-alignment contexts | `'Geist Mono', ui-monospace, monospace` | same |
| `--font-display` | font-family | Headings and display text (defaults to sans) | `var(--font-sans)` | same |

### Typography tokens — type-scale sizes and weights

Five semantic roles cover the full UI hierarchy. Skin rules compose them as
`font-size: var(--text-{role}-size); font-weight: var(--text-{role}-weight)`.
All type-scale tokens are mode-invariant (set once in `:root`).

| Token | Type | Role | Default | Dark default |
|-------|------|------|---------|--------------|
| `--text-display-size` | font-size | Page titles (`PageHeader` h1/h2) | `1.75rem` (28 px) | same |
| `--text-display-weight` | font-weight | Weight for display text | `600` | same |
| `--text-section-size` | font-size | Card titles, tab labels, group headings | `0.9375rem` (15 px) | same |
| `--text-section-weight` | font-weight | Weight for section text | `600` | same |
| `--text-body-size` | font-size | Default content: table cells, description values | `0.875rem` (14 px) | same |
| `--text-body-weight` | font-weight | Weight for body text | `400` | same |
| `--text-meta-size` | font-size | Muted labels, secondary info, breadcrumb items | `0.8125rem` (13 px) | same |
| `--text-meta-weight` | font-weight | Weight for meta text | `400` | same |
| `--text-micro-size` | font-size | Badges, captions, DataTable column headers | `0.75rem` (12 px) | same |
| `--text-micro-weight` | font-weight | Weight for micro text (controls, badges) | `500` | same |

### Density token

| Token | Type | Role | Default | Dark default |
|-------|------|------|---------|--------------|
| `--spacing` | spacing | Base unit; spacing utilities resolve as `calc(var(--spacing) * N)` | `0.25rem` (4 px) | same |

### Motion tokens

| Token | Type | Role | Default | Dark default |
|-------|------|------|---------|--------------|
| `--motion-duration-fast` | duration | Micro-interactions: hover color, badge appear | `120ms` | same |
| `--motion-duration-base` | duration | Panel open/close, dropdown appear | `220ms` | same |
| `--motion-duration-slow` | duration | Modal entry, toast | `320ms` | same |
| `--motion-ease` | easing | Standard easing curve (settled, no bounce) | `cubic-bezier(0.2, 0, 0.38, 0.9)` | same |

---

## 2. Theming Guide

A consumer rebrand requires only one file: `tokens.css`. The framework skin reads
exclusively from the token variables above — no raw values appear in component rules.

### Minimal brand palette override

To apply a brand palette, override the surface and role tokens in `:root` and `.dark`:

```css
/* tokens.css — brand override example */
:root {
  /* Replace with your brand primary — keep text contrast ≥4.5:1 */
  --color-primary: oklch(40% 0.18 260);           /* brand blue */
  --color-primary-foreground: oklch(100% 0 0);    /* white text on brand */

  /* Optional: shift the neutral surface hue to match */
  --color-background: oklch(100% 0 0);
  --color-surface: oklch(97% 0.004 260);
  --color-card: oklch(100% 0 0);
  --color-border: oklch(88% 0.006 260);
}

.dark {
  --color-primary: oklch(70% 0.18 260);
  --color-primary-foreground: oklch(12% 0 0);
}
```

All status colors (`--color-success`, `--color-warning`, `--color-destructive`) carry
accessible defaults and do not need to change for a palette rebrand.

### Font override

To substitute a different typeface, override `--font-sans` (and optionally
`--font-mono`). The font must be self-hosted or system — no CDN font requests are
permitted by the framework's Content-Security-Policy baseline.

```css
:root {
  --font-sans: 'Inter', ui-sans-serif, system-ui, sans-serif;
  --font-mono: 'JetBrains Mono', ui-monospace, monospace;
}
```

Add your own `@font-face` declarations for the woff2 files above the `:root {}` block.
Geist remains the framework default and ships bundled at `/_ferro/fonts/`.

### Accessibility guardrail — contrast lint

Before shipping any token override, verify that the modified palette passes the contrast
gate:

```sh
ferro design:lint --tokens tokens.css --deny
```

The gate checks every text pair at ≥4.5:1 and every non-text UI pair at ≥3.0:1 in both
light and dark mode blocks. Any pair below the floor fails the build. Pairs checked:

| Pair | Type | Floor |
|------|------|-------|
| `--color-text` vs `--color-background` | text | 4.5:1 |
| `--color-text` vs `--color-card` | text | 4.5:1 |
| `--color-primary-foreground` vs `--color-primary` | text (button) | 4.5:1 |
| `--color-ring` vs `--color-background` | non-text (focus ring) | 3.0:1 |
| `--color-ring` vs `--color-card` | non-text (focus ring on cards) | 3.0:1 |

The starter template below passes this gate with the gestiscilo default palette.
Verify your overrides pass before deploying.

---

## 3. Annotated Starter Template

A complete, copy-paste `tokens.css` derived from the gestiscilo v2 theme. Every
token includes an inline comment explaining its role. This file passes
`ferro design:lint --tokens tokens.css --deny` with zero violations.

```css
/* tokens.css — ferro-json-ui v2 theme contract
   Copy this file, rename it to match your tenant, and override the values below.
   Run `ferro design:lint --tokens tokens.css --deny` to verify accessibility. */

:root {
  /* ── Surface hierarchy ─────────────────────────────────────────────────────
     Three background steps: page (60%) → surface/sidebar (30%) → card (10%).
     All at the same hue; lightness steps create depth without chroma noise. */

  --color-background: oklch(100% 0 0);
  /* Page background — the dominant surface, white in light mode */

  --color-surface: oklch(97% 0.005 270);
  /* Raised surface: sidebar, hover states, input backgrounds
     Slightly cool-270 tint; 3% below card in dark mode */

  --color-card: oklch(100% 0 0);
  /* Card / panel background — same as background in light mode;
     in dark mode set to a lighter step than surface */

  --color-border: oklch(88% 0.008 270);
  /* Hairline separator color — used on all 1px borders, no shadow on flat elements */

  --color-text: oklch(18% 0.01 270);
  /* Primary body text — near-black with a faint cool tint */

  --color-text-muted: oklch(52% 0.01 270);
  /* Secondary / muted text — labels, breadcrumbs, empty-state body copy */

  /* ── Role colors ────────────────────────────────────────────────────────────
     Primary is the single chroma-bearing element in the chrome (monochrome palette).
     Status colors (success/warning/destructive) are the only other chroma sources. */

  --color-primary: oklch(18% 0 0);
  /* Primary action: button background, active nav indicator — near-black in light mode */

  --color-primary-foreground: oklch(100% 0 0);
  /* Text on primary background — must achieve ≥4.5:1 contrast vs --color-primary */

  --color-secondary: oklch(94% 0 0);
  /* Secondary button / surface background — light grey */

  --color-secondary-foreground: oklch(18% 0 0);
  /* Text on secondary background */

  --color-accent: oklch(18% 0 0);
  /* Decorative accent — mirrors primary by default; override for a tinted accent */

  --color-destructive: oklch(52% 0.22 25);
  /* Destructive / danger: delete buttons, error alerts, error badge text */

  --color-success: oklch(52% 0.18 145);
  /* Success / confirmed: success badge text, status dots */

  --color-warning: oklch(68% 0.18 75);
  /* Warning / caution: warning badge background and text */

  /* ── Focus ring ─────────────────────────────────────────────────────────────
     Applied via `outline: 2px solid var(--color-ring); outline-offset: 2px`
     on every interactive element's :focus-visible pseudo-class.
     Must achieve ≥3:1 contrast vs --color-background and --color-card. */

  --color-ring: oklch(60% 0.01 270);
  /* Cool-270 monochrome ring — visible against white surfaces, ≥3:1 verified */

  /* ── Shape scale ─────────────────────────────────────────────────────────────
     Elevation rule: flat surfaces use radius-sm; overlays use radius-md; pills use full.
     Never apply both border AND box-shadow to the same element. */

  --radius-sm: 0.375rem;   /* 6 px — controls (buttons, inputs, cards) */
  --radius-md: 0.625rem;   /* 10 px — overlays (dropdowns, modals, toasts) */
  --radius-lg: 0.75rem;    /* 12 px — reserved for large panels */
  --radius-full: 9999px;   /* pill — badges, status dots */

  /* ── Shadow scale ────────────────────────────────────────────────────────────
     Flat surfaces have no shadow. Overlays use shadow-md. Toasts use shadow-lg. */

  --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
  --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.08);
  --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.08);

  /* ── Font families ───────────────────────────────────────────────────────────
     Geist ships bundled at /_ferro/fonts/. To use a different face,
     add @font-face above :root and override these tokens.
     Self-hosted or system fonts only — no CDN requests. */

  --font-sans: 'Geist', ui-sans-serif, system-ui, -apple-system, sans-serif;
  /* Default UI typeface — used for all body, label, and control text */

  --font-mono: 'Geist Mono', ui-monospace, monospace;
  /* Monospace — code blocks, document editors, numeric-alignment columns */

  --font-display: var(--font-sans);
  /* Display / heading font — defaults to sans; override for a distinct heading face */

  /* ── Type-scale ──────────────────────────────────────────────────────────────
     Five semantic size/weight pairs. Skin rules compose them:
       font-size: var(--text-body-size);
       font-weight: var(--text-body-weight);
     All are mode-invariant — dark mode does not override these. */

  --text-display-size: 1.75rem;      /* 28 px — page titles (PageHeader h2) */
  --text-display-weight: 600;

  --text-section-size: 0.9375rem;    /* 15 px — card titles, group headings, tab labels */
  --text-section-weight: 600;

  --text-body-size: 0.875rem;        /* 14 px — default content: table cells, paragraphs */
  --text-body-weight: 400;

  --text-meta-size: 0.8125rem;       /* 13 px — secondary labels, timestamps, breadcrumbs */
  --text-meta-weight: 400;

  --text-micro-size: 0.75rem;        /* 12 px — badges, captions, DataTable column headers */
  --text-micro-weight: 500;          /* 500 weight makes micro text legible at small sizes */

  /* ── Density ─────────────────────────────────────────────────────────────────
     Base spacing multiplier. All spacing utilities resolve as calc(var(--spacing) * N).
     Control heights and layout gaps are multiples of 4 (grid-aligned). */

  --spacing: 0.25rem;   /* 4 px base unit */

  /* ── Motion ──────────────────────────────────────────────────────────────────
     Three-tier duration scale + a single easing curve.
     prefers-reduced-motion is honored by ferro-base.css (no override needed here). */

  --motion-duration-fast: 120ms;   /* hover color, badge appear */
  --motion-duration-base: 220ms;   /* dropdown open, panel slide */
  --motion-duration-slow: 320ms;   /* modal entry, toast */
  --motion-ease: cubic-bezier(0.2, 0, 0.38, 0.9);   /* calm, settled, no bounce */
}

/* Dark mode — activate by adding class="dark" to <body>.
   Override only the tokens that change between modes.
   Type-scale and motion tokens are intentionally omitted (mode-invariant). */
.dark {
  /* Surface steps shift lighter in dark mode: 16% → 19% → 22% (cool-270 hue) */
  --color-background: oklch(16% 0.01 270);
  --color-surface: oklch(19% 0.01 270);
  --color-card: oklch(22% 0.01 270);
  --color-border: oklch(30% 0 0);

  /* Text inverts: near-white (not pure white) for text; muted at 60% */
  --color-text: oklch(95% 0 0);
  --color-text-muted: oklch(60% 0 0);

  /* Primary inverts: near-white pill on dark background */
  --color-primary: oklch(95% 0 0);
  --color-primary-foreground: oklch(18% 0 0);

  /* Secondary darkens to match surface hierarchy */
  --color-secondary: oklch(25% 0 0);
  --color-secondary-foreground: oklch(95% 0 0);

  --color-accent: oklch(95% 0 0);

  /* Status colors shift slightly lighter for dark-surface legibility */
  --color-destructive: oklch(60% 0.22 25);
  --color-success: oklch(60% 0.18 145);
  --color-warning: oklch(65% 0.18 75);

  /* Focus ring — lighter on dark surfaces to maintain ≥3:1 contrast */
  --color-ring: oklch(72% 0.01 270);
}
```
