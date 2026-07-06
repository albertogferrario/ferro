# Phase 99: Semantic Theme System with Intent-Driven Templates - Research

**Researched:** 2026-03-12
**Domain:** Rust CSS token architecture, JSON template schemas, Tailwind v4 `@theme`, moka caching, multi-crate workspace patterns
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Token Vocabulary:**
- Hybrid token system: surfaces (background, surface, card, border, text, text-muted) + roles (primary, secondary, accent, destructive, success, warning)
- Dark mode built in from day one: light + dark values per token, switched via CSS `@media(prefers-color-scheme)` or `data-theme` attribute
- Shape tokens: radius scale (sm, md, lg, full) + shadow scale (sm, md, lg)
- Spacing/density is NOT a theme token — container/view-level concern
- Token vocabulary is fixed and versioned (ferro-theme/v1, ~25 semantic slots)
- CSS custom properties via Tailwind v4 `@theme` block — components use `bg-primary`, `text-surface`, etc. as Tailwind utility classes mapped to CSS custom properties
- Default theme embedded in Rust as a `const &str` CSS — always available, no filesystem dependency

**Intent Template Format:**
- Slot-based JSON templates: named slots (title, body, fields, actions, relationships, pagination, metadata)
- Server fills slots with field-mapped components — field mapping stays in Rust (`field_map.rs`)
- All 7 intents template-overridable from day one (Browse, Focus, Collect, Process, Summarize, Analyze, Track)
- Templates support display + input mode variants: `{ "browse": { "display": {...}, "input": {...} } }`
- Partial overrides supported: a theme can override just Browse and Focus

**Theme Packaging:**
- Two files: `tokens.css` (Tailwind v4 `@theme`) + `theme.json` (intent template overrides)
- `ferro make:theme <name>` CLI command scaffolds `themes/<name>/tokens.css` + `theme.json`

**Theme Activation:**
- Per-request theme selection via ThemeResolver chain (mirrors Phase 95 TenantResolver pattern)
- Resolution order: TenantContext.theme → request header → app default
- ThemeMiddleware in framework middleware, not ferro-theme crate
- Loaded themes cached with moka TTL (framework-side, matching TenantLookup pattern)

**Crate Architecture:**
- New `ferro-theme` crate: token type definitions, intent template schema types, `Theme` struct, default theme (embedded), `Theme::from_path()`
- ferro-theme is a pure data + loading crate (like ferro-lang's Translator)
- Framework owns ThemeMiddleware and moka cache
- Both ferro-json-ui and ferro-projections depend on ferro-theme
- ferro-theme goes in Wave 1 of publish.yml

**render.rs Migration:**
- One-shot replacement: all ~50+ hardcoded Tailwind classes replaced with semantic token references
- Default theme provides refreshed (not pixel-identical) appearance

### Claude's Discretion

- Typography token depth (font families only vs. families + semantic size scale)
- Component CSS hooks (`.ferro-card`, `.ferro-table` etc.) — evaluate whether needed
- Variant-to-token mapping strategy (ButtonVariant::Primary → `--color-primary` direct vs. CSS class indirection)
- Fixed slot vocabulary vs. extensible custom slots — decide based on 7 intent layouts' actual needs
- Exact crate boundary: what goes in ferro-theme vs. stays in ferro-json-ui/ferro-projections

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope
</user_constraints>

---

## Summary

Phase 99 introduces `ferro-theme`, a new pure-data crate that defines the semantic token vocabulary and intent template schema. It follows the established ferro-lang pattern (pure data + filesystem loader, no runtime concerns). The framework adds ThemeMiddleware modeled exactly on the existing TenantMiddleware + TenantResolver chain. The render.rs migration replaces all ~50+ hardcoded Tailwind utility classes with semantic references that resolve through CSS custom properties defined in the active theme's `tokens.css`.

The key architectural insight: token resolution happens entirely at the CSS layer (browser resolves `var(--color-primary)` from whichever `@theme` CSS was injected). Rust only needs to inject the correct theme CSS into the `<head>` via the existing `LayoutContext.head` field. No runtime token lookup in Rust.

Intent templates redirect structural layout responsibility from hardcoded Rust match arms in `JsonUiRenderer` to JSON slot declarations. `field_map.rs` stays in Rust as the component-filling engine; templates control only slot arrangement.

**Primary recommendation:** Build `ferro-theme` as a near-identical sibling of `ferro-lang` (file loader + embedded default), wire ThemeMiddleware as a TenantMiddleware clone, and migrate render.rs in a single focused pass replacing concrete colors with semantic class names.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde / serde_json | 1 | Theme struct serialization + JSON template parsing | Workspace standard |
| thiserror | 2 | `ThemeError` enum | Workspace standard for leaf crates |
| moka (sync) | 0.12 | Per-name theme caching in ThemeMiddleware | Already in framework, matches TenantLookup pattern |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| schemars | 1 | JSON Schema for template slot types | Consistent with ferro-json-ui and ferro-projections component schemas |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `moka::sync::Cache` | `tokio::sync::RwLock<HashMap>` | moka already in framework dep graph; provides TTL without manual eviction code |
| Embedded `const &str` for default CSS | File on disk | Embedded avoids filesystem dependency, follows FERRO_RUNTIME_JS pattern |

**Installation (ferro-theme Cargo.toml):**
```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
```

**framework/Cargo.toml additions:**
```toml
ferro-theme = { path = "../ferro-theme", version = "0.1", optional = true }
```
Add `theme = ["dep:ferro-theme"]` to `[features]`.

---

## Architecture Patterns

### Recommended Project Structure

```
ferro-theme/
├── Cargo.toml
└── src/
    ├── lib.rs           # re-exports: Theme, ThemeError, TokenVocabulary, IntentTemplate
    ├── error.rs         # ThemeError (thiserror)
    ├── token.rs         # TokenVocabulary struct (typed names, not enforced — just documentation)
    ├── template.rs      # IntentTemplate, SlotMap, ThemeTemplates types
    └── loader.rs        # Theme::from_path(), Theme::default()

framework/src/theme/
├── mod.rs              # re-exports ThemeResolver, ThemeMiddleware, current_theme()
├── resolver.rs         # ThemeResolver trait + TenantThemeResolver, HeaderThemeResolver, DefaultResolver
├── middleware.rs       # ThemeMiddleware (mirrors TenantMiddleware exactly)
└── context.rs          # task-local current_theme() (mirrors tenant context.rs)
```

### Pattern 1: ferro-theme as Pure Data Crate (mirrors ferro-lang)

**What:** `Theme` struct holds the raw CSS string (tokens.css content) and a `ThemeTemplates` map (parsed from theme.json). `Theme::from_path()` loads from disk. `Theme::default()` returns the embedded default.

**When to use:** Any time a handler, middleware, or renderer needs the active theme.

**Example:**
```rust
// ferro-theme/src/loader.rs
// Source: mirrors ferro-lang/src/loader.rs pattern

pub struct Theme {
    /// Raw CSS for injection into <head> (contents of tokens.css)
    pub css: String,
    /// Parsed intent template overrides (contents of theme.json, partial allowed)
    pub templates: ThemeTemplates,
}

impl Theme {
    /// Load a theme from a directory containing tokens.css + theme.json.
    pub fn from_path(path: &str) -> Result<Self, ThemeError> {
        let dir = std::path::Path::new(path);
        let css = std::fs::read_to_string(dir.join("tokens.css"))?;
        let json_path = dir.join("theme.json");
        let templates = if json_path.exists() {
            let raw = std::fs::read_to_string(&json_path)?;
            serde_json::from_str(&raw)?
        } else {
            ThemeTemplates::default()
        };
        Ok(Self { css, templates })
    }

    /// Return the built-in default theme (embedded at compile time).
    pub fn default_theme() -> Self {
        Self {
            css: DEFAULT_THEME_CSS.to_string(),
            templates: ThemeTemplates::default(),
        }
    }
}

pub(crate) const DEFAULT_THEME_CSS: &str = include_str!("../assets/default.css");
```

### Pattern 2: ThemeResolver Chain (mirrors TenantResolver)

**What:** Trait with `resolve(&Request) -> Option<Arc<Theme>>`. Framework middleware tries resolvers in order, first `Some` wins.

**When to use:** Per-request theme selection for multi-tenant white-labeling.

**Example:**
```rust
// framework/src/theme/resolver.rs
// Source: mirrors framework/src/tenant/resolver.rs pattern

#[async_trait]
pub trait ThemeResolver: Send + Sync {
    async fn resolve(&self, req: &Request) -> Option<Arc<Theme>>;
}

pub struct TenantThemeResolver {
    theme_cache: Cache<String, Arc<Theme>>,
    theme_dir: String,
}

#[async_trait]
impl ThemeResolver for TenantThemeResolver {
    async fn resolve(&self, req: &Request) -> Option<Arc<Theme>> {
        // Pull theme name from TenantContext (already resolved by TenantMiddleware)
        let tenant = current_tenant()?;
        let theme_name = tenant.theme_name.as_deref()?;
        // Check moka cache
        if let Some(cached) = self.theme_cache.get(theme_name) {
            return Some(cached);
        }
        // Load from disk, cache it
        let path = format!("{}/{}", self.theme_dir, theme_name);
        let theme = Arc::new(Theme::from_path(&path).ok()?);
        self.theme_cache.insert(theme_name.to_string(), Arc::clone(&theme));
        Some(theme)
    }
}
```

### Pattern 3: ThemeMiddleware (mirrors TenantMiddleware)

**What:** Framework middleware that resolves theme and stores in task-local context.

**Example:**
```rust
// framework/src/theme/middleware.rs
// Source: mirrors framework/src/tenant/middleware.rs

pub struct ThemeMiddleware {
    resolvers: Vec<Box<dyn ThemeResolver>>,
    default: Arc<Theme>,
}

#[async_trait]
impl Middleware for ThemeMiddleware {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let theme = 'resolve: {
            for resolver in &self.resolvers {
                if let Some(t) = resolver.resolve(&request).await {
                    break 'resolve t;
                }
            }
            Arc::clone(&self.default)
        };
        // Store in task-local, call next
        with_theme_scope(theme, next(request)).await
    }
}
```

### Pattern 4: CSS Injection into LayoutContext.head

**What:** The active theme's CSS string is injected as an inline `<style>` tag or `<link>` into `LayoutContext.head`. No new field needed — `head` already accepts arbitrary HTML.

**When to use:** In the framework's JSON-UI rendering pipeline, after resolving the theme.

**Example:**
```rust
// framework/src/json_ui.rs (existing render pipeline)
// Source: ferro-json-ui/src/layout.rs — head field is &str already

let theme_css = current_theme()
    .map(|t| format!("<style>{}</style>", t.css))
    .unwrap_or_default();

let head = format!("{}{}", existing_head, theme_css);
// Pass head into LayoutContext
```

### Pattern 5: render.rs Semantic Token Migration

**What:** Replace every hardcoded Tailwind color class in render.rs with the semantic equivalent. The CSS custom properties in the active theme resolve the actual colors.

**Migration table (representative, not exhaustive):**

| Before (hardcoded) | After (semantic) |
|--------------------|------------------|
| `bg-white` | `bg-background` |
| `bg-gray-50` | `bg-surface` |
| `bg-gray-100` | `bg-card` |
| `border-gray-200` | `border-border` |
| `text-gray-900` | `text-text` |
| `text-gray-600` | `text-text-muted` |
| `text-gray-500` | `text-text-muted` |
| `bg-blue-600` | `bg-primary` |
| `text-blue-600` | `text-primary` |
| `bg-red-500` | `bg-destructive` |
| `text-white` (on colored bg) | keep as-is (contrast color) |
| `rounded-md` | `rounded-radius-md` |
| `shadow-md` | `shadow-shadow-md` |

**Tailwind v4 `@theme` block that backs these classes:**
```css
/* ferro-theme/assets/default.css */
@import "tailwindcss";

@theme {
  /* Surface tokens */
  --color-background: oklch(100% 0 0);
  --color-surface: oklch(97% 0 0);
  --color-card: oklch(95% 0 0);
  --color-border: oklch(90% 0 0);
  --color-text: oklch(15% 0 0);
  --color-text-muted: oklch(50% 0 0);

  /* Role tokens */
  --color-primary: oklch(55% 0.2 250);
  --color-primary-foreground: oklch(100% 0 0);
  --color-secondary: oklch(70% 0.05 250);
  --color-secondary-foreground: oklch(15% 0 0);
  --color-accent: oklch(65% 0.15 200);
  --color-destructive: oklch(55% 0.22 25);
  --color-success: oklch(55% 0.18 145);
  --color-warning: oklch(70% 0.18 80);

  /* Shape tokens */
  --radius-sm: 0.25rem;
  --radius-md: 0.375rem;
  --radius-lg: 0.5rem;
  --radius-full: 9999px;

  /* Shadow tokens */
  --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
  --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1);
  --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1);
}

@media (prefers-color-scheme: dark) {
  @theme {
    --color-background: oklch(12% 0 0);
    --color-surface: oklch(17% 0 0);
    --color-card: oklch(20% 0 0);
    --color-border: oklch(30% 0 0);
    --color-text: oklch(95% 0 0);
    --color-text-muted: oklch(60% 0 0);
    /* role tokens adjust for dark */
    --color-primary: oklch(65% 0.2 250);
  }
}

[data-theme="dark"] {
  /* Same as prefers-color-scheme: dark, for explicit override */
}
```

### Pattern 6: IntentTemplate JSON Schema

**What:** JSON structure defining slot layout for each intent. Server fills named slots with field-mapped components from `field_map.rs`.

**Example schema:**
```json
{
  "browse": {
    "display": {
      "slots": ["title", "body", "actions", "pagination"],
      "layout": "table",
      "title": { "component": "Text", "element": "h1" },
      "body": { "component": "Table" },
      "actions": { "position": "top-right" },
      "pagination": { "position": "bottom" }
    },
    "input": {
      "slots": ["title", "fields", "actions"],
      "layout": "form"
    }
  },
  "focus": {
    "display": {
      "slots": ["title", "body", "relationships", "metadata"],
      "layout": "detail"
    }
  }
}
```

Partial override: a theme only needs to include the intents it wants to customize.

### Anti-Patterns to Avoid

- **Storing theme CSS in the database:** CSS can be large. Keep themes on disk; use moka cache for per-request performance.
- **Resolving token values in Rust:** CSS custom properties resolve at browser paint time. Never replicate this in Rust string manipulation.
- **Adding `theme` field to every component:** Theme injection happens at the page level (LayoutContext.head), not per-component.
- **Making ferro-theme depend on the framework:** ferro-theme must remain a leaf crate (no framework imports) so ferro-json-ui and ferro-projections can depend on it without creating circular dependencies.
- **Replacing render.rs piecemeal across multiple tasks:** render.rs is a single-file migration. Split only at the structural boundary (one task per render function group: layout shell, components, etc.) but ensure tests pass after each section.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Per-request caching of loaded themes | Custom HashMap + RwLock | `moka::sync::Cache` with TTL | Already in workspace deps; handles concurrent eviction, TTL, capacity |
| CSS custom property resolution | Rust string substitution | Browser CSS engine via injected `<style>` | CSS custom properties are a browser primitive — zero Rust code needed |
| JSON template validation | Custom parser | serde_json deserialization + `#[serde(default)]` + Option fields | Partial override semantics come free with serde defaults |
| Theme file scaffolding template | Inline string literals in CLI handler | Templates module (`ferro-cli/src/templates.rs` pattern) | Consistent with how `make_lang` scaffolds JSON files |

---

## Common Pitfalls

### Pitfall 1: Circular Dependency via ferro-theme
**What goes wrong:** If ferro-theme imports anything from ferro-json-ui or ferro-projections (even types), the workspace gets a circular dependency that Cargo cannot resolve.
**Why it happens:** Both ferro-json-ui and ferro-projections must depend on ferro-theme for template/token types. Any reverse dependency creates a cycle.
**How to avoid:** ferro-theme must be a pure leaf crate. No internal workspace imports. Token and template type definitions live in ferro-theme; component/render types stay in ferro-json-ui.
**Warning signs:** Cargo giving "cyclic dependency" errors during `cargo build`.

### Pitfall 2: Tailwind v4 `@theme` Not Processed at Runtime
**What goes wrong:** Injecting raw `@theme` CSS into an HTML page doesn't work if Tailwind is running as a CLI build step with a static output file. The custom properties only exist in the final CSS if Tailwind processes the source file.
**Why it happens:** Tailwind v4's `@theme` is a build-time directive, not a runtime CSS feature. The generated CSS contains the custom properties, not the `@theme` block itself.
**How to avoid:** The default theme CSS injected by the framework should be the **processed output** (containing CSS custom properties directly) rather than the unprocessed `@theme` source. For development, the Tailwind CDN script processes `@theme` at runtime — acceptable for development but not for production output.
**Recommendation:** The embedded `DEFAULT_THEME_CSS` should contain `@layer base { :root { --color-primary: ...; } }` directly. Theme creators write `@theme` for the authoring experience; the CLI `make:theme` scaffolds `@theme` syntax, but deployment docs clarify that it must be built with Tailwind CLI.
**Warning signs:** Browser showing default browser colors instead of theme values; no `--color-primary` in computed styles.

### Pitfall 3: Task-Local Theme Context Not Propagated Across Spawned Tasks
**What goes wrong:** If a handler spawns a `tokio::spawn` and accesses `current_theme()` inside the spawned task, it gets `None` — task-local storage does not cross spawn boundaries.
**Why it happens:** Same issue as `current_tenant()` — task-local storage is per-task, not per-request.
**How to avoid:** Follow the same pattern as tenant context: pass `Arc<Theme>` explicitly to any spawned tasks that need it. Document this limitation in `current_theme()` godoc.
**Warning signs:** `current_theme()` returning None in background tasks.

### Pitfall 4: render.rs Migration Breaking Existing Tests
**What goes wrong:** render.rs has extensive tests that assert specific CSS class names like `"rounded-md"` or `"text-gray-900"`. After migration to semantic classes, all tests asserting exact class names will fail.
**Why it happens:** Tests were written against the old hardcoded Tailwind classes.
**How to avoid:** Update tests in the same pass as the migration. Replace assertions like `assert!(html.contains("text-gray-900"))` with `assert!(html.contains("text-text"))`. Do not leave failing tests between commits.
**Warning signs:** `cargo test` failing on layout.rs or render.rs test modules.

### Pitfall 5: Intent Template Slots with No Component Filler
**What goes wrong:** A template declares a slot (e.g., `"relationships"`) but the service has no relationships. Attempting to fill a slot with no data may produce empty HTML elements or panic on unwrap.
**Why it happens:** Templates and service definitions are authored independently.
**How to avoid:** `JsonUiRenderer` must check whether data exists for each slot before emitting HTML. Empty slots are skipped, not rendered as empty containers. Make this behavior explicit and tested.
**Warning signs:** Empty `<div>` elements in rendered output; unused slots appearing in HTML.

---

## Code Examples

### ferro-theme/src/template.rs — IntentTemplate Types
```rust
// Source: derived from ferro-projections/src/intent.rs + ferro-lang/src/config.rs patterns

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Slot-based template for a single intent × mode combination.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntentSlotTemplate {
    /// Ordered slot names for this layout (e.g., ["title", "body", "pagination"]).
    #[serde(default)]
    pub slots: Vec<String>,
    /// Layout strategy hint (e.g., "table", "form", "detail", "kanban").
    #[serde(default)]
    pub layout: Option<String>,
}

/// Display + input mode templates for one intent.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntentModeTemplates {
    #[serde(default)]
    pub display: IntentSlotTemplate,
    #[serde(default)]
    pub input: IntentSlotTemplate,
}

/// All intent template overrides in a theme. Partial — missing intents use built-in defaults.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeTemplates {
    #[serde(default)]
    pub browse: Option<IntentModeTemplates>,
    #[serde(default)]
    pub focus: Option<IntentModeTemplates>,
    #[serde(default)]
    pub collect: Option<IntentModeTemplates>,
    #[serde(default)]
    pub process: Option<IntentModeTemplates>,
    #[serde(default)]
    pub summarize: Option<IntentModeTemplates>,
    #[serde(default)]
    pub analyze: Option<IntentModeTemplates>,
    #[serde(default)]
    pub track: Option<IntentModeTemplates>,
}
```

### ferro-theme/src/error.rs — ThemeError
```rust
// Source: matches thiserror pattern used by ferro-lang, ferro-cache, etc.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("IO error loading theme: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid theme.json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Theme not found: {0}")]
    NotFound(String),
}
```

### framework/src/theme/context.rs — Task-Local Storage
```rust
// Source: mirrors framework/src/tenant/context.rs exactly

use std::sync::Arc;
use ferro_theme::Theme;
use tokio::task_local;

task_local! {
    static CURRENT_THEME: Arc<Theme>;
}

/// Get the current theme from task-local context.
/// Returns None if called outside ThemeMiddleware scope.
pub fn current_theme() -> Option<Arc<Theme>> {
    CURRENT_THEME.try_with(|t| Arc::clone(t)).ok()
}

pub async fn with_theme_scope<F: Future>(theme: Arc<Theme>, f: F) -> F::Output {
    CURRENT_THEME.scope(theme, f).await
}
```

### ferro-cli make:theme template scaffolding
```
themes/
└── <name>/
    ├── tokens.css   # Tailwind v4 @theme block with all ~25 token slots prefilled to defaults
    └── theme.json   # Empty ThemeTemplates JSON — creator fills only intents they want
```

`tokens.css` scaffold content:
```css
@import "tailwindcss";

@theme {
  /* Surface tokens — edit to customize */
  --color-background: oklch(100% 0 0);
  --color-surface: oklch(97% 0 0);
  --color-card: oklch(95% 0 0);
  --color-border: oklch(90% 0 0);
  --color-text: oklch(15% 0 0);
  --color-text-muted: oklch(50% 0 0);

  /* Role tokens */
  --color-primary: oklch(55% 0.2 250);
  --color-primary-foreground: oklch(100% 0 0);
  --color-secondary: oklch(70% 0.05 250);
  --color-secondary-foreground: oklch(15% 0 0);
  --color-accent: oklch(65% 0.15 200);
  --color-destructive: oklch(55% 0.22 25);
  --color-success: oklch(55% 0.18 145);
  --color-warning: oklch(70% 0.18 80);

  /* Shape tokens */
  --radius-sm: 0.25rem;
  --radius-md: 0.375rem;
  --radius-lg: 0.5rem;
  --radius-full: 9999px;

  /* Shadow tokens */
  --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
  --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1);
  --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1);
}

@media (prefers-color-scheme: dark) {
  @theme {
    --color-background: oklch(12% 0 0);
    /* ... */
  }
}
```

`theme.json` scaffold:
```json
{}
```
(Empty object — partial overrides only needed intents.)

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tailwind v3 CSS variables (hsl()) | Tailwind v4 `@theme` with oklch() | Tailwind v4 (2024) | oklch provides perceptually uniform color, better dark mode |
| Tailwind v3 `theme()` JS config | v4 `@theme` CSS block | Tailwind v4 | No more tailwind.config.js needed for custom properties |
| Hardcoded color classes in render.rs | Semantic token classes (bg-primary, text-text) | This phase | Theme swapping becomes CSS-only |

**Deprecated/outdated:**
- Tailwind v3 `theme()` function in CSS: replaced by CSS custom properties referenced directly as `var(--color-primary)`
- `bg-gray-*` / `text-gray-*` in render.rs: replaced by `bg-surface` / `text-text-muted` etc.

---

## Open Questions

1. **Typography token depth**
   - What we know: 26 components in render.rs use font sizing classes (text-sm, text-xs, text-lg, text-xl, text-2xl)
   - What's unclear: Whether theme creators need per-semantic-role font sizing (e.g., `--font-size-heading` vs. just `--font-family-sans`) or whether Tailwind's built-in size scale suffices
   - Recommendation: Start with font family tokens only (`--font-family-sans`, `--font-family-mono`). Tailwind's built-in size scale (text-sm/md/lg/xl) stays hardcoded in render.rs. Add size scale only if a theme actually needs different relative sizes.

2. **Component CSS hooks**
   - What we know: Plugin system uses CDN CSS; layout.rs uses raw Tailwind utilities
   - What's unclear: Whether theme creators (third parties) need `.ferro-card`, `.ferro-table` selectors to apply targeted overrides beyond what semantic tokens provide
   - Recommendation: Start without component CSS hooks. Semantic tokens cover 90% of customization. If a theme creator demonstrates a concrete need (e.g., table row hover color distinct from card hover), add hooks then.

3. **ButtonVariant::Primary → token mapping**
   - What we know: render.rs maps `ButtonVariant::Primary` → `"bg-blue-600 text-white hover:bg-blue-700"` today
   - What's unclear: Whether to map directly to `bg-primary text-primary-foreground hover:bg-primary/90` (direct) vs. emitting a CSS class `.ferro-btn-primary { background: var(--color-primary); }` (indirection)
   - Recommendation: Direct token mapping in render.rs (`bg-primary text-primary-foreground`). CSS indirection adds a layer with no benefit when Tailwind v4 already compiles utility classes from tokens.

4. **Slot vocabulary extensibility**
   - What we know: 7 intents × 2 modes have concrete component needs from existing `json_ui.rs` render functions
   - What's unclear: Whether custom intent slots (e.g., a "kanban_columns" slot for Process intent) are needed beyond the named 7 in CONTEXT.md
   - Recommendation: Fixed vocabulary: `title`, `body`, `fields`, `actions`, `relationships`, `pagination`, `metadata`, `stats`. No extensible custom slots in v1. Add extensibility only if a concrete theme requires a slot not in this list.

---

## Validation Architecture

> `workflow.nyquist_validation` is absent from config.json — treating as enabled.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none — workspace `cargo test --all-features` |
| Quick run command | `cargo test -p ferro-theme` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| THEME-01 | `Theme::from_path()` loads tokens.css + theme.json | unit | `cargo test -p ferro-theme loader` | Wave 0 |
| THEME-02 | `Theme::default_theme()` returns embedded CSS | unit | `cargo test -p ferro-theme token` | Wave 0 |
| THEME-03 | `ThemeTemplates` deserializes partial JSON (missing intents use None) | unit | `cargo test -p ferro-theme template` | Wave 0 |
| THEME-04 | `ThemeMiddleware` tries resolvers in order, first Some wins | unit | `cargo test -p ferro-rs theme::middleware` | Wave 0 |
| THEME-05 | `ThemeMiddleware` falls back to default when no resolver matches | unit | `cargo test -p ferro-rs theme::middleware` | Wave 0 |
| THEME-06 | `current_theme()` returns resolved theme inside middleware scope | unit | `cargo test -p ferro-rs theme::context` | Wave 0 |
| THEME-07 | render.rs components use semantic classes (no `bg-gray-*` hardcoded) | unit | `cargo test -p ferro-json-ui render` | Existing (update assertions) |
| THEME-08 | layout.rs shell uses semantic classes (no `bg-white border-gray-200`) | unit | `cargo test -p ferro-json-ui layout` | Existing (update assertions) |
| THEME-09 | Theme CSS injected into `LayoutContext.head` | unit | `cargo test -p ferro-rs json_ui` | Wave 0 |
| THEME-10 | `JsonUiRenderer` consumes intent templates from Theme | unit | `cargo test -p ferro-projections render::json_ui` | Existing (update) |
| THEME-11 | `ferro make:theme <name>` scaffolds `themes/<name>/tokens.css` + `theme.json` | unit | `cargo test -p ferro-cli make_theme` | Wave 0 |
| THEME-12 | `make:theme` fails if `themes/<name>/` already exists | unit | `cargo test -p ferro-cli make_theme` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-theme && cargo clippy -p ferro-theme -- -D warnings`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite green + `cargo fmt --all -- --check` before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `ferro-theme/src/lib.rs` — crate does not exist yet, must be created
- [ ] `ferro-theme/src/error.rs` — ThemeError
- [ ] `ferro-theme/src/template.rs` — IntentTemplate, ThemeTemplates
- [ ] `ferro-theme/src/loader.rs` — Theme::from_path(), Theme::default_theme()
- [ ] `ferro-theme/assets/default.css` — embedded default CSS (include_str!)
- [ ] `framework/src/theme/mod.rs` — module skeleton
- [ ] `framework/src/theme/resolver.rs` — ThemeResolver trait
- [ ] `framework/src/theme/middleware.rs` — ThemeMiddleware
- [ ] `framework/src/theme/context.rs` — current_theme() task-local
- [ ] `ferro-cli/src/commands/make_theme.rs` — scaffold command
- [ ] Framework install: no new packages, ferro-theme added to workspace and framework optional deps

---

## Sources

### Primary (HIGH confidence)
- Direct codebase reading — `framework/src/tenant/` (middleware.rs, resolver.rs, context.rs, lookup.rs) — TenantResolver/Middleware pattern to mirror
- Direct codebase reading — `ferro-lang/src/loader.rs` — pure data + file loader crate pattern
- Direct codebase reading — `ferro-json-ui/src/layout.rs` — LayoutContext.head injection point
- Direct codebase reading — `ferro-json-ui/src/runtime.rs` — FERRO_RUNTIME_JS embedded const &str pattern
- Direct codebase reading — `ferro-json-ui/src/render.rs` (first 100 lines) — component render structure
- Direct codebase reading — `ferro-projections/src/render/json_ui.rs` — intent→layout dispatch to migrate
- Direct codebase reading — `ferro-cli/src/commands/make_lang.rs` — CLI scaffold command pattern
- Direct codebase reading — `.github/workflows/publish.yml` — Wave 1 crate list
- Direct codebase reading — `framework/Cargo.toml` — feature flag pattern (json-ui, stripe, projections)

### Secondary (MEDIUM confidence)
- Tailwind v4 CSS `@theme` block specification (design context from CONTEXT.md decisions)
- moka 0.12 sync cache API — verified against existing usage in `framework/src/tenant/lookup.rs`

### Tertiary (LOW confidence)
- oklch() color space behavior for dark mode — industry practice, not formally verified against Tailwind v4 docs in this session

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in workspace
- Architecture: HIGH — directly derived from existing TenantMiddleware + ferro-lang patterns in codebase
- Pitfalls: HIGH for circular deps and task-local propagation (known Rust patterns); MEDIUM for Tailwind v4 `@theme` runtime behavior
- Template schema: HIGH — serde_json deserialization with `Option` fields handles partial overrides cleanly

**Research date:** 2026-03-12
**Valid until:** 2026-04-12 (stable domain — pure Rust patterns)
