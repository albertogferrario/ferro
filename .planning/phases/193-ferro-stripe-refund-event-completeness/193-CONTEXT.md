# Phase 193: ferro-stripe Refund Event Completeness + 0.7.0 Release - Context

**Gathered:** 2026-06-09
**Status:** Ready for planning
**Mode:** focused inline capture (small additive phase; targets pre-scouted)

<domain>
## Phase Boundary

Expose the refund identifier on the `StripeChargeRefunded` typed webhook event so
a consumer can look up its local refund row without bypassing ferro-stripe via
direct `stripe::` imports (the V-95-01 gate). Bump ferro-stripe `0.5.0 → 0.7.0`
with a CHANGELOG that bundles the already-built-but-unpublished Phase 189
manual-capture work into the same release label.

**Scope split (per user decision 2026-06-09):** this phase delivers the CODE only
— field + parser + fixtures + test + version bump + CHANGELOG, all gates green and
committed. It STOPS before `git push` / crates.io publish. SC6 (push → auto-publish)
and SC7 (`cargo search` returns 0.7.0) are a deferred manual step the operator
triggers by pushing master.

In scope: STRIPE-REFUND-01 (refund_id field + parser + test), STRIPE-REFUND-02
(0.7.0 version label + CHANGELOG). Out of scope: the push/publish itself; backporting
to v0.5.x.
</domain>

<decisions>
## Implementation Decisions

### refund_id field + parser
- **D-01:** Add `pub refund_id: Option<String>` to `StripeChargeRefunded`
  (`ferro-stripe/src/webhook/events.rs:296`), positioned BETWEEN `payment_intent_id`
  and `amount_refunded_cents` (ROADMAP SC1).
- **D-02 (ROADMAP SC2 CORRECTED — audit finding):** ROADMAP SC2 says parse from
  `stripe::EventObject::Refund(r) => Some(r.id...)`. That is **wrong** for a
  `charge.refunded` event: its `event.data.object` is a `Charge`
  (`EventObject::Charge` — confirmed at the existing parser, events.rs:310), NOT a
  top-level `Refund`. The correct source is the charge's refunds list —
  `charge.refunds.data[].id` — which ROADMAP **SC3 already confirms** ("fixtures
  include `refunds.data[].id`"). Implement:
  ```rust
  refund_id: charge.refunds.as_ref()
      .and_then(|list| list.data.first())
      .map(|r| r.id.to_string()),
  ```
  Returns `None` if no refund is present (defensive — a malformed `charge.refunded`
  with an empty refunds list). The exact accessor (`.first()` vs `.last()`, and
  whether `refunds` is `Option<List<Refund>>`) is verified against the async-stripe
  `Charge`/`Refund` types at implementation; `.first()` (most-recent-first) is the
  recommended default for the single-refund case this phase targets.
- **D-03:** Populate in the existing `EventObject::Charge(charge)` arm of
  `from_raw` (events.rs:310) — no new match arm, no new EventObject handling.

### fixtures + test
- **D-04:** Update `ferro-stripe/tests/fixtures/stripe_events/charge_refunded.json`:
  add a `refunds` object to the charge — `"refunds": { "object": "list",
  "data": [ { "id": "re_test_refunded_001", "object": "refund", ... } ],
  "has_more": false, "total_count": 1, "url": "..." }` — shaped as real Stripe
  webhooks ship it (minimal valid Refund fields the deserializer needs).
- **D-05:** In `ferro-stripe/tests/parser_contract.rs`, extend (or add) the
  `charge.refunded` parser-contract test to assert the parsed event carries
  `refund_id == Some("re_test_refunded_001".to_string())` matching the fixture.

### release label (code only)
- **D-06:** Bump `ferro-stripe/Cargo.toml` `version = "0.5.0"` → `"0.7.0"`
  (independent of the 0.2.48 workspace version — ferro-stripe is versioned on its
  own line). Skipping 0.6.x is intentional (combined breaking change w/ Phase 189).
- **D-07:** Create `ferro-stripe/CHANGELOG.md` (none exists) with a `## [0.7.0]`
  entry documenting: (a) new `StripeChargeRefunded::refund_id`, (b) the Phase 189
  manual-capture additions (`CheckoutBuilder::manual_capture`, `payment_intent`
  capability module, the two new typed events) that were ready but unpublished,
  (c) the version-skip rationale (no 0.6.x).
- **D-08 (publish deferred):** Do NOT `git push` or publish. After the phase
  commits, the operator pushes master to trigger the GitHub Actions auto-publish
  of ferro-stripe 0.7.0 (per `feedback_ferro_publish.md`). The version bump being
  committed means the publish is *armed* for the next push — make this explicit in
  193-VERIFICATION.md as the one remaining (operator-owned) step.

### Claude's Discretion
- Exact minimal Refund JSON shape in the fixture (only the fields async-stripe's
  `Refund` deserializer requires + `id`).
- `.first()` vs `.last()` on the refunds list (recommend `.first()`).
- Whether the CHANGELOG also notes earlier ferro-stripe history or starts at 0.7.0.
</decisions>

<canonical_refs>
## Canonical References

- `.planning/ROADMAP.md` § "v11.6.2 ... (Phase 193)" — goal, 7 success criteria (SC2 corrected per D-02), consumer pairing.
- `ferro-stripe/src/webhook/events.rs:292-320` — `StripeChargeRefunded` struct + `from_raw` parser (the edit site; `EventObject::Charge` arm).
- `ferro-stripe/tests/fixtures/stripe_events/charge_refunded.json` — the fixture to extend with `refunds`.
- `ferro-stripe/tests/parser_contract.rs` — the parser-contract test to extend.
- `ferro-stripe/Cargo.toml` — version line (0.5.0 → 0.7.0).
- async-stripe crate — `stripe::Charge::refunds` (`Option<List<Refund>>`), `stripe::Refund::id` (`RefundId`); verify exact types at build.
- `.planning/phases/189-ferro-stripe-manual-capture/189-*-SUMMARY.md` — the Phase 189 additions the 0.7.0 CHANGELOG documents.

No external specs beyond the ROADMAP and the async-stripe types.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- The existing `from_raw` already destructures `EventObject::Charge(charge)` and
  reads `charge.payment_intent`, `charge.amount_refunded`, `charge.metadata` — the
  new `refund_id` is one more field read off the same `charge`.
- `parser_contract.rs` already loads fixtures and asserts parsed fields — extend the
  existing pattern, no new harness.

### Established Patterns
- Typed events are plain structs + `impl StripeEvent { fn from_raw(&Event) -> Option<Self> }`; additive `Option<String>` field is non-breaking on the consumer surface.
- ferro-stripe carries its own semver line, published via GH Actions on push to master.

### Integration Points
- `events.rs` (struct + parser), the fixture JSON, `parser_contract.rs`, `Cargo.toml`, new `CHANGELOG.md`. All within the `ferro-stripe` crate.
</code_context>

<specifics>
## Specific Ideas
- A consumer (gestiscilo-it v6.3 Phase 99) is hard-blocked on both the field and the published 0.7.0 — this phase unblocks the field now; the publish is the operator's push.
</specifics>

<deferred>
## Deferred Ideas
- The push/publish (SC6/SC7) — operator-owned, this session stops before it (D-08).
- Backporting refund_id to v0.5.x — out of scope (0.7.0 is opt-in).

### Reviewed Todos (not folded)
None.
</deferred>

---

*Phase: 193-ferro-stripe-refund-event-completeness*
*Context gathered: 2026-06-09 (focused inline capture)*
