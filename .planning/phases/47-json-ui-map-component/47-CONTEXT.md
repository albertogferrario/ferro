# Phase 47: JSON-UI Map Component - Context

**Gathered:** 2026-02-10
**Status:** Ready for research

<vision>
## How This Should Work

This phase is really two things in one: establishing a **plugin system for JSON-UI** and building the **Map component as the first built-in plugin** that proves the pattern works.

JSON-UI today is pure server-rendered HTML + Tailwind — 20 static components, no client-side JS. But developers and agents will need custom interactive components (maps, charts, rich editors). Rather than bolting JS onto the existing renderer ad-hoc, this phase introduces a formal plugin architecture.

A `JsonUiPlugin` trait lets anyone define a custom component by providing: a component name, props schema (for agent discovery), a render function (HTML string), and asset declarations (JS/CSS to load). Plugins register at app startup. The renderer checks built-in components first, then falls back to the plugin registry.

The Map component ships as a built-in plugin using Leaflet. It follows a "start simple, grow" philosophy — pass center coordinates and a markers array, get a working interactive map. Zero JS knowledge needed. Advanced config (tiles, layers, popups, clustering) is available but optional.

</vision>

<essential>
## What Must Be Nailed

- **Plugin system design** — the `JsonUiPlugin` trait, registry, and asset loading pipeline must be clean and extensible. This is the foundation for all future interactive components.
- **Working Map component** — a Leaflet-based map that renders from JSON with markers. Dead simple defaults, optional advanced config. Proves the plugin system works end-to-end.
- **DX and agent experience** — agents must be able to discover plugin components, understand their props, and generate valid JSON just like built-in components. Developers must be able to create custom plugins with minimal boilerplate.

</essential>

<specifics>
## Specific Ideas

- Map library: Leaflet (open source, free, lightweight)
- Plugin components use dynamic dispatch through a registry; built-in components stay as static enum for performance
- Map could be the first "built-in plugin" — ships with ferro-json-ui but uses the plugin system, proving it works
- Props schema exposed by plugins enables MCP/agent discovery of custom components
- Asset management: plugins declare what JS/CSS they need, the renderer handles loading them in the page

</specifics>

<notes>
## Additional Context

This phase expanded from "add a Map component" to "design the plugin system + Map as first example" because the user wants JSON-UI to support custom interactive components with excellent DX and agent experience. The plugin system is equally important as the Map itself — neither ships without the other.

The "start simple, grow" philosophy for Map props mirrors JSON-UI's overall approach: minimal config for common cases, full power available when needed.

</notes>

---

*Phase: 47-json-ui-map-component*
*Context gathered: 2026-02-10*
