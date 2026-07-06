# Phase 119: Page Loader - Context

**Gathered:** 2026-04-21
**Status:** Ready for planning
**Mode:** `--auto` — decisions auto-selected from codebase analysis and upstream phase context. Downstream agents may override any auto-choice by editing this file before `/gsd-plan-phase`.

<domain>
## Phase Boundary

Add a framework-level file-loading pipeline for JSON spec files. The phase ships:

- `Spec::from_file(path)` — runtime file load, structural parse, and catalog validation in one call
- A global spec cache (per-process, keyed by canonical path) with dev-mode mtime invalidation
- Handler data merge via `Spec::merge_data(handler_data: Value) -> Self`
- Layout auto-injection: when a spec declares `"layout": "dashboard"`, the globally-registered `DashboardLayout` is used automatically — no handler code required
- `JsonUi::render_file` on the framework side — one-stop entry that loads (cache-aware), merges handler data, runs the full resolve→validate→render pipeline

**What this phase does NOT do:**
- Add new JSON-UI components (Phase 116 is done)
- Change `Spec`/`Element` shape (Phase 115 is locked)
- Update CLI/MCP generation tools to emit file-backed specs (Phase 120)
- Convert gestiscilo pages (Phase 121)
- Introduce a second expression syntax or extend `$data`/`$template` (Phase 118 D-07, hard cap)
- Replace the per-request `Spec::from_json` path — both coexist; `from_file` is additive

</domain>

<decisions>
## Implementation Decisions

### D-01: `Spec::from_file` is the load-time entry point

**Decision:** Add `Spec::from_file(path: impl AsRef<Path>) -> Result<Spec, LoadError>` to `ferro-json-ui/src/spec.rs` (or a new `ferro-json-ui/src/loader.rs`). It reads the file, calls `Spec::from_json`, then calls `global_catalog().validate(&spec)`. Errors are wrapped in a new `LoadError` enum:

```rust
pub enum LoadError {
    Io(std::io::Error),
    Parse(SpecError),
    Catalog(Vec<CatalogError>),
}
```

Fails fast on any error — no partial specs returned.

**Why:** Phase 118 D-08 explicitly states that Phase 119 "hard-wires" `Catalog::validate` at load time. `Spec::from_json` does structural validation only; catalog validation is the load-time step. Combining them in one function gives a clean entry point and matches success criteria #1 and #2.

**How to apply:** `Spec::from_file` is the uncached variant. The cache wrapper (D-02) calls this internally. Authors using `include_str!()` call `Spec::from_json(include_str!("page.json"))` followed by `global_catalog().validate()` — or use `Spec::from_file_content(json: &str)` as a helper that does both steps without I/O.

### D-02: Global spec cache keyed by canonical path

**Decision:** A global `OnceLock<RwLock<HashMap<PathBuf, (Spec, SystemTime)>>>` in `ferro-json-ui/src/loader.rs` (new module). Value tuple is `(Arc<Spec>, mtime)` for cheap clone. The cache is initialized on first access.

In production (`!cfg(debug_assertions)` or `is_dev: bool` flag — see D-05): entries are never evicted after first load. In dev: mtime is checked on each access (D-03).

**Why:** Success criteria #5 requires "compiled once, reused across requests." A process-level static is the correct scope. `ferro-cache` is Redis-backed and wrong for in-process computed objects. `DashMap` is not in the dependency tree; `RwLock<HashMap>` is sufficient since write contention only occurs on first-load or hot-reload eviction.

**How to apply:** Planner should use `std::sync::OnceLock` + `std::sync::RwLock` from std. Cache key is `fs::canonicalize(path)?` to normalize relative paths. The planner may choose `Arc<Spec>` or plain `Spec` — `Arc` is preferred to avoid cloning the entire spec per request.

### D-03: Dev-mode hot reload via per-request mtime check

**Decision:** When dev mode is active, the cache loader does:
1. `fs::metadata(path).modified()` — get current mtime
2. Compare to cached mtime
3. If current mtime > cached mtime: evict the entry, reload via `Spec::from_file`, re-insert with new mtime

No background thread. No `notify` crate dependency. Invalidation is lazy — triggered on the next request that accesses the stale path.

**Why:** Success criteria #6 requires hot reload without recompilation. A per-request stat check is one syscall (sub-microsecond on modern OS). Background threads add complexity and require shutdown coordination. The `notify` crate would be a new dependency with OS-specific backends — overkill for a dev-mode convenience feature. Lazy invalidation is fine for development because the next page load immediately reflects changes.

**How to apply:** The `is_dev` flag is passed into the cache loader by the framework integration (D-05). In `ferro-json-ui` (standalone, no framework dep), the cache function accepts `reload_if_changed: bool`. Planner may also key off `cfg(debug_assertions)` but a runtime flag is preferred for consistency with `Config::is_production()`.

### D-04: Handler data merge — shallow top-level, handler wins

**Decision:** Add `Spec::merge_data(mut self, handler_data: Value) -> Self`. If `handler_data` is a `Value::Object`, its keys are inserted into `self.data.as_object_mut()` (overwriting matching keys). If `handler_data` is not an Object, it is ignored (debug-assert, no panic in production). Returns `self` for chaining.

**Why:** Success criteria #3: "Handler data merges into `spec.data` (handler data takes precedence over spec defaults)." Shallow merge at the top level is the standard pattern. Deep merge adds complexity and creates ambiguity when spec defaults and handler data have nested overlap. The projection system (`data_path` convention) uses top-level keys (`/user/name` starts from `spec.data["user"]`) so top-level merging is the natural boundary.

**How to apply:** `Spec::merge_data` is a consuming builder method (`mut self` → `Self`), consistent with `Spec::builder()` pattern. Framework integration (`JsonUi::render_file`) calls it automatically.

### D-05: Framework integration — `JsonUi::render_file`

**Decision:** Add `JsonUi::render_file(path: impl AsRef<Path>, handler_data: Value, config: &JsonUiConfig) -> Result<HttpResponse, HttpResponse>` to `framework/src/json_ui/mod.rs`. This function:

1. Loads spec from cache (dev mode uses `!Config::is_production()`)
2. Calls `spec.merge_data(handler_data)`
3. Calls `JsonUi::resolve` (resolve_actions + resolve_expressions per Phase 118 D-08)
4. Calls `global_catalog().validate()` (confirm post-resolution spec is valid)
5. Calls `render_spec_to_html_with_plugins` with the registered layout

Step 4 is a second validate after resolution. This catches cases where expressions resolve to wrong types. Planner may skip step 4 if the double-validate cost is notable — the load-time validate (D-01) already covers static spec correctness; resolution-time validate is a defense-in-depth step.

**Why:** One-stop entry point keeps handler code minimal: `JsonUi::render_file(req, "views/dashboard.json", spec.data)`. Follows the existing `JsonUi::render` / `JsonUi::render_with_config` API shape. Framework owns dev/prod detection via `Config::is_production()`.

**How to apply:** Planner should add a `render_file` variant to the existing `JsonUi` impl block in `framework/src/json_ui/mod.rs`. Signature mirrors `JsonUi::render` but takes a path instead of a `Spec`. For the `Request`-aware variant (which may need the current user for layout), planner may add `JsonUi::render_file_with_req` or make layout data part of `handler_data`. See D-06.

### D-06: Layout auto-injection is via the existing registry

**Decision:** When a spec declares `"layout": "dashboard"`, the existing layout registry (set up at app startup via `register_layout("dashboard", DashboardLayout::new(config))`) is used automatically. No additional injection API is required. The page loader does NOT need to pass per-request sidebar/header data to the layout — that data is already in the globally registered `DashboardLayout::config`.

Per-request user context (e.g., logged-in user's name, notification count) goes in `spec.data` via `merge_data`. Components that display user info use `$data` / `$template` expressions. The sidebar nav and header shell remain static (configured at startup) as they are today.

**Why:** The `DashboardLayoutConfig` is already registered as a static at startup. The `LayoutContext` passed to `Layout::render` contains `view_json` and `data_json` — components can pull per-request data via expressions. Adding a second per-request layout injection mechanism would duplicate the concern and break the "register once at startup" model that keeps layouts framework-decoupled. Success criteria #4 ("Layout data (sidebar, header, sse_url) injects automatically") means the layout registry lookup happens automatically when `spec.layout` is set — not that a new injection API is needed.

**How to apply:** No new layout API. The planner should verify that `render_spec_to_html_with_plugins` correctly passes the layout name from `spec.layout` to the registry lookup, and that `render_file` does not bypass this path.

### D-07: Module layout

**Decision:**

New files in `ferro-json-ui`:
- `ferro-json-ui/src/loader.rs` — `LoadError`, `Spec::from_file` (or free function `load_spec(path)`), global cache, `load_cached(path, reload_if_changed: bool) -> Result<Arc<Spec>, LoadError>`, mtime-based invalidation.

Modified files:
- `ferro-json-ui/src/spec.rs` — add `Spec::merge_data(self, Value) -> Self`
- `ferro-json-ui/src/lib.rs` — `pub mod loader;` + re-export `LoadError`, `load_cached`
- `framework/src/json_ui/mod.rs` — add `JsonUi::render_file`

No other files touch.

**Why:** Single-responsibility: loader.rs owns I/O + cache; spec.rs owns data merging (pure, no I/O); framework mod owns the HTTP response pipeline. Consistent with existing `action.rs`, `expression.rs`, `resolve.rs` file-per-feature pattern.

### D-08: Error type for load failures

**Decision:** `LoadError` (defined in `ferro-json-ui/src/loader.rs`) has three variants:

```rust
pub enum LoadError {
    Io(std::io::Error),
    Parse(SpecError),
    Catalog(Vec<CatalogError>),
}
```

Framework integration converts `LoadError` to an `HttpResponse` (500 Internal Server Error with a descriptive body in dev, generic 500 in production). `thiserror` derive on `LoadError`.

**Why:** Structured variants let callers distinguish I/O failures (path wrong) from parse failures (invalid JSON) from schema failures (spec violates catalog contract). Matching the `thiserror` pattern already used by `SpecError` and `CatalogError`.

### D-09: Test surface

**Decision:**

Unit tests in `ferro-json-ui/src/loader.rs`:
- `load_spec` from valid JSON file → success
- `load_spec` from invalid JSON → `LoadError::Parse`
- `load_spec` from spec with unknown component → `LoadError::Catalog`
- `load_spec` from missing file → `LoadError::Io`
- Cache hit: second call returns same Arc (no re-parse)
- Dev-mode invalidation: after file mtime advances, cache miss triggers reload

Unit tests in `ferro-json-ui/src/spec.rs` (merge_data):
- Handler data keys override spec.data keys
- Non-Object handler data is ignored (no panic)
- Empty handler data leaves spec.data unchanged

Integration tests in `framework/src/json_ui/mod.rs`:
- `JsonUi::render_file` produces HTML with expressions resolved from merged handler data
- Layout is applied when `spec.layout` matches a registered layout

**Why:** Unit tests prove the cache contract; integration tests prove the full pipeline ordering from D-05 actually holds.

### Claude's Discretion
- Whether the cache uses `Arc<Spec>` or clones `Spec` per call — prefer `Arc` if `Spec` is large
- Whether `load_cached` is a free function or a method on a `PageLoader` struct — free function preferred (matches the existing `global_catalog()` / `register_layout()` API style)
- Whether `JsonUi::render_file` returns `Result<HttpResponse, HttpResponse>` or a new `RenderFileError` type — match the existing `JsonUi::render` return type
- Whether step 4 (post-resolution catalog validate) in `render_file` is included or deferred — the planner may omit if it adds measurable overhead; D-01's load-time validate already covers static correctness
- Whether `Spec::from_file_content(json: &str)` helper is added for the `include_str!()` case — add only if integration tests or Phase 121 field test actually need it

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase goal and success criteria
- `.planning/ROADMAP.md` §"Phase 119: Page Loader" — goal, depends-on (Phase 118), Requirements (LOAD-01, LOAD-02, LOAD-03), 6 success criteria
- `.planning/ROADMAP.md` §"v12.0 JSON-UI v2 — Spec-Driven Rendering" — overall milestone context

### Locked upstream decisions (do not re-open)
- `.planning/phases/115-spec-v2-data-structures/115-CONTEXT.md` — `Spec`/`Element` shape, `Spec::from_json`, `SpecError`, parse-time structural validation. **Shape is frozen.**
- `.planning/phases/116-flat-element-renderer/116-CONTEXT.md` — `render_spec_to_html_with_plugins`, walker shape, infallible renderer posture, layout registry lookup path.
- `.planning/phases/117-catalog-and-json-schema/117-CONTEXT.md` — `global_catalog()`, `Catalog::validate`, `CatalogError` variants, compiled validator singleton.
- `.planning/phases/118-server-side-expressions/118-CONTEXT.md` — pipeline order (D-08): parse → resolve_actions → resolve_expressions → Catalog::validate → render. D-08 explicitly assigns "hard-wiring Catalog::validate at load time" to Phase 119.

### Framework integration points
- `framework/src/json_ui/mod.rs` — `JsonUi::render`, `JsonUi::resolve`, `JsonUi::render_with_config`; `render_file` is added here
- `framework/src/config/mod.rs` — `Config::is_production()` for dev/prod gate
- `ferro-json-ui/src/layout.rs` — `DashboardLayout`, `DashboardLayoutConfig`, `LayoutContext`, `Layout` trait, `register_layout`
- `ferro-json-ui/src/catalog.rs` — `global_catalog()`, `Catalog::validate`, `CatalogError`
- `ferro-json-ui/src/spec.rs` — `Spec::from_json`, `SpecError`, `Spec::builder()`
- `ferro-json-ui/src/resolve.rs` — `resolve_actions`, `resolve_errors` (pattern reference for `merge_data` API shape)
- `ferro-json-ui/src/expression.rs` — `resolve_expressions` (Phase 118, shipped)

### Downstream constraints
- `.planning/ROADMAP.md` §"Phase 120: CLI & MCP Updates" — generators will produce file-backed spec invocations; `render_file` API must be ergonomic for generated handler code
- `.planning/ROADMAP.md` §"Phase 121: Documentation & Field Test" — gestiscilo conversion will use `render_file` on a real page; API must be production-safe

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-json-ui/src/spec.rs::Spec::from_json` — structural parse + validation; `from_file` calls this internally
- `ferro-json-ui/src/catalog.rs::global_catalog()` — `OnceLock`-backed compiled catalog; `Catalog::validate` is the Phase 119 load-time validator
- `ferro-json-ui/src/resolve.rs::resolve_actions` — signature pattern (`fn resolve_actions(spec: &mut Spec, resolver: &dyn UrlResolver)`) for `merge_data` shape
- `ferro-json-ui/src/expression.rs::resolve_expressions` — called in `JsonUi::resolve`; Phase 119's `render_file` inherits this via `JsonUi::resolve`
- `framework/src/json_ui/mod.rs::JsonUi::resolve` — already chains resolve_actions + resolve_expressions; `render_file` extends this
- `ferro-json-ui/src/layout.rs::register_layout` + `LayoutRegistry` — existing registry used by `render_file` automatically

### Established Patterns
- `OnceLock` for global singletons: `global_catalog()` and `global_plugin_registry()` both use this pattern
- Builder methods with `mut self → Self`: `SpecBuilder`, `Spec::builder()` use consuming builder style — `merge_data` follows this
- `thiserror` derive for all crate error enums: `SpecError`, `CatalogError` — `LoadError` follows this
- `pub(crate)` helpers: `data::resolve_path` is `pub(crate)` — `loader.rs` internal helpers follow same visibility
- File-per-feature: `action.rs`, `expression.rs`, `resolve.rs`, `visibility.rs` — `loader.rs` follows this

### Integration Points
- `ferro-json-ui/src/lib.rs` — add `pub mod loader;` and re-export `LoadError`, `load_cached`
- `framework/src/json_ui/mod.rs` — add `JsonUi::render_file` in the existing `JsonUi` impl block
- `ferro-json-ui/Cargo.toml` — no new dependencies needed (std `OnceLock`, `RwLock`, `HashMap`, `fs` are all stdlib)

</code_context>

<specifics>
## Specific Ideas

- The `include_str!()` alternative mentioned in the roadmap success criteria is not a new API — it's `Spec::from_json(include_str!("page.json"))` plus `global_catalog().validate()`. No special support needed; document this pattern.
- Dev-mode mtime check is one `fs::metadata` syscall — negligible cost. No feature flag needed to enable/disable it; it's always off in production.
- The `$layout` magic key approach (considered for per-request layout data) was rejected in favor of the existing static registry model (D-06). Do not introduce a `"$layout"` key in `spec.data`.

</specifics>

<deferred>
## Deferred Ideas

- Per-request layout data injection (dynamic sidebar/header from handler) — would require a new `LayoutContext` field or a `DynamicLayout` trait extension. Out of scope for Phase 119; layout data is static (startup-configured). Revisit if gestiscilo field test (Phase 121) surfaces a need.
- Background file watcher via the `notify` crate — no new dependency in Phase 119; per-request mtime is sufficient for dev. Revisit if hot-reload latency (requiring a page refresh to pick up changes) becomes a DX complaint.
- `Spec::from_url` (remote spec loading) — future capability, not in v12.0 scope.
- Spec cache with TTL-based invalidation in production — current design is "never evict in production"; TTL could help if specs are generated at deploy time. Deferred to post-v1.0.

</deferred>

---

*Phase: 119-page-loader*
*Context gathered: 2026-04-21*
