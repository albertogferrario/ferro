# Phase 151: ferro-wallet — Pattern Map

**Mapped:** 2026-05-11
**Files analyzed:** 17 new files (15 source, 2 integration tests) + 2 workspace edits
**Analogs found:** 9 strong analogs in ferro-stripe / ferro-whatsapp / ferro-ai / ferro-inertia / framework
**No-analog files (NEW PATTERN):** 6 — spec §3 + downstream `gestiscilo-it/.../wallet-passes.md` Phase A are the authoritative source

## File Classification

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `ferro-wallet/Cargo.toml` | crate manifest | build | `ferro-whatsapp/Cargo.toml` (newer pattern, `version.workspace = true`) | exact |
| `ferro-wallet/README.md` | docs | n/a | `ferro-stripe/README.md` (10 lines, short, links docs.rs) | exact |
| `ferro-wallet/src/lib.rs` | crate root + re-exports | n/a | `ferro-stripe/src/lib.rs` | exact |
| `ferro-wallet/src/error.rs` | error enum (thiserror) | n/a | `ferro-stripe/src/error.rs` + `ferro-whatsapp/src/error.rs` (test coverage) | exact |
| `ferro-wallet/src/config.rs` | env-driven config (permissive D-02) | request-response (env read) | `ferro-stripe/src/config.rs` (config shape + env-pollution-safe test pattern) + `framework/src/config/providers/app.rs` (APP_NAME/APP_URL defaults) | role-match (Stripe is hard-erroring; ferro-wallet must be permissive) |
| `ferro-wallet/src/subject.rs` | domain trait + value types | n/a | none — first user-implementable trait inside a `ferro-*` crate | NEW PATTERN |
| `ferro-wallet/src/images.rs` | pure transform (image bytes → image bytes) | transform / batch | none — image pipeline is new to the workspace | NEW PATTERN |
| `ferro-wallet/src/qr.rs` | pure transform (string → PNG bytes / data URI) | transform | none — QR pipeline is new to the workspace | NEW PATTERN |
| `ferro-wallet/src/apple/mod.rs` | builder facade (multi-file submodule) | request-response | `ferro-stripe/src/webhook/mod.rs` (submodule + `mod.rs` + `pub use`) | role-match (Stripe submodule reexports parsed events; Apple submodule reexports a builder) |
| `ferro-wallet/src/apple/manifest.rs` | JSON construction + SHA1 digest | transform | partial — no Apple/SHA1 analog; closest is `ferro-stripe/src/webhook/verify.rs` (HMAC over body) | NEW PATTERN (cryptographic primitive isolated in its own file — same shape as `verify.rs`) |
| `ferro-wallet/src/apple/sign.rs` | PKCS#7 detached signing (openssl) | transform / crypto | partial — `ferro-stripe/src/webhook/verify.rs` is the closest "crypto-in-its-own-file" precedent | NEW PATTERN |
| `ferro-wallet/src/apple/package.rs` | ZIP assembly | transform / batch | none — no packaging concern elsewhere in the workspace | NEW PATTERN |
| `ferro-wallet/src/google/mod.rs` | builder facade (multi-file submodule) | request-response | `ferro-stripe/src/webhook/mod.rs` | role-match |
| `ferro-wallet/src/google/object.rs` | JSON object construction | transform | partial — `ferro-stripe/src/checkout.rs` builder produces a JSON-shaped request | role-partial |
| `ferro-wallet/src/google/jwt.rs` | RS256 JWT signing | transform / crypto | partial — `ferro-stripe/src/webhook/verify.rs` (HMAC-SHA256, different algo + verify-vs-sign) | NEW PATTERN |
| `ferro-wallet/tests/apple_integration.rs` | end-to-end integration test (runtime-minted X.509) | n/a | `ferro-stripe/tests/parser_contract.rs` (integration-test layout + `include_str!` fixtures) | role-match (Stripe uses JSON fixtures; ferro-wallet mints crypto at runtime per D-09) |
| `ferro-wallet/tests/google_jwt.rs` | RS256 roundtrip integration test | n/a | `ferro-stripe/tests/parser_contract.rs` | role-match |

### Workspace edits (not files-to-create, but pattern-relevant)

| Edit | Role | Analog | Match Quality |
|------|------|--------|---------------|
| `Cargo.toml` workspace `[workspace] members` array — append `"ferro-wallet",` | build / workspace registration | existing members array (line 23 is `"ferro-whatsapp",` — last entry) | exact |
| `.github/workflows/publish.yml` Wave 1a list — append `ferro-wallet` | release | line 201 `WAVE1A_CRATES="…"` | exact |

## Pattern Assignments

---

### `ferro-wallet/Cargo.toml` (crate manifest)

**Analog:** `ferro-whatsapp/Cargo.toml` (newer pattern — uses `version.workspace = true`; ferro-stripe is on its own version `0.5.0` and is the **wrong** template for new crates per RESEARCH.md §"Pattern Alignment with ferro-stripe")

**`[package]` header pattern** (`ferro-whatsapp/Cargo.toml` lines 1–11):
```toml
[package]
name = "ferro-whatsapp"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "WhatsApp Business Cloud API integration for the Ferro framework"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["whatsapp", "messaging", "notifications", "meta", "ferro"]
categories = ["web-programming"]
readme = "README.md"
homepage = "https://ferro-rs.dev"
```

**Adapt for ferro-wallet:**
```toml
[package]
name = "ferro-wallet"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Digital wallet pass issuance (Apple .pkpass + Google Wallet) for the Ferro framework"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["wallet", "pkpass", "google-wallet", "apple-wallet", "ferro"]
categories = ["web-programming"]
readme = "README.md"
```

**`[dependencies]` shape** (literal versions, **not** `workspace = true` — `Cargo.toml` workspace root has no `[workspace.dependencies]` block; see RESEARCH.md §"No `[workspace.dependencies]` block"). Pull versions from RESEARCH.md §"Standard Stack":
```toml
[dependencies]
openssl = "0.10"
zip = "2"
jsonwebtoken = "9"
image = "0.25"
qrcode-generator = "5"
sha1 = "0.10"
base64 = "0.22"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
```

No `[dev-dependencies]` required — Apple integration test reuses `openssl` (already a dep), Google JWT test reuses `jsonwebtoken` (already a dep). Both mint crypto material at runtime per D-09.

---

### `ferro-wallet/README.md` (docs)

**Analog:** `ferro-stripe/README.md` (10 lines exactly — short, neutral, links docs.rs and the framework repo)

**Pattern** (`ferro-stripe/README.md` lines 1–11):
```markdown
# ferro-stripe

Stripe payment integration for the Ferro framework.

Provides two billing dimensions: platform subscriptions (the Ferro application charges tenants for plan tiers) and Stripe Connect (tenants collect one-time payments from end users via their own connected account). Includes a typed client wrapper, webhook signature verification, and integration with ferro-events and ferro-queue.

Status: part of the [ferro](https://github.com/albertogferrario/ferro) framework workspace.

Documentation: https://docs.rs/ferro-stripe

License: MIT
```

**Adapt:** swap "Stripe payment integration" framing for `.pkpass` + Google Wallet issuance; mention the `WalletSubject` trait as the integration point; point to the spec for the full API; keep length to ~10 lines.

---

### `ferro-wallet/src/lib.rs` (crate root + re-exports)

**Analog:** `ferro-stripe/src/lib.rs` lines 43–69

**Module declaration + re-export block** (`ferro-stripe/src/lib.rs` lines 43–58):
```rust
pub mod account;
pub mod checkout;
pub mod client;
pub mod config;
pub mod error;
pub mod idempotency;
pub mod refund;
#[cfg(any(test, feature = "test-helpers"))]
pub mod testing;
pub mod webhook;

pub use account::{billing_portal_url, create_account, create_link, retrieve_account};
pub use checkout::{CheckoutBuilder, CheckoutIntent, LineItem, Mode};
pub use client::Stripe;
pub use config::StripeConfig;
pub use error::Error;
```

**Adapt for ferro-wallet** (per D-11 — apple/google re-exports stay commented out until their builder bodies land in PLAN-05 / PLAN-07):
```rust
pub mod apple;
pub mod config;
pub mod error;
pub mod google;
pub mod images;
pub mod qr;
pub mod subject;

// Restored in PLAN-05 (apple builder body lands)
pub use apple::ApplePassBuilder;
pub use config::{AppleConfig, GoogleConfig, WalletConfig};
pub use error::WalletError;
// Restored in PLAN-07 (google builder body lands)
pub use google::GoogleWalletBuilder;
pub use subject::{
    Branding, Field, FieldAlignment, GeoPoint, PassKind, RgbColor, TextColorMode, WalletSubject,
};
```

**Crate-level doc comment pattern** (`ferro-stripe/src/lib.rs` lines 1–42, abbreviated): include a `# ferro-wallet` header, one-sentence purpose, and a `## Quick Start` `rust,ignore` block showing `WalletConfig::from_env` + `ApplePassBuilder::new` + `builder.build(&subject)`. Keep doc tone neutral per CLAUDE.md "repository documents must read as neutral".

---

### `ferro-wallet/src/error.rs` (WalletError enum, thiserror)

**Analog (variant shape):** `ferro-stripe/src/error.rs` lines 1–27 — the workspace canonical pattern for `thiserror`-derived enums with name-prefixed `#[error]` strings.
**Analog (test coverage):** `ferro-whatsapp/src/error.rs` lines 38–92 — exhaustive `#[cfg(test)] mod tests` covering one `Display`-format assertion per variant.

**Enum derive + variant pattern** (`ferro-stripe/src/error.rs` lines 1–27):
```rust
/// Errors that can occur in ferro-stripe operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Configuration error (missing env var or invalid value).
    #[error("stripe config error: {0}")]
    Config(String),

    /// Stripe API returned an error.
    #[error("stripe API error: {0}")]
    Stripe(String),

    /// No Connect account linked to this tenant.
    #[error("no Stripe Connect account linked to this tenant")]
    NoConnectAccount,

    /// Webhook signature verification failed.
    #[error("webhook verification failed: {0}")]
    WebhookVerification(String),
    // ...
}
```

**Test pattern** (`ferro-whatsapp/src/error.rs` lines 38–60):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_config_displays_message() {
        let e = Error::Config("WHATSAPP_APP_SECRET not set".into());
        assert_eq!(
            e.to_string(),
            "configuration error: WHATSAPP_APP_SECRET not set"
        );
    }
    // ... one #[test] per variant
}
```

**Adapt for ferro-wallet** (D-04 — every variant prefixes its `Display` with its own name; `Io(#[from] std::io::Error)` for zip + io plumbing):
```rust
#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("config: {0}")]
    Config(String),
    #[error("apple sign: {0}")]
    AppleSign(String),
    #[error("apple package: {0}")]
    ApplePackage(String),
    #[error("google jwt: {0}")]
    GoogleJwt(String),
    #[error("image: {0}")]
    Image(String),
    #[error("qr: {0}")]
    Qr(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
```

Pair with a `#[cfg(test)] mod tests` block in the ferro-whatsapp style — one `#[test] fn …_displays_message()` per variant.

---

### `ferro-wallet/src/config.rs` (WalletConfig + AppleConfig + GoogleConfig + permissive from_env)

**Analog (shape + env-pollution-safe test):** `ferro-stripe/src/config.rs` lines 1–90.
**Analog (defaults source-of-truth for APP_NAME / APP_URL):** `framework/src/config/providers/app.rs` lines 16–26.

**Struct shape pattern** (`ferro-stripe/src/config.rs` lines 1–16):
```rust
use crate::Error;

#[derive(Debug, Clone)]
pub struct StripeConfig {
    pub api_key: String,
    pub webhook_secret: String,
    pub connect_webhook_secret: Option<String>,
    pub application_fee_percent: Option<f64>,
}
```

**`from_env` pattern — ferro-stripe (HARD-ERRORING)** (`ferro-stripe/src/config.rs` lines 18–46):
```rust
pub fn from_env() -> Result<Self, Error> {
    let api_key = std::env::var("STRIPE_SECRET_KEY")
        .map_err(|_| Error::Config("STRIPE_SECRET_KEY not set".to_string()))?;
    let webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET")
        .map_err(|_| Error::Config("STRIPE_WEBHOOK_SECRET not set".to_string()))?;
    let connect_webhook_secret = std::env::var("STRIPE_CONNECT_WEBHOOK_SECRET").ok();
    // ...
    Ok(Self { api_key, webhook_secret, connect_webhook_secret, application_fee_percent })
}
```

**`from_env` pattern — framework AppConfig (DEFAULTS-WITH-FALLBACK)** (`framework/src/config/providers/app.rs` lines 16–26):
```rust
impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            name: env("APP_NAME", "Ferro Application".to_string()),
            environment: Environment::detect(),
            debug: env("APP_DEBUG", true),
            url: env("APP_URL", "http://localhost:8080".to_string()),
        }
    }
}
```

**Adapt for ferro-wallet** (D-02 — PERMISSIVE; combine both patterns: defaults-with-fallback for APP_NAME/APP_URL, optional-cluster for Apple/Google; never errors on missing wallet vars):
```rust
pub fn from_env() -> Result<Self, WalletError> {
    // Fallbacks match framework::config::providers::app.rs lines 20 + 23
    let app_name = std::env::var("APP_NAME")
        .unwrap_or_else(|_| "Ferro Application".to_string());
    let app_url = std::env::var("APP_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    let apple = AppleConfig::from_env_optional()?;   // Ok(None) if any required Apple var missing
    let google = GoogleConfig::from_env_optional()?; // Ok(None) if any required Google var missing

    Ok(Self { app_name, app_url, apple, google })
}
```

**Env-pollution-safe test pattern** (`ferro-stripe/src/config.rs` lines 49–73 — workspace exemplar; RESEARCH.md Pitfall 6 explicitly calls this out):
```rust
#[test]
fn from_env_returns_config_error_when_key_missing() {
    // Save existing values so other tests are not affected by the removal.
    let old_key = std::env::var("STRIPE_SECRET_KEY").ok();
    let old_secret = std::env::var("STRIPE_WEBHOOK_SECRET").ok();

    std::env::remove_var("STRIPE_SECRET_KEY");
    std::env::remove_var("STRIPE_WEBHOOK_SECRET");

    let result = StripeConfig::from_env();
    assert!(matches!(result, Err(Error::Config(_))));

    // Restore to avoid polluting the process-global env for other tests.
    if let Some(k) = old_key { std::env::set_var("STRIPE_SECRET_KEY", k); }
    if let Some(s) = old_secret { std::env::set_var("STRIPE_WEBHOOK_SECRET", s); }
}
```

**Adapt:** apply the save-remove-assert-restore idiom to every config test that touches `APPLE_WALLET_*` or `GOOGLE_WALLET_*` env vars. Test assertions invert to `assert!(result.unwrap().apple.is_none())` since `from_env` is permissive.

---

### `ferro-wallet/src/subject.rs` (WalletSubject trait + value types) — NEW PATTERN

**Analog:** none. No existing `ferro-*` crate exposes a user-implementable domain trait. The closest precedent is `ferro-whatsapp/src/dedup.rs::DeduplicationStore` (a pluggable trait) but its shape is async + storage-backed, not pure-data-contract.

**Authoritative source:** spec §3.1 (`docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md` lines 39–74):
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
    pub organization_name: Option<String>,
    pub logo_text: Option<String>,
    pub background_color: RgbColor,
    pub text_color_mode: TextColorMode,
    pub logo_png_bytes: Vec<u8>,
    pub icon_png_bytes: Option<Vec<u8>>,
    pub hero_png_bytes: Option<Vec<u8>>,
}

pub struct GeoPoint { pub latitude: f64, pub longitude: f64, pub relevant_text: Option<String> }
```

**Workspace convention to follow** (from ferro-stripe / ferro-whatsapp value types):
- `#[derive(Debug, Clone)]` on every value type.
- `#[derive(Debug, Clone, PartialEq, Eq)]` on `PassKind` / `FieldAlignment` / `TextColorMode` (closed enums).
- `#[derive(Debug, Clone, Copy, PartialEq)]` on `RgbColor` (three-byte value).
- `RgbColor::from_hex(&str) -> Result<Self, WalletError>` constructor — covered by unit test ACC-1e.
- BT.601 luminance helper (D-06) lives on `RgbColor` or as a free `fn auto_foreground(bg: RgbColor) -> RgbColor` in `subject.rs` — covered by unit test ACC-1f.

**Implementation reference:** `../gestiscilo-it/app/docs/superpowers/plans/2026-05-11-wallet-passes.md` Phase A (if accessible). If not accessible, derive from spec §3.1 + D-06.

---

### `ferro-wallet/src/images.rs` (fit_to + apple_logo_set + apple_icon_set + google_hero) — NEW PATTERN

**Analog:** none. No image-pipeline crate exists in the workspace.

**Authoritative source:** spec §3.4 + D-03 + RESEARCH.md §"Code Examples" → "image 0.25 fit + centre-pad on transparent canvas".

**Public API** (spec §3.4):
```rust
pub fn fit_to(bytes: &[u8], w: u32, h: u32) -> Result<Vec<u8>, WalletError>;
pub fn apple_logo_set(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, WalletError>;
pub fn apple_icon_set(icon: Option<&[u8]>, logo_fallback: &[u8]) -> Result<Vec<(String, Vec<u8>)>, WalletError>;
pub fn google_hero(bytes: &[u8]) -> Result<Vec<u8>, WalletError>;
```

**`fit_to` body** (from RESEARCH.md §"Code Examples", verified against Context7 `/image-rs/image`):
```rust
use image::{imageops, imageops::FilterType, DynamicImage, ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;

pub fn fit_to(bytes: &[u8], w: u32, h: u32) -> Result<Vec<u8>, WalletError> {
    let src = image::load_from_memory(bytes)
        .map_err(|e| WalletError::Image(format!("decode: {e}")))?;
    let resized = src.resize(w, h, FilterType::Lanczos3).into_rgba8();
    let (rw, rh) = resized.dimensions();

    let mut canvas = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 0])); // fully transparent
    let x = ((w as i64) - (rw as i64)) / 2;
    let y = ((h as i64) - (rh as i64)) / 2;
    imageops::overlay(&mut canvas, &resized, x, y);

    let mut out = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(canvas).write_to(&mut out, ImageFormat::Png)
        .map_err(|e| WalletError::Image(format!("encode png: {e}")))?;
    Ok(out.into_inner())
}
```

**`apple_logo_set` / `apple_icon_set` dimensions** (D-03):
- logo set: `("logo.png", 160×50)`, `("logo@2x.png", 320×100)`, `("logo@3x.png", 480×150)`
- icon set: `("icon.png", 29×29)`, `("icon@2x.png", 58×58)`, `("icon@3x.png", 87×87)`
- icon-from-logo fallback: centre-square-crop the logo bytes, then `fit_to` to each icon size.

Watch RESEARCH.md Pitfall 4: `DynamicImage::resize` returns `DynamicImage`; always `.into_rgba8()` before `imageops::overlay`.

---

### `ferro-wallet/src/qr.rs` (png + data_uri) — NEW PATTERN

**Analog:** none. No QR generator in the workspace.

**Authoritative source:** spec §3.4 + RESEARCH.md §"Code Examples" → "qrcode-generator 5.0.0 PNG output".

**Public API + body** (verified against docs.rs/qrcode-generator/5.0.0):
```rust
use qrcode_generator::QrCodeEcc;
use base64::{engine::general_purpose, Engine as _};

pub fn png(data: &str, size: u32) -> Result<Vec<u8>, WalletError> {
    qrcode_generator::to_png_to_vec(data, QrCodeEcc::Medium, size as usize)
        .map_err(|e| WalletError::Qr(format!("png generate: {e}")))
}

pub fn data_uri(data: &str, size: u32) -> Result<String, WalletError> {
    let bytes = png(data, size)?;
    let b64 = general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:image/png;base64,{b64}"))
}
```

Unit test ACC-1h: assert `png(...)` output starts with the 8-byte PNG magic `[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]`.

---

### `ferro-wallet/src/apple/mod.rs` (ApplePassBuilder)

**Analog:** `ferro-stripe/src/webhook/mod.rs` lines 1–19 — multi-file submodule shape (folder + `mod.rs` + supporting files + `pub use` re-exports).

**Pattern** (`ferro-stripe/src/webhook/mod.rs` lines 1–19):
```rust
//! Stripe webhook handling — signature verification, typed event structs,
//! synchronous dispatch registry, and queue-path job.

pub mod events;
pub mod queue;
pub mod sync;
pub mod verify;

pub use events::StripeEvent;
pub use events::{
    StripeChargeDisputeCreated, StripeChargeRefunded, StripeCheckoutCompleted,
    // ...
};
pub use queue::ProcessStripeWebhook;
pub use sync::SyncDispatcher;
pub use verify::verify_webhook;
```

**Adapt for `apple/mod.rs`:**
```rust
//! Apple Wallet `.pkpass` issuance — SHA1 manifest + PKCS#7 detached signature + ZIP packaging.

pub mod manifest;
pub mod package;
pub mod sign;

use crate::config::AppleConfig;
use crate::subject::WalletSubject;
use crate::WalletError;

pub struct ApplePassBuilder {
    pass_type_id: String,
    team_id: String,
    app_name: String,
    signing: sign::SigningMaterial,
}

impl ApplePassBuilder {
    pub fn new(cfg: AppleConfig, app_name: String) -> Result<Self, WalletError> {
        let signing = sign::SigningMaterial::parse(
            &cfg.cert_pem,
            &cfg.key_pem,
            cfg.key_password.as_deref(),
            &cfg.wwdr_pem,
        )?;
        Ok(Self {
            pass_type_id: cfg.pass_type_id,
            team_id: cfg.team_id,
            app_name,
            signing,
        })
    }

    pub fn build<S: WalletSubject>(&self, s: &S) -> Result<Vec<u8>, WalletError> {
        // 1. build_pass_json(self, s) -> Vec<u8>
        // 2. resolve images (apple_logo_set + apple_icon_set)
        // 3. build_manifest(&entries) -> Vec<u8>
        // 4. self.signing.sign_detached(manifest_bytes) -> Vec<u8>
        // 5. zip_pkpass(all_entries + signature) -> Vec<u8>
        todo!()
    }
}
```

`pub use apple::ApplePassBuilder;` in `lib.rs` is restored once this body lands (D-11).

---

### `ferro-wallet/src/apple/manifest.rs` (build_pass_json + build_manifest) — partial analog

**Analog (file-in-submodule shape):** `ferro-stripe/src/webhook/verify.rs` lines 1–23 — cryptographic primitive isolated in its own file, public function returns `Result<…, Error>`.

**Pattern** (`ferro-stripe/src/webhook/verify.rs` lines 1–23):
```rust
//! Stripe webhook signature verification.

use crate::Error;

pub fn verify_webhook(
    raw_body: &str,
    signature: &str,
    secret: &str,
) -> Result<stripe::Event, Error> {
    stripe::Webhook::construct_event(raw_body, signature, secret)
        .map_err(|e| Error::WebhookVerification(e.to_string()))
}
```

**Adapt for `apple/manifest.rs`** (D-05 — manifest is JSON map of filename → lowercase-hex SHA1; reference RESEARCH.md §"Code Examples" → "sha1 0.10 manifest digest"):
```rust
use sha1::{Digest, Sha1};
use crate::WalletError;

pub(crate) fn build_pass_json<S: WalletSubject>(
    builder: &ApplePassBuilder,
    subject: &S,
) -> Result<Vec<u8>, WalletError> {
    // Compose pass.json per spec §3.1 / D-06 / D-07
    // passTypeIdentifier, teamIdentifier, serialNumber, organizationName,
    // foregroundColor / labelColor derived per D-06,
    // eventTicket.primaryFields / secondaryFields / auxiliaryFields,
    // barcodes[0] = { format: "PKBarcodeFormatQR", message: subject.barcode_token(), … }
    todo!()
}

pub(crate) fn build_manifest(entries: &[(String, Vec<u8>)]) -> Result<Vec<u8>, WalletError> {
    fn sha1_hex_lower(bytes: &[u8]) -> String {
        let mut h = Sha1::new();
        h.update(bytes);
        h.finalize().iter().map(|b| format!("{b:02x}")).collect()
    }
    let manifest = serde_json::Value::Object(
        entries.iter()
            .map(|(name, bytes)| (name.clone(), serde_json::Value::String(sha1_hex_lower(bytes))))
            .collect(),
    );
    serde_json::to_vec(&manifest)
        .map_err(|e| WalletError::ApplePackage(format!("manifest json: {e}")))
}
```

Unit test ACC-1d asserts the hex digest is lowercase per Apple's requirement.

---

### `ferro-wallet/src/apple/sign.rs` (SigningMaterial::parse + sign_detached) — NEW PATTERN

**Analog:** none. PKCS#7 detached signing has no precedent in the workspace.

**Authoritative source:** spec §3 + D-05 + RESEARCH.md §"Code Examples" → "openssl PKCS#7 detached signing (Apple signature)".

**Body** (verified against docs.rs/openssl `Pkcs7::sign`):
```rust
use openssl::pkcs7::{Pkcs7, Pkcs7Flags};
use openssl::pkey::{PKey, Private};
use openssl::stack::Stack;
use openssl::x509::X509;
use crate::WalletError;

pub(crate) struct SigningMaterial {
    pub cert: X509,
    pub key: PKey<Private>,
    pub wwdr: X509,
}

impl SigningMaterial {
    pub fn parse(
        cert_pem: &str,
        key_pem: &str,
        key_password: Option<&str>,
        wwdr_pem: &str,
    ) -> Result<Self, WalletError> {
        let cert = X509::from_pem(cert_pem.as_bytes())
            .map_err(|e| WalletError::AppleSign(format!("cert parse: {e}")))?;
        let key = match key_password {
            Some(pw) => PKey::private_key_from_pem_passphrase(key_pem.as_bytes(), pw.as_bytes()),
            None     => PKey::private_key_from_pem(key_pem.as_bytes()),
        }.map_err(|e| WalletError::AppleSign(format!("key parse: {e}")))?;
        let wwdr = X509::from_pem(wwdr_pem.as_bytes())
            .map_err(|e| WalletError::AppleSign(format!("wwdr parse: {e}")))?;
        Ok(Self { cert, key, wwdr })
    }

    pub fn sign_detached(&self, manifest_bytes: &[u8]) -> Result<Vec<u8>, WalletError> {
        let mut wwdr_stack: Stack<X509> = Stack::new()
            .map_err(|e| WalletError::AppleSign(format!("stack init: {e}")))?;
        wwdr_stack.push(self.wwdr.clone())
            .map_err(|e| WalletError::AppleSign(format!("stack push wwdr: {e}")))?;

        let flags = Pkcs7Flags::DETACHED | Pkcs7Flags::BINARY;
        let pkcs7 = Pkcs7::sign(&self.cert, &self.key, &wwdr_stack, manifest_bytes, flags)
            .map_err(|e| WalletError::AppleSign(format!("pkcs7 sign: {e}")))?;
        pkcs7.to_der()
            .map_err(|e| WalletError::AppleSign(format!("pkcs7 to_der: {e}")))
    }
}
```

Watch RESEARCH.md Pitfall 2 (WWDR must be pushed onto the Stack — empty stack silently produces a chain-less signature) and Pitfall 5 (`openssl-sys` build dep on system OpenSSL).

---

### `ferro-wallet/src/apple/package.rs` (zip_pkpass) — NEW PATTERN

**Analog:** none. No ZIP packaging in the workspace.

**Authoritative source:** spec §3 + RESEARCH.md §"Code Examples" → "zip 2.x .pkpass packaging".

**Body** (verified against Context7 `/zip-rs/zip2`):
```rust
use std::io::{Cursor, Write};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};
use crate::WalletError;

pub(crate) fn zip_pkpass(entries: &[(String, Vec<u8>)]) -> Result<Vec<u8>, WalletError> {
    let mut buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buf);
    let opts = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored);

    for (name, bytes) in entries {
        zip.start_file(name, opts)
            .map_err(|e| WalletError::ApplePackage(format!("start_file {name}: {e}")))?;
        zip.write_all(bytes)
            .map_err(|e| WalletError::ApplePackage(format!("write {name}: {e}")))?;
    }
    zip.finish().map_err(|e| WalletError::ApplePackage(format!("finish: {e}")))?;
    Ok(buf.into_inner())
}
```

Critical: `CompressionMethod::Stored` per RESEARCH.md Pitfall 1 — Apple Wallet rejects deflated entries.

---

### `ferro-wallet/src/google/mod.rs` (GoogleWalletBuilder)

**Analog:** `ferro-stripe/src/webhook/mod.rs` lines 1–19 (same submodule shape as `apple/mod.rs`).

**Adapt:**
```rust
//! Google Wallet save-link issuance — RS256 JWT pointing at an eventTicketObject.

pub mod jwt;
pub mod object;

use crate::config::GoogleConfig;
use crate::subject::WalletSubject;
use crate::WalletError;

pub struct GoogleWalletBuilder {
    issuer_id: String,
    service_account_email: String,
    private_key_pem: String,
    app_name: String,
    app_url: String,
}

impl GoogleWalletBuilder {
    pub fn new(cfg: GoogleConfig, app_name: String, app_url: String) -> Result<Self, WalletError> {
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

---

### `ferro-wallet/src/google/object.rs` (build_event_ticket_object) — partial analog

**Analog:** `ferro-stripe/src/checkout.rs` (builder that composes a JSON-shaped request).

**Pattern (loose):** function takes a typed `WalletSubject` and emits a `serde_json::Value` matching the Google Wallet REST schema. Compose the object per spec §3 + D-07:
- `id = "{issuer_id}.{subject.serial()}"`
- `classId = "{issuer_id}.{pass_type_id_with_dots_replaced_by_underscores}"` (pass type id default = `"booking"` per D-07)
- `state = "active"`
- `barcode = { type: "qrCode", value: subject.barcode_token() }`
- `ticketHolderName / eventName / venue / dateTime` derived from subject fields per spec.

Reference shape lives in `../gestiscilo-it/app/docs/superpowers/plans/2026-05-11-wallet-passes.md` Phase A if accessible; otherwise build directly from spec §3 against `developers.google.com/wallet/tickets/events/rest`.

---

### `ferro-wallet/src/google/jwt.rs` (sign_save_jwt + save_url + pass_type_id_default) — NEW PATTERN

**Analog:** `ferro-stripe/src/webhook/verify.rs` is the closest precedent (crypto-in-its-own-file, returns `Result<…, Error>`) but uses HMAC-SHA256 verify, not RS256 sign.

**Authoritative source:** spec §3 + D-08 + RESEARCH.md §"Code Examples" → "jsonwebtoken RS256 encode".

**Body** (verified against Context7 `/keats/jsonwebtoken`):
```rust
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;
use crate::WalletError;

pub const fn pass_type_id_default() -> &'static str { "booking" }

#[derive(Serialize)]
struct SaveClaims<'a> {
    iss: &'a str,
    aud: &'a str,
    typ: &'a str,
    iat: i64,
    origins: Vec<&'a str>,
    payload: serde_json::Value,
}

pub(crate) fn sign_save_jwt(
    builder: &GoogleWalletBuilder,
    event_ticket_object: serde_json::Value,
) -> Result<String, WalletError> {
    let claims = SaveClaims {
        iss: &builder.service_account_email,
        aud: "google",
        typ: "savetowallet",
        iat: chrono::Utc::now().timestamp(),
        origins: vec![&builder.app_url],
        payload: serde_json::json!({ "eventTicketObjects": [event_ticket_object] }),
    };
    let header = Header::new(Algorithm::RS256);
    let key = EncodingKey::from_rsa_pem(builder.private_key_pem.as_bytes())
        .map_err(|e| WalletError::GoogleJwt(format!("private key parse: {e}")))?;
    encode(&header, &claims, &key)
        .map_err(|e| WalletError::GoogleJwt(format!("encode: {e}")))
}

pub fn save_url(jwt: &str) -> String {
    format!("https://pay.google.com/gp/v/save/{jwt}")
}
```

Unit test ACC-1i: `save_url("abc.def.ghi") == "https://pay.google.com/gp/v/save/abc.def.ghi"`.

---

### `ferro-wallet/tests/apple_integration.rs` (end-to-end with runtime-minted self-signed cert)

**Analog (test layout):** `ferro-stripe/tests/parser_contract.rs` lines 1–17 — integration test layout, `use ferro_…::…` imports, helper function for fixture parsing.

**Pattern** (`ferro-stripe/tests/parser_contract.rs` lines 1–17):
```rust
//! Parser-contract integration tests — asserts every `StripeEvent::from_raw`
//! implementation extracts fields correctly from its golden-JSON fixture, …

use std::collections::HashMap;

use ferro_stripe::{
    StripeChargeDisputeCreated, /* … */
};

fn parse_event(raw: &str) -> stripe::Event {
    serde_json::from_str::<stripe::Event>(raw).expect("fixture should deserialize as stripe::Event")
}
```

**Adapt** (per D-09 — no fixtures, mint crypto at runtime; reference RESEARCH.md §"Code Examples" → "Self-signed X.509 + RSA keypair at test runtime"):
```rust
//! End-to-end Apple .pkpass integration — mints a self-signed cert at runtime,
//! builds a .pkpass from a StubBooking WalletSubject, asserts ZIP shape + pass.json fields.

use ferro_wallet::{
    ApplePassBuilder, AppleConfig, Branding, Field, FieldAlignment, PassKind, RgbColor,
    TextColorMode, WalletSubject,
};
use openssl::asn1::Asn1Time;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use openssl::x509::{X509Builder, X509NameBuilder};

fn mint_self_signed() -> (String, String) {
    // … see RESEARCH.md §"Code Examples" — full body inlined
}

struct StubBooking;
impl WalletSubject for StubBooking { /* … */ }

#[test]
fn build_pkpass_produces_valid_zip_and_pass_json() {
    let (cert_pem, key_pem) = mint_self_signed();
    let cfg = AppleConfig {
        pass_type_id: "pass.com.example.test".into(),
        team_id: "TEAMID1234".into(),
        cert_pem: cert_pem.clone(),
        key_pem,
        key_password: None,
        wwdr_pem: cert_pem, // reuse self-signed as WWDR for test only
    };
    let builder = ApplePassBuilder::new(cfg, "Test App".into()).unwrap();
    let bytes = builder.build(&StubBooking).unwrap();

    // 1. Assert ZIP contains 9 expected files (per ACC-1j):
    //    pass.json, manifest.json, signature,
    //    logo.png, logo@2x.png, logo@3x.png,
    //    icon.png, icon@2x.png, icon@3x.png
    // 2. Parse pass.json — assert passTypeIdentifier, teamIdentifier, serialNumber,
    //    barcode.message, eventTicket.primaryFields[0].value
}
```

Watch RESEARCH.md Pitfall 6 (no env-pollution from tests) and the fact that openssl-sys requires system OpenSSL (Pitfall 5).

---

### `ferro-wallet/tests/google_jwt.rs` (RS256 roundtrip)

**Analog:** same as `tests/apple_integration.rs` — `ferro-stripe/tests/parser_contract.rs` layout.

**Adapt** (per D-09 + RESEARCH.md Pitfall 3 — decode validation must disable `exp` requirement):
```rust
use ferro_wallet::{GoogleConfig, GoogleWalletBuilder, /* StubBooking deps */};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use std::collections::HashSet;

#[test]
fn save_jwt_roundtrips_with_runtime_minted_rsa_keypair() {
    let rsa = Rsa::generate(2048).unwrap();
    let pkey = PKey::from_rsa(rsa.clone()).unwrap();
    let private_pem = String::from_utf8(pkey.private_key_to_pem_pkcs8().unwrap()).unwrap();
    let public_pem  = String::from_utf8(pkey.public_key_to_pem().unwrap()).unwrap();

    let cfg = GoogleConfig {
        issuer_id: "3388000000000000000".into(),
        service_account_email: "sa@example.iam.gserviceaccount.com".into(),
        service_account_private_key_pem: private_pem,
    };
    let builder = GoogleWalletBuilder::new(cfg, "Test App".into(), "https://example.com".into()).unwrap();
    let jwt = builder.save_jwt(&StubBooking).unwrap();

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = false;
    validation.required_spec_claims = HashSet::new(); // ← Pitfall 3
    validation.set_audience(&["google"]);

    let decoded = decode::<serde_json::Value>(
        &jwt,
        &DecodingKey::from_rsa_pem(public_pem.as_bytes()).unwrap(),
        &validation,
    ).unwrap();

    assert_eq!(decoded.claims["typ"], "savetowallet");
    assert_eq!(decoded.claims["aud"], "google");
    // … assert payload.eventTicketObjects[0].id + barcode.value
}

#[test]
fn save_url_returns_pay_google_com_prefix() {
    // builder.save_url(&StubBooking).unwrap().starts_with("https://pay.google.com/gp/v/save/")
}
```

---

## Shared Patterns

### Error variant prefix convention

**Source:** `ferro-stripe/src/error.rs` lines 5, 9, 13, 17 — every variant's `#[error("…")]` string begins with a lowercase, name-prefixed token.

**Apply to:** every `WalletError` `#[error]` string per D-04.

```rust
#[error("config: {0}")]
Config(String),
#[error("apple sign: {0}")]
AppleSign(String),
// …
```

### `thiserror = "2"` derive

**Source:** `ferro-stripe/Cargo.toml` line 20, `ferro-whatsapp/Cargo.toml` line 20.

**Apply to:** `ferro-wallet/Cargo.toml` — match `thiserror = "2"` (RESEARCH.md notes 4 crates still on `"1.0"` — do **not** follow those; the convention has moved to 2).

### Env-pollution-safe test pattern

**Source:** `ferro-stripe/src/config.rs` lines 54–71 — save existing var, `remove_var`, assert, restore.

**Apply to:** every `#[cfg(test)] mod tests` block in `ferro-wallet/src/config.rs` that touches `APPLE_WALLET_*` or `GOOGLE_WALLET_*` or `APP_NAME` / `APP_URL` env vars. RESEARCH.md Pitfall 6 elevates this from "nice to have" to mandatory.

### `version.workspace = true`

**Source:** `ferro-whatsapp/Cargo.toml` line 3 + `ferro-ai/Cargo.toml` line 3 — the **current** convention.

**Do NOT follow:** `ferro-stripe/Cargo.toml` line 3 (`version = "0.5.0"`) — ferro-stripe diverged historically and is the wrong template per RESEARCH.md §"Pattern Alignment".

**Apply to:** `ferro-wallet/Cargo.toml` — tracks workspace `0.2.x` bumps.

### Submodule shape (folder + `mod.rs` + supporting files + `pub use`)

**Source:** `ferro-stripe/src/webhook/mod.rs` lines 1–19.

**Apply to:** `ferro-wallet/src/apple/mod.rs` and `ferro-wallet/src/google/mod.rs` — both ship a folder with `mod.rs` that declares its sub-files and re-exports the public builder type.

---

## Workspace Edits — Exact Insertion Points

### Edit 1: `Cargo.toml` workspace `[workspace] members` append

**File:** `/Users/alberto/repositories/albertogferrario/ferro/Cargo.toml`
**Current state lines 1–24** (members array; **non-alphabetical** — phase-introduction order):

```toml
[workspace]
resolver = "2"
members = [
    "framework",
    "app",
    "ferro-cli",
    "ferro-macros",
    "ferro-events",
    "ferro-queue",
    "ferro-notifications",
    "ferro-broadcast",
    "ferro-storage",
    "ferro-cache",
    "ferro-mcp",
    "ferro-inertia",
    "ferro-json-ui",
    "ferro-lang",
    "ferro-api-mcp",
    "ferro-projections",
    "ferro-stripe",
    "ferro-theme",
    "ferro-ai",
    "ferro-whatsapp",         ← current last member, line 23
]
```

**Required edit:** append `"ferro-wallet",` after line 23. Order is not alphabetical; preserve phase-introduction-order.

**Resulting array (line 23 → 24):**

```toml
    "ferro-whatsapp",
    "ferro-wallet",
]
```

### Edit 2: `.github/workflows/publish.yml` Wave 1a append

**File:** `/Users/alberto/repositories/albertogferrario/ferro/.github/workflows/publish.yml`
**Current state line 201:**

```yaml
          WAVE1A_CRATES="ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp"
```

**Rationale:** `ferro-wallet` is a **leaf crate** (zero internal `ferro-*` deps per spec §5: "No dependency on `framework` — the crate stays pure"). It belongs in Wave 1a, not 1b. Verification: Wave 1b at line 236 lists `ferro-ai ferro-projections ferro-stripe ferro-whatsapp ferro-notifications` — all have internal deps; `ferro-wallet` has none.

**Resulting line 201:**

```yaml
          WAVE1A_CRATES="ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp ferro-wallet"
```

### Edit 3 (PLAN-09 only): `Cargo.toml` `[workspace.package] version` patch bump

**File:** `/Users/alberto/repositories/albertogferrario/ferro/Cargo.toml`
**Current state lines 26–27:**

```toml
[workspace.package]
version = "0.2.23"
```

**PLAN-09 edit:** bump to `"0.2.24"`. Note: RESEARCH.md observes the GH Actions `check-version` job auto-bumps the patch if the current version is already tagged, so manual bump may be redundant; PLAN-09 should check whether `v0.2.23` is already tagged before manually bumping.

---

## No Analog Found — Spec Is Authoritative

The following files have no close analog in the workspace. The planner must build them directly from the spec (`docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md` §3) and the verified code samples in RESEARCH.md §"Code Examples". The downstream reference `../gestiscilo-it/app/docs/superpowers/plans/2026-05-11-wallet-passes.md` Phase A (if accessible) is the field-test origin and contains full reference code.

| File | Reason no analog | Authoritative source |
|------|------------------|----------------------|
| `ferro-wallet/src/subject.rs` | No user-implementable domain trait elsewhere in `ferro-*` crates | spec §3.1 |
| `ferro-wallet/src/images.rs` | No image pipeline in the workspace | spec §3.4 + D-03 + RESEARCH.md §"Code Examples" (image 0.25 fit + centre-pad) |
| `ferro-wallet/src/qr.rs` | No QR generator in the workspace | spec §3.4 + RESEARCH.md §"Code Examples" (qrcode-generator 5.0.0) |
| `ferro-wallet/src/apple/sign.rs` | No PKCS#7 signing elsewhere | spec §3 + D-05 + RESEARCH.md §"Code Examples" (openssl PKCS#7) |
| `ferro-wallet/src/apple/package.rs` | No ZIP packaging elsewhere | spec §3 + RESEARCH.md §"Code Examples" (zip 2.x) |
| `ferro-wallet/src/google/jwt.rs` | No RS256 JWT signing elsewhere (Stripe webhook verify uses HMAC-SHA256, not RS256) | spec §3 + D-08 + RESEARCH.md §"Code Examples" (jsonwebtoken RS256) |

---

## Metadata

**Analog search scope:** `ferro-stripe/`, `ferro-whatsapp/`, `ferro-ai/`, `ferro-inertia/`, `framework/src/config/`, root `Cargo.toml`, `.github/workflows/publish.yml`
**Files scanned (Read):** 11
**Pattern extraction date:** 2026-05-11
**Linked artifacts:**
- `.planning/phases/151-ferro-wallet-crate/151-CONTEXT.md` (decisions D-01..D-11)
- `.planning/phases/151-ferro-wallet-crate/151-RESEARCH.md` (§"Pattern Alignment with ferro-stripe" + §"Code Examples" + §"Common Pitfalls")
- `docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md` (§3 public API, §5 deps, §6 error type)

## PATTERN MAPPING COMPLETE
