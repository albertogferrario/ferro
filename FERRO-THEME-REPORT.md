# Ferro Theme System — Integration Report

Report for the gestiscilo app agent to implement the new semantic theme system.

## What Was Built (Phase 99)

The Ferro framework now has a complete semantic theme system across 5 plans:

1. **ferro-theme crate** — standalone data + loader crate
2. **ThemeMiddleware** — per-request theme resolution with task-local context
3. **Semantic class migration** — all JSON-UI components use semantic tokens
4. **Intent template rendering** — themes can override how intents layout
5. **CLI + docs** — `ferro make:theme` scaffolding command

## How to Integrate

### 1. Enable the `theme` feature

In your app's `Cargo.toml`:

```toml
[dependencies]
ferro-rs = { version = "0.1", features = ["theme"] }
```

### 2. Scaffold a theme

```bash
ferro make:theme gestiscilo
```

Creates:
- `themes/gestiscilo/tokens.css` — 23 semantic token slots in Tailwind v4 `@theme` format (light + dark mode)
- `themes/gestiscilo/theme.json` — empty `{}` (uses built-in layouts)

### 3. Register ThemeMiddleware

The simplest setup — single theme for all requests:

```rust
use ferro::{ThemeMiddleware, Theme};

let theme_mw = ThemeMiddleware::new()
    .default_theme(Theme::from_path("./themes/gestiscilo").unwrap());
```

For multi-tenant (each tenant gets a different theme based on their `plan` field):

```rust
use ferro::{TenantMiddleware, ThemeMiddleware, TenantThemeResolver};

// TenantMiddleware MUST be registered before ThemeMiddleware
let tenant_mw = TenantMiddleware::new().resolver(/* your resolver */);
let theme_mw = ThemeMiddleware::new()
    .resolver(TenantThemeResolver::new("./themes"));
// Tenant with plan "enterprise" → loads themes/enterprise/
```

For header-based selection (useful for testing/previewing):

```rust
use ferro::{ThemeMiddleware, HeaderThemeResolver};

let theme_mw = ThemeMiddleware::new()
    .resolver(HeaderThemeResolver::new("./themes"));
// X-Theme: gestiscilo header → loads themes/gestiscilo/
```

### 4. Process CSS with Tailwind

The `tokens.css` uses Tailwind v4 `@theme` authoring syntax. Process it before serving:

```bash
npx tailwindcss -i themes/gestiscilo/tokens.css -o public/themes/gestiscilo.css
```

### 5. Customize tokens

Edit `themes/gestiscilo/tokens.css` to change the visual identity. The 23 token slots:

**Surface (6):** `--color-background`, `--color-surface`, `--color-card`, `--color-border`, `--color-text`, `--color-text-muted`

**Role (8):** `--color-primary`, `--color-primary-foreground`, `--color-secondary`, `--color-secondary-foreground`, `--color-accent`, `--color-destructive`, `--color-success`, `--color-warning`

**Shape (4):** `--radius-sm`, `--radius-md`, `--radius-lg`, `--radius-full`

**Shadow (3):** `--shadow-sm`, `--shadow-md`, `--shadow-lg`

**Typography (2):** `--font-family-sans`, `--font-family-mono`

## How It Works Internally

### CSS Injection

When `ThemeMiddleware` is active and the `theme` feature is enabled, JSON-UI responses automatically get a `<style>` tag injected into the HTML `<head>`. The injection order is:

1. Tailwind CDN script
2. Custom head content
3. **Theme CSS** ← injected here
4. Plugin CSS assets

### Semantic Classes

All 26 JSON-UI component renderers and 3 layout templates now emit semantic classes:

| Before | After |
|--------|-------|
| `bg-blue-600` | `bg-primary` |
| `text-white` | `text-primary-foreground` |
| `bg-gray-50` | `bg-surface` |
| `text-gray-900` | `text-text` |
| `border-gray-200` | `border-border` |
| `rounded-md` | `rounded-radius-md` |
| `shadow-sm` | `shadow-shadow-sm` |
| `bg-red-600` | `bg-destructive` |
| `bg-green-600` | `bg-success` |

Changing a token value in `tokens.css` changes every component that uses it — no Rust code changes needed.

### Intent Templates (Optional)

`theme.json` can override how intents arrange their slots. Each intent has `display` and `input` modes with ordered slot lists:

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

8 fixed slot names: `title`, `body`, `fields`, `actions`, `relationships`, `pagination`, `metadata`, `stats`.

7 intent keys: `browse`, `focus`, `collect`, `process`, `summarize`, `analyze`, `track`.

Empty `{}` = all built-in layouts. Only override what you need.

### Resolver Chain

ThemeMiddleware tries resolvers in order. First `Some` wins. If none match, the built-in default theme (embedded at compile time) is used. There is no failure mode — a theme is always available.

Both `TenantThemeResolver` and `HeaderThemeResolver` use moka caches (5-min TTL, 100 capacity) to avoid redundant disk reads.

## Key API Surface

```rust
// Types (from ferro-theme, re-exported by ferro-rs)
ferro::Theme                    // { css: String, templates: ThemeTemplates }
ferro::ThemeTemplates           // 7 Optional intent fields
ferro::IntentModeTemplates      // { display: IntentSlotTemplate, input: IntentSlotTemplate }
ferro::IntentSlotTemplate       // { slots: Vec<String>, layout: Option<String> }
ferro::ThemeError               // Io | Json | NotFound

// Construction
Theme::default_theme()          // Embedded default (compile-time)
Theme::from_path("./themes/x")  // Load from directory (tokens.css + optional theme.json)

// Middleware (from framework, feature-gated)
ferro::ThemeMiddleware           // .new() → .resolver(r) → .default_theme(t)
ferro::HeaderThemeResolver       // .new(themes_dir) — reads X-Theme header
ferro::TenantThemeResolver       // .new(themes_dir) — reads tenant.plan
ferro::DefaultResolver           // .new(theme) — always returns given theme
ferro::current_theme()           // Task-local: Option<Arc<Theme>>
```

## What to Verify in the App

After integrating, check these in the browser:

1. **HTML `<head>`** contains a `<style>` tag with the theme CSS custom properties
2. **Component classes** use semantic names (`bg-primary`, not `bg-blue-600`)
3. **Changing a token value** in `tokens.css` and rebuilding CSS changes the visual appearance
4. **Dark mode** works via `@media (prefers-color-scheme: dark)` in the theme CSS
5. **No visual regressions** — the default token values were chosen to match the previous hardcoded colors

## Reference

Full documentation: `docs/src/features/themes.md` in the ferro repo.
