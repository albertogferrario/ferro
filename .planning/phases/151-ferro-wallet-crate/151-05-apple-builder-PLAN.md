---
phase: 151
plan: 151-05
slug: apple-builder
wave: 3
depends_on: [151-02, 151-03, 151-04]
files_modified:
  - ferro-wallet/src/apple/manifest.rs
  - ferro-wallet/src/apple/sign.rs
  - ferro-wallet/src/apple/package.rs
  - ferro-wallet/src/apple/mod.rs
  - ferro-wallet/src/lib.rs
autonomous: true
requirements: [ACC-1d]
must_haves:
  truths:
    - "`ApplePassBuilder::new(cfg, app_name)` parses cert + key + WWDR via `SigningMaterial::parse`"
    - "`ApplePassBuilder::build(subject)` returns `Vec<u8>` containing a `.pkpass` ZIP with 9 entries (pass.json, manifest.json, signature, logo×3, icon×3)"
    - "`build_manifest` produces lowercase-hex SHA1 digests in a deterministic-order JSON map"
    - "Signature is PKCS#7 DETACHED + BINARY with WWDR intermediate pushed onto the cert stack"
    - "ZIP uses `CompressionMethod::Stored` (Apple rejects deflated entries)"
    - "`pass.json` carries `passTypeIdentifier`, `teamIdentifier`, `serialNumber`, `organizationName`, `foregroundColor` (derived per D-06), `labelColor` (tracks foreground), `barcodes[0]`, and eventTicket field arrays"
  artifacts:
    - path: "ferro-wallet/src/apple/manifest.rs"
      provides: "build_pass_json + build_manifest"
      contains: "pub(crate) fn build_manifest"
    - path: "ferro-wallet/src/apple/sign.rs"
      provides: "SigningMaterial::parse + sign_detached"
      contains: "Pkcs7Flags::DETACHED"
    - path: "ferro-wallet/src/apple/package.rs"
      provides: "zip_pkpass"
      contains: "CompressionMethod::Stored"
    - path: "ferro-wallet/src/apple/mod.rs"
      provides: "ApplePassBuilder { new, build }"
      contains: "pub struct ApplePassBuilder"
    - path: "ferro-wallet/src/lib.rs"
      provides: "Restored `pub use apple::ApplePassBuilder;`"
      contains: "pub use apple::ApplePassBuilder;"
  key_links:
    - from: "ApplePassBuilder::build"
      to: "build_pass_json + apple_logo_set + apple_icon_set + build_manifest + sign_detached + zip_pkpass"
      via: "linear pipeline composing all 4 sub-modules"
      pattern: "fn build"
    - from: "build_manifest"
      to: "sha1::Sha1"
      via: "SHA1 digest per file (D-05)"
      pattern: "Sha1::new"
    - from: "SigningMaterial::sign_detached"
      to: "Pkcs7::sign with WWDR stack"
      via: "PKCS#7 detached signature (D-05)"
      pattern: "Pkcs7::sign"
---

<objective>
Land the full Apple builder pipeline as a single plan with four sequential tasks: manifest → sign → package → builder facade + lib.rs re-export. The four files share data dependencies (`build_pass_json` output feeds the manifest; `build_manifest` output feeds `sign_detached`; everything feeds `zip_pkpass`) so they belong in one plan.
</objective>

<context>
@.planning/phases/151-ferro-wallet-crate/151-CONTEXT.md
@.planning/phases/151-ferro-wallet-crate/151-PATTERNS.md
@.planning/phases/151-ferro-wallet-crate/151-RESEARCH.md
@.planning/phases/151-ferro-wallet-crate/151-VALIDATION.md
@docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md
@ferro-wallet/src/subject.rs
@ferro-wallet/src/config.rs
@ferro-wallet/src/images.rs
@ferro-wallet/src/error.rs
@ferro-wallet/src/lib.rs

<interfaces>
Public API per spec §3.3:
```rust
pub struct ApplePassBuilder { /* parsed cert + key + wwdr + identifiers + app_name */ }

impl ApplePassBuilder {
    pub fn new(cfg: AppleConfig, app_name: String) -> Result<Self, WalletError>;
    pub fn build<S: WalletSubject>(&self, s: &S) -> Result<Vec<u8>, WalletError>;
}
```

Internal contracts (crate-private):
```rust
// apple/manifest.rs
pub(crate) fn build_pass_json<S: WalletSubject>(builder: &ApplePassBuilder, subject: &S) -> Result<Vec<u8>, WalletError>;
pub(crate) fn build_manifest(entries: &[(String, Vec<u8>)]) -> Result<Vec<u8>, WalletError>;

// apple/sign.rs
pub(crate) struct SigningMaterial { pub cert: X509, pub key: PKey<Private>, pub wwdr: X509 }
impl SigningMaterial {
    pub fn parse(cert_pem: &str, key_pem: &str, key_password: Option<&str>, wwdr_pem: &str) -> Result<Self, WalletError>;
    pub fn sign_detached(&self, manifest_bytes: &[u8]) -> Result<Vec<u8>, WalletError>;
}

// apple/package.rs
pub(crate) fn zip_pkpass(entries: &[(String, Vec<u8>)]) -> Result<Vec<u8>, WalletError>;
```

Reference code blocks in 151-RESEARCH.md §"Code Examples": sha1 digest, openssl PKCS#7 detached signing, zip 2.x SimpleFileOptions + Stored. Reference dimensions for the 9 ZIP entries are in 151-PATTERNS.md and ACC-1j.

`pass.json` shape (per Apple Wallet docs + spec §3 + D-06 + D-07):
```json
{
  "formatVersion": 1,
  "passTypeIdentifier": "<builder.pass_type_id>",
  "teamIdentifier": "<builder.team_id>",
  "serialNumber": "<subject.serial()>",
  "organizationName": "<subject.branding().organization_name OR builder.app_name>",
  "description": "<builder.app_name>",
  "backgroundColor": "<subject.branding().background_color CSS rgb()>",
  "foregroundColor": "<derived per D-06>",
  "labelColor": "<same as foregroundColor in v1>",
  "logoText": "<subject.branding().logo_text or omit>",
  "barcodes": [
    { "format": "PKBarcodeFormatQR", "message": "<subject.barcode_token()>", "messageEncoding": "iso-8859-1" }
  ],
  "relevantDate": "<subject.relevant_at() ISO-8601 or omit>",
  "expirationDate": "<subject.expires_at() ISO-8601 or omit>",
  "locations": [ { "latitude": <f64>, "longitude": <f64>, "relevantText": "<text or omit>" } ],
  "eventTicket": {
    "primaryFields":   [ <subject.primary()> ],
    "secondaryFields": <subject.secondary()>,
    "auxiliaryFields": <subject.auxiliary()>,
    "backFields":      <subject.back()>
  }
}
```

Field serialisation: each `Field` becomes `{ "key", "label", "value", "textAlignment": "PKTextAlignment<Left|Center|Right|Natural>" }`.
</interfaces>
</context>

<must_haves>
- Four `apple/` files populated; `// placeholder` lines removed.
- `SigningMaterial::sign_detached` produces DER-encoded PKCS#7 with `Pkcs7Flags::DETACHED | Pkcs7Flags::BINARY` and WWDR pushed onto a non-empty `Stack<X509>` (RESEARCH.md Pitfall 2).
- `zip_pkpass` uses `CompressionMethod::Stored` (RESEARCH.md Pitfall 1).
- `build_manifest` outputs lowercase-hex SHA1 digests; manifest is byte-stable (use `BTreeMap<String, String>` per RESEARCH.md Open Question 7 / Risk 7).
- `ApplePassBuilder::build` assembles the 9 ZIP entries in this order: `pass.json`, `manifest.json`, `signature`, `logo.png`, `logo@2x.png`, `logo@3x.png`, `icon.png`, `icon@2x.png`, `icon@3x.png`. Note: `manifest.json` and `signature` are NOT included in the manifest digest map (the manifest digests `pass.json` + the 6 images only); the manifest itself is then signed; both are added to the ZIP after that.
- `pass.json` includes `formatVersion: 1` plus all required identifiers, colours, barcode, and field arrays.
- ACC-1d test (`apple::manifest::tests::manifest_sha1_lowercase_hex`) exists and passes.
- `lib.rs` restores `pub use apple::ApplePassBuilder;`.
</must_haves>

<tasks>

<task type="auto">
  <name>Task 1: Implement apple/manifest.rs (build_pass_json + build_manifest) + ACC-1d test</name>
  <files>ferro-wallet/src/apple/manifest.rs</files>
  <read_first>
    - docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md §3 (Apple builder shape, manifest description)
    - 151-RESEARCH.md §"Code Examples" → "sha1 0.10 manifest digest" (full body)
    - 151-PATTERNS.md §"ferro-wallet/src/apple/manifest.rs"
    - 151-CONTEXT.md D-05 (manifest format), D-06 (foreground/labelColor derivation), D-07 (pass.json identifiers)
    - 151-RESEARCH.md §"Risks & Open Questions" item 7 (BTreeMap for deterministic manifest)
    - ferro-wallet/src/subject.rs (WalletSubject trait, RgbColor, auto_foreground)
    - 151-VALIDATION.md ACC-1d row (test name)
  </read_first>
  <action>
    Replace the `// placeholder` line in `ferro-wallet/src/apple/manifest.rs`. Implement:

    ```rust
    //! Apple pass.json + manifest.json construction.
    //!
    //! Manifest is a JSON map of filename → lowercase-hex SHA1 of file contents (D-05).
    //! Use `BTreeMap` for byte-stable key ordering (RESEARCH.md Risk 7).

    use std::collections::BTreeMap;

    use sha1::{Digest, Sha1};

    use crate::subject::{auto_foreground, FieldAlignment, RgbColor, TextColorMode, WalletSubject};
    use crate::WalletError;

    use super::ApplePassBuilder;

    pub(crate) fn build_pass_json<S: WalletSubject>(
        builder: &ApplePassBuilder,
        subject: &S,
    ) -> Result<Vec<u8>, WalletError> {
        let branding = subject.branding();
        let org_name = branding
            .organization_name
            .clone()
            .unwrap_or_else(|| builder.app_name.clone());

        let foreground = match branding.text_color_mode {
            TextColorMode::Auto => auto_foreground(branding.background_color),
            TextColorMode::Light => RgbColor { r: 255, g: 255, b: 255 },
            TextColorMode::Dark  => RgbColor { r: 0,   g: 0,   b: 0   },
        };
        // D-06: labelColor tracks foreground in v1.
        let label_color = foreground;

        let mut pass = serde_json::Map::new();
        pass.insert("formatVersion".into(), serde_json::json!(1));
        pass.insert("passTypeIdentifier".into(), serde_json::json!(builder.pass_type_id));
        pass.insert("teamIdentifier".into(), serde_json::json!(builder.team_id));
        pass.insert("serialNumber".into(), serde_json::json!(subject.serial()));
        pass.insert("organizationName".into(), serde_json::json!(org_name));
        pass.insert("description".into(), serde_json::json!(builder.app_name.clone()));
        pass.insert("backgroundColor".into(), serde_json::json!(branding.background_color.css_rgb()));
        pass.insert("foregroundColor".into(), serde_json::json!(foreground.css_rgb()));
        pass.insert("labelColor".into(), serde_json::json!(label_color.css_rgb()));
        if let Some(logo_text) = branding.logo_text.clone() {
            pass.insert("logoText".into(), serde_json::json!(logo_text));
        }
        pass.insert("barcodes".into(), serde_json::json!([{
            "format": "PKBarcodeFormatQR",
            "message": subject.barcode_token(),
            "messageEncoding": "iso-8859-1"
        }]));
        if let Some(t) = subject.relevant_at() {
            pass.insert("relevantDate".into(), serde_json::json!(t.to_rfc3339()));
        }
        if let Some(t) = subject.expires_at() {
            pass.insert("expirationDate".into(), serde_json::json!(t.to_rfc3339()));
        }
        let locs: Vec<serde_json::Value> = subject.locations().iter().map(|g| {
            let mut obj = serde_json::Map::new();
            obj.insert("latitude".into(), serde_json::json!(g.latitude));
            obj.insert("longitude".into(), serde_json::json!(g.longitude));
            if let Some(t) = &g.relevant_text {
                obj.insert("relevantText".into(), serde_json::json!(t));
            }
            serde_json::Value::Object(obj)
        }).collect();
        if !locs.is_empty() {
            pass.insert("locations".into(), serde_json::Value::Array(locs));
        }

        // Field array key per spec §3 — `eventTicket` for the EventTicket kind.
        let kind_key = match subject.pass_kind() {
            crate::subject::PassKind::EventTicket => "eventTicket",
            crate::subject::PassKind::Generic     => "generic",
            crate::subject::PassKind::Coupon      => "coupon",
        };
        let serialise_field = |f: &crate::subject::Field| -> serde_json::Value {
            let alignment = match f.alignment {
                FieldAlignment::Left    => "PKTextAlignmentLeft",
                FieldAlignment::Center  => "PKTextAlignmentCenter",
                FieldAlignment::Right   => "PKTextAlignmentRight",
                FieldAlignment::Natural => "PKTextAlignmentNatural",
            };
            serde_json::json!({
                "key": f.key,
                "label": f.label,
                "value": f.value,
                "textAlignment": alignment,
            })
        };
        let fields = serde_json::json!({
            "primaryFields":   [serialise_field(&subject.primary())],
            "secondaryFields": subject.secondary().iter().map(serialise_field).collect::<Vec<_>>(),
            "auxiliaryFields": subject.auxiliary().iter().map(serialise_field).collect::<Vec<_>>(),
            "backFields":      subject.back().iter().map(serialise_field).collect::<Vec<_>>(),
        });
        pass.insert(kind_key.to_string(), fields);

        serde_json::to_vec(&serde_json::Value::Object(pass))
            .map_err(|e| WalletError::ApplePackage(format!("pass.json serialise: {e}")))
    }

    pub(crate) fn build_manifest(entries: &[(String, Vec<u8>)]) -> Result<Vec<u8>, WalletError> {
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        for (name, bytes) in entries {
            map.insert(name.clone(), sha1_hex_lower(bytes));
        }
        serde_json::to_vec(&map)
            .map_err(|e| WalletError::ApplePackage(format!("manifest json: {e}")))
    }

    fn sha1_hex_lower(bytes: &[u8]) -> String {
        let mut h = Sha1::new();
        h.update(bytes);
        h.finalize().iter().map(|b| format!("{b:02x}")).collect()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// ACC-1d — build_manifest produces lowercase hex SHA1 per file.
        #[test]
        fn manifest_sha1_lowercase_hex() {
            // SHA1("hello") == "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"
            let entries = vec![("hello.txt".to_string(), b"hello".to_vec())];
            let manifest_bytes = build_manifest(&entries).unwrap();
            let v: serde_json::Value = serde_json::from_slice(&manifest_bytes).unwrap();
            assert_eq!(v["hello.txt"], "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
        }

        #[test]
        fn manifest_is_deterministic_across_calls() {
            let entries = vec![
                ("a.txt".to_string(), b"a".to_vec()),
                ("b.txt".to_string(), b"b".to_vec()),
                ("c.txt".to_string(), b"c".to_vec()),
            ];
            let m1 = build_manifest(&entries).unwrap();
            let m2 = build_manifest(&entries).unwrap();
            assert_eq!(m1, m2);
        }

        #[test]
        fn manifest_uses_sorted_keys() {
            // BTreeMap sorts keys alphabetically — verify by checking first key in the JSON byte output.
            let entries = vec![
                ("zebra.png".to_string(), b"z".to_vec()),
                ("alpha.png".to_string(), b"a".to_vec()),
            ];
            let manifest_bytes = build_manifest(&entries).unwrap();
            let s = std::str::from_utf8(&manifest_bytes).unwrap();
            let alpha_pos = s.find("alpha.png").unwrap();
            let zebra_pos = s.find("zebra.png").unwrap();
            assert!(alpha_pos < zebra_pos, "alpha.png should sort before zebra.png in manifest");
        }
    }
    ```
  </action>
  <verify>
    <automated>cargo build -p ferro-wallet &amp;&amp; cargo test -p ferro-wallet --lib apple::manifest::tests::manifest_sha1_lowercase_hex &amp;&amp; cargo test -p ferro-wallet --lib apple::manifest::tests &amp;&amp; cargo clippy -p ferro-wallet --all-targets -- -D warnings &amp;&amp; cargo fmt -p ferro-wallet -- --check &amp;&amp; grep -F 'pub(crate) fn build_manifest' ferro-wallet/src/apple/manifest.rs &amp;&amp; grep -F 'pub(crate) fn build_pass_json' ferro-wallet/src/apple/manifest.rs &amp;&amp; grep -F 'BTreeMap' ferro-wallet/src/apple/manifest.rs</automated>
  </verify>
  <done>`build_pass_json` and `build_manifest` land. ACC-1d test (`manifest_sha1_lowercase_hex`) passes. Deterministic-output test passes. Sorted-key test passes.</done>
</task>

<task type="auto">
  <name>Task 2: Implement apple/sign.rs (SigningMaterial::parse + sign_detached)</name>
  <files>ferro-wallet/src/apple/sign.rs</files>
  <read_first>
    - 151-RESEARCH.md §"Code Examples" → "openssl PKCS#7 detached signing (Apple signature)" (full body)
    - 151-PATTERNS.md §"ferro-wallet/src/apple/sign.rs"
    - 151-CONTEXT.md D-05 (signature format)
    - 151-RESEARCH.md §"Common Pitfalls" Pitfall 2 (WWDR must be on the stack), Pitfall 5 (openssl-sys system dep)
    - docs.rs/openssl `Pkcs7::sign` documentation (via Context7 `/websites/rs_openssl_openssl`)
  </read_first>
  <action>
    Replace the `// placeholder` line. Implement exactly the SigningMaterial pattern from PATTERNS.md, with these specifics:

    ```rust
    //! Apple Wallet PKCS#7 detached signing.
    //!
    //! `SigningMaterial::parse` loads the signing cert, private key (optionally
    //! passphrase-protected), and WWDR intermediate from PEM strings.
    //! `sign_detached(manifest_bytes)` produces DER-encoded PKCS#7 with
    //! `DETACHED | BINARY` flags. The WWDR cert is pushed onto a non-empty
    //! `Stack<X509>` so the on-device chain is well-formed (RESEARCH.md Pitfall 2).

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
            }
            .map_err(|e| WalletError::AppleSign(format!("key parse: {e}")))?;
            let wwdr = X509::from_pem(wwdr_pem.as_bytes())
                .map_err(|e| WalletError::AppleSign(format!("wwdr parse: {e}")))?;
            Ok(Self { cert, key, wwdr })
        }

        pub fn sign_detached(&self, manifest_bytes: &[u8]) -> Result<Vec<u8>, WalletError> {
            let mut wwdr_stack: Stack<X509> = Stack::new()
                .map_err(|e| WalletError::AppleSign(format!("stack init: {e}")))?;
            wwdr_stack
                .push(self.wwdr.clone())
                .map_err(|e| WalletError::AppleSign(format!("stack push wwdr: {e}")))?;

            let flags = Pkcs7Flags::DETACHED | Pkcs7Flags::BINARY;
            let pkcs7 = Pkcs7::sign(&self.cert, &self.key, &wwdr_stack, manifest_bytes, flags)
                .map_err(|e| WalletError::AppleSign(format!("pkcs7 sign: {e}")))?;
            pkcs7
                .to_der()
                .map_err(|e| WalletError::AppleSign(format!("pkcs7 to_der: {e}")))
        }
    }
    ```

    No `#[cfg(test)]` block here — the parse/sign roundtrip is exercised end-to-end in `tests/apple_integration.rs` (PLAN-06). A unit test would require runtime cert minting, which is duplicate work.
  </action>
  <verify>
    <automated>cargo build -p ferro-wallet &amp;&amp; cargo clippy -p ferro-wallet --all-targets -- -D warnings &amp;&amp; cargo fmt -p ferro-wallet -- --check &amp;&amp; grep -F 'Pkcs7Flags::DETACHED | Pkcs7Flags::BINARY' ferro-wallet/src/apple/sign.rs &amp;&amp; grep -F 'wwdr_stack.push' ferro-wallet/src/apple/sign.rs &amp;&amp; grep -F 'pub(crate) struct SigningMaterial' ferro-wallet/src/apple/sign.rs</automated>
  </verify>
  <done>`SigningMaterial::parse` and `sign_detached` land. WWDR is pushed onto the stack (verified via grep). DETACHED | BINARY flags present. No clippy warnings.</done>
</task>

<task type="auto">
  <name>Task 3: Implement apple/package.rs (zip_pkpass)</name>
  <files>ferro-wallet/src/apple/package.rs</files>
  <read_first>
    - 151-RESEARCH.md §"Code Examples" → "zip 2.x .pkpass packaging (Stored compression)" (full body)
    - 151-PATTERNS.md §"ferro-wallet/src/apple/package.rs"
    - 151-RESEARCH.md §"Common Pitfalls" Pitfall 1 (Stored vs Deflated)
  </read_first>
  <action>
    Replace the `// placeholder` line. Implement:

    ```rust
    //! .pkpass ZIP assembly.
    //!
    //! Apple Wallet rejects deflated entries — always use `CompressionMethod::Stored`
    //! (RESEARCH.md Pitfall 1).

    use std::io::{Cursor, Write};

    use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

    use crate::WalletError;

    pub(crate) fn zip_pkpass(entries: &[(String, Vec<u8>)]) -> Result<Vec<u8>, WalletError> {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut buf);
            let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

            for (name, bytes) in entries {
                zip.start_file(name, opts)
                    .map_err(|e| WalletError::ApplePackage(format!("start_file {name}: {e}")))?;
                zip.write_all(bytes)
                    .map_err(|e| WalletError::ApplePackage(format!("write {name}: {e}")))?;
            }
            zip.finish()
                .map_err(|e| WalletError::ApplePackage(format!("finish: {e}")))?;
        }
        Ok(buf.into_inner())
    }
    ```

    Append a minimal `#[cfg(test)] mod tests` block:

    - `zip_pkpass_returns_valid_zip` — call `zip_pkpass(&[("a.txt".into(), b"alpha".to_vec()), ("b.txt".into(), b"beta".to_vec())]).unwrap()`. Verify bytes start with the ZIP local-file-header magic `[0x50, 0x4B, 0x03, 0x04]`. Re-parse using `zip::ZipArchive::new(Cursor::new(bytes))` and assert the archive contains exactly 2 entries with names `a.txt` and `b.txt`.
    - `zip_pkpass_uses_stored_compression` — same as above, but iterate the archive and assert `file.compression() == CompressionMethod::Stored` for each entry.
  </action>
  <verify>
    <automated>cargo build -p ferro-wallet &amp;&amp; cargo test -p ferro-wallet --lib apple::package::tests &amp;&amp; cargo clippy -p ferro-wallet --all-targets -- -D warnings &amp;&amp; cargo fmt -p ferro-wallet -- --check &amp;&amp; grep -F 'CompressionMethod::Stored' ferro-wallet/src/apple/package.rs &amp;&amp; grep -F 'pub(crate) fn zip_pkpass' ferro-wallet/src/apple/package.rs</automated>
  </verify>
  <done>`zip_pkpass` lands. Two tests pass — magic bytes verified, Stored compression verified by re-parsing.</done>
</task>

<task type="auto">
  <name>Task 4: Implement apple/mod.rs (ApplePassBuilder facade) + restore lib.rs re-export</name>
  <files>ferro-wallet/src/apple/mod.rs, ferro-wallet/src/lib.rs</files>
  <read_first>
    - 151-PATTERNS.md §"ferro-wallet/src/apple/mod.rs (ApplePassBuilder)"
    - ferro-wallet/src/apple/manifest.rs (after Task 1 — for `build_pass_json` + `build_manifest` signatures)
    - ferro-wallet/src/apple/sign.rs (after Task 2 — for `SigningMaterial`)
    - ferro-wallet/src/apple/package.rs (after Task 3 — for `zip_pkpass`)
    - ferro-wallet/src/images.rs (`apple_logo_set`, `apple_icon_set`)
    - ferro-wallet/src/lib.rs (commented-out `pub use apple::ApplePassBuilder;`)
    - 151-CONTEXT.md D-11 (re-export restoration)
  </read_first>
  <action>
    Replace `ferro-wallet/src/apple/mod.rs` body (currently has the placeholder + 3 `pub mod` lines from PLAN-01):

    ```rust
    //! Apple Wallet `.pkpass` issuance — SHA1 manifest + PKCS#7 detached signature + ZIP packaging.

    pub mod manifest;
    pub mod package;
    pub mod sign;

    use crate::config::AppleConfig;
    use crate::images;
    use crate::subject::WalletSubject;
    use crate::WalletError;

    pub struct ApplePassBuilder {
        pub(crate) pass_type_id: String,
        pub(crate) team_id: String,
        pub(crate) app_name: String,
        pub(crate) signing: sign::SigningMaterial,
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

        /// Build a complete `.pkpass` ZIP for the given subject.
        pub fn build<S: WalletSubject>(&self, s: &S) -> Result<Vec<u8>, WalletError> {
            // 1. pass.json
            let pass_json_bytes = manifest::build_pass_json(self, s)?;

            // 2. images (logo set + icon set)
            let branding = s.branding();
            let mut image_entries = images::apple_logo_set(&branding.logo_png_bytes)?;
            let icon_entries = images::apple_icon_set(
                branding.icon_png_bytes.as_deref(),
                &branding.logo_png_bytes,
            )?;
            image_entries.extend(icon_entries);

            // 3. manifest = SHA1 of pass.json + each image entry (NOT manifest itself, NOT signature).
            let mut manifest_inputs: Vec<(String, Vec<u8>)> = vec![
                ("pass.json".to_string(), pass_json_bytes.clone()),
            ];
            manifest_inputs.extend(image_entries.iter().cloned());

            let manifest_bytes = manifest::build_manifest(&manifest_inputs)?;

            // 4. signature = PKCS#7 detached over manifest bytes (DER-encoded).
            let signature_bytes = self.signing.sign_detached(&manifest_bytes)?;

            // 5. ZIP order: pass.json, manifest.json, signature, then logo×3 + icon×3.
            let mut zip_entries: Vec<(String, Vec<u8>)> = Vec::with_capacity(9);
            zip_entries.push(("pass.json".to_string(), pass_json_bytes));
            zip_entries.push(("manifest.json".to_string(), manifest_bytes));
            zip_entries.push(("signature".to_string(), signature_bytes));
            zip_entries.extend(image_entries);

            package::zip_pkpass(&zip_entries)
        }
    }
    ```

    Then edit `ferro-wallet/src/lib.rs` and uncomment `pub use apple::ApplePassBuilder;`.
  </action>
  <verify>
    <automated>cargo build -p ferro-wallet &amp;&amp; cargo test -p ferro-wallet --lib &amp;&amp; cargo clippy -p ferro-wallet --all-targets -- -D warnings &amp;&amp; cargo fmt -p ferro-wallet -- --check &amp;&amp; grep -F 'pub struct ApplePassBuilder' ferro-wallet/src/apple/mod.rs &amp;&amp; grep -F 'pub fn build&lt;S: WalletSubject&gt;' ferro-wallet/src/apple/mod.rs &amp;&amp; grep -F 'pub use apple::ApplePassBuilder;' ferro-wallet/src/lib.rs &amp;&amp; cargo build --workspace</automated>
  </verify>
  <done>`ApplePassBuilder::new` and `::build` exist with the documented contract. `lib.rs` re-export restored. Full workspace builds. Existing unit tests still pass; integration test lands in PLAN-06.</done>
</task>

</tasks>

<threat_model>
This plan introduces the security-critical Apple signing pipeline. Cryptographic correctness is load-bearing.

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-151-Apple-MANIFEST | T | `apple/manifest.rs::build_manifest` | mitigate | ACC-1d unit test (`manifest_sha1_lowercase_hex`) pins SHA1 output against the known digest of `"hello"`. `manifest_is_deterministic_across_calls` test confirms byte-stability (via BTreeMap). End-to-end ACC-1j integration test (PLAN-06) confirms the manifest covers all 7 non-signature non-manifest entries. |
| T-151-Apple-SIGN | T | `apple/sign.rs::SigningMaterial::sign_detached` | mitigate | (a) `wwdr_stack.push(self.wwdr.clone())` is unconditional before `Pkcs7::sign` — verified by grep. (b) Flags are `Pkcs7Flags::DETACHED \| Pkcs7Flags::BINARY` — verified by grep. (c) Private key never escapes `SigningMaterial` — owned via `PKey<Private>`, no `Debug` impl exposes raw bytes. (d) PEM strings live in `AppleConfig` (passed by reference into `parse`) — caller is responsible for not logging them. |
| T-151-Apple-COLOR | D | `apple/manifest.rs::build_pass_json` (foreground colour branch) | mitigate | `auto_foreground` (PLAN-02 ACC-1f) deterministically returns white for dark backgrounds and dark slate for light ones; `labelColor` tracks `foregroundColor` per D-06. Wrong-luminance branches would produce illegible-but-installable passes, not security failures. |
| T-151-DEFAULT-CRED | I | `apple/sign.rs::SigningMaterial::parse` | accept | Parse takes `&str` PEM by reference; nothing is persisted. Tests in PLAN-06 mint crypto at runtime — no real Apple WWDR material in the repo. |
</threat_model>

<verification>
- `cargo test -p ferro-wallet --lib apple::manifest::tests::manifest_sha1_lowercase_hex` exits 0 (ACC-1d).
- `cargo test -p ferro-wallet --lib apple::manifest::tests` runs 3 tests, all pass.
- `cargo test -p ferro-wallet --lib apple::package::tests` runs 2 tests, all pass.
- `cargo test -p ferro-wallet --lib` runs every unit test from PLAN-01..05, all pass.
- `cargo build --workspace` exits 0.
- `cargo clippy --all --all-targets -- -D warnings` exits 0.
- `cargo fmt --all -- --check` exits 0.
- `grep -F 'pub use apple::ApplePassBuilder;' ferro-wallet/src/lib.rs` returns one match.
- `grep -F 'Pkcs7Flags::DETACHED | Pkcs7Flags::BINARY' ferro-wallet/src/apple/sign.rs` returns one match.
- `grep -F 'CompressionMethod::Stored' ferro-wallet/src/apple/package.rs` returns one match.
</verification>

<success_criteria>
PLAN-06 can write `tests/apple_integration.rs` against `ApplePassBuilder::new` and `::build`. The integration test mints a self-signed cert + reuses it as WWDR, builds a `.pkpass`, and asserts ZIP shape + pass.json field values.
</success_criteria>

<output>
After completion, create `.planning/phases/151-ferro-wallet-crate/151-05-SUMMARY.md` documenting (a) the exact 9-entry ZIP order produced by `build`, (b) the list of pass.json keys emitted, (c) the manifest input set (7 entries: pass.json + 6 images, NOT manifest.json or signature), and (d) confirmation that ACC-1d passed.
</output>

## PLANNING COMPLETE
