# ferro-wallet — Crate Design

**Status:** Draft for implementation
**Date:** 2026-05-11
**Phase:** 151 (v11.10 ferro-wallet milestone)
**Source:** gestiscilo-it digital wallet booking passes field test — full design at `gestiscilo-it/app/docs/superpowers/specs/2026-05-11-wallet-passes-design.md`.

## 1. Goal

Ship a project-agnostic `ferro-wallet` crate that any ferro application can use to issue Apple `.pkpass` files and Google Wallet save-links. The crate provides the `WalletSubject` trait and two builders. It must respect ferro architecture principle 6: no hardcoded application identity; reads `APP_NAME` / `APP_URL` via its own `WalletConfig::from_env`, mirroring `ferro-inertia::InertiaConfig::app_name` and `ferro-stripe::StripeConfig::from_env`.

The killer property is the round-trip-with-no-app: an end-user installs a wallet pass on their phone and the OS surfaces it at the right time/place without the issuing application running. The crate's job is to make that round trip cleanly type-safe and reusable.

## 2. Scope

### In scope (Phase 151)

- `WalletSubject` trait + value types (`Field`, `Branding`, `PassKind`, `GeoPoint`, `RgbColor`, `TextColorMode`, `FieldAlignment`).
- `WalletConfig`, `AppleConfig`, `GoogleConfig` + permissive `from_env` (missing Apple cluster ⇒ `apple: None`, same for Google).
- `ApplePassBuilder`: SHA1 manifest, PKCS#7 detached signing (via `openssl`), ZIP packaging.
- `GoogleWalletBuilder`: `eventTicketObject` JSON construction + RS256 JWT (`jsonwebtoken`) + `pay.google.com/gp/v/save/{jwt}` URL.
- `images` module: input-image normalisation to all required Apple resolutions (1x / 2x / 3x logos + icons) and Google hero (1032×336).
- `qr` module: PNG QR generator + base64 data-URI helper.
- Unit tests for every public API surface; one end-to-end Apple integration test that mints a self-signed cert at runtime and asserts the produced `.pkpass` shape; one Google JWT roundtrip test.

### Out of scope (deferred to future ferro-wallet phases)

- Apple Wallet Web Service Protocol (live updates, `passesUpdatedSince`, APNs push) — punted.
- Apple Express Mode (NFC tap) — requires entitlements + Web Service Protocol.
- Google Wallet `objects.patch` API (live updates).
- Locale-aware label resolution (v1 ships callers' raw strings).
- Live SVG preview of the rendered card — render only via the real `.pkpass`.
- Wallet subjects beyond the three pass kinds (EventTicket / Generic / Coupon) — the trait already supports them; only EventTicket is exercised in tests.

## 3. Public API

### 3.1 The trait

```rust
pub trait WalletSubject {
    fn pass_kind(&self) -> PassKind;
    fn serial(&self) -> String;
    fn primary(&self) -> Field;
    fn secondary(&self) -> Vec<Field>;
    fn auxiliary(&self) -> Vec<Field>;
    fn back(&self) -> Vec<Field>;
    fn barcode_token(&self) -> String;
    fn relevant_at(&self) -> Option<DateTime<Utc>>;
    fn expires_at(&self) -> Option<DateTime<Utc>>;
    fn locations(&self) -> Vec<GeoPoint>;
    fn branding(&self) -> Branding;
}

pub enum PassKind { EventTicket, Generic, Coupon }

pub struct Field {
    pub key: String,
    pub label: String,
    pub value: String,
    pub alignment: FieldAlignment,
}

pub struct Branding {
    pub organization_name: Option<String>,   // None → builder falls back to WalletConfig.app_name
    pub logo_text: Option<String>,
    pub background_color: RgbColor,
    pub text_color_mode: TextColorMode,       // Auto | Light | Dark
    pub logo_png_bytes: Vec<u8>,              // builder resizes to 1x/2x/3x
    pub icon_png_bytes: Option<Vec<u8>>,      // builder derives from logo if absent
    pub hero_png_bytes: Option<Vec<u8>>,      // Google Wallet hero; optional
}

pub struct GeoPoint { pub latitude: f64, pub longitude: f64, pub relevant_text: Option<String> }
```

### 3.2 Config

```rust
pub struct WalletConfig {
    pub app_name: String,   // APP_NAME (matches framework::config::AppConfig)
    pub app_url: String,    // APP_URL
    pub apple: Option<AppleConfig>,
    pub google: Option<GoogleConfig>,
}

pub struct AppleConfig {
    pub pass_type_id: String,        // APPLE_WALLET_PASS_TYPE_ID
    pub team_id: String,             // APPLE_WALLET_TEAM_ID
    pub cert_pem: String,            // APPLE_WALLET_CERT_PEM
    pub key_pem: String,             // APPLE_WALLET_KEY_PEM
    pub key_password: Option<String>,// APPLE_WALLET_KEY_PASSWORD
    pub wwdr_pem: String,            // APPLE_WALLET_WWDR_PEM
}

pub struct GoogleConfig {
    pub issuer_id: String,                          // GOOGLE_WALLET_ISSUER_ID
    pub service_account_email: String,              // GOOGLE_WALLET_SERVICE_ACCOUNT_EMAIL
    pub service_account_private_key_pem: String,    // GOOGLE_WALLET_SERVICE_ACCOUNT_KEY_PEM
}

impl WalletConfig {
    /// Permissive — missing Apple cluster ⇒ apple: None, same for Google.
    /// Never returns a hard error for missing wallet env vars (callers gate
    /// features on builder availability).
    pub fn from_env() -> Result<Self, WalletError>;
}
```

### 3.3 Builders

```rust
pub struct ApplePassBuilder { /* parsed cert + key + wwdr + identifiers */ }
pub struct GoogleWalletBuilder { /* parsed SA key + identifiers */ }

impl ApplePassBuilder {
    pub fn new(cfg: AppleConfig, app_name: String) -> Result<Self, WalletError>;
    pub fn build<S: WalletSubject>(&self, s: &S) -> Result<Vec<u8>, WalletError>; // .pkpass bytes
}

impl GoogleWalletBuilder {
    pub fn new(cfg: GoogleConfig, app_name: String, app_url: String) -> Result<Self, WalletError>;
    pub fn save_jwt<S: WalletSubject>(&self, s: &S) -> Result<String, WalletError>;
    pub fn save_url<S: WalletSubject>(&self, s: &S) -> Result<String, WalletError>;
}
```

### 3.4 Image + QR helpers

```rust
// images.rs — pure transformation, no I/O
pub fn fit_to(bytes: &[u8], w: u32, h: u32) -> Result<Vec<u8>, WalletError>;
pub fn apple_logo_set(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, WalletError>; // logo.png, logo@2x.png, logo@3x.png
pub fn apple_icon_set(icon: Option<&[u8]>, logo_fallback: &[u8]) -> Result<Vec<(String, Vec<u8>)>, WalletError>;
pub fn google_hero(bytes: &[u8]) -> Result<Vec<u8>, WalletError>; // 1032x336

// qr.rs
pub fn png(data: &str, size: u32) -> Result<Vec<u8>, WalletError>;
pub fn data_uri(data: &str, size: u32) -> Result<String, WalletError>;
```

## 4. Two builders, deliberately separate

Apple and Google share nothing at the wire-format level. Apple is PKCS#7 detached signing over a SHA1 manifest inside a ZIP. Google is an RS256-signed JWT pointing at a JSON object. A unified builder would obscure format-specific failure modes and gain no shared code. The `WalletSubject` trait is the only abstraction; the builders are deliberately split.

## 5. Crate dependencies

| Dep                     | Why                                                       |
| ----------------------- | --------------------------------------------------------- |
| `openssl = "0.10"`      | PKCS#7 detached signing for Apple                         |
| `zip = "2"`             | `.pkpass` packaging                                       |
| `jsonwebtoken = "9"`    | Google save-link JWT (RS256)                              |
| `image = "0.25"`        | Logo / icon / hero normalisation                          |
| `qrcode-generator = "5"`| QR PNG generation                                         |
| `sha1 = "0.10"`         | Apple manifest digest                                     |
| `base64 = "0.22"`       | QR data-URI encoding                                      |
| `serde`, `serde_json`, `thiserror`, `chrono` | std-grade plumbing                       |

No dependency on `framework` — the crate stays pure.

## 6. Error type

```rust
pub enum WalletError {
    Config(String),
    AppleSign(String),
    ApplePackage(String),
    GoogleJwt(String),
    Image(String),
    Qr(String),
    InvalidInput(String),
    Io(#[from] std::io::Error),
}
```

`thiserror`-derived. Each variant prefixes its display with its name (`"config: …"`, `"apple sign: …"`) so log greps stay surgical.

## 7. Testing strategy

- **Unit tests** in each module file: pass.json shape, manifest SHA1, `RgbColor::from_hex`, luminance, image resize dimensions, config env parsing, JWT URL formatting.
- **Integration test** `tests/apple_integration.rs`: mint a self-signed cert at runtime, build a `.pkpass` from a `StubBooking` `WalletSubject`, unzip and assert the bundle contains all required files (`pass.json`, `manifest.json`, `signature`, 1x/2x/3x logo + icon), then assert key `pass.json` fields (`passTypeIdentifier`, `teamIdentifier`, `serialNumber`, `barcode.message`, `eventTicket.primaryFields[0].value`).
- **Integration test** `tests/google_jwt.rs`: mint an RSA keypair at runtime, build a save JWT for a `StubBooking`, decode with `jsonwebtoken` using the public key, assert claims (`iss`, `aud=google`, `typ=savetowallet`, payload structure, object ID format). Plus a test that `save_url(...)` returns a `https://pay.google.com/gp/v/save/<jwt>` string.

No reliance on real Apple/Google certificates in CI.

## 8. Release

Auto-publishes via the existing ferro GitHub Actions workflow on push to master (per `feedback_ferro_publish.md` — do not run `cargo publish` manually). Bumps the workspace `[workspace.package] version` once Phase 151 is verified.

## 9. Acceptance criteria

- `cargo test -p ferro-wallet` is green (all unit + integration tests).
- `cargo build --workspace` is green with the new crate in `members`.
- `cargo doc --no-deps -p ferro-wallet` produces clean output.
- A downstream consumer (gestiscilo) can depend on `ferro-wallet = "0.2.X"` (the version bumped at release) and successfully build a real `.pkpass` against its own booking model — verified out-of-band by the gestiscilo integration phase.

## 10. Out-of-band consumer notes

The downstream gestiscilo integration is tracked separately in `gestiscilo-it/app/docs/superpowers/plans/2026-05-11-wallet-passes.md` and `gestiscilo-it/app/docs/superpowers/specs/2026-05-11-wallet-passes-design.md`. Phase 151 must be shipped and the version bumped before that integration can compile against the published crate.
