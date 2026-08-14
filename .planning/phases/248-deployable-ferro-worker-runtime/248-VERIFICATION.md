---
phase: 248-deployable-ferro-worker-runtime
verified: 2026-08-14T00:00:00Z
status: human_needed
score: 9/10 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Multi-process OFFLOAD-05 UAT: run `./target/debug/app serve --no-worker` in terminal 1, `./target/debug/app worker --queue reports` in terminal 2, `./target/debug/app worker --queue default` in terminal 3. Enqueue a batch of jobs on both queues. Observe each job processed exactly once (no duplicates across workers). Confirm the reports worker does not claim default jobs and vice versa. Kill terminal 2 and verify terminal 3 continues processing default jobs unimpeded. Finally run `./target/debug/app worker` (no flag) and confirm it consumes all registered queues."
    expected: "Each job is processed exactly once by exactly one worker. Queue scoping is respected. A failed worker class does not stall an unrelated class. The all-queues default produces a WorkerLoop consuming all registered queues."
    why_human: "True multi-process at-least-once exactly-once guarantee requires live DB accessible to multiple OS processes. The in-process SC#2 two-loop test is an adequate proxy, but the VALIDATION.md explicitly designates cross-process runtime behaviour as Manual-Only for OFFLOAD-05."
  - test: "Cross-replica broadcast delivery over Redis transport (WR-01): with `BROADCAST_REDIS_URL` set and the `redis-transport` feature enabled, run two web replicas. Publish a delta on replica A and confirm a client subscribed on replica B receives it."
    expected: "Delta published on replica A reaches subscriber on replica B via the shared RedisTransport."
    why_human: "Requires a live redis-server and two separate processes. The automated worker_boot.rs WR-01 scenario skips without REDIS_URL."
---

# Phase 248: Deployable Ferro Worker Runtime — Verification Report

**Phase Goal:** Make background capacity horizontally scalable — a deployable consumer process (the same app binary, run as `<app-bin> worker [--queue <name>]`) that consumes queued jobs at N replicas with at-least-once idempotent ack and fault-domain isolation; `serve` keeps its in-process worker and gains `--no-worker` for scale-out web replicas. Capacity scales by running more workers — NO framework autoscaler (explicitly deferred to 2.0).
**Verified:** 2026-08-14
**Status:** human_needed
**Re-verification:** No — initial verification.

---

## Goal Achievement

### Observable Truths

All must-haves are drawn from four sources merged per Step 2c: ROADMAP.md success criteria (SC#1–SC#4), PLAN frontmatter truths across Plans 00–03, and the phase goal narrative. The SC#1 `--class` phrasing in ROADMAP is explicitly annotated in the ROADMAP itself as loose shorthand for the decided `<app-bin> worker --queue <name>` surface (248-CONTEXT.md); that annotation is treated as the authoritative form.

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `<app-bin> worker --queue <name>` runs a process consuming only that queue's jobs (SC#1 / D-01/D-02) | VERIFIED | `Worker { queue: Vec<String> }` with `ArgAction::Append` in `app/src/main.rs:85–88`; wired to `ferro::run_worker(..., queue)` at line 154. `./target/debug/app worker --help` shows `--queue` (pre-confirmed by orchestrator). |
| 2 | Two worker replicas against one queue split work without double-processing (SC#2, in-process proxy) | VERIFIED | `two_loops_split_work_no_duplicates()` in `ferro-queue/tests/worker_runtime.rs:172` asserts `unique.len() == all.len() == N` (no duplicate). Suite passes (1 passed, 0 failed). Multi-process assertion is human-UAT. |
| 3 | Saturating one worker class does not stall an unrelated class — fault-domain isolation (SC#3) | VERIFIED | `queue_fault_isolation()` at line 252 uses `tokio::sync::Barrier::new(2)` (no `time::sleep`) to coordinate concurrent drains on `"media"` and `"reports"`. Asserts all `"reports"` jobs claimed while `"media"` still has backlog. Passes. |
| 4 | No framework-managed autoscaling introduced (SC#4) | VERIFIED | `grep -rniE "autoscal\|scale_to_zero\|KEDA" framework/src/` filtered for non-FakeDatabase matches returns empty. Only pre-existing `FakeDatabase` substring in `container/testing.rs:58` (not autoscaling code). |
| 5 | Single shared boot surface `run_common_boot`; serve and run_worker do not fork bootstrap logic | VERIFIED | `framework/src/app.rs:423` defines `pub async fn run_common_boot(bootstrap_fn, no_worker)`. `run_server_internal` calls it (line 567 shows `Server::from_config` only in `run_server_internal`, not in `run_common_boot`). `run_worker` calls `run_common_boot(…, no_worker=true)` at line 549. Module-level free functions at lines 710 and 719 re-export both. |
| 6 | `Queue::registered_queue_names()` returns distinct declared-queue set (≥ `["default"]`) (D-05) | VERIFIED | `ferro-queue/src/db.rs:84` defines `pub fn registered_queue_names() -> Vec<String>` using `BTreeSet` (deterministic order) at lines 85–98. `WorkerConfig::default()` replaced by `registered_queue_names()` in `framework/src/app.rs:529`. No `WorkerConfig::default()` remains in framework. |
| 7 | `#[offload(queue = "name")]` parses; bare `#[offload]` still works; unknown args produce a clear compile error (D-04) | VERIFIED | `parse_nested_meta` in `ferro-macros/src/service.rs:199`; error message `unknown #[offload] argument; expected \`queue = "name"\`` at line 206. `declared_queue: Option<String>` field in `ferro-macros/src/offload.rs:67`. Trybuild UI gate 9/9 (queue_arg.rs pass; queue_unknown_arg.rs fail with expected stderr). |
| 8 | Declared queue threaded into emitted `JobRegistrarEntry` (`queue: Some("name")`) and into `Offloadable::offload()` (`.on_queue("name")`) | VERIFIED | `ferro-macros/src/offload.rs:323–334` defines `queue_name_tokens` and `on_queue_tokens` conditionally. Emits `queue: #queue_name_tokens` at line 410 and `#on_queue_tokens` in the offload override body at line 345. No `::ferro_queue::` bare paths in emission. |
| 9 | WR-01: when `redis-transport` is off and `transport_redis_url` is set, `tracing::warn!` fires and in-process hub used — no panic (D-07) | VERIFIED | `framework/src/app.rs:489–497`: `#[cfg(not(feature = "redis-transport"))]` branch emits `tracing::warn!("BROADCAST_REDIS_URL is set but the \`redis-transport\` feature is disabled…")`. `framework/tests/worker_boot.rs:93` calls `ferro::run_common_boot(None, true)` with a pre-registered Broadcaster with `transport_redis_url = Some(...)`. Test passes (no panic; `App::get::<Broadcaster>()` is `Some` after boot). |
| 10 | Multi-process OFFLOAD-05 runtime behaviour: N worker processes split shared-queue load exactly once; `serve --no-worker` accepts HTTP without spawning an in-process worker | UNCERTAIN | Cannot verify programmatically — requires live multi-process setup. Designated Manual-Only in 248-VALIDATION.md. |

**Score:** 9/10 truths verified (1 requires human verification)

---

### Deferred Items

None — all truths are either verified or classified as human-UAT items within this phase's own scope.

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-queue/tests/worker_runtime.rs` | SC#1/SC#2/SC#3 as sub-functions of one `#[tokio::test]`, NamedTempFile, Barrier | VERIFIED | Exists, 300+ lines, single `#[tokio::test]`, three named sub-functions, `NamedTempFile` at each scenario, `Barrier` in SC#3. No `time::sleep`. |
| `framework/tests/worker_boot.rs` | WR-01 bootstrap + D-07 scenario, `extern crate ferro_rs as ferro`, redis feature gating | VERIFIED | Exists. `extern crate ferro_rs as ferro` at line 20. `transport_url_no_feature_warns` drives real `ferro::run_common_boot` at line 93. Redis scenario behind `#[cfg(feature = "redis-transport")]` with `REDIS_URL` skip guard. No `TODO(plan-01)`. |
| `ferro-macros/tests/ui/offload/pass/queue_arg.rs` | Pass fixture with `#[offload(queue = "reports")]` | VERIFIED | Exists, contains `#[offload(queue = "reports")]` at line 24. Compiles (9/9 trybuild gate). |
| `ferro-macros/tests/ui/offload/fail/queue_unknown_arg.rs` | Fail fixture with rejectable arg | VERIFIED | Exists, uses `#[offload(retries = 3)]` — an argument the macro rejects. |
| `ferro-macros/tests/ui/offload/fail/queue_unknown_arg.stderr` | Regenerated stderr, no placeholder | VERIFIED | Contains real error text (`unknown #[offload] argument…` + E0405 cascade). Grep for "regenerate" returns nothing — placeholder replaced. |
| `ferro-queue/src/db.rs` | `registered_queue_names()` + `queue: Option<&'static str>` on `JobRegistrarEntry` | VERIFIED | Both present: `pub fn registered_queue_names()` at line 84; `pub queue: Option<&'static str>` at line 129. BTreeSet used for ordering. |
| `framework/src/app.rs` | `run_common_boot`, `run_worker`, WR-01 `with_transport`, D-07 `tracing::warn!`, `registered_queue_names()` call, no `WorkerConfig::default()` | VERIFIED | All present. `run_common_boot` at 423, `run_worker` at 548 (struct method) and 710 (free function). `with_transport` at 448. D-07 warn at 492. `registered_queue_names()` at 529. No `WorkerConfig::default()` remains. `Server::from_config` only in `run_server_internal`, not in `run_common_boot`. |
| `framework/src/lib.rs` | `run_worker` and `run_common_boot` re-exported | VERIFIED | Line 82: `pub use app::{run_common_boot, run_worker, Application}`. |
| `app/src/main.rs` | `Worker { queue: Vec<String> }`, `ArgAction::Append`, `no_worker` on Serve, `run_worker` wiring, no wildcard Serve arm | VERIFIED | All present. No `Commands::Serve { .. }` wildcard. Three Serve arms each destructure `no_worker` explicitly. Worker arm at line 152–155. |
| `app/src/bootstrap.rs` | `with_config(BroadcastConfig::from_env())` and `App::singleton` retained; no `with_transport` | VERIFIED | Lines 187–188 retain construction and registration. `with_transport` absent from bootstrap.rs. |
| `ferro-macros/src/service.rs` | `parse_nested_meta` and exact error message for unknown args | VERIFIED | `parse_nested_meta` at line 199; error message at line 206. |
| `ferro-macros/src/offload.rs` | `declared_queue: Option<String>`, `queue_name_tokens`, `on_queue_tokens`, no `::ferro_queue::` | VERIFIED | All present. Emission uses `::ferro::queue::*` paths exclusively. |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `app/src/main.rs: Commands::Worker { queue }` | `ferro::run_worker(Some(bootstrap_fn), queue)` | Worker match arm line 152–155 | WIRED | Exact wiring confirmed in main.rs. |
| `app/src/main.rs: Serve { no_migrate, no_worker }` | `run_server(no_worker)` → `ferro::run_common_boot(…, no_worker)` | `run_server(no_worker: bool)` at line 180; passes flag to `run_common_boot` at line 186 | WIRED | Confirmed. No wildcard arm suppresses the field. |
| `framework/src/app.rs::run_common_boot` | `ferro_broadcast::Broadcaster::with_transport` | `#[cfg(feature = "redis-transport")]` branch at line 442–448 | WIRED | `with_transport(Arc::new(t))` call confirmed. |
| `framework/src/app.rs::run_common_boot` | `ferro_queue::Queue::registered_queue_names` | In-process worker spawn at line 529 | WIRED | `let all_queues = ferro_queue::Queue::registered_queue_names()` then `WorkerConfig::new(all_queues)`. |
| `framework/src/app.rs::run_worker` | `framework/src/app.rs::run_common_boot` | `Self::run_common_boot(bootstrap_fn, true).await` at line 549 | WIRED | Single call, no duplication. |
| `ferro-macros/src/service.rs` | `ferro-macros/src/offload.rs::collect_info` | `declared_queue: Option<String>` passed into `collect_info` | WIRED | `collect_info(&trait_ident, method, declared_queue)` call site confirmed at service.rs, `declared_queue` parameter in `offload.rs:137`. |
| `ferro-macros/src/offload.rs::emit_job_items` | `JobRegistrarEntry { queue }` and `Offloadable::offload()` with `.on_queue()` | `queue_name_tokens` at line 323; `on_queue_tokens` at line 327; emitted at lines 345, 410 | WIRED | Both token fragments confirmed present and conditional on `declared_queue`. |
| `ferro-queue/tests/worker_runtime.rs` | `WorkerConfig / claim / enqueue` via NamedTempFile SQLite | `NamedTempFile` at each scenario; `claim`, `enqueue`, `delete_job` used throughout | WIRED | Confirmed. `use ferro_queue::{claim, delete_job, enqueue, CreateJobsTable, WorkerConfig, WorkerLoop}` in test imports. |

---

### Data-Flow Trace (Level 4)

Plan 248 does not introduce new rendering components or dashboard pages. The key data-flow concern is the queue-routing pipeline from `#[offload(queue = "name")]` through to the claim query.

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `ferro-macros/src/offload.rs` emit | `queue_name_tokens` / `on_queue_tokens` | `info.declared_queue` from parsed `#[offload(queue = "name")]` attr | Yes — compile-time literal, flows to `JobRegistrarEntry.queue` (DB claim filter) and `PendingDispatch::on_queue()` (dispatch routing) | FLOWING |
| `ferro-queue/src/db.rs::registered_queue_names()` | `names: BTreeSet<String>` | `inventory::iter::<JobRegistrarEntry>` + `JOB_REGISTRARS` mutex | Yes — reads live registrar data; falls back to `["default"]` when empty (never hollow) | FLOWING |
| `framework/src/app.rs::run_common_boot` | `all_queues` (in-process worker) | `Queue::registered_queue_names()` | Yes — reads from registrar | FLOWING |
| `framework/src/app.rs::run_worker` | `effective_queues` | CLI `queues` arg (if non-empty) else `registered_queue_names()` (D-03) | Yes — either explicit operator input or live registrar | FLOWING |

---

### Behavioral Spot-Checks

Pre-confirmed by orchestrator evidence (reused per thermal constraint):

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `worker --help` shows `--queue` | `./target/debug/app worker --help` | Shows `--queue` option | PASS |
| `serve --help` shows `--no-worker` | `./target/debug/app serve --help` | Shows `--no-worker` option | PASS |
| SC#1–SC#3 suite resolves to exactly one test | `cargo test -p ferro-queue --test worker_runtime -- --list` | `worker_runtime_suite: test` (1 test, 0 benchmarks) | PASS |
| SC#1–SC#3 suite passes | `cargo test -p ferro-queue --test worker_runtime` | `ok. 1 passed; 0 failed` | PASS |
| Single `#[tokio::test]` (test-collapse guard) | `grep -c "#\[tokio::test"` | `1` | PASS |
| Trybuild UI gate 9/9 | `cargo test -p ferro-macros --test offload_macro` | 9/9 passed | PASS |
| Full CI-exact gate | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | Green (exit 0) | PASS |
| SC#4 structural guard | `grep -rniE "autoscal\|scale_to_zero\|KEDA" framework/src/` (filtered) | Empty | PASS |

---

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| OFFLOAD-05 | Plans 00–03 | Offloaded work runs on a deployable `ferro worker` process runnable at N replicas; fault-domain isolation; no autoscaling | SATISFIED (multi-process UAT pending) | CLI surface verified in main.rs; automated SC#1–SC#3 green; SC#4 structural guard clean; multi-process runtime is human-UAT per VALIDATION.md |

Note: The REQUIREMENTS.md traceability table still reads `Not started` for OFFLOAD-05 at row 75 — this is a known mechanical gap in `gsd-tools phase complete` (it updates the checkbox but not the table). The requirement checkbox at line 49 reads `[x]`. The stale table row is informational only and does not represent a real gap.

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `framework/tests/worker_boot.rs` | 104–107 | `#[cfg(feature = "redis-transport")] async fn transport_url_no_feature_warns()` returns `{}` — a stub body | Info | This is the feature-on branch of a cfg-pair; the feature-off branch (the real D-07 test) runs under default features. The stub exists only when the feature is enabled, where the D-07 warning path is genuinely inactive. Not a goal-blocking stub. |

No blockers. No placeholder comments (`TODO`, `FIXME`, `PLACEHOLDER`) in production code paths introduced by this phase. The `None`-broadcaster fallback (`register_offload_hooks()` branch) in `run_common_boot` is intentionally retained per RESEARCH critical constraint; Phase 249.1 removes it.

---

### Human Verification Required

#### 1. OFFLOAD-05 Multi-Process Runtime Behaviour

**Test:** Build `cargo build -p app`. Set `DATABASE_URL` and `QUEUE_CONNECTION=db`.
- Terminal 1: `./target/debug/app serve --no-worker` — confirm HTTP server starts.
- Terminal 2: `./target/debug/app worker --queue reports` — confirm worker starts.
- Terminal 3: `./target/debug/app worker --queue default` — confirm worker starts.
- Enqueue jobs on both queues. Confirm each processed exactly once; queue scoping respected.
- Kill terminal 2; confirm terminal 3 continues (fault-domain isolation).
- Run `./target/debug/app worker` (no flag); confirm it consumes all registered queues.

**Expected:** Each job processed exactly once across replicas. `reports` worker never claims `default` jobs and vice versa. A failed worker class does not stall an unrelated class. All-queues default functions correctly.

**Why human:** True cross-process at-least-once guarantee requires live DB accessible to multiple OS processes. The automated in-process proxy (SC#2 two-loop test) is sufficient evidence for the DB-level exclusive-claim mechanism, but the VALIDATION.md designates this multi-process scenario explicitly as Manual-Only for OFFLOAD-05.

#### 2. WR-01 Cross-Replica Broadcast over Redis (bonus UAT)

**Test:** With `BROADCAST_REDIS_URL` set and the `redis-transport` feature enabled, run two web replicas. Publish a delta on replica A; confirm it reaches a client subscribed on replica B.

**Expected:** Delta published on replica A reaches subscribers on replica B via the shared RedisTransport.

**Why human:** Requires a live `redis-server` and two separate processes. The automated `worker_boot.rs` WR-01 scenario skips when `REDIS_URL` is unset. This validates Phase 246.1 multi-replica UAT as well.

---

### Gaps Summary

No automated gaps found. All nine verifiable truths pass. The tenth truth (multi-process runtime behaviour) is designated human-UAT per the phase's own VALIDATION.md and per the ROADMAP.md note on OFFLOAD-05. The phase goal is substantively achieved across all automated dimensions — the human UAT is the final confirmation gate, not a remediation.

---

_Verified: 2026-08-14_
_Verifier: Claude (gsd-verifier)_
