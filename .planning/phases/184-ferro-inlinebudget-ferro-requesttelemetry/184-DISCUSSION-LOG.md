# Phase 184: `ferro::InlineBudget` + `ferro::RequestTelemetry` — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-06
**Phase:** 184 — `ferro::InlineBudget` + `ferro::RequestTelemetry`
**Mode:** `--auto` (Claude selected the recommended option for every gray area without interactive prompts)
**Areas discussed:** Crate location, API surface shape, Decision enum, Threshold configuration, Call-site signature, Warning channel, Sample shape, Ring-buffer capacity, Scope parameter, Storage model, Module placement, AppConfig integration, Publish cadence, Docs, Test isolation

---

## G-01: Crate location

| Option | Description | Selected |
|--------|-------------|----------|
| Module inside `framework` (the core, published as `ferro-rs`) — methods on `Request` impl + free function for snapshot | "Extension trait in ferro-core" reading: ferro-core IS `framework`. No new crate, no publish-bootstrap friction, conceptual coherence with the existing `Request` extensions API. | ✓ |
| New `ferro-telemetry` crate — extension trait on `Request` that consumers import | Forces opt-in via `Cargo.toml` and an extra `use ferro_telemetry::RequestExt;` per file. Contradicts the roadmap's "any ferro app" framing. | |
| Two separate crates (`ferro-inline-budget` + `ferro-telemetry`) | Even more import friction; over-engineered for two ~150-line primitives. | |

**Auto-selected:** Module inside `framework`. CONTEXT D-01 enumerates the five rationale points: conceptual coherence, roadmap framing, "any ferro app" discovery framing, cheap future split, bootstrap-friction avoidance, and the 25-crate workspace footprint argument.

---

## G-02: Public API surface shape

| Option | Description | Selected |
|--------|-------------|----------|
| Methods on `Request` impl for the writer side + static `RequestTelemetry::snapshot(key, scope)` for the reader side | `req.inline_budget(...)`, `req.telemetry_record(...)`, `RequestTelemetry::snapshot(...)`. Matches existing ferro idiom. | ✓ |
| Builder-style: `InlineBudget::for_request(&mut req).key(k).bytes(n).decide()` | Verbose; offers no additional capability over the method form. | |
| Trait import required for methods (`use ferro_rs::RequestExt;`) | Friction without benefit when methods live in `framework` itself. | |

**Auto-selected:** Methods on `Request` impl + static snapshot. CONTEXT D-02.

---

## G-03: `Decision` enum shape

| Option | Description | Selected |
|--------|-------------|----------|
| `enum Decision { Inline, Preload(String) }` — roadmap-locked variants, `String` for the URL | Allocation cost negligible; ownership clear. | ✓ |
| `enum Decision { Inline, Preload(Cow<'static, str>) }` | Marginal saving when callers pass `&'static str` literals; complicates the API. | |
| `enum Decision<U> { Inline, Preload(U) }` — generic over URL type | Type parameter cascades through call sites for no real benefit. | |

**Auto-selected:** `Preload(String)`. CONTEXT D-03.

---

## G-04: Threshold configuration model

| Option | Description | Selected |
|--------|-------------|----------|
| Global default 100 KB via `AppConfig` + env var `INLINE_BUDGET_BYTES`. No per-key override in v1. | Simplest surface; covers the discovery use case (gestiscilo 200 KB pain). Per-key override is an additive future change. | ✓ |
| Per-key override at call site: `req.inline_budget_with_limit(key, bytes, fallback_url, limit)` | Useful but premature; no consumer surfaces heterogeneous budgets yet. | |
| Hardcoded compile-time threshold | Removes operator control; bad for production tuning. | |

**Auto-selected:** Global default via `AppConfig`. CONTEXT D-04 (100 KB rationale).

---

## G-05: Call-site signature for `inline_budget`

| Option | Description | Selected |
|--------|-------------|----------|
| `req.inline_budget(key, bytes, fallback_url)` — caller passes fallback URL at the decision point | Caller knows the URL when making the decision; single call. | ✓ |
| `req.set_inline_fallback(key, url)` then `req.inline_budget(key, bytes)` | Two-step API; separates concerns that belong together. | |
| Return `Decision::Preload(impl Fn() -> String)` for late URL construction | Closure ceremony for no benefit. | |

**Auto-selected:** Pass URL upfront. CONTEXT D-05.

---

## G-06: Warning channel and fire-once semantics

| Option | Description | Selected |
|--------|-------------|----------|
| `tracing::warn!` with structured fields (key, cumulative_bytes, threshold_bytes, fallback_url, route_pattern). Fire-once per (key, request) tracked in per-request `InlineBudgetState`. | Matches `framework/src/http/action.rs:305,355` pattern. | ✓ |
| `log::warn!` (older `log` crate) | Inconsistent with rest of framework. | |
| Global fire-once (suppress across requests) | Hides the per-request signal operators want. | |

**Auto-selected:** `tracing::warn!` per-request fire-once. CONTEXT D-06.

---

## G-07: `Sample` shape

| Option | Description | Selected |
|--------|-------------|----------|
| Concrete struct `Sample { recorded_at: SystemTime, value: serde_json::Value }` | Round-trips via serde (SC-3); pervasive value type in ferro; no generic cascade. | ✓ |
| Generic `Sample<T: Serialize + DeserializeOwned + Send + Sync>` | Locks T at compile time; snapshot consumers can't aggregate across heterogeneous T. | |
| Numeric-only `Sample { ts: SystemTime, value: f64 }` | Too narrow; ferro consumers want structured payloads. | |
| Trait-object `Box<dyn TelemetrySample>` | Runtime polymorphism with no win over `serde_json::Value`. | |

**Auto-selected:** Concrete struct with `serde_json::Value` payload. CONTEXT D-07.

---

## G-08: Ring buffer capacity

| Option | Description | Selected |
|--------|-------------|----------|
| Hardcoded 128 samples per (key, scope) | Power of 2; bounded memory (~28 KB per pair worst case); "lost on restart" means long histories belong elsewhere. | ✓ |
| Configurable per key via `RequestTelemetry::set_capacity(key, n)` | Premature; no real consumer demand. | |
| Configurable via env var (single global N) | Adds knob without enough signal on what to tune. | |

**Auto-selected:** Hardcoded 128. CONTEXT D-08.

---

## G-09: `scope` parameter shape

| Option | Description | Selected |
|--------|-------------|----------|
| `Option<&str>` — caller-defined string convention (`"tenant:42"`, `"route:/api/products"`, etc.) | Flexible; doesn't calcify vocabulary. | ✓ |
| `enum Scope { Global, Tenant(String), Route(String) }` | Locks a vocabulary too early. | |
| Two methods (`telemetry_record` for global + `telemetry_record_scoped` for scoped) on top of `Option<&str>` reader API | Adopted in conjunction with `Option<&str>` to avoid forcing every caller to type `None`. | ✓ |

**Auto-selected:** `Option<&str>` + two writer methods. CONTEXT D-09.

---

## G-10: Storage model

| Option | Description | Selected |
|--------|-------------|----------|
| Process-global `OnceLock<DashMap<(String, Option<String>), VecDeque<Sample>>>` | Matches Phase 183 `ferro-bundle` D-02 pattern. DashMap is already a transitive dep. O(1) push-back / pop-front via VecDeque. | ✓ |
| `std::sync::Mutex<HashMap<...>>` | Single global lock hurts under concurrent record load. | |
| Per-tenant storage with explicit registration | Overkill; the `scope` parameter already partitions by tenant/route. | |

**Auto-selected:** `OnceLock<DashMap<...>>`. CONTEXT D-10.

---

## G-11: Module placement inside `framework`

| Option | Description | Selected |
|--------|-------------|----------|
| `framework/src/telemetry/{mod.rs, inline_budget.rs, request_telemetry.rs}` | Co-located concerns; clean separation; method bodies on `Request` are thin delegates. | ✓ |
| Inline into `framework/src/http/request.rs` directly | Would balloon request.rs into a god module. | |
| Split into `framework/src/inline_budget.rs` + `framework/src/request_telemetry.rs` at the crate root | Two top-level modules for one logical concern; awkward. | |

**Auto-selected:** `framework/src/telemetry/`. CONTEXT D-11.

---

## G-12: `AppConfig` integration

| Option | Description | Selected |
|--------|-------------|----------|
| Additive field `inline_budget_threshold_bytes: usize` on `AppConfig` + env var `INLINE_BUDGET_BYTES` + parallel builder setter | No breaking change; standard ferro env-backed config pattern. | ✓ |
| Separate `TelemetryConfig` struct | Two configs to wire; no real win. | |
| Ad-hoc `std::env::var(...)` at the call site | Bypasses the `AppConfig` abstraction; inconsistent with rest of framework. | |

**Auto-selected:** Additive `AppConfig` field. CONTEXT D-12.

---

## G-13: Publish cadence

| Option | Description | Selected |
|--------|-------------|----------|
| Single workspace bump (0.2.43 → 0.2.44) shipped via existing WAVE2 (`ferro-rs`) | Matches memory `feedback_friction_loop_release_cadence.md`: single publish at end of release loop. gestiscilo Phase 187 bumps after. | ✓ |
| Per-crate semver staging | Workspace uses unified version; staging adds release-engineering load with no benefit. | |
| New publish wave for telemetry-specific crate | N/A — no new crate (D-01). | |

**Auto-selected:** Single workspace bump in WAVE2. CONTEXT D-13.

---

## G-14: Documentation

| Option | Description | Selected |
|--------|-------------|----------|
| New page `docs/src/the-basics/inline-budget-and-telemetry.md` + `SUMMARY.md` link | CLAUDE.md mandates docs updates when framework changes; both primitives are public Rust API. | ✓ |
| Module-level rustdoc only | Insufficient for a public-API surface this size; consumers need a usage guide outside `cargo doc`. | |
| Inline into existing `docs/src/the-basics/configuration.md` | Conflates topics. | |

**Auto-selected:** New docs page. CONTEXT D-14.

---

## G-15: Test isolation

| Option | Description | Selected |
|--------|-------------|----------|
| `#[cfg(test)] pub(crate) fn reset()` on `RequestTelemetry` that clears the global DashMap | Matches Phase 183 D-13; integration tests already isolated by Cargo's per-binary execution model. | ✓ |
| Per-test `OnceLock` reset via reflection or test-only constructor | Over-engineered. | |
| No isolation (rely on test naming to avoid collisions) | Fragile; collisions silently corrupt assertions. | |

**Auto-selected:** `#[cfg(test)] reset()`. CONTEXT D-15.

---

## Claude's Discretion

The following decisions were explicitly deferred to the planner (CONTEXT.md "Claude's Discretion" subsection):

- Exact `tracing` field names and message wording (the required structured-field set is locked in D-06; prose is flexible).
- Whether `Sample` has only `now(...)` and `at(when, ...)` constructors or additionally `from_value(value)` (defaulting to `SystemTime::now()`).
- Exact docs page section ordering and sub-headers (D-14).
- Whether to use `#[rstest]`-style parameterization or vanilla `#[test]` functions for the test corpus.
- Whether the integration test exercises both primitives in one handler or two parallel handlers.

## Deferred Ideas

Captured in CONTEXT.md `<deferred>` section. The substantive ones:

1. Per-key threshold override at the call site (`req.inline_budget_with_limit`).
2. Persistent / cross-process telemetry (Prometheus, OpenTelemetry, custom sinks).
3. Typed surfaces (Counter / Gauge / Histogram) on top of `Sample`.
4. Cross-request aggregation primitives.
5. `Scope` enum with named variants.
6. Extraction to a `ferro-telemetry` crate (re-evaluate at second consumer, or v2.0 split).
7. Bootstrap endpoint pattern (gestiscilo-side concern, not ferro's primitive).
8. Histogram of cumulative-bytes distribution per key.
9. `ferro-mcp` introspection of `RequestTelemetry::keys()`.

## Folded Todos

None — `gsd-tools todo match-phase 184` returned `todo_count: 0`.
