# Phase 125: Module Scaffolder + JSON-UI Runtime Split - Context

**Gathered:** 2026-04-07
**Status:** Ready for planning
**Mode:** Auto (decisions sourced from SCOPE.md)

<domain>
## Phase Boundary

Two independent DX-cleanup deliverables bundled by theme:
1. `ferro make:module <name>` — codify the feature-module convention (controller/model/views/routes per feature) into a generator.
2. ferro-json-ui runtime split — break the monolithic IIFE in `ferro-json-ui/src/runtime.rs` into per-concern Rust modules that compose into the same emitted bundle (zero extra HTTP requests), with Rust-side unit tests.

Source of truth: `.planning/phases/125-module-scaffolder-and-json-ui-runtime-split/SCOPE.md`.
</domain>

<decisions>
## Implementation Decisions

### make:module
- **D-01:** Command: `ferro make:module <name>`. Generates the skeleton:
  ```
  src/modules/{name}/
    mod.rs (re-exports controller/model/views/routes)
    controller.rs (empty handler stubs)
    model.rs (empty SeaORM entity stub)
    views/mod.rs
    views/index.rs (ferro-json-ui view stub)
    routes.rs (router builder, exported as `register(router)`)
  ```
- **D-02:** Update `src/modules/mod.rs` (create if missing) to declare the new module.
- **D-03:** Optionally update `src/lib.rs` or `src/main.rs` route registration when a stable hook is detected.
- **D-04:** Flags:
  - `--with-migration` → also drop `migration/src/m_{ts}_create_{name}.rs`.
  - `--no-views` → headless modules (skip views/ subtree).
  - `--force` → overwrite existing files.
- **D-05:** Generated module must compile cleanly in a fresh project (Verification target).

### ferro-json-ui runtime split
- **D-06:** Refactor `ferro-json-ui/src/runtime.rs` into named JS functions per concern: `setupTabs()`, `setupSSE()`, `setupToasts()`, `setupSidebar()`.
- **D-07:** Wrap in a single dispatcher `function ferroRuntime() { ... }` invoked once on `DOMContentLoaded`.
- **D-08:** Emit as one bundled file (no extra HTTP requests) — but assembled from multiple Rust source string constants, one per Rust submodule.
- **D-09:** Each concern lives in its own Rust module (e.g. `runtime/tabs.rs`, `runtime/sse.rs`, `runtime/toasts.rs`, `runtime/sidebar.rs`, `runtime/mod.rs` as the assembler).
- **D-10:** Add Rust-side unit tests that the emitted bundle contains the expected function names — no JS runtime required.
- **D-11:** Behavior must be unchanged — gestiscilo tabs/SSE/toasts/sidebar continue to work post-refactor (manual Chrome MCP UAT per CLAUDE.md).

### Cross-cutting
- **D-12:** Two deliverables are independent — they can be sliced into separate plans, no ordering between them.

### Claude's Discretion
- Exact stub content for generated controller/model/views (planner picks minimal-but-illustrative).
- How to detect a "stable route registration hook" in src/main.rs/lib.rs (heuristic: comment marker, `register_modules!()` macro, or skip if not found).
- Whether to add a `register_all!()` macro vs explicit calls in `modules/mod.rs`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope
- `.planning/phases/125-module-scaffolder-and-json-ui-runtime-split/SCOPE.md` — authoritative scope.

### Existing code to refactor
- `ferro-json-ui/src/runtime.rs` — current monolithic IIFE (the refactor target).
- `ferro-cli/src/commands/` — pattern for `make:*` commands (likely existing `make:model`, `make:controller`, `make:view`).
- `ferro-cli/src/templates/files/` — template directory pattern.
- `ferro-cli/src/main.rs` — clap registration.
- `ferro-cli/src/project.rs` — `find_project_root` (Phase 122) for walk-up Cargo.toml discovery.

### Convention reference
- gestiscilo `src/modules/` and mkmenu `src/modules/` (off-disk) — the de facto convention. Stub content in this phase encodes the canonical shape.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro_cli::project::find_project_root` (walk-up Cargo.toml).
- Existing template rendering pipeline in `ferro-cli/src/templates/`.
- `--force` flag pattern from Phase 122/124.
- Existing `make:model` / `make:controller` (or similar) — locate and follow the pattern.

### Established Patterns
- ferro-cli command pattern: `commands/<name>.rs` + clap variant in `main.rs`.
- Templates as `include_str!` literals under `templates/files/` rendered via context structs.

### Integration Points
- `make:module` writes into the user's project, not the framework — pure scaffolder.
- runtime split is purely internal to `ferro-json-ui` crate; public API stays the same (the function that returns the runtime JS string).

</code_context>

<specifics>
## Specific Ideas

- The scaffolded module's `routes.rs` exports `pub fn register(router: Router) -> Router` — this is the canonical hook so users can wire it from `main.rs` with one call.
- The dispatcher function is named `ferroRuntime` (camelCase JS) — must appear verbatim in the emitted bundle (test assertion).
- Use Chrome MCP for the runtime refactor UAT per CLAUDE.md "Always test UI changes with Chrome MCP proactively".

</specifics>

<deferred>
## Deferred Ideas

- Migrating existing gestiscilo modules to the new layout → out of scope (one-time human task).
- Replacing IIFE with ES modules / build-step bundler → out of scope (would break zero-HTTP-request guarantee).

</deferred>

---

*Phase: 125-module-scaffolder-and-json-ui-runtime-split*
*Context gathered: 2026-04-07 (auto mode)*
