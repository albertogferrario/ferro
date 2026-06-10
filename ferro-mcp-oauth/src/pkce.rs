//! PKCE S256 verification (Plan 03).
//!
//! Implements RFC 7636: `code_challenge = BASE64URL(SHA256(code_verifier))`.
//! Uses `sha2` for hashing and `subtle::ConstantTimeEq` to prevent timing
//! oracles on the challenge comparison.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::Rng;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// Generates a high-entropy authorization code (256 bits, URL-safe base64, ~43 chars).
///
/// Used for authorization codes stored server-side. Also the same encoding as
/// PKCE code verifiers (RFC 7636 §4.1).
pub fn generate_auth_code() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Verifies a PKCE S256 challenge: `BASE64URL(SHA256(code_verifier)) == stored_challenge`.
///
/// Uses constant-time comparison (`subtle::ConstantTimeEq`) to prevent timing
/// oracles (T-11). Returns `true` if and only if the verifier matches the challenge.
///
/// Note: rejection of `code_challenge_method=plain` happens at `/authorize` (Plan 04),
/// not here. This function only computes and compares S256.
pub fn verify_s256(code_verifier: &str, stored_challenge: &str) -> bool {
    let hash = Sha256::digest(code_verifier.as_bytes());
    let recomputed = URL_SAFE_NO_PAD.encode(hash);
    recomputed.as_bytes().ct_eq(stored_challenge.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 7636 §4.2: verify_s256(v, BASE64URL(SHA256(v))) must return true.
    #[test]
    fn correct_verifier_matches_stored_challenge() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let hash = Sha256::digest(verifier.as_bytes());
        let challenge = URL_SAFE_NO_PAD.encode(hash);
        assert!(verify_s256(verifier, &challenge));
    }

    /// A different verifier must not match the stored challenge.
    #[test]
    fn wrong_verifier_does_not_match() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let other = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let hash = Sha256::digest(other.as_bytes());
        let challenge = URL_SAFE_NO_PAD.encode(hash);
        assert!(!verify_s256(verifier, &challenge));
    }

    /// generate_auth_code returns a ~43-char URL-safe string; two calls differ.
    #[test]
    fn generate_auth_code_is_url_safe_and_unique() {
        let code1 = generate_auth_code();
        let code2 = generate_auth_code();
        // 32 bytes → 43 base64url chars (no padding)
        assert_eq!(code1.len(), 43, "expected 43 chars, got {}", code1.len());
        // URL-safe alphabet only
        assert!(
            code1.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'),
            "non-URL-safe char in: {code1}"
        );
        // Two calls yield different values (birthday probability ~2^-256)
        assert_ne!(code1, code2);
    }
}
