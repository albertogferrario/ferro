# Phase 238: Inertia First-Load HTML Shell — Research

**Researched:** 2026-06-21
**Domain:** ferro-inertia config plumbing, HTML template extension, process-global pattern, Vite proxy, documentation
**Confidence:** HIGH — all claims verified against live source files with line numbers

## Summary

This phase is a wiring + surfacing + docs + hardening task. The HTML shell substrate is
fully implemented in `ferro-inertia/src/response.rs`. The confirmed gaps are: (1) the
common render path hardcodes `InertiaConfig::default()` instead of reading an app-level
configured value; (2) `App::set_inertia_config()` and `InertiaConfig::from_env()` are
documented in `docs/src/features/inertia.md:43-44` but do not exist; (3) the default
template is not configurable for title, `<head>` extras, or mount-node id; (4) the docs
have a stale struct-literal example and no same-origin or Vite proxy recipe; (5) no
end-to-end test proves content negotiation or both asset modes.

The framework's established global-config pattern is `OnceLock<RwLock<T>>` in a static,
readable via a typed `get()`/`register()` API in `framework/src/config/repository.rs`. A
separate, lighter-weight precedent for "set once at bootstrap, read by all request
handlers" is the middleware registry: `OnceLock<RwLock<Vec<…>>>` in
`framework/src/middleware/registry.rs:10`. Both are async-safe. The manifest OnceLock in
`ferro-inertia/src/manifest.rs:57` uses a path-unkeyed single global; this is a
test-isolation risk that the planner must assess.

**Primary recommendation:** Implement `InertiaConfig::from_env()` (move env-reading out
of `Default::default()`), add a `static INERTIA_CONFIG: OnceLock<InertiaConfig>` in
`framework/src/inertia/context.rs` (or a dedicated `framework/src/inertia/global.rs`),
expose `App::set_inertia_config(config)` that writes it once, and make `render` /
`render_ctx` call `get_inertia_config()` instead of `InertiaConfig::default()`. Extend
`InertiaConfig` with `title: Option<String>`, `head_extras: Option<String>`, and
`mount_id: String` (default `"app"`). Do all of this without touching `ferro-inertia`'s
dependency list (it must remain a pure leaf with only `serde` + `serde_json`).

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- D-01: Keep a single content-negotiated `render` — do NOT add a separate `render_document()` method.
- D-02: Introduce a process-global app-level InertiaConfig (OnceLock-style) so `framework::Inertia::render` / `render_ctx` resolve it instead of hardcoding `default()`.
- D-03: `InertiaConfig::from_env()` is an explicit constructor reading `APP_NAME`, `APP_URL`/`VITE_DEV_SERVER`, `VITE_ENTRY_POINT`, `INERTIA_VERSION`, `APP_ENV` — mirroring the framework `from_env()` convention.
- D-04: When no config is set via `set_inertia_config`, the render path falls back to `from_env()`/`default()` so existing apps keep working with zero changes.
- D-05: Extend `InertiaConfig` with structured fields the default template honors: title, `head_extras`, and a configurable mount node id (default `"app"`).
- D-06: Keep the existing `html_template` string-replace escape hatch. No templating engine.
- D-07: Preserve current `data-page` HTML-attribute escaping (`&`, `<`, `>`, `"`, `'`).
- D-08: Keep Vite-manifest resolution inside `ferro-inertia` (`manifest.rs`). Do NOT add a `ferro-assets` dependency.
- D-09: Confirm the manifest OnceLock cache does not break tests/multi-config use (planner assesses whether to key on path).
- D-10: Document both the same-origin story (primary) and a Vite `server.proxy` recipe for split-port dev.
- D-11: Fix doc drift in `docs/src/features/inertia.md` (stale struct literal at :53-59, nonexistent APIs at :43-44). Update ferro-mcp `generation_context` if it surfaces Inertia bootstrap.
- D-12: Add an end-to-end test proving content negotiation + both asset modes.

### Claude's Discretion
- Exact field naming/shape on `InertiaConfig` (title vs app_name collapse, `head_extras` type).
- Whether the global config store keys the manifest cache (D-09 assessment).
- Whether `from_env()` reads `APP_URL` vs `VITE_DEV_SERVER` (or both) for the dev-server URL.

### Deferred Ideas (OUT OF SCOPE)
- True SSR (executing JS bundle on the server).
- A shared `ferro-assets` Vite-manifest resolver consumed by multiple crates.
</user_constraints>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Content negotiation (HTML vs JSON) | `ferro-inertia` (library) | `framework::Inertia` (wrapper) | Logic lives in `response.rs:render_internal`; framework just calls it |
| Global config storage | `framework::inertia` | — | App-level concerns belong in the framework layer; `ferro-inertia` is framework-agnostic |
| `InertiaConfig::from_env()` | `ferro-inertia` | — | Config struct owns its construction; CLAUDE.md "project-agnostic crates" rule |
| `App::set_inertia_config()` | `framework` (`container` or `inertia` module) | — | `App` struct is in `framework/src/container/mod.rs:206` |
| HTML template extension (title, head_extras, mount_id) | `ferro-inertia/src/config.rs` + `response.rs` | — | Fields on `InertiaConfig`, consumed by `to_html_response` |
| Vite manifest caching | `ferro-inertia/src/manifest.rs` | — | Must stay leaf-crate local (D-08) |
| Docs | `docs/src/features/inertia.md` | `ferro-mcp generation_context` | Both surfaces need update |
| End-to-end tests | `ferro-inertia/src/response.rs` (unit) | `framework/tests/` (integration) | Unit for content negotiation, integration optional |

---

## Existing Implementation — Confirmed Code Paths

All line numbers verified against the live tree on 2026-06-21.

### Content Negotiation (`ferro-inertia/src/response.rs`) [VERIFIED: direct read]

| Symbol | Location | What it does |
|--------|----------|--------------|
| `Inertia::render_internal` | `response.rs:237` | Dispatches to `to_json_response()` or `to_html_response()` based on `req.is_inertia()` (`:293-297`) |
| `InertiaResponse::to_html_response` | `response.rs:374` | Emits full `<!DOCTYPE html>` with `data-page` attribute |
| Dev-mode template branch | `response.rs:402-432` | Emits `@react-refresh` preamble + `@vite/client` + entry module `<script>` tags |
| Prod-mode template branch | `response.rs:433-461` | Calls `resolve_assets()`, emits hashed `<script>` + `<link>` tags |
| `data-page` escaping | `response.rs:384-389` | `&` → `&amp;`, `<` → `&lt;`, `>` → `&gt;`, `"` → `&quot;`, `'` → `&#x27;` (D-07 satisfied) |
| Custom-template path | `response.rs:394-399` | `html_template.replace("{page}", …).replace("{csrf}", …)` |
| `InertiaResponse::to_json_response` | `response.rs:362` | Builds `{"component","props","url","version"}` JSON object |

Both `to_json_response` and `to_html_response` build the page object from the same four
fields (`component`, `props`, `url`, `config.version`) — the embedded `data-page` JSON is
byte-for-byte the same object as the XHR JSON response before HTML escaping. Success
Criterion 1 (equality) is architecturally guaranteed by the single `InertiaResponse`
builder.

### Config (`ferro-inertia/src/config.rs`) [VERIFIED: direct read]

| Field | Type | Default source |
|-------|------|----------------|
| `app_name` | `String` | `APP_NAME` env or `"Ferro"` (`config.rs:51`) |
| `vite_dev_server` | `String` | `VITE_DEV_SERVER` env or `"http://localhost:5173"` (`config.rs:43`) |
| `entry_point` | `String` | `"src/main.tsx"` hardcoded (`config.rs:56`) |
| `version` | `String` | `"1.0"` hardcoded (`config.rs:57`) |
| `development` | `bool` | `APP_ENV != production/staging` (`config.rs:46`) |
| `html_template` | `Option<String>` | `None` |
| `manifest_path` | `String` | `"public/.vite/manifest.json"` hardcoded (`config.rs:60`) |

**Missing fields** (D-05): `title: Option<String>`, `head_extras: Option<String>`,
`mount_id: String`.

**Missing constructor** (D-03): `from_env()` does not exist — env-reading is embedded in
`Default::default()` at `config.rs:41-63`. The fix is to extract the body into
`from_env()` and have `default()` delegate to it.

### Framework Wrapper Gap (`framework/src/inertia/context.rs`) [VERIFIED: direct read]

| Call site | Location | Problem |
|-----------|----------|---------|
| `Inertia::render` | `context.rs:126` | `Self::render_with_config(req, component, props, InertiaConfig::default())` — hardcodes `default()` |
| `Inertia::render_ctx` | `context.rs:200` | `ferro_inertia::Inertia::render_with_options(ctx, component, props, Some(&shared), InertiaConfig::default())` — hardcodes `default()` |

The fix for both call sites is to replace `InertiaConfig::default()` with a
`get_inertia_config()` call that reads the process global, falling back to
`InertiaConfig::default()` when nothing is registered.

---

## Global Config Pattern — Established Framework Convention

### Pattern: `OnceLock<RwLock<T>>` typed repository [VERIFIED: direct read]

The canonical multi-type global config store is `framework/src/config/repository.rs`:

```
static CONFIG_REPOSITORY: OnceLock<RwLock<ConfigRepository>> = OnceLock::new();
// register(T) — inserts by TypeId
// get::<T>() — returns Clone
```

`Config::init()` calls `repository::register(AppConfig::from_env())` at startup.
`AppConfig::from_env()` reads `APP_NAME`, `APP_URL`, `APP_ENV`, `APP_DEBUG` (`providers/app.rs:21-33`).

### Pattern: `OnceLock<RwLock<Vec<T>>>` registry (lighter) [VERIFIED: direct read]

The middleware registry (`middleware/registry.rs:10`) uses a static
`OnceLock<RwLock<Vec<BoxedMiddleware>>>` with a `register_global_middleware(M)` free
function. This is the "set multiple times at bootstrap, read once at server build" variant.

### Recommended pattern for `InertiaConfig` global

Since `InertiaConfig` is a single struct (not a collection), and the write must happen
once before any request is served, the lightest correct implementation is:

```rust
// in framework/src/inertia/global.rs  (new file)
use ferro_inertia::InertiaConfig;
use std::sync::OnceLock;

static INERTIA_CONFIG: OnceLock<InertiaConfig> = OnceLock::new();

/// Set the process-global InertiaConfig. Call once from bootstrap.
/// Subsequent calls are silently ignored (OnceLock semantics).
pub fn set_inertia_config(config: InertiaConfig) {
    let _ = INERTIA_CONFIG.set(config);
}

/// Get the active InertiaConfig, falling back to from_env()/default().
pub fn get_inertia_config() -> InertiaConfig {
    INERTIA_CONFIG
        .get()
        .cloned()
        .unwrap_or_else(InertiaConfig::default)
}
```

`App::set_inertia_config(config)` in `framework/src/container/mod.rs` delegates to
`crate::inertia::global::set_inertia_config(config)`. This keeps the DI container clean
(InertiaConfig is not a service, it is a config) and matches how `AppConfig` is stored
separately from the container.

The `OnceLock<InertiaConfig>` (no `RwLock`) is sufficient because:
- Set once at bootstrap (before tokio runtime starts accepting requests).
- All subsequent reads are immutable.
- `Clone` on `InertiaConfig` is derived, so `cloned()` is cheap.

**Test isolation:** In tests, `OnceLock::set()` is idempotent after the first call. The
manifest cache has the same property. Tests that need to exercise multiple configs must
either use distinct manifest files on disk or call `resolve_assets()` only with the same
path per process. This is not worse than the current state (manifest is already
single-global). The planner should note this and ensure the D-12 tests do not fight each
other over the manifest OnceLock — use a tempfile-per-test as manifest.rs's own tests do
(`manifest.rs:130-143`).

---

## Standard Stack

### Core (all already in scope — no new dependencies)

| Crate | Purpose | Status |
|-------|---------|--------|
| `ferro-inertia` | Leaf crate: config, manifest, response | Extend existing fields + add `from_env()` |
| `framework/src/inertia/` | Framework wrapper + new global module | Add `global.rs`, update `context.rs` |
| `std::sync::OnceLock` | Process-global store for InertiaConfig | Already used in `manifest.rs:57`, `middleware/registry.rs:10`, `metrics/mod.rs:93` |
| `tempfile` | Test-only: temp manifest.json in tests | Already in `ferro-inertia/dev-dependencies` (`Cargo.toml:17`) |

No new crate dependencies. `ferro-inertia/Cargo.toml` must remain `serde` + `serde_json`
only. [VERIFIED: `ferro-inertia/Cargo.toml:13-14`]

---

## Architecture Patterns

### Pattern 1: `InertiaConfig::from_env()` extraction

**What:** Move the env-reading logic from `Default::default()` into an explicit
`from_env()` constructor; have `default()` call `from_env()`.

**Current state:** `Default::default()` at `config.rs:41-63` reads env vars inline.

**Change:**
```rust
// ferro-inertia/src/config.rs
impl InertiaConfig {
    pub fn from_env() -> Self {
        let vite_dev_server = std::env::var("VITE_DEV_SERVER")
            .unwrap_or_else(|_| "http://localhost:5173".to_string());
        let is_dev = !matches!(
            std::env::var("APP_ENV").ok().as_deref(),
            Some("production") | Some("staging")
        );
        let app_name = std::env::var("APP_NAME").unwrap_or_else(|_| "Ferro".to_string());
        // entry_point: VITE_ENTRY_POINT env or "src/main.tsx"
        // version: INERTIA_VERSION env or "1.0"
        // manifest_path: keep hardcoded default (no env needed yet)
        Self {
            app_name,
            vite_dev_server,
            entry_point: std::env::var("VITE_ENTRY_POINT")
                .unwrap_or_else(|_| "src/main.tsx".to_string()),
            version: std::env::var("INERTIA_VERSION")
                .unwrap_or_else(|_| "1.0".to_string()),
            development: is_dev,
            html_template: None,
            manifest_path: "public/.vite/manifest.json".to_string(),
            // New fields:
            title: None,
            head_extras: None,
            mount_id: "app".to_string(),
        }
    }
}

impl Default for InertiaConfig {
    fn default() -> Self {
        Self::from_env()
    }
}
```

The CONTEXT.md says D-03 should also read `APP_URL` — clarification: `APP_URL` is the
app base URL used for constructing absolute URLs, not the Vite dev server URL. The current
`Default` does not read `APP_URL`. Claude's discretion (from CONTEXT.md §Discretion):
`from_env()` may read `APP_URL` vs `VITE_DEV_SERVER` or both. **Recommendation:**
`VITE_DEV_SERVER` is the correct env for the dev-server base URL (already used); `APP_URL`
is the backend base URL and is already in `AppConfig::from_env()`. Do not read `APP_URL`
into `InertiaConfig::from_env()` — these are separate concerns.

### Pattern 2: New fields on `InertiaConfig` (D-05)

Add to `ferro-inertia/src/config.rs` struct body:

```rust
/// Optional page title override. When Some, overrides app_name in <title>.
pub title: Option<String>,
/// Raw HTML injected into <head> before </head> (meta, favicon, font, etc.).
/// Ignored when html_template is set (custom template owns <head>).
pub head_extras: Option<String>,
/// id of the mount node. Defaults to "app".
pub mount_id: String,
```

Add builder methods:
```rust
pub fn title(mut self, t: impl Into<String>) -> Self { self.title = Some(t.into()); self }
pub fn head_extras(mut self, h: impl Into<String>) -> Self { self.head_extras = Some(h.into()); self }
pub fn mount_id(mut self, id: impl Into<String>) -> Self { self.mount_id = id.into(); self }
```

**Field shape decision (Claude's discretion):** `head_extras: Option<String>` (raw HTML
string) is simpler than `Vec<String>` — consumers concatenate multiple tags if needed.
`title: Option<String>` is separate from `app_name` so that `app_name` retains its doc
comment ("used in title tag") while `title` provides an explicit override. When `title`
is `Some`, use it; otherwise fall back to `app_name`.

### Pattern 3: Inject new fields in `to_html_response` (D-05/D-06)

**Current dev-mode template** (`response.rs:403-432`):
- `<title>{}</title>` uses `self.config.app_name` (format arg)
- `<div id="app" data-page="{}">` hardcodes `"app"`

**Change:** Replace `app_name` reference with `title_text` (derived from
`config.title.as_deref().unwrap_or(&config.app_name)`), replace `"app"` with
`config.mount_id`, and inject `config.head_extras.as_deref().unwrap_or("")` before
`</head>`.

Same for the prod-mode template branch (`response.rs:443-461`).

The custom-template path (`response.rs:394-399`) must remain unchanged — when
`html_template` is `Some`, `head_extras`/`mount_id`/`title` are irrelevant (the template
owns everything). [VERIFIED: `response.rs:394`]

### Pattern 4: Process-global in framework `context.rs`

**Current call sites:**
- `context.rs:126` — `Inertia::render` calls `Self::render_with_config(req, component, props, InertiaConfig::default())`
- `context.rs:200` — `Inertia::render_ctx` calls `…InertiaConfig::default()`

**Change:** Both call `crate::inertia::global::get_inertia_config()` instead.

### Pattern 5: `App::set_inertia_config` exposure

`App` is defined in `framework/src/container/mod.rs:206`. Add:

```rust
/// Set the process-global Inertia configuration.
/// Call once from bootstrap.rs before the server starts.
#[cfg(feature = "inertia")]
pub fn set_inertia_config(config: ferro_inertia::InertiaConfig) {
    crate::inertia::global::set_inertia_config(config);
}
```

Re-export from `framework/src/lib.rs` (where `App` is already re-exported at `:71`). No
additional re-export needed — users call `ferro::App::set_inertia_config(config)` via the
existing `App` re-export.

### Anti-Patterns to Avoid

- **Thread-local state:** The deprecated `InertiaContext` at `context.rs:313` was
  thread-local and async-unsafe. The new global must use `OnceLock` (not `thread_local!`).
  [VERIFIED: `context.rs:313-318`]
- **Multiple write surfaces:** Do not add `InertiaConfig` to the `Config` typed repository
  AND a dedicated `OnceLock`. Use only one. Recommendation: dedicated `OnceLock` in
  `framework/src/inertia/global.rs` — lighter than the full typed repository and clearer
  in intent.
- **Blocking in `get_or_init`:** The `OnceLock::get_or_init(|| from_env())` fallback
  pattern is safe here because `from_env()` only reads env vars (no async, no I/O risk).

---

## Manifest OnceLock — Test Isolation Assessment (D-09)

**Current state:** `manifest.rs:57` — `static MANIFEST: OnceLock<Option<ViteManifest>> = OnceLock::new()`

`resolve_assets(manifest_path, entry_point)` calls `MANIFEST.get_or_init(|| ViteManifest::load(manifest_path))`. The `manifest_path` argument is ignored on all but the first call process-wide.

**Problem:** Tests that call `resolve_assets` with different `manifest_path` values in the same process will silently get the first-call result. The existing manifest tests (`manifest.rs:77-173`) avoid this by testing `ViteManifest` methods directly (not `resolve_assets`), bypassing the global. [VERIFIED: `manifest.rs:82-173`]

**Assessment:** For D-12 tests covering the prod-mode HTML output, the test must either:
1. Call `InertiaResponse::to_html_response()` on a fresh `InertiaResponse` built from an `InertiaConfig` with `development: true` (dev-mode test — bypasses manifest entirely), or
2. For the prod-mode test, write a tempfile manifest and ensure it is the first call to `resolve_assets` in the test process — use `serial_test::serial` to serialize manifest tests.

**Decision for planner:** Do NOT key the manifest cache on path. The existing test-isolation pattern (test `ViteManifest::resolve()` directly, use `development: true` for HTML structure tests) is sufficient. Add a comment in `manifest.rs` documenting the single-global behavior for future contributors.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead |
|---------|-------------|-------------|
| Async-safe global state | Custom `Arc<Mutex<Option<T>>>` | `std::sync::OnceLock<T>` — already used in 3 places in the framework |
| HTML escaping for `data-page` | Custom escape function | Already implemented at `response.rs:384-389` — just preserve it |
| Vite manifest parsing | Custom JSON schema | Already in `manifest.rs:20-32` — `ViteManifest` + `ManifestEntry` |

---

## Docs Gap — Detailed Diff (D-10/D-11)

### `docs/src/features/inertia.md` — confirmed drift [VERIFIED: direct read]

1. **Lines 43-44** (Bootstrap Setup code block): References `InertiaConfig::from_env()` and
   `App::set_inertia_config(config)` which do not exist yet. These become accurate once
   D-02/D-03 land. No doc change needed here — the doc will be correct after implementation.

2. **Lines 53-59** (Manual Configuration code block): Struct literal is missing current
   fields `app_name` and `manifest_path`. After D-05, it will also be missing `title`,
   `head_extras`, `mount_id`. Fix: replace the struct literal with a builder chain that
   matches the actual `InertiaConfig` API (not a struct literal — struct fields are
   `pub` but the preferred API is builder methods).

3. **Missing sections** (D-10): No "First-Load HTML Shell", no same-origin story, no Vite
   proxy recipe.

### ferro-mcp `generation_context.rs` — assessment [VERIFIED: direct read]

`generation_context.rs:107-115` (field `inertia_render`) shows a snippet of
`Inertia::render()` usage. It does NOT show bootstrap / `App::set_inertia_config`. The
doc comment in the tool description (`docs/src/features/inertia.md` line 36-45 reproduces
the bootstrap pattern) is the canonical surface. The `code_templates.rs:422-477`
(`inertia_handler` template) does not reference config bootstrap. **Decision:** No
ferro-mcp change needed for this phase — the generation context correctly shows handler
usage; bootstrap belongs in docs, not in every handler template.

### Vite `server.proxy` recipe for docs (D-10)

**Confirmed via Context7/vitejs/vite** [CITED: https://github.com/vitejs/vite/blob/main/docs/config/server-options.md]:

```typescript
// frontend/vite.config.ts — split-port dev flow
// Backend runs on :8080 (or $APP_PORT), Vite dev server on :5173
export default defineConfig({
  server: {
    proxy: {
      // Route all non-asset requests to the Ferro backend.
      // This is what makes session cookies work: the browser
      // sends requests to :5173, Vite forwards them to :8080,
      // and the Set-Cookie header from Ferro is accepted because
      // from the browser's perspective the origin is :5173.
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: false,  // Keep origin as-is for session cookies
      },
      // If backend serves the HTML shell, proxy the root too:
      '/': {
        target: 'http://localhost:8080',
        changeOrigin: false,
      },
    },
  },
})
```

**Cookie note:** `changeOrigin: false` is recommended when the backend validates the
`Origin` or `Referer` header. With `changeOrigin: true`, Vite rewrites the `Origin` to
match the target — this can break CSRF validation. Session cookies set with `SameSite=Lax`
flow correctly because from the browser's perspective all requests go to the Vite origin
(:5173). However, `HttpOnly` + `SameSite=Strict` cookies require the same-origin story
(backend serves the shell directly at the same port) — the proxy approach only works with
`SameSite=Lax` or `None`.

**Same-origin story (primary):** Backend serves `GET /` → full HTML document with Vite
assets embedded. Browser and backend share the same origin (e.g., `http://localhost:8080`).
No proxy needed. Session cookies work with any `SameSite` value. This is the recommended
first-load pattern; the proxy recipe is a fallback for teams that prefer the Vite HMR
workflow without giving up dev-server hot reload.

---

## Common Pitfalls

### Pitfall 1: Manifest OnceLock bleed in parallel tests
**What goes wrong:** Two tests call `Inertia::render()` with `development: false` and
different `manifest_path` values. The second test gets the first test's assets.
**Prevention:** Use `development: true` for HTML structure tests. For prod-mode path tests,
test `ViteManifest::resolve()` directly (not `resolve_assets`) — as the existing tests do.

### Pitfall 2: `OnceLock::set()` silent no-op after first call
**What goes wrong:** `App::set_inertia_config()` is called twice (e.g., once from
`config::register_all()` and once from bootstrap). The second call silently does nothing.
**Prevention:** Document the set-once semantics in the method docstring. Consider `debug_assert` or a `tracing::warn!` on second-call if the framework has a logging dep (it does — `tracing` is used in other modules).

### Pitfall 3: Stale doc drift on `html_template` escape hatch
**What goes wrong:** After adding `head_extras`, users set both `html_template` and
`head_extras`, expecting both to be honored. `head_extras` is ignored when
`html_template` is `Some` (by design, D-06).
**Prevention:** Document this in the `head_extras` field doc comment and in the docs.

### Pitfall 4: `from_env()` re-reads env vars on every call
**What goes wrong:** If `from_env()` is called in the hot path (e.g., as the fallback in
`get_inertia_config()`), it reads env vars on every request.
**Prevention:** The fallback pattern `INERTIA_CONFIG.get().cloned().unwrap_or_else(InertiaConfig::default)` calls `default()` (which calls `from_env()`) only when the global is not set. After the first request, `OnceLock::get()` is a simple pointer load with no env reads. This is safe.

---

## Code Examples

### End-to-End Test Pattern (D-12)

Existing integration tests in `framework/tests/` use `ferro_rs` directly and `serial_test::serial`. The `InertiaRequest` trait makes it easy to build a mock request:

```rust
// ferro-inertia/src/response.rs — add a #[cfg(test)] mod at the bottom

#[cfg(test)]
mod content_negotiation_tests {
    use super::*;
    use crate::config::InertiaConfig;

    // Minimal mock request implementing InertiaRequest
    struct MockReq {
        is_inertia: bool,
        path: &'static str,
    }

    impl crate::request::InertiaRequest for MockReq {
        fn inertia_header(&self, name: &str) -> Option<&str> {
            if name == "X-Inertia" && self.is_inertia { Some("true") } else { None }
        }
        fn path(&self) -> &str { self.path }
    }

    #[test]
    fn non_inertia_request_returns_html_document() {
        let req = MockReq { is_inertia: false, path: "/home" };
        let config = InertiaConfig::new().development(true);
        let resp = Inertia::render_with_config(&req, "Home", serde_json::json!({"title": "Hi"}), config);
        assert_eq!(resp.content_type, "text/html; charset=utf-8");
        assert!(resp.body.contains("<!DOCTYPE html>"));
        assert!(resp.body.contains(r#"data-page=""#));
    }

    #[test]
    fn inertia_request_returns_json_contract() {
        let req = MockReq { is_inertia: true, path: "/home" };
        let config = InertiaConfig::new().development(true);
        let resp = Inertia::render_with_config(&req, "Home", serde_json::json!({"title": "Hi"}), config);
        assert_eq!(resp.content_type, "application/json");
        let body: serde_json::Value = serde_json::from_str(&resp.body).unwrap();
        assert_eq!(body["component"], "Home");
    }

    #[test]
    fn html_data_page_equals_json_contract_page_object() {
        // Same props, same component, same url — both paths must produce
        // the same page JSON (modulo HTML escaping).
        let props = serde_json::json!({"title": "Hi", "count": 42});
        let config = InertiaConfig::new().development(true).version("test-1");

        let non_inertia = MockReq { is_inertia: false, path: "/home" };
        let html_resp = Inertia::render_with_config(&non_inertia, "Home", props.clone(), config.clone());

        let inertia = MockReq { is_inertia: true, path: "/home" };
        let json_resp = Inertia::render_with_config(&inertia, "Home", props, config);

        // Extract data-page value from HTML
        let start = html_resp.body.find(r#"data-page=""#).unwrap() + 11;
        let end = html_resp.body[start..].find('"').unwrap() + start;
        let page_json_escaped = &html_resp.body[start..end];
        // Unescape HTML attribute encoding
        let page_json = page_json_escaped
            .replace("&quot;", "\"")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&#x27;", "'");

        let html_page: serde_json::Value = serde_json::from_str(&page_json).unwrap();
        let json_page: serde_json::Value = serde_json::from_str(&json_resp.body).unwrap();
        assert_eq!(html_page, json_page);
    }

    #[test]
    fn dev_mode_emits_vite_client_script() {
        let req = MockReq { is_inertia: false, path: "/" };
        let config = InertiaConfig::new()
            .development(true)
            .vite_dev_server("http://localhost:5173");
        let resp = Inertia::render_with_config(&req, "App", serde_json::json!({}), config);
        assert!(resp.body.contains("http://localhost:5173/@vite/client"));
    }

    #[test]
    fn prod_mode_emits_hashed_asset_paths() {
        // Use ViteManifest::resolve() directly to avoid OnceLock bleed
        use crate::manifest::ViteManifest;
        let manifest_json = r#"{"src/main.tsx":{"file":"assets/app-abc.js","isEntry":true,"css":["assets/app-def.css"]}}"#;
        let manifest: ViteManifest = serde_json::from_str(manifest_json).unwrap();
        let resolved = manifest.resolve("src/main.tsx").unwrap();
        assert_eq!(resolved.js, "/assets/app-abc.js");
        assert_eq!(resolved.css[0], "/assets/app-def.css");
    }
}
```

Note: `ViteManifest` is currently `pub(crate)` (not `pub`). For the prod-mode test to
call `manifest.resolve()` directly from outside the module, either make `ViteManifest`
pub (not recommended — it is an internal detail) or move the prod-mode assertion inside
`manifest.rs` tests (already done — see `manifest.rs:82`). The D-12 requirement for
prod-mode is covered by the existing manifest unit tests; the new test only needs to cover
content-negotiation and the new configurable fields.

### Bootstrap pattern (for docs) [ASSUMED — pattern does not exist yet]

```rust
// src/bootstrap.rs
use ferro::{App, InertiaConfig};

pub async fn register() {
    App::set_inertia_config(
        InertiaConfig::from_env()
            .title("My App")
            .head_extras(r#"<link rel="icon" href="/favicon.ico">"#)
    );
    // other bootstrap...
}
```

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `tokio::test` where async needed |
| Dev-dep | `tempfile` already in `ferro-inertia` dev-dependencies |
| Serialization | `serial_test::serial` for manifest tests (used in framework tests) |
| Quick run command | `cargo test -p ferro-inertia -- content_negotiation` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Behavior | Test Type | Automated Command | Location |
|----------|-----------|-------------------|----------|
| Same handler: non-X-Inertia → HTML document | unit | `cargo test -p ferro-inertia -- non_inertia_request_returns_html_document` | `ferro-inertia/src/response.rs` — new |
| Same handler: X-Inertia → JSON contract | unit | `cargo test -p ferro-inertia -- inertia_request_returns_json_contract` | `ferro-inertia/src/response.rs` — new |
| HTML `data-page` == JSON page object | unit | `cargo test -p ferro-inertia -- html_data_page_equals_json_contract` | `ferro-inertia/src/response.rs` — new |
| Dev mode: emits `@vite/client` script tag | unit | `cargo test -p ferro-inertia -- dev_mode_emits_vite_client_script` | `ferro-inertia/src/response.rs` — new |
| Prod mode: manifest resolves to hashed paths | unit | `cargo test -p ferro-inertia -- parse_manifest_and_resolve_entry` | `ferro-inertia/src/manifest.rs:82` — exists |
| `from_env()` reads env vars | unit | `cargo test -p ferro-inertia -- from_env_reads` | `ferro-inertia/src/config.rs` — new |
| `head_extras` injected into `<head>` | unit | `cargo test -p ferro-inertia -- head_extras_in_html` | `ferro-inertia/src/response.rs` — new |
| `mount_id` applied to mount div | unit | `cargo test -p ferro-inertia -- mount_id_applied` | `ferro-inertia/src/response.rs` — new |
| `title` overrides `app_name` in `<title>` | unit | `cargo test -p ferro-inertia -- title_override` | `ferro-inertia/src/response.rs` — new |

### Sampling Rate
- Per task commit: `cargo test -p ferro-inertia`
- Per wave merge: `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- Phase gate: Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- No existing test file covers `to_html_response()` — add `#[cfg(test)] mod content_negotiation_tests` at the bottom of `response.rs`.
- No existing test for `InertiaConfig::from_env()` — add to `config.rs`.

---

## Open Questions

1. **`from_env()` for `VITE_ENTRY_POINT` env var**
   - Current: `entry_point` is hardcoded `"src/main.tsx"` in `Default::default()`.
   - `VITE_ENTRY_POINT` is listed in `docs/src/features/inertia.md:26` as a supported env var but NOT read in `Default::default()`.
   - Recommendation: `from_env()` should read `VITE_ENTRY_POINT` — this is clearly the intent. Fix the gap.

2. **`INERTIA_VERSION` env var**
   - `docs/src/features/inertia.md:28` lists `INERTIA_VERSION`. Not read in `Default::default()` — `version` is hardcoded `"1.0"`.
   - Recommendation: `from_env()` reads `INERTIA_VERSION` env var. Fix this gap too.

3. **`title` vs `app_name` — collapse?**
   - Claude's discretion. Recommendation: keep both. `app_name` is used in the dev/prod template `<title>` currently and also in the `X-Frame-Options`-style meta. `title: Option<String>` provides an explicit override without breaking existing `app_name` usage. When `title` is `Some`, use it; when `None`, use `app_name`. One line of change in the template.

4. **Second call to `App::set_inertia_config` behavior**
   - `OnceLock::set()` returns `Err(value)` if already set. Should the framework log a warning? Recommendation: `if INERTIA_CONFIG.set(config).is_err() { eprintln!("Warning: InertiaConfig already set; second call ignored"); }` — consistent with how the middleware registry handles duplicate registrations silently but visible in debug.

---

## Environment Availability

Step 2.6: SKIPPED — this phase is code/config changes only (no external services, databases, or CLIs beyond the Rust toolchain).

---

## Sources

### Primary (HIGH confidence)
- `ferro-inertia/src/response.rs` — full read, all line numbers verified
- `ferro-inertia/src/config.rs` — full read
- `ferro-inertia/src/manifest.rs` — full read
- `ferro-inertia/src/lib.rs` — full read
- `ferro-inertia/Cargo.toml` — full read
- `framework/src/inertia/context.rs` — full read
- `framework/src/inertia/mod.rs` — full read
- `framework/src/inertia/config.rs` — full read
- `framework/src/container/mod.rs` — full read
- `framework/src/config/repository.rs` — full read
- `framework/src/config/providers/app.rs` — full read
- `framework/src/middleware/registry.rs` — full read
- `framework/src/lib.rs` — partial read (re-export surface)
- `docs/src/features/inertia.md` — full read

### Secondary (MEDIUM confidence — verified)
- [CITED: https://github.com/vitejs/vite/blob/main/docs/config/server-options.md] — `server.proxy` options, `changeOrigin`, cookie behavior via Context7 `/vitejs/vite`

### Tertiary (LOW confidence)
- None — all claims verified against live source.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `OnceLock<InertiaConfig>` (no RwLock) is sufficient because set happens once before tokio starts accepting connections | Global Config Pattern | If `App::set_inertia_config` is called after request handling begins (unlikely but possible in tests), a race exists. Mitigation: document set-before-serve requirement. |
| A2 | `changeOrigin: false` is the right cookie-forwarding config for Vite proxy | Docs / Vite proxy recipe | The exact `changeOrigin` value depends on whether the backend validates Origin header. Correct recommendation is to document both options and let consumer choose. |

**All other claims are VERIFIED against live source files.**

---

## Metadata

**Confidence breakdown:**
- Config plumbing gaps: HIGH — verified at exact line numbers
- Global config pattern selection: HIGH — compared all three existing patterns
- Template extension: HIGH — template strings read verbatim
- Manifest OnceLock test isolation: HIGH — verified existing test pattern avoids `resolve_assets`
- Vite proxy recipe: MEDIUM — from official Vite docs via Context7

**Research date:** 2026-06-21
**Valid until:** 90 days (stable, no external moving parts)
