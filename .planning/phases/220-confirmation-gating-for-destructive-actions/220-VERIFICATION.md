---
phase: 220-confirmation-gating-for-destructive-actions
verified: 2026-06-14T00:00:00Z
status: passed
score: 5/5
overrides_applied: 0
---

# Phase 220: Confirmation Gating for Destructive Actions — Verification Report

**Phase Goal:** A destructive or irreversible action cannot execute in a single tool call; it requires an explicit confirmation token issued by the server and validated at dispatch time.
**Verified:** 2026-06-14
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Bare destructive call without token returns confirmation_required, not executed | VERIFIED | `write_dispatch.rs:319-322` — D-08 seam: `if action.transition_trigger.is_some() && !is_confirmed` returns `Err(ConfirmationRequired)`; `is_confirmed=false` on all bare `handle_write_call` paths (line 451). SC#1 test GREEN. |
| 2 | Two-step flow (request_confirm → confirm) executes exactly once; second confirm rejected | VERIFIED | `handle_request_confirm` (line 520) issues token; `handle_confirm` (line 627) calls `store.confirm()` (single-use consume); re-confirm returns `confirmation_expired`. SC#2 test GREEN (40/40 suite pass). |
| 3 | Expired token rejected, action not executed | VERIFIED | `handle_confirm:663` — `store.confirm()` returns `None` for expired → `confirmation_expired` error, no executor call. SC#3 test GREEN (uses real clock advance). |
| 4 | Token mismatch on action or record rejects, action not executed | VERIFIED | `handle_confirm:682-703` — three binding checks: `tenant_id`, `action_name`, `record_id` all verified. SC#4 action + record tests GREEN. Guard re-eval at confirm time also verified (`sc_guard_denied_at_confirm_time` GREEN). |
| 5 | Feature-off compiles, reads unaffected; ferro-ai absent without confirmation feature | VERIFIED | `cargo tree -p ferro-mcp-server --edges normal` shows no ferro-ai. With `--features confirmation` ferro-ai appears. 3 reqwest lines are pre-existing from ferro-mcp-oauth (not from ferro-ai). |

**Score:** 5/5 truths verified

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-ai/Cargo.toml` | `default=["llm"]`, reqwest optional, `confirmation=[]` | VERIFIED | Lines 50-62: `default=["llm"]`, reqwest/reqwest-eventsource/futures/async-stream/schemars all `optional=true`, `confirmation=[]` declared |
| `ferro-mcp-server/Cargo.toml` | `confirmation=["dep:ferro-ai","dep:rand"]`, `ferro-ai` with `default-features=false` | VERIFIED | Lines 20-33: ferro-ai `optional=true, default-features=false, features=["confirmation"]`; `confirmation=["dep:ferro-ai","dep:rand"]` |
| `ferro-mcp-server/src/error.rs` | `ConfirmationRequired(String)` variant, feature-gated | VERIFIED | Lines 39-41: `#[cfg(feature="confirmation")]` + `ConfirmationRequired(String)` |
| `ferro-mcp-server/src/write_dispatch.rs` | D-08 seam, token gen, handle_request_confirm, handle_confirm, binding, guard re-eval, redaction | VERIFIED | Full implementation present; all paths substantive (see key links) |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `handle_write_call` | `handle_request_confirm` | `tool_name.strip_prefix("request_confirm_")` | VERIFIED | Lines 382-397 |
| `handle_write_call` | `handle_confirm` | `tool_name.strip_prefix("confirm_")` | VERIFIED | Lines 398-412 |
| `dispatch_write` | D-08 seam | `transition_trigger.is_some() && !is_confirmed` | VERIFIED | Lines 319-322; `is_confirmed=false` passed at line 451 for bare calls |
| `handle_confirm` | `store.confirm()` (single-use) | `match store.confirm(&token)` | VERIFIED | Lines 663-677; `None` path = expired/used |
| `handle_confirm` | binding check | `binding["tenant_id"]`, `binding["action_name"]`, `call_record_id != stored_record_id` | VERIFIED | Lines 682-703; all three dimensions verified |
| `handle_confirm` | guard re-eval | `(dispatcher.guard_evaluator)(...)` at confirm time | VERIFIED | Lines 716-731 |
| `generate_confirmation_token` | CSPRNG / BASE62 / cfm_ prefix | `rand::thread_rng()` + BASE62 charset | VERIFIED | Lines 227-237; server-generated, never agent-supplied |
| Error strings | redaction (WR-01/02/03) | Fixed strings, no `{e}` interpolation | VERIFIED | Line 609: `"failed to store confirmation token"`; line 674: `"confirmation store error"`; line 723: `"precondition not met at confirm time"` |

---

## Data-Flow Trace (Level 4)

Not applicable — this phase delivers a security control (gating logic), not a data-rendering component.

---

## Behavioral Spot-Checks

| Behavior | Result | Status |
|----------|--------|--------|
| All 6 SC tests (sc1–sc4 + sc_guard_denied) pass under `--features confirmation` | 40/40 passed; 0 failed | PASS |
| ferro-ai absent from feature-off dep graph | `cargo tree -p ferro-mcp-server` shows no ferro-ai | PASS |
| ferro-ai present (confirmation module only) with `--features confirmation` | 1 ferro-ai entry, no reqwest from ferro-ai | PASS |
| reqwest in confirmation graph comes from ferro-mcp-oauth only (pre-existing) | 3 reqwest lines, all via ferro-mcp-oauth subtree | PASS |

---

## Requirements Coverage

| Requirement | Description | Status | Evidence |
|-------------|-------------|--------|----------|
| AMCP-05 | Destructive action requires explicit confirmation step — two-tool confirm flow, TTL, unconfirmed/mismatched/expired does not mutate | SATISFIED | D-08 seam (line 319-322), handle_request_confirm + handle_confirm fully implemented, 6 SC tests GREEN, binding on (tenant_id, action_name, record_id) |

---

## Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| None | — | — | — |

The WR-04 TOCTOU window (retried request_confirm mints a new token) is documented inline at line 590-598 and accepted: each token is single-use, TTL-bounded, and tuple-bound; the hardening path (re-key on (tenant, action, record)) is deferred to the DB-backed store per REQUIREMENTS.md future items.

---

## Human Verification Required

None. All success criteria are verifiable from the codebase and test suite results.

---

## Gaps Summary

No gaps. All 5 success criteria verified against actual source code and live test run (40/40 passed).

---

_Verified: 2026-06-14_
_Verifier: Claude (gsd-verifier)_
