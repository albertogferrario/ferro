//! Apple Wallet `.pkpass` issuance — SHA1 manifest + PKCS#7 detached signature + ZIP packaging.

pub mod manifest;
pub mod package;
pub mod sign;

use crate::config::AppleConfig;
use crate::images;
use crate::subject::WalletSubject;
use crate::WalletError;

/// Issues Apple Wallet `.pkpass` files for any [`WalletSubject`].
///
/// Constructed once per process (or per signing-material rotation) via
/// [`ApplePassBuilder::new`], which parses + retains the signing cert,
/// private key, and WWDR intermediate. [`ApplePassBuilder::build`] then
/// composes a `.pkpass` ZIP per subject: `pass.json` → image set → SHA1
/// manifest → PKCS#7 detached signature → ZIP packaging (Stored, never
/// Deflated — Apple rejects deflated entries).
pub struct ApplePassBuilder {
    pub(crate) pass_type_id: String,
    pub(crate) team_id: String,
    pub(crate) app_name: String,
    pub(crate) signing: sign::SigningMaterial,
}

impl ApplePassBuilder {
    /// Parse signing material from the provided [`AppleConfig`] and capture
    /// the pass-type / team identifiers used for every issued pass.
    ///
    /// The private key may be passphrase-protected; `cfg.key_password` is
    /// forwarded to [`sign::SigningMaterial::parse`].
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
    ///
    /// The output contains exactly nine entries in this order:
    /// `pass.json`, `manifest.json`, `signature`, `logo.png`, `logo@2x.png`,
    /// `logo@3x.png`, `icon.png`, `icon@2x.png`, `icon@3x.png`.
    ///
    /// The manifest digests `pass.json` + the six image entries only;
    /// `manifest.json` and `signature` are not part of the digest map.
    pub fn build<S: WalletSubject>(&self, s: &S) -> Result<Vec<u8>, WalletError> {
        // 1. pass.json
        let pass_json_bytes = manifest::build_pass_json(self, s)?;

        // 2. images (optional logo set + icon set + optional strip set).
        // Apple Wallet treats `logo.png` as optional — when omitted, `logoText`
        // renders left-aligned from the header edge instead of being offset by
        // the reserved logo slot. `icon.png` IS required (used in notifications
        // and Mail previews), so when no logo is supplied we fall back to a 1×1
        // transparent icon source.
        let branding = s.branding();
        let mut image_entries: Vec<(String, Vec<u8>)> = Vec::new();
        let icon_source: Vec<u8> = match (branding.icon_png_bytes.as_deref(), branding.logo_png_bytes.as_deref()) {
            (Some(_), _) => Vec::new(),                        // icon explicit, no fallback needed
            (None, Some(logo)) => logo.to_vec(),               // derive icon from logo
            (None, None) => images::transparent_1x1_png()?,    // both absent — synthesise placeholder
        };
        if let Some(logo) = branding.logo_png_bytes.as_deref() {
            image_entries.extend(images::apple_logo_set(logo)?);
        }
        let icon_entries = images::apple_icon_set(
            branding.icon_png_bytes.as_deref(),
            &icon_source,
        )?;
        image_entries.extend(icon_entries);
        // Strip image (optional) — gives eventTicket passes the banner/ticket look.
        if let Some(hero) = branding.hero_png_bytes.as_deref() {
            let strip_entries = images::apple_strip_set(hero)?;
            image_entries.extend(strip_entries);
        }

        // 3. manifest = SHA1 of pass.json + each image entry (NOT manifest itself, NOT signature).
        let mut manifest_inputs: Vec<(String, Vec<u8>)> =
            vec![("pass.json".to_string(), pass_json_bytes.clone())];
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
