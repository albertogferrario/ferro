---
phase: 133-generalize-renderer-trait
plan: "01"
subsystem: ferro-projections
tags: [renderer, projections, refactor, architecture]
dependency_graph:
  requires: []
  provides: [modality-agnostic Renderer trait, BaseContext, VisualContext, optional visual feature]
  affects: [ferro-projections, ferro-mcp (consumers of JsonUiRenderer/VisualContext)]
tech_stack:
  added: []
  patterns: [associated types on traits, optional Cargo features, feature-gated modules]
key_files:
  created: []
  modified:
    - ferro-projections/Cargo.toml
    - ferro-projections/src/render/mod.rs
    - ferro-projections/src/render/json_ui.rs
    - ferro-projections/src/render/template.rs
    - ferro-projections/src/lib.rs
decisions:
  - "VisualContext is a flat struct (not composed from BaseContext) — avoids extra indirection and matches D-06"
  - "visual feature defaults to enabled so existing consumers (ferro-mcp) work without opt-in"
  - "RenderMode moved to json_ui.rs co-located with VisualContext as visual concerns"
metrics:
  duration: "~3.5 min"
  completed: "2026-04-14"
  tasks_completed: 1
  files_modified: 5
---

# Phase 133 Plan 01: Generalize Renderer Trait Summary

Renderer trait refactored to associated types. The monolithic `RenderContext` is split into `BaseContext` (modality-agnostic) and `VisualContext` (visual-only). `ferro-theme` is now optional behind the `visual` feature flag.

## What Was Built

The `Renderer` trait in `ferro-projections` previously hardcoded `serde_json::Value` as its output type and `RenderContext` as its context type, coupling all future renderers to visual-only concerns (theme templates, render mode). This refactor makes the trait output-agnostic via associated types, enabling non-visual renderers (WhatsApp, voice, etc.) to implement the same trait without dragging in visual dependencies.

**Key changes:**

- `Renderer` trait now has `type Output` and `type Context: Default` associated types
- `RenderContext` is removed; replaced by `BaseContext` (only `intent_index` and `current_state`) in `render/mod.rs`
- `RenderMode` and `VisualContext` (adding `mode` and `templates`) move to `render/json_ui.rs`
- `json_ui` module gated behind `#[cfg(feature = "visual")]`
- `ferro-theme` dependency made `optional = true` behind the `visual` feature (default enabled)
- `JsonUiRenderer` implements `Renderer<Output=Value, Context=VisualContext>`
- `TemplateRenderer` implements `Renderer<Output=Value, Context=BaseContext>`
- `lib.rs` exports `BaseContext` unconditionally; `JsonUiRenderer`, `RenderMode`, `VisualContext` behind `#[cfg(feature = "visual")]`

## Verification

All acceptance criteria pass:

- `grep 'type Output' ferro-projections/src/render/mod.rs` succeeds
- `grep 'type Context: Default' ferro-projections/src/render/mod.rs` succeeds
- `grep 'pub struct BaseContext' ferro-projections/src/render/mod.rs` succeeds
- `grep 'pub struct VisualContext' ferro-projections/src/render/json_ui.rs` succeeds
- `grep 'pub enum RenderMode' ferro-projections/src/render/json_ui.rs` succeeds
- `grep 'optional = true' ferro-projections/Cargo.toml` succeeds
- `grep 'BaseContext' ferro-projections/src/lib.rs` succeeds
- No `RenderContext` in `ferro-projections/src/lib.rs`
- `cargo test -p ferro-projections --all-features`: 317 tests passed, 0 failed

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. All rendered data flows through real field/relationship mapping functions.

## Self-Check: PASSED

- `ferro-projections/src/render/mod.rs` — exists, contains `type Output`, `type Context: Default`, `pub struct BaseContext`
- `ferro-projections/src/render/json_ui.rs` — exists, contains `pub struct VisualContext`, `pub enum RenderMode`
- `ferro-projections/src/render/template.rs` — exists, contains `type Context = BaseContext`
- `ferro-projections/src/lib.rs` — exists, contains `pub use render::{BaseContext, Renderer}` and conditional `VisualContext`
- `ferro-projections/Cargo.toml` — exists, contains `optional = true` for ferro-theme
- Commit `48b3cf14` — present in git log
