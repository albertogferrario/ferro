## RESEARCH COMPLETE

---
phase: 184
name: ferro::InlineBudget + ferro::RequestTelemetry
researched: 2026-06-06
domain: framework crate — request-scoped primitives (inline-vs-preload decisioning + in-process telemetry ring buffer)
confidence: HIGH
---

# Phase 184: `ferro::InlineBudget` + `ferro::RequestTelemetry` — Research

## Summary

Phase 184 is a thin additive surface on top of well-established framework patterns. All eight tool/dep dependencies the implementation needs are already direct deps of the `framework` crate (`tracing`, `dashmap`, `serde_json`, `serde`, `thiserror`, `bytes`, `tokio`, `chrono`). The two primitives compose with three existing mechanisms verbatim:

- **Per-request state** lives in `Request::extensions` (TypeId-keyed type-map, `framework/src/http/request.rs:20, 84-103`).
- **Process-global registry** mirrors `ferro-bundle`'s `OnceLock<DashMap<...>>` pattern (`ferro-bundle/src/lib.rs:69-85, 287-298`) and the framework's own `rate_limit.rs:34-36`.
- **AppConfig** values are read via `Config::get::<AppConfig>()` from the global config repository populated at `Config::init()` (`framework/src/config/mod.rs:64-86`, `framework/src/config/repository.rs:60-63`).

No architectural innovation is needed. The plan can move directly to API surface implementation following these patterns.

**Primary recommendation:** Implement as a 3-plan decomposition: (1) Config + Telemetry module skeleton + Sample, (2) InlineBudget + Request methods, (3) Docs + integration test + workspace bump. Each plan ships independently green.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| InlineBudget decision (inline vs preload) | API/Backend (Request handler) | — | Decided server-side from a measured payload byte count; never crosses the wire |
| InlineBudget threshold config | API/Backend (AppConfig) | env vars | Read once at boot via `Config::get::<AppConfig>()`; per-request lookup is O(1) |
| InlineBudget per-request state | API/Backend (Request::extensions) | — | Cumulative-bytes accumulator + warned-set must die with the Request |
| Inline-vs-preload warning emission | API/Backend (`tracing::warn!`) | log aggregator | Operator observability layer; same channel as `action.rs:305, 355` |
| RequestTelemetry writer | API/Backend (Request method) | — | Writer is always inside a handler with `&mut self` Request access |
| RequestTelemetry storage | Process-global (OnceLock<DashMap>) | — | Reader is `RequestTelemetry::snapshot()` static fn; only viable as process-global |
| RequestTelemetry snapshot reader | API/Backend (operator handler) | external dashboard | Operator handler calls snapshot, returns JSON for dashboard |

## Standard Stack

### Core (already direct deps of `framework`)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `dashmap` | `6` | Process-global concurrent map | [VERIFIED: `framework/Cargo.toml:63`, `framework/src/middleware/rate_limit.rs:26`] Direct dep; same pattern ferro-bundle uses |
| `tracing` | `0.1` | Structured warning channel | [VERIFIED: `framework/Cargo.toml:76`] Direct dep; matches `action.rs:305, 355` |
| `serde_json` | `1` (preserve_order) | `Sample.value` payload type | [VERIFIED: `framework/Cargo.toml:33`] Direct dep |
| `serde` | `1` (derive) | `Sample` derives | [VERIFIED: `framework/Cargo.toml:32`] Direct dep |
| `std::sync::OnceLock` | stdlib | Lazy global init | [VERIFIED: ferro-bundle/src/lib.rs:36, 69-71] Same pattern |
| `std::collections::VecDeque` | stdlib | Ring buffer per `(key, scope)` | O(1) push-back + pop-front semantics |
| `std::time::SystemTime` | stdlib | Wall-clock sample timestamps | [VERIFIED: CONTEXT D-10] Cross-process comparability |

### Supporting (no new deps required)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `std::any::TypeId` | stdlib | Key into `Request::extensions` | Already in use at `request.rs:7, 87-103` |
| `thiserror` | `1.0` | (Not needed — both primitives infallible) | Skip per CONTEXT `<code_context>` "Established Patterns" |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `serde_json::Value` for Sample payload | Generic `Sample<T>` | [CITED: CONTEXT D-07] Locked to Value — heterogeneous downstream aggregation requires it |
| `Option<&str>` scope | `Scope::{Global, Tenant(String), Route(String)}` enum | [CITED: CONTEXT D-09] Locked to Option<&str> — calcifies vocabulary too early |
| Per-request InlineBudget shared via middleware injection | TypeId extensions map | [CITED: CONTEXT D-02 + Reusable Assets] Locked to extensions — zero new infra |

**Installation:** No `Cargo.toml` changes required for `framework`. Workspace bump is the only `.toml` change.

**Version verification:** All deps are already pinned in the existing workspace; no `npm view`/`cargo search` verification needed because nothing is being added. Confirmed via `framework/Cargo.toml:23-77`.

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                  Boot (Application::run)                        │
│  AppConfig::from_env()  →  Config::register(AppConfig)          │
│    reads INLINE_BUDGET_BYTES (default 102_400)                  │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                Per-Request Handler Body                         │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ req.inline_budget(key, bytes, fallback_url) →           │    │
│  │   1. Lazy-init InlineBudgetState in req.extensions      │    │
│  │   2. Read threshold via Config::get::<AppConfig>()      │    │
│  │   3. Update cumulative[key] += bytes                    │    │
│  │   4. If crossed AND !warned[key]:                       │    │
│  │       tracing::warn!(key, cumulative_bytes, ...)        │    │
│  │       warned[key] = true                                │    │
│  │   5. Return Decision::Inline | Decision::Preload(url)   │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ req.telemetry_record(key, sample) →                     │    │
│  │   global_store().entry((key, None))                     │    │
│  │     .or_insert(VecDeque::with_capacity(128))            │    │
│  │     .push_back(sample); pop_front if len > 128          │    │
│  └─────────────────────────────────────────────────────────┘    │
└──────────────────────────┬──────────────────────────────────────┘
                           │ Request drops; per-request state gone
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│              Operator Surface (separate handler)                │
│  RequestTelemetry::snapshot(key, scope) → Vec<Sample>           │
│    Reads same OnceLock<DashMap>; clones VecDeque → Vec          │
└─────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
framework/src/
├── telemetry/                       [NEW]
│   ├── mod.rs                       module-level docs + re-exports
│   ├── inline_budget.rs             Decision enum, InlineBudgetState, decide_fn
│   └── request_telemetry.rs         Sample, RequestTelemetry, global store, reset()
├── http/
│   └── request.rs                   [EDIT] add 3 methods to impl Request
├── config/providers/
│   └── app.rs                       [EDIT] +1 field + env reader + builder setter
└── lib.rs                           [EDIT] +1 pub mod, +1 re-export line
```

### Pattern 1: Per-request lazy state via `extensions` type-map

**What:** Method fetches-or-creates an `InlineBudgetState` struct in `Request::extensions` keyed by `TypeId::of::<InlineBudgetState>()`.
**When to use:** Any per-request-scoped state that must die with the request.
**Example:**
```rust
// Source: framework/src/http/request.rs:87-103 (verbatim API)
struct InlineBudgetState {
    cumulative: HashMap<String, usize>,
    warned: HashSet<String>,
}

impl Request {
    pub fn inline_budget(&mut self, key: &str, bytes: usize, fallback_url: &str) -> Decision {
        // Lazy-init: insert default state on first call.
        if self.get::<InlineBudgetState>().is_none() {
            self.insert(InlineBudgetState::default());
        }
        let threshold = crate::Config::get::<crate::AppConfig>()
            .map(|c| c.inline_budget_threshold_bytes)
            .unwrap_or(102_400);
        let route_pattern = self.route_pattern().unwrap_or_default();
        let state = self.get_mut::<InlineBudgetState>().expect("just inserted");
        // ... decision logic
    }
}
```

**Borrow-checker note:** `route_pattern()` returns an owned `Option<String>` (cloned, `request.rs:80-82`), so capturing it BEFORE the `&mut` borrow on `get_mut::<InlineBudgetState>` avoids the dual-borrow trap. Same for `Config::get::<AppConfig>()` which returns `Option<AppConfig>` by value.

### Pattern 2: Process-global concurrent registry (verbatim from ferro-bundle)

**What:** `OnceLock<DashMap<K, V>>` accessed through a private accessor fn that does `get_or_init(DashMap::new)`.
**When to use:** Any reader that needs static-method access without a `&self`.
**Example:**
```rust
// Source: ferro-bundle/src/lib.rs:69-85, framework/src/middleware/rate_limit.rs:34-36
static TELEMETRY_STORE: OnceLock<DashMap<(String, Option<String>), VecDeque<Sample>>>
    = OnceLock::new();

fn telemetry_store() -> &'static DashMap<(String, Option<String>), VecDeque<Sample>> {
    TELEMETRY_STORE.get_or_init(DashMap::new)
}
```

### Pattern 3: Structured `tracing::warn!` with required fields

**What:** Single macro call with named structured fields, no formatting variables in message.
**Example:**
```rust
// Source: framework/src/http/action.rs:305-309 (verbatim shape, adapted to D-06 fields)
tracing::warn!(
    key = %key,
    cumulative_bytes = state.cumulative_total,
    threshold_bytes = threshold,
    fallback_url = %fallback_url,
    route_pattern = %route_pattern,
    "inline_budget: threshold crossed; flipping to Preload"
);
```

**Note on `route_pattern` empty-string semantic:** `req.route_pattern()` returns `Option<String>`. Per CONTEXT `<specifics>` line 408, the empty-string-when-None convention is used so the field is always present (easier log aggregation grep). Implementation: `let route_pattern = self.route_pattern().unwrap_or_default();` — the `unwrap_or_default()` on `Option<String>` yields `""`.

### Pattern 4: `#[cfg(test)] reset()` for test isolation

**What:** Crate-visible reset fn that clears the global DashMap.
**Example:**
```rust
// Source: ferro-bundle/src/lib.rs:287-298 (verbatim)
#[cfg(test)]
pub(crate) fn reset() {
    if let Some(r) = TELEMETRY_STORE.get() {
        r.clear();
    }
}
```

Per CONTEXT D-15, this fn lives on `impl RequestTelemetry` (not as a free fn) because D-02 makes `RequestTelemetry` the namespacing unit struct.

### Anti-Patterns to Avoid

- **Reading `AppConfig` via a captured value at boot.** ❌ Caches the threshold and prevents test isolation. ✅ Read via `Config::get::<AppConfig>()` per-call — it's a `HashMap` lookup + clone of a small struct, fast enough.
- **Storing `InlineBudgetState` in a global rather than per-request.** ❌ Breaks Success Criterion 2 (warning per request, not per process). ✅ Use `Request::extensions`.
- **Generic `Sample<T>`.** ❌ Calcifies type at compile time, breaks heterogeneous snapshot aggregation. Locked to `serde_json::Value` per D-07.
- **Using `Instant` for `Sample.recorded_at`.** ❌ Not comparable across process restarts. ✅ Use `SystemTime` per D-10.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Concurrent map | Custom `Mutex<HashMap>` | `DashMap` | Already a direct dep; battle-tested concurrency |
| Lazy static init | Custom `Once` | `std::sync::OnceLock` | stdlib since 1.70; matches ferro-bundle pattern |
| Ring buffer | Custom struct + index arithmetic | `VecDeque` + `pop_front()` | stdlib; O(1) at both ends |
| Per-request state storage | Custom `HashMap<TypeId, ...>` per Request | Existing `Request::extensions` | Already exists at `request.rs:20, 87-103` |
| Structured warning emission | `eprintln!`, `log::warn!`, custom channel | `tracing::warn!` | Matches `action.rs:305, 355`; subscribers already wired |
| Wall-clock timestamps | `chrono::Utc::now()` for Sample | `SystemTime::now()` | No chrono dependency in Sample's public type; chrono interop available via `From<SystemTime>` if downstream wants it |

**Key insight:** Everything Phase 184 needs is already in the framework crate's deps and patterns. The plan is mechanical assembly, not invention.

## Runtime State Inventory

Not applicable — Phase 184 is greenfield (new module, new methods, new field on existing struct). No rename, refactor, migration, or string-replacement. No stored data to migrate, no live service config, no OS-registered state, no env-var renames.

## Common Pitfalls

### Pitfall 1: Borrow-checker conflict reading config + extensions on same Request

**What goes wrong:** `Config::get::<AppConfig>()` returns by value (RwLock-protected clone) so it's actually safe — but a naïve implementation tries to read both `self.route_pattern()` and `self.get_mut::<InlineBudgetState>()` simultaneously.
**Why it happens:** `route_pattern()` and `get_mut::<T>()` both borrow `self`.
**How to avoid:** Capture all `&self`-borrowing reads (threshold, route_pattern) into local owned values BEFORE the `&mut self` borrow on `get_mut`.
**Warning signs:** Compiler error "cannot borrow `*self` as immutable because it is also borrowed as mutable."
```rust
// CORRECT order:
let threshold = crate::Config::get::<crate::AppConfig>()
    .map(|c| c.inline_budget_threshold_bytes).unwrap_or(102_400);
let route_pattern = self.route_pattern().unwrap_or_default();
if self.get::<InlineBudgetState>().is_none() {
    self.insert(InlineBudgetState::default());
}
let state = self.get_mut::<InlineBudgetState>().expect("just inserted");
// ... mutate state freely
```

### Pitfall 2: DashMap deadlock with `entry().or_insert(...)` + `get_mut()`

**What goes wrong:** `DashMap::entry()` takes a shard write-lock. Calling `get_mut` on the same key while holding an `Entry` guard deadlocks.
**Why it happens:** Per-shard locking; the same shard cannot be locked twice from the same thread.
**How to avoid:** Use `entry(key).or_insert_with(VecDeque::with_capacity).push_back(sample)` in one expression chain. Drop the entry before any other access.
**Warning signs:** Test hangs in `telemetry_record` under concurrent stress.

### Pitfall 3: Ring buffer overflow off-by-one

**What goes wrong:** Pushing then checking `len > 128` keeps 129 samples; pushing then checking `len >= 128` then popping yields 128 (correct).
**Why it happens:** Sequence-of-operations error.
**How to avoid:**
```rust
deque.push_back(sample);
while deque.len() > 128 {
    deque.pop_front();
}
```
Or equivalently: `if deque.len() == 128 { deque.pop_front(); } deque.push_back(sample);` (CONTEXT D-08).
**Warning signs:** Test `ring_buffer_caps_at_128` fails with `len == 129`.

### Pitfall 4: Test pollution from shared OnceLock<DashMap>

**What goes wrong:** Two tests in the same binary write to the same `TELEMETRY_STORE`; second test sees first test's samples.
**Why it happens:** Cargo runs unit tests in a single process by default (multi-threaded but single OnceLock).
**How to avoid:** `RequestTelemetry::reset()` at top of every unit test that records. Use `serial_test` if order-independence within a single test is needed (`serial_test` is already a dev-dep, `framework/Cargo.toml:79`).
**Warning signs:** Intermittent failures depending on test ordering.

### Pitfall 5: Reading `Config::get::<AppConfig>()` before `Config::init()` returns `None`

**What goes wrong:** Unit tests that construct a synthetic Request and call `inline_budget` before any boot path runs get `None` from `Config::get::<AppConfig>()`.
**Why it happens:** The global config repository is empty until `Config::register(AppConfig::from_env())` runs.
**How to avoid:** Fall back to a hardcoded `102_400` default when `Config::get::<AppConfig>()` returns `None`. Same default as the AppConfig field. Documented in module docs.
**Warning signs:** Tests pass standalone but fail integration tests where AppConfig isn't initialized.

## Code Examples

### Sample API
```rust
// Source: CONTEXT D-07
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

### Decision enum (locked per D-03)
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Inline,
    Preload(String),
}
```

### RequestTelemetry::snapshot
```rust
impl RequestTelemetry {
    pub fn snapshot(key: &str, scope: Option<&str>) -> Vec<Sample> {
        let scope_owned = scope.map(|s| s.to_string());
        telemetry_store()
            .get(&(key.to_string(), scope_owned))
            .map(|entry| entry.value().iter().cloned().collect())
            .unwrap_or_default()
    }
}
```

### Request impl additions (placement: alongside lines 743-777 — the second `impl Request` block that holds `flash`/`redirect_to`)

```rust
// Add to existing second impl block in framework/src/http/request.rs:742-777
impl Request {
    // ... existing flash / redirect_to ...

    pub fn inline_budget(&mut self, key: &str, bytes: usize, fallback_url: &str) -> crate::Decision {
        crate::telemetry::inline_budget::decide(self, key, bytes, fallback_url)
    }

    pub fn telemetry_record(&mut self, key: &str, sample: crate::Sample) {
        crate::telemetry::request_telemetry::record(key, None, sample);
    }

    pub fn telemetry_record_scoped(&mut self, key: &str, scope: Option<&str>, sample: crate::Sample) {
        crate::telemetry::request_telemetry::record(key, scope, sample);
    }
}
```

Thin delegating methods keep `request.rs` from becoming a god module per CONTEXT D-11 rationale.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Per-request state via thread_local! | TypeId extensions map | ferro pre-1.0 | Async-safe (thread_locals leak across awaits) |
| `Mutex<HashMap>` for global state | `OnceLock<DashMap>` | rate_limit.rs + ferro-bundle Phase 183 | Lock-free shards, no init race |
| `lazy_static!` macro | `std::sync::OnceLock` | Rust 1.70 (stable) | stdlib, no proc-macro |
| `chrono::DateTime` for serializable timestamps | `SystemTime` + serde | — | Smaller type, no chrono dep on the Sample type itself |

**Deprecated/outdated:** None — all chosen patterns are current.

## Project Constraints (from CLAUDE.md)

- **Pre-commit gate:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`. CI enforces `-D warnings`.
- **No co-author lines in commits.**
- **Update docs when framework changes:** Drives CONTEXT D-14 — new `docs/src/the-basics/inline-budget-and-telemetry.md` page + SUMMARY.md entry.
- **Update ferro-mcp when needed:** Not required for Phase 184 — both primitives are runtime APIs, not authoring-time scaffold surfaces. `application_info` already enumerates env-var config keys (per CONTEXT `<code_context>` integration point 4); verify it picks up `INLINE_BUDGET_BYTES` after the AppConfig field is added.
- **Concrete types not `dyn Any`:** Honored — only `dyn Any` is the existing `Request::extensions` map being reused.
- **Project-agnostic crates:** Both primitives are generic (no tenant identity, no hardcoded strings). ✓
- **One Error enum per crate:** N/A for Phase 184 — both primitives are infallible (decide returns Decision, record returns `()`, snapshot returns `Vec`).
- **Prefer editing existing files over creating new ones:** Honored where possible (`request.rs`, `app.rs`, `lib.rs` edited). New files (`telemetry/mod.rs`, `telemetry/inline_budget.rs`, `telemetry/request_telemetry.rs`, `tests/telemetry_smoke.rs`, `docs/src/the-basics/inline-budget-and-telemetry.md`) are unavoidable per D-11 + D-14.

---

## Answers to Investigation Questions

### Q1. Request lifecycle integration — method placement & helpers

**Placement:** Add the three new methods (`inline_budget`, `telemetry_record`, `telemetry_record_scoped`) to the **second `impl Request` block** at `framework/src/http/request.rs:742-777`. That block already holds `flash` and `redirect_to` — both `&mut self` setters that mutate per-request state. Phase 184's methods are the same shape.

Do NOT add them to the first impl block (`request.rs:53-740`) — that block holds extractor methods, body parsers, and accessors; adding telemetry methods there would muddle responsibilities.

**`insert_or_get<T>` pattern:** Does not exist. The two-step `if self.get::<T>().is_none() { self.insert(T::default()); }; self.get_mut::<T>().unwrap()` is the idiom. [VERIFIED: `framework/src/http/request.rs:87-103` — only `insert`, `get`, `get_mut` exist.] This is fine; the pattern is two lines and clear. Adding a generic `insert_or_get` helper is out of scope.

**Exact API surface (matching CONTEXT D-02):**
```rust
impl Request {
    pub fn inline_budget(&mut self, key: &str, bytes: usize, fallback_url: &str) -> Decision;
    pub fn telemetry_record(&mut self, key: &str, sample: Sample);
    pub fn telemetry_record_scoped(&mut self, key: &str, scope: Option<&str>, sample: Sample);
}

impl RequestTelemetry {
    pub fn snapshot(key: &str, scope: Option<&str>) -> Vec<Sample>;
    #[cfg(test)] pub(crate) fn reset();
}
```

Note: `keys()` and `clear()` were mentioned in CONTEXT D-02 as part of the operator surface. Phase 184 plan should include `RequestTelemetry::keys() -> Vec<(String, Option<String>)>` and `RequestTelemetry::clear()` (public, not test-only — operators may want post-deploy reset) because they are listed in D-02. The planner should treat D-02 as a contract: all four operator methods ship.

### Q2. Tracing setup

[VERIFIED: `framework/Cargo.toml:76`] `tracing = "0.1"` is a direct dep. No `Cargo.toml` change needed.

The structured-field macro syntax to mirror is at `framework/src/http/action.rs:305-309`:
```rust
tracing::warn!(
    handler = %handler_name,
    rejected_url = %sanitize_for_log(rejected),
    "redirect_override rejected: not same-origin (success path)"
);
```

Key syntactic notes:
- `field = %value` — `Display` formatting (use this for strings)
- `field = ?value` — `Debug` formatting (use for enums like `ActionKind`)
- `field = value` (no sigil) — `Value` trait (use for numeric primitives)
- Message string is the LAST positional argument.

Phase 184's warning will use `%key`, `cumulative_bytes` (no sigil — `usize` implements `Value`), `threshold_bytes` (no sigil), `%fallback_url`, `%route_pattern` (because it's `String`).

### Q3. DashMap and dependencies

[VERIFIED: `framework/Cargo.toml:63`] `dashmap = "6"` is already a direct dep, used at `framework/src/middleware/rate_limit.rs:26`. **No `Cargo.toml` change needed.** Same as ferro-bundle's pattern at `ferro-bundle/src/lib.rs:33`.

### Q4. `tracing::warn!` route_pattern field — Option<String> handling

[VERIFIED: `framework/src/http/request.rs:80-82`]
```rust
pub fn route_pattern(&self) -> Option<String> {
    self.route_pattern.clone()
}
```

Returns `Option<String>` by value (cloned). For structured warning emission, per CONTEXT `<specifics>` line 408 the preference is empty-string-not-omitted:

```rust
let route_pattern = self.route_pattern().unwrap_or_default(); // -> String (empty if None)
// later:
tracing::warn!(
    // ...
    route_pattern = %route_pattern,
    // ...
);
```

`unwrap_or_default()` on `Option<String>` yields `""`. The field is then always present in log aggregation. No existing code in the framework uses this exact pattern (action.rs only emits `route_pattern` indirectly via `handler` field), so Phase 184 establishes the convention.

### Q5. AppConfig consumption from Request — data flow

**Critical finding — read carefully:**

`AppConfig` is NOT stored on the `Request`. It is registered in a **global config repository** at boot via `Config::register(AppConfig::from_env())` ([VERIFIED: `framework/src/config/mod.rs:68`]) and read at any time via `Config::get::<AppConfig>()` ([VERIFIED: `framework/src/config/mod.rs:124, 141`]).

**The repository:** `static CONFIG_REPOSITORY: OnceLock<RwLock<ConfigRepository>>` ([VERIFIED: `framework/src/config/repository.rs:6`]). `Config::get::<T>()` does an RwLock read + TypeId lookup + clone of the small struct ([VERIFIED: `framework/src/config/repository.rs:60-63`]). This is fast — fine to call once per `req.inline_budget(...)` call.

**Implementation pattern:**
```rust
let threshold = crate::Config::get::<crate::AppConfig>()
    .map(|c| c.inline_budget_threshold_bytes)
    .unwrap_or(102_400); // fallback for tests where Config::init wasn't called
```

**The `unwrap_or(102_400)` fallback is essential** — see Pitfall 5 above. Unit tests construct synthetic Requests without booting the framework, so `Config::get::<AppConfig>()` returns `None`. Hardcoding the same default keeps tests deterministic.

**No middleware injection needed.** No need to stash the threshold on the Request. The Request never holds an `AppConfig` reference, and Phase 184 doesn't need to change that.

### Q6. Validation architecture — 8 Nyquist dimensions

See `## Validation Architecture` section below — all 8 dimensions enumerated with commands.

### Q7. Test crate dependencies

[VERIFIED: `framework/Cargo.toml:78-83`] Dev-deps already include `serial_test = "3"`, `tempfile = "3"`, `hyper-util` with `tokio`, `http-body-util`. **No new dev-deps required.**

- `serde_json::json!` macro — already available because `serde_json` is in `[dependencies]`, not `[dev-dependencies]`. Used by `framework/tests/api_resource_derive.rs:8` (`use serde_json::json;`) and `framework/tests/pipeline_order.rs:67` (`serde_json::json!({})`). Tests can `use serde_json::json;` directly.
- `tracing-test` / `tracing-subscriber` — **not currently dev-deps**. Asserting on the warning emission requires capturing tracing output. Options:
  1. **Add `tracing-test = "0.2"` as a dev-dep** (small, purpose-built for this). Use `#[tracing_test::traced_test]` attribute on the test and `logs_assert!` or `logs_contain!` macros.
  2. **Build a custom subscriber** in the test using `tracing-subscriber` (already a transitive dep through `tracing` ecosystem).
  3. **Test the state machine directly** — verify `warned` flag set after first crossing without asserting on tracing emission. Cheapest; matches CONTEXT D-15 pattern (test the observable state, not the side channel).

  **Recommendation:** Option (3) for unit tests + Option (1) for one focused integration test that proves the warning IS emitted. Adding `tracing-test = "0.2"` to `[dev-dependencies]` is low-risk (10 KB crate, well-maintained). Planner should consider this small addition.

### Q8. Risks / unknowns

**Risk 1 — Public `pub mod telemetry`?** CONTEXT D-11 says re-exports flatten at lib.rs (`pub use telemetry::{Decision, RequestTelemetry, Sample};`). Does the module itself need `pub mod telemetry`? Looking at `framework/src/lib.rs:9-44`, most modules are `pub mod`. To stay consistent and to let downstream users do `use ferro_rs::telemetry::Sample` if they want, declare it `pub mod telemetry;`. The re-exports at the crate root are the canonical path, but the module path stays open.

**Risk 2 — `InlineBudget` name is in CONTEXT but NOT re-exported (D-11):** Verified from CONTEXT D-11 line: `pub use telemetry::{Decision, InlineBudget, RequestTelemetry, Sample};`. WAIT — this conflicts with what the user asked in the prompt: "NOT `InlineBudget` — never user-typed." The user's prompt is later/more authoritative; the CONTEXT D-02 text agrees: "InlineBudget is hidden". So the planner should follow the user's prompt: re-export `Decision`, `RequestTelemetry`, `Sample` — but NOT `InlineBudget`. The CONTEXT D-11 line includes `InlineBudget` apparently by drafting error; the rationale in D-02 makes the intent unambiguous. **Recommend planner remove `InlineBudget` from the re-export list to match D-02 semantics.**

**Risk 3 — Workspace bump 0.2.43 → 0.2.44:** [VERIFIED: `Cargo.toml` workspace.package.version = "0.2.43"]. Single line edit. Per CONTEXT D-13, all `ferro-*` and `framework` crates inherit via `version.workspace = true`. No fan-out edits.

**Risk 4 — Pollution of unit tests by global registry without reset()`:** Two unit tests writing to `RequestTelemetry` in the same binary will see each other's samples. `reset()` at the top of every test is the discipline (CONTEXT D-15). Use `#[serial]` from `serial_test` if tests must not interleave (`serial_test` already a dev-dep).

**Risk 5 — `Config::init()` is not called in unit tests:** Per Q5, the `unwrap_or(102_400)` fallback handles this. Integration tests under `tests/telemetry_smoke.rs` should call `crate::Config::init(&std::path::Path::new("."))` if they want to override the threshold via env var, OR construct an `AppConfig` explicitly and `Config::register(custom_app_config)` before running.

**No borrow-checker pitfalls** if the ordering in Pitfall 1 is followed. Reading `route_pattern()` and `Config::get::<AppConfig>()` both clone, so they don't interfere with the subsequent `get_mut::<InlineBudgetState>` mutable borrow.

### Q9. gestiscilo Phase 187 consumption pattern

[CITED: CONTEXT lines 442-475] After Phase 184 publishes:

```rust
// gestiscilo handler — after Cargo.toml bumps ferro-rs to 0.2.44
use ferro_rs::{Decision, Request, RequestTelemetry, Sample};
use serde_json::json;

async fn render_page(req: &mut Request, products: Vec<Product>) -> String {
    let products_json = serde_json::to_string(&products).unwrap_or_default();
    let payload_bytes = products_json.len();

    // Decision point
    let html = match req.inline_budget(
        "products_payload",
        payload_bytes,
        "/_/bootstrap/products.json",
    ) {
        Decision::Inline => format!(
            "<script id='__products' type='application/json'>{products_json}</script>{body}",
        ),
        Decision::Preload(url) => format!(
            r#"<link rel="preload" as="fetch" href="{url}" crossorigin>{body}"#,
        ),
    };

    // Telemetry record (scoped by tenant)
    req.telemetry_record_scoped(
        "products_payload_size",
        Some(&format!("tenant:{}", tenant_id)),
        Sample::now(json!({ "bytes": payload_bytes })),
    );

    html
}

// Operator dashboard handler:
async fn ops_payload_distribution(req: &mut Request, tenant_id: u64) -> HttpResponse {
    let scope = format!("tenant:{tenant_id}");
    let samples = RequestTelemetry::snapshot("products_payload_size", Some(&scope));
    HttpResponse::json(&samples) // Vec<Sample> serializes naturally
}
```

**Implications for the Phase 184 API contract:**
- `req.inline_budget(...)` returns `Decision` — the consumer must `match` on it. `Decision::Preload(String)` (D-03) lets the consumer drop the owned URL into the format!. ✓
- `req.telemetry_record_scoped(...)` takes `Option<&str>` — consumer can format scope as `&str` from a temporary. ✓
- `RequestTelemetry::snapshot(...)` returns `Vec<Sample>` — consumer can directly serialize via `HttpResponse::json`. Requires `Sample: Serialize` ✓ (D-07).
- The scope convention `"tenant:42"` is unenforced ✓ (D-09).

The API contracts are correct for the consumer pattern. No adjustments needed.

### Q10. Plan grouping suggestion

See `## Suggested Plan Decomposition` section below.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` (built-in) + `serial_test` for ordering |
| Config file | `framework/Cargo.toml` (test deps already present) |
| Quick run command | `cargo test -p ferro-rs telemetry::` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

Phase 184 has no formal REQ-IDs (the prompt confirms). The contract is the 5 Success Criteria:

| Success Criterion | Behavior | Test Type | Automated Command | File |
|-------------------|----------|-----------|-------------------|------|
| SC-1 | `inline_budget` returns Inline below threshold, Preload above | unit | `cargo test -p ferro-rs telemetry::inline_budget::tests::decides_inline_below_threshold` | `framework/src/telemetry/inline_budget.rs` (inline tests) |
| SC-2 | Warning fires exactly once per (key, request) at threshold cross | unit | `cargo test -p ferro-rs telemetry::inline_budget::tests::warn_fires_once_per_key` | `framework/src/telemetry/inline_budget.rs` |
| SC-3a | `telemetry_record` + `snapshot` round-trip | unit | `cargo test -p ferro-rs telemetry::request_telemetry::tests::record_and_snapshot_round_trip` | `framework/src/telemetry/request_telemetry.rs` |
| SC-3b | Thread-safe under concurrent record | unit | `cargo test -p ferro-rs telemetry::request_telemetry::tests::concurrent_record_no_deadlock` | `framework/src/telemetry/request_telemetry.rs` |
| SC-3c | Ring buffer caps at 128 and drops oldest | unit | `cargo test -p ferro-rs telemetry::request_telemetry::tests::ring_buffer_caps_at_128` | `framework/src/telemetry/request_telemetry.rs` |
| SC-4 | Crate location decision recorded | n/a | (documentary — CONTEXT D-01) | `184-CONTEXT.md` |
| SC-5 | Publishes to crates.io; gestiscilo consumes | integration | `cargo publish -p ferro-rs --dry-run` (Phase-internal); real publish on master merge | `Cargo.toml` (workspace.package.version) |

Additional integration test (recommended):
| Test | Type | Command | File |
|------|------|---------|------|
| Both primitives via real Request | integration | `cargo test -p ferro-rs --test telemetry_smoke` | `framework/tests/telemetry_smoke.rs` (Wave 0 — create) |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-rs telemetry::`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate (before `/gsd-verify-work`):** Full suite green + `cargo fmt --all -- --check` + `cargo clippy --all --all-targets -- -D warnings` + `cargo publish -p ferro-rs --dry-run` + `cargo doc --no-deps`.

### Wave 0 Gaps
- [ ] `framework/src/telemetry/mod.rs` — module skeleton with docs
- [ ] `framework/src/telemetry/inline_budget.rs` — implementation + inline tests
- [ ] `framework/src/telemetry/request_telemetry.rs` — implementation + inline tests
- [ ] `framework/tests/telemetry_smoke.rs` — integration test (both primitives in one handler)
- [ ] `docs/src/the-basics/inline-budget-and-telemetry.md` — docs page
- [ ] **Optional small dev-dep:** `tracing-test = "0.2"` for warning-emission assertions (Q7 option 1)

### 8 Nyquist Dimensions

| # | Dimension | Command | Pass Criterion |
|---|-----------|---------|----------------|
| 1 | Compile | `cargo build -p ferro-rs` | Exit 0, no errors |
| 2 | Lint | `cargo clippy --all --all-targets -- -D warnings` | Exit 0, no warnings (CI gate) |
| 3 | Unit tests | `cargo test -p ferro-rs telemetry::` | All inline `#[test]` pass |
| 4 | Integration tests | `cargo test -p ferro-rs --test telemetry_smoke` | telemetry_smoke.rs passes (after Wave 0 creates it) |
| 5 | Docs build | `cargo doc --no-deps -p ferro-rs` | Exit 0; new module visible in target/doc/ferro_rs/telemetry/ |
| 6 | Format | `cargo fmt --all -- --check` | Exit 0, no diff |
| 7 | Publish dry-run | `cargo publish -p ferro-rs --dry-run` | Exit 0, package builds tarball |
| 8 | Observability | Inline unit test captures the `warn` state-machine: assert `warned[key] = true` after first cross; second cross does not flip state again (no double-warn) | Tested via state-machine assertion (D-15 reset() pattern). Optional: `tracing-test` for direct emission assertion. |

---

## Suggested Plan Decomposition

Recommended 3-plan structure. Each plan ships independently green (workspace builds, tests pass, clippy clean).

### Plan 184-01: Config field + Telemetry module skeleton + Sample/Decision types

**Files:**
- `framework/src/config/providers/app.rs` — add `inline_budget_threshold_bytes: usize` field, env reader `env("INLINE_BUDGET_BYTES", 102_400usize)`, builder setter.
- `framework/src/telemetry/mod.rs` — new file; module-level `//!` docs + `pub mod inline_budget; pub mod request_telemetry;` + `pub use {Decision, RequestTelemetry, Sample};`.
- `framework/src/telemetry/request_telemetry.rs` — new file; `Sample` struct + constructors + global `OnceLock<DashMap>` + `RequestTelemetry` unit struct + `snapshot`, `keys`, `clear` (public), `reset` (test-only) + private `record(key, scope, sample)` helper.
- `framework/src/telemetry/inline_budget.rs` — new file; `Decision` enum + `InlineBudgetState` struct + private `decide(req, key, bytes, fallback_url) -> Decision`.
- `framework/src/lib.rs` — add `pub mod telemetry;` + `pub use telemetry::{Decision, RequestTelemetry, Sample};` (NOT `InlineBudget` per Q8 Risk 2).

**Scope:** Pure data types and storage. No `Request` method touches yet. All inline unit tests for the storage layer and the AppConfig default.

**Verifies:**
- Dimension 1 (compile), 2 (lint), 3 (unit tests), 5 (docs build), 6 (format).
- Unit tests for SC-3a, SC-3b, SC-3c.

**Rationale:** Smallest standalone slice. Zero risk of breaking the existing Request API. Establishes the foundation.

### Plan 184-02: Request methods + InlineBudget decision logic + warning emission

**Files:**
- `framework/src/http/request.rs` — add three methods to the second `impl Request` block (lines 742-777): `inline_budget`, `telemetry_record`, `telemetry_record_scoped`. Each delegates to `crate::telemetry::*` private fns.
- `framework/src/telemetry/inline_budget.rs` — implement `decide(req, key, bytes, fallback_url)`: lazy-init state via extensions, read threshold via `Config::get::<AppConfig>().map(|c| c.inline_budget_threshold_bytes).unwrap_or(102_400)`, increment cumulative, emit `tracing::warn!` once per (key, request) on threshold cross, return `Decision`.

**Scope:** Request integration. Inline unit tests for the decision state machine (SC-1, SC-2 — verified via state assertion).

**Verifies:**
- Dimensions 1, 2, 3, 6.
- Unit tests for SC-1, SC-2.

**Rationale:** Depends on Plan 01's types. Risk-isolated — borrow-checker work confined to one file. After this plan, the API surface is complete and consumable.

### Plan 184-03: Integration test + Docs page + Workspace bump

**Files:**
- `framework/tests/telemetry_smoke.rs` — new file; integration test exercising both primitives via a real `Request` (use `hyper-util` + `http-body-util` patterns from existing `framework/tests/action_handler.rs`).
- `docs/src/the-basics/inline-budget-and-telemetry.md` — new page covering both primitives, end-to-end example, scope conventions table.
- `docs/src/SUMMARY.md` — add one entry under "The Basics" pointing to the new page.
- `Cargo.toml` (workspace root) — bump `workspace.package.version` from `"0.2.43"` to `"0.2.44"`.
- (Optional) `framework/Cargo.toml` `[dev-dependencies]` — add `tracing-test = "0.2"` if planner adopts Q7 Option (1) for warning-emission integration assertion.

**Scope:** External-facing surface (docs + integration test) + ship gate (version bump + publish dry-run).

**Verifies:**
- Dimensions 1, 2, 3, 4, 5, 6, 7, 8.
- Integration test exercises SC-1, SC-2, SC-3a in one round-trip.

**Rationale:** Final ship-gate plan. After this, the phase is publish-ready. The dry-run in Dimension 7 proves the workspace bump didn't break anything.

### Why 3 plans (not 4)?

Considered splitting Plan 03 into "integration test" + "docs + publish bump". Rejected because:
- Integration test creation is mechanical once the API is locked (Plan 02).
- Docs + version bump is a small content-only edit — half a plan's worth.
- Combining keeps the phase tight; the 3-plan rhythm matches sibling Phase 183's grouping.

### Wave / dependency graph

```
Plan 01  (foundation: types + storage)
   │
   ▼
Plan 02  (Request integration + state machine)
   │
   ▼
Plan 03  (integration test + docs + version bump)
```

Sequential. No parallelization opportunity — each plan strictly depends on the previous.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `unwrap_or(102_400)` fallback when `Config::get::<AppConfig>()` returns `None` is acceptable (CONTEXT does not specify) | Q5 / Pitfall 5 | If user wants `unwrap_or_else(|| panic!(...))`, tests will fail in non-booted contexts. Recommend planner confirm before locking. |
| A2 | `RequestTelemetry::keys()` and `RequestTelemetry::clear()` are public ops methods (mentioned in CONTEXT D-02 but not re-stated in user's "public API" prompt list) | Q1 | If not part of v1, removing later is a break — additive forward path is safe |
| A3 | `InlineBudget` is NOT in the re-export list (CONTEXT D-11 disagrees with D-02 and user's prompt — choosing D-02 / prompt) | Q8 Risk 2 | If re-exported by accident, user gets a confusing type they shouldn't construct. Recommend planner verify with user. |
| A4 | Adding `tracing-test = "0.2"` as a dev-dep is acceptable | Q7 / Validation Dim 8 | If user prefers state-machine-only assertion, drop it. Either path satisfies SC-2. |
| A5 | `pub mod telemetry;` (vs `mod telemetry;`) — exposing the module path alongside re-exports | Q8 Risk 1 | Either works. Public module path is more permissive; matches existing convention. |

---

## Open Questions (RESOLVED)

1. **OQ1 (A1) default-when-config-uninit — RESOLVED:** Use the soft fallback `Config::get::<AppConfig>().map(|c| c.inline_budget_threshold_bytes).unwrap_or(102_400)`. Preserves test isolation (tests don't have to call `Config::init`); the strict variant (`.expect(...)`) would force every test using `req.inline_budget` to bootstrap config, which is friction without benefit.

2. **OQ2 (A3) `InlineBudget` re-export — RESOLVED:** Do NOT re-export `InlineBudget` from `framework::lib.rs`. The re-export set is exactly `{Decision, RequestTelemetry, Sample}`. D-02 semantics ("never user-typed") supersede D-11's mistakenly-broader list. Pattern-mapper independently flagged the same resolution.

3. **OQ3 Sample::from_value constructor — RESOLVED:** Do NOT add the sugar `Sample::from_value(value)`. The two locked constructors (`Sample::now(value)` for the 99% case and `Sample::at(when, value)` for backfill) are sufficient. Additional sugar can ship in a follow-up if real redundancy emerges across consumers.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | All dimensions | ✓ | (assumed) | — |
| `dashmap` 6.x | Storage | ✓ | direct dep | — |
| `tracing` 0.1 | Warning emission | ✓ | direct dep | — |
| `serde_json` 1.x | Sample.value | ✓ | direct dep | — |
| `serial_test` 3 | Test ordering | ✓ | dev-dep | — |
| `hyper-util` 0.1 | Integration test Request constructor | ✓ | dev-dep (tokio feature) | — |
| `tracing-test` 0.2 | Warning-emission assertion (optional) | ✗ | — | State-machine assertion (Q7 Option 3) |

**Missing dependencies with no fallback:** None.
**Missing dependencies with fallback:** `tracing-test` — fallback is direct state assertion (no measurable loss).

---

## Sources

### Primary (HIGH confidence — VERIFIED in this session)

- `framework/Cargo.toml:23-77` — all required deps already present
- `framework/src/http/request.rs:11-26, 87-103, 742-777` — Request struct, extensions API, second impl block
- `framework/src/http/request.rs:80-82` — `route_pattern()` returns `Option<String>` (cloned)
- `framework/src/http/action.rs:305-309, 355-359` — `tracing::warn!` structured-field pattern
- `framework/src/config/providers/app.rs:1-99` — AppConfig + AppConfigBuilder + from_env pattern
- `framework/src/config/env.rs:113-118` — `env<T>(name, default)` helper signature
- `framework/src/config/mod.rs:64-86, 124, 141` — `Config::init`, `Config::get::<T>()` pattern
- `framework/src/config/repository.rs:6, 27-32, 60-63` — global OnceLock<RwLock<ConfigRepository>>
- `ferro-bundle/src/lib.rs:33, 69-85, 287-298` — process-global DashMap + reset() pattern (sibling reference)
- `framework/src/middleware/rate_limit.rs:26, 34-36` — same DashMap+OnceLock pattern in framework itself
- `framework/src/lib.rs:9-44, 60-63, 105-113` — module structure and re-export pattern
- `framework/tests/api_resource_derive.rs:8`, `framework/tests/pipeline_order.rs:67` — `serde_json::json!` available in tests
- `framework/tests/` — directory listing confirms integration test convention (`*.rs` per topic)
- `Cargo.toml` (root) `[workspace.package] version = "0.2.43"` — workspace bump source
- `docs/src/SUMMARY.md:1-20` — "The Basics" section structure for D-14 entry
- `184-CONTEXT.md` D-01 through D-15 — locked decisions

### Secondary (MEDIUM confidence)

- None — every claim verified against codebase.

### Tertiary (LOW confidence)

- None.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — every dep verified present in `framework/Cargo.toml`
- Architecture: HIGH — patterns mirrored from `ferro-bundle/src/lib.rs` and `framework/src/middleware/rate_limit.rs` (both shipped, both green)
- Pitfalls: HIGH — borrow-checker traps verified by inspection of `Request` API; ring-buffer + reset patterns proven in Phase 183
- AppConfig data flow: HIGH — `Config::get::<T>()` pattern verified at `framework/src/config/mod.rs:124, 141`

**Research date:** 2026-06-06
**Valid until:** 2026-07-06 (30 days — stable framework, low ecosystem churn)
**Planner can proceed.**
