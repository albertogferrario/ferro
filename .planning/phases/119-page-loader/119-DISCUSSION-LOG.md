# Phase 119: Page Loader - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-21
**Phase:** 119-page-loader
**Mode:** `--auto` (all areas auto-selected, recommended options chosen)
**Areas discussed:** API Surface, Caching, Dev-mode Hot Reload, Handler Data Merge, Layout Injection, Framework Integration

---

## API Surface

| Option | Description | Selected |
|--------|-------------|----------|
| `Spec::from_file` in `spec.rs` | Extend the existing Spec struct directly | |
| `load_spec` free function in `loader.rs` | New module, matches `global_catalog()` style | ✓ |
| `PageLoader` struct | OOP wrapper, stateful | |

**Auto-selected:** New `ferro-json-ui/src/loader.rs` module with `load_cached` free function. `Spec::from_file` may live here or as a thin wrapper in `spec.rs`. Planner chooses final placement.
**Notes:** `include_str!()` use case is handled by `Spec::from_json` + explicit validate — no new API needed.

---

## Caching

| Option | Description | Selected |
|--------|-------------|----------|
| `OnceLock<RwLock<HashMap<PathBuf, (Arc<Spec>, SystemTime)>>>` | Std-only, no new deps | ✓ |
| `DashMap` | Lock-free concurrent map, requires new dep | |
| `ferro-cache` (Redis) | Wrong scope — process-local not distributed | |

**Auto-selected:** `OnceLock<RwLock<HashMap>>` with `Arc<Spec>` values and mtime stored alongside.
**Notes:** Cache key is `fs::canonicalize(path)?` to normalize relative paths.

---

## Dev-mode Hot Reload

| Option | Description | Selected |
|--------|-------------|----------|
| Per-request mtime check (lazy invalidation) | One syscall, no new dep, no background thread | ✓ |
| `notify` crate (OS-level file watcher) | Background thread, new dependency | |
| Poll timer (background thread) | Periodic full scan, wasteful | |

**Auto-selected:** Per-request mtime check. When mtime > cached mtime: evict + reload.
**Notes:** Controlled by `!Config::is_production()` in framework integration.

---

## Handler Data Merge

| Option | Description | Selected |
|--------|-------------|----------|
| Shallow top-level merge, handler wins | Simple, matches `data_path` top-level key convention | ✓ |
| Deep recursive merge | Complex, ambiguous on nested overlap | |
| Full replace (handler data replaces spec.data) | Loses spec defaults | |

**Auto-selected:** `Spec::merge_data(self, Value) -> Self` — shallow top-level merge.
**Notes:** Non-Object handler_data is ignored with debug assertion.

---

## Layout Injection

| Option | Description | Selected |
|--------|-------------|----------|
| Existing layout registry (auto from `spec.layout`) | No new API, matches current architecture | ✓ |
| Per-request `DashboardLayoutConfig` parameter | Dynamic sidebar/header per request | |
| `"$layout"` magic key in `spec.data` | Magic key approach, fragile | |

**Auto-selected:** Existing registry. When `spec.layout = "dashboard"`, `DashboardLayout` is used automatically.
**Notes:** Per-request user data (name, notifications) goes in `spec.data` via `merge_data`; components use `$data`/`$template`.

---

## Framework Integration

| Option | Description | Selected |
|--------|-------------|----------|
| `JsonUi::render_file(path, handler_data, config)` | One-stop entry, mirrors `JsonUi::render` | ✓ |
| Standalone `render_file` free function | Doesn't fit `JsonUi` API shape | |
| Middleware-level interception | Too implicit | |

**Auto-selected:** `JsonUi::render_file` in `framework/src/json_ui/mod.rs`.
**Notes:** Full pipeline: load (cached) → merge_data → resolve_actions → resolve_expressions → validate → render.

---

## Claude's Discretion

- `Arc<Spec>` vs clone per call — prefer `Arc`
- `load_cached` as free function vs `PageLoader` struct — free function
- Whether post-resolution catalog validate (step 4 in D-05) is included — planner decides based on overhead
- Whether `Spec::from_file_content` helper is added for `include_str!()` — only if tests need it

## Deferred Ideas

- Per-request dynamic layout data (sidebar/header from handler)
- `notify` crate for background file watching
- Spec cache with production TTL
- `Spec::from_url` for remote spec loading
