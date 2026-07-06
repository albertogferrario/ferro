---
phase: 231-statemachine-derived-executor-derivation-guard-re-eval-override-hook-sync-by-construction
fixed_at: 2026-06-16T00:00:00Z
review_path: .planning/phases/231-statemachine-derived-executor-derivation-guard-re-eval-override-hook-sync-by-construction/231-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 231: Code Review Fix Report

**Fixed at:** 2026-06-16T00:00:00Z
**Source review:** .planning/phases/231-statemachine-derived-executor-derivation-guard-re-eval-override-hook-sync-by-construction/231-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 2 (0 critical, 2 warning; `fix_scope = critical_warning`)
- Fixed: 2
- Skipped: 0

Info findings IN-01..IN-04 were out of scope for this pass and left untouched.

## Fixed Issues

### WR-01: Override runs after base audit; a failing override leaves an audit entry that overstates what persisted

**Files modified:** `ferro-mcp-server/src/write_dispatch.rs`
**Commit:** 0c01a950
**Applied fix:** Reordered `dispatch_write` so the base persist's idempotency-key
store (step 5) and audit entry (step 6) are sealed BEFORE the post-persist
override hook (now step 7). There is no surrounding transaction, so the base
persist commits at step 4; sealing its idempotency key and audit first
guarantees a committed base transition is always recorded and never
re-executable, even when an app-specific override fails and `?`-returns.

Supporting changes in the same commit:
- Updated the `dispatch_write` pipeline-order doc comment to list step 7 and
  explain the WR-01 ordering rationale.
- Updated the `OverrideFn` doc comment with an "Ordering vs. audit/idempotency
  (WR-01)" section documenting that an override failure does NOT roll back the
  base persist but the base persist IS still audited and idempotency-keyed.
- Rewrote the misleading `override_error_surfaces` test comment (which claimed
  "the base write's audit already happened" under the old order) to match the
  corrected order.
- Extended `override_error_surfaces` to supply an `idempotency_key` and assert
  that BOTH the `audit_log` entry (`COUNT = 1` for `mcp.action.submit`) AND the
  stored idempotency result exist after the override errors — proving the base
  write is never left unaudited or re-executable.

**Verification note:** This is an ordering/durability-invariant change, not a
pure logic rewrite; the corrected order is covered by the extended
`override_error_surfaces` assertions (audit entry + idempotency key both present
on override failure), which pass under `--all-features`.

### WR-02: Confirmation handlers' guard pre-loops evaluated only `action.preconditions`, not the transition-guard union

**Files modified:** `ferro-mcp-server/src/write_dispatch.rs`
**Commit:** aff10007
**Applied fix:** Both confirmation pre-checks now evaluate the same
`merged_guards(preconditions, transition_guard)` union that `handle_write_call`
and `dispatch_write` use, deduped and order-preserving:

- `handle_request_confirm`: changed `_svc` to `svc`, derived the plan via
  `derive_transition_plan(svc, &action.name).ok()`, built the merged guard set,
  and iterated it in the token-issuance pre-loop (previously
  `action.preconditions` only).
- `handle_confirm`: reused the already-derived `transition_guard` to build the
  merged guard set and iterate it in the confirm-time pre-loop (previously
  `action.preconditions` only). The actual write was already fully guarded via
  `dispatch_write(..., transition_guard, true)`; this aligns the fail-fast
  pre-check with the enforced union so a transition-only guard denies at the
  pre-check point rather than wasting the round-trip.

## Validation Gate

Run after both fixes were in place (combined state):

| Gate | Command | Result |
|------|---------|--------|
| Tests | `cargo test -p ferro-mcp-server --all-features` | PASS — 50 unit + 5 + 9 (1 ignored: live_eval) + 5 + 4 integration; `override_error_surfaces` (new audit + idempotency assertions) PASS |
| Clippy | `cargo clippy -p ferro-mcp-server --all-targets --all-features -- -D warnings` | PASS — no warnings |
| Format | `cargo fmt --all -- --check` | PASS — exit 0 |

Each fix was also built and tested in isolation before commit:
- WR-02 applied to an otherwise-clean file: `cargo build -p ferro-mcp-server --all-features` PASS.
- WR-01 full state: full test suite + clippy + fmt PASS (above).

`Cargo.lock` showed pre-existing workspace version-bump churn (0.2.61 → 0.2.65)
unrelated to the fix; reverted via `git checkout -- Cargo.lock` before each
commit, not folded into the fix commits. The schema-export test did not dirty
`docs/protocol/schemas/*.json` on these runs. No ENOSPC encountered (7.6 GiB
free at start).

---

_Fixed: 2026-06-16T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
