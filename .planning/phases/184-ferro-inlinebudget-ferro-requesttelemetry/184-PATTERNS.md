# Phase 184: `ferro::InlineBudget` + `ferro::RequestTelemetry` — Pattern Map

**Mapped:** 2026-06-06
**Files analyzed:** 10 (5 new, 5 modified, 1 optional)
**Analogs found:** 10 / 10

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `framework/src/telemetry/mod.rs` (NEW) | module-root | re-export hub | `framework/src/http/mod.rs:1-26` | exact |
| `framework/src/telemetry/inline_budget.rs` (NEW) | request-scoped state machine | request-response | `framework/src/http/action.rs:33-98, 290-359` (enum + state + tracing::warn!) | role-match |
| `framework/src/telemetry/request_telemetry.rs` (NEW) | process-global concurrent registry | event-driven (push samples) | `ferro-bundle/src/lib.rs:32-85, 282-298` (OnceLock<DashMap> + reset) | exact |
| `framework/tests/telemetry_smoke.rs` (NEW) | integration test | request-response | `framework/tests/action_handler.rs:1-90` (real Request via TCP loopback) | exact |
| `docs/src/the-basics/inline-budget-and-telemetry.md` (NEW) | docs page | n/a | `docs/src/the-basics/action-handlers.md:1-130` | exact |
| `framework/src/config/providers/app.rs` (MODIFIED) | config provider | boot-time read | `framework/src/config/providers/app.rs:1-98` (self-analog — additive field) | exact |
| `framework/src/http/request.rs` (MODIFIED) | per-request setter methods | request-response | `framework/src/http/request.rs:742-777` (`flash` / `redirect_to` block) | exact |
| `framework/src/lib.rs` (MODIFIED) | crate-root re-exports | re-export hub | `framework/src/lib.rs:60-113` (existing `pub use` lines) | exact |
| `Cargo.toml` (MODIFIED) | workspace version | n/a | `Cargo.toml:33-34` (single-line edit) | exact |
| `docs/src/SUMMARY.md` (MODIFIED) | docs ToC | n/a | `docs/src/SUMMARY.md:12-18` ("The Basics" section) | exact |
| `framework/Cargo.toml` (OPTIONAL) | dev-deps | n/a | existing `[dev-dependencies]` block | exact |

---

## Pattern Assignments

### `framework/src/telemetry/mod.rs` (module-root, re-export hub)

**Analog:** `framework/src/http/mod.rs`

**Module declaration + flat re-export pattern** (`framework/src/http/mod.rs:1-26`):
```rust
pub mod action;
mod body;
pub mod cookie;
mod extract;
mod form_request;
mod multipart;
mod request;
pub mod request_context;
/// API resource and pagination types.
pub mod resources;
mod response;

pub use action::{
    ActionError, ActionKind, ActionResult, ActionResultExt, FlashVariant, IntoActionError,
};
pub use body::{collect_body, parse_form, parse_json};
pub use cookie::{parse_cookies, Cookie, CookieOptions, SameSite};
pub use extract::{FromParam, FromRequest};
pub use form_request::FormRequest;
pub use multipart::{validate_mime, validate_size, MultipartForm, UploadedFile};
pub use request::{Request, RequestParts};
```

**Notes for planner:**
- Declare `pub mod inline_budget;` and `pub mod request_telemetry;` (matches the public-module convention so `use ferro_rs::telemetry::Sample` is reachable — addresses RESEARCH Q8 Risk 1).
- Re-export at module level: `pub use inline_budget::Decision; pub use request_telemetry::{RequestTelemetry, Sample};`. Do NOT re-export `InlineBudget` (RESEARCH Q8 Risk 2 / CONTEXT D-02).
- Add a `//!` module-level doc block covering both primitives, the lost-on-restart semantic (D-10), and the 100 KB default threshold (D-04). Match the documentation density of `framework/src/http/action.rs:1-27` (28-line `//!` header).

---

### `framework/src/telemetry/inline_budget.rs` (request-scoped state machine, request-response)

**Analog:** `framework/src/http/action.rs` (enum + struct + `tracing::warn!` site)

**Locked enum pattern** (CONTEXT D-03):
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Inline,
    Preload(String),
}
```

This matches the shape of `ActionKind` at `framework/src/http/action.rs:35-47` (small enum with `derive(Debug, Clone, ..., PartialEq, Eq)`).

**Per-request state struct** — analog: `ActionOverrides` at `framework/src/http/action.rs` (used by `request.rs:25`). State that lives in `Request::extensions`:
```rust
#[derive(Default)]
pub(crate) struct InlineBudgetState {
    cumulative: HashMap<String, usize>,
    warned: HashSet<String>,
}
```

Use `pub(crate)` (not `pub`) — the user never names this type (CONTEXT D-02 / RESEARCH Q8 Risk 2).

**Structured `tracing::warn!` pattern** (`framework/src/http/action.rs:305-309`):
```rust
tracing::warn!(
    handler = %handler_name,
    rejected_url = %sanitize_for_log(rejected),
    "redirect_override rejected: not same-origin (success path)"
);
```

Adapt to Phase 184's required fields (CONTEXT D-06):
```rust
tracing::warn!(
    key = %key,
    cumulative_bytes = state.cumulative.get(key).copied().unwrap_or(0),
    threshold_bytes = threshold,
    fallback_url = %fallback_url,
    route_pattern = %route_pattern,
    "inline_budget: threshold crossed; flipping to Preload"
);
```

Sigil rules (RESEARCH Q2): `%` for `Display` (strings), `?` for `Debug` (enums), bare for numeric primitives (`usize`, `u64`).

**Decision function — borrow-checker-safe ordering** (RESEARCH Pitfall 1 + Q1):
```rust
pub(crate) fn decide(
    req: &mut crate::http::Request,
    key: &str,
    bytes: usize,
    fallback_url: &str,
) -> Decision {
    // 1. Read all &self-borrowing values BEFORE &mut self borrow.
    let threshold = crate::Config::get::<crate::AppConfig>()
        .map(|c| c.inline_budget_threshold_bytes)
        .unwrap_or(102_400);
    let route_pattern = req.route_pattern().unwrap_or_default();

    // 2. Lazy-init InlineBudgetState in extensions.
    if req.get::<InlineBudgetState>().is_none() {
        req.insert(InlineBudgetState::default());
    }
    let state = req.get_mut::<InlineBudgetState>().expect("just inserted");

    // 3. State machine: increment, decide, fire-once warn.
    let entry = state.cumulative.entry(key.to_string()).or_insert(0);
    *entry = entry.saturating_add(bytes);
    let cumulative = *entry;
    if cumulative <= threshold {
        return Decision::Inline;
    }
    if !state.warned.contains(key) {
        state.warned.insert(key.to_string());
        tracing::warn!( /* see structured-fields excerpt above */ );
    }
    Decision::Preload(fallback_url.to_string())
}
```

**Notes for planner:**
- The `unwrap_or(102_400)` fallback is mandatory — synthetic Requests in unit tests bypass `Config::init()` (RESEARCH Pitfall 5).
- `route_pattern().unwrap_or_default()` yields `""` when None — explicitly the convention per CONTEXT `<specifics>` line 408 (RESEARCH Q4).
- `Decision::Preload(String)` carries an owned `String` so the caller can drop it into `format!` (RESEARCH Q9 consumer pattern).
- Add inline `#[cfg(test)] mod tests` at the bottom — covers SC-1 (decides_inline_below_threshold) and SC-2 (warn_fires_once_per_key, state-machine assertion).

---

### `framework/src/telemetry/request_telemetry.rs` (process-global concurrent registry, event-driven)

**Analog:** `ferro-bundle/src/lib.rs` (OnceLock + DashMap + `reset()`)

**Global storage declaration** (`ferro-bundle/src/lib.rs:69-85`):
```rust
static BUNDLE_REGISTRY: OnceLock<DashMap<String, BundleEntry>> = OnceLock::new();
static ALIAS_REGISTRY: OnceLock<DashMap<String, String>> = OnceLock::new();
static NAME_INDEX: OnceLock<DashMap<String, String>> = OnceLock::new();

fn bundle_registry() -> &'static DashMap<String, BundleEntry> {
    BUNDLE_REGISTRY.get_or_init(DashMap::new)
}

fn alias_registry() -> &'static DashMap<String, String> {
    ALIAS_REGISTRY.get_or_init(DashMap::new)
}
```

Adapt for Phase 184:
```rust
use dashmap::DashMap;
use std::collections::VecDeque;
use std::sync::OnceLock;

static TELEMETRY_STORE: OnceLock<DashMap<(String, Option<String>), VecDeque<Sample>>>
    = OnceLock::new();

fn telemetry_store() -> &'static DashMap<(String, Option<String>), VecDeque<Sample>> {
    TELEMETRY_STORE.get_or_init(DashMap::new)
}
```

**Sample struct** (CONTEXT D-07 + RESEARCH Code Examples):
```rust
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    pub recorded_at: SystemTime,
    pub value: serde_json::Value,
}

impl Sample {
    pub fn now(value: serde_json::Value) -> Self {
        Self { recorded_at: SystemTime::now(), value }
    }
    pub fn at(when: SystemTime, value: serde_json::Value) -> Self {
        Self { recorded_at: when, value }
    }
}
```

**`RequestTelemetry` unit struct + namespaced static methods** (CONTEXT D-02):
```rust
pub struct RequestTelemetry;

impl RequestTelemetry {
    pub fn snapshot(key: &str, scope: Option<&str>) -> Vec<Sample> {
        let scope_owned = scope.map(|s| s.to_string());
        telemetry_store()
            .get(&(key.to_string(), scope_owned))
            .map(|entry| entry.value().iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn keys() -> Vec<(String, Option<String>)> {
        telemetry_store().iter().map(|e| e.key().clone()).collect()
    }

    pub fn clear() {
        if let Some(r) = TELEMETRY_STORE.get() {
            r.clear();
        }
    }

    #[cfg(test)]
    pub(crate) fn reset() {
        Self::clear();
    }
}
```

**Private writer with ring-buffer cap** (CONTEXT D-08 + RESEARCH Pitfall 3):
```rust
pub(crate) fn record(key: &str, scope: Option<&str>, sample: Sample) {
    let map_key = (key.to_string(), scope.map(|s| s.to_string()));
    let mut entry = telemetry_store()
        .entry(map_key)
        .or_insert_with(|| VecDeque::with_capacity(128));
    entry.push_back(sample);
    while entry.len() > 128 {
        entry.pop_front();
    }
}
```

**Test-isolation reset pattern** (`ferro-bundle/src/lib.rs:287-298` — verbatim shape):
```rust
#[cfg(test)]
pub(crate) fn reset() {
    if let Some(r) = BUNDLE_REGISTRY.get() {
        r.clear();
    }
    // ... repeat for other registries
}
```

For Phase 184 this lives as `RequestTelemetry::reset()` (D-15) — see the `impl RequestTelemetry` excerpt above.

**Notes for planner:**
- DashMap `entry().or_insert_with(...)` deadlock pitfall (RESEARCH Pitfall 2): keep the entry guard in a single statement chain; do not call `get_mut` on the same key while the entry is alive.
- Inline `#[cfg(test)] mod tests` covers SC-3a (round_trip), SC-3b (concurrent_record_no_deadlock — spawn N threads), SC-3c (ring_buffer_caps_at_128 — push 200, snapshot, assert `len == 128`).
- Use `#[serial]` from `serial_test` (already a dev-dep at `framework/Cargo.toml:79`) on any test where order could leak state, OR `RequestTelemetry::reset()` at the top of every test (RESEARCH Pitfall 4 + CONTEXT D-15).

---

### `framework/src/http/request.rs` (per-request setter methods, request-response)

**Analog:** Self — the second `impl Request` block at `framework/src/http/request.rs:742-777` (holds `flash` and `redirect_to`)

**Existing impl-block shape** (verbatim from `request.rs:742-777`):
```rust
impl Request {
    /// Record a success-side flash key for the `#[action]` macro runtime to write
    /// to the session `_action` flash slot when the handler returns `Ok(())`.
    ///
    /// Has no observable effect outside an `#[action]`-decorated handler.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[action(redirect_to = "/dashboard/pagine")]
    /// pub async fn create(req: Request) -> ActionResult {
    ///     let new_id = Page::create(...).await?;
    ///     req.redirect_to(format!("/dashboard/pagine/{new_id}"));
    ///     req.flash("created");
    ///     Ok(())
    /// }
    /// ```
    pub fn flash(&mut self, key: impl Into<String>) {
        self.action_overrides.flash = Some(key.into());
    }

    pub fn redirect_to(&mut self, url: impl Into<String>) {
        self.action_overrides.redirect_override = Some(url.into());
    }
}
```

**Phase 184 additions to the SAME second impl block** (RESEARCH Q1 — research is explicit: "Add to the second impl block at lines 742-777"):
```rust
impl Request {
    // ... existing flash / redirect_to ...

    /// Decide whether `bytes` should be inlined into the response or preloaded
    /// from `fallback_url`. Decision is request-scoped; cumulative bytes per
    /// `key` accumulate across calls within a single request.
    ///
    /// A `tracing::warn!` fires exactly once per `(key, request)` when the
    /// cumulative byte count first crosses `AppConfig::inline_budget_threshold_bytes`
    /// (default 100 KiB).
    pub fn inline_budget(
        &mut self,
        key: &str,
        bytes: usize,
        fallback_url: &str,
    ) -> crate::Decision {
        crate::telemetry::inline_budget::decide(self, key, bytes, fallback_url)
    }

    /// Record a telemetry `Sample` against `key` in the global ring buffer.
    /// Equivalent to `telemetry_record_scoped(key, None, sample)`.
    pub fn telemetry_record(&mut self, key: &str, sample: crate::Sample) {
        crate::telemetry::request_telemetry::record(key, None, sample);
    }

    pub fn telemetry_record_scoped(
        &mut self,
        key: &str,
        scope: Option<&str>,
        sample: crate::Sample,
    ) {
        crate::telemetry::request_telemetry::record(key, scope, sample);
    }
}
```

**Existing extension API to reuse** (`framework/src/http/request.rs:87-103`):
```rust
pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) {
    self.extensions.insert(TypeId::of::<T>(), Box::new(value));
}

pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
    self.extensions
        .get(&TypeId::of::<T>())
        .and_then(|boxed| boxed.downcast_ref::<T>())
}

pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
    self.extensions
        .get_mut(&TypeId::of::<T>())
        .and_then(|boxed| boxed.downcast_mut::<T>())
}
```

`InlineBudgetState` is stored/retrieved via this existing API inside `crate::telemetry::inline_budget::decide` — no new infrastructure on `Request`.

**Notes for planner:**
- Method bodies are thin delegators — request.rs stays slim (CONTEXT D-11 rationale).
- Doc comments follow the `flash`/`redirect_to` density: short summary + optional `# Example` block. Match `request.rs:743-758` for tone.
- Do NOT touch the first impl block (lines 53-740) — that block holds extractor / body / accessor methods; mixing telemetry there muddles responsibilities (RESEARCH Q1).

---

### `framework/src/config/providers/app.rs` (config provider, boot-time read)

**Analog:** Self — the existing `AppConfig` struct + `from_env()` + `AppConfigBuilder` at `framework/src/config/providers/app.rs:1-98`

**Existing struct + `from_env` pattern** (`app.rs:1-25`):
```rust
use crate::config::env::{env, Environment};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub name: String,
    pub environment: Environment,
    pub debug: bool,
    pub url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            name: env("APP_NAME", "Ferro Application".to_string()),
            environment: Environment::detect(),
            debug: env("APP_DEBUG", true),
            url: env("APP_URL", "http://localhost:8080".to_string()),
        }
    }
}
```

**Existing builder pattern** (`app.rs:54-98`):
```rust
#[derive(Default)]
pub struct AppConfigBuilder {
    name: Option<String>,
    environment: Option<Environment>,
    debug: Option<bool>,
    url: Option<String>,
}

impl AppConfigBuilder {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    // ... other setters ...

    pub fn build(self) -> AppConfig {
        let default = AppConfig::from_env();
        AppConfig {
            name: self.name.unwrap_or(default.name),
            environment: self.environment.unwrap_or(default.environment),
            debug: self.debug.unwrap_or(default.debug),
            url: self.url.unwrap_or(default.url),
        }
    }
}
```

**Phase 184 additive changes** (CONTEXT D-12):
1. Add field to struct: `pub inline_budget_threshold_bytes: usize,`
2. Add to `from_env`: `inline_budget_threshold_bytes: env("INLINE_BUDGET_BYTES", 102_400usize),`
3. Add to builder struct: `inline_budget_threshold_bytes: Option<usize>,`
4. Add builder setter (consuming `mut self -> Self` per project conventions):
   ```rust
   pub fn inline_budget_threshold_bytes(mut self, bytes: usize) -> Self {
       self.inline_budget_threshold_bytes = Some(bytes);
       self
   }
   ```
5. Add to `build()`: `inline_budget_threshold_bytes: self.inline_budget_threshold_bytes.unwrap_or(default.inline_budget_threshold_bytes),`

**Notes for planner:**
- `env<T: FromStr>(name, default)` helper (`framework/src/config/env.rs:113-118`) supports `usize` via `FromStr` — no new helper needed.
- The `102_400usize` literal must include the `usize` suffix so type inference flows through `env<usize>(...)`.
- This is a purely additive field — no breaking change (CONTEXT D-12).

---

### `framework/src/lib.rs` (crate-root re-exports, re-export hub)

**Analog:** Self — existing `pub use` lines at `framework/src/lib.rs:60-113`

**Existing module declarations** (`framework/src/lib.rs:9-44`):
```rust
pub mod api;
pub mod app;
pub mod auth;
// ...
pub mod http;
// ...
pub mod validation;
```

**Existing flat re-export pattern** (`framework/src/lib.rs:60-113`):
```rust
pub use config::{
    env, env_optional, env_required, AppConfig, Config, Environment, LangConfig, LangConfigBuilder,
    ServerConfig,
};
// ...
pub use http::action::{
    ActionError, ActionKind, ActionResult, ActionResultExt, FlashVariant, IntoActionError,
};
pub use http::{
    bytes, json, request_host, text, validate_mime, validate_size, Cookie, CookieOptions,
    FormRequest, FromParam, FromRequest, HttpResponse, InertiaRedirect, MultipartForm,
    PaginationLinks, PaginationMeta, Redirect, Request, Resource, ResourceCollection, ResourceMap,
    Response, ResponseExt, SameSite, UploadedFile,
};
```

**Phase 184 additions:**
1. Add module declaration alongside existing `pub mod` lines (after `pub mod tenant;` or alphabetically):
   ```rust
   /// Request-scoped telemetry primitives — inline-vs-preload decisioning and a
   /// process-global ring-buffer for sampled time-series telemetry.
   pub mod telemetry;
   ```
2. Add re-export line in the flat re-export region (after `pub use http::action::...`):
   ```rust
   pub use telemetry::{Decision, RequestTelemetry, Sample};
   ```

**Notes for planner:**
- Do NOT re-export `InlineBudget` (it doesn't exist as a public type per CONTEXT D-02 — only the `pub(crate) InlineBudgetState` lives in the module). Resolves RESEARCH Q8 Risk 2 / Assumption A3.
- Match the doc-comment style of the existing `pub mod http;` line (`framework/src/lib.rs:22-23`): a single `///` summary above the module declaration.

---

### `framework/tests/telemetry_smoke.rs` (integration test, request-response)

**Analog:** `framework/tests/action_handler.rs` (real-Request-via-TCP-loopback template)

**Header + extern + imports pattern** (`framework/tests/action_handler.rs:1-26`):
```rust
//! Integration tests for the `#[action]` runtime helper.
//!
//! Exercises ... against simulated `Ok(())` and `Err(ActionError::...)` inputs,
//! asserting on:
//!
//! - 303 Location header (happy path)
//! - Success-side overrides via `req.flash(...)` / `req.redirect_to(...)` (D-02)
//! ...

extern crate ferro_rs as ferro;

use ferro::http::action::handle_action_result;
use ferro::{action, ActionError, ActionResult, FlashVariant, HttpResponse, Request, Response};

use hyper_util::rt::TokioIo;
use tokio::sync::oneshot;
```

**Real-Request constructor pattern** (`framework/tests/action_handler.rs:47-90` — verbatim copy):
```rust
async fn make_request() -> Request {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (tx, rx) = oneshot::channel::<Request>();
    let tx_holder = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));

    tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            let io = TokioIo::new(stream);
            // ... hyper service_fn that sends Request::new(req) via tx ...
        }
    });

    let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await.unwrap();
    tokio::spawn(async move { conn.await.ok() });

    let req = hyper::Request::builder()
        .uri("/test")
        .body(http_body_util::Empty::<bytes::Bytes>::new())
        // ... await response, return Request from oneshot ...
}
```

**Phase 184 test shape** (per CONTEXT line 318 — single handler exercising both primitives):
```rust
extern crate ferro_rs as ferro;

use ferro::{Decision, Request, RequestTelemetry, Sample};
use serde_json::json;

async fn make_request() -> Request {
    // Verbatim copy from framework/tests/action_handler.rs:47-90
}

#[tokio::test]
async fn inline_budget_and_telemetry_round_trip() {
    RequestTelemetry::reset();  // test-isolation per CONTEXT D-15
    let mut req = make_request().await;

    // Inline-budget below threshold → Inline.
    let decision = req.inline_budget("products_payload", 10_000, "/_/products.json");
    assert_eq!(decision, Decision::Inline);

    // Push past threshold → Preload + warning (state-machine assertion).
    let decision = req.inline_budget("products_payload", 200_000, "/_/products.json");
    assert!(matches!(decision, Decision::Preload(_)));

    // Telemetry record + snapshot round-trip.
    req.telemetry_record("render_latency", Sample::now(json!({"ms": 42})));
    let samples = RequestTelemetry::snapshot("render_latency", None);
    assert_eq!(samples.len(), 1);
}
```

**Notes for planner:**
- `serde_json::json!` works in tests because `serde_json` is in `[dependencies]`, not `[dev-dependencies]` (RESEARCH Q7, `framework/tests/api_resource_derive.rs:8`).
- No new dev-deps required for this test — `hyper-util`, `http-body-util`, `tokio` already present (`framework/Cargo.toml:78-83`).
- Use `RequestTelemetry::reset()` at the top of EVERY test that records — global state would otherwise leak (RESEARCH Pitfall 4).
- Mark `#[serial]` from `serial_test` if multiple tests in this file write to the same key/scope.

---

### `docs/src/the-basics/inline-budget-and-telemetry.md` (docs page)

**Analog:** `docs/src/the-basics/action-handlers.md` (Phase 180 — most recent docs page following current convention)

**Page structure pattern** (`docs/src/the-basics/action-handlers.md:1-130`):
```markdown
# Action Handlers

`#[action]` is the attribute macro for POST-style handlers that mutate state
and then redirect. ... [opening framing: what is it / when to use it]

## When to use `#[action]`

Use `#[action]` when the handler:

- Receives a POST ...
- Mutates state ...
- ...

## Quick example

\`\`\`rust
use ferro::{action, ActionError, ActionResult, Request};

#[action(redirect_to = "/dashboard/pages")]
pub async fn publish_by_id(req: Request, id: i64) -> ActionResult {
    let page = Page::find_by_id(id).await?
        .ok_or(ActionError::not_found("Page not found"))?;
    page.publish().await?;
    Ok(())
}
\`\`\`

## The macro shape

[detailed parameter table]

## Return type — `ActionResult`

[type signature + rationale]
```

**Section ordering template for Phase 184** (per CONTEXT D-14):
1. Frontmatter `# InlineBudget & RequestTelemetry` — one-paragraph framing.
2. `## When to use InlineBudget` — bulleted list (HTML pages with sizable inline payloads, decide inline vs preload).
3. `## Quick example` — the gestiscilo Phase 187 consumption snippet verbatim from CONTEXT lines 456-475.
4. `## The Decision enum` — both variants documented, match pattern shown.
5. `## Threshold configuration` — env var `INLINE_BUDGET_BYTES` (default 102_400), `AppConfig::builder().inline_budget_threshold_bytes(n)`.
6. `## Warning channel` — `tracing::warn!` fields + fire-once semantic.
7. `## When to use RequestTelemetry` — bulleted list (sampled time-series, operator dashboards).
8. `## Sample shape` — struct + `now` / `at` constructors.
9. `## Writer methods` — `req.telemetry_record(...)` and `req.telemetry_record_scoped(...)`.
10. `## Reader — snapshot` — `RequestTelemetry::snapshot(key, scope)` for operator handlers.
11. `## Scope conventions` — table of recommended `key:value` strings.
12. `## Lost-on-restart semantic` — explicit note from CONTEXT D-10.
13. `## End-to-end example` — single handler combining both primitives.

**Notes for planner:**
- Match the tone of `action-handlers.md`: scientific, minimal, no marketing voice (per global `CLAUDE.md` "Comments and documentation").
- Code blocks use ` ```rust ` fences (not `rust,ignore`) when the snippet compiles standalone, ` ```rust,ignore ` when it references undefined types (matches `action-handlers.md:23`).
- Use `ferro::` import paths in examples (matches `action-handlers.md:25`).

---

### `Cargo.toml` (workspace root — workspace version bump)

**Analog:** Self — `Cargo.toml:33-37`

**Current value** (`Cargo.toml:33-37`):
```toml
[workspace.package]
version = "0.2.43"
edition = "2021"
rust-version = "1.88.0"
license = "MIT"
```

**Phase 184 change** (CONTEXT D-13):
```toml
[workspace.package]
version = "0.2.44"
```

**Notes for planner:**
- Single-line edit. All `ferro-*` and `framework` crates inherit via `version.workspace = true` (RESEARCH Q8 Risk 3) — no fan-out edits needed.
- Verify post-edit: `cargo publish -p ferro-rs --dry-run` (Validation Dim 7).

---

### `docs/src/SUMMARY.md` (docs ToC — single link addition)

**Analog:** Self — "The Basics" section at `docs/src/SUMMARY.md:12-18`

**Existing section pattern** (`docs/src/SUMMARY.md:12-18`):
```markdown
# The Basics

- [Routing](the-basics/routing.md)
- [Middleware](the-basics/middleware.md)
- [Controllers](the-basics/controllers.md)
- [Action Handlers](the-basics/action-handlers.md)
- [Request & Response](the-basics/request-response.md)
```

**Phase 184 addition** (single line, append to "The Basics" — placement after "Request & Response" preserves the surface-area ordering: routing → middleware → controllers → actions → request/response → telemetry primitives):
```markdown
- [Inline Budget & Telemetry](the-basics/inline-budget-and-telemetry.md)
```

**Notes for planner:**
- One-line insert; do not reorder existing entries.
- Use mdBook-style relative-link syntax (no leading `./`), matching the rest of "The Basics."

---

### `framework/Cargo.toml` (OPTIONAL — `[dev-dependencies]` for tracing-test)

**Analog:** Existing `[dev-dependencies]` block at `framework/Cargo.toml:78-83`

**Phase 184 optional addition** (RESEARCH Q7 Option 1 / Assumption A4):
```toml
[dev-dependencies]
# ... existing dev-deps (serial_test, tempfile, hyper-util, http-body-util) ...
tracing-test = "0.2"  # optional: warning-emission assertion for SC-2
```

**Notes for planner:**
- Skip this addition if planner adopts Option 3 (state-machine-only assertion via `state.warned` flag). Either path satisfies SC-2 (RESEARCH Q7).
- Recommendation: skip the dep, assert on state directly. The fire-once invariant is provable from `state.warned.contains(key)` after each `decide()` call.

---

## Shared Patterns

### Per-request state via `Request::extensions` (used by `inline_budget.rs`)
**Source:** `framework/src/http/request.rs:20, 84-103`
**Apply to:** `framework/src/telemetry/inline_budget.rs` (`decide` function)
```rust
// Existing API — Phase 184 consumes it, does NOT extend it.
pub fn insert<T: Send + Sync + 'static>(&mut self, value: T);
pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T>;
pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T>;

// Idiom: fetch-or-create
if req.get::<InlineBudgetState>().is_none() {
    req.insert(InlineBudgetState::default());
}
let state = req.get_mut::<InlineBudgetState>().expect("just inserted");
```

### Process-global `OnceLock<DashMap>` registry (used by `request_telemetry.rs`)
**Source:** `ferro-bundle/src/lib.rs:69-85, 287-298` and `framework/src/middleware/rate_limit.rs:34-36`
**Apply to:** `framework/src/telemetry/request_telemetry.rs`
- Static `OnceLock<DashMap<K, V>>` at module top.
- Private accessor `fn store() -> &'static DashMap<K, V> { STORE.get_or_init(DashMap::new) }`.
- `#[cfg(test)] reset()` clears via `.clear()` if `get().is_some()`.

### Structured `tracing::warn!` (used by `inline_budget.rs`)
**Source:** `framework/src/http/action.rs:305-309, 355-359`
**Apply to:** `framework/src/telemetry/inline_budget.rs` (the fire-once warn site)
- Named structured fields with sigil rules: `%` for Display, `?` for Debug, bare for numeric.
- Message string is the LAST positional argument.
- Empty-string-not-omitted convention for `Option<String>` fields (route_pattern) — use `unwrap_or_default()` (RESEARCH Q4).

### Builder pattern (used by `app.rs` field addition)
**Source:** `framework/src/config/providers/app.rs:54-98`
**Apply to:** `framework/src/config/providers/app.rs` (`AppConfigBuilder::inline_budget_threshold_bytes` setter)
- Consuming `mut self -> Self`.
- `build()` uses `self.field.unwrap_or(default.field)` — backed by `AppConfig::from_env()`.
- Field stored as `Option<T>` in builder, materialized in `build()`.

### Test isolation reset (used by `request_telemetry.rs`)
**Source:** `ferro-bundle/src/lib.rs:287-298` (Phase 183)
**Apply to:** `RequestTelemetry::reset()` (D-15)
```rust
#[cfg(test)]
pub(crate) fn reset() {
    if let Some(r) = TELEMETRY_STORE.get() {
        r.clear();
    }
}
```
Call at the top of every test that records samples to prevent cross-test pollution (RESEARCH Pitfall 4).

### Real-Request integration-test scaffold (used by `tests/telemetry_smoke.rs`)
**Source:** `framework/tests/action_handler.rs:47-90`
**Apply to:** `framework/tests/telemetry_smoke.rs`
TCP-loopback `make_request()` helper — the canonical pattern for constructing a real `ferro::Request` in integration tests without a full app boot.

### `Config::get::<AppConfig>()` with hardcoded fallback (used by `inline_budget.rs`)
**Source:** RESEARCH Q5 + Pitfall 5 (CONTEXT D-04)
**Apply to:** `framework/src/telemetry/inline_budget.rs::decide`
```rust
let threshold = crate::Config::get::<crate::AppConfig>()
    .map(|c| c.inline_budget_threshold_bytes)
    .unwrap_or(102_400);
```
The `unwrap_or(102_400)` fallback is mandatory — synthetic Requests in unit tests bypass `Config::init()`.

---

## No Analog Found

None. Every Phase 184 file has a direct or near-direct analog in the existing codebase (RESEARCH summary: "The plan is mechanical assembly, not invention").

---

## Files Grouped by Plan (Wave / Dependency Order)

Mirrors RESEARCH "Suggested Plan Decomposition" (3-plan sequential structure).

### Plan 184-01 — Foundation: types + storage + config field
| File | Action |
|------|--------|
| `framework/src/config/providers/app.rs` | MODIFY — add `inline_budget_threshold_bytes` field + builder setter |
| `framework/src/telemetry/mod.rs` | NEW — module-root with `pub mod` + re-exports |
| `framework/src/telemetry/request_telemetry.rs` | NEW — `Sample`, `RequestTelemetry`, global store, `record`, `reset` |
| `framework/src/telemetry/inline_budget.rs` | NEW — `Decision` enum, `InlineBudgetState` struct (no `decide` yet — Plan 02) |
| `framework/src/lib.rs` | MODIFY — add `pub mod telemetry;` + `pub use telemetry::{Decision, RequestTelemetry, Sample};` |

**Inline unit tests in Plan 01:**
- `request_telemetry.rs`: `record_and_snapshot_round_trip`, `concurrent_record_no_deadlock`, `ring_buffer_caps_at_128`, `scope_isolation`.
- `app.rs`: `inline_budget_threshold_default_is_100kb`, `env_var_overrides_threshold`.

### Plan 184-02 — Request integration + decision state machine
| File | Action |
|------|--------|
| `framework/src/telemetry/inline_budget.rs` | MODIFY — implement `pub(crate) fn decide(...)` with lazy init, threshold read, fire-once warn |
| `framework/src/http/request.rs` | MODIFY — add `inline_budget`, `telemetry_record`, `telemetry_record_scoped` to second impl block (lines 742-777) |

**Inline unit tests in Plan 02:**
- `inline_budget.rs`: `decides_inline_below_threshold`, `decides_preload_above_threshold`, `warn_fires_once_per_key`, `different_keys_warn_independently`.

### Plan 184-03 — Integration test + docs + workspace bump
| File | Action |
|------|--------|
| `framework/tests/telemetry_smoke.rs` | NEW — both primitives via real Request (TCP loopback) |
| `docs/src/the-basics/inline-budget-and-telemetry.md` | NEW — docs page covering both primitives |
| `docs/src/SUMMARY.md` | MODIFY — add one entry under "The Basics" |
| `Cargo.toml` (workspace root) | MODIFY — bump version 0.2.43 → 0.2.44 |
| `framework/Cargo.toml` | OPTIONAL — add `tracing-test = "0.2"` to `[dev-dependencies]` (skip per planner recommendation) |

**Gate checks at Plan 03 end:**
- `cargo fmt --all -- --check`
- `cargo clippy --all --all-targets -- -D warnings`
- `cargo test --all-features`
- `cargo publish -p ferro-rs --dry-run`
- `cargo doc --no-deps -p ferro-rs`

---

## Metadata

**Analog search scope:** `framework/src/`, `framework/tests/`, `ferro-bundle/src/`, `docs/src/`, root `Cargo.toml`
**Files scanned:** 12 primary analog files (request.rs, action.rs, app.rs, env.rs, lib.rs, http/mod.rs, ferro-bundle/lib.rs, rate_limit.rs, action_handler.rs, action-handlers.md, SUMMARY.md, Cargo.toml)
**Pattern extraction date:** 2026-06-06
