---
phase: 193-ferro-stripe-refund-event-completeness
verified: 2026-06-09
status: passed
score: 5/5 code SCs (SC6/SC7 publish = deferred operator step)
overrides_applied: 0
---

# Phase 193: ferro-stripe Refund Event Completeness + 0.7.0 Release — Verification Report

**Phase Goal:** Expose `refund_id` on `StripeChargeRefunded` (parsed from `charge.refunds`) + bump ferro-stripe to 0.7.0 with CHANGELOG, so gestiscilo Phase 99 can round-trip operator-initiated refunds without bypassing ferro-stripe. **Code-only scope** (per operator decision 2026-06-09): publish is deferred.
**Verified:** 2026-06-09 (grep + parser-contract test + full gate)
**Requirements:** STRIPE-REFUND-01, STRIPE-REFUND-02

---

## Success Criteria

| SC | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| SC1 | `refund_id: Option<String>` between `payment_intent_id` and `amount_refunded_cents` | PASS | `events.rs` — `pub refund_id: Option<String>` present in `StripeChargeRefunded`. |
| SC2 | parser populates from `charge.refunds` (Charge object), NOT `EventObject::Refund` | PASS | `from_raw` `EventObject::Charge(charge)` arm: `charge.refunds.as_ref().and_then(\|l\| l.data.first()).map(\|r\| r.id.to_string())`; returns `None` on absent/empty list (no panic). grep confirms `charge.refunds` present and `EventObject::Refund` absent. *(ROADMAP SC2 was corrected to this during planning — the original `EventObject::Refund` wording did not match a `charge.refunded` event's object shape.)* |
| SC3 | fixture has `refunds.data[].id`; parser-contract test asserts `refund_id = Some("re_...")` | PASS | `charge_refunded.json` carries `re_test_refunded_001`; `parser_contract.rs` asserts it; `cargo test -p ferro-stripe --test parser_contract` green (19 tests). |
| SC4 | version 0.5.0 → 0.7.0; CHANGELOG `## [0.7.0]` with the three items | PASS | `ferro-stripe/Cargo.toml` `version = "0.7.0"`; new `CHANGELOG.md` `## [0.7.0]` documents `refund_id`, the Phase 189 manual-capture additions, and the no-0.6.x rationale. `framework/Cargo.toml` ferro-stripe pin updated `0.5` → `0.7` (workspace constraint fix). |
| SC5 | full gate green | PASS | `cargo fmt --all -- --check` + `cargo clippy --all --all-targets -- -D warnings` + `cargo test --all-features` all green. |

**Score:** 5/5 code SCs.

---

## Deferred — Operator-Owned (NOT done this session, by design)

| SC | Step | Status |
|----|------|--------|
| SC6 | Push master → GitHub Actions auto-publishes ferro-stripe 0.7.0 to crates.io | PENDING — operator action |
| SC7 | `cargo search ferro-stripe --limit 1` returns `0.7.0` after publish | PENDING — follows SC6 |

The 0.7.0 version bump is **committed**, which *arms* the publish for the next push. To complete the milestone and unblock gestiscilo Phase 99 Plan 03 (refund_id) and Plan 04 (0.7.0 publish), the operator runs:

```bash
git push        # triggers GH Actions auto-publish of ferro-stripe 0.7.0
cargo search ferro-stripe --limit 1   # confirm 0.7.0 once the action completes
```

This was an explicit user decision (build code, stop before publish).

---

_Verified: 2026-06-09 (code phase; publish deferred to operator)_
