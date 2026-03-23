pub mod events;

use crate::Error;

/// Verifies a WhatsApp webhook signature against the raw request body.
///
/// Meta signs the raw body with the app secret using HMAC-SHA256 and places the
/// result in the `X-Hub-Signature-256` header with the format `sha256=<hex>`.
///
/// Uses constant-time comparison to prevent timing attacks.
///
/// # Errors
///
/// Returns [`Error::WebhookVerification`] when:
/// - The header lacks the `sha256=` prefix
/// - The HMAC does not match
pub fn verify_whatsapp_webhook(
    raw_body: &[u8],
    signature_header: &str,
    app_secret: &str,
) -> Result<(), Error> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let expected_sig = signature_header
        .strip_prefix("sha256=")
        .ok_or_else(|| Error::WebhookVerification("Missing sha256= prefix".into()))?;

    let mut mac =
        Hmac::<Sha256>::new_from_slice(app_secret.as_bytes()).expect("HMAC accepts any key size");
    mac.update(raw_body);
    let computed = hex::encode(mac.finalize().into_bytes());

    if !constant_time_eq(computed.as_bytes(), expected_sig.as_bytes()) {
        return Err(Error::WebhookVerification("Signature mismatch".into()));
    }

    Ok(())
}

/// Computes an HMAC-SHA256 signature for a raw body using the given secret.
///
/// Returns the signature in the `sha256=<hex>` format expected by
/// [`verify_whatsapp_webhook`]. Use this in test suites to generate valid
/// signatures without making live HTTP calls.
pub fn signed_whatsapp_payload(raw_body: &[u8], app_secret: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac =
        Hmac::<Sha256>::new_from_slice(app_secret.as_bytes()).expect("HMAC accepts any key size");
    mac.update(raw_body);
    let digest = hex::encode(mac.finalize().into_bytes());
    format!("sha256={digest}")
}

/// Constant-time byte slice comparison. Returns true if slices are equal.
///
/// Uses XOR accumulation so that the comparison time does not depend on
/// where the first differing byte is, preventing timing attacks.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test_app_secret";
    const BODY: &[u8] = b"{\"object\":\"whatsapp_business_account\"}";

    #[test]
    fn verify_webhook_valid() {
        let sig = signed_whatsapp_payload(BODY, SECRET);
        let result = verify_whatsapp_webhook(BODY, &sig, SECRET);
        assert!(result.is_ok(), "expected Ok but got: {result:?}");
    }

    #[test]
    fn verify_webhook_tampered() {
        let sig = signed_whatsapp_payload(BODY, SECRET);
        let tampered = b"{\"object\":\"hacked\"}";
        let result = verify_whatsapp_webhook(tampered, &sig, SECRET);
        assert!(
            matches!(result, Err(Error::WebhookVerification(_))),
            "expected WebhookVerification error but got: {result:?}"
        );
    }

    #[test]
    fn verify_webhook_wrong_secret() {
        let sig = signed_whatsapp_payload(BODY, SECRET);
        let result = verify_whatsapp_webhook(BODY, &sig, "wrong_secret");
        assert!(
            matches!(result, Err(Error::WebhookVerification(_))),
            "expected WebhookVerification error but got: {result:?}"
        );
    }

    #[test]
    fn verify_webhook_bad_prefix() {
        // Header without "sha256=" prefix
        let result = verify_whatsapp_webhook(BODY, "abc123def456", SECRET);
        assert!(
            matches!(result, Err(Error::WebhookVerification(_))),
            "expected WebhookVerification error but got: {result:?}"
        );
    }

    #[test]
    fn signed_payload_roundtrip() {
        let body = b"meta webhook payload";
        let sig = signed_whatsapp_payload(body, SECRET);
        assert!(
            sig.starts_with("sha256="),
            "signature must have sha256= prefix"
        );
        let result = verify_whatsapp_webhook(body, &sig, SECRET);
        assert!(result.is_ok(), "roundtrip must verify: {result:?}");
    }
}
