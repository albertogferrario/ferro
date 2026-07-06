---
phase: 151-ferro-wallet-crate
plan: 151-08
subsystem: testing
tags: [ferro-wallet, google-wallet, jwt, rs256, jsonwebtoken, openssl, integration-test]

# Dependency graph
requires:
  - phase: 151-07
    provides: GoogleWalletBuilder + save_jwt + save_url + event_ticket_object construction
provides:
  - End-to-end RS256 JWT roundtrip integration test (ACC-1k) — mints RSA keypair at runtime, signs save JWT, decodes with the public key, asserts claim shape per D-08
  - Save-URL prefix assertion against the production `https://pay.google.com/gp/v/save/` endpoint
  - Pitfall-3 mitigation pattern (validate_exp=false + cleared required_spec_claims) reusable by future jsonwebtoken-decode tests in this workspace
affects: [151-09 (release readiness — phase ready to ship), gestiscilo-it wallet-passes integration]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Runtime-minted RSA-2048 keypair via openssl::rsa::Rsa::generate (D-09) — public PEM via SubjectPublicKeyInfo, private PEM via PKCS#8"
    - "jsonwebtoken Validation construction for exp-less tokens: validate_exp=false + required_spec_claims=HashSet::new() + set_audience(&[expected_aud])"
    - "Integration-test stub WalletSubject mirroring the apple_integration.rs convention — known-constant field values asserted post-decode"

key-files:
  created:
    - "ferro-wallet/tests/google_jwt.rs"
  modified: []

key-decisions:
  - "Tests live in tests/google_jwt.rs (integration crate) not src/google/jwt.rs (#[cfg(test)] module) — keeps the openssl + jsonwebtoken roundtrip out of the lib build path and isolates per-test RSA generation cost from the 38 lib-test fast path."
  - "RSA-2048 (not 4096) for the runtime mint — fastest secure size; 2 generations per test invocation total (~80ms on Apple Silicon), no measurable contribution to ferro-wallet's 0.10s integration test wallclock."
  - "set_audience(&[\"google\"]) on Validation despite required_spec_claims being cleared — locks aud-check ON so a future regression that mis-spells the aud claim is caught even though aud is no longer 'required to be present'."

patterns-established:
  - "Pitfall-3 mitigation: when decoding tokens that lack `exp` (e.g. Google save JWTs, OIDC id_tokens used as bearer assertions), construct Validation explicitly: `Validation::new(alg)` → `validate_exp = false` → `required_spec_claims = HashSet::new()` → `set_audience(...)` to keep aud-check active."
  - "Runtime keypair mint as a credential-free CI pattern: pair the test's mint helper with the production builder's PEM-loading path so the test exercises the same parse-and-sign code production would hit with real credentials."

requirements-completed: [ACC-1k]

# Metrics
duration: 94s
completed: 2026-05-11
---

# Phase 151 Plan 151-08: Google JWT Test Summary

**End-to-end RS256 save-JWT roundtrip integration test — runtime-minted RSA keypair signs a save JWT for a stub event-ticket subject, public-key decode asserts all 10 documented claim fields plus the `pay.google.com/gp/v/save/` URL prefix.**

## Performance

- **Duration:** 94s
- **Started:** 2026-05-11T04:19:16Z
- **Completed:** 2026-05-11T04:20:50Z
- **Tasks:** 1
- **Files modified:** 1 (created)

## Accomplishments

- Closed acceptance criterion ACC-1k — the Google Wallet builder is now end-to-end validated (subject → eventTicketObject JSON → RS256 JWT → decode-with-public-key roundtrip → claim shape assertions).
- Documented and exercised the Pitfall-3 mitigation pattern for decoding `exp`-less tokens via jsonwebtoken (load-bearing — default `Validation::new(RS256)` rejects the save JWT with `MissingRequiredClaim("exp")`).
- ferro-wallet test surface now stands at 38 lib + 1 apple integration + 2 google_jwt = **41 green tests**; the phase is ready for the release plan (151-09).

## Task Commits

1. **Task 1: Author tests/google_jwt.rs end-to-end (ACC-1k)** — `67e3ac8e` (test)

_Single-task plan; no separate RED/GREEN/REFACTOR commits — the test was added with the GoogleWalletBuilder already landed in plan 07, so the gate was "test passes against existing implementation" rather than red-then-green._

## Files Created/Modified

- `ferro-wallet/tests/google_jwt.rs` (created, 185 lines) — Two `#[test]` functions:
  - `save_jwt_roundtrips_with_runtime_minted_rsa_keypair` (ACC-1k): mints a fresh RSA-2048 keypair, builds GoogleWalletBuilder, calls `save_jwt(&StubBooking)`, decodes with the matching public PEM, asserts `iss/aud/typ/origins[0]` (4 claim fields) and `eventTicketObjects[0].{id,classId,state,barcode.type,barcode.value}` (5 nested fields plus the array-length-1 invariant) — 10 assertions total.
  - `save_url_returns_pay_google_com_prefix`: builds a separate builder via `build_test_builder()` and asserts the URL starts with `https://pay.google.com/gp/v/save/`.

## Decisions Made

See `key-decisions` in frontmatter — three decisions made during execution:

1. Integration test crate (`tests/`) instead of `src/google/jwt.rs#[cfg(test)]` module — isolates openssl key generation cost from the 38-test lib fast path.
2. RSA-2048 keypair size — minimum secure size, ~40ms per generation, no measurable wallclock impact.
3. `set_audience(&["google"])` retained despite `required_spec_claims` being cleared — keeps aud-check active so a future regression mis-spelling the `aud` claim still fails.

## Deviations from Plan

None — plan executed exactly as written. The test body matches the plan's reference body in `<action>` (151-08-PLAN.md lines 119–242); after rustfmt reformatted the `build_test_builder` body to the multi-line form rustfmt prefers, no semantic changes were made.

## Issues Encountered

- Initial `cargo fmt -p ferro-wallet -- --check` reported a violation on `build_test_builder` and the in-place builder construction in the main test (rustfmt prefers `GoogleWalletBuilder::new(\n    cfg,\n    ...\n)\n.expect(...)` over the dotted-call form). Resolved by running `cargo fmt -p ferro-wallet`, which rewrote two short bodies; no logic changes. Final `cargo fmt --all -- --check` exited clean.

## Threat Surface Scan

No new threat surface introduced — this is test-only code, RSA material is generated at runtime and discarded, no committed credentials. The plan's threat register (T-151-Google-JWT, T-151-DEFAULT-CRED) is fully exercised:
- T-151-Google-JWT: the signature side is locked to RS256 via `Algorithm::RS256` on both ends; an attacker swapping algorithms would fail signature verification because the decode key is a public RSA PEM.
- T-151-DEFAULT-CRED: no PEM bytes are committed; `Rsa::generate(2048)` runs per test invocation.

## TDD Gate Compliance

Plan has `type: tdd` semantics at the task level (`<task type="auto" tdd="true">`), but the implementation under test (GoogleWalletBuilder) was landed in plan 151-07. The test was written, ran, and passed against the existing implementation immediately — no separate RED commit is meaningful here (an artificially failing test would not exercise the real production path). The single `test(151-08): ...` commit captures the green gate.

## User Setup Required

None — no external service configuration required. Tests run hermetically in CI; no `APPLE_WALLET_*` / `GOOGLE_WALLET_*` env vars touched.

## Self-Check: PASSED

- File `ferro-wallet/tests/google_jwt.rs` exists.
- Commit `67e3ac8e` present on master (`git log --oneline | head -1`).
- `cargo test -p ferro-wallet --test google_jwt` exits 0 (2/2 passing).
- `cargo test -p ferro-wallet` exits 0 (38 lib + 1 apple + 2 google_jwt = 41 green).
- `cargo build --workspace` exits 0.
- `cargo clippy -p ferro-wallet --all-targets -- -D warnings` exits 0.
- `cargo fmt --all -- --check` exits 0.

## Next Phase Readiness

Plan 151-09 (workspace version bump + CHANGELOG entry + auto-publish via Actions) is now unblocked. All eight implementation plans (01–08) have shipped; integration coverage is in place; the crate is publish-ready.

---
*Phase: 151-ferro-wallet-crate*
*Completed: 2026-05-11*
