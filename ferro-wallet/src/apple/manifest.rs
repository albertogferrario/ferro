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
        TextColorMode::Light => RgbColor {
            r: 255,
            g: 255,
            b: 255,
        },
        TextColorMode::Dark => RgbColor { r: 0, g: 0, b: 0 },
    };
    // D-06: labelColor tracks foreground in v1.
    let label_color = foreground;

    let mut pass = serde_json::Map::new();
    pass.insert("formatVersion".into(), serde_json::json!(1));
    pass.insert(
        "passTypeIdentifier".into(),
        serde_json::json!(builder.pass_type_id),
    );
    pass.insert("teamIdentifier".into(), serde_json::json!(builder.team_id));
    pass.insert("serialNumber".into(), serde_json::json!(subject.serial()));
    pass.insert("organizationName".into(), serde_json::json!(org_name));
    pass.insert(
        "description".into(),
        serde_json::json!(builder.app_name.clone()),
    );
    pass.insert(
        "backgroundColor".into(),
        serde_json::json!(branding.background_color.css_rgb()),
    );
    pass.insert(
        "foregroundColor".into(),
        serde_json::json!(foreground.css_rgb()),
    );
    pass.insert(
        "labelColor".into(),
        serde_json::json!(label_color.css_rgb()),
    );
    if let Some(logo_text) = branding.logo_text.clone() {
        pass.insert("logoText".into(), serde_json::json!(logo_text));
    }
    pass.insert(
        "barcodes".into(),
        serde_json::json!([{
            "format": "PKBarcodeFormatQR",
            "message": subject.barcode_token(),
            "messageEncoding": "iso-8859-1"
        }]),
    );
    if let Some(t) = subject.relevant_at() {
        pass.insert("relevantDate".into(), serde_json::json!(t.to_rfc3339()));
    }
    if let Some(t) = subject.expires_at() {
        pass.insert("expirationDate".into(), serde_json::json!(t.to_rfc3339()));
    }
    let locs: Vec<serde_json::Value> = subject
        .locations()
        .iter()
        .map(|g| {
            let mut obj = serde_json::Map::new();
            obj.insert("latitude".into(), serde_json::json!(g.latitude));
            obj.insert("longitude".into(), serde_json::json!(g.longitude));
            if let Some(t) = &g.relevant_text {
                obj.insert("relevantText".into(), serde_json::json!(t));
            }
            serde_json::Value::Object(obj)
        })
        .collect();
    if !locs.is_empty() {
        pass.insert("locations".into(), serde_json::Value::Array(locs));
    }

    // Field array key per spec §3 — `eventTicket` / `boardingPass` / `generic` / `coupon`.
    // `boardingPass` carries an extra required `transitType` selector.
    let kind = subject.pass_kind();
    let kind_key = match &kind {
        crate::subject::PassKind::EventTicket => "eventTicket",
        crate::subject::PassKind::BoardingPass(_) => "boardingPass",
        crate::subject::PassKind::Generic => "generic",
        crate::subject::PassKind::Coupon => "coupon",
    };
    let serialise_field = |f: &crate::subject::Field| -> serde_json::Value {
        let alignment = match f.alignment {
            FieldAlignment::Left => "PKTextAlignmentLeft",
            FieldAlignment::Center => "PKTextAlignmentCenter",
            FieldAlignment::Right => "PKTextAlignmentRight",
            FieldAlignment::Natural => "PKTextAlignmentNatural",
        };
        serde_json::json!({
            "key": f.key,
            "label": f.label,
            "value": f.value,
            "textAlignment": alignment,
        })
    };
    let mut fields = serde_json::Map::new();
    fields.insert(
        "primaryFields".into(),
        serde_json::Value::Array(vec![serialise_field(&subject.primary())]),
    );
    fields.insert(
        "secondaryFields".into(),
        serde_json::Value::Array(subject.secondary().iter().map(serialise_field).collect()),
    );
    fields.insert(
        "auxiliaryFields".into(),
        serde_json::Value::Array(subject.auxiliary().iter().map(serialise_field).collect()),
    );
    fields.insert(
        "backFields".into(),
        serde_json::Value::Array(subject.back().iter().map(serialise_field).collect()),
    );
    if let crate::subject::PassKind::BoardingPass(transit) = &kind {
        fields.insert(
            "transitType".into(),
            serde_json::Value::String(transit.as_apple_str().to_string()),
        );
    }
    pass.insert(kind_key.to_string(), serde_json::Value::Object(fields));

    serde_json::to_vec(&serde_json::Value::Object(pass))
        .map_err(|e| WalletError::ApplePackage(format!("pass.json serialise: {e}")))
}

pub(crate) fn build_manifest(entries: &[(String, Vec<u8>)]) -> Result<Vec<u8>, WalletError> {
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    for (name, bytes) in entries {
        map.insert(name.clone(), sha1_hex_lower(bytes));
    }
    serde_json::to_vec(&map).map_err(|e| WalletError::ApplePackage(format!("manifest json: {e}")))
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
        assert!(
            alpha_pos < zebra_pos,
            "alpha.png should sort before zebra.png in manifest"
        );
    }
}
