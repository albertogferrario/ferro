# Phase 29: Layout System - Research

**Researched:** 2026-02-09
**Domain:** Server-driven UI layout system for Rust HTML rendering
**Confidence:** HIGH

<research_summary>
## Summary

Researched layout patterns across server-driven UI systems (Airbnb Ghost, DoorDash Mosaic, Shopify, PhonePe LiquidUI), HTML-first frameworks (Phoenix LiveView, Blazor Server, HTMX), and Rust web frameworks (Leptos, Dioxus, Askama, Maud, Tera).

Ferro's JSON-UI is an SSR-first, HTML-first system with no frontend build step. The established pattern for this architecture is the **Phoenix LiveView model**: a layout registry with functional wrapper layouts, where the view specifies a layout name and the framework wraps rendered content in the corresponding HTML shell. Layouts are Rust functions (not template files), consistent with ferro-json-ui's existing string-based HTML generation approach.

The existing `JsonUiView.layout: Option<String>` field already captures the view's layout intent. The work is: (1) define a layout trait/function signature, (2) build a registry mapping names to layout implementations, (3) provide default layouts, (4) integrate layout wrapping into the framework render pipeline, and (5) support partials as composable Rust functions for navigation, sidebar, footer, etc.

**Primary recommendation:** Layouts as `Fn(&LayoutContext) -> String` registered by name. Partials as standalone functions returning `String`. Slots via struct fields on `LayoutContext`. No template engine needed — pure Rust composition.
</research_summary>

<standard_stack>
## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ferro-json-ui | internal | HTML rendering engine | Already renders component trees to HTML |
| serde_json | existing | Layout context data | Already a dependency for view data |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| None needed | - | - | The layout system is pure Rust functions generating strings — no additional dependencies |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Pure Rust functions | Askama templates | Template files add filesystem dependency; lose type safety of current approach |
| Pure Rust functions | Tera templates | Runtime template parsing; inconsistent with existing render.rs pattern |
| Pure Rust functions | Maud macro | Additional dependency; cosmetic improvement only, same conceptual model |
| String concatenation | HTML builder crate | Over-engineering; format!() works fine for layout shells |

**Installation:**
No new dependencies required. Layout system uses existing crate infrastructure.
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Recommended Architecture

```
ferro-json-ui/src/
├── layout.rs          # Layout trait, LayoutContext, default layouts, registry
├── partial.rs         # Reusable partial functions (nav, sidebar, footer)
├── render.rs          # (existing) Component tree → HTML fragment
├── ...

framework/src/json_ui/
├── mod.rs             # (existing) Wraps ferro-json-ui, now layout-aware render pipeline
```

### Pattern 1: Functional Layout with Context
**What:** Layouts are functions that receive a context struct and return an HTML string wrapping the page content.
**When to use:** Every JSON-UI page render.
**How it works:**

```rust
/// Context passed to layout functions.
pub struct LayoutContext<'a> {
    pub title: &'a str,
    pub content: &'a str,        // Rendered component HTML
    pub head: &'a str,           // Additional <head> content (Tailwind CDN, custom)
    pub body_class: &'a str,     // CSS classes for <body>
    pub data: &'a serde_json::Value,  // View data (for layout conditional rendering)
    pub view_json: &'a str,      // Serialized view (for data-view attr)
    pub data_json: &'a str,      // Serialized data (for data-props attr)
}

/// Trait for layout implementations.
pub trait Layout: Send + Sync {
    fn render(&self, ctx: &LayoutContext) -> String;
}
```

### Pattern 2: Layout Registry
**What:** Named layout registration, matching the `JsonUiView.layout` field.
**When to use:** Framework initialization + render pipeline.

```rust
pub struct LayoutRegistry {
    layouts: HashMap<String, Box<dyn Layout>>,
    default: String,
}

impl LayoutRegistry {
    pub fn new() -> Self { /* "default" layout pre-registered */ }
    pub fn register(&mut self, name: impl Into<String>, layout: impl Layout + 'static) { ... }
    pub fn render(&self, name: Option<&str>, ctx: &LayoutContext) -> String { ... }
}
```

### Pattern 3: Partials as Composable Functions
**What:** Reusable HTML fragments (nav, sidebar, footer) as standalone functions.
**When to use:** Inside layout implementations for shared chrome.

```rust
// Users compose partials into custom layouts
pub fn navigation(items: &[NavItem]) -> String { ... }
pub fn sidebar(sections: &[SidebarSection]) -> String { ... }
pub fn footer(text: &str) -> String { ... }
```

### Pattern 4: Slot-Based Layout via Context Fields
**What:** Named content areas via LayoutContext fields rather than generic slot maps.
**When to use:** When layouts need multiple injection points beyond just "content".
**Decision:** Start with single content slot. Add named slots only if Phase 30+ demonstrates need.

### Anti-Patterns to Avoid
- **Generic slot maps:** `HashMap<String, String>` for slots is over-engineering. Named struct fields are clearer and type-safe.
- **Template files:** Introducing `.html` template files breaks the pure-Rust pattern established in render.rs.
- **Layout inheritance chains:** Deeply nested layout extends (A extends B extends C) is a template engine pattern. For Rust functions, prefer composition over inheritance.
- **Runtime layout discovery:** Scanning filesystem for layout files. Layouts should be registered at startup.
- **Persistent layout state:** Ferro JSON-UI is stateless SSR. Each request renders a complete page. Don't try to cache layout state between requests.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTML escaping | Custom escape function | Existing `html_escape()` in render.rs / framework json_ui mod | Already tested, handles all OWASP cases |
| Tailwind CDN injection | Per-layout CDN logic | Existing `JsonUiConfig.tailwind_cdn` | Already configurable, pass through LayoutContext |
| Component rendering | Layout-specific renderers | Existing `render_to_html()` | Layouts wrap content, never render components themselves |
| Route resolution | Layout-level route helpers | Existing `crate::routing::route()` | Already works in action resolver |

**Key insight:** The layout system should ONLY handle page wrapping (HTML shell around content). All component rendering, action resolution, error handling, and data binding are already solved in Phases 23-28. A layout is just `<html><head>...</head><body>{nav}{content}{footer}</body></html>`.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Over-Abstracting the Layout API
**What goes wrong:** Building a generic "layout engine" with slots, blocks, inheritance, middleware hooks.
**Why it happens:** Familiarity with template engine features (Jinja2, Blade, Twig) that solve problems this system doesn't have.
**How to avoid:** Layouts are functions. Content goes in a struct. That's it. Lyft's principle: "design for the flexibility needed, not maximal flexibility."
**Warning signs:** If the layout API has more than 5-6 types, it's over-engineered.

### Pitfall 2: Duplicating the HTML Shell
**What goes wrong:** Each layout function duplicates `<!DOCTYPE html><html><head>...</head>...` boilerplate.
**Why it happens:** Treating each layout as a completely independent HTML page.
**How to avoid:** Extract a `base_document(head, body) -> String` helper that all layouts call. Layouts only differ in their `<body>` content structure.
**Warning signs:** Copy-pasted `<head>` content across layout implementations.

### Pitfall 3: Breaking the data-view / data-props Contract
**What goes wrong:** Layout wrapping loses the `data-view` and `data-props` attributes that enable potential JS hydration.
**Why it happens:** Moving the HTML shell generation from framework/json_ui/mod.rs into layouts without preserving the wrapper div contract.
**How to avoid:** LayoutContext includes the serialized view/data JSON. The wrapper `<div id="ferro-json-ui" data-view="..." data-props="...">` is part of the layout contract, not the component renderer.
**Warning signs:** Frontend hydration breaks after layout system integration.

### Pitfall 4: Layouts That Know About Components
**What goes wrong:** Layout code imports component types or tries to render specific components.
**Why it happens:** Wanting the layout to include dynamic elements (user avatar in nav, unread count in sidebar).
**How to avoid:** Layout receives pre-rendered `content` string and optional structured context data. If nav needs user info, pass it via LayoutContext.data, not by importing ComponentNode.
**Warning signs:** `use ferro_json_ui::component::*` in layout code.

### Pitfall 5: Not Providing a Sensible Default
**What goes wrong:** Views without `layout` field set get no HTML wrapper or a broken page.
**Why it happens:** Forgetting that `layout: None` is the common case during development.
**How to avoid:** Registry has a `default` layout that produces a minimal but functional page. `None` maps to default.
**Warning signs:** Blank pages when `layout` is not explicitly set.
</common_pitfalls>

<code_examples>
## Code Examples

### Layout Trait and Default Implementation
```rust
// Source: Derived from Maud functional composition + Phoenix LiveView model
pub trait Layout: Send + Sync {
    fn render(&self, ctx: &LayoutContext) -> String;
}

/// Minimal layout — just wraps content in a valid HTML page.
pub struct DefaultLayout;

impl Layout for DefaultLayout {
    fn render(&self, ctx: &LayoutContext) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    {head}
</head>
<body class="{body_class}">
    <div id="ferro-json-ui"
         data-view="{view_escaped}"
         data-props="{data_escaped}">
        {content}
    </div>
</body>
</html>"#,
            title = ctx.title,
            head = ctx.head,
            body_class = ctx.body_class,
            view_escaped = ctx.view_json,
            data_escaped = ctx.data_json,
            content = ctx.content,
        )
    }
}
```

### App Layout with Navigation and Sidebar
```rust
// Source: Phoenix LiveView two-tier layout pattern
pub struct AppLayout;

impl Layout for AppLayout {
    fn render(&self, ctx: &LayoutContext) -> String {
        let nav = navigation(&[
            NavItem::new("Dashboard", "/"),
            NavItem::new("Users", "/users"),
        ]);
        let sidebar = sidebar(&[
            SidebarSection::new("Quick Actions", vec![/* ... */]),
        ]);

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    {head}
</head>
<body class="{body_class}">
    {nav}
    <div class="flex">
        {sidebar}
        <main class="flex-1 p-6">
            <div id="ferro-json-ui"
                 data-view="{view_escaped}"
                 data-props="{data_escaped}">
                {content}
            </div>
        </main>
    </div>
</body>
</html>"#,
            title = ctx.title,
            head = ctx.head,
            body_class = ctx.body_class,
            nav = nav,
            sidebar = sidebar,
            view_escaped = ctx.view_json,
            data_escaped = ctx.data_json,
            content = ctx.content,
        )
    }
}
```

### Closure-Based Layout (No Struct Needed)
```rust
// Source: Maud functional pattern — simpler alternative to trait
use std::sync::Arc;

type LayoutFn = Arc<dyn Fn(&LayoutContext) -> String + Send + Sync>;

// Register inline
registry.register("minimal", Arc::new(|ctx: &LayoutContext| {
    format!("<!DOCTYPE html><html><body>{}</body></html>", ctx.content)
}));
```

### Framework Integration Point
```rust
// Source: Current framework/src/json_ui/mod.rs pattern, evolved
impl JsonUi {
    pub fn render(view: &JsonUiView, data: &serde_json::Value) -> Response {
        let config = JsonUiConfig::new();
        let resolved = Self::resolve(view);
        let content = render_to_html(&resolved, data);

        let ctx = LayoutContext {
            title: resolved.title.as_deref().unwrap_or("Ferro"),
            content: &content,
            head: &Self::build_head(&config),
            body_class: &config.body_class,
            data,
            view_json: &serde_json::to_string(&resolved).unwrap_or_default(),
            data_json: &serde_json::to_string(data).unwrap_or_default(),
        };

        // Look up layout from view.layout field
        let layout_name = resolved.layout.as_deref();
        let html = LAYOUT_REGISTRY.render(layout_name, &ctx);

        Ok(HttpResponse::text(html)
            .status(200)
            .header("Content-Type", "text/html; charset=utf-8"))
    }
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Template file inheritance (Tera/Askama extends) | Functional composition (Maud/Leptos style) | 2023-2024 | No filesystem dependency, type-safe |
| Implicit layout nesting (Rails convention) | Explicit layout selection per view | 2024+ | Clearer control flow, easier debugging |
| Generic slot maps | Typed context structs | Ongoing | Better DX, compile-time safety |
| Client-side layout rendering (SPA) | Server-side layout + HTML streaming | 2023+ | Better SEO, reduced client complexity |

**New patterns to consider:**
- **Streaming SSR layouts:** Phoenix LiveView 0.20+ streams layout + content separately. Relevant for large pages but likely overkill for Phase 29.
- **Layout-as-component:** Leptos/Dioxus treat layouts as route-level components. For JSON-UI (no client-side router), layouts stay server-side.

**Deprecated/outdated:**
- **Template file scanning at runtime:** Tera's `Tera::new("templates/**/*")` is a development convenience. Production systems compile templates at build time or use Rust functions.
- **Deep inheritance chains:** 3+ level extends (`base > app > admin > page`) considered fragile. Prefer flat composition with partials.
</sota_updates>

<open_questions>
## Open Questions

1. **Should layouts be in ferro-json-ui or framework crate?**
   - What we know: Layout trait/registry is infrastructure (ferro-json-ui), but default layouts with navigation/sidebar may depend on framework features (auth, routing).
   - What's unclear: Whether `ferro-json-ui` should own the registry or just the trait.
   - Recommendation: Trait + LayoutContext + DefaultLayout in `ferro-json-ui`. Registry + app-specific layouts in `framework`. This matches the existing crate boundary pattern.

2. **Static or dynamic layout registry?**
   - What we know: Current `JsonUiConfig` is passed per-render. A global registry (like route names) is simpler to use.
   - What's unclear: Whether users need to register layouts at app startup or if a fixed set of built-in layouts suffices for v3.0.
   - Recommendation: Start with a global `once_cell` registry (matching route name registration pattern). Users call `register_layout("name", layout)` at app startup.

3. **How do partials get their data?**
   - What we know: Navigation needs route links, sidebar may need user info, footer is typically static.
   - What's unclear: How much dynamic data partials should receive vs how much should be hardcoded.
   - Recommendation: Partials receive `&LayoutContext` which includes `data: &serde_json::Value`. Layout implementations extract what they need. Keep it simple for v3.0.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- Ferro codebase — ferro-json-ui/src/render.rs, framework/src/json_ui/mod.rs (current architecture)
- Ferro codebase — ferro-json-ui/src/view.rs (existing `layout: Option<String>` field)
- Phase 28 summary — render pipeline architecture, 20-component HTML renderer

### Secondary (MEDIUM confidence)
- [Phoenix LiveView layouts](https://hexdocs.pm/phoenix_live_view/live-layouts.html) — Three-tier layout model (root/app/content), `@inner_content` slot pattern
- [Blazor Server layouts](https://learn.microsoft.com/en-us/aspnet/core/blazor/components/layouts) — `@Body` directive, `LayoutComponentBase` inheritance
- [Maud functional composition](https://maud.lambda.xyz/) — Layout-as-function pattern, `Markup` type composition
- [Askama template inheritance](https://askama.readthedocs.io/en/stable/template_syntax) — `extends`/`block` conceptual model
- [Leptos nested routing layouts](https://book.leptos.dev/router/17_nested_routing.html) — `<Outlet />` pattern
- [Dioxus layout system](https://dioxuslabs.com/learn/0.6/router/reference/layouts/) — `#[layout()]` attribute
- [Airbnb Ghost SDUI](https://medium.com/airbnb-engineering/a-deep-dive-into-airbnbs-server-driven-ui-system-842244c5f5) — Section/Screen/Placement model
- [HTMX template fragments](https://htmx.org/essays/template-fragments/) — Partial update patterns

### Tertiary (LOW confidence - needs validation)
- Lyft SDUI philosophy ("design for needed flexibility, not maximal") — From secondary article, not direct Lyft source
- Streaming SSR layout patterns — Mentioned in LiveView context, not directly applicable to Ferro's sync render
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Rust string-based HTML generation (existing ferro-json-ui pattern)
- Ecosystem: SDUI layout patterns (Airbnb, Phoenix, Blazor, HTMX), Rust HTML frameworks (Maud, Askama, Leptos, Dioxus)
- Patterns: Functional layouts, layout registry, partials, slot composition
- Pitfalls: Over-abstraction, duplication, contract breakage, component coupling

**Confidence breakdown:**
- Standard stack: HIGH — No new dependencies, extends existing architecture
- Architecture: HIGH — Functional layout pattern is well-established across Maud, Phoenix, Blazor
- Pitfalls: HIGH — Identified from real SDUI production systems and template engine anti-patterns
- Code examples: HIGH — Derived from existing ferro-json-ui code structure and verified patterns

**Research date:** 2026-02-09
**Valid until:** 2026-03-11 (30 days — stable domain, no fast-moving ecosystem)
</metadata>

---

*Phase: 29-layout-system*
*Research completed: 2026-02-09*
*Ready for planning: yes*
