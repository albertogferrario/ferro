---
phase: 232
slug: single-source-write-surfaces-wire-the-derived-executor-retire-the-hand-written-writedispatcher
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-16
---

# Phase 232 — Validation Strategy

> Per-phase validation contract. Detailed validation architecture in `232-RESEARCH.md`.
> Scope (Option A, locked): relocate the write kernel to `framework::write` (channel-agnostic),
> keep MCP framing calling into it, BUILD the visual `POST /{service}/{action}` handler calling
> the same kernel, prove single-source with a both-channels test. Do NOT delete the
> `WriteDispatcher`/`ExecutorFn` security envelope.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `#[test]` / `#[tokio::test]` |
| **Config file** | workspace `Cargo.toml` |
| **Quick run command** | `cargo test -p ferro-mcp-server --all-features` (kernel + MCP framing) |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | quick minutes; full gate longer (watch disk — `project_ferro_disk_full_test_gate`) |

---

## Sampling Rate

- **After every task commit:** `cargo test -p <touched crate>` (`framework` for the relocated kernel, `ferro-mcp-server` for framing, `app` for the visual handler).
- **After every plan wave:** `cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`.
- **Before `/gsd-verify-work`:** full gate green.

---

## Per-Criterion Verification Map

| Criterion (EXEC-05) | Secure Behavior | Test Type | Automated Command | Status |
|---------------------|-----------------|-----------|-------------------|--------|
| SC1 — MCP path uses derived executor; `match` deleted | MCP write still re-evals guard server-side, idempotent, audited — now via the relocated shared kernel | verify + regression | `grep -rn 'match action_name' app/src` empty; `cargo test -p ferro-mcp-server --all-features` | ⬜ pending |
| SC2 — visual path executes same transition via same derived executor | `POST /{service}/{action}` drives the transition through the SAME `framework::write` kernel (guard re-eval + persist + audit), persisting derived `to_state` | integration (new) | `cargo test -p app visual_action_drives_derived_transition` | ⬜ pending |
| SC3 — one transition exercised through BOTH surfaces, identical semantics | Same `ServiceDef` transition driven via MCP and via the visual handler yields identical guard re-eval + persisted state; no second executor | integration (new) | `cargo test -p app single_source_both_channels` | ⬜ pending |
| SC4 — no hand-authored `match` re-encoding transitions in the write path | Transition facts exist only in the `StateMachine`; no `match` mapping action→new-status anywhere on the write path | grep | `grep -rnE 'match .*action' app/src ferro-mcp-server/src framework/src` shows no transition-target match | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Both-channels integration fixture: a `ServiceDef` with one guarded `StateMachine` transition, exercised through (a) the MCP `dispatch_write` framing and (b) the new visual `POST /{service}/{action}` handler, asserting identical guard re-eval + persisted `to_state`. Reuse the existing synthetic order/approval anchor.

*Reuse existing anchors; do not author new app models.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| "No per-channel executor" is partly a structural claim | EXEC-05 | Automatable via grep, but the architectural single-source property is best eyeballed once | Confirm the kernel (`derive plan → guard re-eval → persist → audit → idempotency → override`) exists in exactly ONE location (`framework::write`) and both MCP + visual call it; no duplicated execution path |
