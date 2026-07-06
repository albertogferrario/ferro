# Phase 125 — Module scaffolder and json-ui runtime split

## Context
Both gestiscilo and mkmenu independently arrived at the same `src/modules/<name>/`
layout (controller / model / views / routes per feature), and both did it by
hand-copying from the previous module. The convention is real but unowned by
the framework, so it drifts: gestiscilo's `pages` module nests `views/`
differently than its `bookings` module. A `make:module` command captures the
shape once.

The runtime split is unrelated in scope but shares the "growing pains" theme:
`ferro-json-ui/src/runtime.rs` started as a 30-line IIFE for tab switching and
has accreted SSE, toasts, and the sidebar toggle. It is one giant string
literal in Rust, so editing any feature touches the whole bundle and there are
no tests. Splitting into per-concern Rust modules that compose into the same
emitted file keeps the zero-HTTP-request shape while making each piece
testable. Bundled here because both items are pure DX cleanup without API
impact.

## Goal
Codify the feature-module convention real Ferro apps converge on (gestiscilo,
mkmenu) into a `make:module` scaffolder, and break up the ferro-json-ui runtime
IIFE so it stays maintainable as feature surface grows.

## Scope

### `ferro make:module <name>`
Generate a feature-module skeleton:
```
src/modules/{name}/
  mod.rs              # re-exports controller/model/views/routes
  controller.rs       # empty handler stubs
  model.rs            # empty SeaORM entity stub
  views/
    mod.rs
    index.rs          # ferro-json-ui view stub
  routes.rs           # router builder, exported as register(router)
```
Update `src/modules/mod.rs` (creating it if missing) to declare the new module.
Optionally update `src/lib.rs` or `src/main.rs` route registration if a stable
hook is detected.

Flags:
- `--with-migration` to also drop a `migration/src/m_{ts}_create_{name}.rs`.
- `--no-views` for headless modules.
- `--force` to overwrite.

### ferro-json-ui runtime split
Current state: `ferro-json-ui/src/runtime.rs` emits a single monolithic IIFE
containing tab switching, SSE, toasts, sidebar toggle. Hard to test, hard to
extend.

Refactor:
- Split into named JS functions per concern: `setupTabs()`, `setupSSE()`,
  `setupToasts()`, `setupSidebar()`.
- Wrap in a single dispatcher `function ferroRuntime() { ... }` that calls each
  setup function and is invoked once on `DOMContentLoaded`.
- Keep emitted as one file (no extra HTTP requests) but built from multiple
  source string constants in Rust so each concern lives in its own Rust module.
- Add minimal Rust-side unit test that the emitted bundle contains all expected
  function names (no JS runtime needed).

## Verification
- `ferro make:module orders` in a fresh project produces a compiling module
  declared in `src/modules/mod.rs` with a registered route.
- gestiscilo's existing tabs/SSE/toasts/sidebar behavior unchanged after the
  runtime refactor (manual UAT via Chrome MCP per CLAUDE.md).

## Out of scope
- Migrating existing gestiscilo modules to the new layout.
- Replacing the IIFE with ES modules or a build-step bundler.
