//! HAND-ROLLED — not framework-provided. Ferro auth is session-based (RESEARCH §1).
//! Uses a benchmark-only HS256 secret; NOT production-grade.
//!
//! This module is the ONLY non-framework-provided capability in ferro-conduit. It is
//! kept self-contained so the static-compression report (Plan 07) can count it separately.

use jsonwebtoken::{
    decode, encode, get_current_timestamp, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

/// Claims carried by a Conduit JWT.
///
/// `sub` is the user id; `email` is convenience for the Conduit user envelope.
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject: user id.
    pub sub: i64,
    /// User email (convenience for the user envelope).
    pub email: String,
    /// Expiry (Unix timestamp, seconds).
    pub exp: usize,
}

/// Default token lifetime: 24 hours.
const DEFAULT_TTL_SECS: i64 = 86_400;

/// Benchmark-only HS256 secret. Reads `JWT_SECRET`, falling back to a constant.
/// NOT production-grade — this is a local benchmark app, not a deployment.
pub fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "benchmark-only-not-production-grade".into())
}

/// Mints an HS256 JWT with the default 24h TTL.
pub fn mint_token(user_id: i64, email: &str, secret: &str) -> String {
    mint_token_with_ttl(user_id, email, secret, DEFAULT_TTL_SECS)
}

/// Mints an HS256 JWT with an explicit TTL (seconds). A negative TTL yields an
/// already-expired token (used by the expiry test).
pub fn mint_token_with_ttl(user_id: i64, email: &str, secret: &str, ttl_secs: i64) -> String {
    let exp = (get_current_timestamp() as i64 + ttl_secs).max(0) as usize;
    let claims = JwtClaims {
        sub: user_id,
        email: email.to_string(),
        exp,
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("JWT encoding failed")
}

/// Decodes and validates an HS256 JWT. Default `Validation` enforces `exp`,
/// so expired tokens are rejected; signature mismatch is rejected too.
pub fn decode_token(token: &str, secret: &str) -> Result<JwtClaims, jsonwebtoken::errors::Error> {
    decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|d| d.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "benchmark-secret";

    /// mint_token then decode_token round-trips; sub survives.
    #[test]
    fn round_trip() {
        let t = mint_token(42, "alice@x.com", SECRET);
        let c = decode_token(&t, SECRET).unwrap();
        assert_eq!(c.sub, 42);
        assert_eq!(c.email, "alice@x.com");
    }

    /// A token minted with an expiry in the past (beyond jsonwebtoken's default
    /// 60s leeway) is rejected.
    #[test]
    fn expired_token_rejected() {
        let t = mint_token_with_ttl(1, "bob@x.com", SECRET, -120);
        assert!(decode_token(&t, SECRET).is_err());
    }

    /// A token minted with secret A fails to decode with secret B.
    #[test]
    fn bad_signature_rejected() {
        let t = mint_token(7, "carol@x.com", "secret-a");
        assert!(decode_token(&t, "secret-b").is_err());
    }
}
