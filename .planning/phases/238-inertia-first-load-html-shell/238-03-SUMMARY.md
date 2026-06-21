---
phase: 238-inertia-first-load-html-shell
plan: 03
subsystem: framework/inertia
tags: [inertia, framework, global-config, app]
requirements: [D-02, D-04]

dependency_graph:
  requires:
    - 238-01 (InertiaConfig::from_env() + new fields: title, head_extras, mount_id)
  provides:
    - INERTIA_CONFIG OnceLock<InertiaConfig> process global
    - set_inertia_config() / get_inertia_config() free functions (re-exported from inertia module)
    - App::set_inertia_config(config) method (#[cfg(feature = "inertia")])
    - Inertia::render and render_ctx read the configured global (D-02 fulfilled)
    - Fallback to from_env()/default() when nothing set (D-04 fulfilled)
  affects:
    - framework/src/inertia/global.rs (new)
    - framework/src/inertia/mod.rs
    - framework/src/inertia/context.rs
    - framework/src/container/mod.rs

tech_stack:
  added: []
  patterns:
    - Bare OnceLock<T> (no RwLock) — set-once semantics, immutable reads (mirrors ferro-inertia/src/manifest.rs:57)
    - eprintln! warning on second set call (visible without requiring tracing dep)
    - #[cfg(feature = "inertia")] gating on App method
    - Crate-path delegation: crate::inertia::global::set_inertia_config / get_inertia_config

key_files:
  created:
    - framework/src/inertia/global.rs
  modified:
    - framework/src/inertia/mod.rs
    - framework/src/inertia/context.rs
    - framework/src/container/mod.rs

decisions:
  - global.rs uses bare OnceLock<InertiaConfig> (not OnceLock<RwLock<T>>) — single value, set-once at bootstrap
  - App::set_inertia_config delegates to crate::inertia::global::set_inertia_config via feature-gated method
  - ferro_inertia::InertiaConfig used as parameter type in container/mod.rs (resolves through existing inertia feature dep)
  - No lib.rs change required — App already re-exported at lib.rs:71; new method rides on existing export
  - InertiaConfig import in context.rs retained — still used as parameter type in render_with_config signature
  - Fallback test placed inline in global.rs (not integration test file) — tests default fallback only, avoids OnceLock process-global contamination

metrics:
  duration_seconds: 262
  completed_date: "2026-06-21"
  tasks_completed: 2
  files_modified: 4
---

# Phase 238 Plan 03: Global InertiaConfig Plumbing Summary

Process-global `InertiaConfig` store wired end-to-end: `App::set_inertia_config(config)` writes a `OnceLock<InertiaConfig>` once at bootstrap; `Inertia::render` and `render_ctx` now read it instead of hardcoding `InertiaConfig::default()`, with a `from_env()`/`default()` fallback when nothing is set.

## What Was Built

### `framework/src/inertia/global.rs` (new, commit `57ea44cf`)

New module with a bare `OnceLock<InertiaConfig>` (no `RwLock` — set-once before server start, all reads are immutable clones):

- `set_inertia_config(config)` — writes once; emits `eprintln!` warning on second call (T-238-05 mitigation)
- `get_inertia_config()` — returns `INERTIA_CONFIG.get().cloned().unwrap_or_else(InertiaConfig::default)`
- Inline fallback test: `get_inertia_config_falls_back_to_default_when_unset` asserts `mount_id == "app"` (D-04)

### `framework/src/inertia/mod.rs` (modified, commit `57ea44cf`)

Added `pub(crate) mod global;` declaration and `pub use global::{get_inertia_config, set_inertia_config};` re-export alongside existing module decls.

### `framework/src/inertia/context.rs` (modified, commit `7e5d7044`)

Replaced both hardcoded `InertiaConfig::default()` call sites with `crate::inertia::global::get_inertia_config()`:
- Line 126 (`Inertia::render`) — call expanded to multi-line to satisfy `rustfmt` line-length
- Line 200 (`Inertia::render_ctx`)

`InertiaConfig` import retained — still used as parameter type in `render_with_config`.

### `framework/src/container/mod.rs` (modified, commit `7e5d7044`)

New associated fn on `App`:
```rust
#[cfg(feature = "inertia")]
pub fn set_inertia_config(config: ferro_inertia::InertiaConfig) {
    crate::inertia::global::set_inertia_config(config);
}
```

No `framework/src/lib.rs` change needed — `App` is already re-exported at line 71; the new method is accessible as `ferro::App::set_inertia_config(config)` without any additional export.

## Import Path Used (for Plan 04 docs reference)

`ferro_inertia::InertiaConfig` is used in `container/mod.rs` — this resolves through the existing `ferro-inertia` workspace dependency behind the `inertia` feature. The `crate::inertia::InertiaConfig` alias (re-exported at `inertia/mod.rs:28`) would also work; `ferro_inertia::` was chosen for clarity at the container layer.

## lib.rs Change Required

**None.** `App` is re-exported at `framework/src/lib.rs:71` (`pub use container::{App, Container};`). The new `set_inertia_config` method is accessible via the existing re-export. `get_inertia_config` and `set_inertia_config` free functions are also accessible via the `inertia` module re-export (line 25: `pub mod inertia;`), but the `App` method surface covers D-02.

## Verification Results

All acceptance criteria met:
- `grep -c "InertiaConfig::default()" context.rs` → 0 (both replaced)
- `grep -c "get_inertia_config" context.rs` → 2 (render + render_ctx)
- `grep -n "pub fn set_inertia_config" container/mod.rs` → line 415
- `grep -n "#[cfg(feature = \"inertia\")]" container/mod.rs` → line 414 (adjacent)
- `grep -c "RwLock" global.rs` → 0
- `cargo build -p ferro-rs --features inertia` → exit 0
- `cargo test -p ferro-rs --features inertia -- get_inertia_config_falls_back_to_default_when_unset` → 1 passed
- `cargo clippy -p ferro-rs --features inertia --all-targets -- -D warnings` → clean
- `cargo fmt --all -- --check` → clean

## Deviations from Plan

None — plan executed exactly as written. The `ferro_inertia::InertiaConfig` path resolved directly in container/mod.rs without needing the `crate::inertia::InertiaConfig` fallback (OQ from plan notes both options).

## Known Stubs

None. The global is wired end-to-end: `App::set_inertia_config` writes it, `get_inertia_config` reads it, both render sites use it. The fallback to `from_env()`/`default()` is the correct behavior for unset state (D-04), not a stub.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes. T-238-04 (OnceLock set-once documented contract) and T-238-05 (second-set warning via `eprintln!`) are both implemented as designed.

## Self-Check: PASSED
