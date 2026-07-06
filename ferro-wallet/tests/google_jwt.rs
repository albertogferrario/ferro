//! End-to-end Google Wallet save-link JWT integration test (ACC-1k).
//!
//! Mints an RSA keypair at runtime (D-09), signs a save JWT for a [`StubBooking`]
//! subject via [`GoogleWalletBuilder::save_jwt`], and decodes the JWT with the
//! matching public key. Verifies the claim shape per D-08:
//!
//! - `iss = service_account_email`
//! - `aud = "google"`
//! - `typ = "savetowallet"`
//! - `origins = [app_url]`
//! - `payload.eventTicketObjects` is a single-element array
//! - `[0].id = "{issuer_id}.{serial}"`
//! - `[0].classId = "{issuer_id}.{pass_type_id_default}"`
//! - `[0].state = "active"`
//! - `[0].barcode.{type,value}` carries the QR token
//!
//! Pitfall 3 mitigation: [`Validation::new(Algorithm::RS256)`] requires `exp` and
//! `iat` by default — save JWTs have no `exp`. We disable that and clear the
//! required-claims set before decoding.

use std::collections::HashSet;

use ferro_wallet::{
    Branding, Field, FieldAlignment, GeoPoint, GoogleConfig, GoogleWalletBuilder, PassKind,
    RgbColor, TextColorMode, WalletSubject,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use openssl::pkey::PKey;
use openssl::rsa::Rsa;

/// Mint a fresh RSA-2048 keypair and return PKCS#8 PEM (private) + SubjectPublicKeyInfo
/// PEM (public). Keys are generated per-test and discarded — no real Google credentials
/// touch CI.
fn mint_rsa_keypair() -> (String, String) {
    let rsa = Rsa::generate(2048).expect("generate rsa keypair");
    let pkey = PKey::from_rsa(rsa).expect("wrap rsa as pkey");
    let private_pem = String::from_utf8(
        pkey.private_key_to_pem_pkcs8()
            .expect("encode private key as pkcs8 pem"),
    )
    .expect("pkcs8 pem is utf-8");
    let public_pem = String::from_utf8(pkey.public_key_to_pem().expect("encode public key as pem"))
        .expect("public pem is utf-8");
    (private_pem, public_pem)
}

/// Minimal `WalletSubject` impl mirroring the stub used by `tests/apple_integration.rs`.
/// Field values are known constants the test asserts on after the JWT roundtrip.
struct StubBooking;

impl WalletSubject for StubBooking {
    fn pass_kind(&self) -> PassKind {
        PassKind::EventTicket
    }
    fn serial(&self) -> String {
        "BOOK-1".to_string()
    }
    fn primary(&self) -> Vec<Field> {
        vec![Field {
            key: "event".to_string(),
            label: "Event".to_string(),
            value: "Test Event".to_string(),
            alignment: FieldAlignment::Left,
        }]
    }
    fn secondary(&self) -> Vec<Field> {
        vec![]
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
            logo_text: None,
            background_color: RgbColor { r: 0, g: 0, b: 0 },
            text_color_mode: TextColorMode::Auto,
            logo_png_bytes: None,
            icon_png_bytes: None,
            hero_png_bytes: None,
        }
    }
}

/// Construct a fully-wired builder backed by a freshly minted private key — used by
/// the `save_url` test where we don't need the public PEM.
fn build_test_builder() -> GoogleWalletBuilder {
    let (private_pem, _) = mint_rsa_keypair();
    let cfg = GoogleConfig {
        issuer_id: "3388000000000000000".to_string(),
        service_account_email: "sa@example.iam.gserviceaccount.com".to_string(),
        service_account_private_key_pem: private_pem,
    };
    GoogleWalletBuilder::new(
        cfg,
        "Test App".to_string(),
        "https://example.com".to_string(),
    )
    .expect("builder construction never errors in v1")
}

/// ACC-1k: full RS256 JWT roundtrip — mint keypair → sign save JWT → decode with the
/// matching public key → assert the documented claim shape (D-08, D-07).
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
    .expect("builder construction never errors in v1");

    let jwt = builder.save_jwt(&StubBooking).expect("sign save jwt");

    // Pitfall 3: save JWT has no `exp` claim. Disable exp validation AND clear the
    // required-claims set — `Validation::new(Algorithm::RS256)` requires `exp` by
    // default, which would reject our valid token with `MissingRequiredClaim("exp")`.
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = false;
    validation.required_spec_claims = HashSet::new();
    validation.set_audience(&["google"]);

    let decoded = decode::<serde_json::Value>(
        &jwt,
        &DecodingKey::from_rsa_pem(public_pem.as_bytes()).expect("public pem parses"),
        &validation,
    )
    .expect("decode roundtrip");

    let c = decoded.claims;
    assert_eq!(c["iss"], "sa@example.iam.gserviceaccount.com");
    assert_eq!(c["aud"], "google");
    assert_eq!(c["typ"], "savetowallet");
    assert_eq!(
        c["origins"][0], "https://example.com",
        "origins[0] must equal the configured app_url"
    );

    let objects = c["payload"]["eventTicketObjects"]
        .as_array()
        .expect("payload.eventTicketObjects must be an array");
    assert_eq!(
        objects.len(),
        1,
        "expected exactly one eventTicketObject per save JWT"
    );
    let obj = &objects[0];
    assert_eq!(obj["id"], "3388000000000000000.BOOK-1");
    assert_eq!(obj["classId"], "3388000000000000000.booking");
    assert_eq!(obj["state"], "active");
    assert_eq!(obj["barcode"]["type"], "qrCode");
    assert_eq!(obj["barcode"]["value"], "ticket-token-abc");
}

/// `save_url` composes the Google save endpoint URL around the signed JWT.
#[test]
fn save_url_returns_pay_google_com_prefix() {
    let builder = build_test_builder();
    let url = builder.save_url(&StubBooking).expect("save_url");
    assert!(
        url.starts_with("https://pay.google.com/gp/v/save/"),
        "save_url should start with https://pay.google.com/gp/v/save/, got {url}"
    );
}
