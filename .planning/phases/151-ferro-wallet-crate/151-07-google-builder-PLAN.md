---
phase: 151
plan: 151-07
slug: google-builder
wave: 3
depends_on: [151-02, 151-03]
files_modified:
  - ferro-wallet/src/google/object.rs
  - ferro-wallet/src/google/jwt.rs
  - ferro-wallet/src/google/mod.rs
  - ferro-wallet/src/lib.rs
autonomous: true
requirements: [ACC-1i]
must_haves:
  truths:
    - "`GoogleWalletBuilder::new(cfg, app_name, app_url)` constructs the builder from `GoogleConfig`"
    - "`save_jwt(subject)` returns an RS256-signed JWT with claims `iss/aud=google/typ=savetowallet/iat/origins/payload.eventTicketObjects[…]`"
    - "`save_url(subject)` returns `https://pay.google.com/gp/v/save/{jwt}`"
    - "`build_event_ticket_object` produces a JSON object with `id`, `classId`, `state`, `barcode` matching spec §3 + D-07"
    - "`pass_type_id_default() == \"booking\"` (D-07)"
  artifacts:
    - path: "ferro-wallet/src/google/object.rs"
      provides: "build_event_ticket_object"
      contains: "pub(crate) fn build_event_ticket_object"
    - path: "ferro-wallet/src/google/jwt.rs"
      provides: "sign_save_jwt + save_url + pass_type_id_default"
      contains: "Algorithm::RS256"
    - path: "ferro-wallet/src/google/mod.rs"
      provides: "GoogleWalletBuilder { new, save_jwt, save_url }"
      contains: "pub struct GoogleWalletBuilder"
    - path: "ferro-wallet/src/lib.rs"
      provides: "Restored `pub use google::GoogleWalletBuilder;`"
      contains: "pub use google::GoogleWalletBuilder;"
  key_links:
    - from: "GoogleWalletBuilder::save_jwt"
      to: "object::build_event_ticket_object + jwt::sign_save_jwt"
      via: "linear pipeline"
      pattern: "fn save_jwt"
    - from: "jwt::sign_save_jwt"
      to: "jsonwebtoken::encode(RS256)"
      via: "EncodingKey::from_rsa_pem"
      pattern: "Algorithm::RS256"
---

<objective>
Land the Google Wallet builder. Three sub-files (`object.rs`, `jwt.rs`, `mod.rs`) form a tight pipeline (subject → JSON object → RS256 JWT → URL). Restore `lib.rs` re-export.
</objective>

<context>
@.planning/phases/151-ferro-wallet-crate/151-CONTEXT.md
@.planning/phases/151-ferro-wallet-crate/151-PATTERNS.md
@.planning/phases/151-ferro-wallet-crate/151-RESEARCH.md
@.planning/phases/151-ferro-wallet-crate/151-VALIDATION.md
@docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md
@ferro-wallet/src/subject.rs
@ferro-wallet/src/config.rs
@ferro-wallet/src/error.rs
@ferro-wallet/src/lib.rs

<interfaces>
Public API per spec §3.3:

```rust
pub struct GoogleWalletBuilder { /* issuer_id + sa_email + private_key_pem + app_name + app_url */ }

impl GoogleWalletBuilder {
    pub fn new(cfg: GoogleConfig, app_name: String, app_url: String) -> Result<Self, WalletError>;
    pub fn save_jwt<S: WalletSubject>(&self, s: &S) -> Result<String, WalletError>;
    pub fn save_url<S: WalletSubject>(&self, s: &S) -> Result<String, WalletError>;
}
```

Internal contracts:

```rust
// google/object.rs
pub(crate) fn build_event_ticket_object<S: WalletSubject>(builder: &GoogleWalletBuilder, subject: &S) -> Result<serde_json::Value, WalletError>;

// google/jwt.rs
pub const fn pass_type_id_default() -> &'static str { "booking" }
pub(crate) fn sign_save_jwt(builder: &GoogleWalletBuilder, event_ticket_object: serde_json::Value) -> Result<String, WalletError>;
pub fn save_url(jwt: &str) -> String;
```

JWT claim shape per D-08:

```json
{
  "iss": "<service_account_email>",
  "aud": "google",
  "typ": "savetowallet",
  "iat": <unix>,
  "origins": ["<app_url>"],
  "payload": { "eventTicketObjects": [ <one object> ] }
}
```

EventTicketObject shape per spec §3 + D-07:

```json
{
  "id":      "<issuer_id>.<subject.serial()>",
  "classId": "<issuer_id>.booking",
  "state":   "active",
  "barcode": { "type": "qrCode", "value": "<subject.barcode_token()>" },
  "ticketHolderName": "<subject.primary().value>",
  "eventName": { "defaultValue": { "language": "en", "value": "<subject.primary().value>" } }
}
```

Note on `classId` per D-07: `class_id = "{issuer_id}.{pass_type_id_with_dots_replaced_by_underscores}"` and the v1 fixed `pass_type_id_default() = "booking"`. "booking" has no dots, so `class_id = "{issuer_id}.booking"`. The dot-to-underscore rule is no-op in v1 but kept future-proof.

Reference code: 151-RESEARCH.md §"Code Examples" → "jsonwebtoken RS256 encode".
</interfaces>
</context>

<must_haves>
- Three google/ files populated; `// placeholder` lines removed.
- `pass_type_id_default()` is a `const fn` returning `"booking"`.
- `save_url(jwt)` returns `format!("https://pay.google.com/gp/v/save/{jwt}")`.
- `sign_save_jwt` uses `Header::new(Algorithm::RS256)` + `EncodingKey::from_rsa_pem`.
- `build_event_ticket_object` produces the JSON structure documented above with the correct `id` / `classId` format from D-07.
- ACC-1i test (`google::jwt::tests::save_url_format`) exists and passes.
- `lib.rs` restores `pub use google::GoogleWalletBuilder;`.
</must_haves>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Implement google/jwt.rs (sign_save_jwt + save_url + pass_type_id_default) + ACC-1i test</name>
  <files>ferro-wallet/src/google/jwt.rs</files>
  <read_first>
    - docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md §3
    - 151-RESEARCH.md §"Code Examples" → "jsonwebtoken RS256 encode" (full body)
    - 151-PATTERNS.md §"ferro-wallet/src/google/jwt.rs"
    - 151-CONTEXT.md D-07 + D-08
    - 151-VALIDATION.md ACC-1i row (test name `save_url_format`)
  </read_first>
  <behavior>
    - `pass_type_id_default() == "booking"`.
    - `save_url("abc.def.ghi") == "https://pay.google.com/gp/v/save/abc.def.ghi"` (ACC-1i).
    - `save_url("") == "https://pay.google.com/gp/v/save/"`.
    - `sign_save_jwt(&builder, value)` with a valid RSA PEM returns a 3-segment dotted JWT.
    - `sign_save_jwt` with a malformed PEM returns `Err(WalletError::GoogleJwt(_))`.
  </behavior>
  <action>
    Replace the placeholder. Implement `pass_type_id_default()`, the `SaveClaims<'a>` serializer struct, `sign_save_jwt(builder, event_ticket_object)` using `jsonwebtoken::encode(&Header::new(Algorithm::RS256), &claims, &EncodingKey::from_rsa_pem(builder.private_key_pem.as_bytes())?)`, and `save_url(jwt) = format!("https://pay.google.com/gp/v/save/{jwt}")`. Map all jsonwebtoken errors to `WalletError::GoogleJwt(format!("…: {e}"))`. Reference the full body in 151-RESEARCH.md §"Code Examples" → "jsonwebtoken RS256 encode".

    Append `#[cfg(test)] mod tests` with three tests:
    - `save_url_format` (ACC-1i) — asserts `save_url("abc.def.ghi") == "https://pay.google.com/gp/v/save/abc.def.ghi"`.
    - `save_url_empty_jwt` — asserts `save_url("") == "https://pay.google.com/gp/v/save/"`.
    - `pass_type_id_default_is_booking` — asserts `pass_type_id_default() == "booking"`.
  </action>
  <verify>
    <automated>cargo build -p ferro-wallet &amp;&amp; cargo test -p ferro-wallet --lib google::jwt::tests::save_url_format &amp;&amp; cargo test -p ferro-wallet --lib google::jwt::tests &amp;&amp; cargo clippy -p ferro-wallet --all-targets -- -D warnings &amp;&amp; cargo fmt -p ferro-wallet -- --check &amp;&amp; grep -F 'Algorithm::RS256' ferro-wallet/src/google/jwt.rs &amp;&amp; grep -F 'pub const fn pass_type_id_default' ferro-wallet/src/google/jwt.rs &amp;&amp; grep -F 'pub fn save_url' ferro-wallet/src/google/jwt.rs</automated>
  </verify>
  <done>`sign_save_jwt`, `save_url`, `pass_type_id_default` land. ACC-1i test passes. Three unit tests pass.</done>
</task>

<task type="auto">
  <name>Task 2: Implement google/object.rs (build_event_ticket_object)</name>
  <files>ferro-wallet/src/google/object.rs</files>
  <read_first>
    - docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md §3 (eventTicketObject shape)
    - 151-PATTERNS.md §"ferro-wallet/src/google/object.rs"
    - 151-CONTEXT.md D-07 (class_id / object_id format)
    - ferro-wallet/src/google/jwt.rs (Task 1 output — for `pass_type_id_default()`)
  </read_first>
  <action>
    Replace the placeholder. Implement `build_event_ticket_object<S: WalletSubject>(builder, subject)` returning a `serde_json::Value`. Build:

    - `class_suffix = pass_type_id_default().replace('.', "_")` (no-op in v1, future-proofs the dot-substitution rule from D-07).
    - `class_id = format!("{}.{}", builder.issuer_id, class_suffix)`.
    - `object_id = format!("{}.{}", builder.issuer_id, subject.serial())`.

    Compose the JSON via `serde_json::json!({...})`:
    - `"id": object_id`
    - `"classId": class_id`
    - `"state": "active"`
    - `"barcode": { "type": "qrCode", "value": subject.barcode_token() }`
    - `"ticketHolderName": primary.value` (where `primary = subject.primary()`)
    - `"eventName": { "defaultValue": { "language": "en", "value": primary.value } }`

    No unit tests in this file — the full pipeline is exercised in `tests/google_jwt.rs` (PLAN-08).
  </action>
  <verify>
    <automated>cargo build -p ferro-wallet &amp;&amp; cargo clippy -p ferro-wallet --all-targets -- -D warnings &amp;&amp; cargo fmt -p ferro-wallet -- --check &amp;&amp; grep -F 'pub(crate) fn build_event_ticket_object' ferro-wallet/src/google/object.rs &amp;&amp; grep -F 'state' ferro-wallet/src/google/object.rs &amp;&amp; grep -F 'qrCode' ferro-wallet/src/google/object.rs &amp;&amp; grep -F 'pass_type_id_default' ferro-wallet/src/google/object.rs</automated>
  </verify>
  <done>`build_event_ticket_object` lands. Produces correct shape per spec + D-07. Build + clippy clean.</done>
</task>

<task type="auto">
  <name>Task 3: Implement google/mod.rs (GoogleWalletBuilder facade) + restore lib.rs re-export</name>
  <files>ferro-wallet/src/google/mod.rs, ferro-wallet/src/lib.rs</files>
  <read_first>
    - 151-PATTERNS.md §"ferro-wallet/src/google/mod.rs (GoogleWalletBuilder)"
    - ferro-wallet/src/google/jwt.rs (Task 1)
    - ferro-wallet/src/google/object.rs (Task 2)
    - ferro-wallet/src/lib.rs (commented-out `pub use google::GoogleWalletBuilder;`)
    - 151-CONTEXT.md D-11
  </read_first>
  <action>
    Replace `ferro-wallet/src/google/mod.rs` body (currently has placeholder + `pub mod` lines):

    ```rust
    //! Google Wallet save-link issuance — RS256 JWT pointing at an eventTicketObject.

    pub mod jwt;
    pub mod object;

    use crate::config::GoogleConfig;
    use crate::subject::WalletSubject;
    use crate::WalletError;

    pub struct GoogleWalletBuilder {
        pub(crate) issuer_id: String,
        pub(crate) service_account_email: String,
        pub(crate) private_key_pem: String,
        pub(crate) app_name: String,
        pub(crate) app_url: String,
    }

    impl GoogleWalletBuilder {
        pub fn new(
            cfg: GoogleConfig,
            app_name: String,
            app_url: String,
        ) -> Result<Self, WalletError> {
            Ok(Self {
                issuer_id: cfg.issuer_id,
                service_account_email: cfg.service_account_email,
                private_key_pem: cfg.service_account_private_key_pem,
                app_name,
                app_url,
            })
        }

        pub fn save_jwt<S: WalletSubject>(&self, s: &S) -> Result<String, WalletError> {
            let obj = object::build_event_ticket_object(self, s)?;
            jwt::sign_save_jwt(self, obj)
        }

        pub fn save_url<S: WalletSubject>(&self, s: &S) -> Result<String, WalletError> {
            Ok(jwt::save_url(&self.save_jwt(s)?))
        }
    }
    ```

    The `app_name` field is currently unread; mark `#[allow(dead_code)]` on the struct field OR plumb it into the JWT payload's metadata (spec does not require it in v1, so keep it as a stored field for symmetry with the Apple builder). Choose `#[allow(dead_code)]` on the field to keep the surface minimal — note this matches D-08 (JWT payload does not need `app_name`).

    Then edit `ferro-wallet/src/lib.rs` and uncomment `pub use google::GoogleWalletBuilder;`.
  </action>
  <verify>
    <automated>cargo build -p ferro-wallet &amp;&amp; cargo test -p ferro-wallet --lib &amp;&amp; cargo clippy -p ferro-wallet --all-targets -- -D warnings &amp;&amp; cargo fmt -p ferro-wallet -- --check &amp;&amp; grep -F 'pub struct GoogleWalletBuilder' ferro-wallet/src/google/mod.rs &amp;&amp; grep -F 'pub fn save_jwt' ferro-wallet/src/google/mod.rs &amp;&amp; grep -F 'pub fn save_url' ferro-wallet/src/google/mod.rs &amp;&amp; grep -F 'pub use google::GoogleWalletBuilder;' ferro-wallet/src/lib.rs &amp;&amp; cargo build --workspace</automated>
  </verify>
  <done>`GoogleWalletBuilder::new`, `save_jwt`, `save_url` exist with documented contract. `lib.rs` re-export restored. Full workspace builds. Existing unit tests still pass; integration test lands in PLAN-08.</done>
</task>

</tasks>

<threat_model>
This plan introduces the Google Wallet save-link JWT signing path. RS256 with a private key.

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-151-Google-JWT | S | `google/jwt.rs::sign_save_jwt` | mitigate | RS256 asymmetric signature — only the holder of the service-account private key can mint a valid save-link JWT. `origins: [app_url]` claim binds the JWT to the configured `APP_URL`; Google's save endpoint validates the origin. No `exp` claim by design — save-links are user-initiated and validated at click time by Google, not by our code (PLAN-08 test sets `Validation.validate_exp = false` to match). |
| T-151-DEFAULT-CRED | I | `GoogleWalletBuilder.private_key_pem: String` | accept | Private key PEM lives in the builder's owned `String`. The builder does NOT derive `Debug` — verified by grep. Same disclosure surface as `AppleConfig` (acceptable per workspace convention). |
</threat_model>

<verification>
- `cargo test -p ferro-wallet --lib google::jwt::tests::save_url_format` exits 0 (ACC-1i).
- `cargo test -p ferro-wallet --lib google::jwt::tests` runs 3 tests, all pass.
- `cargo test -p ferro-wallet --lib` runs every unit test from PLAN-01..05, 07; all pass.
- `cargo build --workspace` exits 0.
- `cargo clippy --all --all-targets -- -D warnings` exits 0.
- `cargo fmt --all -- --check` exits 0.
- `grep -F 'pub use google::GoogleWalletBuilder;' ferro-wallet/src/lib.rs` returns one match.
- `grep -F 'Algorithm::RS256' ferro-wallet/src/google/jwt.rs` returns one match.
</verification>

<success_criteria>
PLAN-08 can write `tests/google_jwt.rs` against `GoogleWalletBuilder::new` and `::save_jwt`. The integration test mints an RSA keypair, signs a save JWT, decodes with the public key, and asserts the claim shape.
</success_criteria>

<output>
After completion, create `.planning/phases/151-ferro-wallet-crate/151-07-SUMMARY.md` documenting the JWT claim shape, the eventTicketObject shape, the `classId` / `id` derivation rule, and confirmation that ACC-1i passed.
</output>

## PLANNING COMPLETE
