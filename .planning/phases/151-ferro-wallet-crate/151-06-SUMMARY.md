---
phase: 151-ferro-wallet-crate
plan: 06
subsystem: testing
tags: [apple-wallet, pkpass, openssl, pkcs7, integration-test, x509]

# Dependency graph
requires:
  - phase: 151-05
    provides: ApplePassBuilder (PKCS#7 detached over SHA1 manifest in ZIP), SigningMaterial::parse, manifest::build_pass_json, package::zip_pkpass
  - phase: 151-02
    provides: WalletSubject trait, Field, Branding, FieldAlignment, PassKind, RgbColor, TextColorMode, GeoPoint
  - phase: 151-03
    provides: AppleConfig (PEM signing material struct)
  - phase: 151-04
    provides: image crate already a workspace dep — re-used for tiny_png helper
provides:
  - End-to-end structural validation of ApplePassBuilder::build under credential-free runtime cert minting (ACC-1j)
  - Reusable test pattern: mint_self_signed() + tiny_png() helpers for future wallet integration tests
affects: [151-09 release gate, gestiscilo-it wallet-passes integration]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Runtime self-signed X.509 minting via openssl::x509::X509Builder (no committed PEM)"
    - "Self-signed cert reused as WWDR intermediate — structurally valid for openssl PKCS#7 signing, deliberately ill-formed for Apple Wallet on-device chain validation"
    - "tiny_png() helper: solid-colour PNG via image::RgbaImage::from_pixel"
    - "ZIP shape + pass.json field assertions via zip::ZipArchive + serde_json::Value indexing"

key-files:
  created:
    - ferro-wallet/tests/apple_integration.rs
  modified: []

key-decisions:
  - "Self-signed cert minted at runtime, reused as both signer and WWDR (D-09). File-header comment carries the structure-only-NOT-Apple-validity disclaimer."
  - "ZIP entry-name assertions use HashSet (order-independent) — the entry count of 9 is asserted separately. Matches ACC-1j wording ('contains exactly 9 expected files') without coupling the test to ZIP iteration order."
  - "Assertions hit only the six pass.json fields enumerated in ACC-1j (passTypeIdentifier, teamIdentifier, serialNumber, barcode format+message, eventTicket.primaryFields[0].value). foregroundColor / labelColor / organizationName left unchecked — those have unit-test coverage in apple/manifest.rs."

patterns-established:
  - "Credential-free Apple integration testing: mint cert at runtime, never commit PEM material. Re-applicable to any future ferro-wallet test that needs a SigningMaterial."
  - "tiny_png(w, h, r, g, b): one-liner branding image helper for wallet tests."

requirements-completed: [ACC-1j]

# Metrics
duration: ~4min
completed: 2026-05-11
---

# Phase 151-ferro-wallet-crate Plan 06: Apple Integration Test Summary

**End-to-end `.pkpass` ZIP and `pass.json` structural validation against a runtime-minted self-signed X.509 cert — ACC-1j now green without any committed Apple credentials.**

## Performance

- **Duration:** ~4 min
- **Started:** 2026-05-11T04:13:33Z
- **Completed:** 2026-05-11T04:16:30Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments
- Added `ferro-wallet/tests/apple_integration.rs` (199 lines) — exercises the full `ApplePassBuilder::build` pipeline end-to-end against a runtime-minted RSA-2048 / SHA-256 self-signed cert.
- ACC-1j passes: ZIP contains exactly 9 named entries; `pass.json` carries the expected `passTypeIdentifier`, `teamIdentifier`, `serialNumber`, `barcodes[0].format`, `barcodes[0].message`, and `eventTicket.primaryFields[0].value`.
- No CI dependency on real Apple Developer credentials introduced — the test mints crypto material per-run and discards it.
- File-header comment makes the structure-only-NOT-Apple-validity disclaimer explicit (RESEARCH.md Risk 3 honoured).

## Task Commits

Each task was committed atomically:

1. **Task 1: Author tests/apple_integration.rs end-to-end (ACC-1j)** — `1d910994` (test)

_Note: this plan's single task is TDD-flavoured but the RED step is degenerate — the integration test cannot run before the file exists. The test passes immediately because 151-05 already shipped `ApplePassBuilder::build`. The "RED" discipline collapses to "test must fail by virtue of file absence", which is structurally true._

## Files Created/Modified

- `ferro-wallet/tests/apple_integration.rs` — Runtime cert minting + StubBooking + the `build_pkpass_produces_valid_zip_and_pass_json` test (199 lines).

## Decisions Made
- Reuse the self-signed cert as the WWDR intermediate per D-09 — openssl accepts the resulting `Pkcs7::sign(cert, key, [wwdr], …)` call because the WWDR `Stack<X509>` is non-empty, even though the cert/issuer relationship is not Apple-valid.
- Use `HashSet<String>` (order-independent) for the ZIP-entry-name assertions, paired with a separate `zip.len() == 9` check. This is faithful to ACC-1j's "contains 9 expected files" wording and doesn't lock the test to `package::zip_pkpass`'s insertion order.
- Use `expect("…")` with descriptive panic messages inside helpers and the test body. The integration test layer is allowed to panic on infrastructure failures — there is no upstream error handler in `cargo test`.

## Deviations from Plan

None — plan executed exactly as written. The reference body in the plan's `<action>` block was reproduced with two cosmetic adjustments:
1. `rustfmt` rewrapped one `let pkpass_bytes = builder.build(&StubBooking).expect(...)` line and one `assert_eq!(pass["eventTicket"]…)` line across multiple lines. No semantic change.
2. Added per-`expect()` panic messages (e.g., `"X509Builder::new"`, `"set X.509 v3"`) and a brief docstring on each helper. These improve debuggability if the openssl API ever changes — they don't alter behaviour and align with the project's "scientific and minimalistic" comment standard.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required. The test mints all crypto material at runtime.

## Next Phase Readiness

- ACC-1j now green. Apple builder is end-to-end validated under credential-free conditions.
- Plan 151-07 (Google builder) was already complete prior to this plan; Plan 151-08 (google_jwt integration test) is the next executable plan in the phase.
- Plan 151-09 (release gate) depends on 151-08 also being green. No blockers introduced by this plan.

## Self-Check: PASSED

- `ferro-wallet/tests/apple_integration.rs` — FOUND
- Commit `1d910994` — FOUND in `git log`
- `cargo test -p ferro-wallet --test apple_integration` exits 0 — VERIFIED
- `cargo test -p ferro-wallet` exits 0 (38 lib + 1 integration = 39 tests) — VERIFIED
- `cargo clippy -p ferro-wallet --all-targets -- -D warnings` exits 0 — VERIFIED
- `cargo fmt --all -- --check` exits 0 — VERIFIED
- `cargo build --workspace` exits 0 — VERIFIED
- `grep -F 'mint_self_signed' ferro-wallet/tests/apple_integration.rs` — MATCH
- `grep -F 'fn build_pkpass_produces_valid_zip_and_pass_json' ferro-wallet/tests/apple_integration.rs` — MATCH
- `grep -F 'NOT prove' ferro-wallet/tests/apple_integration.rs` — MATCH

---
*Phase: 151-ferro-wallet-crate*
*Completed: 2026-05-11*
