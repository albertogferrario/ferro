//! End-to-end Apple `.pkpass` integration test.
//!
//! Mints a self-signed X.509 + RSA keypair at runtime (D-09) and reuses the cert as
//! both the signing cert and the WWDR intermediate. openssl is happy with this; Apple
//! Wallet on-device would NOT accept the resulting chain. This test verifies the
//! STRUCTURE of the `.pkpass` ZIP and the shape of `pass.json` — it does NOT prove
//! that real WWDR-issued passes will install on an iPhone. Real-device validation
//! requires real Apple Developer credentials and is gated by the downstream
//! gestiscilo-it integration phase (see 151-RESEARCH.md Risk 3).

use std::io::{Cursor, Read};

use ferro_wallet::{
    AppleConfig, ApplePassBuilder, Branding, Field, FieldAlignment, GeoPoint, PassKind, RgbColor,
    TextColorMode, WalletSubject,
};
use openssl::asn1::Asn1Time;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use openssl::x509::{X509Builder, X509NameBuilder};

/// Mint a self-signed RSA-2048 / SHA-256 X.509 certificate valid for one year.
/// Returns the PEM-encoded cert and the PEM-encoded PKCS#8 private key.
///
/// The certificate is reused as the WWDR intermediate by the caller — see D-09.
/// This is structurally well-formed for openssl PKCS#7 signing but is NOT a
/// valid Apple Wallet pass certificate chain.
fn mint_self_signed() -> (String, String) {
    let rsa = Rsa::generate(2048).expect("generate rsa keypair");
    let pkey = PKey::from_rsa(rsa).expect("wrap rsa in PKey");

    let mut name = X509NameBuilder::new().expect("X509NameBuilder::new");
    name.append_entry_by_text("CN", "ferro-wallet test")
        .expect("append CN");
    let name = name.build();

    let mut builder = X509Builder::new().expect("X509Builder::new");
    builder.set_version(2).expect("set X.509 v3");
    builder.set_subject_name(&name).expect("set subject");
    builder.set_issuer_name(&name).expect("set issuer (self)");
    builder.set_pubkey(&pkey).expect("set pubkey");
    builder
        .set_not_before(&Asn1Time::days_from_now(0).expect("not_before"))
        .expect("set not_before");
    builder
        .set_not_after(&Asn1Time::days_from_now(365).expect("not_after"))
        .expect("set not_after");
    builder
        .sign(&pkey, MessageDigest::sha256())
        .expect("self-sign");
    let cert = builder.build();

    let cert_pem =
        String::from_utf8(cert.to_pem().expect("cert to_pem")).expect("cert PEM is valid UTF-8");
    let key_pem = String::from_utf8(
        pkey.private_key_to_pem_pkcs8()
            .expect("private_key_to_pem_pkcs8"),
    )
    .expect("key PEM is valid UTF-8");
    (cert_pem, key_pem)
}

/// Build a solid-colour PNG of the given dimensions. Used to feed the
/// `Branding.logo_png_bytes` slot — the `images` module derives the full
/// Apple logo + icon set from this single source.
fn tiny_png(w: u32, h: u32, r: u8, g: u8, b: u8) -> Vec<u8> {
    use image::{ImageFormat, Rgba, RgbaImage};
    let img = RgbaImage::from_pixel(w, h, Rgba([r, g, b, 255]));
    let mut buf = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut buf, ImageFormat::Png)
        .expect("encode tiny PNG");
    buf.into_inner()
}

/// Minimal `WalletSubject` implementation with deterministic field values that
/// the assertions below expect verbatim.
struct StubBooking;

impl WalletSubject for StubBooking {
    fn pass_kind(&self) -> PassKind {
        PassKind::EventTicket
    }
    fn serial(&self) -> String {
        "BOOK-1".to_string()
    }
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
    fn auxiliary(&self) -> Vec<Field> {
        vec![]
    }
    fn back(&self) -> Vec<Field> {
        vec![]
    }
    fn barcode_token(&self) -> String {
        "ticket-token-abc".to_string()
    }
    fn relevant_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        None
    }
    fn expires_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        None
    }
    fn locations(&self) -> Vec<GeoPoint> {
        vec![]
    }
    fn branding(&self) -> Branding {
        Branding {
            organization_name: Some("Test Org".to_string()),
            logo_text: Some("Test".to_string()),
            background_color: RgbColor { r: 0, g: 0, b: 0 },
            text_color_mode: TextColorMode::Auto,
            logo_png_bytes: Some(tiny_png(200, 80, 200, 50, 50)),
            icon_png_bytes: None,
            hero_png_bytes: None,
        }
    }
}

/// ACC-1j — `.pkpass` ZIP contains 9 expected files and `pass.json` carries
/// the correct identifiers, barcode message, and primary field value.
#[test]
fn build_pkpass_produces_valid_zip_and_pass_json() {
    let (cert_pem, key_pem) = mint_self_signed();

    let cfg = AppleConfig {
        pass_type_id: "pass.com.example.test".to_string(),
        team_id: "TEAMID1234".to_string(),
        cert_pem: cert_pem.clone(),
        key_pem,
        key_password: None,
        // D-09: reuse the self-signed cert as WWDR for structural test only.
        wwdr_pem: cert_pem,
    };

    let builder =
        ApplePassBuilder::new(cfg, "Test App".to_string()).expect("ApplePassBuilder::new");
    let pkpass_bytes = builder
        .build(&StubBooking)
        .expect("ApplePassBuilder::build");

    // 1. Re-parse the ZIP and verify entry count + names.
    let mut zip = zip::ZipArchive::new(Cursor::new(&pkpass_bytes)).expect("parse pkpass as zip");
    assert_eq!(zip.len(), 9, "expected 9 ZIP entries, got {}", zip.len());

    let names: std::collections::HashSet<String> = (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().to_string())
        .collect();
    let expected: std::collections::HashSet<&str> = [
        "pass.json",
        "manifest.json",
        "signature",
        "logo.png",
        "logo@2x.png",
        "logo@3x.png",
        "icon.png",
        "icon@2x.png",
        "icon@3x.png",
    ]
    .into_iter()
    .collect();
    for e in expected {
        assert!(names.contains(e), "ZIP missing entry: {e}");
    }

    // 2. Extract pass.json and assert key fields.
    let mut pass_json_str = String::new();
    zip.by_name("pass.json")
        .expect("pass.json entry")
        .read_to_string(&mut pass_json_str)
        .expect("read pass.json");
    let pass: serde_json::Value =
        serde_json::from_str(&pass_json_str).expect("pass.json is valid JSON");

    assert_eq!(pass["passTypeIdentifier"], "pass.com.example.test");
    assert_eq!(pass["teamIdentifier"], "TEAMID1234");
    assert_eq!(pass["serialNumber"], "BOOK-1");
    assert_eq!(pass["barcodes"][0]["format"], "PKBarcodeFormatQR");
    assert_eq!(pass["barcodes"][0]["message"], "ticket-token-abc");
    assert_eq!(
        pass["eventTicket"]["primaryFields"][0]["value"],
        "Test Event"
    );
}
