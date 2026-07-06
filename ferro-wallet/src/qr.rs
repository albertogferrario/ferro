//! QR code generation — PNG bytes + base64 data URI helper.
//!
//! Wraps `qrcode-generator` with the wallet-builder error type. `png` returns
//! raw PNG bytes; `data_uri` wraps them in a `data:image/png;base64,...` URI
//! suitable for inline embedding (e.g. Google Wallet `eventTicketObject.barcode`).

use base64::{engine::general_purpose, Engine as _};
use qrcode_generator::QrCodeEcc;

use crate::WalletError;

/// Generate a QR code as PNG bytes encoding `data` at `size` pixels per side.
///
/// Uses error-correction level `Medium`.
pub fn png(data: &str, size: u32) -> Result<Vec<u8>, WalletError> {
    qrcode_generator::to_png_to_vec(data, QrCodeEcc::Medium, size as usize)
        .map_err(|e| WalletError::Qr(format!("png generate: {e}")))
}

/// Generate a QR code as a `data:image/png;base64,...` URI.
pub fn data_uri(data: &str, size: u32) -> Result<String, WalletError> {
    let bytes = png(data, size)?;
    let b64 = general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:image/png;base64,{b64}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose;

    /// PNG file magic — first 8 bytes of every valid PNG.
    const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

    #[test]
    fn png_starts_with_png_magic() {
        // ACC-1h
        let bytes = png("hello", 200).expect("qr::png");
        assert!(
            bytes.len() >= 8,
            "qr png bytes must be at least 8 bytes long, got {}",
            bytes.len()
        );
        assert_eq!(
            &bytes[..8],
            &PNG_MAGIC,
            "qr png bytes must start with PNG magic"
        );
    }

    #[test]
    fn data_uri_starts_with_data_image_png_base64() {
        let uri = data_uri("hello", 200).expect("qr::data_uri");
        assert!(
            uri.starts_with("data:image/png;base64,"),
            "expected data:image/png;base64, prefix, got: {prefix}",
            prefix = &uri[..uri.len().min(40)]
        );
    }

    #[test]
    fn data_uri_payload_decodes_to_png_bytes() {
        let png_bytes = png("hello", 200).expect("qr::png");
        let uri = data_uri("hello", 200).expect("qr::data_uri");
        let payload = uri
            .strip_prefix("data:image/png;base64,")
            .expect("data uri prefix");
        let decoded = general_purpose::STANDARD
            .decode(payload)
            .expect("base64 decode");
        assert_eq!(
            decoded, png_bytes,
            "base64 payload must decode to the same bytes as qr::png"
        );
    }
}
