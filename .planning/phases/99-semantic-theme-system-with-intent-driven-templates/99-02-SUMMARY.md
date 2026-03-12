---
phase: 99
plan: 02
subsystem: framework/theme
tags: [theme, middleware, resolver, json-ui, moka-cache, task-local]
requires: [99-01]
provides: [ThemeMiddleware, ThemeResolver, TenantThemeResolver, HeaderThemeResolver, DefaultResolver, current_theme, theme-css-injection]
affects: [framework/src/theme/, framework/src/json_ui/mod.rs, framework/src/lib.rs]
tech-stack:
  added: [ferro-theme optional dep, moka sync cache TTL]
  patterns: [task-local scope, resolver chain first-match, consuming builder]
key-files:
  created:
    - framework/src/theme/mod.rs
    - framework/src/theme/context.rs
    - framework/src/theme/resolver.rs
    - framework/src/theme/middleware.rs
  modified:
    - framework/Cargo.toml
    - framework/src/lib.rs
    - framework/src/json_ui/mod.rs
decisions:
  - "Arc<Theme> in task-local (not Theme directly) because Theme holds a CSS String — Arc avoids cloning large content per-request"
  - "TenantThemeResolver uses tenant.plan as theme selector for v1 — dedicated theme_name field deferred to future phase"
  - "ThemeMiddleware has no failure mode (unlike TenantMiddleware) — DefaultResolver always provides a fallback"
  - "Theme CSS injected after Tailwind CDN and custom_head but before plugin CSS so CDN processes @theme directives first"
metrics:
  duration: "~8 minutes"
  completed: "2026-03-12"
  tasks: 2
  files_modified: 7
  tests_added: 28
---

# Phase 99 Plan 02: ThemeMiddleware and JSON-UI CSS Injection Summary

ThemeMiddleware with resolver chain (TenantContext.plan, X-Theme header, default), task-local context, moka TTL cache, and inline style injection into JSON-UI head.

## What Was Built

### Task 1: Framework theme module (context, resolver trait, 3 concrete resolvers, middleware)

**framework/src/theme/context.rs** — Task-local theme storage mirroring `tenant/context.rs`:
- `tokio::task_local! { CURRENT_THEME: Arc<RwLock<Option<Arc<Theme>>>> }`
- `current_theme() -> Option<Arc<Theme>>` — reads from task-local
- `theme_scope()` and `with_theme_scope()` — pub(crate) scope helpers used by middleware

**framework/src/theme/resolver.rs** — ThemeResolver trait and 3 concrete implementations:
- `ThemeResolver` trait: `async fn resolve(&self, req: &Request) -> Option<Arc<Theme>>` (object-safe, async_trait)
- `TenantThemeResolver`: reads `current_tenant().plan` as theme name, loads from disk, caches with moka 5-min TTL 100-capacity cache
- `HeaderThemeResolver`: reads `X-Theme` header, loads from disk, same moka cache parameters
- `DefaultResolver`: always returns configured Arc<Theme>

**framework/src/theme/middleware.rs** — ThemeMiddleware:
- Consuming builder: `.resolver()` and `.default_theme()`
- Iterates resolvers in order, first Some wins; falls back to default when none match
- Stores result in task-local via `with_theme_scope`
- No failure mode — a theme is always available downstream

**framework/Cargo.toml** — `ferro-theme = { path = "../ferro-theme", version = "0.1", optional = true }` + `theme = ["dep:ferro-theme"]` feature

**framework/src/lib.rs** — Feature-gated module declaration and re-exports:
- `#[cfg(feature = "theme")] pub mod theme;`
- Re-exports: `Theme, ThemeError, ThemeTemplates, IntentModeTemplates, IntentSlotTemplate` from ferro_theme
- Re-exports: `ThemeMiddleware, ThemeResolver, TenantThemeResolver, HeaderThemeResolver, DefaultResolver, current_theme` from theme module

### Task 2: Inject theme CSS into JSON-UI head

**framework/src/json_ui/mod.rs** — CSS injection in `build_response()`:
```rust
#[cfg(feature = "theme")]
{
    if let Some(theme) = crate::theme::context::current_theme() {
        head.push_str(&format!("<style>{}</style>", theme.css));
    }
}
```
Positioned after Tailwind CDN script and custom_head, before plugin CSS assets.

## Tests

24 theme module tests + 4 json_ui theme integration tests = 28 new tests total:

- context: `current_theme_returns_none_outside_scope`, `current_theme_returns_some_within_scope`, `theme_scope_creates_arc_rwlock_initialized_to_none`, `with_theme_scope_returns_none_outside_and_some_inside`
- middleware: `new_creates_empty_instance_with_default_theme`, `resolver_adds_to_chain`, `resolves_theme_from_first_matching_resolver`, `tries_resolvers_in_order_first_some_wins`, `uses_default_theme_when_no_resolver_matches`, `current_theme_available_in_downstream_handler`, `no_resolvers_uses_default`, `default_theme_sets_custom_default`, `middleware_always_continues_request`
- resolver: `theme_resolver_is_object_safe`, `default_resolver_always_returns_default`, `default_resolver_returns_theme_for_any_request`, `header_theme_resolver_returns_some_when_header_present_and_dir_exists`, `header_theme_resolver_returns_none_when_header_absent`, `header_theme_resolver_returns_none_when_dir_does_not_exist`, `header_theme_resolver_cache_returns_on_second_resolve` (disk deletion proves cache), `tenant_theme_resolver_returns_some_when_plan_matches_dir`, `tenant_theme_resolver_returns_none_when_no_tenant`, `tenant_theme_resolver_returns_none_when_no_plan`, `tenant_theme_resolver_cache_returns_on_second_resolve` (disk deletion proves cache)
- json_ui theme_tests: `theme_css_injected_into_head_when_theme_active`, `no_theme_css_injected_when_no_middleware`, `theme_css_injected_after_tailwind_cdn`, `theme_css_does_not_duplicate_custom_head_content`

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | 7358286 | feat(99-02): add ThemeMiddleware, ThemeResolver trait, and 3 concrete resolvers |
| 2 | 2ff7b0d | feat(99-02): inject theme CSS into JSON-UI head as inline style tag |

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- framework/src/theme/mod.rs: FOUND
- framework/src/theme/context.rs: FOUND
- framework/src/theme/resolver.rs: FOUND
- framework/src/theme/middleware.rs: FOUND
- Commit 7358286: FOUND
- Commit 2ff7b0d: FOUND
- 24 theme module tests pass
- 29 json_ui tests pass (including 4 new theme injection tests)
- cargo clippy --all-targets -D warnings: PASSED
- cargo fmt check: PASSED
