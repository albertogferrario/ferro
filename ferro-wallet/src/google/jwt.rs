//! Google Wallet save-JWT signing — RS256 over a `SaveClaims` envelope.
//!
//! The JWT is the wire-format Google Wallet expects from the
//! `https://pay.google.com/gp/v/save/{jwt}` save endpoint. Its `payload.eventTicketObjects[0]`
//! is the actual pass content; the surrounding envelope is the auth context Google validates
//! at click time. See D-08 for the claim shape and D-07 for the fixed `pass_type_id_default()`.

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;

use super::GoogleWalletBuilder;
use crate::WalletError;

/// v1 pass type identifier suffix used to derive the Google Wallet `classId`
/// (`"{issuer_id}.{pass_type_id_default()}"`, with any `.` in the suffix replaced
/// by `_` per D-07). Returns `"booking"` — has no dots, so the substitution is a
/// no-op today but kept future-proof.
pub const fn pass_type_id_default() -> &'static str {
    "booking"
}

/// Internal `SaveClaims` envelope serialized as the JWT payload (D-08).
///
/// Fields borrow from the builder + the runtime-constructed `event_ticket_object`
/// for zero-copy serialization. `iat` is the only timestamp claim — Google's save
/// endpoint validates the JWT at user-click time, not on a server-side `exp`.
#[allow(dead_code)] // Wired up by Task 3's `GoogleWalletBuilder::save_jwt`.
#[derive(Serialize)]
struct SaveClaims<'a> {
    iss: &'a str,
    aud: &'a str,
    typ: &'a str,
    iat: i64,
    origins: Vec<&'a str>,
    payload: serde_json::Value,
}

/// Sign the save JWT for a pre-built `eventTicketObject`.
///
/// The builder supplies `service_account_email` (→ `iss`), `app_url` (→ single-element
/// `origins`), and `private_key_pem` (→ RS256 signing key). All other claims are fixed:
/// `aud = "google"`, `typ = "savetowallet"`, `iat = <unix-now>`. The payload wraps the
/// supplied object as `{ "eventTicketObjects": [<object>] }` — v1 always emits exactly
/// one object per save link.
///
/// # Errors
///
/// - `WalletError::GoogleJwt("private key parse: …")` if `private_key_pem` is not a valid RSA PEM.
/// - `WalletError::GoogleJwt("encode: …")` if `jsonwebtoken::encode` itself fails.
#[allow(dead_code)] // Wired up by Task 3's `GoogleWalletBuilder::save_jwt`.
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
    encode(&header, &claims, &key).map_err(|e| WalletError::GoogleJwt(format!("encode: {e}")))
}

/// Compose the Google Wallet save URL for a previously signed JWT.
///
/// Pure formatting — does NOT validate the JWT segment structure. The caller is
/// responsible for passing a JWT produced by [`sign_save_jwt`].
pub fn save_url(jwt: &str) -> String {
    format!("https://pay.google.com/gp/v/save/{jwt}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ACC-1i: canonical save URL formatting.
    #[test]
    fn save_url_format() {
        assert_eq!(
            save_url("abc.def.ghi"),
            "https://pay.google.com/gp/v/save/abc.def.ghi"
        );
    }

    /// Edge case: empty JWT still yields the prefix-only URL (does not validate).
    #[test]
    fn save_url_empty_jwt() {
        assert_eq!(save_url(""), "https://pay.google.com/gp/v/save/");
    }

    /// D-07: `pass_type_id_default()` returns the v1 fixed `"booking"` suffix.
    #[test]
    fn pass_type_id_default_is_booking() {
        assert_eq!(pass_type_id_default(), "booking");
    }
}
