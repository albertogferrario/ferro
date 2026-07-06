//! Custom JWT middleware. Ferro's auth is session-based (RESEARCH §1, Pitfall 1),
//! so Conduit's `Authorization: Token <jwt>` scheme is hand-rolled here.

pub mod jwt_auth;
pub mod optional_jwt;

/// Authenticated user id, inserted into the request extension map by the JWT
/// middlewares. Handlers read it via `req.get::<UserId>()` (never `AuthUser<T>`,
/// which is session-bound — RESEARCH Pitfall 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserId(pub i64);

/// Shared header-parsing + decode logic for both JWT middlewares.
///
/// Returns `Some(UserId)` only for an `Authorization: Token <jwt>` header whose
/// JWT decodes and verifies against the current secret; `None` otherwise (no
/// header, wrong scheme, malformed/expired/bad-signature token).
pub fn extract_user_id(auth_header: Option<&str>) -> Option<UserId> {
    let token = auth_header?.strip_prefix("Token ")?;
    let claims = crate::jwt::decode_token(token, &crate::jwt::jwt_secret()).ok()?;
    Some(UserId(claims.sub))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwt::mint_token;

    fn header_for(user_id: i64) -> String {
        format!("Token {}", mint_token(user_id, "x@x.com", &crate::jwt::jwt_secret()))
    }

    /// No Authorization header → guest (None).
    #[test]
    fn no_header_is_none() {
        assert_eq!(extract_user_id(None), None);
    }

    /// Valid `Token <jwt>` → Some(UserId).
    #[test]
    fn valid_token_yields_user_id() {
        let h = header_for(7);
        assert_eq!(extract_user_id(Some(&h)), Some(UserId(7)));
    }

    /// Garbage/non-JWT token → None (never trusted).
    #[test]
    fn bad_token_is_none() {
        assert_eq!(extract_user_id(Some("Token not-a-real-jwt")), None);
        assert_eq!(extract_user_id(Some("Bearer something")), None);
    }
}
