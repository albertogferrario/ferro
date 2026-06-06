---
phase: 184
slug: ferro-inlinebudget-ferro-requesttelemetry
review_depth: standard
files_reviewed: 10
status: issues
findings:
  critical: 0
  warning: 0
  info: 6
  total: 6
reviewed: 2026-06-06
---

# Phase 184 Code Review

Standard-depth review of the two request-scoped primitives shipped in Phase 184:
`InlineBudget` (inline-vs-preload decisioning, request-scoped state in
`Request::extensions`) and `RequestTelemetry` (process-global ring buffer keyed
by `(key, scope)`).

Overall code health: **good**. The state machine is split correctly between a
pure `record_and_decide` and a thin `Request`-side `decide` wrapper, the
borrow-ordering pitfall is handled, the ring buffer cap is enforced, the global
store is properly scoped to `OnceLock`, the public `Request` methods are clear,
and the docs page matches the code's behaviour. Findings below are all
`info`-level — polish, doc gaps, and a small redundancy.

No critical bugs, no security issues, no concurrency hazards.

## Files Reviewed

- `Cargo.toml`
- `docs/src/SUMMARY.md`
- `docs/src/the-basics/inline-budget-and-telemetry.md`
- `framework/src/config/providers/app.rs`
- `framework/src/http/request.rs`
- `framework/src/lib.rs`
- `framework/src/telemetry/inline_budget.rs`
- `framework/src/telemetry/mod.rs`
- `framework/src/telemetry/request_telemetry.rs`
- `framework/tests/telemetry_smoke.rs`

## Info

### IN-01: Duplicated default threshold constant `102_400`

**File:** `framework/src/telemetry/inline_budget.rs:101-105`
**Also:** `framework/src/config/providers/app.rs:27`

**Issue:** The default threshold `102_400` is hardcoded in two places —
`AppConfig::from_env` (the canonical source) and in `decide()` as the
`unwrap_or(102_400)` fallback when `Config::get::<AppConfig>()` returns `None`.
Drift between the two would silently change behaviour depending on whether the
`AppConfig` is registered in the container.

**Evidence:**

```rust
// inline_budget.rs:101-105
let threshold = crate::Config::get::<crate::AppConfig>()
    .map(|c| c.inline_budget_threshold_bytes)
    .unwrap_or(102_400);
```

```rust
// app.rs:27
inline_budget_threshold_bytes: env("INLINE_BUDGET_BYTES", 102_400usize),
```

**Recommendation:** Promote the default to a single `pub const DEFAULT_INLINE_BUDGET_THRESHOLD_BYTES: usize = 102_400;` in `inline_budget.rs` (or alongside `AppConfig`) and reference it from both call sites. Optionally expose it in the public re-exports so callers can refer to it symbolically rather than the magic number.

---

### IN-02: `record_and_decide` uses two HashSet operations where one suffices

**File:** `framework/src/telemetry/inline_budget.rs:75-86`

**Issue:** The fire-once guard does `contains(key)` then `insert(key.to_string())`. `HashSet::insert` already returns `bool` (`true` if newly inserted, `false` if already present). One call would do the same work and avoid hashing `key` twice — and avoids the `to_string()` allocation entirely on the second-and-later crossings.

**Evidence:**

```rust
if !self.warned.contains(key) {
    self.warned.insert(key.to_string());
    tracing::warn!(...);
}
```

**Recommendation:**

```rust
if self.warned.insert(key.to_string()) {
    tracing::warn!(...);
}
```

This makes the fire-once semantics structurally guaranteed (a single atomic test-and-set) rather than relying on the reader to verify the surrounding `contains`/`insert` pair. Correctness is unchanged; readability and a small alloc-on-hot-path win.

---

### IN-03: `AppConfig` programmatic override path is silently dependent on container registration

**File:** `docs/src/the-basics/inline-budget-and-telemetry.md:82-89`
**Code:** `framework/src/telemetry/inline_budget.rs:103-105`

**Issue:** The docs show:

```rust,ignore
let cfg = AppConfigBuilder::default()
    .inline_budget_threshold_bytes(204_800)
    .build();
```

…but `decide()` reads `crate::Config::get::<crate::AppConfig>()`, which only returns the *registered* `AppConfig` in the container. Building an `AppConfig` and binding it to a local variable has no effect on `inline_budget` decisions — the user must also register the built config with the container. The docs don't show the registration step, so a reader following the example will silently get the `INLINE_BUDGET_BYTES` env value (or the `102_400` fallback) instead of `204_800`.

**Evidence:** No code excerpt — this is a documentation completeness issue rather than a code defect.

**Recommendation:** Either (a) extend the docs example to show the container-registration step (`Config::set(cfg)` or whatever the actual API is), or (b) add a one-line note: "The built config must be registered with the framework's container before `inline_budget` will honour the override; see the Config chapter."

---

### IN-04: `RING_BUFFER_CAPACITY` is `pub(crate)` — operators have no programmatic way to read it

**File:** `framework/src/telemetry/request_telemetry.rs:65`

**Issue:** The cap is documented as `128` in the docs page and in the module-level doc comment, but is exposed only as `pub(crate) const RING_BUFFER_CAPACITY`. An operator dashboard that wants to render "showing X of N samples" has to hardcode `128` on its side, recreating the drift risk solved by IN-01.

**Evidence:**

```rust
pub(crate) const RING_BUFFER_CAPACITY: usize = 128;
```

**Recommendation:** Promote to `pub const` and add it to the re-export list at `framework/src/lib.rs:183` (`pub use telemetry::{Decision, RequestTelemetry, Sample, RING_BUFFER_CAPACITY}`). Trivial change; locks the constant as the single source of truth.

---

### IN-05: Smoke test does not exercise the once-per-request warning channel or `RequestTelemetry::keys`

**File:** `framework/tests/telemetry_smoke.rs`

**Issue:** The integration test covers the happy paths for `inline_budget` (Inline → Preload transition) and the unscoped/scoped `telemetry_record*` round-trips, but does not:

1. Assert that the `tracing::warn!` fires exactly once across multiple over-threshold calls — that contract lives entirely in the unit tests at `inline_budget.rs:197-213`.
2. Exercise `RequestTelemetry::keys()` end-to-end.

Both are covered by unit tests in their own modules, but the integration test is the only place where the full `Request` → `decide` → `record_and_decide` path is exercised together. A `tracing-subscriber` capture (or `tracing_test`) here would catch a future regression where the once-per-request guard is lost when the state moves between `Request::extensions` and the state machine.

**Evidence:** No code excerpt — coverage gap, not a defect.

**Recommendation:** Optional. If you add a `tracing-test` capture, also extend the smoke test to call `RequestTelemetry::keys()` after the two `telemetry_record*` calls and assert the expected `(key, scope)` pairs are present. Both are low-cost additions; both close the "integration vs unit" coverage gap.

---

### IN-06: `Sample.value: serde_json::Value` is unbounded in serialized size

**File:** `framework/src/telemetry/request_telemetry.rs:17-22`

**Issue:** `Sample.value` is a caller-supplied `serde_json::Value` with no enforced size cap. Combined with `RING_BUFFER_CAPACITY = 128` per `(key, scope)`, a caller passing a multi-megabyte `Value` per sample could pin tens of megabytes per bucket. The docs warn about `(key, scope)` cardinality (correct), but say nothing about per-sample payload size.

This is consistent with the "lost on process restart" framing and the explicit "caller discipline" stance taken everywhere else in this module — so I'd classify this as a docs gap, not a code defect.

**Evidence:**

```rust
pub struct Sample {
    pub recorded_at: SystemTime,
    pub value: serde_json::Value,
}
```

**Recommendation:** Add a short bullet in the "Sample shape" section of `docs/src/the-basics/inline-budget-and-telemetry.md`: "Sample payload size is unbounded; keep payloads small (a few hundred bytes typical). The ring buffer holds 128 samples per `(key, scope)` — payload size × 128 is the bucket's memory ceiling." Same "caller discipline, not framework enforcement" framing as the cardinality note already there.

---

## Summary

Phase 184 ships two well-scoped primitives. The `InlineBudget` state machine is correctly split into a pure `record_and_decide` (unit-testable without a `Request`) and a `Request`-bound `decide` wrapper that correctly orders the `&self` reads before the `&mut self` extension borrow — exactly the pitfall the RESEARCH artifact called out. The fire-once warning logic is correct (though IN-02 has a small idiomatic win). Ring-buffer cap enforcement is correct (`while entry.len() > CAPACITY` is robust even though `push_back` only adds 1 per call). DashMap usage is sound — `record` and `snapshot` each touch one shard and hold no cross-shard locks, so no deadlock is possible. The `OnceLock<DashMap>` global is properly lazy-initialized.

`tracing::warn!` uses structured fields, which protects against log-injection through the standard subscribers (JSON, structured key-value). The docs correctly warn that `fallback_url` must not be user-controlled — this is the right place for that warning since the framework cannot enforce it without sanitising URLs (which would defeat the use case).

Cross-cutting themes:

- **One single-source-of-truth gap** (IN-01) — the `102_400` default is duplicated.
- **One documentation completeness gap** (IN-03) — programmatic `AppConfig` override path doesn't mention container registration.
- **One docs gap** (IN-06) — per-sample payload size discipline isn't called out.
- **One ergonomic exposure gap** (IN-04) — `RING_BUFFER_CAPACITY` isn't `pub`.
- **One micro-refactor** (IN-02) — `insert` returns `bool`, no need for `contains` + `insert`.
- **One optional coverage extension** (IN-05) — warning channel + `keys()` round-trip would benefit from integration-level assertions.

All findings are `info`. None block the v0.2.44 ship. IN-01 + IN-02 + IN-04 are trivially addressable in a single follow-up commit; IN-03 + IN-06 are docs touch-ups; IN-05 is optional.
