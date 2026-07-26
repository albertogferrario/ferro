//! HMAC-signed channel subscription authorization (Pusher-style).
//!
//! Private and presence channels are gated by a signature the server issues from
//! the session-authenticated `/broadcasting/auth` endpoint and verifies when the
//! client subscribes over the WebSocket. Because the signing secret never leaves
//! the server, a client cannot forge a subscription to a channel (or, for
//! presence channels, a `user_id`) it was not authorized for.
//!
//! Signed payload:
//! - private channel:  `"{socket_id}:{channel}"`
//! - presence channel: `"{socket_id}:{channel}:{user_id}"`  (binds the identity)

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// The exact byte string that gets signed for a subscription.
fn payload(socket_id: &str, channel: &str, user_id: Option<&str>) -> String {
    match user_id {
        Some(uid) => format!("{socket_id}:{channel}:{uid}"),
        None => format!("{socket_id}:{channel}"),
    }
}

/// Compute the hex HMAC-SHA256 signature for a channel subscription.
pub(crate) fn sign(secret: &str, socket_id: &str, channel: &str, user_id: Option<&str>) -> String {
    // HMAC accepts a key of any length, so this never errors.
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key");
    mac.update(payload(socket_id, channel, user_id).as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Constant-time verification of a client-provided hex signature.
pub(crate) fn verify(
    secret: &str,
    socket_id: &str,
    channel: &str,
    user_id: Option<&str>,
    provided_hex: &str,
) -> bool {
    let Ok(provided) = hex::decode(provided_hex) else {
        return false;
    };
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key");
    mac.update(payload(socket_id, channel, user_id).as_bytes());
    mac.verify_slice(&provided).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_then_verify_roundtrips() {
        let sig = sign("s3cret", "sock-1", "private-user.7", None);
        assert!(verify("s3cret", "sock-1", "private-user.7", None, &sig));
    }

    #[test]
    fn wrong_secret_fails() {
        let sig = sign("s3cret", "sock-1", "private-user.7", None);
        assert!(!verify("other", "sock-1", "private-user.7", None, &sig));
    }

    #[test]
    fn different_channel_or_socket_fails() {
        let sig = sign("s3cret", "sock-1", "private-user.7", None);
        assert!(!verify("s3cret", "sock-2", "private-user.7", None, &sig));
        assert!(!verify("s3cret", "sock-1", "private-user.8", None, &sig));
    }

    #[test]
    fn presence_identity_is_bound() {
        // A token issued for user 7 must not authorize a subscription claiming user 8.
        let sig = sign("s3cret", "sock-1", "presence-nearby", Some("7"));
        assert!(verify(
            "s3cret",
            "sock-1",
            "presence-nearby",
            Some("7"),
            &sig
        ));
        assert!(!verify(
            "s3cret",
            "sock-1",
            "presence-nearby",
            Some("8"),
            &sig
        ));
    }

    #[test]
    fn garbage_signature_fails() {
        assert!(!verify("s3cret", "sock-1", "private-x", None, "not-hex"));
        assert!(!verify("s3cret", "sock-1", "private-x", None, ""));
    }
}
