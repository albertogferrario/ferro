# Token Reference

For how to write and activate a theme that customizes these tokens, see [Themes](../features/themes.md).

The 30 semantic token slots below form the complete vocabulary. Components use these token names as class names; themes supply the actual values through CSS custom properties.

## Surface Tokens (6)

| Token | Purpose |
|-------|---------|
| `--color-background` | Page/canvas background |
| `--color-surface` | Component surface (cards, panels) |
| `--color-card` | Card background (may differ from surface) |
| `--color-border` | Dividers, input borders, separators |
| `--color-text` | Primary text |
| `--color-text-muted` | Secondary/muted text, placeholders |

## Role Tokens (8)

| Token | Purpose |
|-------|---------|
| `--color-primary` | Primary action color (buttons, links) |
| `--color-primary-foreground` | Text on primary-colored surfaces |
| `--color-secondary` | Secondary action / subdued UI elements |
| `--color-secondary-foreground` | Text on secondary-colored surfaces |
| `--color-accent` | Accent highlight (hover, selection) |
| `--color-destructive` | Destructive actions and danger states |
| `--color-success` | Success / positive states |
| `--color-warning` | Warning / caution states |

## Shape Tokens (4)

| Token | Purpose |
|-------|---------|
| `--radius-sm` | Small corner radius (badges, chips) |
| `--radius-md` | Medium corner radius (buttons, inputs) |
| `--radius-lg` | Large corner radius (cards, modals) |
| `--radius-full` | Full / pill corner radius (avatars) |

## Shadow Tokens (3)

| Token | Purpose |
|-------|---------|
| `--shadow-sm` | Small elevation shadow |
| `--shadow-md` | Medium elevation shadow (dropdowns) |
| `--shadow-lg` | Large elevation shadow (modals) |

## Typography Tokens (2)

| Token | Purpose |
|-------|---------|
| `--font-sans` | Body / UI sans-serif font stack |
| `--font-mono` | Monospace font stack (code, IDs) |

## Density Token (1)

| Token | Purpose |
|-------|---------|
| `--spacing` | Base spacing unit (density scale) |

## Motion Tokens (4)

| Token | Purpose |
|-------|---------|
| `--motion-duration-fast` | Fast transitions (100–150 ms) |
| `--motion-duration-base` | Standard transitions (200–250 ms) |
| `--motion-duration-slow` | Slow transitions (300–400 ms) |
| `--motion-ease` | Default easing curve |

All three duration tokens collapse to `0.01ms` under `prefers-reduced-motion: reduce`.

## Focus Ring Token (1)

| Token | Purpose |
|-------|---------|
| `--color-ring` | Focus ring / outline color |

## Display Font Token (1)

| Token | Purpose |
|-------|---------|
| `--font-display` | Display/heading font (defaults to `--font-sans`) |
