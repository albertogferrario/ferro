# Phase 99: Semantic Theme System with Intent-Driven Templates - Context

**Gathered:** 2026-03-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Make JSON-UI visually customizable through semantic CSS tokens and make intent-to-layout mappings configurable through declarative templates. A theme is a complete "visual identity" package (CSS tokens + intent templates) that creators can build without writing Rust. Themes are selectable per-request for multi-tenant white-labeling. Existing render.rs hardcoded Tailwind classes are migrated to semantic token references.

</domain>

<decisions>
## Implementation Decisions

### Token Vocabulary
- Hybrid token system: surfaces for structure (background, surface, card, border, text, text-muted) + roles for semantics (primary, secondary, accent, destructive, success, warning) — designed as a cohesive set, not two separate systems
- Dark mode built in from day one: each theme defines light + dark values for every token, switched via CSS `@media(prefers-color-scheme)` or `data-theme` attribute
- Shape tokens: radius scale (sm, md, lg, full) + shadow scale (sm, md, lg) — theme controls overall "roundness" and "depth" feel
- Spacing/density is NOT a theme token — it's a container/view-level property (e.g., DashboardLayout or JsonUiView can specify compact/normal/relaxed)
- Token vocabulary is fixed and versioned (ferro-theme/v1, ~25 semantic slots)
- CSS custom properties via Tailwind v4 `@theme` block — components use `bg-primary`, `text-surface`, etc. as Tailwind utility classes mapped to CSS custom properties
- Default theme embedded in Rust as a `const &str` CSS — always available, no filesystem dependency. Custom themes override by providing their own CSS file

### Intent Template Format
- Slot-based JSON templates: each intent template defines a structural skeleton with named slots (title, body, fields, actions, relationships, pagination, metadata)
- Server fills slots with field-mapped components — field mapping (FieldMeaning → Component) stays in Rust (`field_map.rs`), templates control structural layout only
- All 7 intents template-overridable from day one (Browse, Focus, Collect, Process, Summarize, Analyze, Track)
- Templates support display + input mode variants: `{ "browse": { "display": {...}, "input": {...} } }`
- Partial overrides supported: a theme can override just Browse and Focus, leaving other intents on built-in defaults
- JsonUiRenderer in ferro-projections updated to consume templates instead of hardcoding layouts — single source of truth for intent layouts

### Theme Packaging
- A theme is two files: `tokens.css` (Tailwind v4 `@theme` with CSS custom properties, light+dark) + `theme.json` (intent template overrides for whichever intents the creator wants to customize)
- Single JSON file per theme for templates (all intent overrides in one file)
- `ferro make:theme <name>` CLI command scaffolds `themes/<name>/tokens.css` + `theme.json` with all default values as starting point

### Theme Activation
- Per-request theme selection via ThemeResolver chain (mirrors Phase 95 TenantResolver pattern): check TenantContext.theme → request header → app default
- Resolution logic lives in framework middleware (ThemeMiddleware), not in ferro-theme crate
- Loaded themes cached with moka TTL (framework-side cache, matching TenantLookup pattern)

### Crate Architecture
- New `ferro-theme` crate: token type definitions, intent template schema types, Theme struct (tokens + templates), default theme (embedded), file loader (`Theme::from_path()`)
- ferro-theme is a pure data + loading crate (like ferro-lang's Translator) — no runtime/middleware concerns
- Framework owns ThemeMiddleware for per-request resolution and moka cache for loaded themes
- Both ferro-json-ui and ferro-projections depend on ferro-theme — preserves their mutual decoupling

### render.rs Migration
- One-shot replacement: all ~50+ hardcoded Tailwind classes replaced with semantic token references in a single pass
- Default theme provides a refreshed visual appearance (not pixel-identical to current look, since no production users exist)

### Claude's Discretion
- Typography token depth (font families only vs. families + semantic size scale) — decide based on what 26 components actually need
- Component CSS hooks (`.ferro-card`, `.ferro-table` etc.) — evaluate whether semantic tokens alone suffice or component-specific targeting is needed for theme creators
- Variant-to-token mapping strategy (ButtonVariant::Primary → `--color-primary` direct mapping vs. CSS class indirection)
- Fixed slot vocabulary vs. extensible custom slots — decide based on what the 7 intent layouts actually need
- Exact crate boundary: what goes in ferro-theme vs. stays in ferro-json-ui/ferro-projections — evaluate based on actual dependency analysis

</decisions>

<specifics>
## Specific Ideas

- "UIs are interchangeable expressions of data + functionality" — three ecosystem layers: Components (atomic), Intent Templates (compositions), Themes (CSS tokens + template overrides)
- Ecosystem play: third parties can contribute components (via plugin system), themes, or both independently
- Theme creators ship a complete visual identity without writing Rust — CSS for tokens, JSON for layout structure
- Per-request themes enable multi-tenant white-labeling (gestiscilo.it tenants get their own look)
- Density is a container concern, not a theme concern — DashboardLayout might be "compact", a landing page might be "relaxed"

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-json-ui/src/render.rs` — HTML render engine with ~50+ hardcoded Tailwind classes across 26 components. Migration target for semantic tokens
- `ferro-json-ui/src/plugin.rs` — Plugin system with CSS/JS asset injection. Theme CSS could use similar injection pattern
- `ferro-json-ui/src/layout.rs` — Layout trait, LayoutRegistry, LayoutContext with `head` field for injecting styles. Theme CSS injected here
- `ferro-json-ui/src/runtime.rs` — Built-in JS runtime (FERRO_RUNTIME_JS) as embedded const &str. Default theme CSS follows same embedded pattern
- `ferro-projections/src/render/json_ui.rs` — JsonUiRenderer with hardcoded intent→layout mappings. Will consume intent templates instead
- `ferro-projections/src/render/field_map.rs` — Field-to-component mapping (18 FieldMeaning variants). Stays in Rust, fills template slots
- `ferro-projections/src/intent.rs` — 7 Intent variants + Custom(String). Templates correspond 1:1 to these intents

### Established Patterns
- Embedded const for runtime assets: `FERRO_RUNTIME_JS` in runtime.rs — default theme CSS follows same pattern
- Moka cache for runtime lookups: used in TenantLookup, InMemoryCache. Theme cache follows same pattern
- Resolver chain: TenantResolver trait with SubdomainResolver, HeaderResolver, etc. ThemeResolver follows same pattern
- Feature-gated re-exports: `#[cfg(feature = "theme")]` in framework/src/lib.rs
- New crate pattern: ferro-cache, ferro-queue, ferro-stripe all follow workspace conventions with thiserror Error enum, builder APIs

### Integration Points
- `ferro-json-ui` depends on ferro-theme for token types
- `ferro-projections` depends on ferro-theme for template types
- Framework re-exports ferro-theme types behind feature flag
- Framework adds ThemeMiddleware for per-request resolution
- `ferro-cli` gets `make:theme` command
- `ferro-mcp` gets theme introspection tools
- `.github/workflows/publish.yml` adds ferro-theme to Wave 1

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 99-semantic-theme-system-with-intent-driven-templates*
*Context gathered: 2026-03-12*
