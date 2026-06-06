---
phase: 184-ferro-inlinebudget-ferro-requesttelemetry
plan: 01
subsystem: infra
tags: [telemetry, dashmap, oncelock, ring-buffer, serde, tracing, appconfig]

# Dependency graph
requires: []
provides:
  - "framework/src/telemetry/ module with Sample struct, Decision enum, RequestTelemetry namespace, OnceLock<DashMap<(String, Option<String>), VecDeque<Sample>>> global store, private record() writer, snapshot/keys/clear/reset operator methods"
  - "AppConfig::inline_budget_threshold_bytes: usize field (default 102_400, env override INLINE_BUDGET_BYTES) with parallel AppConfigBuilder setter"
  - "Crate-root re-exports of Decision, RequestTelemetry, Sample at framework/src/lib.rs (InlineBudget intentionally NOT re-exported per D-02/OQ2)"
affects:
  - 184-02 (Plan 02 wires Request methods inline_budget / telemetry_record / telemetry_record_scoped and implements the decide() body on top of these types and storage)
  - 184-03 (Plan 03 ships integration test, docs page, workspace version bump on top of Plan 02's API)

# Tech tracking
tech-stack:
  added: []  # All deps were already direct deps of ferro-rs (dashmap 6, tracing 0.1, serde 1, serde_json 1, serial_test 3 dev-dep)
  patterns:
    - "OnceLock<DashMap<K, V>> process-global registry with #[cfg(test)] reset() helper (mirrored verbatim from ferro-bundle Phase 183)"
    - "VecDeque ring-buffer cap enforced via `while len > N { pop_front() }` after push_back (CONTEXT D-08, RESEARCH Pitfall 3)"
    - "Additive AppConfig field with env<T: FromStr>(name, default) reader + builder setter (preserves backward-compat)"
    - "Type alias for complex generic storage type to satisfy clippy::type_complexity without #[allow(...)]"

key-files:
  created:
    - "framework/src/telemetry/mod.rs"
    - "framework/src/telemetry/request_telemetry.rs"
    - "framework/src/telemetry/inline_budget.rs"
  modified:
    - "framework/src/config/providers/app.rs"
    - "framework/src/lib.rs"

key-decisions:
  - "InlineBudget is NOT re-exported at the crate root — only Decision, RequestTelemetry, Sample. User never constructs InlineBudgetState directly (locked per D-02 / OQ2)."
  - "RING_BUFFER_CAPACITY = 128 declared as pub(crate) const (not a config knob) — D-08 locks the value as global, no per-key override in v1."
  - "decide() body deferred entirely to Plan 02 — Plan 01 ships only the Decision enum + InlineBudgetState struct shape. Keeps Plan 01 fully self-contained and risk-isolated from Request integration."
  - "TELEMETRY_STORE typedef'd as BucketKey + TelemetryStore aliases to resolve clippy::type_complexity (deviation Rule 1; chose type aliases over #[allow] to keep the surface readable)."

patterns-established:
  - "Pre-1.0 telemetry storage convention: process-global, lost on restart, bounded ring buffer per (key, scope); long histories belong in external systems (Prometheus, OpenTelemetry, custom DB sinks). Documented at module level in mod.rs."
  - "Caller controls (key, scope) vocabulary — user-controlled strings MUST NOT be passed as key/scope. Documented as a security note at module level (mitigates T-184-01 partial DoS from unbounded bucket growth)."

requirements-completed: [SC-3a, SC-3b, SC-3c, SC-4]

# Metrics
duration: 17min
completed: 2026-06-06
---

# Phase 184 Plan 01: Foundation Types and Storage Summary

**Telemetry module foundation — Sample/Decision/InlineBudgetState types, OnceLock<DashMap> process-global ring buffer (cap 128), AppConfig inline_budget_threshold_bytes field, crate-root re-exports of Decision/RequestTelemetry/Sample (InlineBudget intentionally hidden).**

## Performance

- **Duration:** 17 min
- **Started:** 2026-06-06T20:01:25Z
- **Completed:** 2026-06-06T20:18:39Z
- **Tasks:** 3 (TDD on Tasks 1 + 2; gate-only on Task 3)
- **Files modified:** 5 (3 created, 2 modified)

## Accomplishments

- Shipped `framework/src/telemetry/` module with `Sample`, `Decision`, `InlineBudgetState`, `RequestTelemetry` types
- Built process-global `OnceLock<DashMap<(String, Option<String>), VecDeque<Sample>>>` storage with 128-sample ring buffer per bucket (D-08), private `record()` writer, public `RequestTelemetry::snapshot`/`keys`/`clear` operator methods, `#[cfg(test)] reset()` test-isolation helper
- Added `AppConfig::inline_budget_threshold_bytes: usize` field (default 102_400, env `INLINE_BUDGET_BYTES` override) plus parallel `AppConfigBuilder::inline_budget_threshold_bytes(n)` setter — purely additive, no breaking change
- Wired the module into `framework/src/lib.rs` with `pub mod telemetry;` declaration plus flat re-export `pub use telemetry::{Decision, RequestTelemetry, Sample};` (InlineBudget intentionally NOT re-exported per D-02 / OQ2)
- 13 new unit tests landed (8 in `telemetry::request_telemetry::tests`, 2 in `telemetry::inline_budget::tests`, 3 in `config::providers::app::tests`) — all green; full ferro-rs lib suite at 542 tests passing
- Pre-commit gate (fmt + clippy `-D warnings` + cargo test --all-features) shipped green

## Task Commits

Each task was committed atomically:

1. **Task 1: telemetry module + Sample + RequestTelemetry storage + Decision** — `8d935683` (feat)
2. **Task 2: AppConfig field + builder + crate-root re-exports** — `ee0ba19e` (feat)
3. **Task 3: pre-commit gate (fmt + clippy + test) green** — `eb5e7c36` (chore)

## Files Created/Modified

**Created:**
- `framework/src/telemetry/mod.rs` — module root with `//!` docs covering lost-on-restart semantic, 100 KB default, controlled-vocabulary security note; declares `pub mod inline_budget; pub mod request_telemetry;` and re-exports `Decision`, `RequestTelemetry`, `Sample`.
- `framework/src/telemetry/request_telemetry.rs` — `Sample` struct (SystemTime + serde_json::Value, derives Debug/Clone/Serialize/Deserialize), `Sample::now` + `Sample::at` constructors, `RequestTelemetry` unit struct namespacing static methods, `BucketKey` + `TelemetryStore` type aliases, `OnceLock<TelemetryStore>` global, `RING_BUFFER_CAPACITY = 128` const, private `record(key, scope, sample)` writer, public `snapshot`/`keys`/`clear` methods, `#[cfg(test)] reset()` helper, 8 inline unit tests (all `#[serial]`).
- `framework/src/telemetry/inline_budget.rs` — `Decision` enum (`Inline | Preload(String)`, derives Debug/Clone/PartialEq/Eq), `pub(crate) InlineBudgetState` struct with `cumulative: HashMap<String, usize>` + `warned: HashSet<String>` fields, 2 smoke tests. `decide()` body deferred to Plan 02 (intentional — see Decisions).

**Modified:**
- `framework/src/config/providers/app.rs` — added `pub inline_budget_threshold_bytes: usize` field to `AppConfig` struct, `inline_budget_threshold_bytes: env("INLINE_BUDGET_BYTES", 102_400usize)` line in `AppConfig::from_env()`, `inline_budget_threshold_bytes: Option<usize>` field to `AppConfigBuilder`, `pub fn inline_budget_threshold_bytes(mut self, bytes: usize) -> Self` builder setter, `inline_budget_threshold_bytes: self.inline_budget_threshold_bytes.unwrap_or(default.inline_budget_threshold_bytes)` line in `build()`, and 3 `#[serial]` unit tests (default, env override, builder override).
- `framework/src/lib.rs` — added `pub mod telemetry;` module declaration (with one-line doc comment) and `pub use telemetry::{Decision, RequestTelemetry, Sample};` flat re-export.

## Decisions Made

All decisions in this plan follow CONTEXT.md D-01..D-15 verbatim. Plan-time clarifications worth recording:

- **InlineBudget hidden by design.** D-02 says the user never constructs `InlineBudgetState` directly — the only public surface is `Decision`, the `Request::inline_budget` method (Plan 02), and the threshold config knob. `framework/src/lib.rs` re-exports only `Decision`, `RequestTelemetry`, `Sample`. `grep -c InlineBudget framework/src/lib.rs` returns 0. (Resolves OQ2 — CONTEXT D-11 had a drafting error including `InlineBudget` in its re-export list; D-02 + the user prompt supersede.)
- **`decide()` body deferred to Plan 02.** Plan 01 ships only the `Decision` enum and `InlineBudgetState` struct shape (`pub(crate)` with `pub(crate)` fields). The borrow-checker-safe ordering for reading config + extensions, the once-per-request `tracing::warn!` site, and the per-key cumulative-bytes state machine all land in Plan 02 alongside the `Request::inline_budget` method. Keeps Plan 01 fully isolated from `Request` API changes.
- **Type aliases over `#[allow(clippy::type_complexity)]`.** Clippy flagged `OnceLock<DashMap<(String, Option<String>), VecDeque<Sample>>>` as too complex. Resolved by extracting `BucketKey = (String, Option<String>)` and `TelemetryStore = DashMap<BucketKey, VecDeque<Sample>>` aliases. The storage shape locked in D-10 is unchanged; the aliases just make the type-name reusable and the surface readable. No `#[allow]` introduced.
- **`#[allow(dead_code)]` on `record()` and `InlineBudgetState` fields.** Until Plan 02 wires `Request::telemetry_record` and implements `decide()`, the private writer and the state fields are unused. Suppressed locally with a one-line comment pointing forward to Plan 02; both attributes will become unnecessary in Plan 02 (and can be removed there if not auto-elided).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Lint Failure] Extract type aliases for `OnceLock<DashMap<(String, Option<String>), VecDeque<Sample>>>`**
- **Found during:** Task 3 (pre-commit gate)
- **Issue:** `cargo clippy --all --all-targets -- -D warnings` flagged `clippy::type_complexity` on the `TELEMETRY_STORE` declaration. CI uses this exact command (CLAUDE.md project rule), so the workspace would fail CI without a fix.
- **Fix:** Extracted `type BucketKey = (String, Option<String>);` and `type TelemetryStore = DashMap<BucketKey, VecDeque<Sample>>;` at module scope. Updated `TELEMETRY_STORE` static and `telemetry_store()` accessor to use the aliases. Storage shape unchanged — D-10 still locks the underlying type.
- **Files modified:** `framework/src/telemetry/request_telemetry.rs`
- **Verification:** `cargo clippy --all --all-targets -- -D warnings` clean; all 10 telemetry tests still pass.
- **Committed in:** `eb5e7c36` (Task 3 commit)

**2. [Rule 1 - Lint Failure] Suppress dead-code warning on `record()` and `InlineBudgetState` fields**
- **Found during:** Task 3 (pre-commit gate, post-fmt)
- **Issue:** `record()` is called only from `Request::telemetry_record` (Plan 02), and `InlineBudgetState.cumulative` / `.warned` are read only by `decide()` (Plan 02). Without Plan 02 wired in, rustc warns "function/field is never used", and clippy `-D warnings` rejects the build.
- **Fix:** Added `#[allow(dead_code)] // Plan 02 wires Request::telemetry_record to this.` on `record()` and `#[allow(dead_code)] // Plan 02 reads cumulative and warned to drive the decision.` on `InlineBudgetState`. Local-scoped attributes with explicit forward-pointer comments — not a blanket suppression.
- **Files modified:** `framework/src/telemetry/request_telemetry.rs`, `framework/src/telemetry/inline_budget.rs`
- **Verification:** clippy `-D warnings` clean; the two attributes are visible in source for the Plan 02 executor to remove if rustc auto-resolves them after wiring `Request` methods.
- **Committed in:** `8d935683` (Task 1 commit — applied inline at file-creation time rather than retroactively)

**3. [Rule 1 - Format] cargo fmt collapsed multi-line record() in concurrent test**
- **Found during:** Task 3 (`cargo fmt --all -- --check`)
- **Issue:** Original test wrote the multi-arg `record(...)` call across 5 lines for readability; rustfmt collapsed it onto a single line (fits within line width).
- **Fix:** Ran `cargo fmt --all` once; verified `--check` clean afterward.
- **Files modified:** `framework/src/telemetry/request_telemetry.rs`
- **Committed in:** `eb5e7c36` (Task 3 commit)

---

**Total deviations:** 3 auto-fixed (3× Rule 1 — lint/format failures during the gate)
**Impact on plan:** All three are mechanical adjustments to satisfy the CI-equivalent gate (CLAUDE.md mandates `cargo fmt + clippy -D warnings + test` before every commit). Storage shape, public surface, and semantics are unchanged from the plan. No scope creep.

## Issues Encountered

- **Transient ferro-api-mcp e2e failure (host environment, not Plan 184).** During the first `cargo test --all-features` run the host filesystem hit 100% capacity (`/dev/disk3s5: 3.8 GiB free`). One SQLite migration in `ferro-api-mcp/tests/e2e.rs::test_openapi_spec_served` aborted with "database or disk is full". The failure occurred entirely inside a crate Plan 184 does not touch (telemetry primitives live in `framework`, not `ferro-api-mcp`). Re-running the same test after the host freed disk space (back to 93% / 31 GiB free) returned green on the first try. Classified as environmental — see CLAUDE.md "Scope Boundary" (out-of-scope discoveries are not fixed by execute-plan). No code change applied.

## User Setup Required

None — Plan 01 is pure framework-internal additive surface. Consumers will not see the new types until Plan 02 (Request methods) ships; even then no setup is required because the env override is optional with a 100 KB default.

## Next Phase Readiness

Plan 02 can land directly on top of this foundation:

- Types and storage are present and unit-tested. The Plan 02 executor will: (a) implement `pub(crate) fn decide(req, key, bytes, fallback_url) -> Decision` in `framework/src/telemetry/inline_budget.rs` using the borrow-checker-safe ordering documented in PATTERNS.md (read `Config::get::<AppConfig>()` and `req.route_pattern()` first, then `get_mut::<InlineBudgetState>`); (b) add the once-per-request `tracing::warn!` site; (c) add the three thin delegators `inline_budget`, `telemetry_record`, `telemetry_record_scoped` to the second `impl Request` block at `framework/src/http/request.rs:742-777` next to the existing `flash`/`redirect_to` methods. Both `#[allow(dead_code)]` attributes from Plan 01 should become removable once Plan 02 wires the `Request` methods.
- AppConfig is consumable via `crate::Config::get::<crate::AppConfig>().map(|c| c.inline_budget_threshold_bytes).unwrap_or(102_400)` per RESEARCH Q5 + Pitfall 5; the `.unwrap_or(102_400)` fallback is mandatory for unit tests that bypass `Config::init`.
- Pre-commit gate is green at this commit — no inherited lint debt for Plan 02 to clean up.
- `InlineBudget` is not in the crate-root re-export list and must stay out — Plan 02 must not introduce a `pub use telemetry::InlineBudget;` line.

## Self-Check: PASSED

**Files verified to exist:**
- `framework/src/telemetry/mod.rs` — FOUND
- `framework/src/telemetry/request_telemetry.rs` — FOUND
- `framework/src/telemetry/inline_budget.rs` — FOUND
- `framework/src/config/providers/app.rs` — FOUND (modified)
- `framework/src/lib.rs` — FOUND (modified)

**Commits verified in git log:**
- `8d935683` — FOUND (Task 1: feat(184-01): telemetry module — Sample + RequestTelemetry + Decision)
- `ee0ba19e` — FOUND (Task 2: feat(184-01): AppConfig.inline_budget_threshold_bytes + crate-root re-exports)
- `eb5e7c36` — FOUND (Task 3: chore(184-01): pre-commit gate green (fmt + clippy + test))

**OQ2 confirmation:** `grep -c InlineBudget framework/src/lib.rs` returns 0. `InlineBudget` is not re-exported at the crate root.

---
*Phase: 184-ferro-inlinebudget-ferro-requesttelemetry*
*Plan: 01-foundation-types-and-storage*
*Completed: 2026-06-06*
