# Phase 151: ferro-wallet — Research

**Researched:** 2026-05-11
**Domain:** New project-agnostic Rust crate for digital wallet pass issuance (Apple `.pkpass` + Google Wallet save-links)
**Confidence:** HIGH (CONTEXT.md is design-doc-grade; this research is verification-oriented)

## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01: Two builders, deliberately separate.** Apple PKCS#7-over-ZIP vs. Google RS256-JWT-pointing-at-JSON share nothing at the wire-format level. `WalletSubject` trait is the only shared abstraction.
- **D-02: Permissive `WalletConfig::from_env`.** Missing Apple cluster ⇒ `apple: None`; missing Google cluster ⇒ `google: None`. `APP_NAME` / `APP_URL` fall back to `"Ferro Application"` / `"http://localhost:8080"` (matches `framework::config::AppConfig`).
- **D-03: Image pipeline.** `fit_to` = resize-preserve-aspect + centre-pad on transparent canvas, encoded PNG. `apple_logo_set` emits 160×50 / 320×100 / 480×150. `apple_icon_set` accepts optional explicit icon or derives via centre-square-crop + resize to 29 / 58 / 87.
- **D-04: `WalletError`.** `thiserror`-derived; each variant prefixes its `Display` with its name (`"config: …"`, `"apple sign: …"`, `"google jwt: …"`). `Io(#[from] std::io::Error)` for plumbing.
- **D-05: Apple manifest + signature.** SHA1 manifest as JSON map; PKCS#7 detached over manifest bytes with `Pkcs7Flags::DETACHED | Pkcs7Flags::BINARY`; WWDR intermediate pushed onto a single-element `Stack<X509>`; output goes into ZIP as `signature` (no extension).
- **D-06: BT.601 luminance threshold.** Auto foreground colour: luminance `< 0.5` ⇒ white, `>= 0.5` ⇒ dark slate `rgb(17,24,39)`. `Light` / `Dark` force white / black. `labelColor` always tracks `foregroundColor` in v1.
- **D-07: Google class/object ID format.** `class_id = "{issuer_id}.{pass_type_id_with_dots_replaced_by_underscores}"`; `object_id = "{issuer_id}.{subject.serial()}"`; fixed `"booking"` suffix via `pass_type_id_default()` const.
- **D-08: RS256 JWT shape.** Claims = `{iss, aud="google", typ="savetowallet", iat, origins:[app_url], payload:{eventTicketObjects:[…]}}`. `save_url(jwt) = "https://pay.google.com/gp/v/save/{jwt}"`.
- **D-09: Self-signed crypto at test runtime.** Apple integration test mints a self-signed X.509 via `openssl`, reuses it as both signing cert and WWDR. Google integration test mints an RSA keypair, signs, decodes with public key. No real Apple/Google credentials in CI.
- **D-10: Workspace + auto-publish.** Crate lives at `ferro/ferro-wallet/`; added to `[workspace] members` in workspace root `Cargo.toml`; workspace version patch-bumps at phase verification; release auto-publishes via existing GitHub Actions workflow on push to master.
- **D-11: lib.rs scaffold order.** Stub all module files with `// placeholder` lines in Task 01. Temporarily strip `pub use apple::ApplePassBuilder;` / `pub use google::GoogleWalletBuilder;` re-exports until the builder body lands; restore in the same plan that lands the body.

### Claude's Discretion

- Internal API ergonomics (helper function shapes, internal module split inside `apple/` and `google/`).
- README copy, doctests, error-message wording (within D-04 prefix convention).
- Workspace member alphabetical positioning within `[workspace] members` array.

### Deferred Ideas (OUT OF SCOPE)

- Apple Web Service Protocol (live updates, `passesUpdatedSince`, APNs push, Express Mode device registration).
- Google `objects.patch` API (live updates).
- Locale-aware label resolution beyond raw string passthrough.
- Pass kinds beyond `EventTicket` (Generic / Coupon / Boarding / StoreCard declared in `PassKind` but un-tested in v1).
- Live SVG preview of the rendered card.

## Project Constraints (from CLAUDE.md)

- **Architecture Principle 6 — project-agnostic crates.** No hardcoded app identity. `ferro-wallet` reads `APP_NAME` / `APP_URL` via its own `WalletConfig::from_env` (mirroring `ferro-inertia::InertiaConfig::app_name` and `ferro-stripe::StripeConfig::from_env`). Reviewers must reject hardcoded strings like `"gestiscilo"`, `"Ferro Application"`, `"https://example.com"` inside the crate. [VERIFIED: CLAUDE.md §"Architecture Principles" #6]
- **Pre-commit gate.** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` before every commit. CI enforces `-D warnings`. [VERIFIED: CLAUDE.md §"Testing & Linting"]
- **`--all-targets` required** — catches issues in test code that `--all` alone misses. [VERIFIED: CLAUDE.md]
- **One Error enum per crate** with `thiserror` derive (workspace convention).
- **No co-author attribution** in commits. [VERIFIED: CLAUDE.md]
- **No deprecation, no versioned names** — always a feature branch. [VERIFIED: ~/.claude/CLAUDE.md]
- **MEMORY.md note:** when adding a new crate to the workspace, always add it to `.github/workflows/publish.yml` in the correct wave. [VERIFIED: ~/.claude/.../MEMORY.md "Key Conventions"]

## Summary

Phase 151 ships a new project-agnostic crate `ferro-wallet` that exposes two builders (`ApplePassBuilder`, `GoogleWalletBuilder`) plus image / QR helpers, gated behind the `WalletSubject` trait. CONTEXT.md is design-doc-grade — eleven locked decisions (D-01..D-11), the file structure, the suggested wave decomposition, the canonical reference set, and the implementation-reference pointer to the downstream gestiscilo plan are all already settled. The spec in `docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md` defines the public API surface, the dependency set, the error variants, the test strategy, and the release flow.

Because of this, the research below is **verification-oriented, not exploratory**. It confirms (a) which of the spec's dependencies are already present in the workspace and at what versions, (b) the exact workspace integration points (`Cargo.toml` members entry + `.github/workflows/publish.yml` wave assignment), (c) the side-by-side pattern alignment between each `ferro-wallet` module and its `ferro-stripe` analog, (d) that the third-party APIs the builders rely on (openssl PKCS#7, jsonwebtoken RS256, zip SimpleFileOptions, image RgbaImage + imageops::overlay, qrcode-generator to_png_to_vec) match the shape the spec assumes, and (e) the open risks the planner must resolve in PLAN-01 before code lands.

**Primary recommendation:** Treat the suggested wave decomposition in CONTEXT.md `<task_breakdown>` (151-01 through 151-09) as the planning skeleton. PLAN-01 must complete the workspace edits (`Cargo.toml` `[workspace] members` append, `.github/workflows/publish.yml` Wave 1a list extension) atomically with the scaffold so `cargo check --workspace` stays green between plans.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `WalletSubject` trait + value types | Library / Domain Contract | — | Pure types, no I/O; downstream models implement it for their domain object |
| `WalletConfig::from_env` | Library / Configuration | — | Environment-driven config; mirrors `framework::config::AppConfig::from_env` |
| Apple `.pkpass` signing | Library / Cryptography | Library / Packaging (ZIP) | PKCS#7 detached signature is the security-critical primitive; ZIP is the carrier |
| Apple `.pkpass` packaging | Library / Packaging | — | Pure data transform; SHA1 manifest + ZIP assembly |
| Google save-link JWT | Library / Cryptography | — | RS256 signing of a fixed claim shape |
| Image normalisation | Library / Media | — | Pure transformation, no I/O; `image` crate + `imageops::overlay` |
| QR generation | Library / Media | — | Pure transformation; `qrcode-generator` PNG output |
| Workspace integration | Build / Release | CI / Publish | Cargo workspace member registration + GH Actions wave assignment |

The crate occupies the **library** tier exclusively. It has no HTTP, database, or framework dependency — the spec is explicit on this (§5: "No dependency on `framework` — the crate stays pure").

## Phase Requirements

Phase 151 has no specific `phase_req_ids` — `REQUIREMENTS.md` was not found at the expected path (`.planning/REQUIREMENTS.md`) and `phase_req_ids: null` per the orchestrator. The phase is roadmap-driven (v11.10 milestone in `.planning/ROADMAP.md` line 40) and traces to the gestiscilo-it digital wallet booking pass field test. The spec's §9 acceptance criteria function as the verifiable requirements:

| Implicit ID | Description | Research Support |
|----|-------------|------------------|
| ACC-1 | `cargo test -p ferro-wallet` is green (all unit + integration tests) | Validation Architecture §Behaviour |
| ACC-2 | `cargo build --workspace` is green with the new crate in `members` | Workspace Integration §`Cargo.toml` edit |
| ACC-3 | `cargo doc --no-deps -p ferro-wallet` produces clean output | Validation Architecture §Build/Lint |
| ACC-4 | Downstream consumer (gestiscilo) can depend on `ferro-wallet = "0.2.X"` after auto-publish | Workspace Integration §`publish.yml` Wave 1a |

## Standard Stack

### Core

| Library | Version pin (spec §5) | Latest stable (2026-05-11) | Already in workspace? | Why standard |
|---------|----------|----------------------------|------------------------|--------------|
| `openssl` | `"0.10"` | 0.10.79 | Transitive via `native-tls` (locked at 0.10.75) — **no direct consumer** | Only mainstream Rust crate exposing PKCS#7 detached signing; required for Apple `.pkpass` signature [VERIFIED: docs.rs/openssl `Pkcs7::sign`] |
| `zip` | `"2"` | 2.4.2 (line 2.x); 9.0.0-pre1 exists but pre-release | **Not in workspace** | Standard Rust ZIP library; spec pins to 2.x stable line. `SimpleFileOptions::default().compression_method(Stored)` is the right shape for `.pkpass` (Apple recommends stored, not deflated) [VERIFIED: Context7 `/zip-rs/zip2`] |
| `jsonwebtoken` | `"9"` | 9.3.1 (line 9.x); 10.3.0 latest | **Not in workspace** | De-facto JWT library for Rust; `Algorithm::RS256` + `EncodingKey::from_rsa_pem` + `encode(...)` is the exact 3-call surface the spec uses [VERIFIED: Context7 `/keats/jsonwebtoken`] |
| `image` | `"0.25"` | 0.25.10 | **Not in workspace** | Standard pure-Rust image processing crate; supports D-03's required operations: `DynamicImage::resize(w, h, FilterType::Lanczos3)`, `RgbaImage::new(w, h)` (transparent canvas), `imageops::overlay(&mut canvas, &resized, x, y)` (centre-pad), `write_to(&mut buf, ImageFormat::Png)` [VERIFIED: Context7 `/image-rs/image`] |
| `qrcode-generator` | `"5"` | 5.0.0 | **Not in workspace** | `qrcode_generator::to_png_to_vec(data, QrCodeEcc::Medium, size) -> Result<Vec<u8>, QRCodeError>` returns PNG bytes directly — no intermediate matrix-to-pixel step needed [VERIFIED: docs.rs/qrcode-generator/5.0.0] |
| `sha1` | `"0.10"` | 0.10.6 (workspace-locked) | **Yes, transitive at 0.10.6** | RustCrypto crate for the SHA1 manifest digest required by Apple's `manifest.json` format [VERIFIED: Cargo.lock] |
| `base64` | `"0.22"` | 0.22.1 (workspace-locked) | **Yes** (used in `ferro-notifications`, `framework`) | QR data-URI encoding |

### Supporting

| Library | Version | Workspace pin | Use case |
|---------|---------|----------------|----------|
| `serde` | `"1"` features `["derive"]` | Already used everywhere | Field/Branding/Config serialisation |
| `serde_json` | `"1"` | Already used everywhere | `pass.json` / `manifest.json` / JWT payload construction |
| `thiserror` | `"2"` | Used in 10 crates (notes: 4 crates still on `"1.0"` — `framework`, `ferro-cache`, `ferro-storage`, `ferro-projections`) | `WalletError` derive |
| `chrono` | `"0.4"` features `["serde"]` | Used in 8 crates | `DateTime<Utc>` for `relevant_at` / `expires_at` / `iat` claim |

### Alternatives Considered

| Instead of | Could Use | Tradeoff | Locked? |
|------------|-----------|----------|---------|
| `qrcode-generator` | `qrcode` (8.6× more snippets in Context7) | `qrcode` requires a separate rendering step (matrix → image); `qrcode-generator` ships `to_png_to_vec` as a one-call API | Yes — D-spec §5 |
| `openssl` | RustCrypto `cms` crate or `rasn-pkix` | Pure-Rust alternatives exist for CMS/PKCS#7 but are immature; `openssl` is the only crate with a stable `Pkcs7::sign` API surface | Yes — D-spec §5 |
| `zip = "2"` | `zip = "9.0.0-pre1"` | Pre-release; spec pins stable | Yes — D-spec §5 |
| `jsonwebtoken = "9"` | `jsonwebtoken = "10"` | v10 is current latest; v9 is stable + widely deployed; choosing v10 introduces breaking-change risk unnecessarily for an RS256-only use case | Yes — D-spec §5 |

**Installation (new direct deps, ordered by appearance):**
```bash
# No npm here — these go into ferro-wallet/Cargo.toml [dependencies]
# openssl = "0.10"
# zip = "2"
# jsonwebtoken = "9"
# image = "0.25"
# qrcode-generator = "5"
# sha1 = "0.10"
# base64 = "0.22"
# serde = { version = "1", features = ["derive"] }
# serde_json = "1"
# thiserror = "2"
# chrono = { version = "0.4", features = ["serde"] }
```

**Version verification (run before writing `Cargo.toml` in PLAN-01):**
```bash
cargo search openssl --limit 1
cargo search zip --limit 1
cargo search jsonwebtoken --limit 1
cargo search image --limit 1
cargo search qrcode-generator --limit 1
```
Spec versions verified against crates.io 2026-05-11. [VERIFIED: crates.io API]

## Architecture Patterns

### System Architecture Diagram

```
                    ┌─────────────────────┐
                    │   Consumer crate    │
                    │  (gestiscilo, app)  │
                    │  impl WalletSubject │
                    └──────────┬──────────┘
                               │
                ┌──────────────┴──────────────┐
                │                              │
                ▼                              ▼
┌───────────────────────────┐    ┌───────────────────────────┐
│   ApplePassBuilder        │    │   GoogleWalletBuilder     │
│   ──────────────────      │    │   ────────────────────    │
│   1. build_pass_json      │    │   1. build_event_ticket   │
│      (subject → JSON)     │    │      _object (subject     │
│   2. images::apple_logo   │    │      → JSON)              │
│      _set + icon_set      │    │   2. sign_save_jwt        │
│   3. build_manifest       │    │      (RS256 over claims)  │
│      (SHA1 hex per file)  │    │   3. save_url(jwt)        │
│   4. SigningMaterial::    │    │                           │
│      sign_detached        │    │   Output:                 │
│      (PKCS#7)             │    │   "https://pay.google.com │
│   5. zip_pkpass           │    │   /gp/v/save/{jwt}"       │
│      (Stored compression) │    │                           │
│                           │    └───────────────────────────┘
│   Output: Vec<u8>         │
│   (the .pkpass bytes)     │
└───────────────────────────┘
                │                              │
                └──────────────┬───────────────┘
                               │
                               ▼
                ┌──────────────────────────────┐
                │   End-user device wallet     │
                │   (iOS Wallet / Google Pay)  │
                └──────────────────────────────┘
```

**Shared path:** `WalletSubject` trait → builders extract `Field` / `Branding` / `serial()` / `barcode_token()` etc.
**Apple-specific path:** SHA1 manifest → PKCS#7 detached signature → ZIP (Stored compression).
**Google-specific path:** Event-ticket JSON object → RS256 JWT → URL concatenation.

### Component Responsibilities

| File | Responsibility | API surface (exported from `lib.rs`) |
|------|---------------|----------------------------------------|
| `lib.rs` | Crate root + re-exports | `WalletConfig`, `AppleConfig`, `GoogleConfig`, `WalletSubject`, `Field`, `Branding`, `PassKind`, `GeoPoint`, `RgbColor`, `TextColorMode`, `FieldAlignment`, `ApplePassBuilder`, `GoogleWalletBuilder`, `WalletError` |
| `error.rs` | `WalletError` enum | `WalletError` |
| `subject.rs` | Trait + value types | `WalletSubject`, all value types listed above |
| `config.rs` | Env-driven config | `WalletConfig::from_env`, `AppleConfig`, `GoogleConfig` |
| `images.rs` | Image normalisation | `fit_to`, `apple_logo_set`, `apple_icon_set`, `google_hero` |
| `qr.rs` | QR generation | `png`, `data_uri` |
| `apple/mod.rs` | Apple builder facade | `ApplePassBuilder::new`, `ApplePassBuilder::build` |
| `apple/manifest.rs` | `pass.json` + `manifest.json` construction | `build_pass_json` (crate-internal), `build_manifest` |
| `apple/sign.rs` | PKCS#7 detached signing | `SigningMaterial::parse`, `SigningMaterial::sign_detached` |
| `apple/package.rs` | ZIP assembly | `zip_pkpass` |
| `google/mod.rs` | Google builder facade | `GoogleWalletBuilder::new`, `save_jwt`, `save_url` |
| `google/object.rs` | Event-ticket JSON object | `build_event_ticket_object` |
| `google/jwt.rs` | RS256 JWT + save-URL helpers | `sign_save_jwt`, `save_url`, `pass_type_id_default` |
| `tests/apple_integration.rs` | End-to-end Apple test | (test only) |
| `tests/google_jwt.rs` | RS256 roundtrip test | (test only) |

### Recommended Project Structure

```
ferro-wallet/
├── Cargo.toml                  # version.workspace=true (matches ferro-whatsapp/ferro-ai)
├── README.md                   # short, mirrors ferro-stripe/README.md (~10 lines)
└── src/
    ├── lib.rs                  # re-exports + crate-level docs
    ├── error.rs                # WalletError
    ├── subject.rs              # WalletSubject trait + value types
    ├── config.rs               # WalletConfig + Apple/GoogleConfig + from_env
    ├── images.rs               # fit_to + apple_logo_set + apple_icon_set + google_hero
    ├── qr.rs                   # png + data_uri
    ├── apple/
    │   ├── mod.rs              # ApplePassBuilder
    │   ├── manifest.rs         # build_pass_json + build_manifest
    │   ├── sign.rs             # SigningMaterial::parse + sign_detached
    │   └── package.rs          # zip_pkpass
    └── google/
        ├── mod.rs              # GoogleWalletBuilder
        ├── object.rs           # build_event_ticket_object
        └── jwt.rs              # sign_save_jwt + save_url + pass_type_id_default
tests/
├── apple_integration.rs
└── google_jwt.rs
```

### Anti-Patterns to Avoid

- **Unified `WalletBuilder`.** Apple and Google share nothing at the wire-format level. A merged builder would obscure format-specific failure modes and gain no shared code. (D-01.)
- **Hard-erroring `from_env`.** Missing wallet env vars must never panic or fail. Apple-only or Google-only deployments are first-class. (D-02.)
- **Hardcoded app identity.** `"Ferro Application"` literal, `"https://example.com"`, or any tenant string baked into the crate. (CLAUDE.md Architecture Principle 6.)
- **Real Apple WWDR / Google service-account secrets in tests.** CI must mint its own crypto at runtime. (D-09.)
- **Deflated `.pkpass` ZIP entries.** Apple recommends `Stored` (no compression) for `.pkpass` payloads — `SimpleFileOptions::default().compression_method(CompressionMethod::Stored)` per zip 2.x API.
- **`labelColor` drift from `foregroundColor`.** v1 keeps them locked together (D-06).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| PKCS#7 detached signing | Custom ASN.1 encoder | `openssl::pkcs7::Pkcs7::sign` with `Pkcs7Flags::DETACHED \| BINARY` | DER encoding, signed-attribute construction, and certificate-chain handling are subtle; openssl gets it right [VERIFIED: docs.rs/openssl] |
| RS256 JWT | Manual JSON + RSA sign + base64url | `jsonwebtoken::encode(&Header::new(Algorithm::RS256), &claims, &EncodingKey::from_rsa_pem(...))` | base64url-without-padding, JSON canonical form, signature framing — handled |
| ZIP assembly | Manual ZIP local headers + central directory | `zip::ZipWriter::new(...) + start_file(...) + write_all(...) + finish()` | Local file headers, central directory, end-of-central-directory record are non-trivial to get byte-correct |
| Image resize + centre-pad | Manual pixel-by-pixel loop | `image::DynamicImage::resize(w, h, FilterType::Lanczos3) + RgbaImage::new + imageops::overlay` | Lanczos3 vs. nearest-neighbour matters for logo quality; transparent-canvas overlay handles alpha correctly |
| QR PNG generation | Manual matrix → pixel → PNG | `qrcode_generator::to_png_to_vec(data, QrCodeEcc::Medium, size)` | Reed-Solomon + masking + PNG encoding in a single call |
| SHA1 digest | Manual block processing | `sha1::Sha1::new() + update + finalize` | Standard digest — already at 0.10.6 in workspace |
| `from_env` parser | Hand-rolled env var dispatch | Pattern from `ferro_stripe::config::StripeConfig::from_env` + `framework::config::env()` | Established workspace convention |

**Key insight:** Every component in `ferro-wallet` maps cleanly onto a single mature library call. The crate's job is composition (subject → builder pipeline → bytes), not cryptographic primitives. Hand-rolling any of the above would re-introduce well-known bug classes (PKCS#7 ASN.1 corner cases, ZIP CRC mistakes, PNG color-space confusion) for zero benefit.

## Pattern Alignment with ferro-stripe

`ferro-wallet` should read like `ferro-stripe`'s sibling. Side-by-side mapping:

### `Cargo.toml` `[package]` header

**ferro-stripe** (note: ferro-stripe is *not* on `version.workspace = true`; ferro-whatsapp / ferro-ai are):

```toml
[package]
name = "ferro-stripe"
version = "0.5.0"                                # ferro-stripe diverged — uses own version
edition.workspace = true
license.workspace = true
description = "Stripe payment integration for the Ferro framework"
repository = "https://github.com/albertogferrario/exo/ferro"
keywords = ["stripe", "payments", "billing", "subscriptions", "ferro"]
categories = ["web-programming"]
readme = "README.md"
```

**ferro-wallet recommended (matches the newer ferro-whatsapp / ferro-ai pattern, NOT ferro-stripe — workspace version tracking is the current convention):**

```toml
[package]
name = "ferro-wallet"
version.workspace = true                         # tracks workspace 0.2.x bumps
edition.workspace = true
license.workspace = true
description = "Digital wallet pass issuance (Apple .pkpass + Google Wallet) for the Ferro framework"
repository = "https://github.com/albertogferrario/ferro"
keywords = ["wallet", "pkpass", "google-wallet", "apple-wallet", "ferro"]
categories = ["web-programming"]
readme = "README.md"
```

### `from_env` shape

**ferro-stripe/src/config.rs:27–46** (Stripe — hard-erroring on required vars, soft on optional):

```rust
pub fn from_env() -> Result<Self, Error> {
    let api_key = std::env::var("STRIPE_SECRET_KEY")
        .map_err(|_| Error::Config("STRIPE_SECRET_KEY not set".to_string()))?;
    let webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET")
        .map_err(|_| Error::Config("STRIPE_WEBHOOK_SECRET not set".to_string()))?;
    let connect_webhook_secret = std::env::var("STRIPE_CONNECT_WEBHOOK_SECRET").ok();
    // ...
}
```

**ferro-wallet target shape (D-02 permissive — missing cluster ⇒ None, never errors):**

```rust
pub fn from_env() -> Result<Self, WalletError> {
    let app_name = std::env::var("APP_NAME")
        .unwrap_or_else(|_| "Ferro Application".to_string());          // matches framework::config::AppConfig
    let app_url = std::env::var("APP_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());      // matches framework::config::AppConfig

    let apple = AppleConfig::from_env_optional()?;   // returns Ok(None) if any required Apple var missing
    let google = GoogleConfig::from_env_optional()?; // returns Ok(None) if any required Google var missing

    Ok(Self { app_name, app_url, apple, google })
}
```

### `Error` enum derive + name-prefixed `Display`

**ferro-stripe/src/error.rs:1–27** (the canonical pattern):

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("stripe config error: {0}")]
    Config(String),
    #[error("stripe API error: {0}")]
    Stripe(String),
    #[error("no Stripe Connect account linked to this tenant")]
    NoConnectAccount,
    #[error("webhook verification failed: {0}")]
    WebhookVerification(String),
    // ...
}
```

**ferro-wallet target shape (D-04 — every variant prefixes its Display with its own name):**

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

### `lib.rs` re-export shape

**ferro-stripe/src/lib.rs:43–69**:

```rust
pub mod account;
pub mod checkout;
pub mod client;
pub mod config;
pub mod error;
pub mod idempotency;
pub mod refund;
pub mod webhook;

pub use checkout::{CheckoutBuilder, CheckoutIntent, LineItem, Mode};
pub use client::Stripe;
pub use config::StripeConfig;
pub use error::Error;
```

**ferro-wallet target shape (D-11 — re-exports stripped during scaffold, restored per builder landing):**

```rust
pub mod apple;
pub mod config;
pub mod error;
pub mod google;
pub mod images;
pub mod qr;
pub mod subject;

pub use apple::ApplePassBuilder;                 // Restored in PLAN-05
pub use config::{AppleConfig, GoogleConfig, WalletConfig};
pub use error::WalletError;
pub use google::GoogleWalletBuilder;              // Restored in PLAN-07
pub use subject::{
    Branding, Field, FieldAlignment, GeoPoint, PassKind, RgbColor, TextColorMode, WalletSubject,
};
```

### Module structure (per-concern split)

| ferro-wallet module | ferro-stripe analog | Notes |
|---------------------|----------------------|-------|
| `apple/mod.rs` | `webhook/mod.rs` (multi-file submodule pattern) | Both crates use folder + `mod.rs` + supporting files for the biggest concerns |
| `apple/sign.rs` | `webhook/verify.rs` | Cryptographic primitive isolated in its own file |
| `apple/package.rs` | (no analog — Stripe has no packaging concern) | New pattern, no precedent |
| `google/jwt.rs` | (no analog — Stripe uses HMAC-SHA256 in `webhook/verify.rs`, not JWT) | New pattern; closest precedent is the HMAC-SHA256 verification in `ferro-stripe/src/webhook/verify.rs` |
| `subject.rs` | (no analog — Stripe has no domain trait) | New pattern; `WalletSubject` is the user-implemented contract |
| `error.rs` | `error.rs` | Identical pattern: `thiserror::Error` derive + name-prefixed `#[error]` strings |
| `config.rs` | `config.rs` | Same pattern, but D-02 makes `ferro-wallet`'s `from_env` permissive vs. Stripe's hard-erroring required-vars |
| `README.md` | `README.md` (~10 lines) | Match length and tone — short, directs reader to docs.rs and the spec |

### `tests/` layout

**ferro-stripe** has `tests/dispatcher.rs` + `tests/parser_contract.rs` + `tests/fixtures/`.

**ferro-wallet** ships exactly two integration tests per D-09: `tests/apple_integration.rs` (mints self-signed X.509, builds `.pkpass`, asserts ZIP contents) and `tests/google_jwt.rs` (mints RSA keypair, signs save JWT, decodes with public key, asserts claims).

## Workspace Integration

### Edit 1: `Cargo.toml` workspace root members array

**File:** `/Users/alberto/repositories/albertogferrario/ferro/Cargo.toml`
**Lines 1–24** (current state):

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
    "ferro-whatsapp",
]
```

**Required edit:** append `"ferro-wallet",` to the `members` array. The existing order is **not alphabetical** (e.g., `ferro-stripe` precedes `ferro-theme`, `ferro-theme` precedes `ferro-ai`) — it appears to follow phase introduction order. Recommend appending at the end (after `ferro-whatsapp`), consistent with insertion order.

**Resulting array (line 23 → 24):**

```toml
    "ferro-whatsapp",
    "ferro-wallet",
]
```

### Edit 2: `Cargo.toml` `[workspace.package]` version bump (PLAN-09 only)

**File:** `/Users/alberto/repositories/albertogferrario/ferro/Cargo.toml`
**Line 27** (current state):

```toml
[workspace.package]
version = "0.2.23"
```

PLAN-09 (the release plan) bumps to `"0.2.24"` only after PLAN-01..08 land. Note: the GH Actions workflow **auto-bumps the patch version** if the current version is already tagged (see `publish.yml` lines 75–98 — the `check-version` job runs `git tag | grep -q "^v$VERSION$"` and auto-bumps if matched). PLAN-09 only needs to land the workspace version bump *iff* `0.2.23` is already tagged; in practice, the post-Phase-150 commit `5496da6e chore: bump version to 0.2.23` was the GH-Actions bump, so manual bump is optional — the CI will handle it.

### Edit 3: `.github/workflows/publish.yml` Wave 1a list

**File:** `/Users/alberto/repositories/albertogferrario/ferro/.github/workflows/publish.yml`
**Line 201** (current state):

```yaml
WAVE1A_CRATES="ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp"
```

**Required edit:** `ferro-wallet` is a **leaf crate** (zero internal `ferro-*` workspace deps per spec §5: "No dependency on `framework` — the crate stays pure"). It belongs in **Wave 1a**.

**Resulting line:**

```yaml
WAVE1A_CRATES="ferro-macros ferro-events ferro-queue ferro-broadcast ferro-storage ferro-cache ferro-lang ferro-theme ferro-json-ui ferro-inertia ferro-api-mcp ferro-wallet"
```

**Verification:** Wave 1b (line 236) lists `ferro-ai ferro-projections ferro-stripe ferro-whatsapp ferro-notifications` — these all have at least one internal `ferro-*` dep (e.g., `ferro-stripe → ferro-queue`, `ferro-whatsapp → ferro-events + ferro-queue`). `ferro-wallet` has none, so Wave 1a is correct. [VERIFIED: spec §5 + `publish.yml` lines 201, 236]

### Edit 4 (optional but recommended): `.planning/STATE.md`

Update milestone field to `v11.10` (or note the discrepancy explicitly — STATE.md currently says `milestone: v11.0` despite the project being on v11.10 per ROADMAP.md). Out of scope for PLAN-01..08; can be touched in PLAN-09 alongside the version bump and CHANGELOG entry.

### No `[workspace.dependencies]` block

The workspace does **not** use `[workspace.dependencies]`. Every crate declares its own deps with literal version strings. `ferro-wallet/Cargo.toml` should follow the same convention (literal versions, not `workspace = true` for deps).

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust's built-in `cargo test` + `#[test]` attribute (no external runner) |
| Config file | None (workspace `Cargo.toml` `[workspace]` section drives discovery) |
| Quick run command | `cargo test -p ferro-wallet --lib` (unit tests only, ~5s expected) |
| Per-test-file command | `cargo test -p ferro-wallet --test apple_integration` / `--test google_jwt` |
| Full suite command | `cargo test --all-features` (workspace-wide; matches CI) |

### Phase Requirements → Test Map

| Req ID | Behaviour | Test type | Automated command | File exists? |
|--------|-----------|-----------|-------------------|--------------|
| ACC-1a | `WalletConfig::from_env` returns `apple: None` when Apple cluster missing | unit | `cargo test -p ferro-wallet --lib config::tests::from_env_apple_missing_is_none` | ❌ Wave 0 (PLAN-03) |
| ACC-1b | `WalletConfig::from_env` returns `google: None` when Google cluster missing | unit | `cargo test -p ferro-wallet --lib config::tests::from_env_google_missing_is_none` | ❌ Wave 0 (PLAN-03) |
| ACC-1c | `WalletConfig::from_env` falls back to `"Ferro Application"` / `"http://localhost:8080"` defaults | unit | `cargo test -p ferro-wallet --lib config::tests::from_env_defaults_match_appconfig` | ❌ Wave 0 (PLAN-03) |
| ACC-1d | `build_manifest` produces lowercase hex SHA1 per file | unit | `cargo test -p ferro-wallet --lib apple::manifest::tests::manifest_sha1_lowercase_hex` | ❌ Wave 0 (PLAN-05) |
| ACC-1e | `RgbColor::from_hex` parses `#RRGGBB` correctly | unit | `cargo test -p ferro-wallet --lib subject::tests::rgb_from_hex` | ❌ Wave 0 (PLAN-02) |
| ACC-1f | BT.601 luminance threshold derives white for dark backgrounds | unit | `cargo test -p ferro-wallet --lib subject::tests::auto_foreground_dark_bg_is_white` | ❌ Wave 0 (PLAN-02) |
| ACC-1g | `fit_to` produces exact target dimensions with transparent padding | unit | `cargo test -p ferro-wallet --lib images::tests::fit_to_exact_dims_transparent` | ❌ Wave 0 (PLAN-04) |
| ACC-1h | `qr::png` returns valid PNG bytes (magic-byte check) | unit | `cargo test -p ferro-wallet --lib qr::tests::png_starts_with_png_magic` | ❌ Wave 0 (PLAN-04) |
| ACC-1i | `save_url(jwt)` returns `https://pay.google.com/gp/v/save/{jwt}` | unit | `cargo test -p ferro-wallet --lib google::jwt::tests::save_url_format` | ❌ Wave 0 (PLAN-07) |
| ACC-1j (integration) | `.pkpass` ZIP contains 9 files (`pass.json`, `manifest.json`, `signature`, logo + icon × {1x,2x,3x}); `pass.json` carries correct `passTypeIdentifier`, `teamIdentifier`, `serialNumber`, `barcode.message`, `eventTicket.primaryFields[0].value` | integration | `cargo test -p ferro-wallet --test apple_integration` | ❌ Wave 0 (PLAN-06) |
| ACC-1k (integration) | RS256 JWT decodes with public key; claims `iss/aud=google/typ=savetowallet` match; payload contains exactly one `eventTicketObjects` entry with expected `id` and `barcode.value` | integration | `cargo test -p ferro-wallet --test google_jwt` | ❌ Wave 0 (PLAN-08) |
| ACC-2 | `cargo build --workspace` is green with new crate | build | `cargo build --workspace` | ✅ (always available) |
| ACC-3 | `cargo doc --no-deps -p ferro-wallet` produces clean output | build | `cargo doc --no-deps -p ferro-wallet` | ✅ (always available) |
| ACC-4 | `ferro-wallet` published to crates.io after workspace version bump | release | (GH Actions `publish.yml` Wave 1a runs `cargo publish -p ferro-wallet --no-verify`) | ✅ |

### Sampling Rate

- **Per task commit:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test -p ferro-wallet` (per CLAUDE.md gate; scoped to ferro-wallet for speed)
- **Per wave merge:** `cargo test --all-features` (full workspace suite)
- **Phase gate:** Full suite green + `cargo doc --no-deps -p ferro-wallet` clean before `/gsd-verify-work`
- **Release gate:** GH Actions `publish.yml` workflow succeeds (Wave 1a → 1b → 2 → 3 sequence)

### Nyquist Dimensions

| Dimension | Command | Triggered by |
|-----------|---------|--------------|
| **Behaviour** | `cargo test -p ferro-wallet` (covers all 11 unit + 2 integration tests above) | Every task commit |
| **Build / Lint** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo build --workspace && cargo doc --no-deps -p ferro-wallet` | Every task commit + phase gate |
| **Release** | GH Actions `publish.yml` Wave 1a — `cargo publish -p ferro-wallet --no-verify` — verifiable via crates.io showing the published version landing | Post-merge to master, PLAN-09 |

### Wave 0 Gaps

All test files are net-new:

- [ ] `ferro-wallet/src/config.rs` `#[cfg(test)] mod tests` block — covers ACC-1a/1b/1c (PLAN-03)
- [ ] `ferro-wallet/src/subject.rs` `#[cfg(test)] mod tests` block — covers ACC-1e/1f (PLAN-02)
- [ ] `ferro-wallet/src/apple/manifest.rs` `#[cfg(test)] mod tests` block — covers ACC-1d (PLAN-05)
- [ ] `ferro-wallet/src/images.rs` `#[cfg(test)] mod tests` block — covers ACC-1g (PLAN-04)
- [ ] `ferro-wallet/src/qr.rs` `#[cfg(test)] mod tests` block — covers ACC-1h (PLAN-04)
- [ ] `ferro-wallet/src/google/jwt.rs` `#[cfg(test)] mod tests` block — covers ACC-1i (PLAN-07)
- [ ] `ferro-wallet/tests/apple_integration.rs` — covers ACC-1j (PLAN-06)
- [ ] `ferro-wallet/tests/google_jwt.rs` — covers ACC-1k (PLAN-08)

No framework install needed — Rust's built-in test runner ships with the toolchain. No additional `[dev-dependencies]` beyond what the implementation already pulls in (openssl for the Apple integration test's self-signed cert, jsonwebtoken for the Google JWT roundtrip).

## Code Examples

Verified patterns from official sources (use directly in plan code blocks).

### jsonwebtoken RS256 encode (Google JWT signing)

```rust
// Source: https://github.com/Keats/jsonwebtoken (README) + Context7 /keats/jsonwebtoken
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;

#[derive(Serialize)]
struct SaveClaims<'a> {
    iss: &'a str,
    aud: &'a str,
    typ: &'a str,
    iat: i64,
    origins: Vec<&'a str>,
    payload: serde_json::Value,
}

let claims = SaveClaims {
    iss: &self.service_account_email,
    aud: "google",
    typ: "savetowallet",
    iat: chrono::Utc::now().timestamp(),
    origins: vec![&self.app_url],
    payload: serde_json::json!({ "eventTicketObjects": [event_ticket_object] }),
};

let header = Header::new(Algorithm::RS256);
let key = EncodingKey::from_rsa_pem(self.service_account_private_key_pem.as_bytes())
    .map_err(|e| WalletError::GoogleJwt(format!("private key parse: {e}")))?;
let jwt = encode(&header, &claims, &key)
    .map_err(|e| WalletError::GoogleJwt(format!("encode: {e}")))?;
```

### openssl PKCS#7 detached signing (Apple signature)

```rust
// Source: docs.rs/openssl/latest/openssl/pkcs7/struct.Pkcs7.html
use openssl::pkcs7::{Pkcs7, Pkcs7Flags};
use openssl::stack::Stack;
use openssl::x509::X509;

let mut wwdr_stack: Stack<X509> = Stack::new()
    .map_err(|e| WalletError::AppleSign(format!("stack init: {e}")))?;
wwdr_stack.push(self.wwdr.clone())
    .map_err(|e| WalletError::AppleSign(format!("stack push wwdr: {e}")))?;

let flags = Pkcs7Flags::DETACHED | Pkcs7Flags::BINARY;
let pkcs7 = Pkcs7::sign(&self.signing_cert, &self.private_key, &wwdr_stack, manifest_bytes, flags)
    .map_err(|e| WalletError::AppleSign(format!("pkcs7 sign: {e}")))?;
let signature_der = pkcs7.to_der()
    .map_err(|e| WalletError::AppleSign(format!("pkcs7 to_der: {e}")))?;
```

### zip 2.x `.pkpass` packaging (Stored compression)

```rust
// Source: Context7 /zip-rs/zip2
use std::io::{Cursor, Write};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

let mut buf = Cursor::new(Vec::new());
let mut zip = ZipWriter::new(&mut buf);
let opts = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Stored);  // Apple recommends stored, not deflated

for (name, bytes) in entries {
    zip.start_file(name, opts)
        .map_err(|e| WalletError::ApplePackage(format!("start_file {name}: {e}")))?;
    zip.write_all(&bytes)
        .map_err(|e| WalletError::ApplePackage(format!("write {name}: {e}")))?;
}
zip.finish().map_err(|e| WalletError::ApplePackage(format!("finish: {e}")))?;
Ok(buf.into_inner())
```

### image 0.25 fit + centre-pad on transparent canvas

```rust
// Source: Context7 /image-rs/image (imageops::overlay + RgbaImage::new)
use image::{imageops, imageops::FilterType, DynamicImage, ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;

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
```

### qrcode-generator 5.0.0 PNG output

```rust
// Source: docs.rs/qrcode-generator/5.0.0/qrcode_generator/fn.to_png_to_vec.html
use qrcode_generator::QrCodeEcc;

pub fn png(data: &str, size: u32) -> Result<Vec<u8>, WalletError> {
    qrcode_generator::to_png_to_vec(data, QrCodeEcc::Medium, size as usize)
        .map_err(|e| WalletError::Qr(format!("png generate: {e}")))
}
```

### sha1 0.10 manifest digest (Apple manifest)

```rust
// Source: workspace-locked sha1 = "0.10.6"
use sha1::{Digest, Sha1};

fn sha1_hex_lower(bytes: &[u8]) -> String {
    let mut h = Sha1::new();
    h.update(bytes);
    let digest = h.finalize();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

let manifest: serde_json::Value = serde_json::Value::Object(
    entries.iter()
        .map(|(name, bytes)| (name.clone(), serde_json::Value::String(sha1_hex_lower(bytes))))
        .collect(),
);
let manifest_bytes = serde_json::to_vec(&manifest)
    .map_err(|e| WalletError::ApplePackage(format!("manifest json: {e}")))?;
```

### Self-signed X.509 + RSA keypair at test runtime (D-09)

```rust
// Source: docs.rs/openssl X509Builder + Rsa::generate (verified via Context7 /websites/rs_openssl_openssl)
use openssl::asn1::Asn1Time;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use openssl::x509::{X509Builder, X509NameBuilder};
use openssl::hash::MessageDigest;

fn mint_self_signed() -> (String /* cert_pem */, String /* key_pem */) {
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
```

## Common Pitfalls

### Pitfall 1: `.pkpass` deflate vs. stored

**What goes wrong:** ZIP entries written with `CompressionMethod::Deflated` (zip 2.x's default for many flags) — iOS Wallet rejects the pass on import.
**Why it happens:** `SimpleFileOptions::default()` does not pin `Stored`; CompressionMethod default depends on feature flags.
**How to avoid:** Always `.compression_method(CompressionMethod::Stored)` explicitly in `package.rs`.
**Warning sign:** iOS Wallet "pass cannot be installed" error with no diagnostics.

### Pitfall 2: PKCS#7 chain order matters for Apple

**What goes wrong:** Apple validates the signature against `signature` + WWDR intermediate. If the WWDR cert isn't included in the `Stack<X509>`, the signature is unverifiable on-device.
**Why it happens:** `Pkcs7::sign`'s `certs` parameter is **optional additional certs** — it's easy to pass `&Stack::new()?` and silently produce a chain-less signature.
**How to avoid:** Always push the WWDR intermediate onto the Stack before calling `Pkcs7::sign` (D-05).
**Warning sign:** Self-signed test passes succeed but real WWDR-issued passes fail on-device.

### Pitfall 3: jsonwebtoken `exp` claim default validation

**What goes wrong:** `jsonwebtoken::Validation::default()` requires an `exp` claim. The Apple/Google wallet save JWT does NOT have one — only `iat`.
**Why it happens:** Decoding tests using `decode::<Claims>(...)` will fail with `ErrorKind::MissingRequiredClaim("exp")` against a save JWT.
**How to avoid:** In `tests/google_jwt.rs`, configure `Validation::new(Algorithm::RS256)` with `validate_exp = false` and `required_spec_claims = HashSet::new()`. (Encoding path is fine — only decode validation is affected.)
**Warning sign:** Test fails with "Missing required claim: exp" even though the JWT is correctly formed.

### Pitfall 4: `image` crate `resize` returns DynamicImage, not RgbaImage

**What goes wrong:** Trying to call `imageops::overlay(&mut canvas, &dyn_img, ...)` directly — `overlay` requires the same pixel type on both sides.
**Why it happens:** `DynamicImage::resize` returns `DynamicImage`; `RgbaImage::new` creates `ImageBuffer<Rgba<u8>, _>`.
**How to avoid:** Always `.into_rgba8()` on the resized result before overlay (shown in code example above).
**Warning sign:** Cryptic trait-bound compile error mentioning `GenericImage`.

### Pitfall 5: `openssl-sys` build dependency on system OpenSSL

**What goes wrong:** `cargo build` fails in clean CI environments without `libssl-dev` / `openssl@3` installed.
**Why it happens:** The `openssl` crate links against the system OpenSSL by default. GH Actions `ubuntu-latest` has it installed; macOS without `brew install openssl@3` does not.
**How to avoid:** Document the build dependency in README. Optionally consider `openssl = { version = "0.10", features = ["vendored"] }` to compile OpenSSL from source — adds ~30s to first build but eliminates the system dep. Decision deferred to planner.
**Warning sign:** `error: failed to run custom build command for openssl-sys`.

### Pitfall 6: `WalletConfig::from_env` must NOT cause env-pollution in tests

**What goes wrong:** Tests that `std::env::set_var(...)` to verify `from_env` behaviour leave the process env mutated for subsequent tests in the same binary, causing flaky failures.
**Why it happens:** Cargo runs tests in a single process per crate (parallelised across threads, but `std::env` is process-global).
**How to avoid:** Save the existing value before `set_var`, restore it after the assertion. Pattern in `ferro-stripe/src/config.rs:54–73` is the workspace exemplar.
**Warning sign:** Tests pass in isolation, fail when run as a suite.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hand-rolled PKCS#7 with `der` / `picky-asn1` | `openssl::pkcs7::Pkcs7::sign` with detached + binary flags | openssl 0.10.x stable | Mature, battle-tested; standard for Apple `.pkpass` signing in Rust |
| `qrcode` 0.x + manual PNG encode | `qrcode-generator::to_png_to_vec` one-call API | qrcode-generator 5.0.0 | Simpler surface; spec pins this |
| `zip` 0.x with `FileOptions` builder | `zip` 2.x with `SimpleFileOptions` (copy-able) | zip 2.0 release | API surface simplified; `SimpleFileOptions` is the recommended type alias |
| `image` 0.24 with `imageops::overlay(&mut out, &src, u32, u32)` | `image` 0.25 with `imageops::overlay(&mut out, &src, i64, i64)` (negative coords now allowed) | image 0.25 release | Centre-padding math should use `i64` offsets to allow `(w - rw) / 2` to go negative if input is larger than target (defensive coding) |

**Deprecated/outdated:**
- `zip = "0.x"` — superseded by 2.x; no longer maintained.
- `jsonwebtoken = "8.x"` — Algorithm enum changed; current major is 9.x stable, 10.x latest.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Append-to-end (not alphabetical) is the right placement in `[workspace] members` | Workspace Integration | None — purely cosmetic; cargo doesn't care about ordering |
| A2 | The "0.2.23 → 0.2.24" bump is appropriate for Phase 151 (patch, not minor) | Workspace Integration | Low — workspace is `0.2.x` series; consistent with prior phases bumping patch (e.g., `5496da6e chore: bump version to 0.2.23` was post-Phase-150 patch). Could be argued that "new crate" warrants a minor bump under semver — recommend confirming with user in `/gsd-discuss-phase` if no prior decision was made. **[ASSUMED]** |
| A3 | Wave 1a placement is correct for ferro-wallet (no internal deps) | Workspace Integration | None — verified by spec §5 + publish.yml Wave-1b list ([CITED: publish.yml lines 201, 236]). |
| A4 | `version.workspace = true` is preferred over standalone version (matching ferro-whatsapp / ferro-ai pattern) | Pattern Alignment | Low — ferro-stripe is the divergent case (own version 0.5.0); the newer convention is workspace tracking. **[ASSUMED]** Decision rests with planner. |
| A5 | `openssl-sys` vendored vs. system-linked decision left to planner | Pitfalls | Medium — affects CI portability; current workspace has no direct openssl consumer, so this is a green-field decision. **[ASSUMED]** |
| A6 | Manifest JSON key ordering is not deterministic in `serde_json::Map` and may need a `BTreeMap` for byte-stable manifests | Code Examples | Low-Medium — Apple's signature verifier signs the manifest bytes as-emitted; non-determinism between builds doesn't break verification but breaks reproducible-build properties. **[ASSUMED]** worth confirming in PLAN-05. |

## Open Questions

1. **Workspace version bump policy (minor vs. patch).**
   - What we know: Recent phases (146, 147, 148, 149, 150) all patch-bumped; STATE.md shows `0.2.23`.
   - What's unclear: Is "new public crate" semantically a minor (0.3.0) or a patch (0.2.24)? Pre-1.0 semver is ambiguous on this.
   - Recommendation: Patch-bump (0.2.24). The workspace is pre-1.0, project is "not in production" per MEMORY.md, and the GH Actions auto-bump path defaults to patch. Document the decision in PLAN-09 commit message.

2. **`openssl = "0.10"` vendored feature.**
   - What we know: No existing direct openssl consumer in the workspace; openssl-sys requires libssl headers at build time.
   - What's unclear: Does CI's `ubuntu-latest` runner have libssl-dev preinstalled? (Almost certainly yes for Ubuntu 22.04+, but not guaranteed.)
   - Recommendation: Start without `vendored` feature (faster builds); if CI green light fails, switch to `features = ["vendored"]` in PLAN-01 retroactively. Defer to planner.

3. **`Validation` configuration for the Google JWT decode test.**
   - What we know: Spec asserts the JWT decodes with the public key and the claims match; `jsonwebtoken`'s default `Validation` requires `exp`, which the wallet save JWT doesn't include.
   - What's unclear: Whether the test author will configure `Validation::new(Algorithm::RS256)` with `validate_exp = false` correctly on first try.
   - Recommendation: Include the explicit `Validation` construction in PLAN-08's reference code block to prevent the pitfall (already cited in Pitfall §3 above).

4. **`StateMD` milestone-field discrepancy (out of scope for this phase, flag for future).**
   - What we know: `.planning/STATE.md` line 2 says `milestone: v11.0` despite ROADMAP showing v11.10 is the active milestone.
   - What's unclear: Whether STATE.md should be patched in PLAN-09 alongside the version bump.
   - Recommendation: Out of scope for Phase 151 unless `/gsd-execute-phase` orchestration touches STATE.md anyway. File as a meta-cleanup task.

## Environment Availability

| Dependency | Required by | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Rust toolchain (1.88.0) | All builds | ✅ | matches rust-toolchain.toml | — |
| `cargo` | All builds | ✅ | ships with rustc | — |
| `cargo fmt`, `cargo clippy`, `cargo test` | Pre-commit gate | ✅ | rustfmt + clippy components | — |
| System OpenSSL (`libssl-dev` / `openssl@3`) | `openssl-sys` build dep | ✅ on macOS dev (`brew install openssl@3`); ✅ on ubuntu-latest in CI | — | Switch to `openssl = { version = "0.10", features = ["vendored"] }` if CI fails — compiles OpenSSL from source |
| `git` | Workspace operations | ✅ | — | — |
| GH Actions `publish.yml` | PLAN-09 release | ✅ | Already deployed at `.github/workflows/publish.yml` | Manual `cargo publish -p ferro-wallet --token <token>` from terminal (per MEMORY.md `project_ferro_publish_token_scoping.md` — token has publish-update only, **new-crate bootstrap needs local terminal publish first** — see Risks below) |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** None requiring action.

## Risks & Open Questions

1. **First-publish bootstrap token scoping (HIGH — from MEMORY.md).**
   `project_ferro_publish_token_scoping.md` notes: *"CI publish token has publish-update only, not publish-new; new crates need bootstrap from local terminal."* PLAN-09 (or a pre-PLAN-09 manual step) needs to run `cargo publish -p ferro-wallet --token <local-token>` for the **first** publish of `ferro-wallet`. After that, CI's update-only token takes over for subsequent versions.
   **Action for planner:** Add an explicit manual-bootstrap task to PLAN-09 *before* the workflow auto-publish kicks in.

2. **Workspace version bump timing.**
   Per `publish.yml`'s `check-version` job (lines 75–98), if `Cargo.toml` `[workspace.package].version` is already tagged, CI **auto-bumps** the patch version and commits the bump back to master. If PLAN-09 manually bumps to `0.2.24` and pushes, and `0.2.23` is already tagged, CI will see `0.2.24` as untagged and publish directly. If PLAN-09 doesn't bump, CI bumps to `0.2.24` automatically. Either path works; documentation should clarify the intended flow.

3. **WWDR chain handling in tests vs. production.**
   D-09 says the integration test mints a self-signed cert and uses it as both signing cert and WWDR. openssl is happy with this (verified API); Apple Wallet on-device would reject the chain. The test verifies *structure*, not *Apple-validity*. The risk is that someone reads the test as proof of correctness for real Apple passes and skips real-device testing. **Mitigation:** explicit code comment in `tests/apple_integration.rs` stating this limitation. Suggest planner add this comment requirement as part of PLAN-06 acceptance.

4. **`image` crate transitive dep weight (LOW).**
   The `image` 0.25 crate pulls in PNG, JPEG, WebP, TIFF, GIF, BMP decoders by default. The crate only needs PNG. Plan can opt into `default-features = false, features = ["png"]` to trim the dependency footprint. Not strictly required — defer to planner discretion.

5. **`Validation` configuration in `tests/google_jwt.rs` (LOW, documented in Pitfall §3).**
   Already in Open Questions §3.

6. **`openssl-sys` vendored feature decision (LOW, documented in Pitfall §5).**
   Already in Open Questions §2.

7. **Manifest JSON byte-stability (LOW).**
   `serde_json::Map` ordering depends on the `preserve_order` feature; default is HashMap-based (non-deterministic). Apple signs whatever bytes you produce, so this isn't a correctness issue, but reproducible builds want determinism. Recommend using `BTreeMap<String, String>` for the manifest map in PLAN-05 to lock alphabetic ordering.

## Sources

### Primary (HIGH confidence)

- `/Users/alberto/repositories/albertogferrario/ferro/.planning/phases/151-ferro-wallet-crate/151-CONTEXT.md` — locked decisions D-01..D-11, file structure, suggested wave decomposition
- `/Users/alberto/repositories/albertogferrario/ferro/docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md` — public API surface (§3), dependency set (§5), error variants (§6), test strategy (§7), acceptance criteria (§9)
- `/Users/alberto/repositories/albertogferrario/ferro/CLAUDE.md` — Architecture Principle 6 (project-agnostic crates), pre-commit gate, workspace conventions
- `/Users/alberto/repositories/albertogferrario/ferro/Cargo.toml` (workspace root) — `members` array (lines 3–24), workspace version 0.2.23 (line 27)
- `/Users/alberto/repositories/albertogferrario/ferro/.github/workflows/publish.yml` — Wave 1a list (line 201), Wave 1b list (line 236), auto-bump logic (lines 75–98)
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-stripe/Cargo.toml` + `src/config.rs` + `src/error.rs` + `src/lib.rs` + `README.md` — canonical pattern exemplar
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-whatsapp/Cargo.toml`, `ferro-ai/Cargo.toml` — `version.workspace = true` convention
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-inertia/src/config.rs` — `InertiaConfig::app_name` pattern
- `/Users/alberto/repositories/albertogferrario/ferro/framework/src/config/providers/app.rs` — `AppConfig::from_env` defaults (`"Ferro Application"` / `"http://localhost:8080"`)
- `/Users/alberto/repositories/albertogferrario/ferro/Cargo.lock` — verified workspace-locked versions: openssl 0.10.75 (transitive only), sha1 0.10.6, base64 0.22.1, thiserror 2.0.17, chrono 0.4.42, serde 1.0.228, serde_json 1.0.145

### Secondary (HIGH confidence — Context7 + crates.io API)

- Context7 `/keats/jsonwebtoken` — RS256 + `EncodingKey::from_rsa_pem` + `encode(...)` API
- Context7 `/zip-rs/zip2` — `ZipWriter` + `SimpleFileOptions` + `CompressionMethod::Stored` API
- Context7 `/image-rs/image` — `DynamicImage::resize` + `imageops::overlay` + `RgbaImage::from_pixel` API
- Context7 `/websites/rs_openssl_openssl` — `Pkcs7::sign` + `Pkcs7Flags::DETACHED | BINARY` API; X509Builder + Rsa::generate for self-signed cert minting
- Context7 `/websites/rs_qrcode-generator_5_0_0` — `to_png_to_vec(data, QrCodeEcc::Medium, size)` API
- crates.io `/api/v1/crates/{zip,jsonwebtoken,image,qrcode-generator,openssl}/versions` (2026-05-11) — latest stable: openssl 0.10.79, zip 2.4.2, jsonwebtoken 9.3.1, image 0.25.10, qrcode-generator 5.0.0

### Tertiary

- `~/.claude/projects/-Users-alberto-repositories-albertogferrario-ferro/memory/MEMORY.md` — `project_ferro_publish_token_scoping.md` (token scoping risk), `project_ferro_publication.md` (crates.io status), "When adding a new crate to the workspace" convention
- `/Users/alberto/repositories/albertogferrario/ferro/.planning/ROADMAP.md` line 40 — v11.10 milestone framing

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — every dep version verified against crates.io API (2026-05-11) and Context7 docs
- Architecture: HIGH — CONTEXT.md is design-doc-grade; spec is authoritative
- Pitfalls: HIGH — verified against Context7 and docs.rs API references
- Workspace integration: HIGH — every edit traced to exact file + line number
- Pattern alignment: HIGH — ferro-stripe code read in full; module-by-module mapping concrete
- Risks: MEDIUM — open questions exist (first-publish token bootstrap, openssl-sys vendored, manifest byte-stability) but well-bounded; planner can resolve in PLAN-01 / PLAN-05 / PLAN-09

**Research date:** 2026-05-11
**Valid until:** 2026-06-10 (30 days; deps are stable lines, no churn expected). Re-verify if openssl 0.10.x ships a breaking change or if jsonwebtoken 10.x adoption becomes preferred.

## RESEARCH COMPLETE
