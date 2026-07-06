---
phase: 232-single-source-write-surfaces-wire-the-derived-executor-retire-the-hand-written-writedispatcher
verified: 2026-06-16T04:50:17Z
status: passed
score: 4/4 success-criteria verified
overrides_applied: 0
---

# Phase 232: Single-Source Write Surfaces Verification Report

**Phase Goal:** The Phase-231 derived executor drives writes from a single `ServiceDef` across BOTH write surfaces — the MCP write dispatch AND the visual/form write path — so one declaration backs writes in every modality with no per-channel executor. The hand-written `WriteDispatcher` `match` that re-encoded transition facts is removed (the runtime envelope is relocated/shared, NOT deleted).
**Verified:** 2026-06-16T04:50:17Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (Roadmap Success Criteria)

| # | Truth (SC) | Status | Evidence |
|---|------------|--------|----------|
| 1 | MCP write path dispatches the transition through the relocated `framework::write` kernel; no hand-written `match` re-encodes the transition; MCP write tests pass (guard re-eval + idempotency + audit) | ✓ VERIFIED | `ferro-mcp-server/src/write_dispatch.rs:14` imports `dispatch_write` from `ferro_rs::write`; calls it at `:193`/`:505`/`:913` with literal channel `"mcp"`. In-crate kernel deleted (`grep 'pub async fn dispatch_write\|pub struct WriteDispatcher' write_dispatch.rs` → empty). `cargo test -p ferro-mcp-server --all-features` → 42 lib + integration suites green (guard-denied, idempotency, confirmation flows, tenant isolation). `cargo test -p app mcp_write_dispatch` → 4/4 incl. `submit_persists_derived_to_state`. |
| 2 | Visual/form path `POST /{service}/{action}` executes the same transition through the SAME derived executor; same `ServiceDef`, no second channel-specific executor | ✓ VERIFIED | `app/src/controllers/visual_action.rs:77` calls `ferro::write::dispatch_write(.., "web")`; `:74` reuses `crate::controllers::mcp::make_write_dispatcher()` (the SAME dispatcher MCP uses); `:60` derives `to_state` via `derive_transition_plan` (not the form); `:40` tenant from `current_tenant()`. Route registered tenant-scoped at `routes.rs:118-123`. `cargo test -p app visual_action` → 6/6 green. |
| 3 | One transition declared once exercised through BOTH surfaces in a test with identical semantics (same guard re-eval, same persisted derived `to_state`) | ✓ VERIFIED | `app/src/tests/single_source.rs` — `single_source_both_channels` drives `submit` via MCP (`handle_tools_call`→`dispatch_write(.., "mcp")`) and visual (`dispatch_write(.., "web")`), asserts `mcp_state == visual_state == "submitted"` (derived `Transition.to`) with the audit channel (`mcp.action.submit` vs `web.action.submit`) the only divergence. `single_source_guard_rejects_both` proves identical guard rejection on both. `cargo test -p app single_source` → 2/2 green. |
| 4 | No hand-authored `match` re-encoding `StateMachine` transitions on the write path; transition facts exist only in the `StateMachine` | ✓ VERIFIED | `grep -rn 'pub async fn dispatch_write' framework/src ferro-mcp-server/src app/src` → exactly 1 (`framework/src/write/mod.rs:313`). `grep -rn 'match action_name' app/src ferro-mcp-server/src` (code, excl. doc-comments) → empty. The only `match .*action` hits are `match action.execute()` (`todo.rs` — `Result` on unrelated CRUD) and `match find_action(...)` (registry `Option<(ServiceDef,ActionDef)>` resolution); neither maps an action name to a transition target. |

**Score:** 4/4 success criteria verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `framework/src/write/mod.rs` | Relocated channel-agnostic kernel + envelope | ✓ VERIFIED | `dispatch_write` (`:313`, channel param `:319`), `WriteDispatcher` (`:142`), `ExecutorFn` (`:79`), `GuardEvaluatorFn` (`:95`), `OverrideFn` (`:125`). Audit at `:414` uses `format!("{channel}.action.{}", ..)` — parameterized, no hardcoded `mcp.action` in body. 9/9 kernel tests pass. |
| `ferro-mcp-server/src/write_dispatch.rs` | MCP/JSON-RPC framing only, calling into the kernel | ✓ VERIFIED | In-crate kernel deleted; `use ferro_rs::write::{dispatch_write, WriteDispatcher, WriteError}` (`:14`). Guard-denied framing audit pins literal `mcp.action.{name}` (`:221`). |
| `app/src/controllers/visual_action.rs` | Visual handler calling shared kernel, channel `"web"` | ✓ VERIFIED | Full handler reads tenant from auth, derives to_state from plan, reuses single dispatcher, calls `dispatch_write(.., "web")`; error→4xx mapping (final `match` is on `WriteError`, not transitions). |
| `app/src/routes.rs` | Tenant-scoped `POST /{service}/{action}` route | ✓ VERIFIED | `:118-123` — `projection.visual.action` inside a `group!` with `SessionUserTenantResolver` + `TenantFailureMode::Forbidden` (unresolvable tenant denied, not allowed). |
| `app/src/tests/single_source.rs` | Both-channels single-source proof | ✓ VERIFIED | Contains `single_source_both_channels` + `single_source_guard_rejects_both`; both pass. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `ferro-mcp-server/src/write_dispatch.rs` | `framework::write::dispatch_write` | call in `handle_write_call`/confirm with `"mcp"` | ✓ WIRED | `:193`, `:505`, `:913` pass literal `"mcp"`. |
| `app/src/controllers/visual_action.rs` | `framework::write::dispatch_write` | shared kernel call with `"web"` | ✓ WIRED | `:77-88` passes literal `"web"`. |
| `app/src/controllers/visual_action.rs` | `ferro::derive_transition_plan` | to_state derivation | ✓ WIRED | `:60` derives plan; to_state never read from form. |
| `app/src/routes.rs` | `visual_action::handle` | `post!` registration | ✓ WIRED | `:119`. |
| `framework` ↛ `ferro-mcp-server` | (no cycle) | dependency direction | ✓ VERIFIED | `grep ferro-mcp-server framework/Cargo.toml` → empty. `ferro-mcp-server/Cargo.toml:17` depends on `ferro-rs` — acyclic. |

### Data-Flow Trace (Level 4)

Transition target (`to_state`) flows from a single source on both channels: `ferro::derive_transition_plan(svc, action).to_state` inside the reused `make_write_dispatcher()` executor. Verified the production executor (`mcp.rs:68-142`) and the test fixtures derive the status identically (`plan.to_state`); the form body is consumed only as opaque `inputs["id"]`. No hardcoded/static status, no form-supplied state. `visual_action_rejects_form_supplied_to_state` (green) confirms a bogus body `status`/`to_state` is ignored.

### Behavioral Spot-Checks (tests run, not trusted)

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Both-channels single-source | `cargo test -p app single_source` | 2/2 passed | ✓ PASS |
| Visual write surface (5 STRIDE + routing) | `cargo test -p app visual_action` | 6/6 passed | ✓ PASS |
| MCP write regression (app) | `cargo test -p app mcp_write_dispatch` | 4/4 passed | ✓ PASS |
| Relocated kernel (guard/dedup/override/idempotency/audit-channel) | `cargo test -p ferro-rs --features projections,confirmation --lib write` | 9/9 passed | ✓ PASS |
| MCP-server full suite | `cargo test -p ferro-mcp-server --all-features` | 42 lib + 5 + 4 + 9 integration, all green | ✓ PASS |
| Full workspace gate | `cargo test --all-features` | exit 0, zero failures, no ENOSPC | ✓ PASS |
| Format | `cargo fmt --all -- --check` | exit 0 | ✓ PASS |
| Clippy (workspace) | `cargo clippy --all --all-targets -- -D warnings` | exit 0 | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| EXEC-05 | 232-01/02/03 | Derived executor drives writes from single `ServiceDef` across MCP + visual/form surfaces; one declaration, every modality, no per-channel executor; retire hand-written `WriteDispatcher` match | ✓ SATISFIED | All 4 SCs verified; marked `[x]` / Complete in REQUIREMENTS.md:28,57. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | — | — | — | No blocking anti-patterns. The `match outcome` in `visual_action.rs:92` and `match find_action(...)` / `match action.execute()` are error/Option/Result matches, NOT transition re-encoding. SUMMARYs accurately disclosed these as benign. |

### Observations (non-blocking)

1. **Test drivers call the kernel directly, not the HTTP `visual_action::handle`.** Both `single_source.rs` and `visual_action.rs` exercise the shared `dispatch_write(.., "web")` kernel call through a fixture dispatcher that mirrors production `make_write_dispatcher()` — they do not invoke `handle(req: Request)` end-to-end through the HTTP/middleware stack. This is an explicit, documented decision (232-02/03 SUMMARYs: "rather than spinning the full HTTP+middleware stack"). The handler's own glue (param parse, `current_tenant()` resolution, 403/404/422 mapping, dispatcher reuse) is covered by `cargo build` (the `#[handler]` macro + type-check) and a matchit routing precedence test, but not by a runtime behavioral test. The EXEC-05 single-source claim — one kernel, identical transition semantics across channels — IS directly tested. I verified the test fixtures faithfully reproduce production executor/guard logic (`mcp.rs:68-142` vs `single_source.rs:155-231`). This does not weaken the SC verdicts; it is a coverage note: the visual handler's request-mapping layer has compile-time but not runtime-behavioral coverage.

### Human Verification Required

None. All four success criteria are programmatically verifiable and were confirmed by running the test suites (not by trusting the SUMMARYs). No visual/real-time/external-service behavior is in scope for this phase (it is a write-kernel/handler wiring phase with full test coverage of the kernel path).

### Gaps Summary

No gaps. The kernel was relocated to exactly one location (`framework::write`), the `WriteDispatcher`/`ExecutorFn`/`GuardEvaluatorFn`/`OverrideFn` envelope is preserved (relocated, not deleted), both write surfaces call the single shared kernel with distinct channel tags, the both-channels test proves identical persisted `to_state` + guard re-eval with the audit channel as the only divergence, exactly one `dispatch_write` definition exists with no transition-target `match` on the write path, and the full workspace gate (fmt + clippy `--all --all-targets -D warnings` + test `--all-features`) is green with no ENOSPC.

---

_Verified: 2026-06-16T04:50:17Z_
_Verifier: Claude (gsd-verifier)_
