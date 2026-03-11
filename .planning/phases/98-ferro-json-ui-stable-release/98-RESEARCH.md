# Phase 98: ferro-json-ui Stable Release - Research

**Researched:** 2026-03-11
**Domain:** Rust library API stabilization, server-driven UI components, JSON Schema generation, JS runtime bundling
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**API Surface**
- Add convenience constructors for all components (e.g., `ComponentNode::card("key", CardProps { ... })` or `Component::card(title)` shortcuts) — struct literals remain available but constructors improve DX
- Stabilize the entire API including the plugin system (JsonUiPlugin trait, PluginRegistry, PluginProps) — no experimental gates
- Audit and restrict public visibility: internal helpers (resolve_path, resolve_path_string, collect_plugin_types, render_to_html_with_plugins) become `pub(crate)` unless users genuinely need them
- Layout system (AppLayout, AuthLayout, LayoutRegistry, global_registry, register_layout, render_layout) becomes framework-internal (`pub(crate)`) — users set layout by name string only
- Full audit of all 20 component variants before stable — remove unused/redundant, consolidate where possible, lock only what's proven
- Include JSON Schema generation for JsonUiView and all component types (schemars derives, following ferro-projections pattern)

**New Components (from gestiscilo requirements)**
- **StatCard** — single metric display: label, value, optional icon, subtitle. Value formats: integer count, currency. Live-updateable via SSE
- **Checklist** — container with title, dismiss button, list of checkbox items with label/link/checked state. Auto-hides when all checked. Dismissible. Server-side state persistence via data attributes
- **Toast** — viewport-anchored notification. Auto-dismiss (~5s default, configurable). Manual dismiss. Variants: info/success/warning/error. Stackable. SSE-triggered via JS runtime
- **NotificationDropdown** — anchored to bell icon, recent notifications list, each with icon/text/timestamp, "mark as read" action, empty state
- **Sidebar** — dynamic composition from data: fixed top/bottom items, collapsible groups with icon+label child items, active state highlighting, conditional rendering based on tenant services
- **Header** — business name, bell notification icon with unread count badge, user avatar/logout dropdown

**Dashboard Shell**
- DashboardLayout is a new layout type (alongside AppLayout/AuthLayout), not a component in the view tree
- Sidebar and Header are layout-level constructs that persist across page navigation — content area swaps on route change
- Mobile: sidebar collapses into hamburger menu

**Built-in JS Runtime**
- ferro-json-ui ships a small JS file (~5-10KB) as a core part of the library — not a plugin
- Handles: SSE connections, toast display/stacking/auto-dismiss, live value replacement on components
- Auto-initializes on page load — zero config for users
- Components emit semantic data attributes (data-sse-target, data-toast-variant, data-live-value, etc.) that the JS runtime reads

**Documentation**
- Full component catalog: one page (or section) per component with props, code examples, and rendered preview description
- Dedicated plugin guide: how to create a plugin, register it, handle assets — separate page
- No migration guide (project not in production, no external users)

**Test Coverage**
- Comprehensive test suite: serde round-trip for every component + render pipeline integration tests + edge cases (empty children, null data, missing optional fields, nested components) — targeting 60+ tests
- JSON Schema generation tests with snapshot comparison (following ferro-projections pattern)
- Full plugin pipeline tests: MapPlugin registration, rendering, asset collection — validates entire plugin contract

### Claude's Discretion
- Evaluate whether string-based handler references ("users.create") should gain a type-safe companion (Action::route(name)) or remain string-only. Respect the JSON-UI vision of server-driven declarative UI
- Evaluate whether compound conditions (AND/OR) are needed based on real usage in projections and gestiscilo, or if current path-based conditions suffice
- Decide whether to keep `pub use serde_json` based on whether it creates problematic version coupling
- Rustdoc example coverage level (key types vs all pub items)
- Doc structure (dedicated json-ui/ section vs flat under features/)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

## Summary

ferro-json-ui is a mature experimental library with 20 fully implemented component variants, a working plugin system, a layout registry, action resolution, visibility conditions, and data path resolution — all with 263 existing tests. The codebase is internally consistent, follows the project's builder/consuming patterns, and produces correct serde tagged JSON.

Phase 98 is a stabilization effort, not a greenfield build. The work divides into four categories: (1) add 6 new components driven by gestiscilo dashboard requirements, (2) add a DashboardLayout and make layouts framework-internal, (3) ship a built-in JS runtime (~5-10KB) for SSE/toast/live-value behaviors, and (4) lock the API by auditing visibility, adding schemars JSON Schema derives, expanding tests to 60+, and writing complete documentation.

The schemars pattern to follow is already established in ferro-projections: add `schemars = { version = "1", features = ["derive"] }` to Cargo.toml, add `JsonSchema` to derives, and write `schemars::schema_for!(Type)` snapshot tests.

**Primary recommendation:** Work wave-by-wave: (1) new components + renders, (2) DashboardLayout + JS runtime, (3) API visibility audit + schemars, (4) tests to 60+, (5) documentation.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1.0 | Serialization | Already used, all types derive it |
| serde_json | 1.0 | JSON encoding/decoding | Already used throughout |
| schemars | 1 (features = ["derive"]) | JSON Schema generation | ferro-projections pattern; enables MCP/agent discovery |

### Already Present, No Changes Needed
| Library | Version | Purpose |
|---------|---------|---------|
| serde | 1.0 | All component types |
| serde_json | 1.0 | view.rs, data.rs, plugin.rs |

**Installation (additions only):**
```bash
# Add to ferro-json-ui/Cargo.toml [dependencies]
schemars = { version = "1", features = ["derive"] }
```

## Architecture Patterns

### Current Project Structure
```
ferro-json-ui/src/
├── lib.rs           # Public API re-exports (to be audited)
├── component.rs     # 20 component variants (enum + props structs)
├── view.rs          # JsonUiView builder
├── action.rs        # Action, ActionOutcome, ConfirmDialog
├── visibility.rs    # Visibility (AND/OR/NOT/Condition)
├── render.rs        # HTML render engine with Tailwind classes
├── layout.rs        # Layout trait, LayoutRegistry, built-in layouts
├── plugin.rs        # JsonUiPlugin trait, PluginRegistry, Asset
├── plugins/
│   ├── mod.rs       # register_built_in_plugins
│   └── map.rs       # MapPlugin (Leaflet 1.9.4)
├── data.rs          # resolve_path, resolve_path_string
├── resolve.rs       # resolve_actions, resolve_errors
└── config.rs        # JsonUiConfig
```

### Pattern 1: New Component Props Struct
**What:** Add `*Props` struct with `#[serde(rename_all = "snake_case")]` on enums and `skip_serializing_if` on optionals. Add variant to `Component` enum. Implement render in `render.rs`.
**When to use:** Every new component follows this exact shape.
**Example:**
```rust
// Source: existing component.rs pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StatCardProps {
    pub label: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sse_target: Option<String>,  // data-sse-target attribute key
}

// In Component enum:
StatCard(StatCardProps),
```

### Pattern 2: schemars Derive (from ferro-projections)
**What:** Add `JsonSchema` to every public type's derive list. Use `schemars::schema_for!(T)` in tests to verify schema is generated without panic.
**Example:**
```rust
// Source: ferro-projections/src/field.rs
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FieldDef { ... }

#[test]
fn schema_generates() {
    let schema = schemars::schema_for!(FieldDef);
    let json = serde_json::to_value(&schema).unwrap();
    assert!(json.get("properties").is_some());
}
```

### Pattern 3: Built-in JS Runtime (following MapPlugin precedent)
**What:** A `const RUNTIME_JS: &str = r#"..."#` embedded in the crate, injected once per page as the first script in `LayoutContext.scripts`. Not a plugin — part of the default render pipeline.
**When to use:** Auto-initialized when any layout renders. JS reads semantic data attributes.
**Data attribute contract:**
- `data-sse-url="/path/to/events"` on body or wrapper div — JS opens EventSource
- `data-sse-target="metric_key"` on a StatCard value element — JS swaps text on matching event
- `data-toast-container` on the viewport anchor — JS appends toast elements here
- `data-live-value` marks an element whose text content is SSE-replaceable

### Pattern 4: DashboardLayout
**What:** New `DashboardLayout` struct implementing the `Layout` trait. Registered as "dashboard" in `LayoutRegistry`. Sidebar and Header are NOT component variants — they are layout-level constructs rendered by the layout. Server sends `Sidebar`/`Header` configuration as typed structs injected into `LayoutContext` or a new `DashboardLayoutContext`.
**Key constraint:** Sidebar and Header must persist across navigation — they are in the outer HTML shell, never inside the `ferro-json-ui` wrapper div.

### Pattern 5: API Visibility Audit
**What:** Move internal symbols from `pub` to `pub(crate)`. Update `lib.rs` re-exports accordingly.
**Symbols to demote:**
- `resolve_path`, `resolve_path_string` — used by render.rs internally, not needed by users
- `collect_plugin_types` — render pipeline internal
- `render_to_html_with_plugins` — framework uses `JsonUi::render()` which wraps this; direct exposure is rarely needed
- All layout partials (`navigation`, `sidebar`, `footer` functions) — superseded by typed layout structs
- `global_registry`, `register_layout`, `render_layout` — framework-internal, users register by name string via framework API
- `AppLayout`, `AuthLayout`, `DefaultLayout` — framework-internal layout structs

### Pattern 6: Convenience Constructors
**What:** `impl ComponentNode` static methods for common construction patterns. Struct literals remain valid.
**Example:**
```rust
impl ComponentNode {
    pub fn card(key: impl Into<String>, props: CardProps) -> Self {
        Self { key: key.into(), component: Component::Card(props), action: None, visibility: None }
    }
    pub fn button(key: impl Into<String>, props: ButtonProps) -> Self {
        Self { key: key.into(), component: Component::Button(props), action: None, visibility: None }
    }
    // one per variant
}
```

### Anti-Patterns to Avoid
- **Implementing Toast as a Component variant:** Toast is viewport-anchored and JS-driven — it belongs in the JS runtime, triggered by SSE events. The `Toast` component variant in the view tree only declares *intent* (triggers, config), not the DOM element.
- **Embedding full HTML frameworks in JS runtime:** Keep the runtime minimal (~5-10KB). No jQuery, no Alpine. Data attributes + vanilla JS only.
- **Making DashboardLayout depend on component tree internals:** Layout must be composable without walking the component tree. Sidebar config is separate from view components.
- **Adding schemars to PluginProps:** PluginProps uses custom Serialize/Deserialize — JsonSchema derive would conflict. Skip it or implement manually.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON Schema generation | Custom schema builder | `schemars` crate | Handles serde compat, optional fields, enums with tags, generics |
| SSE parsing in JS | Custom EventSource wrapper | Native `EventSource` API | All modern browsers support it natively; auto-reconnect built in |
| CSS animation for toasts | Custom keyframe builder | Tailwind `transition`/`animate-` classes | Consistent with existing render engine approach |
| Inline JS minification | Build-time minifier | Readable source embedded as `const` | No build pipeline; keep it maintainable |

**Key insight:** The existing MapPlugin's pattern (data attribute + embedded init script const) is the correct model for the JS runtime. Scale it from plugin-scoped to crate-scoped.

## Common Pitfalls

### Pitfall 1: schemars v1 vs v0.8 API difference
**What goes wrong:** schemars v1 has a different API than v0.8. ferro-projections uses `schemars = { version = "1", features = ["derive"] }` — use exactly this version specifier.
**Why it happens:** Many tutorials and crates still use v0.8.
**How to avoid:** Copy Cargo.toml entry from ferro-projections exactly. Verify with `cargo check` before writing schema tests.
**Warning signs:** `schema_for!` macro not found, or `JsonSchema` derive complaining about unknown attributes.

### Pitfall 2: PluginProps custom serde conflicts with JsonSchema derive
**What goes wrong:** `PluginProps` has hand-written `Serialize`/`Deserialize` impls. Adding `#[derive(JsonSchema)]` will work but the generated schema won't reflect the actual serialized shape.
**How to avoid:** Either skip `JsonSchema` on `PluginProps` and `Component::Plugin`, or add a manual `impl JsonSchema for PluginProps` that returns a passthrough object schema.

### Pitfall 3: Component enum custom serde + schemars
**What goes wrong:** `Component` also has custom `Serialize`/`Deserialize`. `#[derive(JsonSchema)]` will not auto-generate a tagged union schema.
**How to avoid:** Implement `JsonSchema` manually for `Component`, or generate per-variant schemas (one schema per props struct) rather than a unified `Component` schema. The ferro-projections approach generates schemas per type, not per enum.

### Pitfall 4: Layout visibility demotion breaks framework re-exports
**What goes wrong:** `framework/src/lib.rs` re-exports layout symbols under `#[cfg(feature = "json-ui")]`. Demoting to `pub(crate)` inside `ferro-json-ui` will break those re-exports.
**How to avoid:** Audit `framework/src/lib.rs` and `framework/src/json_ui.rs` before demoting. Decide what the framework actually needs to re-export for its own use vs what's user-facing. Framework-internal use is fine with `pub(crate)` — but framework's `json_ui.rs` module must use `crate::` paths inside ferro-json-ui, not external re-exports.

### Pitfall 5: DashboardLayout sidebar/header HTML escaping
**What goes wrong:** Sidebar group labels and nav item hrefs come from user data. The existing `html_escape` helper must be used consistently for all dynamic content rendered in DashboardLayout.
**How to avoid:** Follow the exact pattern in `layout.rs` — every user-controlled string goes through `html_escape(&value)`.

### Pitfall 6: Built-in JS runtime injected multiple times
**What goes wrong:** If `render_to_html_with_plugins` and layout rendering both inject the runtime script, it initializes twice.
**How to avoid:** Inject the runtime exactly once — as part of `LayoutContext.scripts`, prepended before plugin scripts. The layout's `render()` method controls script injection order.

### Pitfall 7: Lib.rs re-export audit breaks downstream usages
**What goes wrong:** ferro-json-ui is used by ferro-projections (for JsonUiRenderer) and ferro-mcp. Removing public items that these crates import will break compilation.
**How to avoid:** `grep -r "ferro_json_ui::"` across the workspace before demoting any symbol. Check `ferro-projections/src/render/json_ui.rs` and `ferro-mcp/src/` specifically.

## Code Examples

### Adding a New Component (StatCard)
```rust
// In component.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StatCardProps {
    pub label: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    /// Key for SSE live-value updates. Maps to data-sse-target attribute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sse_target: Option<String>,
}

// In Component enum:
StatCard(StatCardProps),

// In Component serialize match:
Component::StatCard(p) => serialize_tagged(serializer, "StatCard", p),

// In render.rs:
Component::StatCard(props) => render_stat_card(props),
```

### schemars Pattern (from ferro-projections)
```rust
// Cargo.toml
// schemars = { version = "1", features = ["derive"] }

use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JsonUiView { ... }

// Test:
#[test]
fn json_ui_view_schema_generates() {
    let schema = schemars::schema_for!(JsonUiView);
    let json = serde_json::to_value(&schema).unwrap();
    assert!(json.get("title").is_some() || json.get("properties").is_some());
}
```

### Convenience Constructor Pattern
```rust
impl ComponentNode {
    pub fn stat_card(key: impl Into<String>, props: StatCardProps) -> Self {
        Self {
            key: key.into(),
            component: Component::StatCard(props),
            action: None,
            visibility: None,
        }
    }
}
// Usage:
let node = ComponentNode::stat_card("orders-today", StatCardProps {
    label: "Ordini oggi".to_string(),
    value: "12".to_string(),
    ..Default::default()
});
```

### Built-in JS Runtime Injection
```rust
// In render.rs (or a new runtime.rs)
pub(crate) const FERRO_RUNTIME_JS: &str = r#"
(function() {
  // SSE connection management
  var sseUrl = document.body.getAttribute('data-sse-url');
  if (sseUrl) {
    var es = new EventSource(sseUrl);
    es.onmessage = function(e) {
      var payload = JSON.parse(e.data);
      handleEvent(payload);
    };
  }

  function handleEvent(payload) {
    // Live value update
    document.querySelectorAll('[data-sse-target="' + payload.key + '"]').forEach(function(el) {
      el.textContent = payload.value;
    });
    // Toast
    if (payload.toast) {
      showToast(payload.toast);
    }
  }

  function showToast(config) {
    var container = document.querySelector('[data-toast-container]');
    if (!container) return;
    var el = document.createElement('div');
    el.className = 'ferro-toast ferro-toast--' + (config.variant || 'info');
    el.setAttribute('data-toast-variant', config.variant || 'info');
    el.textContent = config.message;
    container.appendChild(el);
    setTimeout(function() { el.remove(); }, (config.timeout || 5) * 1000);
  }
})();
"#;

// In layout rendering (layout.rs):
// The DashboardLayout injects FERRO_RUNTIME_JS first in ctx.scripts
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Experimental crate-level disclaimer | Stable 1.0 API | Phase 98 | API locked, no breaking changes without major version |
| `pub` on all symbols | Selective `pub` vs `pub(crate)` | Phase 98 | Cleaner user-facing surface, internal freedom |
| No JSON Schema | schemars derives on all public types | Phase 98 | MCP/agent discoverability, IDE tooling |
| No convenience constructors | `ComponentNode::card(key, props)` | Phase 98 | Improved ergonomics, less verbosity |
| Layouts fully pub | Layout internals `pub(crate)` | Phase 98 | Users interact with layout names only |
| No live-update support | SSE data attributes + JS runtime | Phase 98 | StatCard values update without page reload |

**Deprecated/outdated after Phase 98:**
- `experimental` crate doc disclaimer in lib.rs — remove after API lock
- Exporting `AppLayout`, `AuthLayout`, `DefaultLayout` as user-facing types — they become internal implementation details
- `navigation()`, `sidebar()`, `footer()` partial functions in public API — superseded by layout structs

## Open Questions

1. **Action::route() type-safe companion**
   - What we know: Current actions use `handler: String` in "controller.method" format. This matches the server-driven UI philosophy.
   - What's unclear: Whether gestiscilo's usage patterns show enough repetition to justify a typed constructor.
   - Recommendation: Keep string-only for now. The JSON-UI vision is declarative JSON that AI/agents can generate. Adding `Action::route()` can be done later without breaking changes.

2. **serde_json re-export (`pub use serde_json`)**
   - What we know: Currently re-exported in lib.rs. Downstream crates using `ferro_json_ui::serde_json` are version-locked to ferro-json-ui's serde_json version.
   - What's unclear: Whether ferro-projections or ferro-mcp actually use this re-export path.
   - Recommendation: Check with `grep -r "ferro_json_ui::serde_json"` in the workspace. If unused externally, remove. If used, keep but add a doc comment warning about version coupling.

3. **Component audit — which of the 20 to remove/consolidate**
   - What we know: All 20 are fully implemented with renders. The gestiscilo dashboard uses: Card, Table, Form, Button, Input, Select, Alert, Badge, Modal, Text, Checkbox, Switch, Tabs, Breadcrumb, Pagination, DescriptionList. Separator, Progress, Avatar, Skeleton appear less critical but have legitimate general-purpose use.
   - Recommendation: Keep all 20 — all are useful and removing any would reduce the library's completeness without a strong reason. The audit should confirm all have correct render output, not remove them.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` (cargo test) |
| Config file | none — standard cargo test runner |
| Quick run command | `cargo test -p ferro-json-ui` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command |
|--------|----------|-----------|-------------------|
| API-01 | All new component props round-trip serde | unit | `cargo test -p ferro-json-ui` |
| API-02 | Convenience constructors produce correct ComponentNodes | unit | `cargo test -p ferro-json-ui` |
| API-03 | JSON Schema generates for JsonUiView and all component types | unit | `cargo test -p ferro-json-ui` |
| API-04 | Visibility pub audit — demoted symbols not accessible externally | compile | `cargo build -p ferro-json-ui` |
| API-05 | DashboardLayout renders sidebar, header, content area | unit | `cargo test -p ferro-json-ui` |
| API-06 | Built-in JS runtime injected once in DashboardLayout output | unit | `cargo test -p ferro-json-ui` |
| API-07 | StatCard renders with data-sse-target attribute when sse_target set | unit | `cargo test -p ferro-json-ui` |
| API-08 | Toast component renders with correct data-toast-variant | unit | `cargo test -p ferro-json-ui` |
| API-09 | Plugin pipeline: MapPlugin registration, rendering, asset collection | unit | `cargo test -p ferro-json-ui` |
| API-10 | 60+ total tests pass | suite | `cargo test -p ferro-json-ui` |
| DOCS-01 | Rustdoc builds without warnings | doc | `cargo doc -p ferro-json-ui --no-deps` |

### Sampling Rate
- **Per task commit:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test -p ferro-json-ui`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
None — existing test infrastructure covers all phase requirements. The framework uses standard cargo test; no new test infrastructure needed.

## Sources

### Primary (HIGH confidence)
- Direct source inspection: `/ferro-json-ui/src/*.rs` — all 13 source files read
- Direct source inspection: `/ferro-projections/Cargo.toml` and `src/` — schemars v1 pattern confirmed
- Direct source inspection: `/docs/src/json-ui/` — existing doc structure (5 files, 2037 lines)
- Direct source inspection: `docs/FERRO-JSON-UI-REQUIREMENTS.md` — gestiscilo dashboard requirements
- Direct source inspection: `98-CONTEXT.md` — locked decisions

### Secondary (MEDIUM confidence)
- schemars v1 derive pattern confirmed by reading ferro-projections source directly; no web search needed given clear local evidence

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies already in workspace or directly verified in ferro-projections
- Architecture: HIGH — entire codebase read; all 13 source files examined
- Pitfalls: HIGH — identified from direct code inspection (custom serde impls, framework re-exports, layout visibility chain)

**Research date:** 2026-03-11
**Valid until:** 2026-06-11 (stable library, 90-day validity)
