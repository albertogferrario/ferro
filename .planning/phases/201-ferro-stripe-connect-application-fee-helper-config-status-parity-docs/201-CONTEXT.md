# Phase 201: ferro-stripe Connect application-fee helper + config-status parity + docs - Context

**Gathered:** 2026-06-11
**Status:** Already implemented — see 201-VERIFICATION.md

<domain>
## Phase Boundary

A consumer holding a charge amount and a configured platform fee percent can
compute the application fee in one call (`StripeConfig::application_fee_for`),
introspect Connect-fee readiness via ferro-mcp `stripe_config_status`, and
follow a documented end-to-end Connect application-fee example. Additive on
ferro-stripe 0.8.0; publishes 0.9.0.
</domain>

<decisions>
## Implementation Decisions

This phase required no discussion. The deliverable was implemented directly on
master in commit `705bac6b` ("feat(stripe): add application_fee_for helper +
mcp parity + docs (0.9.0)"), outside the GSD discuss→plan→execute flow, before
the Phase 200 work landed. All design choices were fixed by the roadmap success
criteria and are realized as-built:

- **D-01:** `application_fee_for(amount_cents: i64) -> Option<i64>` returns
  `Some(round(amount_cents × percent / 100))` when `application_fee_percent` is
  set and strictly positive; `None` when unset or `≤ 0`. Result clamped to
  `[0, amount_cents.max(0)]` — never negative, never exceeds the charge.
- **D-02:** ferro-mcp `stripe_config_status` reports `connect_webhook_secret_present`
  (bool; the secret value is never returned) and `application_fee_percent`
  (parsed number, or null when unset).
- **D-03:** `docs/src/features/stripe.md` documents the full Connect
  destination-charge-with-platform-fee flow, cross-referenced to the
  manual-capture flow (Phase 189).
- **D-04:** ferro-stripe bumped `0.8.0 → 0.9.0`; CHANGELOG `[0.9.0]` records
  helper + mcp parity + docs as additive/non-breaking.

### Claude's Discretion
None — fully specified by roadmap success criteria.
</decisions>

<canonical_refs>
## Canonical References

### Roadmap / requirements
- `.planning/ROADMAP.md` §"v11.6.3 ferro-stripe Connect Application Fee Helper (Phase 201)" — phase goal + 6 success criteria

### Implementation (as-built)
- `ferro-stripe/src/config.rs:63` — `application_fee_for` + 8 unit tests
- `ferro-mcp/src/tools/stripe.rs:40` — `StripeConfigStatus` Connect fields + tests
- `ferro-mcp/src/service.rs:1676` — `stripe_config_status` tool wrapper
- `docs/src/features/stripe.md` §"Connect destination charges with a platform fee" (line 231)
- `ferro-stripe/CHANGELOG.md` §`[0.9.0]`

### Correspondence
- Phase 189 manual-capture flow (`docs/src/features/stripe.md` §Connect composition)
</canonical_refs>

<code_context>
## Existing Code Insights

The Connect destination-charge surface shipped complete in 0.8.0
(`account::*`, `CheckoutBuilder::destination`, `WebhookEvent.account`,
`StripeConnectAccountUpdated`, `verify_webhook`, `StripeConfig.{connect_webhook_secret,
application_fee_percent}`). This phase added only the missing fee-computation
primitive plus introspection parity and docs.
</code_context>

<specifics>
## Specific Ideas

Source: gestiscilo-it v7.1 photographer payment-gated-share field test (Marea
Studio). Consumer pairing: gestiscilo-it v6.10 Phase 204 consumes
`application_fee_for` via the published 0.9.0 bump.
</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.
</deferred>

---

*Phase: 201-ferro-stripe-connect-application-fee-helper-config-status-parity-docs*
*Context gathered: 2026-06-11*
