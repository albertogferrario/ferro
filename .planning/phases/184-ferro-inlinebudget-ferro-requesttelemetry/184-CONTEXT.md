---
phase: 184
name: ferro::InlineBudget + ferro::RequestTelemetry
status: Ready for planning
gathered: 2026-06-06
discovered-by: jetskiadriatic startup-lifecycle audit (2026-06-06)
mode: auto
---

# Phase 184: `ferro::InlineBudget` + `ferro::RequestTelemetry` — Context

<domain>
## Phase Boundary

Ship two request-scoped framework primitives that any ferro application can use without opt-in beyond a single import line. Both live in the `framework` crate (published as `ferro-rs`) — the locked-during-discuss crate-location decision (D-01).

### (a) `InlineBudget`

Request-scoped accumulator that decides whether a chunk of bytes should be inlined into the HTML response or preloaded via `<link rel=preload>` based on cumulative byte cost per `key`.

```rust
let bytes = render_jsonld_blob();
let decision = req.inline_budget("jsonld_blob", bytes.len(), "/_/jsonld/v1.json");
match decision {
    Decision::Inline       => emit_inline_script(&bytes),
    Decision::Preload(url) => emit_preload_link(&url),
}
```

Per-request state lives in the existing `Request::extensions` type-map (`framework/src/http/request.rs:20`). Threshold is read from `AppConfig` (env-backed). A structured `tracing::warn!` fires exactly once per `(key, request)` when the threshold is crossed (Success Criterion 2). Subsequent inline_budget calls past the threshold for the same key flip to `Preload` silently.

### (b) `RequestTelemetry`

Per-key in-process ring buffer keyed by `(key, scope)` for sampled time-series telemetry.

```rust
// Writer side: inside a handler.
req.telemetry_record("render_latency", Sample::now(json!({ "ms": elapsed.as_millis() })));

// Reader side: operator dashboard handler aggregates across requests.
let samples = RequestTelemetry::snapshot("render_latency", Some("tenant:42"));
```

Process-global storage (`OnceLock<DashMap<(String, Option<String>), VecDeque<Sample>>>`) bounded at 128 samples per `(key, scope)` — oldest dropped on overflow (Success Criterion 3). Thread-safe via `DashMap` concurrency. Lost on process restart — documented in module docs.

### In scope

- New module `framework/src/telemetry/` with submodules `inline_budget.rs` and `request_telemetry.rs`.
- Public API surface:
  - `framework::InlineBudget`, `framework::Decision` (re-export of `inline_budget::Decision`), accessed via `req.inline_budget(key, bytes, fallback_url)`.
  - `framework::RequestTelemetry`, `framework::Sample`, accessed via `req.telemetry_record(key, sample)` and `RequestTelemetry::snapshot(key, scope)`.
- `AppConfig::inline_budget_threshold_bytes` field + `INLINE_BUDGET_BYTES` env var (default 102_400 — 100 KB).
- Re-exports at the workspace root `framework::lib.rs` so consumers do `use ferro_rs::{InlineBudget, Decision, RequestTelemetry, Sample};` (matches existing convention used for `HttpResponse`, `Request`, etc.).
- Unit tests for the in-process state machine (warning-fires-once, ring-buffer-overflow, thread-safety smoke test, scope isolation).
- Integration test: full handler that exercises both primitives via the real `Request` lifecycle.
- Module-level docs in `lib.rs` covering the lost-on-restart semantic, the threshold default, and the recommended consumption pattern.
- A new docs page `docs/src/the-basics/inline-budget-and-telemetry.md` covering both primitives, plus a `SUMMARY.md` link entry (D-14).

### Out of scope

- Per-key threshold override at the call site. v1 uses the global `AppConfig` threshold; a future phase can add `req.inline_budget_with_limit(...)` if a consumer surfaces real need (D-04 deferred path).
- Cross-request aggregation primitives (percentiles, rolling averages, etc.). The ring buffer is a raw sample store; downstream operator surfaces compute aggregates.
- Persistent telemetry — explicitly out per Success Criterion 3 framing ("lost-on-restart documented"). External telemetry systems (Prometheus, OpenTelemetry) belong outside this phase.
- A `Histogram` / `Counter` / `Gauge` typed API on top of `Sample`. Phase 184 ships only the raw `Sample` ring buffer; typed surfaces are a v2 question.
- `ferro-mcp` introspection coverage. Both primitives are runtime Rust APIs, not authoring-time scaffold surfaces. `application_info` already enumerates the env-var config; that's sufficient.
- Consumer adoption in gestiscilo. Cross-tracked as gestiscilo Phase 187 [FERRO REPO]; bumps `ferro-rs` after Phase 184 publishes.

</domain>

<decisions>
## Implementation Decisions

### D-01: Crate location — `framework` (the core), NOT a new crate

This is the locked-during-discuss decision flagged by the roadmap (Success Criterion 4). Three options were considered:

| Option | Surface shape | Verdict |
|--------|---------------|---------|
| (A) Module inside `framework` (the core crate, published as `ferro-rs`). Methods on `Request` impl + free function for snapshot. | `req.inline_budget(...)`, `req.telemetry_record(...)`, `RequestTelemetry::snapshot(...)` after `use ferro_rs::*;`. | ✅ **Adopted** |
| (B) New `ferro-telemetry` crate. Extension trait on `Request` that consumers import. | `use ferro_telemetry::RequestExt;` then `req.inline_budget(...)`. | ❌ Friction (extra import per file) + opt-in defeats the "every ferro app gets it" framing of the roadmap discovery |
| (C) Two separate crates (`ferro-inline-budget` + `ferro-telemetry`). | Even more import friction. | ❌ Over-engineered for two ~150-line primitives |

**Rationale for (A):**

1. **Conceptual coherence (PROJECT.md beauty dimension #3).** Both primitives are request-scoped extensions of the request lifecycle. `framework` already owns the `Request` type and its `extensions` type-map (`framework/src/http/request.rs:20-26, 84-101`). Adding two methods is the smallest possible surface extension. Putting them in a separate crate splits the request surface across two crates — that's the kind of duplicate-control-surface erosion that memory `feedback_no_duplicate_control_surface.md` calls out.

2. **Roadmap framing.** The roadmap phrases the gray area as "extension trait in `ferro-core` vs new `ferro-telemetry` crate." There is no `ferro-core` crate today — the core IS `framework`. So Option (A) is the literal reading of "extension trait in ferro-core" with the terminology corrected.

3. **Discovery framing.** The roadmap discovery note says "any ferro app shipping HTML responses can benefit." Opt-in via `Cargo.toml` (Option B) directly contradicts "any ferro app" — it makes the primitive invisible to apps that don't enable a feature flag.

4. **Future-split is cheap.** If a real consumer (or a future v2.0 split) needs `ferro-telemetry` as a separate crate, the move is mechanical: relocate `framework/src/telemetry/` into a new crate, add an extension trait re-export at `framework::lib.rs`. Per memory `feedback_breaking_changes_v12_ai.md`: pre-1.0, breaking changes acceptable. The cost of moving from (A) → (B) later is much smaller than the cost of starting at (B) and discovering every ferro consumer needed it on by default.

5. **Bootstrap friction avoided.** New crates require the publish-token bootstrap flow (per memory `project_ferro_publish_token_scoping.md`: first publish must be manual from local terminal because CI token has `publish-update` only, not `publish-new`). Phase 183 just exercised this for `ferro-bundle`; doing it again for `ferro-telemetry` adds operational drag without architectural benefit at this scale.

6. **Workspace footprint.** Ferro already has 25 crates in the workspace. Each addition costs review time and adds a node to the dependency graph CI must orchestrate. Two ~150-line primitives don't earn a new crate per the "Compressive" beauty dimension (small inputs produce disproportionate outputs).

### D-02: Public API — methods on `Request` impl + static snapshot function

```rust
// In framework/src/http/request.rs (impl Request)
impl Request {
    pub fn inline_budget(&mut self, key: &str, bytes: usize, fallback_url: &str) -> Decision { ... }
    pub fn telemetry_record(&mut self, key: &str, sample: Sample) { ... }
}

// In framework/src/telemetry/request_telemetry.rs
impl RequestTelemetry {
    pub fn snapshot(key: &str, scope: Option<&str>) -> Vec<Sample> { ... }
    pub fn keys() -> Vec<(String, Option<String>)> { ... }
    pub fn clear() { ... }       // operator surfaces (e.g., post-deploy reset)
    #[cfg(test)] pub(crate) fn reset() { ... }  // test isolation
}
```

`InlineBudget` is NOT exposed as a constructed type — it lives only in the request's `extensions` map. The user never types `InlineBudget` (consistent with how request state types in ferro are usually hidden behind method calls). The `Decision` enum IS public because consumers match on it.

`RequestTelemetry` IS exposed as a unit-like struct that namespaces the static `snapshot` / `keys` functions. This is the same shape as `ferro-bundle::Bundle::serve` (also a static-ish dispatcher).

### D-03: `Decision` enum — locked variants, `String` for the preload URL

```rust
pub enum Decision {
    Inline,
    Preload(String),
}
```

`String` (not `&'static str` or `Cow<'static, str>`) because the URL is usually constructed at request time from the call site's `fallback_url: &str` parameter. Allocation cost per decision is negligible. The enum's bytes-per-instance is dominated by the String anyway.

### D-04: Threshold — global default 100 KB via `AppConfig`, no per-key override in v1

- Default: `102_400` bytes (100 KiB).
- Configurable via env var `INLINE_BUDGET_BYTES` (parsed as `usize`).
- Configurable via `AppConfig::builder().inline_budget_threshold_bytes(N)` for programmatic override (matches `AppConfig::builder()` pattern at `framework/src/config/providers/app.rs:54-60`).
- No per-key override in v1. If a consumer needs one (e.g., critical-CSS gets a different budget than JSON-LD), the future path is `req.inline_budget_with_limit(key, bytes, fallback_url, limit)` — additive, no break.

**100 KB rationale:** the discovery context (gestiscilo `inject_config_and_products` blowing past 200 KB) sets the upper-pain bound. 100 KB trips BEFORE the operator-visible pain manifests, giving the warning channel time to surface the problem in observability before it becomes a perf regression report. Round numbers are easy to remember in operator post-mortems.

### D-05: `req.inline_budget(key, bytes, fallback_url)` — caller passes URL at the decision point

The caller knows the fallback URL at decision time (it's the URL the same byte payload would be served from if NOT inlined). Passing it upfront keeps the API to a single call:

```rust
let decision = req.inline_budget("sdk_bundle", BUNDLE.len(), "/embed/v1.js");
```

Rejected alternatives:
- Returning `Decision::Preload(fn -> String)` for late URL construction — adds closure ceremony for negligible benefit.
- Pre-registering URLs via `req.set_inline_fallback(key, url)` — separates concerns that belong together (key, payload size, fallback URL all conceptually relate to the same "inline-or-link?" question).

### D-06: Warning — `tracing::warn!`, fires exactly once per `(key, request)`

Channel: `tracing::warn!` (matches the existing pattern in `framework/src/http/action.rs:305, 355`). Structured fields:

```rust
tracing::warn!(
    key = key,
    cumulative_bytes = state.cumulative,
    threshold_bytes = threshold,
    fallback_url = fallback_url,
    route_pattern = req.route_pattern().as_deref().unwrap_or(""),
    "inline_budget: threshold crossed; flipping to Preload",
);
```

Fire-once tracking: per-key `bool` in the request-scoped `InlineBudgetState` (the struct stored in `Request::extensions`). The state machine:

1. First call past threshold for a given key → emit warning, set `warned = true`, return `Preload`.
2. Subsequent calls past threshold for the SAME key → no warning, return `Preload`.
3. Different key past its own threshold → its own warning fires once.

State is dropped with the `Request`. Next request starts fresh.

### D-07: `Sample` shape — concrete struct, `serde_json::Value` payload

```rust
pub struct Sample {
    pub recorded_at: std::time::SystemTime,  // wall-clock, serializable
    pub value: serde_json::Value,            // caller-chosen payload
}

impl Sample {
    pub fn now(value: serde_json::Value) -> Self { ... }
    pub fn at(when: SystemTime, value: serde_json::Value) -> Self { ... }
}
```

Rationale for `serde_json::Value`:
- Round-trips through serde (Success Criterion 3).
- Pervasive in ferro already (`HttpResponse::json`, `JsonUi::render`, etc.).
- No generic-parameter cascade through `Sample`, `VecDeque<Sample>`, the `DashMap`, the snapshot return type.

Rejected alternatives:
- Generic `Sample<T: Serialize + DeserializeOwned + Send + Sync>` — every consumer locks the type at compile time; snapshot consumers can't aggregate across keys with heterogeneous T.
- Concrete `f64` / `i64` numeric value — too narrow; ferro consumers will want to record structured payloads (e.g., `{ "ms": 42, "rows": 1024 }`).
- `Box<dyn TelemetrySample>` trait object — runtime polymorphism with no clear win over `serde_json::Value`.

### D-08: Ring buffer capacity — hardcoded 128 samples per `(key, scope)`

Memory bound: 128 × sizeof(Sample) ≈ 128 × (~200 bytes typical `serde_json::Value` + 16 bytes SystemTime) ≈ ~28 KB per `(key, scope)` worst case. For a busy app with 50 distinct `(key, scope)` pairs that's ~1.4 MB — comfortable.

128 is chosen because:
- Power of 2 (cheap arithmetic).
- Enough to see meaningful patterns in an operator snapshot.
- Small enough that "lost on restart" isn't a regression (long histories belong in external systems anyway).

No per-key capacity override in v1. If a consumer needs more headroom, they can call `snapshot()` regularly and aggregate downstream. The future path is `RequestTelemetry::set_capacity(key, n)` if real need surfaces — additive, no break.

### D-09: `scope` parameter — `Option<&str>`, caller-defined convention

```rust
req.telemetry_record("render_latency", Sample::now(json!({"ms": 42})));
// scope is None — recorded into the (key="render_latency", scope=None) bucket.

req.telemetry_record_scoped("render_latency", Some("tenant:42"), Sample::now(json!({"ms": 42})));
// scope is Some("tenant:42") — separate bucket from None.
```

Two writer methods to avoid forcing every caller to type `None`:

```rust
impl Request {
    pub fn telemetry_record(&mut self, key: &str, sample: Sample);
    pub fn telemetry_record_scoped(&mut self, key: &str, scope: Option<&str>, sample: Sample);
}
```

Reader (operator side) uses one method:

```rust
RequestTelemetry::snapshot(key: &str, scope: Option<&str>) -> Vec<Sample>
```

`Option<&str>` (not an enum like `Scope::{Global, Tenant(String), Route(String)}`) because callers encode their own scoping convention. `"tenant:42"`, `"route:/api/products"`, `"region:eu-west-1"` — all valid. An enum would calcify a vocabulary too early.

### D-10: Storage — process-global `OnceLock<DashMap<(String, Option<String>), VecDeque<Sample>>>`

Same pattern as `ferro-bundle`'s D-02 (process-global concurrent registry). Reasons:
- Reader side `RequestTelemetry::snapshot(key, scope)` is a static call with no `&self` — global state is the only viable shape.
- `DashMap` is already a transitive dep in multiple ferro-* crates.
- `VecDeque` gives O(1) push-back and pop-front for the ring-buffer semantics.

`SystemTime` chosen over `Instant` for `Sample.recorded_at` because operator dashboards consume snapshots across processes / restarts — wall-clock comparability matters.

Test isolation via `RequestTelemetry::reset()` (visible in `#[cfg(test)]` only). Matches Phase 183 D-13.

### D-11: Module placement — `framework/src/telemetry/`

```
framework/src/telemetry/
    mod.rs                  // module-level docs + re-exports
    inline_budget.rs        // Decision enum + InlineBudgetState + threshold reading
    request_telemetry.rs    // Sample struct + RequestTelemetry global storage + snapshot
```

Re-exports at `framework/src/lib.rs`:

```rust
pub use telemetry::{Decision, InlineBudget, RequestTelemetry, Sample};
```

Request impl methods live in `framework/src/http/request.rs` alongside the existing `insert<T>` / `get<T>` extensions API. Method bodies are thin — they call into `crate::telemetry::*` for the actual state work, keeping `request.rs` from growing into a god module.

### D-12: AppConfig wiring — additive field, env-backed, no breaking change

`framework/src/config/providers/app.rs`:

```rust
pub struct AppConfig {
    pub name: String,
    pub environment: Environment,
    pub debug: bool,
    pub url: String,
    pub inline_budget_threshold_bytes: usize,  // NEW — default 102_400
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            // ... existing fields ...
            inline_budget_threshold_bytes: env("INLINE_BUDGET_BYTES", 102_400usize),
        }
    }
}
```

`AppConfigBuilder` gets a parallel `.inline_budget_threshold_bytes(n: usize)` setter for programmatic override.

### D-13: Publish — single workspace bump, ships via existing WAVE2

Workspace version `0.2.43` (post-Phase 183) bumps to `0.2.44` as part of the merge cycle (D-13 mirrors Phase 183 D-13). No new wave entry — `ferro-rs` is already in `WAVE2_CRATES` at `.github/workflows/publish.yml:274` and ships on every workspace bump. Per memory `feedback_friction_loop_release_cadence.md`: single publish at end of release loop; gestiscilo Phase 187 bumps after merge.

### D-14: Docs — new page `docs/src/the-basics/inline-budget-and-telemetry.md`

CLAUDE.md mandates docs updates when framework changes. Both primitives are part of the public ferro Rust API (consumer-facing), not internal implementation. The page covers:

1. **InlineBudget** — when to use it, the `req.inline_budget(...)` API, the `Decision` match pattern, the threshold env var, the warning channel.
2. **RequestTelemetry** — Sample shape, writer methods (`record` vs `record_scoped`), `snapshot` for operator surfaces, lost-on-restart semantic, ring-buffer capacity.
3. **End-to-end example** — a single handler that uses both: decide inline-vs-preload for a payload, record render latency telemetry, return the response.
4. **Scoping conventions** — short table of recommended `scope` string formats (`tenant:N`, `route:/path`, `region:X`).

Sibling phase 183 (which shipped a Rust-only API too) declined a docs page because the API was niche (immutable byte blobs). Phase 184's primitives are general-purpose; consumer adoption is on the gestiscilo Phase 187 path and other ferro apps. Docs page IS required.

`docs/src/SUMMARY.md` gets a single new link entry under "The Basics."

### D-15: Test isolation — `#[cfg(test)] reset()` clears both registries

`RequestTelemetry::reset()` clears the process-global `DashMap`. Each unit test that records samples calls `reset()` at the top. Integration tests under `tests/` run in separate binaries (Cargo default) — the OS already isolates them. Same pattern as Phase 183 D-13.

`InlineBudget` per-request state is dropped with the `Request` automatically — no global reset needed. Unit tests construct a fresh `Request` per test case.

### Claude's Discretion

- Exact `tracing` field names and message wording — the planner picks; D-06 documents the required structured-field set, the prose is flexible.
- Whether `Sample::now(...)` and `Sample::at(when, ...)` are the only constructors, or whether to add `Sample::from_value(value)` defaulting to `SystemTime::now()`. The planner picks.
- Exact docs page section ordering and sub-headers (D-14).
- Whether to split unit tests across multiple `#[test]` functions or use `#[rstest]`-style parameterization. (Ferro's test corpus uses both styles; planner picks the natural fit.)
- The exact form of the integration test handler — whether it exercises both primitives in one handler or two parallel handlers under `tests/telemetry_smoke.rs`.

### Folded Todos

None — `gsd-tools todo match-phase 184` returned `todo_count: 0`. The roadmap entry is the sole source of scope.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Source — roadmap and prior decisions
- `.planning/ROADMAP.md` §`Phase 184: ferro::InlineBudget + ferro::RequestTelemetry` (lines 1999-2017) — locked API shape, success criteria 1–5, discovery context, cross-tracked gestiscilo Phase 187, the crate-location gray area called out for discuss.
- `.planning/PROJECT.md` — pre-1.0 status, v1.0 criteria, conceptual coherence as a v1.0 blocker, four beauty dimensions.
- `.planning/phases/182-ferro-json-ui-data-lazy-hero-runtime-primitive/182-CONTEXT.md` — sibling phase (same v12.2 milestone). Establishes the "ferro-* primitive surfaced by jetskiadriatic audit" framing, the auto-mode CONTEXT.md template, the single-publish-at-end cadence.
- `.planning/phases/183-ferro-bundle-capability-new-crate/183-CONTEXT.md` — sibling phase that exercised the "process-global registry + `#[cfg(test)] reset()`" pattern (Phase 184 D-10 / D-15 mirror this). Also the publish-bootstrap reference (Phase 184 avoids this by NOT creating a new crate — D-01).

### Source — workspace integration patterns
- `Cargo.toml` (workspace root) — `[workspace.package.version]` currently `0.2.43` (post-Phase 183). Phase 184 bumps to `0.2.44`.
- `.github/workflows/publish.yml` §211 `WAVE1A_CRATES`, §246 `WAVE1B_CRATES`, §274 `WAVE2_CRATES`. Phase 184 modifies `framework` (published as `ferro-rs` in WAVE2). No new wave entry.
- `framework/Cargo.toml` — `ferro-rs` package metadata, dependency list. Phase 184 may add `dashmap = "6"` if not already a direct dep (it's a transitive dep today via ferro-* crates).

### Source — framework HTTP and request types
- `framework/src/http/request.rs:11-26` — `Request` struct: `parts`, `body`, `params`, `extensions: HashMap<TypeId, Box<dyn Any + Send + Sync>>`, `route_pattern`, `action_overrides`. The `extensions` field IS the storage for `InlineBudgetState`.
- `framework/src/http/request.rs:84-101` — `insert<T>`, `get<T>`, `get_mut<T>` extension API. Phase 184's `req.inline_budget(...)` uses these internally to fetch-or-create the per-request `InlineBudgetState`.
- `framework/src/http/request.rs:74-82` — `route_pattern()` accessor. Phase 184's structured warning includes the route pattern (D-06).
- `framework/src/http/action.rs:305, 355` — existing `tracing::warn!` call sites in ferro. Phase 184's warning follows the same shape.
- `framework/src/lib.rs` — re-exports `HttpResponse`, `Request`, `FromRequest`, etc. Phase 184 adds re-exports for `Decision`, `InlineBudget`, `RequestTelemetry`, `Sample`.

### Source — config provider
- `framework/src/config/providers/app.rs:1-60` — `AppConfig` struct + `from_env()` + `AppConfigBuilder` pattern. Phase 184 adds the `inline_budget_threshold_bytes: usize` field + env-var reader + builder setter.
- `framework/src/config/env.rs` — `env<T>(name, default)` helper used by `from_env()`. Phase 184 uses it for `INLINE_BUDGET_BYTES`.

### Project conventions (CLAUDE.md)
- `CLAUDE.md` (project root) — "Run fmt + clippy + tests before every commit" (validate gate).
- `CLAUDE.md` (project root) — "Always update docs when framework changes" — drives D-14.
- `CLAUDE.md` (project root) — "Project-agnostic crates" rule. Both primitives are generic — no tenant identity, no hardcoded app strings. ✓
- `CLAUDE.md` (project root) — "When adding a new crate to the workspace, always add it to `.github/workflows/publish.yml`" — N/A for Phase 184 (extends existing `framework` crate, not adding new).
- `CLAUDE.md` (project root) — "Concrete types not interface{} or any" — `Sample` is concrete; `Decision` is a concrete enum; the only `dyn Any` is in the existing `Request::extensions` map which Phase 184 reuses.

### Project memory (referenced for behavior, not committed in the repo)
- Memory `feedback_no_duplicate_control_surface.md` — before adding a new annotation/config knob, check if an existing ferro layer already decides that thing. Verified: no existing primitive in `framework` does inline-vs-preload decisioning or per-request ring-buffer telemetry. Phase 184 fills a genuine gap.
- Memory `feedback_friction_loop_release_cadence.md` — single publish at end of release loop; gestiscilo Phase 187 bumps after merge (drives D-13).
- Memory `feedback_breaking_changes_v12_ai.md` — pre-1.0 breaking changes acceptable; underpins D-01's "future-split is cheap" argument.
- Memory `project_ferro_publish_token_scoping.md` — CI publish token has `publish-update` only, not `publish-new`. Avoided in Phase 184 because no new crate is created (D-01).

### Discovery context
- Roadmap Phase 184 discovery note (2026-06-06): surfaced during the jetskiadriatic startup-lifecycle audit. gestiscilo `inject_config_and_products` unconditionally inlines up to 100 products into every HTML response — fat tenants blow past 200 KB, paid as HTML-parse cost on every page load. The right primitive (decide inline vs preload based on measured bytes) is request-scoped + framework-level. Cross-tracked as gestiscilo Phase 187 [FERRO REPO].

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`Request::extensions` type-map** (`framework/src/http/request.rs:20, 84-101`). Existing storage mechanism for arbitrary per-request state. `InlineBudgetState` (a private struct holding `cumulative: HashMap<String, usize>` + `warned: HashSet<String>`) is inserted into this map on first `req.inline_budget(...)` call and fetched-or-created on subsequent calls. Zero new infrastructure required.
- **`tracing::warn!` macro** — already pervasive in the framework crate (`http/action.rs`). Structured field syntax `tracing::warn!(field1 = value1, field2 = value2, "message")` is the locked pattern. Phase 184's warning channel reuses this verbatim.
- **`AppConfig::from_env()` + `AppConfigBuilder`** (`framework/src/config/providers/app.rs:16-46, 54-60`). Existing config provider with env-var-backed defaults + programmatic builder override. Phase 184 adds one field; no architectural change.
- **`env<T>(name, default)` helper** (used in `app.rs:20-23`). Standard ferro pattern for env-var-backed config. Phase 184 uses it for `INLINE_BUDGET_BYTES`.
- **`DashMap` from sibling phases** — `ferro-bundle` (Phase 183) just adopted `dashmap = "6"` as a direct dep. Phase 184 follows the same pattern. Already a transitive dep in multiple ferro-* crates.
- **`#[cfg(test)] reset()` pattern** — established in Phase 183 (`ferro-bundle` D-13). Phase 184 reuses the pattern verbatim for `RequestTelemetry::reset()`.

### Established Patterns

- **One `Error` enum per crate with `thiserror` derive** — Phase 184 does NOT need a new error enum. `req.inline_budget(...)` returns `Decision` (infallible). `req.telemetry_record(...)` returns `()` (infallible — overflow drops oldest sample). `RequestTelemetry::snapshot(...)` returns `Vec<Sample>` (empty Vec on no-such-key, not an error).
- **Re-exports at `framework::lib.rs` flatten the import path** — consumers do `use ferro_rs::{Request, HttpResponse}`, not `use ferro_rs::http::Request`. Phase 184 follows: `use ferro_rs::{Decision, RequestTelemetry, Sample};`.
- **Documentation lives in `//!` module headers + `///` item docs** — `cargo doc` is the single source for the public Rust API surface. Phase 184's `framework/src/telemetry/mod.rs` carries the module-level overview; each item gets a one-line `///` plus a usage example.
- **Tests colocated with the implementation** — `#[cfg(test)] mod tests` at the bottom of each `.rs` file. Integration tests under `tests/` directory at the crate root. Phase 184 follows: unit tests in `inline_budget.rs` / `request_telemetry.rs`, integration smoke test in `framework/tests/telemetry_smoke.rs`.

### Integration Points

- **`Request` impl block** (`framework/src/http/request.rs:53+`). Phase 184 adds `inline_budget`, `telemetry_record`, `telemetry_record_scoped` methods to this block. Method bodies are thin — they delegate to `crate::telemetry::*`.
- **`framework::lib.rs` re-exports** — Phase 184 adds four new symbols to the existing re-export list. No new feature flag (the primitives are unconditionally available — matches the roadmap framing "any ferro app").
- **`AppConfig` consumers** — every ferro app reads `AppConfig::from_env()` or `AppConfig::default()` at boot. The new `inline_budget_threshold_bytes` field gets the default value via env var; no consumer code change required to adopt the default behavior. Custom thresholds via builder.
- **`ferro-mcp` introspection** — `application_info` already lists env-var-backed config keys. After Phase 184, it'll list `INLINE_BUDGET_BYTES` automatically (assuming the introspection covers all `AppConfig` fields — researcher should verify and add the field to introspection if it doesn't auto-include).

</code_context>

<specifics>
## Specific Ideas

- The roadmap's example `key`s — `"products_payload"`, `"jsonld_blob"`, `"critical_css"` — are illustrative. Real-world adoption (gestiscilo Phase 187) will pick its own conventions. The primitive doesn't enforce a naming scheme; the README example uses the roadmap's keys verbatim so reviewers see continuity.
- The discovery framing ("inject_config_and_products unconditionally inlines up to 100 products") is the canonical pain point. The Phase 184 demo handler in the docs page should walk through that exact scenario: a payload that fits under 100 KB for thin tenants (`Decision::Inline`) but flips for fat tenants (`Decision::Preload("/_/products.json")`). Operator dashboards then read `RequestTelemetry::snapshot("products_payload_size", ...)` to monitor the flip rate.
- The 100 KB default (`102_400` bytes) is round enough to remember without being so round that it looks arbitrary. The exact pivot point (95 KB? 110 KB?) doesn't matter at this resolution — the operator-visible signal is whether the warning fires for a given tenant, not the exact byte count.
- `Sample::now(value)` is the 99% case (record the value at "right now"). `Sample::at(when, value)` exists for the rare case where a writer batches samples and needs to backfill timestamps from an external source.
- The `"key:value"` scope convention (`"tenant:42"`, `"route:/api/products"`) is a recommendation, not a constraint. The ring buffer keys by the `Option<String>` exactly as given; consumers pick whatever string they want. The recommendation lives in the docs page (D-14) and the module docs.
- The fire-once warning state lives in the per-request `InlineBudgetState`, not in a global. That means: 10 simultaneous requests all crossing the threshold on the same key will each emit one warning (10 total). Not one global warning suppressing nine others. The roadmap's Success Criterion 2 says "once per `key` per request" — verbatim follows this design.
- The structured warning's `route_pattern` field uses `req.route_pattern()` which returns `Option<String>` (per `framework/src/http/request.rs:74-82`). When the request hasn't yet been matched to a route (e.g., middleware running before the router) the field is empty string, NOT omitted — easier to grep in log aggregation.

</specifics>

<deferred>
## Deferred Ideas

- **Per-key threshold override at the call site** — `req.inline_budget_with_limit(key, bytes, fallback_url, limit)`. Additive, no break. Ship if a real consumer needs heterogeneous budgets per key.
- **Persistent / cross-process telemetry** — explicitly out of Phase 184 per Success Criterion 3 framing. External systems (Prometheus, OpenTelemetry, custom DB sinks) are the right answer for cross-process aggregation.
- **Typed telemetry surfaces** — `Counter`, `Gauge`, `Histogram` on top of `Sample`. v2 of telemetry. Phase 184 ships only the raw `Sample` ring buffer.
- **Cross-request aggregation primitives** (rolling p50 / p99, etc.) — operator-side concern; lives in whatever consumes `RequestTelemetry::snapshot()`. Could be a future `ferro-telemetry-aggregations` crate if real demand surfaces.
- **`Scope` enum with named variants** — `Scope::Tenant(String)`, `Scope::Route(String)`, etc. Calcifies a vocabulary too early. Phase 184 ships `Option<&str>` and lets callers pick conventions. If a clear winning vocabulary emerges across 3+ ferro consumers, promote it.
- **`RequestTelemetry` extraction to a `ferro-telemetry` crate** — re-evaluate once a second consumer (beyond gestiscilo Phase 187) emerges, OR once the primitives' surface area grows beyond ~200 lines, OR once the v2.0 crate-split happens. Memory `feedback_breaking_changes_v12_ai.md` keeps this future move open.
- **Bootstrap endpoint fallback referenced in roadmap** — the roadmap mentions "bootstrap-endpoint fallback" as part of gestiscilo Phase 187. The bootstrap-endpoint pattern is a consumer-side concern (gestiscilo emits the preload URL pointing at a tenant-served endpoint). Ferro provides the primitive; the endpoint is gestiscilo's wiring.
- **Histogram of the cumulative-bytes distribution per key** — a derived metric the operator surface could compute from `RequestTelemetry::snapshot()`. Lives downstream.
- **Auto-discover and surface `RequestTelemetry::keys()` via `ferro-mcp`** — could expose a tool like `request_telemetry_keys` for agents to introspect what's being recorded. Defer until a consumer asks; not blocking Phase 184.

### Reviewed Todos (not folded)

None — `gsd-tools todo match-phase 184` returned `todo_count: 0`. No backlog items to consider.

</deferred>

---

## Discovery Transcript (preserved from roadmap)

Roadmap Phase 184 discovery note, verbatim:

> Discovery: surfaced during the 2026-06-06 jetskiadriatic startup-lifecycle audit. gestiscilo `inject_config_and_products` unconditionally inlines up to 100 products into every HTML response — fat tenants can blow past 200 KB, paid as HTML-parse cost on every page load. The right primitive (decide inline vs preload based on measured bytes) is request-scoped + framework-level, not gestiscilo-specific. Same elevation rule as `feedback_ferro_first_primitives.md`: cross-cutting capabilities go in ferro by default rather than waiting for N consumers. Cross-tracked as gestiscilo Phase 187 [FERRO REPO].

### Concrete consumer impact (gestiscilo Phase 187)

Pre-Phase-184 pattern (gestiscilo today):

```rust
// inject_config_and_products renders all products inline, every time:
let products_json = serde_json::to_string(&products)?;
let html = format!(
    "<script id='__products' type='application/json'>{}</script>{}",
    products_json,
    body,
);
// On a fat-tenant page with 100 products * 2 KB each ≈ 200 KB, every page load.
```

Post-Phase-184 pattern (gestiscilo Phase 187 adopts):

```rust
let products_json = serde_json::to_string(&products)?;
let html = match req.inline_budget(
    "products_payload",
    products_json.len(),
    "/_/bootstrap/products.json",  // tenant-served fallback endpoint
) {
    Decision::Inline => format!(
        "<script id='__products' type='application/json'>{}</script>{}",
        products_json, body,
    ),
    Decision::Preload(url) => format!(
        r#"<link rel="preload" as="fetch" href="{url}" crossorigin>{body}"#,
    ),
};

// Operator dashboard reads:
let recent = RequestTelemetry::snapshot("products_payload_size", Some("tenant:42"));
// → Vec<Sample> showing the byte-size distribution for tenant 42 over the last 128 requests.
```

The thin-tenant case keeps the synchronous inline path (no extra network round-trip). The fat-tenant case offloads to the tenant-served bootstrap endpoint, with the browser pre-warming via `<link rel=preload>`. The structured warning fires once per request the first time a fat-tenant page crosses the threshold — operator observability picks up the flip rate without any per-tenant tuning.

---

*Phase: 184-ferro-inlinebudget-ferro-requesttelemetry*
*Context gathered: 2026-06-06 (--auto)*
