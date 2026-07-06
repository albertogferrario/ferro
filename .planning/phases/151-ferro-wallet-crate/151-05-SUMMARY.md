---
phase: 151-ferro-wallet-crate
plan: 05
subsystem: payments
tags: [apple-wallet, pkpass, pkcs7, openssl, zip, sha1, ferro-wallet]

# Dependency graph
requires:
  - phase: 151-02
    provides: WalletSubject trait + Branding/Field/RgbColor/TextColorMode/PassKind/FieldAlignment/GeoPoint value types + auto_foreground helper
  - phase: 151-03
    provides: AppleConfig (PEM signing cluster) + permissive WalletConfig::from_env
  - phase: 151-04
    provides: apple_logo_set + apple_icon_set image helpers (1x/2x/3x)
provides:
  - ApplePassBuilder::new (parse signing material from AppleConfig)
  - ApplePassBuilder::build<S: WalletSubject> (produce signed .pkpass ZIP)
  - apple/manifest.rs::build_pass_json + build_manifest (pure helpers)
  - apple/sign.rs::SigningMaterial::parse + sign_detached (PKCS#7 detached, DER)
  - apple/package.rs::zip_pkpass (Stored compression, 9-entry ZIP)
  - lib.rs re-export of ApplePassBuilder (D-11 restoration)
affects: [151-06 apple integration test, downstream gestiscilo-it wallet-passes consumer]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "PKCS#7 detached signing via openssl::pkcs7::Pkcs7::sign with non-empty Stack<X509> for WWDR"
    - "Byte-stable JSON manifest via BTreeMap<String, String> key ordering"
    - "Crate-private (pub(crate)) pipeline helpers with public facade type"
    - "Apple Wallet ZIP: CompressionMethod::Stored exclusively (deflate rejected)"

key-files:
  created: []
  modified:
    - ferro-wallet/src/apple/manifest.rs
    - ferro-wallet/src/apple/sign.rs
    - ferro-wallet/src/apple/package.rs
    - ferro-wallet/src/apple/mod.rs
    - ferro-wallet/src/lib.rs

key-decisions:
  - "labelColor tracks foregroundColor in v1 (D-06 v1 simplification): both fields receive the same BT.601-derived value"
  - "Manifest map ordering uses BTreeMap for byte-stable output regardless of input order — guards against signature non-determinism across multiple invocations of the same subject"
  - "manifest.rs imports super::ApplePassBuilder for borrowing pass_type_id/team_id/app_name fields; this is the only cross-file Apple coupling, kept intentional to avoid passing 3 parameters through the pipeline"
  - "Task atomicity preserved as 4 commits despite shared data dependencies — chain-builds only after Task 4 lands (deviation Rule 3, see below)"

patterns-established:
  - "Multi-file submodule with mod.rs facade + supporting pure helpers (mirrors ferro-stripe/src/webhook/ layout)"
  - "All openssl-error paths wrapped in WalletError::AppleSign with surgical 'subsystem: action: …' prefixes for production log greps"

requirements-completed: [ACC-1d]

# Metrics
duration: 4m 34s
completed: 2026-05-11
---

# Phase 151 Plan 05: Apple Builder Summary

**Apple `.pkpass` issuance pipeline — SHA1-manifest + PKCS#7-detached-signature + Stored-compression ZIP — landed as ApplePassBuilder::new/::build with full pass.json composition over WalletSubject.**

## Performance

- **Duration:** 4m 34s
- **Started:** 2026-05-11T03:58:25Z
- **Completed:** 2026-05-11T04:03:00Z (approximate)
- **Tasks:** 4 (all green)
- **Files modified:** 5

## Accomplishments

- `ApplePassBuilder::new(cfg, app_name)` parses signing cert + (optionally passphrase-protected) private key + WWDR intermediate via `SigningMaterial::parse`
- `ApplePassBuilder::build<S: WalletSubject>(s)` produces a complete `.pkpass` ZIP with exactly 9 entries
- `build_pass_json` emits Apple-compliant `pass.json` with all required identifiers, BT.601-derived colours (D-06), QR barcode, and `eventTicket`/`generic`/`coupon` field arrays per `PassKind`
- `build_manifest` emits byte-stable lowercase-hex SHA1 JSON map (BTreeMap key ordering)
- `sign_detached` produces DER-encoded PKCS#7 with `DETACHED | BINARY` flags, WWDR on non-empty `Stack<X509>`
- `zip_pkpass` uses `CompressionMethod::Stored` exclusively
- ACC-1d unit test (`manifest_sha1_lowercase_hex`) pins `SHA1("hello")` against the canonical digest
- 35 ferro-wallet unit tests green (up from 30 pre-plan: +3 manifest tests, +2 package tests)
- `lib.rs` `pub use apple::ApplePassBuilder;` re-export restored per D-11

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement apple/manifest.rs (build_pass_json + build_manifest) + ACC-1d test** — `66a5bede` (feat)
2. **Task 2: Implement apple/sign.rs (SigningMaterial::parse + sign_detached)** — `10e126db` (feat)
3. **Task 3: Implement apple/package.rs (zip_pkpass)** — `01022468` (feat)
4. **Task 4: Implement apple/mod.rs (ApplePassBuilder facade) + restore lib.rs re-export** — `79d6dfb3` (feat)

## ZIP Entry Order (output of ApplePassBuilder::build)

The produced `.pkpass` contains exactly 9 entries in this order:

1. `pass.json`
2. `manifest.json`
3. `signature`
4. `logo.png` (160×50)
5. `logo@2x.png` (320×100)
6. `logo@3x.png` (480×150)
7. `icon.png` (29×29)
8. `icon@2x.png` (58×58)
9. `icon@3x.png` (87×87)

## pass.json Keys Emitted

Always present:

- `formatVersion` (= 1)
- `passTypeIdentifier` (from `AppleConfig.pass_type_id`)
- `teamIdentifier` (from `AppleConfig.team_id`)
- `serialNumber` (from `subject.serial()`)
- `organizationName` (from `branding.organization_name` falling back to `app_name`)
- `description` (= `app_name`)
- `backgroundColor` (CSS `rgb(r,g,b)` from `branding.background_color`)
- `foregroundColor` (BT.601-derived per D-06: white for dark backgrounds, dark slate for light)
- `labelColor` (tracks `foregroundColor` per D-06 v1)
- `barcodes` (single QR entry: `PKBarcodeFormatQR`, `subject.barcode_token()`, `iso-8859-1` encoding)
- One of `eventTicket` / `generic` / `coupon` field bundle (matching `subject.pass_kind()`)

Conditionally present (only when non-empty / `Some`):

- `logoText` (when `branding.logo_text` is `Some`)
- `relevantDate` (when `subject.relevant_at()` is `Some`, RFC-3339)
- `expirationDate` (when `subject.expires_at()` is `Some`, RFC-3339)
- `locations` (when `subject.locations()` is non-empty; each `latitude`/`longitude` plus optional `relevantText`)

Field bundle shape (`eventTicket`/`generic`/`coupon`):

- `primaryFields`: one-element array `[ subject.primary() ]`
- `secondaryFields`: array of `subject.secondary()`
- `auxiliaryFields`: array of `subject.auxiliary()`
- `backFields`: array of `subject.back()`

Each `Field` serialises as `{ key, label, value, textAlignment }` where `textAlignment` is one of `PKTextAlignmentLeft` / `PKTextAlignmentCenter` / `PKTextAlignmentRight` / `PKTextAlignmentNatural`.

## Manifest Input Set

The SHA1 manifest digests exactly 7 entries (NOT `manifest.json`, NOT `signature`):

1. `pass.json`
2. `logo.png`
3. `logo@2x.png`
4. `logo@3x.png`
5. `icon.png`
6. `icon@2x.png`
7. `icon@3x.png`

The manifest is then PKCS#7-signed (producing `signature`), and both `manifest.json` and `signature` are appended to the ZIP afterwards.

## Decisions Made

- **labelColor tracks foregroundColor in v1** — per CONTEXT.md D-06; defers per-pass label-vs-foreground colour separation to a future phase.
- **BTreeMap for manifest ordering** — guarantees byte-stable manifest JSON across identical inputs, which is load-bearing for signature determinism (RESEARCH.md Risk 7).
- **Stored compression only** — Apple Wallet rejects deflated entries (RESEARCH.md Pitfall 1).
- **WWDR pushed onto non-empty `Stack<X509>`** — `Pkcs7::sign` with an empty stack produces a chain that fails on-device validation (RESEARCH.md Pitfall 2).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Per-task `cargo build` verifies are physically impossible**

- **Found during:** Task 1 verification
- **Issue:** The plan's per-task `verify.automated` blocks each require `cargo build -p ferro-wallet` to pass. However, the four task files have explicit data dependencies — `manifest.rs` references `super::ApplePassBuilder` (defined in `mod.rs` only after Task 4), `mod.rs` references `sign::SigningMaterial` (Task 2) and `package::zip_pkpass` (Task 3). Each task in isolation cannot compile.
- **Fix:** Implemented all 4 tasks first, then validated the full chain (`cargo build` + `cargo test` + `cargo clippy -D warnings` + `cargo fmt --check`) once, then made 4 atomic commits in plan-specified order. This preserves the atomic-commit-per-task git history the plan requires while acknowledging the unavoidable data coupling the plan itself states upfront: "The four files share data dependencies ... so they belong in one plan."
- **Files modified:** No additional files — the deviation is in workflow ordering, not in code.
- **Verification:** `cargo test -p ferro-wallet --lib` runs 35 tests, all pass. `cargo clippy -p ferro-wallet --all-targets -- -D warnings` clean. `cargo fmt --all -- --check` clean. `cargo build --workspace` clean.
- **Committed in:** All four task commits (`66a5bede`, `10e126db`, `01022468`, `79d6dfb3`) — chain compiles end-to-end at HEAD; intermediate commits do not standalone-compile.

**2. [Rule 1 - Formatting fixup] rustfmt rewrapped Stack::new() chain**

- **Found during:** Task 4 fmt-check verification
- **Issue:** Initial sign.rs formatting placed `Stack::new().map_err(...)?` on two lines via the `let mut wwdr_stack: Stack<X509> = Stack::new()` pattern; rustfmt 2024-edition preferred collapsing to `let mut wwdr_stack: Stack<X509> = Stack::new().map_err(...)?;`
- **Fix:** Ran `cargo fmt --all` once, accepted the rewrite. No semantic change.
- **Files modified:** `ferro-wallet/src/apple/sign.rs`
- **Verification:** `cargo fmt --all -- --check` exits 0 after the apply.
- **Committed in:** `10e126db` (Task 2 commit) — the fmt fix was applied before the Task 2 commit landed.

---

**Total deviations:** 2 auto-fixed (1 Rule 3 blocking workflow, 1 Rule 1 formatting)
**Impact on plan:** No scope creep. The Rule 3 deviation is purely about how to interpret the per-task verify blocks given inherent data coupling; the final state matches the plan's stated success criteria exactly.

## Issues Encountered

None beyond the documented deviations.

## Threat Flags

None — this plan implements exactly the threat surface the plan's `<threat_model>` already enumerated (T-151-Apple-MANIFEST, T-151-Apple-SIGN, T-151-Apple-COLOR, T-151-DEFAULT-CRED). All `mitigate` dispositions have implementations in place; ACC-1d test pins manifest format; grep checks pin DETACHED|BINARY + WWDR stack push + Stored compression.

## Next Phase Readiness

- PLAN-06 (`tests/apple_integration.rs`) can now write end-to-end coverage against `ApplePassBuilder::new` and `::build`. Runtime-minted self-signed X.509 will exercise the full pipeline.
- `ApplePassBuilder` is publicly exported from `ferro_wallet::ApplePassBuilder`.

## Self-Check: PASSED

Files verified:

- `ferro-wallet/src/apple/manifest.rs` — FOUND
- `ferro-wallet/src/apple/sign.rs` — FOUND
- `ferro-wallet/src/apple/package.rs` — FOUND
- `ferro-wallet/src/apple/mod.rs` — FOUND
- `ferro-wallet/src/lib.rs` — FOUND (with `pub use apple::ApplePassBuilder;` restored)

Commits verified:

- `66a5bede` — FOUND (Task 1: manifest)
- `10e126db` — FOUND (Task 2: sign)
- `01022468` — FOUND (Task 3: package)
- `79d6dfb3` — FOUND (Task 4: mod.rs + lib.rs)

Grep invariants pinned:

- `pub(crate) fn build_manifest` in manifest.rs — FOUND
- `pub(crate) fn build_pass_json` in manifest.rs — FOUND
- `BTreeMap` in manifest.rs — FOUND (3 matches)
- `Pkcs7Flags::DETACHED | Pkcs7Flags::BINARY` in sign.rs — FOUND
- `wwdr_stack.push` in sign.rs — FOUND
- `pub(crate) struct SigningMaterial` in sign.rs — FOUND
- `CompressionMethod::Stored` in package.rs — FOUND
- `pub(crate) fn zip_pkpass` in package.rs — FOUND
- `pub struct ApplePassBuilder` in apple/mod.rs — FOUND
- `pub fn build<S: WalletSubject>` in apple/mod.rs — FOUND
- `pub use apple::ApplePassBuilder;` in lib.rs — FOUND

Test results:

- `cargo test -p ferro-wallet --lib apple::manifest::tests::manifest_sha1_lowercase_hex` — passed (ACC-1d)
- `cargo test -p ferro-wallet --lib apple::manifest::tests` — 3 tests passed
- `cargo test -p ferro-wallet --lib apple::package::tests` — 2 tests passed
- `cargo test -p ferro-wallet --lib` — 35/35 passed
- `cargo clippy -p ferro-wallet --all-targets -- -D warnings` — clean
- `cargo fmt --all -- --check` — clean
- `cargo build --workspace` — clean

---
*Phase: 151-ferro-wallet-crate*
*Completed: 2026-05-11*
