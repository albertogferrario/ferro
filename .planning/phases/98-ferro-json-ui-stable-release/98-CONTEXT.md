# Phase 98: ferro-json-ui Stable Release - Context

**Gathered:** 2026-03-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Stabilize ferro-json-ui from experimental (0.1.x) to production-ready. Complete the component catalog against real dashboard requirements (gestiscilo), add a built-in JS runtime for client-side behaviors, audit and restrict the public API surface, add comprehensive tests, generate JSON Schema, and write full documentation. This is not just a version bump — it completes the library against a real use case, then locks the stable API.

</domain>

<decisions>
## Implementation Decisions

### API Surface
- Add convenience constructors for all components (e.g., `ComponentNode::card("key", CardProps { ... })` or `Component::card(title)` shortcuts) — struct literals remain available but constructors improve DX
- Stabilize the entire API including the plugin system (JsonUiPlugin trait, PluginRegistry, PluginProps) — no experimental gates
- Audit and restrict public visibility: internal helpers (resolve_path, resolve_path_string, collect_plugin_types, render_to_html_with_plugins) become `pub(crate)` unless users genuinely need them
- Layout system (AppLayout, AuthLayout, LayoutRegistry, global_registry, register_layout, render_layout) becomes framework-internal (`pub(crate)`) — users set layout by name string only
- Full audit of all 20 component variants before stable — remove unused/redundant, consolidate where possible, lock only what's proven
- Include JSON Schema generation for JsonUiView and all component types (schemars derives, following ferro-projections pattern)

### New Components (from gestiscilo requirements)
- **StatCard** — single metric display: label, value, optional icon, subtitle. Value formats: integer count, currency. Live-updateable via SSE
- **Checklist** — container with title, dismiss button, list of checkbox items with label/link/checked state. Auto-hides when all checked. Dismissible. Server-side state persistence via data attributes
- **Toast** — viewport-anchored notification. Auto-dismiss (~5s default, configurable). Manual dismiss. Variants: info/success/warning/error. Stackable. SSE-triggered via JS runtime
- **NotificationDropdown** — anchored to bell icon, recent notifications list, each with icon/text/timestamp, "mark as read" action, empty state
- **Sidebar** — dynamic composition from data: fixed top/bottom items, collapsible groups with icon+label child items, active state highlighting, conditional rendering based on tenant services
- **Header** — business name, bell notification icon with unread count badge, user avatar/logout dropdown

### Dashboard Shell
- DashboardLayout is a new layout type (alongside AppLayout/AuthLayout), not a component in the view tree
- Sidebar and Header are layout-level constructs that persist across page navigation — content area swaps on route change
- Mobile: sidebar collapses into hamburger menu

### Built-in JS Runtime
- ferro-json-ui ships a small JS file (~5-10KB) as a core part of the library — not a plugin
- Handles: SSE connections, toast display/stacking/auto-dismiss, live value replacement on components
- Auto-initializes on page load — zero config for users
- Components emit semantic data attributes (data-sse-target, data-toast-variant, data-live-value, etc.) that the JS runtime reads

### Action System
- Claude's Discretion: evaluate whether string-based handler references ("users.create") should gain a type-safe companion (Action::route(name)) or remain string-only. Respect the JSON-UI vision of server-driven declarative UI

### Visibility System
- Claude's Discretion: evaluate whether compound conditions (AND/OR) are needed based on real usage in projections and gestiscilo, or if current path-based conditions suffice

### serde_json Re-export
- Claude's Discretion: decide whether to keep `pub use serde_json` based on whether it creates problematic version coupling

### Documentation
- Full component catalog: one page (or section) per component with props, code examples, and rendered preview description
- Dedicated plugin guide: how to create a plugin, register it, handle assets — separate page
- No migration guide (project not in production, no external users)
- Claude's Discretion: rustdoc example coverage level (key types vs all pub items)
- Claude's Discretion: doc structure (dedicated json-ui/ section vs flat under features/)

### Test Coverage
- Comprehensive test suite: serde round-trip for every component + render pipeline integration tests + edge cases (empty children, null data, missing optional fields, nested components) — targeting 60+ tests
- JSON Schema generation tests with snapshot comparison (following ferro-projections pattern)
- Full plugin pipeline tests: MapPlugin registration, rendering, asset collection — validates entire plugin contract

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-json-ui/src/component.rs` — 20 existing component variants (Card, Table, Form, Button, Switch, Input, Select, Alert, Tabs, Badge, Avatar, Breadcrumb, DescriptionList, Modal, Pagination, Progress, Separator, Skeleton, Text, Plugin)
- `ferro-json-ui/src/view.rs` — JsonUiView builder with schema versioning ("ferro-json-ui/v1")
- `ferro-json-ui/src/plugin.rs` — Full plugin system (JsonUiPlugin trait, registry, assets)
- `ferro-json-ui/src/render.rs` — HTML render engine with Tailwind CSS classes for all 20 components
- `ferro-json-ui/src/visibility.rs` — Conditional rendering system (path/operator/value)
- `ferro-json-ui/src/action.rs` — Action system with handler references, HTTP methods, confirm dialogs
- `ferro-json-ui/src/layout.rs` — AppLayout, AuthLayout, LayoutRegistry, NavItem, SidebarSection

### Established Patterns
- Components use serde tagged enum (`Component::Card(CardProps)`) with `#[serde(tag = "type")]`
- Builder pattern: `JsonUiView::new().title("X").component(node)` — consuming `mut self -> Self`
- Plugin components dispatched to registry, CSS/JS assets collected separately
- Render engine walks component tree producing HTML fragment with Tailwind classes
- No TODO/FIXME/unimplemented markers — all 20 components fully implemented

### Integration Points
- Framework re-exports behind `#[cfg(feature = "json-ui")]` in `framework/src/lib.rs`
- ferro-mcp uses ferro-json-ui for projection rendering
- ferro-projections' JsonUiRenderer outputs ferro-json-ui/v1 JSON envelopes
- Publish workflow: Wave 1 crate (no internal dependencies)
- Workspace version: 0.1.87

</code_context>

<specifics>
## Specific Ideas

- Real-world validation: gestiscilo dashboard (docs/FERRO-JSON-UI-REQUIREMENTS.md) drives the component catalog — every new component has a concrete use case
- StatCard live updates via SSE: `data-sse-target="orders_today"` attribute, JS runtime swaps the value element when matching event arrives
- Toast stacking: multiple toasts visible simultaneously, auto-dismiss with configurable timeout
- Sidebar dynamic composition: server sends JSON structure with groups/items, active state derived from current route
- "Persistent frames are sacred" — DashboardLayout sidebar and header never unmount or reflow during navigation

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 98-ferro-json-ui-stable-release*
*Context gathered: 2026-03-11*
