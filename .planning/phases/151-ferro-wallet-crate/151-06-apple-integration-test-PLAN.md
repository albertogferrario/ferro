---
phase: 151
plan: 151-06
slug: apple-integration-test
wave: 4
depends_on: [151-05]
files_modified:
  - ferro-wallet/tests/apple_integration.rs
autonomous: true
requirements: [ACC-1j]
must_haves:
  truths:
    - "Test mints a self-signed X.509 + RSA keypair at runtime — no real Apple credentials in CI"
    - "Test builds a `.pkpass` from a `StubBooking` `WalletSubject` and parses the resulting ZIP"
    - "Test asserts the ZIP contains exactly 9 entries (pass.json, manifest.json, signature, logo×3, icon×3)"
    - "Test parses `pass.json` and asserts `passTypeIdentifier`, `teamIdentifier`, `serialNumber`, `barcode.message`, `eventTicket.primaryFields[0].value`"
    - "Test includes a comment that the self-signed cert is structure-only, NOT proof of Apple-validity"
  artifacts:
    - path: "ferro-wallet/tests/apple_integration.rs"
      provides: "End-to-end Apple integration test (ACC-1j)"
      contains: "fn build_pkpass_produces_valid_zip_and_pass_json"
      min_lines: 100
  key_links:
    - from: "tests/apple_integration.rs"
      to: "ApplePassBuilder + WalletSubject + RgbColor + Field + Branding"
      via: "import from ferro_wallet"
      pattern: "use ferro_wallet::"
---

<objective>
Land the end-to-end Apple integration test that exercises `ApplePassBuilder::build` against a runtime-minted self-signed certificate. Asserts ZIP shape (9 entries) and key `pass.json` fields per ACC-1j.
</objective>

<context>
@.planning/phases/151-ferro-wallet-crate/151-CONTEXT.md
@.planning/phases/151-ferro-wallet-crate/151-PATTERNS.md
@.planning/phases/151-ferro-wallet-crate/151-RESEARCH.md
@.planning/phases/151-ferro-wallet-crate/151-VALIDATION.md
@docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md
@ferro-wallet/src/lib.rs
@ferro-wallet/src/subject.rs
@ferro-wallet/src/config.rs
@ferro-wallet/src/apple/mod.rs
@ferro-stripe/tests/parser_contract.rs

<interfaces>
Test exercises the public API surface from PLAN-02 + PLAN-03 + PLAN-05:
```rust
use ferro_wallet::{
    ApplePassBuilder, AppleConfig, Branding, Field, FieldAlignment, GeoPoint, PassKind,
    RgbColor, TextColorMode, WalletSubject,
};
```

Reference code for runtime cert minting: 151-RESEARCH.md §"Code Examples" → "Self-signed X.509 + RSA keypair at test runtime".

Reference ZIP entry list per ACC-1j (in order produced by `ApplePassBuilder::build`):
1. `pass.json`
2. `manifest.json`
3. `signature`
4. `logo.png`
5. `logo@2x.png`
6. `logo@3x.png`
7. `icon.png`
8. `icon@2x.png`
9. `icon@3x.png`

The stub subject needs branding image bytes — produce a tiny in-test PNG (e.g., a 64×64 RgbaImage encoded to PNG via the `image` crate, which is already a dep).
</interfaces>
</context>

<must_haves>
- File: `ferro-wallet/tests/apple_integration.rs`.
- Helper `fn mint_self_signed() -> (String /*cert_pem*/, String /*key_pem*/)` mints a self-signed cert via openssl.
- Helper `fn tiny_png(w: u32, h: u32, r: u8, g: u8, b: u8) -> Vec<u8>` produces a solid-colour PNG via the `image` crate.
- `struct StubBooking` implements `WalletSubject` with known field values.
- `#[test] fn build_pkpass_produces_valid_zip_and_pass_json` (ACC-1j):
  - Mints cert; constructs `AppleConfig` reusing the cert as both `cert_pem` and `wwdr_pem` (D-09).
  - Builds the `.pkpass` bytes via `ApplePassBuilder::build`.
  - Re-parses bytes with `zip::ZipArchive`.
  - Asserts exactly 9 entries with the expected names.
  - Extracts `pass.json` bytes and parses as `serde_json::Value`.
  - Asserts: `passTypeIdentifier`, `teamIdentifier`, `serialNumber`, `barcodes[0].message`, `eventTicket.primaryFields[0].value`.
- File-level comment explicitly states the self-signed cert verifies structure, NOT Apple-validity (RESEARCH.md Risk 3).
</must_haves>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Author tests/apple_integration.rs end-to-end (ACC-1j)</name>
  <files>ferro-wallet/tests/apple_integration.rs</files>
  <read_first>
    - 151-RESEARCH.md §"Code Examples" → "Self-signed X.509 + RSA keypair at test runtime" (full mint_self_signed body)
    - 151-PATTERNS.md §"ferro-wallet/tests/apple_integration.rs" (test layout reference)
    - 151-CONTEXT.md D-09 (self-signed-as-both-signer-and-WWDR pattern)
    - 151-RESEARCH.md §"Risks & Open Questions" item 3 (mandatory comment about structure-vs-Apple-validity)
    - docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md §7 (integration test description)
    - ferro-stripe/tests/parser_contract.rs lines 1–17 (integration test layout style)
    - ferro-wallet/src/lib.rs (public re-exports — confirm trait + value types are reachable from `ferro_wallet::`)
    - 151-VALIDATION.md ACC-1j row (cargo test command)
  </read_first>
  <behavior>
    Running `cargo test -p ferro-wallet --test apple_integration` exits 0.

    Specifically the test asserts:
    - `zip::ZipArchive::new(Cursor::new(pkpass_bytes)).unwrap().len() == 9`
    - The 9 entry names in any order are exactly: `pass.json`, `manifest.json`, `signature`, `logo.png`, `logo@2x.png`, `logo@3x.png`, `icon.png`, `icon@2x.png`, `icon@3x.png`.
    - `pass_json["passTypeIdentifier"] == "pass.com.example.test"`.
    - `pass_json["teamIdentifier"] == "TEAMID1234"`.
    - `pass_json["serialNumber"] == "BOOK-1"`.
    - `pass_json["barcodes"][0]["message"] == "ticket-token-abc"`.
    - `pass_json["barcodes"][0]["format"] == "PKBarcodeFormatQR"`.
    - `pass_json["eventTicket"]["primaryFields"][0]["value"] == "Test Event"`.
  </behavior>
  <action>
    Create `ferro-wallet/tests/apple_integration.rs` with the following structure:

    ```rust
    //! End-to-end Apple `.pkpass` integration test.
    //!
    //! Mints a self-signed X.509 + RSA keypair at runtime (D-09) and reuses the cert as
    //! both the signing cert and the WWDR intermediate. openssl is happy with this; Apple
    //! Wallet on-device would NOT accept the resulting chain. This test verifies the
    //! STRUCTURE of the `.pkpass` ZIP and the shape of `pass.json` — it does NOT prove
    //! that real WWDR-issued passes will install on an iPhone. Real-device validation
    //! requires real Apple Developer credentials and is gated by the downstream
    //! gestiscilo-it integration phase (see RESEARCH.md Risk 3).

    use std::io::{Cursor, Read};

    use ferro_wallet::{
        ApplePassBuilder, AppleConfig, Branding, Field, FieldAlignment, GeoPoint, PassKind,
        RgbColor, TextColorMode, WalletSubject,
    };
    use openssl::asn1::Asn1Time;
    use openssl::hash::MessageDigest;
    use openssl::pkey::PKey;
    use openssl::rsa::Rsa;
    use openssl::x509::{X509Builder, X509NameBuilder};

    fn mint_self_signed() -> (String, String) {
        let rsa = Rsa::generate(2048).unwrap();
        let pkey = PKey::from_rsa(rsa).unwrap();

        let mut name = X509NameBuilder::new().unwrap();
        name.append_entry_by_text("CN", "ferro-wallet test").unwrap();
        let name = name.build();

        let mut builder = X509Builder::new().unwrap();
        builder.set_version(2).unwrap();
        builder.set_subject_name(&name).unwrap();
        builder.set_issuer_name(&name).unwrap();
        builder.set_pubkey(&pkey).unwrap();
        builder.set_not_before(&Asn1Time::days_from_now(0).unwrap()).unwrap();
        builder.set_not_after(&Asn1Time::days_from_now(365).unwrap()).unwrap();
        builder.sign(&pkey, MessageDigest::sha256()).unwrap();
        let cert = builder.build();

        let cert_pem = String::from_utf8(cert.to_pem().unwrap()).unwrap();
        let key_pem = String::from_utf8(pkey.private_key_to_pem_pkcs8().unwrap()).unwrap();
        (cert_pem, key_pem)
    }

    fn tiny_png(w: u32, h: u32, r: u8, g: u8, b: u8) -> Vec<u8> {
        use image::{ImageFormat, Rgba, RgbaImage};
        let img = RgbaImage::from_pixel(w, h, Rgba([r, g, b, 255]));
        let mut buf = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut buf, ImageFormat::Png)
            .unwrap();
        buf.into_inner()
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
        fn secondary(&self) -> Vec<Field> {
            vec![Field {
                key: "venue".to_string(),
                label: "Venue".to_string(),
                value: "Test Hall".to_string(),
                alignment: FieldAlignment::Natural,
            }]
        }
        fn auxiliary(&self) -> Vec<Field> { vec![] }
        fn back(&self) -> Vec<Field> { vec![] }
        fn barcode_token(&self) -> String { "ticket-token-abc".to_string() }
        fn relevant_at(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
        fn expires_at(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
        fn locations(&self) -> Vec<GeoPoint> { vec![] }
        fn branding(&self) -> Branding {
            Branding {
                organization_name: Some("Test Org".to_string()),
                logo_text: Some("Test".to_string()),
                background_color: RgbColor { r: 0, g: 0, b: 0 },
                text_color_mode: TextColorMode::Auto,
                logo_png_bytes: tiny_png(200, 80, 200, 50, 50),
                icon_png_bytes: None,
                hero_png_bytes: None,
            }
        }
    }

    #[test]
    fn build_pkpass_produces_valid_zip_and_pass_json() {
        let (cert_pem, key_pem) = mint_self_signed();

        let cfg = AppleConfig {
            pass_type_id: "pass.com.example.test".to_string(),
            team_id: "TEAMID1234".to_string(),
            cert_pem: cert_pem.clone(),
            key_pem,
            key_password: None,
            wwdr_pem: cert_pem, // D-09: reuse the self-signed cert as WWDR for structural test only.
        };

        let builder = ApplePassBuilder::new(cfg, "Test App".to_string()).unwrap();
        let pkpass_bytes = builder.build(&StubBooking).unwrap();

        // 1. Re-parse the ZIP and verify entry count + names.
        let mut zip = zip::ZipArchive::new(Cursor::new(&pkpass_bytes)).unwrap();
        assert_eq!(zip.len(), 9, "expected 9 ZIP entries, got {}", zip.len());

        let names: std::collections::HashSet<String> =
            (0..zip.len()).map(|i| zip.by_index(i).unwrap().name().to_string()).collect();
        let expected: std::collections::HashSet<&str> = [
            "pass.json", "manifest.json", "signature",
            "logo.png", "logo@2x.png", "logo@3x.png",
            "icon.png", "icon@2x.png", "icon@3x.png",
        ].into_iter().collect();
        for e in expected {
            assert!(names.contains(e), "ZIP missing entry: {e}");
        }

        // 2. Extract pass.json and assert key fields.
        let mut pass_json_str = String::new();
        zip.by_name("pass.json").unwrap().read_to_string(&mut pass_json_str).unwrap();
        let pass: serde_json::Value = serde_json::from_str(&pass_json_str).unwrap();

        assert_eq!(pass["passTypeIdentifier"], "pass.com.example.test");
        assert_eq!(pass["teamIdentifier"], "TEAMID1234");
        assert_eq!(pass["serialNumber"], "BOOK-1");
        assert_eq!(pass["barcodes"][0]["format"], "PKBarcodeFormatQR");
        assert_eq!(pass["barcodes"][0]["message"], "ticket-token-abc");
        assert_eq!(pass["eventTicket"]["primaryFields"][0]["value"], "Test Event");
    }
    ```

    Note: `zip` and `image` and `chrono` are already dependencies of `ferro-wallet` (declared in PLAN-01), so this test compiles without additional `[dev-dependencies]`.
  </action>
  <verify>
    <automated>cargo test -p ferro-wallet --test apple_integration &amp;&amp; cargo clippy -p ferro-wallet --all-targets -- -D warnings &amp;&amp; cargo fmt -p ferro-wallet -- --check &amp;&amp; test -f ferro-wallet/tests/apple_integration.rs &amp;&amp; grep -F 'mint_self_signed' ferro-wallet/tests/apple_integration.rs &amp;&amp; grep -F 'fn build_pkpass_produces_valid_zip_and_pass_json' ferro-wallet/tests/apple_integration.rs &amp;&amp; grep -F 'NOT prove' ferro-wallet/tests/apple_integration.rs</automated>
  </verify>
  <done>Integration test exists. `cargo test -p ferro-wallet --test apple_integration` passes. All 9 ZIP entries verified by name. All 6 pass.json field assertions pass. File header comment includes the "structure-only, NOT Apple-validity" disclaimer.</done>
</task>

</tasks>

<threat_model>
This plan introduces test-only code. It mints crypto material at runtime (no secrets in repo) and reuses the self-signed cert as WWDR — explicitly documented as structure-only verification.

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-151-Apple-SIGN | T | `tests/apple_integration.rs` self-signed cert reused as WWDR | accept | Documented in file header comment: this test verifies ZIP/JSON structure, NOT chain validity for Apple Wallet. Real-device validation requires real Apple Developer cert + WWDR intermediate and is gated by the downstream gestiscilo phase. |
| T-151-DEFAULT-CRED | I | `mint_self_signed()` | mitigate | No PEM material is committed to the repo. Cert + key are generated at test runtime via `Rsa::generate(2048)` + `X509Builder` and discarded when the test ends. |
</threat_model>

<verification>
- `cargo test -p ferro-wallet --test apple_integration` exits 0 (ACC-1j).
- `cargo test -p ferro-wallet` (full crate suite) exits 0.
- `cargo clippy -p ferro-wallet --all-targets -- -D warnings` exits 0.
- `cargo fmt -p ferro-wallet -- --check` exits 0.
- `grep -F 'NOT prove' ferro-wallet/tests/apple_integration.rs` returns at least one match (the disclaimer comment).
</verification>

<success_criteria>
ACC-1j passes. The Apple builder is end-to-end validated: cert parsing → manifest construction → PKCS#7 signing → ZIP assembly all produce a structurally-valid `.pkpass` with the expected entry list and pass.json shape.
</success_criteria>

<output>
After completion, create `.planning/phases/151-ferro-wallet-crate/151-06-SUMMARY.md` documenting the runtime cert minting strategy, the StubBooking field values, and the structure-only disclaimer location.
</output>

## PLANNING COMPLETE
