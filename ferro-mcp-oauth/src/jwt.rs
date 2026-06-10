//! HS256 mint/decode (Plan 03).
//!
//! Mints and validates HS256 JWTs using `jsonwebtoken` v9. Claims include
//! `sub`, `tenant_id` (exact name — matches `JwtClaimResolver` in
//! `framework/src/tenant/resolver.rs`), `aud`, `iss`, `iat`, and `exp`.
//!
//! Security properties:
//! - Algorithm pinned to HS256 via `validation.algorithms = vec![Algorithm::HS256]`
//!   (T-06, T-07: rejects `alg=none` and RS256→HS256 confusion).
//! - Audience bound to `{APP_URL}/mcp` (T-08: audience confusion).
//! - `iss` and `aud` sourced from the same `OAuthConfig` (T-17: mix-up prevention).
//! - Zero clock-skew leeway for short-lived tokens.

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

/// Claims carried by an MCP access token.
///
/// Field `tenant_id` uses the EXACT name expected by `JwtClaimResolver`
/// (`framework/src/tenant/resolver.rs` line 211: `claims["tenant_id"].as_i64()`).
/// Changing this name silently breaks Phase 200 tenant resolution.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct McpTokenClaims {
    /// Subject: user ID as string.
    pub sub: String,
    /// Tenant ID. `None` for single-tenant apps (token remains valid; tenant check skipped).
    /// LOAD-BEARING name: must be `tenant_id` to match `JwtClaimResolver`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<i64>,
    /// Audience: `["{APP_URL}/mcp"]`.
    pub aud: Vec<String>,
    /// Issuer: `"{APP_URL}"`.
    pub iss: String,
    /// Issued-at (Unix timestamp).
    pub iat: i64,
    /// Expiry (Unix timestamp).
    pub exp: i64,
}

/// Builds claims for an MCP access token.
///
/// `aud` is set to `["{app_url}/mcp"]`, `iss` to `{app_url}`, `iat` to now,
/// `exp` to `now + ttl_secs`. Default TTL is 3600 s (D-02 short expiry).
pub fn build_claims(
    user_id: i64,
    tenant_id: Option<i64>,
    app_url: &str,
    ttl_secs: i64,
) -> McpTokenClaims {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_secs() as i64;
    McpTokenClaims {
        sub: user_id.to_string(),
        tenant_id,
        aud: vec![format!("{app_url}/mcp")],
        iss: app_url.to_string(),
        iat: now,
        exp: now + ttl_secs,
    }
}

/// Mints an HS256 JWT.
///
/// Uses `EncodingKey::from_secret(secret)` with `Algorithm::HS256`.
pub fn mint_token(claims: &McpTokenClaims, secret: &[u8]) -> Result<String, crate::OAuthError> {
    let header = Header::new(Algorithm::HS256);
    let key = EncodingKey::from_secret(secret);
    encode(&header, claims, &key).map_err(crate::OAuthError::Jwt)
}

/// Decodes and validates an HS256 JWT.
///
/// Security properties enforced here:
/// - `validation.algorithms = vec![Algorithm::HS256]` — pins algorithm; rejects `alg=none`
///   and RS256→HS256 confusion (T-06, T-07).
/// - `set_audience(&[expected_aud])` — exact audience match required (T-08).
/// - `validate_exp = true`, `leeway = 0` — expired tokens are rejected immediately.
///
/// Returns the raw `jsonwebtoken::errors::Error` so `validate_bearer` can map
/// `ErrorKind` → HTTP status (see `validate.rs`).
pub fn decode_token(
    token: &str,
    secret: &[u8],
    expected_aud: &str,
) -> Result<McpTokenClaims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(Algorithm::HS256);
    // T-06/T-07: algorithm pin — rejects alg=none and any client-supplied alg
    validation.algorithms = vec![Algorithm::HS256];
    // T-08: audience binding
    validation.set_audience(&[expected_aud]);
    validation.validate_exp = true;
    validation.leeway = 0;
    let key = DecodingKey::from_secret(secret);
    Ok(decode::<McpTokenClaims>(token, &key, &validation)?.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &[u8] = b"test-secret-that-is-at-least-32-bytes-long!!";
    const APP_URL: &str = "https://example.com";

    fn make_claims() -> McpTokenClaims {
        build_claims(42, Some(7), APP_URL, 3600)
    }

    /// mint_token then decode_token round-trips; decoded claims equal input.
    #[test]
    fn mint_decode_round_trip() {
        let claims = make_claims();
        let token = mint_token(&claims, SECRET).expect("mint failed");
        let decoded =
            decode_token(&token, SECRET, &format!("{APP_URL}/mcp")).expect("decode failed");
        assert_eq!(decoded.sub, "42");
        assert_eq!(decoded.tenant_id, Some(7));
        assert_eq!(decoded.iss, APP_URL);
        assert_eq!(decoded.aud, vec![format!("{APP_URL}/mcp")]);
    }

    /// decode_token with a token signed by a different secret returns an error.
    #[test]
    fn wrong_secret_returns_error() {
        let claims = make_claims();
        let token = mint_token(&claims, SECRET).expect("mint failed");
        let other_secret = b"completely-different-secret-at-least-32-bytes";
        let result = decode_token(&token, other_secret, &format!("{APP_URL}/mcp"));
        assert!(result.is_err(), "expected error for wrong secret");
    }

    /// decode_token with exp in the past returns an error.
    #[test]
    fn expired_token_returns_error() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let expired_claims = McpTokenClaims {
            sub: "42".to_string(),
            tenant_id: Some(7),
            aud: vec![format!("{APP_URL}/mcp")],
            iss: APP_URL.to_string(),
            iat: now - 7200,
            exp: now - 3600, // expired 1 hour ago
        };
        let token = mint_token(&expired_claims, SECRET).expect("mint failed");
        let result = decode_token(&token, SECRET, &format!("{APP_URL}/mcp"));
        assert!(result.is_err(), "expected error for expired token");
        assert!(
            matches!(
                result.unwrap_err().kind(),
                jsonwebtoken::errors::ErrorKind::ExpiredSignature
            ),
            "expected ExpiredSignature"
        );
    }

    /// decode_token with aud != expected returns an error.
    #[test]
    fn wrong_audience_returns_error() {
        let claims = make_claims();
        let token = mint_token(&claims, SECRET).expect("mint failed");
        let result = decode_token(&token, SECRET, "https://other.com/mcp");
        assert!(result.is_err(), "expected error for wrong audience");
        assert!(
            matches!(
                result.unwrap_err().kind(),
                jsonwebtoken::errors::ErrorKind::InvalidAudience
            ),
            "expected InvalidAudience"
        );
    }

    /// The minted claims serialize the tenant field under the EXACT key `tenant_id`.
    #[test]
    fn tenant_claim_key_is_exactly_tenant_id() {
        let claims = make_claims();
        let json = serde_json::to_value(&claims).expect("serialize failed");
        assert!(
            json.get("tenant_id").is_some(),
            "serialized JSON must have key `tenant_id`, got: {json}"
        );
        assert_eq!(
            json["tenant_id"].as_i64(),
            Some(7),
            "tenant_id value must be 7"
        );
    }
}
