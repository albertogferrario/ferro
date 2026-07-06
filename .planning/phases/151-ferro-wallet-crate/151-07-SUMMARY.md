---
phase: 151-ferro-wallet-crate
plan: 07
subsystem: payments
tags: [google-wallet, jwt, rs256, jsonwebtoken, wallet-passes]

# Dependency graph
requires:
  - phase: 151-02
    provides: WalletSubject trait + Field/Branding value types
  - phase: 151-03
    provides: GoogleConfig (issuer_id, service_account_email, service_account_private_key_pem) + WalletConfig.app_name/app_url
provides:
  - GoogleWalletBuilder public API (new/save_jwt/save_url)
  - sign_save_jwt internal helper (RS256 over SaveClaims envelope)
  - build_event_ticket_object internal helper (JSON shape per spec §3 + D-07)
  - pass_type_id_default() const fn returning the v1 fixed "booking" suffix
  - save_url(jwt) → "https://pay.google.com/gp/v/save/{jwt}"
affects:
  - 151-08-google-jwt-test (integration test consumes GoogleWalletBuilder)
  - 151-09-release (final crate version bump)
  - downstream gestiscilo-wallet-passes (depends on this re-export landing)

# Tech tracking
tech-stack:
  added:
    - jsonwebtoken 9 (RS256 EncodingKey::from_rsa_pem + encode)
    - chrono::Utc::now().timestamp() for the JWT iat claim
  patterns:
    - "Forward-declared builder struct: GoogleWalletBuilder is declared in google/mod.rs in the same commit that lands jwt.rs so the helper functions can take &GoogleWalletBuilder without forward-declaration tricks; the impl block lands in the same plan one commit later."
    - "Internal pub(crate) helpers + thin public facade: jwt::sign_save_jwt and object::build_event_ticket_object stay crate-private; the public surface is exactly GoogleWalletBuilder::{new, save_jwt, save_url} + jwt::{pass_type_id_default, save_url}."

key-files:
  created: []
  modified:
    - ferro-wallet/src/google/jwt.rs (was placeholder)
    - ferro-wallet/src/google/object.rs (was placeholder)
    - ferro-wallet/src/google/mod.rs (was placeholder)
    - ferro-wallet/src/lib.rs (restored 'pub use google::GoogleWalletBuilder;' per D-11)

key-decisions:
  - "JWT claim shape per D-08 unchanged: iss = service_account_email, aud = \"google\", typ = \"savetowallet\", iat = unix-now, origins = [app_url], payload.eventTicketObjects = [single object]. No exp claim — Google validates the JWT at user-click time via origin pinning, not via server-side expiry."
  - "EventTicketObject shape per spec §3 + D-07: id = \"{issuer_id}.{subject.serial()}\", classId = \"{issuer_id}.{pass_type_id_default()}\" with future-proof dot-to-underscore substitution on the pass-type suffix (no-op for the v1 \"booking\" value)."
  - "pass_type_id_default() declared const fn returning \"booking\" — keeps the symbol callable from both runtime code and any future const-context that needs it; the dot-to-underscore rule from D-07 is implemented in object.rs at runtime since &str::replace is non-const."
  - "GoogleWalletBuilder::app_name stored for symmetry with ApplePassBuilder; D-08 explicitly omits it from the JWT payload, so the field is marked #[allow(dead_code)] until a downstream builder phase wires it into per-pass metadata."
  - "Task ordering produced an internal-only compile-island problem: jwt.rs and object.rs both reference &GoogleWalletBuilder, but the struct lives in mod.rs (Task 3). Resolved by landing the struct definition in Task 1's commit (with #[allow(dead_code)] on the unused fields/items) and the impl block in Task 3 — keeps every per-task commit individually build-clean without splitting jwt.rs across two commits."

patterns-established:
  - "Forward-declared internal-use struct pattern: when a multi-file submodule's helper functions take a &Type reference where Type is defined in mod.rs, declare the struct (fields only) in mod.rs first, then have helper files reference it via super::. The impl block lands later. Avoids a circular-build problem cleanly."
  - "#[allow(dead_code)] as a short-lived plan-internal annotation: applied to symbols that will be wired up by a later task in the same plan, removed in the wiring task's commit. Documented in the annotation comment so reviewers can see the intent."

requirements-completed: [ACC-1i]

# Metrics
duration: ~10min
completed: 2026-05-11
---

# Phase 151 Plan 07: Google Wallet Builder Summary

**RS256-signed save JWT pipeline (subject → eventTicketObject → SaveClaims envelope → pay.google.com/gp/v/save/{jwt}) wired through GoogleWalletBuilder, with the v1 fixed "booking" pass type and full crate-level re-export restored.**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-05-11T04:01:00Z (approx; first Task 1 commit timestamp)
- **Completed:** 2026-05-11T04:11:36Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- `jsonwebtoken::encode(&Header::new(Algorithm::RS256), &SaveClaims { … }, &EncodingKey::from_rsa_pem(…))` pipeline lands in `google/jwt.rs` with the exact claim shape D-08 specifies (iss/aud=google/typ=savetowallet/iat/origins/payload.eventTicketObjects).
- `build_event_ticket_object` in `google/object.rs` produces the JSON shape Google Wallet's REST API consumes (id, classId, state="active", barcode={type:"qrCode", value}, ticketHolderName, eventName.defaultValue), with D-07's `class_id = "{issuer_id}.{pass_type_id_default()}"` future-proofed via the dot-to-underscore substitution.
- `GoogleWalletBuilder::{new, save_jwt, save_url}` in `google/mod.rs` is the thin public facade — `save_jwt` linearly composes the object then signs the envelope; `save_url` wraps that JWT in the canonical save URL.
- `pub use google::GoogleWalletBuilder;` restored in `lib.rs` per D-11 — the crate's public surface is now complete for the Google path; the integration test (PLAN-08) can write against it.
- ACC-1i (`google::jwt::tests::save_url_format`) passes alongside two additional unit tests (`save_url_empty_jwt`, `pass_type_id_default_is_booking`); 38 ferro-wallet unit tests pass total with zero regressions.

## Task Commits

Each task was committed atomically:

1. **Task 1: jwt.rs (sign_save_jwt + save_url + pass_type_id_default) + ACC-1i test** — `d3f9fced` (feat)
2. **Task 2: object.rs (build_event_ticket_object)** — `5d2a9eee` (feat)
3. **Task 3: GoogleWalletBuilder impl + lib.rs re-export** — `c5fb975e` (feat)

## Files Created/Modified

- `ferro-wallet/src/google/jwt.rs` — `pass_type_id_default()` const fn, `SaveClaims<'a>` serde envelope, `sign_save_jwt(builder, event_ticket_object) -> Result<String, WalletError>` (RS256 via `jsonwebtoken`), `save_url(jwt) -> String`. Three unit tests: `save_url_format` (ACC-1i), `save_url_empty_jwt`, `pass_type_id_default_is_booking`.
- `ferro-wallet/src/google/object.rs` — `build_event_ticket_object<S: WalletSubject>(builder, subject) -> Result<Value, WalletError>` producing the JSON shape per spec §3 + D-07 (id, classId, state, barcode={type:"qrCode", value}, ticketHolderName, eventName).
- `ferro-wallet/src/google/mod.rs` — `pub struct GoogleWalletBuilder { issuer_id, service_account_email, private_key_pem, app_name, app_url }` + `impl GoogleWalletBuilder { new, save_jwt<S>, save_url<S> }`. Struct fields are `pub(crate)`; `app_name` is `#[allow(dead_code)]` because D-08 omits it from the JWT payload.
- `ferro-wallet/src/lib.rs` — replaced `// pub use google::GoogleWalletBuilder;   // Restored in PLAN-07` with the live `pub use google::GoogleWalletBuilder;` re-export.

## Decisions Made

- **JWT claim shape per D-08 (unchanged):** `iss = service_account_email`, `aud = "google"`, `typ = "savetowallet"`, `iat = unix-now`, `origins = [app_url]`, `payload = { "eventTicketObjects": [<one object>] }`. No `exp` claim — by design (D-08 + threat-model row T-151-Google-JWT). PLAN-08's integration test will set `Validation.validate_exp = false` to match.
- **Class + object ID derivation per D-07:** `class_id = format!("{}.{}", issuer_id, pass_type_id_default().replace('.', "_"))` and `object_id = format!("{}.{}", issuer_id, subject.serial())`. The dot-to-underscore substitution is a no-op for v1's `"booking"` but kept future-proof for any pass type that includes dots.
- **`pass_type_id_default()` declared as `const fn`** so the symbol stays callable from both runtime code and any future const context. The dot-substitution itself is runtime (string `replace` is non-const), implemented inside `object.rs`.
- **Forward-declare `GoogleWalletBuilder` struct in Task 1** to keep every per-task commit individually build-clean. Without this, Task 1's `jwt.rs` and Task 2's `object.rs` would not compile until Task 3's struct definition landed. Resolution: land struct + fields in mod.rs in Task 1's commit (annotated `#[allow(dead_code)]`), then add the `impl` block in Task 3 alongside the `lib.rs` re-export restoration.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Forward-declared GoogleWalletBuilder struct in Task 1's commit to make per-task commits individually build-clean**
- **Found during:** Task 1 (jwt.rs)
- **Issue:** The plan's stated task order — Task 1 = jwt.rs (uses `&GoogleWalletBuilder`), Task 2 = object.rs (also uses `&GoogleWalletBuilder`), Task 3 = mod.rs (defines `GoogleWalletBuilder`) — leaves Task 1's commit unable to compile because `GoogleWalletBuilder` doesn't exist as a type yet. Task 1's `<verify>` block explicitly runs `cargo build -p ferro-wallet` which would fail.
- **Fix:** Added the `GoogleWalletBuilder` struct definition (fields only, no `impl`) to `mod.rs` as part of Task 1's commit, with `#[allow(dead_code)]` on the struct and the unused helper items (`SaveClaims`, `sign_save_jwt`, `build_event_ticket_object`). Task 3 adds the `impl` block + restores the `lib.rs` re-export + drops the `#[allow(dead_code)]` annotations on `sign_save_jwt` / `build_event_ticket_object` (still needed on the `app_name` field per D-08).
- **Files modified:** `ferro-wallet/src/google/mod.rs` (Task 1 commit `d3f9fced`), `ferro-wallet/src/google/jwt.rs`, `ferro-wallet/src/google/object.rs`
- **Verification:** Every per-task commit (`d3f9fced`, `5d2a9eee`, `c5fb975e`) passes `cargo build -p ferro-wallet && cargo clippy -p ferro-wallet --all-targets -- -D warnings && cargo fmt -p ferro-wallet -- --check` independently. The final clippy check after Task 3 is also workspace-wide clean.
- **Committed in:** Spread across `d3f9fced` (struct + dead-code annotations), `5d2a9eee` (Task 2 inherits unchanged), `c5fb975e` (impl block + drop dead-code annotations).

---

**Total deviations:** 1 auto-fixed (Rule 3 — blocking issue)
**Impact on plan:** Necessary for per-task atomic commits to pass `cargo build`. No semantic change to the plan's public surface — `GoogleWalletBuilder::{new, save_jwt, save_url}` still lands in Task 3 exactly as specified; only the struct's field declarations move one commit earlier. Zero scope creep.

## Issues Encountered

- `rustfmt` collapsed `GoogleWalletBuilder::new`'s multi-line signature onto a single line. Detected by `cargo fmt -p ferro-wallet -- --check` after Task 3's initial Write; resolved by running `cargo fmt -p ferro-wallet` and re-verifying. No code-content change.

## Threat Flags

No new threat surface introduced beyond what the plan's `<threat_model>` already documented (T-151-Google-JWT, T-151-DEFAULT-CRED). Both mitigations are present:
- `GoogleWalletBuilder` does NOT derive `Debug` — verified by reading the struct definition; no PEM disclosure via formatting.
- RS256 asymmetric signature with `origins: [app_url]` claim binds the JWT to the configured `APP_URL` per D-08.

## User Setup Required

None - no external service configuration required. PLAN-08's integration test will mint an RSA keypair at runtime (D-09) so CI has no dependency on real Google service-account secrets.

## Next Phase Readiness

- **PLAN-08 (`tests/google_jwt.rs`)** can now write against `GoogleWalletBuilder::new(cfg, app_name, app_url)` and `builder.save_jwt(&subject)`. The integration test will mint an RSA keypair via `openssl::rsa::Rsa::generate(2048)`, populate `GoogleConfig` with the PEM, sign a save JWT, decode with the public key (with `Validation.validate_exp = false` + `required_spec_claims = HashSet::new()` per RESEARCH.md Pitfall 3), and assert the claim shape + `payload.eventTicketObjects[0].id` + `barcode.value`.
- **PLAN-09 (release)** can now patch-bump `[workspace.package] version` from `0.2.23` once PLAN-08 lands green.

## Self-Check: PASSED

- `ferro-wallet/src/google/jwt.rs` exists, contains `Algorithm::RS256`, `pub const fn pass_type_id_default`, `pub fn save_url` ✓
- `ferro-wallet/src/google/object.rs` exists, contains `pub(crate) fn build_event_ticket_object`, `state`, `qrCode`, `pass_type_id_default` ✓
- `ferro-wallet/src/google/mod.rs` exists, contains `pub struct GoogleWalletBuilder`, `pub fn save_jwt`, `pub fn save_url` ✓
- `ferro-wallet/src/lib.rs` contains `pub use google::GoogleWalletBuilder;` ✓
- Commits exist: `d3f9fced`, `5d2a9eee`, `c5fb975e` ✓
- `cargo test -p ferro-wallet --lib`: 38/38 pass ✓
- `cargo build --workspace`: clean ✓
- `cargo clippy --all --all-targets -- -D warnings`: clean ✓
- `cargo fmt --all -- --check`: clean ✓

---
*Phase: 151-ferro-wallet-crate*
*Completed: 2026-05-11*
