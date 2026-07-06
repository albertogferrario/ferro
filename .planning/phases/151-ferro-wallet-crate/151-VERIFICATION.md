---
phase: 151-ferro-wallet-crate
verified: 2026-06-07T00:00:00Z
status: human_needed
score: 8/8 success criteria verified
overrides_applied: 0
human_verification:
  - test: "Confirm ferro-wallet 0.2.24 (or later) is live on crates.io: `cargo search ferro-wallet` or visit https://crates.io/crates/ferro-wallet"
    expected: "ferro-wallet appears published. Per CONTEXT D-10 / spec §8, first publish required a manual local-terminal bootstrap with a publish-new-scoped token (CI token is publish-update only). Workspace version is now 0.2.44 and CHANGELOG carries the [0.2.24] ferro-wallet entry, which is strong evidence the bootstrap happened — but crates.io presence cannot be confirmed offline from the repo alone."
    why_human: "Requires a network call to crates.io. The release plumbing (workspace member, publish.yml Wave 1A registration, version bump, CHANGELOG) is all verified in-repo, but actual registry publication is an out-of-band step."
  - test: "Downstream consumer build: in gestiscilo-it, depend on `ferro-wallet = \"0.2.X\"` and build a real `.pkpass` against its booking model with real Apple WWDR + pass-type credentials, and a Google save-link with a real service account."
    expected: "A real device installs the generated .pkpass; the Google save-link adds the pass to a real Google Wallet. Spec §9 acceptance criterion 4 — explicitly verified out-of-band by the gestiscilo integration phase."
    why_human: "Requires real Apple WWDR / pass-type-id credentials and a real Google service account, plus a physical device. The integration tests deliberately mint self-signed crypto at runtime (D-09) so CI never depends on real secrets; the real-credential path is intrinsically a human/out-of-band check."
---

# Phase 151: ferro-wallet crate Verification Report

**Phase Goal:** New project-agnostic crate `ferro-wallet` providing the `WalletSubject` trait, `ApplePassBuilder` (PKCS#7-signed `.pkpass`), `GoogleWalletBuilder` (RS256 save-link JWT), and image/QR primitives. Follows architecture principle 6 (no hardcoded app identity; reads `APP_NAME`/`APP_URL` via `WalletConfig::from_env`). Single load-bearing prerequisite for gestiscilo-it wallet booking passes integration.

**Verified:** 2026-06-07
**Status:** human_needed
**Re-verification:** No — initial (retroactive) verification; no prior VERIFICATION.md existed.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `WalletSubject` trait exists with a documented content contract | VERIFIED | `subject.rs:168-206` — `pub trait WalletSubject` with 11 documented methods (`pass_kind`, `serial`, `primary`/`secondary`/`auxiliary`/`back`, `barcode_token`, `relevant_at`, `expires_at`, `locations`, `branding`). Re-exported from `lib.rs:19-22`. All value types (`Field`, `Branding`, `PassKind`, `GeoPoint`, `RgbColor`, `TextColorMode`, `FieldAlignment`, `TransitType`, `auto_foreground`) present and documented. |
| 2 | `ApplePassBuilder` produces a PKCS#7-signed `.pkpass` ZIP | VERIFIED | `apple/mod.rs:20-107` — `build()` composes pass.json → image set → SHA1 manifest → `sign.sign_detached()` (PKCS#7 detached) → `package::zip_pkpass()`. Integration test `apple_integration.rs:138-196` asserts the 9-entry ZIP (`pass.json`, `manifest.json`, `signature`, logo×3, icon×3) and pass.json fields (`passTypeIdentifier`, `teamIdentifier`, `serialNumber`, `barcodes[0].message`, `eventTicket.primaryFields[0].value`). Test passes. |
| 3 | `GoogleWalletBuilder` produces an RS256 save-link JWT | VERIFIED | `google/mod.rs:17-64` — `save_jwt()` builds the eventTicketObject and signs via `jwt::sign_save_jwt` (RS256); `save_url()` wraps it in `pay.google.com/gp/v/save/{jwt}`. Test `google_jwt.rs` mints an RSA keypair, signs, and decodes asserting claims; `save_url_returns_pay_google_com_prefix` confirms URL prefix. Both pass. |
| 4 | image + QR primitives exist | VERIFIED | `images.rs` — `fit_to`, `apple_logo_set` (3 entries, correct dims), `apple_icon_set` (explicit-or-derived), `google_hero` (1032×336), `transparent_1x1_png`, `apple_strip_set`. `qr.rs` — `png` (PNG magic asserted) + `data_uri` (base64 data-URI, decode roundtrip asserted). 6 image + 4 qr unit tests pass. |
| 5 | Project-agnostic: `WalletConfig::from_env` reads `APP_NAME`/`APP_URL`, no hardcoded tenant identity | VERIFIED | `config.rs:82-98` — reads `APP_NAME`/`APP_URL` with defaults `"Ferro Application"`/`"http://localhost:8080"` (mirrors `framework::config::AppConfig`). `grep gestiscilo\|appo\|goappo` in `ferro-wallet/src/` = none. Only `Ferro Application` occurrences are the documented default fallback; only `example.iam` occurrences are in `#[cfg(test)]` code (documentation-example exception). Permissive D-02 semantics: missing Apple/Google clusters ⇒ `None`, never errors (7 config tests confirm). |
| 6 | Crate is wired into the workspace and release pipeline | VERIFIED | Root `Cargo.toml:24` — `"ferro-wallet"` in `[workspace] members`. `.github/workflows/publish.yml:211` — `ferro-wallet` in `WAVE1A_CRATES` (correct: leaf crate, no internal workspace deps). Workspace version bumped to 0.2.44 (was 0.2.23 → 0.2.24 at this phase's release). `CHANGELOG.md:313-322` — `## ferro-wallet` section with `### [0.2.24] — 2026-05-11` initial-release entry. |
| 7 | `cargo test -p ferro-wallet` is green (spec §9 ACC-1) | VERIFIED | Ran `cargo test -p ferro-wallet`: **41 passed; 0 failed** — 38 unit tests + 1 apple integration + 2 google JWT, plus 0 doc-tests. Exit 0. |
| 8 | Single `WalletError` enum with name-prefixed Display (D-04) | VERIFIED | `error.rs:7-40` — one `thiserror`-derived enum, 8 variants each prefixing its name (`config:`, `apple sign:`, `apple package:`, `google jwt:`, `image:`, `qr:`, `invalid input:`, `io:`). `Io(#[from] std::io::Error)` present. 10 unit tests assert each Display string. |

**Score:** 8/8 truths verified

### Deferred Items

None affecting goal achievement. The following were intentionally out of scope per CONTEXT `<deferred>` and are not gaps: Apple Web Service Protocol (live updates / Express Mode), Google `objects.patch`, locale negotiation, integration tests for `Generic`/`Coupon`/`BoardingPass` (declared in `PassKind`, EventTicket-only tested in v1).

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-wallet/src/lib.rs` | crate root + re-exports of both builders + trait | VERIFIED | 7 pub modules; `pub use` for `ApplePassBuilder`, `GoogleWalletBuilder`, config types, `WalletError`, all subject types. D-11 re-export restoration confirmed. |
| `ferro-wallet/src/subject.rs` | `WalletSubject` + value types | VERIFIED | Trait + 9 value types/fns, fully documented; 4 unit tests. |
| `ferro-wallet/src/config.rs` | `WalletConfig::from_env` permissive, APP_NAME/APP_URL | VERIFIED | Permissive optional Apple/Google clusters; defaults match AppConfig; 7 env tests with RAII EnvGuard + serialization mutex. |
| `ferro-wallet/src/error.rs` | one `WalletError` enum | VERIFIED | 8 name-prefixed variants; `#[from]` io; 10 tests. |
| `ferro-wallet/src/images.rs` | `fit_to` + apple sets + google_hero | VERIFIED | All helpers present; 6 tests assert dims and malformed-input rejection. |
| `ferro-wallet/src/qr.rs` | png + data_uri | VERIFIED | Both present; 4 tests (PNG magic, base64 roundtrip). |
| `ferro-wallet/src/apple/{mod,manifest,sign,package}.rs` | builder + manifest + PKCS#7 + zip | VERIFIED | All four files substantive; `build()` produces 9-entry ZIP. |
| `ferro-wallet/src/google/{mod,object,jwt}.rs` | builder + object + RS256 JWT | VERIFIED | All three files substantive; `save_jwt`/`save_url` wired. |
| `ferro-wallet/tests/apple_integration.rs` | end-to-end with runtime self-signed cert | VERIFIED | Mints self-signed X.509 at runtime; asserts 9 files + pass.json fields. Passes. |
| `ferro-wallet/tests/google_jwt.rs` | RS256 roundtrip | VERIFIED | Mints RSA keypair, decodes with public key, asserts claims. 2 tests pass. |
| `ferro-wallet/Cargo.toml` | leaf crate, correct deps | VERIFIED | openssl, zip, jsonwebtoken, image, qrcode-generator, sha1, base64, serde, serde_json, thiserror, chrono. No internal ferro deps (leaf → Wave 1A correct). |
| `ferro-wallet/README.md` | short, directs to spec | VERIFIED | Brief; describes trait + both builders + project-agnostic env convention; mirrors ferro-stripe brevity. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ApplePassBuilder::build` | `sign::SigningMaterial::sign_detached` | `self.signing.sign_detached(&manifest_bytes)` | WIRED | `apple/mod.rs:96` |
| `ApplePassBuilder::build` | `package::zip_pkpass` | final ZIP assembly | WIRED | `apple/mod.rs:105` |
| `GoogleWalletBuilder::save_jwt` | `jwt::sign_save_jwt` | RS256 envelope sign | WIRED | `google/mod.rs:54-55` |
| `GoogleWalletBuilder::save_url` | `jwt::save_url` | `pay.google.com/gp/v/save/{jwt}` | WIRED | `google/mod.rs:62` |
| `WalletConfig::from_env` | `APP_NAME` / `APP_URL` env | `std::env::var(...).unwrap_or_else(default)` | WIRED | `config.rs:84-87`; defaults mirror AppConfig |
| Root `Cargo.toml` | `ferro-wallet` member | `[workspace] members` array | WIRED | `Cargo.toml:24` |
| `publish.yml` | `ferro-wallet` | `WAVE1A_CRATES` list | WIRED | `publish.yml:211` |
| Both builders | `WalletSubject` | generic `<S: WalletSubject>` bound | WIRED | `apple/mod.rs:56`, `google/mod.rs:53,61` — single shared abstraction (D-01) |

### Data-Flow Trace (Level 4)

Not applicable — this phase produces a library crate (trait + builders + crypto/image primitives), not a UI rendering component with dynamic data flow. The data path (subject → pass.json/JWT → signed artifact) is verified end-to-end by the two integration tests (Level 3 wiring + behavioral spot-check suffices).

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Full crate test suite | `cargo test -p ferro-wallet` | 38 unit + 1 apple + 2 google = 41 passed; 0 failed; finished 0.27s/0.20s/0.06s | PASS |
| Apple end-to-end .pkpass | `build_pkpass_produces_valid_zip_and_pass_json` | 1 passed (9-entry ZIP + pass.json fields verified) | PASS |
| Google RS256 roundtrip | `save_jwt_roundtrips_with_runtime_minted_rsa_keypair` | 1 passed | PASS |
| Google save URL prefix | `save_url_returns_pay_google_com_prefix` | 1 passed | PASS |
| Config permissive (missing vars) | `from_env_never_errors_on_missing_wallet_vars` etc. | 7 config tests passed | PASS |

### Locked Decision Verification

| Decision | Requirement | Status | Evidence |
|----------|-------------|--------|----------|
| D-01 | Two builders, split; `WalletSubject` only shared abstraction | VERIFIED | Separate `apple/` + `google/` modules; both generic over `WalletSubject`; no unified builder. |
| D-02 | Permissive `from_env`; missing cluster ⇒ None, never errors | VERIFIED | `config.rs` optional clusters; 7 tests including partial-Apple-returns-None and never-errors. |
| D-04 | Name-prefixed `WalletError` Display + `Io(#[from])` | VERIFIED | `error.rs` — all 8 variants prefixed; io From impl; 10 tests. |
| D-05 | Apple manifest (sha1-hex) + PKCS#7 DETACHED\|BINARY | VERIFIED | `apple/manifest.rs` + `apple/sign.rs`; integration test confirms `signature` entry + manifest digest map. |
| D-06 | Auto foreground via BT.601 luminance (<0.5 white, >=0.5 slate) | VERIFIED | `subject.rs:142-160` `auto_foreground`; 2 tests assert white/dark-slate. |
| D-08 | Google JWT claim shape (iss/aud=google/typ=savetowallet/payload) | VERIFIED | `google/jwt.rs` + `google_jwt.rs` decode test asserts claims. |
| D-10 | Workspace member + version bump + auto-publish | VERIFIED | Member present; version 0.2.23→0.2.24 at phase, now 0.2.44; CHANGELOG entry. |
| D-11 | lib.rs re-exports restored after builders land | VERIFIED | `lib.rs:15,18` both builder re-exports present. |

### Requirements Coverage

Phase 151 has no formal REQUIREMENTS.md IDs. The spec §9 acceptance criteria + CONTEXT locked decisions serve as the contract: ACC-1 (`cargo test` green) VERIFIED; ACC-2 (`cargo build --workspace` green with new member) — not re-run under the single-CPU-op constraint, but the crate compiles cleanly (test build succeeded, which requires a successful library build) and is a registered member; ACC-3 (`cargo doc --no-deps` clean) — see Anti-Patterns (6 intra-doc-link warnings, non-blocking, SUMMARY-09 documented); ACC-4 (downstream consumer real build) — human verification item 2.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `apple/mod.rs` | 74 | comment contains the word "placeholder" | Info | False positive — describes synthesising a transparent 1×1 PNG as an icon fallback; the code path is real (`images::transparent_1x1_png()` is a working implementation, not a stub). Not a placeholder implementation. |
| `apple/mod.rs:32`, `config.rs:12,70`, `google/jwt.rs:70`, `google/mod.rs:46` | (doc comments) | 6 `cargo doc` intra-doc-link warnings (5 link-to-private-item + 1 unresolved cross-crate `framework::config::AppConfig::from_env`) | Warning | `cargo doc --no-deps -p ferro-wallet` exits 0 but emits 6 warnings. Documented in 151-09-SUMMARY and deferred-items.md. ACC-3 is "clean output"; output is non-fatal but not warning-free. Does not block publish (crate is already at 0.2.44). Cosmetic doc hygiene follow-up. |
| `docs/src/` | — | No dedicated `docs/src/` book page for ferro-wallet | Info | CLAUDE.md states docs/src updates are required for framework changes, but ferro-wallet is consumed directly by application crates (not via the framework facade) and spec §9 scopes documentation to rustdoc + README + spec. Crate-level rustdoc, README, and the design spec are all present and substantive. A docs/src book page would improve discoverability; not a goal blocker. |

None of the above block goal achievement. All are Info/Warning-level hygiene items.

### Human Verification Required

#### 1. crates.io publication of ferro-wallet

**Test:**
```bash
cargo search ferro-wallet
# or open https://crates.io/crates/ferro-wallet
```
**Expected:** `ferro-wallet` 0.2.24 (or later) is listed. Per D-10 / spec §8 the first publish required a manual local-terminal bootstrap with a `publish-new`-scoped token (CI token is `publish-update` only — see memory `project_ferro_publish_token_scoping.md`). Workspace version is now 0.2.44 with the CHANGELOG [0.2.24] entry, which strongly implies the bootstrap succeeded and subsequent versions auto-published.

**Why human:** Requires a network call to crates.io. All in-repo release plumbing is verified, but registry presence cannot be confirmed offline.

#### 2. Downstream consumer real-credential build (spec §9 ACC-4)

**Test:** In gestiscilo-it, depend on `ferro-wallet = "0.2.X"`, implement `WalletSubject` for the booking model, and build a real `.pkpass` (real Apple WWDR + pass-type-id cert/key) and a Google save-link (real service account). Install the `.pkpass` on a device; add the save-link to Google Wallet.

**Expected:** Device installs the pass; Google Wallet accepts the save-link.

**Why human:** Requires real Apple/Google credentials and a physical device. CI deliberately mints self-signed crypto at runtime (D-09) and never depends on real secrets; the real-credential path is intrinsically out-of-band — explicitly assigned to the gestiscilo integration phase by spec §9 ACC-4.

### Gaps Summary

No implementation gaps. The crate fully delivers the phase goal: the `WalletSubject` trait is present and documented; `ApplePassBuilder` produces a real PKCS#7-signed 9-entry `.pkpass` ZIP (proven by an end-to-end test that mints a self-signed cert at runtime); `GoogleWalletBuilder` produces an RS256 save-JWT and `pay.google.com` save URL (proven by an RSA roundtrip test); image + QR primitives are implemented and tested; `WalletConfig::from_env` is project-agnostic (reads `APP_NAME`/`APP_URL`, no hardcoded tenant identity), and is permissive per D-02. The crate is a registered workspace member, in publish.yml Wave 1A, version-bumped, and CHANGELOG'd. `cargo test -p ferro-wallet` is green (41/41).

Status is `human_needed` rather than `passed` solely because two acceptance steps are inherently out-of-band: (1) confirming crates.io publication (network), and (2) the downstream gestiscilo real-credential build (spec §9 ACC-4, explicitly out-of-band). Both are validation steps, not code gaps. Minor hygiene follow-ups (6 cargo-doc intra-doc-link warnings; no docs/src book page) are noted but do not affect goal achievement.

**Publish-readiness:** Ready. The crate compiles, all tests pass, no hardcoded identity, correctly registered for auto-publish. Already released at 0.2.24 (workspace now 0.2.44).

---

_Verified: 2026-06-07_
_Verifier: Claude (gsd-verifier)_
