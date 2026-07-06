# Phase 238: Inertia First-Load HTML Shell — Pattern Map

**Mapped:** 2026-06-21
**Files analyzed:** 6 (1 new, 5 modified)
**Analogs found:** 6 / 6

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| NEW `framework/src/inertia/global.rs` | config | request-response | `ferro-inertia/src/manifest.rs:57` (bare `OnceLock<T>` set-once) | exact |
| MODIFY `ferro-inertia/src/config.rs` | config | request-response | `framework/src/config/providers/app.rs` (`from_env()` + `default()` delegation) | exact |
| MODIFY `ferro-inertia/src/response.rs` | utility | request-response | itself — extend dev/prod template branches at :402/:433 | self-analog |
| MODIFY `framework/src/inertia/context.rs` | utility | request-response | `framework/src/middleware/registry.rs` (`get_global_*` reading a `OnceLock` static) | role-match |
| MODIFY `framework/src/lib.rs` | config | — | itself — `#[cfg(feature = "inertia")] pub use inertia::{...}` at :121-122 | self-analog |
| MODIFY `docs/src/features/inertia.md` | docs | — | itself — existing sections and builder-chain examples | self-analog |

---

## Pattern Assignments

### NEW `framework/src/inertia/global.rs` (config, set-once process global)

**Analog:** `ferro-inertia/src/manifest.rs` — bare `OnceLock<T>` with no `RwLock`. This is the correct analog because `InertiaConfig` is set once at bootstrap before any request is served, identical to how `MANIFEST` is written once on first resolve. The `OnceLock<RwLock<T>>` pattern from `middleware/registry.rs` and `config/repository.rs` is for collections that can grow; `InertiaConfig` is a single value written once.

**Imports pattern** (`manifest.rs:6-8`):
```rust
use std::sync::OnceLock;
```

**Core OnceLock pattern** (`manifest.rs:57`, `manifest.rs:64-74`):
```rust
/// Global cache for the parsed manifest.
static MANIFEST: OnceLock<Option<ViteManifest>> = OnceLock::new();

pub fn resolve_assets(manifest_path: &str, entry_point: &str) -> ResolvedAssets {
    let manifest = MANIFEST.get_or_init(|| ViteManifest::load(manifest_path));
    manifest
        .as_ref()
        .and_then(|m| m.resolve(entry_point))
        .unwrap_or_else(|| ResolvedAssets { ... })
}
```

**Adapt for `global.rs`** — single-value, not `Option<T>`, set is explicit (not `get_or_init`):
```rust
use ferro_inertia::InertiaConfig;
use std::sync::OnceLock;

static INERTIA_CONFIG: OnceLock<InertiaConfig> = OnceLock::new();

/// Set the process-global InertiaConfig. Call once from bootstrap before the server starts.
/// Subsequent calls are silently ignored (OnceLock semantics).
pub fn set_inertia_config(config: InertiaConfig) {
    if INERTIA_CONFIG.set(config).is_err() {
        eprintln!("Warning: InertiaConfig already set; second call ignored");
    }
}

/// Get the active InertiaConfig, falling back to from_env()/default() when unset.
pub fn get_inertia_config() -> InertiaConfig {
    INERTIA_CONFIG
        .get()
        .cloned()
        .unwrap_or_else(InertiaConfig::default)
}
```

**Module declaration:** Add `mod global;` in `framework/src/inertia/mod.rs` (alongside the existing `mod config;`, `mod context;`, `mod response;` at lines 23-25). Add `pub use global::{get_inertia_config, set_inertia_config};` in `mod.rs`.

---

### MODIFY `ferro-inertia/src/config.rs` (config, from_env extraction + new fields)

**Analog:** `framework/src/config/providers/app.rs` — the canonical `from_env()` constructor pattern in this codebase.

**`from_env()` pattern** (`providers/app.rs:19-59`):
```rust
impl AppConfig {
    /// Build config from environment variables
    pub fn from_env() -> Self {
        Self {
            name: env("APP_NAME", "Ferro Application".to_string()),
            environment: Environment::detect(),
            debug: env("APP_DEBUG", true),
            url: env("APP_URL", "http://localhost:8080".to_string()),
            // ...
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::from_env()   // default() delegates to from_env()
    }
}
```

**Existing consuming-builder pattern** (`config.rs:72-141`):
```rust
/// Set the Vite dev server URL.
pub fn vite_dev_server(mut self, url: impl Into<String>) -> Self {
    self.vite_dev_server = url.into();
    self
}
// ... all builders follow this shape: `pub fn NAME(mut self, val: impl Into<String>) -> Self`
```

**What to change in `config.rs`:**

1. Add three new fields to the `InertiaConfig` struct (after `manifest_path`, before closing `}`):
```rust
/// Optional page title. When `Some`, overrides `app_name` in `<title>`.
pub title: Option<String>,
/// Raw HTML injected into `<head>` before `</head>`. Ignored when `html_template` is set.
pub head_extras: Option<String>,
/// id attribute of the mount node. Defaults to `"app"`.
pub mount_id: String,
```

2. Extract `Default::default()` body into `from_env()`, adding the three new fields and reading two missing env vars (`VITE_ENTRY_POINT`, `INERTIA_VERSION`):
```rust
impl InertiaConfig {
    pub fn from_env() -> Self {
        let vite_dev_server = std::env::var("VITE_DEV_SERVER")
            .unwrap_or_else(|_| "http://localhost:5173".to_string());
        let is_dev = !matches!(
            std::env::var("APP_ENV").ok().as_deref(),
            Some("production") | Some("staging")
        );
        let app_name = std::env::var("APP_NAME").unwrap_or_else(|_| "Ferro".to_string());
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

3. Add three new consuming builders at the end of the `impl InertiaConfig` block, following the existing `pub fn NAME(mut self, ...) -> Self` shape:
```rust
/// Override the `<title>` tag. When `None`, falls back to `app_name`.
pub fn title(mut self, t: impl Into<String>) -> Self {
    self.title = Some(t.into());
    self
}

/// Raw HTML injected into `<head>` before `</head>`.
/// Ignored when `html_template` is set.
pub fn head_extras(mut self, h: impl Into<String>) -> Self {
    self.head_extras = Some(h.into());
    self
}

/// Set the mount node id (default `"app"`).
pub fn mount_id(mut self, id: impl Into<String>) -> Self {
    self.mount_id = id.into();
    self
}
```

---

### MODIFY `ferro-inertia/src/response.rs` (utility, extend HTML templates)

**Analog:** Itself — the dev-mode template branch at lines 402-432 and the prod-mode branch at lines 433-461 are the exact targets. The custom-template path at lines 394-399 must remain untouched.

**Current hardcoded values to replace** (lines 422 and 455 in dev/prod branches respectively):
- `<title>{}</title>` using `self.config.app_name` → replace format arg with `title_text`
- `<div id="app" data-page="{}">` hardcoded `"app"` → replace with `self.config.mount_id`
- Before `</head>` → inject `self.config.head_extras.as_deref().unwrap_or("")`

**Pattern for deriving title_text** (add once before the `if self.config.development` branch):
```rust
let title_text = self.config.title.as_deref().unwrap_or(&self.config.app_name);
```

**Dev template update** — replace the relevant format args:
```rust
// was: self.config.app_name,  (title format arg)
// becomes: title_text,

// was: <div id="app" data-page="{}">
// becomes: <div id="{}" data-page="{}">
// format args: self.config.mount_id, page_json

// head_extras injection: add {} before </head>
// format arg: self.config.head_extras.as_deref().unwrap_or("")
```

**Prod template update** — same substitutions in named format args:
```rust
// was: app_name = self.config.app_name,
// becomes: title_text = title_text,

// add: mount_id = self.config.mount_id,
// add: head_extras = self.config.head_extras.as_deref().unwrap_or(""),
```

**Custom-template path** (lines 394-399) — do NOT modify. It owns the full HTML when `html_template` is `Some`.

**Test module to add** — add `#[cfg(test)] mod content_negotiation_tests` at the bottom of `response.rs`. Use `development: true` for all HTML-structure tests to bypass the manifest `OnceLock`. Test `ViteManifest` directly (not `resolve_assets`) for prod-mode manifest assertions. See RESEARCH.md §Code Examples for the full test scaffold.

---

### MODIFY `framework/src/inertia/context.rs` (utility, read process global)

**Analog:** `framework/src/middleware/registry.rs:53-59` — the `get_global_middleware()` function reads a `OnceLock` static and falls back gracefully.

**Reading pattern** (`registry.rs:53-59`):
```rust
pub fn get_global_middleware() -> Vec<BoxedMiddleware> {
    GLOBAL_MIDDLEWARE
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|vec| vec.clone())
        .unwrap_or_default()
}
```

For `InertiaConfig` (no `RwLock` layer), the equivalent in `global.rs` is simpler:
```rust
INERTIA_CONFIG.get().cloned().unwrap_or_else(InertiaConfig::default)
```

**Two call sites to update in `context.rs`:**

Line 126 — `Inertia::render`:
```rust
// Before:
Self::render_with_config(req, component, props, InertiaConfig::default())
// After:
Self::render_with_config(req, component, props, crate::inertia::global::get_inertia_config())
```

Line 200 — `Inertia::render_ctx`:
```rust
// Before:
ferro_inertia::Inertia::render_with_options(ctx, component, props, Some(&shared), InertiaConfig::default())
// After:
ferro_inertia::Inertia::render_with_options(ctx, component, props, Some(&shared), crate::inertia::global::get_inertia_config())
```

No other changes to `context.rs`.

---

### MODIFY `framework/src/lib.rs` (public re-export surface)

**Analog:** Itself — existing inertia re-export at line 121-122:
```rust
#[cfg(feature = "inertia")]
pub use inertia::{Inertia, InertiaConfig, InertiaResponse, InertiaShared, SavedInertiaContext};
```

`App::set_inertia_config` is a method on `App` (defined in `framework/src/container/mod.rs`), not a free function requiring a separate re-export. Users call `ferro::App::set_inertia_config(config)` via the existing `App` re-export at line 71: `pub use container::{App, Container};`.

**What to add in `container/mod.rs`** — new method on `App`:
```rust
/// Set the process-global Inertia configuration.
/// Call once from `bootstrap.rs` before the server starts accepting requests.
#[cfg(feature = "inertia")]
pub fn set_inertia_config(config: ferro_inertia::InertiaConfig) {
    crate::inertia::global::set_inertia_config(config);
}
```

No change to `framework/src/lib.rs` needed — `App` is already exported and the new method is on `App`.

If `set_inertia_config` or `get_inertia_config` need to be directly importable as free functions, add them to the `framework/src/inertia/mod.rs` pub-use and then to `framework/src/lib.rs` line 122 — but the method-on-App surface covers D-02 without that.

---

### MODIFY `docs/src/features/inertia.md` (docs)

**Analog:** Itself — existing structure with fenced code blocks using `rust,ignore`.

**Line 43-44** — already correct once implementation lands; no doc change needed for these lines.

**Lines 53-59** — replace the stale struct literal with a builder chain:
```rust
// Before (struct literal, missing fields, won't compile):
// InertiaConfig { ... }
//
// After (builder chain — matches the pub API):
let config = InertiaConfig::from_env()
    .title("My App")
    .head_extras(r#"<link rel="icon" href="/favicon.ico">"#)
    .mount_id("root");
```

**New section to add: "First-Load HTML Shell"** — covers:
- Same-origin story (backend serves `GET /` → full HTML + Vite asset tags, session cookies work with any `SameSite`)
- Vite `server.proxy` recipe for split-port dev (see RESEARCH.md §Docs Gap for the full `vite.config.ts` snippet)
- Note that `head_extras` is ignored when `html_template` is set (D-06)

---

## Shared Patterns

### Process-global OnceLock (bare, no RwLock)
**Source:** `ferro-inertia/src/manifest.rs:57`
**Apply to:** `framework/src/inertia/global.rs` (the new file)
```rust
static MANIFEST: OnceLock<Option<ViteManifest>> = OnceLock::new();
// get_or_init on first read (lazy); for InertiaConfig use explicit set() + get().cloned()
```

### `from_env()` + `default()` delegation
**Source:** `framework/src/config/providers/app.rs:19-58`
**Apply to:** `ferro-inertia/src/config.rs` (`InertiaConfig::from_env()` extraction)
```rust
impl Default for AppConfig {
    fn default() -> Self {
        Self::from_env()
    }
}
```

### Consuming builder `with_*(mut self) -> Self`
**Source:** `ferro-inertia/src/config.rs:72-141` (existing builders on `InertiaConfig`)
**Apply to:** New `title`, `head_extras`, `mount_id` builders in the same file
```rust
pub fn vite_dev_server(mut self, url: impl Into<String>) -> Self {
    self.vite_dev_server = url.into();
    self
}
```

### `#[cfg(feature = "inertia")]` gating
**Source:** `framework/src/lib.rs:121`, `framework/src/container/mod.rs` (pattern inferred from existing cfg gates)
**Apply to:** The new `App::set_inertia_config` method in `container/mod.rs`

### Deprecation comment on thread-local anti-pattern
**Source:** `framework/src/inertia/context.rs:312-318`
**Negative pattern — do NOT repeat:** `thread_local!` was the old approach; `OnceLock` is the correct async-safe replacement. The `#[deprecated]` `InertiaContext` is a live reminder of this.

### Test isolation for global OnceLock
**Source:** `ferro-inertia/src/manifest.rs:82-173` (tests call `ViteManifest::resolve()` directly, never `resolve_assets()`)
**Apply to:** D-12 tests in `ferro-inertia/src/response.rs` — use `development: true` for HTML-structure tests; use `ViteManifest::resolve()` directly for prod-mode path assertions. Add `#[serial]` (from `serial_test`) if any test does reach `resolve_assets`.

---

## No Analog Found

All 6 files have analogs. None require falling back to RESEARCH.md patterns alone — though RESEARCH.md §Code Examples provides a complete D-12 test scaffold that the planner should include directly.

---

## Metadata

**Analog search scope:** `framework/src/inertia/`, `framework/src/config/`, `framework/src/middleware/`, `framework/src/container/`, `ferro-inertia/src/`, `framework/src/lib.rs`
**Files read:** 12 source files, all verified against live tree
**Pattern extraction date:** 2026-06-21
