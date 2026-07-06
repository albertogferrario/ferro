---
phase: 232-single-source-write-surfaces-wire-the-derived-executor-retire-the-hand-written-writedispatcher
plan: 01
subsystem: write-kernel
tags: [framework::write, ferro-mcp-server, EXEC-05, relocation, audit-channel]
requires:
  - "ferro-projections derive_transition_plan + ActionDef/ServiceDef (v16.0 Phase 231)"
  - "ferro-audit AuditEntry (existing)"
provides:
  - "framework::write — the single channel-agnostic transition-execution kernel"
  - "ferro::write::WriteError / WriteResult (facade-re-exported kernel error)"
  - "channel-parameterized audit prefix format!(\"{channel}.action.{name}\")"
affects:
  - "ferro-mcp-server (now framing that calls into framework::write)"
  - "app/src/controllers/mcp.rs make_write_dispatcher() closure"
tech-stack:
  added:
    - "framework optional dep ferro-audit (under the projections feature)"
    - "framework `confirmation` feature flag (pure flag, no ferro-ai)"
    - "ferro-mcp-server → ferro-rs (framework) dependency edge (acyclic)"
  patterns:
    - "Self-contained kernel error mapped to each channel via From at the framing boundary"
    - "Channel literal passed by the framing, not computed by the kernel"
key-files:
  created:
    - "framework/src/write/mod.rs"
  modified:
    - "framework/Cargo.toml"
    - "framework/src/lib.rs"
    - "app/src/controllers/mcp.rs"
    - "app/src/tests/mcp_write_dispatch.rs"
    - "ferro-mcp-server/Cargo.toml"
    - "ferro-mcp-server/src/write_dispatch.rs"
    - "ferro-mcp-server/src/error.rs"
    - "ferro-mcp-server/src/lib.rs"
    - "ferro-mcp-server/src/jsonrpc.rs"
    - "ferro-mcp-server/src/intent.rs"
    - "ferro-mcp-server/tests/jsonrpc_integration.rs"
    - "ferro-mcp-server/tests/mcp_tenant_isolation.rs"
decisions:
  - "framework `confirmation` is a pure feature flag (no ferro-ai dep): the kernel's D-08 seam uses only transition_trigger.is_some() + is_confirmed + a self-contained WriteError::ConfirmationRequired"
  - "merged_guards exported pub from the kernel so the MCP confirm pre-check evaluates the identical guard union as dispatch_write"
  - "WriteError → crate::Error mapped via From at the ferro-mcp-server boundary; framing matches WriteError directly on the dispatch result"
metrics:
  duration: "~16m"
  tasks: 3
  files: 12
  completed: "2026-06-16"
---

# Phase 232 Plan 01: Single-Source Write Kernel Relocation Summary

Relocated the channel-agnostic transition-execution kernel from `ferro-mcp-server` into a new `framework::write` module with a self-contained `WriteError`, a `channel`-parameterized audit prefix, and the security envelope (`WriteDispatcher`/`ExecutorFn`/`GuardEvaluatorFn`/`OverrideFn`) preserved; MCP framing now calls into it passing the literal `"mcp"`, and the app executor closure was migrated to the kernel error — EXEC-05 coherence: one execution kernel, one location, callable by every write channel.

## What Shipped

- **`framework::write` kernel (new).** `dispatch_write` (guard re-eval → idempotency → confirm seam → persist → audit → override), `WriteDispatcher` + `new`/`with_override`, `ExecutorFn`/`GuardEvaluatorFn`/`OverrideFn`, `merged_guards`, and the idempotency lookup/store helpers, relocated verbatim from `ferro-mcp-server`. Behavior identical.
- **Self-contained `WriteError`/`WriteResult`.** The kernel owns its error (no dependency back on any channel). Re-exported via the `ferro::` facade so the app and the MCP crate name `ferro::write::WriteError` / `ferro_mcp_server::WriteError`.
- **Audit channel parameterized.** Success-path audit is `format!("{channel}.action.{name}")`. MCP passes the literal `"mcp"` at every call site → `mcp.action.{name}` stays regression-stable. A new kernel test (`audit_channel_is_parameterized`) proves a `"web"` channel writes `web.action.submit` and no `mcp.action.*`.
- **`ferro-mcp-server` reduced to framing.** In-crate kernel deleted; imports from `ferro_rs::write`. `find_action`, `handle_write_call`, `handle_request_confirm`, `handle_confirm`, JSON-RPC envelopes, and the guard-denied framing audit (literal `mcp.action.{name}`, pinned) stay. `From<WriteError> for crate::Error` added at the boundary (variant-for-variant, T-232-03 mitigated). New `ferro-rs` dependency edge — acyclic.
- **App executor closure migrated.** All 7 error-construction sites in `make_write_dispatcher()` now build `ferro::write::WriteError`; the fail-closed `_ => Err(GuardFailed(...))` unknown-guard arm preserved (T-232-05).

## Verification

| Check | Result |
|-------|--------|
| `cargo build -p ferro-rs --features projections` | exit 0 |
| `cargo build -p ferro-rs --features projections,confirmation` | exit 0 |
| `cargo build -p app --all-features` | exit 0 |
| `cargo build -p ferro-mcp-server --all-features --all-targets` | exit 0 |
| `cargo test -p ferro-rs --features projections,confirmation --lib write` | 9/9 passed (guard re-eval, dedup, override, idempotency, audit-channel) |
| `cargo test -p ferro-mcp-server --all-features` | 42 lib + integration suites, all green (confirmation flows, framing parse, tenant isolation, intent loop) |
| `cargo test -p app mcp_write_dispatch` | 4/4 passed incl. `submit_persists_derived_to_state` |
| clippy (ferro-rs / ferro-mcp-server ±confirmation / app) `-D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| SC1: `grep -rn 'match action_name' app/src` | only a doc-comment mention; no code match |
| No dependency cycle: ferro-mcp-server ∈ framework deps | none |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] App write-dispatch test fixture closures used the old error type**
- **Found during:** Task 3 (`cargo test -p app mcp_write_dispatch` failed to compile)
- **Issue:** `app/src/tests/mcp_write_dispatch.rs` builds its own `WriteDispatcher` closures (two copies) constructing `ferro_mcp_server::Error::{...}`. After the kernel relocation the `WriteResult`-typed `ExecutorFn`/`GuardEvaluatorFn` require `WriteError`, so the test file no longer typechecked (the plan's interfaces section only listed the production closure, not the test fixture).
- **Fix:** Migrated all `ferro_mcp_server::Error::` → `ferro::write::WriteError::` in the test file. Logic untouched.
- **Files modified:** `app/src/tests/mcp_write_dispatch.rs`
- **Commit:** 247ff8e7

**2. [Rule 1 - Bug] Unused `merged_guards` import without the confirmation feature**
- **Found during:** Task 3 (warning surfaced building `app` against ferro-mcp-server without `confirmation`)
- **Issue:** `merged_guards`'s only consumers in the framing are the two confirm-handler pre-check loops, both `#[cfg(feature = "confirmation")]`. The unconditional import was dead without the feature; CI's `-D warnings` would fail.
- **Fix:** Gated the `merged_guards` import to `#[cfg(feature = "confirmation")]`.
- **Files modified:** `ferro-mcp-server/src/write_dispatch.rs`
- **Commit:** 247ff8e7

**3. [scope clarification] Internal-import repointing in `ferro-mcp-server`**
- `jsonrpc.rs` and `intent.rs` imported `WriteDispatcher` from `crate::write_dispatch` (now a private re-import). Repointed both to the crate-root re-export `crate::WriteDispatcher`. Two integration tests (`jsonrpc_integration.rs`, `mcp_tenant_isolation.rs`) imported `ferro_mcp_server::write_dispatch::WriteDispatcher` → repointed to `ferro_mcp_server::WriteDispatcher`. Mechanical consequence of the move; no behavior change.
- **Commit:** 87ea6007

### Acceptance-grep nuance (not a deviation)

The Task 1 acceptance line `grep -rn 'mcp.action' framework/src/write/mod.rs` expected EMPTY. The kernel **production body** has no `mcp.action` (it uses `{channel}.action`); the only matches are inside the relocated **test assertions** (`override_error_surfaces` and `audit_channel_is_parameterized`), which assert that `channel="mcp"` produces `mcp.action.submit` and `channel="web"` does not. This is the stronger outcome — it proves the parameterization rather than just the absence of the literal — so the matches were kept intentionally.

## Threat Model Outcomes

- **T-232-01 (guard re-eval EoP):** `merged_guards` re-eval loop moved verbatim; `guard_rejects_illegal_transition` / `guard_denied_at_call_time` green in framework.
- **T-232-02 (audit channel repudiation):** success audit is `{channel}.action.{name}`; MCP literal `"mcp"` pinned by `override_error_surfaces` (asserts `mcp.action.submit`) + the framing `:1308`-equivalent path; `audit_channel_is_parameterized` proves the prefix tracks the arg.
- **T-232-03 (error-mapping tampering):** `From<WriteError> for crate::Error` is variant-for-variant; no downgrade to success.
- **T-232-05 (unknown-guard EoP):** app closure fail-closed `_ => Err(GuardFailed)` arm preserved; `cargo build -p app` + app tests green.

## Known Stubs

None.

## Threat Flags

None — pure relocation introduced no new network endpoint, auth path, or schema surface.

## Self-Check: PASSED

- `framework/src/write/mod.rs` — FOUND
- `232-01-SUMMARY.md` — FOUND
- commits 7e980a01, 87ea6007, 247ff8e7 — all FOUND
