# Themes

ferro-theme provides a semantic token system that gives JSON-UI applications consistent, customizable visual identities. A theme is a CSS file plus an optional JSON file — no Rust code is needed to create or modify one.

## Overview

A theme consists of two files:

- **`tokens.css`** — Tailwind v4 `@theme` block defining the 23 semantic token slots (colors, shapes, shadows, typography)
- **`theme.json`** — partial JSON object overriding intent template layouts (optional; empty `{}` uses defaults)

Themes live in the `themes/` directory at the project root:

```
themes/
  myapp/
    tokens.css
    theme.json
```

## Quick Start

Scaffold a new theme:

```bash
ferro make:theme myapp
```

This creates `themes/myapp/tokens.css` with all 23 token slots pre-filled with defaults, and `themes/myapp/theme.json` as an empty object.

Activate the theme in `bootstrap.rs`:

```rust
use ferro::{global_middleware, ThemeMiddleware, HeaderThemeResolver};

pub fn register() {
    global_middleware!(
        ThemeMiddleware::new()
            .resolver(HeaderThemeResolver::new("./themes"))
    );
}
```

Edit `themes/myapp/tokens.css` to customize the visual identity:

```css
@theme {
  --color-primary: oklch(55% 0.22 160);  /* green brand color */
  --color-accent: oklch(65% 0.18 300);   /* purple accent */
}
```

Process with Tailwind CLI before deploying:

```bash
npx tailwindcss -i themes/myapp/tokens.css -o public/themes/myapp.css
```

## Token Reference

All 23 semantic token slots. Components use these class names — themes control the values.

### Surface Tokens (6)

| Token | Default (light) | Purpose |
|-------|----------------|---------|
| `--color-background` | `oklch(100% 0 0)` | Page background |
| `--color-surface` | `oklch(97% 0 0)` | Section/panel background |
| `--color-card` | `oklch(95% 0 0)` | Card component background |
| `--color-border` | `oklch(90% 0 0)` | Borders and dividers |
| `--color-text` | `oklch(15% 0 0)` | Primary text |
| `--color-text-muted` | `oklch(50% 0 0)` | Secondary/placeholder text |

### Role Tokens (8)

| Token | Default (light) | Purpose |
|-------|----------------|---------|
| `--color-primary` | `oklch(55% 0.2 250)` | Primary actions, links |
| `--color-primary-foreground` | `oklch(100% 0 0)` | Text on primary backgrounds |
| `--color-secondary` | `oklch(70% 0.05 250)` | Secondary actions |
| `--color-secondary-foreground` | `oklch(15% 0 0)` | Text on secondary backgrounds |
| `--color-accent` | `oklch(65% 0.15 200)` | Highlights, badges |
| `--color-destructive` | `oklch(55% 0.22 25)` | Delete, error states |
| `--color-success` | `oklch(55% 0.18 145)` | Success states |
| `--color-warning` | `oklch(70% 0.18 80)` | Warning states |

### Shape Tokens (4)

| Token | Default | Purpose |
|-------|---------|---------|
| `--radius-sm` | `0.25rem` | Small elements (badges, tags) |
| `--radius-md` | `0.375rem` | Medium elements (inputs, buttons) |
| `--radius-lg` | `0.5rem` | Large elements (cards, modals) |
| `--radius-full` | `9999px` | Pill-shaped elements |

### Shadow Tokens (3)

| Token | Purpose |
|-------|---------|
| `--shadow-sm` | Subtle elevation (inputs, dropdowns) |
| `--shadow-md` | Card elevation |
| `--shadow-lg` | Modal/overlay elevation |

### Typography Tokens (2)

| Token | Default | Purpose |
|-------|---------|---------|
| `--font-family-sans` | `ui-sans-serif, system-ui, sans-serif` | Body and UI text |
| `--font-family-mono` | `ui-monospace, monospace` | Code, IDs, technical values |

## Dark Mode

Themes include automatic dark mode support via `@media (prefers-color-scheme: dark)`.

The scaffolded `tokens.css` includes a dark mode block:

```css
@media (prefers-color-scheme: dark) {
  @theme {
    --color-background: oklch(12% 0 0);
    --color-surface: oklch(17% 0 0);
    --color-text: oklch(95% 0 0);
    /* ... remaining dark overrides */
  }
}
```

To add manual dark mode toggle (e.g., user preference stored in a cookie), apply a `data-theme="dark"` attribute on the `<html>` element and scope the `@theme` overrides with `[data-theme="dark"]`:

```css
[data-theme="dark"] {
  --color-background: oklch(12% 0 0);
  --color-surface: oklch(17% 0 0);
  --color-text: oklch(95% 0 0);
}
```

## Intent Templates

`theme.json` controls how JSON-UI intent layouts render. Leave it as `{}` to use the defaults, or override specific intents.

Supported intent keys: `browse`, `focus`, `collect`, `process`, `summarize`, `analyze`, `track`.

Supported slot keys within each intent: `title`, `body`, `fields`, `actions`, `relationships`, `pagination`, `metadata`, `stats`.

Each intent has two modes: `display` (reading data) and `input` (forms/editing). Each mode specifies an ordered list of `slots` and an optional `layout` component.

Example — override Browse display to show title, fields, and pagination in a Table layout:

```json
{
  "browse": {
    "display": {
      "slots": ["title", "fields", "pagination"],
      "layout": "Table"
    }
  }
}
```

Example — override Collect input to show fields and actions in a Form layout:

```json
{
  "collect": {
    "input": {
      "slots": ["fields", "actions"],
      "layout": "Form"
    }
  }
}
```

Unspecified intents and modes fall back to the framework defaults. Only override what you want to change.

## ThemeMiddleware Setup

`ThemeMiddleware` resolves the active theme per request using a resolver chain. Multiple resolvers are tried in order; the first `Some` result wins.

```rust
use ferro::{global_middleware, ThemeMiddleware, HeaderThemeResolver};

pub fn register() {
    global_middleware!(
        ThemeMiddleware::new()
            .resolver(HeaderThemeResolver::new("./themes"))
    );
}
```

The middleware injects the theme CSS into every HTML response. When no resolver matches, `ThemeMiddleware` falls back to the built-in default theme — it never returns an error.

### Resolver Types

**`HeaderThemeResolver`** — selects theme from the `X-Theme` request header:

```rust
use ferro::HeaderThemeResolver;

HeaderThemeResolver::new("./themes")
// Request with `X-Theme: pro` header loads themes/pro/
```

**`TenantThemeResolver`** — selects theme based on `TenantContext.plan`:

```rust
use ferro::TenantThemeResolver;

TenantThemeResolver::new("./themes")
// Tenant with plan "enterprise" loads themes/enterprise/
```

**`DefaultResolver`** — always returns a specific theme:

```rust
use ferro::{DefaultResolver, Theme};

DefaultResolver::new(Theme::from_path("./themes/corporate").unwrap())
```

When no resolver matches, `ThemeMiddleware` falls back to the built-in default theme automatically.

## Multi-Tenant Themes

In multi-tenant applications, each tenant can have a distinct visual identity. The `TenantThemeResolver` reads `TenantContext.plan` and maps it to a theme name.

Setup — register both `TenantMiddleware` and `ThemeMiddleware`:

```rust
use ferro::{global_middleware, TenantMiddleware, ThemeMiddleware, TenantThemeResolver};

pub fn register() {
    global_middleware!(TenantMiddleware::new().resolver(/* ... */));

    global_middleware!(
        ThemeMiddleware::new()
            .resolver(TenantThemeResolver::new("./themes"))
    );
}
```

The tenant's `plan` field is used as the theme directory name. A tenant with `plan: "enterprise"` loads `themes/enterprise/`.

`TenantMiddleware` must run before `ThemeMiddleware` so `TenantContext` is available when the theme is resolved.

## For Theme Creators

**Authoring format vs. deployed format:**

The `tokens.css` file uses the Tailwind v4 `@theme` authoring syntax, which is processed by the Tailwind CLI. Do not serve the raw `tokens.css` directly — run it through Tailwind first:

```bash
npx tailwindcss -i themes/mytheme/tokens.css -o public/themes/mytheme.css
```

Add this to your build pipeline or `package.json` scripts.

**Partial overrides in theme.json:**

`theme.json` only needs to specify the intents and slots you want to change. An empty `{}` is valid and means "use all framework defaults." The framework deep-merges your overrides with the defaults, so you never need to specify the full structure.

**Publishing a theme:**

A theme is just two static files. Publish them as an npm package, a GitHub repo, or any file distribution mechanism. Users add them to their `themes/` directory and point `ThemeMiddleware` to the theme name.
