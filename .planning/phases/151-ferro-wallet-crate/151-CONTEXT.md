# Phase 151: ferro-wallet — Context

**Gathered:** 2026-05-11
**Status:** Ready for planning
**Milestone:** v11.10 ferro-wallet
**Spec:** [docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md](../../../docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md)
**Downstream consumer:** gestiscilo-it digital wallet booking passes integration (separate plan in the gestiscilo repo).

<domain>
## Phase Boundary

Create a new `ferro-wallet` crate inside the ferro workspace that any ferro application can depend on to issue Apple `.pkpass` files and Google Wallet save-links. The crate exposes:

- `WalletSubject` trait — content contract (a downstream model implements it for whatever domain object the pass represents).
- `WalletConfig` with permissive `from_env` — APP_NAME / APP_URL plus Apple cluster + Google cluster, each optional.
- `ApplePassBuilder` — PKCS#7-signed `.pkpass` ZIP via `openssl` + `zip` + `sha1`.
- `GoogleWalletBuilder` — RS256-signed save JWT via `jsonwebtoken`, returns `pay.google.com/gp/v/save/{jwt}` URL.
- `images` module — Apple logo set (1x/2x/3x), Apple icon set (derivable from logo), Google hero (1032×336).
- `qr` module — PNG + base64 data-URI helpers.

The crate must not contain any hardcoded application identity. It mirrors the `ferro-inertia::InertiaConfig::app_name` / `ferro-stripe::StripeConfig::from_env` pattern (architecture principle 6).

Out of scope (deferred): Apple Web Service Protocol (live updates / Express Mode), Google `objects.patch`, locale resolution beyond raw string passthrough, additional pass kinds beyond exercising `EventTicket` in tests (Generic / Coupon are declared but un-tested).
</domain>

<decisions>
## Implementation Decisions

### D-01: Two builders, deliberately separate
Apple `.pkpass` (PKCS#7 detached over SHA1 manifest in ZIP) and Google Wallet (RS256 JWT pointing at JSON object) share nothing at the wire-format level. The `WalletSubject` trait is the only shared abstraction; builders stay split. A unified `WalletBuilder` would obscure format-specific failure modes and gain no shared code.

### D-02: Permissive WalletConfig::from_env
Missing Apple cluster ⇒ `apple: None`. Missing Google cluster ⇒ `google: None`. `from_env` never errors on absent wallet env vars — callers gate features on `WalletConfig.apple.is_some()` / `.google.is_some()`. APP_NAME / APP_URL fall back to the same defaults as `framework::config::AppConfig` (`"Ferro Application"` / `"http://localhost:8080"`).

### D-03: Image pipeline
`images::fit_to(bytes, w, h)` resizes-preserve-aspect then centre-pads onto a transparent canvas of the target size, encoding the result as PNG. `apple_logo_set` emits `logo.png` (160×50), `logo@2x.png` (320×100), `logo@3x.png` (480×150) from a single input. `apple_icon_set` accepts an optional explicit icon; when absent, centre-square-crops the logo and resizes to 29×29 / 58×58 / 87×87.

### D-04: WalletError variants
Each variant prefixes its `Display` impl with its name (`"config: …"`, `"apple sign: …"`, `"google jwt: …"`, etc.) so production log greps stay surgical. `Io(#[from] std::io::Error)` for zip + io plumbing.

### D-05: Apple manifest + signature
`build_manifest(&[(name, bytes)])` returns `manifest.json` as `{ "<name>": "<sha1-hex-lower>", ... }`. `SigningMaterial::sign_detached(manifest_bytes)` produces DER-encoded PKCS#7 with `Pkcs7Flags::DETACHED | Pkcs7Flags::BINARY`, the WWDR intermediate pushed onto a single-element `Stack<X509>`. Output goes into the ZIP as `signature` (no extension).

### D-06: Apple foreground / label color derivation
When `Branding.text_color_mode == Auto`, derive foreground from background ITU-R BT.601 luminance: `< 0.5` ⇒ white `rgb(255,255,255)`, `>= 0.5` ⇒ dark slate `rgb(17,24,39)`. `Light` / `Dark` force white / black respectively. `labelColor` always tracks `foregroundColor` in v1.

### D-07: Google class + object ID format
`class_id = "{issuer_id}.{pass_type_id_with_dots_replaced_by_underscores}"`. `object_id = "{issuer_id}.{subject.serial()}"`. Google has no equivalent to Apple's pass-type-id; v1 uses a fixed `"booking"` suffix (`pass_type_id_default()` const). Downstream callers that need a different logical class can fork the builder in a future phase.

### D-08: JWT claim shape
```json
{
  "iss": "<service_account_email>",
  "aud": "google",
  "typ": "savetowallet",
  "iat": <unix>,
  "origins": ["<app_url>"],
  "payload": { "eventTicketObjects": [ <one object built from subject> ] }
}
```
Signed RS256 with the PEM-loaded private key. `save_url(jwt) = "https://pay.google.com/gp/v/save/{jwt}"`.

### D-09: Test strategy without real Apple/Google credentials
- Apple: `tests/apple_integration.rs` mints a self-signed X.509 at runtime via `openssl` and uses it as both the signing cert and the WWDR (chain ill-formed for Apple but well-formed for openssl). Asserts the ZIP contains the nine expected files and that `pass.json` carries the right `passTypeIdentifier`, `teamIdentifier`, `serialNumber`, `barcode.message`, and `eventTicket.primaryFields[0].value`.
- Google: `tests/google_jwt.rs` mints an RSA keypair, signs a save JWT, decodes with the public key, asserts claims (`iss`, `aud=google`, `typ=savetowallet`, payload contains exactly one `eventTicketObjects` entry with the expected `id` and `barcode.value`).
No CI dependency on real Apple/Google credentials.

### D-10: Workspace + release
Crate goes under `ferro/ferro-wallet/` and gets added to `[workspace] members` in the workspace root `Cargo.toml`. Workspace `[workspace.package] version` bumps to the next patch when Phase 151 is verified. Release publishes automatically via the existing GitHub Actions workflow on push to master.

### D-11: lib.rs scaffold order
Stub all module files with `// placeholder` lines in Task 01 to keep `cargo check` happy across plans. Temporarily strip the `pub use apple::ApplePassBuilder;` / `pub use google::GoogleWalletBuilder;` re-exports from `lib.rs` until the corresponding builder lands; restore in the same plan that lands the builder body.

</decisions>

<files>
## File Structure

```
ferro/ferro-wallet/
├── Cargo.toml
├── README.md                         (short — directs reader to spec)
└── src/
    ├── lib.rs                        crate root + re-exports
    ├── error.rs                      WalletError (thiserror)
    ├── subject.rs                    WalletSubject + Field/Branding/PassKind/GeoPoint/RgbColor/TextColorMode/FieldAlignment
    ├── config.rs                     WalletConfig + AppleConfig + GoogleConfig + from_env
    ├── images.rs                     fit_to + apple_logo_set + apple_icon_set + google_hero
    ├── qr.rs                         png + data_uri
    ├── apple/
    │   ├── mod.rs                    ApplePassBuilder
    │   ├── manifest.rs               build_pass_json + build_manifest
    │   ├── sign.rs                   SigningMaterial::parse + sign_detached
    │   └── package.rs                zip_pkpass
    └── google/
        ├── mod.rs                    GoogleWalletBuilder
        ├── object.rs                 build_event_ticket_object
        └── jwt.rs                    sign_save_jwt + save_url + pass_type_id_default

ferro/ferro-wallet/tests/
├── apple_integration.rs              end-to-end with runtime self-signed cert
└── google_jwt.rs                     RS256 roundtrip
```

Workspace edits:
- `ferro/Cargo.toml` — add `"ferro-wallet"` to `[workspace] members`.
- `ferro/.planning/ROADMAP.md` — v11.10 milestone entry (already added).
- `ferro/CHANGELOG.md` — entry added at release time.

</files>

<task_breakdown>
## Plans (waves)

Suggested wave decomposition for `/gsd-plan-phase 151`:

- **151-01** — Scaffold (Cargo.toml + lib.rs + module stubs) + `WalletError` (sequential, blocks everything)
- **151-02** — `WalletSubject` trait + value types (parallel-safe after 01)
- **151-03** — `WalletConfig::from_env` (parallel-safe after 01)
- **151-04** — `images` module + `qr` module (parallel-safe after 01)
- **151-05** — `apple/` module: `manifest.rs` → `sign.rs` → `package.rs` → `mod.rs` (depends on 02, 03, 04)
- **151-06** — `tests/apple_integration.rs` (depends on 05)
- **151-07** — `google/` module: `object.rs` + `jwt.rs` + `mod.rs` (depends on 02, 03)
- **151-08** — `tests/google_jwt.rs` (depends on 07)
- **151-09** — Workspace version bump + CHANGELOG entry; auto-publish via Actions (depends on 05–08 green)

Implementation reference for code samples, exact API shapes, and full test bodies: `gestiscilo-it/app/docs/superpowers/plans/2026-05-11-wallet-passes.md` Phase A (tasks A1–A10) — kept in the downstream repo as the originating field-test artefact. The full code for each task is reproduced there. The ferro phase agent should treat that document as the authoritative implementation reference for this phase and run `/gsd-plan-phase 151` to derive its own ordered, atomic-commit-aware PLAN files from it.

</task_breakdown>

<consumer_dependency>
## Downstream Consumer

Gestiscilo's `wallet-passes` integration plan depends on this phase shipping and a new ferro version being auto-published. Once 151 is verified and the version bumps, gestiscilo's `Cargo.toml` updates `ferro-wallet = "0.2.X"` and the consumer integration proceeds. Until then the consumer uses a local `[patch.crates-io]` (uncommitted) for development.
</consumer_dependency>
