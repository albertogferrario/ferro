---
phase: 151
plan: 151-08
slug: google-jwt-test
wave: 4
depends_on: [151-07]
files_modified:
  - ferro-wallet/tests/google_jwt.rs
autonomous: true
requirements: [ACC-1k]
must_haves:
  truths:
    - "Test mints an RSA keypair at runtime — no real Google service-account credentials in CI"
    - "Test signs a save JWT, decodes with the public key, asserts claims (`iss/aud=google/typ=savetowallet`)"
    - "Test asserts `payload.eventTicketObjects` contains exactly one entry with the expected `id` and `barcode.value`"
    - "Test asserts `save_url(...)` returns the `https://pay.google.com/gp/v/save/` prefix"
    - "Validation construction disables `exp` requirement (RESEARCH.md Pitfall 3)"
  artifacts:
    - path: "ferro-wallet/tests/google_jwt.rs"
      provides: "End-to-end RS256 JWT roundtrip integration test (ACC-1k)"
      contains: "fn save_jwt_roundtrips_with_runtime_minted_rsa_keypair"
      min_lines: 80
  key_links:
    - from: "tests/google_jwt.rs"
      to: "GoogleWalletBuilder + WalletSubject"
      via: "import from ferro_wallet"
      pattern: "use ferro_wallet::"
    - from: "Validation"
      to: "Algorithm::RS256 + validate_exp=false + required_spec_claims=empty"
      via: "Pitfall 3 mitigation"
      pattern: "validate_exp = false"
---

<objective>
Land the end-to-end Google Wallet integration test. Mints an RSA keypair at runtime, signs a save JWT, decodes with the matching public key, asserts claim shape per ACC-1k.
</objective>

<context>
@.planning/phases/151-ferro-wallet-crate/151-CONTEXT.md
@.planning/phases/151-ferro-wallet-crate/151-PATTERNS.md
@.planning/phases/151-ferro-wallet-crate/151-RESEARCH.md
@.planning/phases/151-ferro-wallet-crate/151-VALIDATION.md
@docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md
@ferro-wallet/src/lib.rs
@ferro-wallet/src/google/mod.rs
@ferro-wallet/src/google/jwt.rs
@ferro-wallet/src/google/object.rs

<interfaces>
Test exercises:
```rust
use ferro_wallet::{
    GoogleConfig, GoogleWalletBuilder, Branding, Field, FieldAlignment, GeoPoint, PassKind,
    RgbColor, TextColorMode, WalletSubject,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
```

RESEARCH.md Pitfall 3: `Validation::default()` requires `exp`. The save JWT has no `exp`. The test MUST configure:
```rust
let mut validation = Validation::new(Algorithm::RS256);
validation.validate_exp = false;
validation.required_spec_claims = std::collections::HashSet::new();
validation.set_audience(&["google"]);
```

Reference cert minting code: 151-RESEARCH.md §"Code Examples" → "Self-signed X.509 + RSA keypair at test runtime" (only the RSA portion is needed — no X509 here).
</interfaces>
</context>

<must_haves>
- File: `ferro-wallet/tests/google_jwt.rs`.
- Helper `fn mint_rsa_keypair() -> (String /* private_pem */, String /* public_pem */)`.
- `struct StubBooking` implements `WalletSubject` with known field values (mirror PLAN-06's stub for consistency).
- `#[test] fn save_jwt_roundtrips_with_runtime_minted_rsa_keypair` (ACC-1k):
  - Mints keypair.
  - Constructs `GoogleConfig { issuer_id: "3388000000000000000", service_account_email: "sa@example.iam.gserviceaccount.com", service_account_private_key_pem: private_pem }`.
  - Builds JWT via `GoogleWalletBuilder::new(...).save_jwt(&StubBooking)`.
  - Decodes JWT with `decode::<serde_json::Value>(&jwt, &DecodingKey::from_rsa_pem(public_pem.as_bytes()).unwrap(), &validation).unwrap()`.
  - Asserts `decoded.claims["iss"] == "sa@example.iam.gserviceaccount.com"`.
  - Asserts `decoded.claims["aud"] == "google"`.
  - Asserts `decoded.claims["typ"] == "savetowallet"`.
  - Asserts `decoded.claims["origins"][0] == "https://example.com"`.
  - Asserts `decoded.claims["payload"]["eventTicketObjects"]` is an array of length 1.
  - Asserts `[0]["id"] == "3388000000000000000.BOOK-1"`.
  - Asserts `[0]["classId"] == "3388000000000000000.booking"`.
  - Asserts `[0]["state"] == "active"`.
  - Asserts `[0]["barcode"]["type"] == "qrCode"`.
  - Asserts `[0]["barcode"]["value"] == "ticket-token-abc"`.
- `#[test] fn save_url_returns_pay_google_com_prefix`: calls `builder.save_url(&StubBooking).unwrap()`; asserts `.starts_with("https://pay.google.com/gp/v/save/")`.
</must_haves>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Author tests/google_jwt.rs end-to-end (ACC-1k)</name>
  <files>ferro-wallet/tests/google_jwt.rs</files>
  <read_first>
    - 151-RESEARCH.md §"Code Examples" → "jsonwebtoken RS256 encode" (encode side, already exercised by PLAN-07)
    - 151-RESEARCH.md §"Code Examples" → "Self-signed X.509 + RSA keypair at test runtime" (RSA mint portion)
    - 151-RESEARCH.md §"Common Pitfalls" Pitfall 3 (Validation::default() requires `exp`)
    - 151-PATTERNS.md §"ferro-wallet/tests/google_jwt.rs"
    - 151-CONTEXT.md D-08 (claim shape), D-09 (runtime-mint pattern)
    - 151-VALIDATION.md ACC-1k row (test command + claim assertions)
    - ferro-wallet/src/lib.rs (verify `GoogleWalletBuilder`, `GoogleConfig`, value types re-exported)
  </read_first>
  <behavior>
    `cargo test -p ferro-wallet --test google_jwt` exits 0.

    Specifically:
    - The decoded JWT claims dict contains exactly the fields documented in must_haves above with the documented values.
    - `save_url(...)` returns a string starting with `https://pay.google.com/gp/v/save/`.
  </behavior>
  <action>
    Create `ferro-wallet/tests/google_jwt.rs`:

    ```rust
    //! End-to-end Google Wallet save-link JWT integration test.
    //!
    //! Mints an RSA keypair at runtime (D-09), signs a save JWT for a StubBooking subject,
    //! and decodes the JWT with the matching public key. Verifies the claim shape per D-08.

    use std::collections::HashSet;

    use ferro_wallet::{
        Branding, Field, FieldAlignment, GeoPoint, GoogleConfig, GoogleWalletBuilder, PassKind,
        RgbColor, TextColorMode, WalletSubject,
    };
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
    use openssl::pkey::PKey;
    use openssl::rsa::Rsa;

    fn mint_rsa_keypair() -> (String, String) {
        let rsa = Rsa::generate(2048).unwrap();
        let pkey = PKey::from_rsa(rsa).unwrap();
        let private_pem = String::from_utf8(pkey.private_key_to_pem_pkcs8().unwrap()).unwrap();
        let public_pem = String::from_utf8(pkey.public_key_to_pem().unwrap()).unwrap();
        (private_pem, public_pem)
    }

    struct StubBooking;

    impl WalletSubject for StubBooking {
        fn pass_kind(&self) -> PassKind { PassKind::EventTicket }
        fn serial(&self) -> String { "BOOK-1".to_string() }
        fn primary(&self) -> Field {
            Field {
                key: "event".to_string(),
                label: "Event".to_string(),
                value: "Test Event".to_string(),
                alignment: FieldAlignment::Left,
            }
        }
        fn secondary(&self) -> Vec<Field> { vec![] }
        fn auxiliary(&self) -> Vec<Field> { vec![] }
        fn back(&self) -> Vec<Field> { vec![] }
        fn barcode_token(&self) -> String { "ticket-token-abc".to_string() }
        fn relevant_at(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
        fn expires_at(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
        fn locations(&self) -> Vec<GeoPoint> { vec![] }
        fn branding(&self) -> Branding {
            Branding {
                organization_name: Some("Test Org".to_string()),
                logo_text: None,
                background_color: RgbColor { r: 0, g: 0, b: 0 },
                text_color_mode: TextColorMode::Auto,
                logo_png_bytes: vec![], // not used by Google save JWT
                icon_png_bytes: None,
                hero_png_bytes: None,
            }
        }
    }

    fn build_test_builder() -> GoogleWalletBuilder {
        let (private_pem, _) = mint_rsa_keypair();
        let cfg = GoogleConfig {
            issuer_id: "3388000000000000000".to_string(),
            service_account_email: "sa@example.iam.gserviceaccount.com".to_string(),
            service_account_private_key_pem: private_pem,
        };
        GoogleWalletBuilder::new(cfg, "Test App".to_string(), "https://example.com".to_string())
            .unwrap()
    }

    #[test]
    fn save_jwt_roundtrips_with_runtime_minted_rsa_keypair() {
        let (private_pem, public_pem) = mint_rsa_keypair();
        let cfg = GoogleConfig {
            issuer_id: "3388000000000000000".to_string(),
            service_account_email: "sa@example.iam.gserviceaccount.com".to_string(),
            service_account_private_key_pem: private_pem,
        };
        let builder = GoogleWalletBuilder::new(
            cfg,
            "Test App".to_string(),
            "https://example.com".to_string(),
        )
        .unwrap();

        let jwt = builder.save_jwt(&StubBooking).unwrap();

        // Pitfall 3: save JWT has no `exp` claim. Disable exp validation and clear required claims.
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = false;
        validation.required_spec_claims = HashSet::new();
        validation.set_audience(&["google"]);

        let decoded = decode::<serde_json::Value>(
            &jwt,
            &DecodingKey::from_rsa_pem(public_pem.as_bytes()).unwrap(),
            &validation,
        )
        .unwrap();

        let c = decoded.claims;
        assert_eq!(c["iss"], "sa@example.iam.gserviceaccount.com");
        assert_eq!(c["aud"], "google");
        assert_eq!(c["typ"], "savetowallet");
        assert_eq!(c["origins"][0], "https://example.com");

        let objects = c["payload"]["eventTicketObjects"].as_array().expect("array");
        assert_eq!(objects.len(), 1, "expected exactly one eventTicketObject");
        let obj = &objects[0];
        assert_eq!(obj["id"], "3388000000000000000.BOOK-1");
        assert_eq!(obj["classId"], "3388000000000000000.booking");
        assert_eq!(obj["state"], "active");
        assert_eq!(obj["barcode"]["type"], "qrCode");
        assert_eq!(obj["barcode"]["value"], "ticket-token-abc");
    }

    #[test]
    fn save_url_returns_pay_google_com_prefix() {
        let builder = build_test_builder();
        let url = builder.save_url(&StubBooking).unwrap();
        assert!(
            url.starts_with("https://pay.google.com/gp/v/save/"),
            "save_url should start with https://pay.google.com/gp/v/save/, got {url}"
        );
    }
    ```

    Note: `jsonwebtoken`, `openssl`, `chrono`, `serde_json` are already `ferro-wallet` dependencies (declared in PLAN-01). No `[dev-dependencies]` additions needed.
  </action>
  <verify>
    <automated>cargo test -p ferro-wallet --test google_jwt &amp;&amp; cargo clippy -p ferro-wallet --all-targets -- -D warnings &amp;&amp; cargo fmt -p ferro-wallet -- --check &amp;&amp; test -f ferro-wallet/tests/google_jwt.rs &amp;&amp; grep -F 'mint_rsa_keypair' ferro-wallet/tests/google_jwt.rs &amp;&amp; grep -F 'fn save_jwt_roundtrips_with_runtime_minted_rsa_keypair' ferro-wallet/tests/google_jwt.rs &amp;&amp; grep -F 'fn save_url_returns_pay_google_com_prefix' ferro-wallet/tests/google_jwt.rs &amp;&amp; grep -F 'validate_exp = false' ferro-wallet/tests/google_jwt.rs &amp;&amp; grep -F 'required_spec_claims = HashSet::new()' ferro-wallet/tests/google_jwt.rs</automated>
  </verify>
  <done>Integration test exists. `cargo test -p ferro-wallet --test google_jwt` passes. Both `#[test]` functions pass. All 10 claim assertions hold. `Validation` construction matches Pitfall 3 mitigation.</done>
</task>

</tasks>

<threat_model>
This plan introduces test-only code. RSA key minted at runtime; no real Google credentials.

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-151-Google-JWT | T | `tests/google_jwt.rs` JWT decode | mitigate | The decode side uses the matching public key from the runtime-minted keypair, asserting signature validity end-to-end. Validation config disables `exp` per Pitfall 3 — this is correct per the spec (save JWTs have no `exp`); the production `aud` check (`"google"`) is exercised. |
| T-151-DEFAULT-CRED | I | `mint_rsa_keypair()` | mitigate | No PEM material is committed. Keys are generated at test runtime via `Rsa::generate(2048)` and discarded when the test ends. |
</threat_model>

<verification>
- `cargo test -p ferro-wallet --test google_jwt` exits 0 (ACC-1k).
- `cargo test -p ferro-wallet` (full crate suite, incl. apple_integration + google_jwt) exits 0.
- `cargo clippy -p ferro-wallet --all-targets -- -D warnings` exits 0.
- `cargo fmt -p ferro-wallet -- --check` exits 0.
- `grep -F 'validate_exp = false' ferro-wallet/tests/google_jwt.rs` returns one match (Pitfall 3 mitigation confirmed).
</verification>

<success_criteria>
ACC-1k passes. The Google builder is end-to-end validated: subject → eventTicketObject → RS256 JWT → decode-with-public-key roundtrip → claim shape assertions. All 11 unit tests + 2 integration tests are now green; phase ready for release (PLAN-09).
</success_criteria>

<output>
After completion, create `.planning/phases/151-ferro-wallet-crate/151-08-SUMMARY.md` documenting the runtime RSA mint pattern, the validation construction (Pitfall 3 mitigation), and the 10 claim assertions checked.
</output>

## PLANNING COMPLETE
