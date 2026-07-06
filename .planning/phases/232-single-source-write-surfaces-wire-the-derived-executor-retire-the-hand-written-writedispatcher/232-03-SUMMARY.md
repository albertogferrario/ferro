---
phase: 232-single-source-write-surfaces-wire-the-derived-executor-retire-the-hand-written-writedispatcher
plan: 03
subsystem: single-source-write-verification
tags: [EXEC-05, SC3, SC4, single-source, both-channels, phase-acceptance]
requires:
  - "framework::write::dispatch_write — the single channel-agnostic kernel (Plan 01)"
  - "ferro-mcp-server::handle_tools_call → dispatch_write(.., \"mcp\") (Plan 01)"
  - "app::controllers::visual_action::handle → dispatch_write(.., \"web\") (Plan 02)"
  - "ferro::derive_transition_plan + order/approval ServiceDef anchor (Phase 231)"
provides:
  - "single_source_both_channels — the SC3 cross-modal coherence proof"
  - "single_source_guard_rejects_both — guard re-eval is the same kernel gate on both channels"
  - "SC4 structural assertion: exactly one dispatch_write, no transition-target match on the write path"
affects:
  - "EXEC-05 / Phase 232 acceptance (closes the v16.0 milestone write-surface requirement)"
tech-stack:
  added: []
  patterns:
    - "Both-channels test drives the SAME ServiceDef transition through handle_tools_call (mcp) and dispatch_write (web); asserts identical persisted to_state + identical guard outcome; audit channel the only divergence"
    - "SC4 enforced as a CI grep step (documented), not a runtime test — structural single-source claim"
key-files:
  created:
    - "app/src/tests/single_source.rs"
  modified:
    - "app/src/tests/mod.rs"
decisions:
  - "MCP channel driven through handle_tools_call (full framing) and visual channel through dispatch_write(.., \"web\") (the exact kernel call the handler makes, mirroring the Plan 02 fixture pattern) — both end in the one kernel, so the single-source claim is proven without spinning the full HTTP stack"
  - "Tests gated not(feature = \"confirmation\") to match the sibling MCP/visual fixtures: submit/approve are destructive, so feature-on requires the two-step confirm flow (covered by the ferro-mcp-server confirmation suite); the structural single-source claim is identical either way"
  - "SC4 left as a documented grep proof (not a source-reading #[test]) — the assertion is structural and belongs in CI/review, not in a shelling-out unit test"
metrics:
  duration: "~30m"
  tasks: 2
  files: 2
  completed: "2026-06-16"
---

# Phase 232 Plan 03: Single-Source Both-Channels Proof Summary

Proved the v16.0 cross-modal claim is TRUE: ONE `submit` transition, declared once on the
order/approval `ServiceDef`, is exercised through BOTH write surfaces — the MCP framing
(`handle_tools_call` → `dispatch_write(.., "mcp")`) and the visual handler
(`dispatch_write(.., "web")`) — and persists the IDENTICAL derived `to_state` with IDENTICAL
guard re-evaluation, the audit channel (`mcp.action.submit` vs `web.action.submit`) being the
ONLY divergence. The structural SC4 grep confirms exactly one `dispatch_write` kernel and no
hand-authored `match` re-encoding a StateMachine transition anywhere on the write path. The full
workspace gate (fmt + clippy `--all --all-targets -D warnings` + test `--all-features`) is green.
EXEC-05 closes; Phase 232 (and the v16.0 write-boundary milestone) is complete.

## What Shipped

- **`single_source_both_channels` (SC3).** Drives the same declared `submit` transition through
  both channels against two fresh records in identical `draft` state (same tenant, guard holds):
  - MCP via `handle_tools_call` (full framing, channel `"mcp"`);
  - visual via `dispatch_write(.., "web")` (the exact kernel call `visual_action::handle` makes).
  Asserts: (1) both persist the derived `Transition.to` (`"submitted"`) and `mcp_state == visual_state`;
  (2) the audit channel is the only divergence — `mcp.action.submit` vs `web.action.submit`, with both
  audit `after` payloads recording the transition. A second per-channel executor or a divergent
  transition target would fail this test.
- **`single_source_guard_rejects_both` (SC3 guard half).** A transition whose live guard does NOT
  hold (`approve` with `is_manager` false for a userless tenant) is rejected on BOTH channels
  (MCP returns `isError`, visual returns `WriteError::GuardFailed`), leaving state unchanged. Proves
  the guard re-evaluation is the SAME kernel gate regardless of caller — not a per-channel
  re-implementation that could diverge.
- **SC4 structural single-source assertion (grep proof, documented below).** Exactly one
  `dispatch_write` definition; no `match action_name` re-encoding a transition; the existing
  `match find_action(...)` / `match action.execute()` hits are action *resolution* / `Result`
  matching, not transition *re-encoding*.
- **WriteDispatcher envelope intact.** Confirmed relocated to `framework::write` (`pub struct
  WriteDispatcher` + `impl`), re-exported via both the `ferro::` facade and `ferro_mcp_server` — the
  "retire the hand-written WriteDispatcher" goal was the already-deleted `match`, not the runtime
  kernel, which is preserved.

## Verification

| Check | Result |
|-------|--------|
| `cargo test -p app single_source` | 2/2 passed (`single_source_both_channels`, `single_source_guard_rejects_both`) |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --all --all-targets -- -D warnings` | clean |
| `cargo test --all-features` | exit 0 — 129 `test result: ok` blocks, zero failures, no ENOSPC |
| Tree churn after full test | none (no schema-export / Cargo.lock dirtying this run) |

### SC4 structural grep proof

| Grep | Expected | Actual |
|------|----------|--------|
| `grep -rn 'pub async fn dispatch_write' framework/src ferro-mcp-server/src app/src` | exactly 1 | 1 — `framework/src/write/mod.rs:313` (single kernel, no per-channel executor) |
| `grep -rn 'match action_name' app/src ferro-mcp-server/src` | no code match | only doc-comment mentions (`mcp_write_dispatch.rs:292`, `visual_action.rs:20,58`); no code |
| `grep -rnE 'match .*action' app/src/controllers ferro-mcp-server/src framework/src/write` | no transition-target match | hits are `match action.execute()` (todo.rs — `Result`), `match find_action(...)` (write_dispatch.rs — `Option<(ServiceDef, ActionDef)>` resolution), and doc-comments; **none re-encode `"submit" => "submitted"`** |

`match find_action(...)` resolves the `(ServiceDef, ActionDef)` pair from the registry — it does NOT
map an action name to a transition target. `match action.execute()` in `todo.rs` matches a `Result`
on an unrelated CRUD handler outside the StateMachine write path. The single source of truth for the
transition target remains `derive_transition_plan(svc, action).to_state`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Lint] `assert_eq!(.., true)` rejected by clippy `bool_assert_comparison`**
- **Found during:** Task 2 (`cargo clippy --all --all-targets -D warnings`)
- **Issue:** The two audit-`after` presence assertions in `single_source_both_channels` used
  `assert_eq!(x.is_some(), true, ..)`, which clippy's `-D warnings` rejects.
- **Fix:** Replaced both with `assert!(x.is_some(), ..)`. No semantic change.
- **Files modified:** `app/src/tests/single_source.rs`
- **Commit:** 801d239a (amended into the Task 1 commit)

**2. [scope clarification] rustfmt reflowed the `dispatch_write(.., "web")` call to multi-line**
- The single-line kernel call exceeded the line budget; `cargo fmt --all` reflowed it. Folded into
  the Task 1 commit (amend). No behavior change.

### Acceptance-grep nuance (not a deviation)

The `match action_name` / `match .*action` greps return doc-comment mentions and benign
action-resolution / `Result` matches — the same nuance Plan 01 and Plan 02 summaries recorded. Each
hit was eyeballed (see SC4 table); none re-encode a StateMachine transition, so SC4 holds.

## Threat Model Outcomes

- **T-232-10 (hidden second executor, EoP):** SC4 grep asserts exactly one `dispatch_write`
  definition — a per-channel executor would have surfaced here. None exists.
- **T-232-11 (divergent transition semantics between channels, Tampering):** `single_source_both_channels`
  asserts identical persisted `to_state` across MCP and visual; `single_source_guard_rejects_both`
  asserts identical guard-rejection outcome. Divergence fails the tests; both pass.

## Known Stubs

None — verification-only plan; no production logic added.

## Threat Flags

None — this plan adds no runtime surface; it only exercises and asserts the Plan 01/02 surfaces.

## Self-Check: PASSED

- `app/src/tests/single_source.rs` — FOUND
- `app/src/tests/mod.rs` registers `pub mod single_source;` — FOUND
- `232-03-SUMMARY.md` — FOUND
- commit 801d239a — verified in git log
