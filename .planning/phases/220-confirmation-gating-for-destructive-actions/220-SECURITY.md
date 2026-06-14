---
status: secured
phase: 220-confirmation-gating-for-destructive-actions
asvs_level: 1
threats_total: 9
threats_closed: 9
threats_open: 0
residual_risks:
  - id: WR-04-TOCTOU
    disposition: accept
    description: >
      Retried request_confirm calls mint a new token each time because the store
      is keyed on the token string, not on the (tenant, action, record) tuple.
      A prior token remains live until its TTL expires. Non-exploitable in
      isolation: each token is single-use (DashMap::remove), TTL-bounded (<=600 s),
      and bound to (tenant_id, action_name, record_id); dispatch_write idempotency
      prevents double-execution even on a race. Hardening path: re-key store on
      (tenant, action, record) when a persistent/DB-backed store replaces
      InMemoryConfirmationStore.
    documented_at: ferro-mcp-server/src/write_dispatch.rs:590-598
---

# Phase 220 Security Audit

**Audited:** 2026-06-14
**ASVS Level:** 1
**Auditor:** gsd-security-auditor

## Threat Verification

| Threat ID | Category | Disposition | Status | Evidence |
|-----------|----------|-------------|--------|----------|
| T-220-01 | Tampering (D-08 seam) | mitigate | CLOSED | `write_dispatch.rs:319-322` — `if action.transition_trigger.is_some() && !is_confirmed { return Err(ConfirmationRequired) }` under `#[cfg(feature = "confirmation")]`. `is_confirmed` is hardcoded `false` at the `handle_write_call` call site (`write_dispatch.rs:451-453`) and only set `true` by `handle_confirm` after token validation (`write_dispatch.rs:735`). No agent-controlled path can pass `true`. |
| T-220-02a (token forgery) | Tampering | mitigate | CLOSED | `write_dispatch.rs:227-238` — `generate_confirmation_token()` uses `rand::thread_rng()` (CSPRNG), `cfm_` prefix + 43 BASE62 chars (~256-bit entropy). Token is server-generated inside `handle_request_confirm`, never read from agent input. `store.confirm()` validates store membership before any binding check (`write_dispatch.rs:663`). |
| T-220-02b (expiry) | Tampering | mitigate | CLOSED | `store.rs:64-84` — TTL expiry task spawned on `request_confirmation`; `store.confirm()` (`store.rs:87-95`) uses `DashMap::remove` which returns `None` after TTL → `confirmation_expired` error at `write_dispatch.rs:665-669`. Executor not reached. |
| T-220-02c (action/record mismatch) | Tampering | mitigate | CLOSED | `write_dispatch.rs:679-703` — binding verified in order: `tenant_id` (line 682), `action_name` (line 689), `record_id` (line 696-703). All three fields set at request time from server-authoritative sources. Mismatch on any returns `confirmation_mismatch` before `dispatch_write` is called. |
| T-220-02d (single-use / reuse) | Tampering | mitigate | CLOSED | `store.rs:87-95` — `confirm()` uses `self.inner.remove(key)` (DashMap atomic remove); second call with same key returns `None` → `confirmation_expired` at `write_dispatch.rs:665`. Test `sc2_two_step_flow_executes_once` asserts second confirm is rejected and executor count stays at 1. |
| T-220-02e (cross-tenant) | Tampering | mitigate | CLOSED | `write_dispatch.rs:682-686` — `handle_confirm` compares `binding["tenant_id"]` (server-set at request time) against authenticated `tid` (from `tenant_id: Option<i64>` arg, never from agent payload). Process-shared `OnceLock<InMemoryConfirmationStore>` is safe because tenant binding is enforced before execution. `app/src/controllers/mcp.rs:306` — `tenant_id` sourced from `ferro::current_tenant()`, not from agent payload. |
| T-220-03 (guard staleness) | Elevation of Privilege | mitigate | CLOSED | `write_dispatch.rs:715-732` — guards re-evaluated at confirm time against live DB via `dispatcher.guard_evaluator` (same `GuardEvaluatorFn` used at request time). Phase 219 fail-closed preserved: `Err(_)` from evaluator returns `guard_denied`, executor not reached. Test `sc_guard_denied_at_confirm_time` asserts guard-deny-at-confirm blocks execution. |
| T-220-CR01 (error leak — WR-01/02/03 fix) | Information Disclosure | mitigate | CLOSED | `write_dispatch.rs:607-610` — WR-01 fix: `"failed to store confirmation token"` (no `{e}`). `write_dispatch.rs:671-676` — WR-02 fix: `"confirmation store error"` (no `{e}`, `Err(_)` binding). `write_dispatch.rs:719-724` — WR-03 fix: `"precondition not met at confirm time"` (no guard name, no `{e}`). Commit afd646d1 confirmed in 220-REVIEW-FIX.md. |
| T-220-D06 (supply chain) | Supply Chain | mitigate | CLOSED | `ferro-mcp-server/Cargo.toml:20` — `ferro-ai = { ..., optional = true, default-features = false, features = ["confirmation"] }`. `ferro-mcp-server/Cargo.toml:33` — `confirmation = ["dep:ferro-ai", "dep:rand"]`. `ferro-ai/Cargo.toml:62` — `confirmation = []` (no HTTP deps). Live `cargo tree -p ferro-mcp-server --edges normal | grep -c ferro-ai` = 0 (feature-off build has zero ferro-ai). |

## Residual Risks

### WR-04: TOCTOU window between token storage and response delivery (accepted)

Retried `request_confirm` calls mint a new independent token each time (store keyed on token string, not on `(tenant, action, record)` tuple). The prior token remains live until TTL expiry. Accepted because:

1. Each token is single-use (`DashMap::remove` in `confirm()`).
2. Every token is bound to `(tenant_id, action_name, record_id)` — no cross-action or cross-record authorization from a stale token.
3. TTL is capped at 600 seconds (`ferro-mcp-server/src/config.rs`).
4. `dispatch_write` idempotency prevents double-execution even if two tokens for the same action somehow both get confirmed concurrently.

Hardening path documented at `write_dispatch.rs:590-598`: re-key store on `(tenant, action, record)` when a persistent/DB-backed store replaces `InMemoryConfirmationStore`.

## Unregistered Flags

None. All threat flags from the REVIEW.md map to registered threat IDs (WR-01/02/03 → T-220-CR01; WR-04 → accepted residual risk above).
