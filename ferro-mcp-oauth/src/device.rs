//! RFC 8628 Device Authorization Grant — store type, cache-key helpers, and code primitives.
//!
//! This module provides the [`DeviceGrant`] ephemeral cache record, the
//! [`DeviceGrantStatus`] state-machine enum, two cache-key helpers, and three code
//! generation/normalization functions consumed by the device authorization handlers
//! (Plans 03 and 04).
//!
//! ## Cache layout
//!
//! Each device grant occupies two cache keys:
//! - `mcp:device:{device_code}` → full [`DeviceGrant`] record (polled by the client).
//! - `mcp:usercode:{normalized_user_code}` → `device_code` string (pointer; used by the
//!   verification page to resolve a user-entered code to the grant).
//!
//! ## Status transitions
//!
//! `Pending` → `Approved` (user consents at verification page) | `Denied` (user denies).
//! `user_id` and `tenant_id` remain `None` until the `Approved` transition captures them
//! from `Auth::id()` and `current_tenant()`.
//! `last_polled_at` is updated on every token poll for `slow_down` enforcement (D-05).

use serde::{Deserialize, Serialize};

// ── Constants ─────────────────────────────────────────────────────────────────

/// TTL for both device-code cache entries (RFC 8628 recommends ~10 min).
pub const DEVICE_CODE_TTL: std::time::Duration = std::time::Duration::from_secs(600);

/// Minimum polling interval in seconds that clients must respect (RFC 8628 §3.5).
pub const DEVICE_INTERVAL_SECS: i64 = 5;

/// RFC 8628 §6.1 recommended charset: 20 unambiguous uppercase consonants.
/// No vowels (avoids profanity); no digits (avoids visual confusion with `0`/`O`, `1`/`I`).
const USER_CODE_CHARSET: &[u8] = b"BCDFGHJKLMNPQRSTVWXZ";

// ── DeviceGrantStatus ─────────────────────────────────────────────────────────

/// State of a device authorization grant (RFC 8628 §3.5).
///
/// Serialized as snake_case (`"pending"`, `"approved"`, `"denied"`) in the cache record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceGrantStatus {
    /// The grant has been issued but not yet acted on by the user.
    Pending,
    /// The user consented; `user_id` and `tenant_id` are bound on the record.
    Approved,
    /// The user explicitly denied the authorization request.
    Denied,
}

// ── DeviceGrant ───────────────────────────────────────────────────────────────

/// Ephemeral device authorization grant stored in `ferro-cache` with [`DEVICE_CODE_TTL`].
///
/// Two cache keys per grant:
/// - `mcp:device:{device_code}` → this record (full state; keyed by the opaque device_code
///   the polling client sends to `POST /token`).
/// - `mcp:usercode:{normalized_user_code}` → `device_code` string (pointer used by the
///   verification page to resolve the human-entered short code back to this record).
///
/// ## Status transitions
///
/// `Pending` (initial) → `Approved` when the user consents at `POST /device`, at which
/// point `user_id` and `tenant_id` are captured from the session and written to this record.
/// `Pending` → `Denied` when the user explicitly denies. Both transitions are terminal.
///
/// ## Fields
///
/// - `user_id`: `None` until `Approved`; set from `Auth::id()` at verification.
/// - `tenant_id`: `None` until `Approved`; set from `current_tenant()` at verification.
/// - `last_polled_at`: updated on every `POST /token` poll; compared against
///   [`DEVICE_INTERVAL_SECS`] to enforce `slow_down` (RFC 8628 §3.5).
/// - `normalized_user_code`: stored so the token handler can forget the
///   `mcp:usercode:{…}` pointer key when issuing the token (get-then-forget discipline,
///   T-199-02).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceGrant {
    /// The registered client that initiated this flow.
    pub client_id: String,
    /// Current state of the grant (Pending → Approved | Denied).
    pub status: DeviceGrantStatus,
    /// Authenticated user id; `None` until the Approved transition.
    pub user_id: Option<i64>,
    /// Tenant id at approval time; `None` until the Approved transition or for
    /// single-tenant apps.
    pub tenant_id: Option<i64>,
    /// Unix timestamp (seconds) when the grant was created.
    pub created_at: i64,
    /// Unix timestamp (seconds) of the most recent token poll; `None` = never polled.
    pub last_polled_at: Option<i64>,
    /// Normalized user code (uppercase, no hyphens/spaces) stored so the token handler
    /// can forget the `mcp:usercode:{…}` pointer entry on token issuance.
    pub normalized_user_code: String,
}

// ── Cache-key helpers ─────────────────────────────────────────────────────────

/// Returns the primary cache key for a device grant: `mcp:device:{device_code}`.
///
/// Used by the polling client (`POST /token`) and the verification page (`POST /device`)
/// to read and update the [`DeviceGrant`] record.
pub fn device_cache_key(device_code: &str) -> String {
    format!("mcp:device:{device_code}")
}

/// Returns the pointer cache key for a user code: `mcp:usercode:{normalized_user_code}`.
///
/// Stores the opaque `device_code` string so the verification page can resolve a
/// human-entered (and normalized) user code to the correct [`DeviceGrant`].
/// Forgotten by the token handler alongside the primary key on token issuance (T-199-02).
pub fn usercode_cache_key(normalized_user_code: &str) -> String {
    format!("mcp:usercode:{normalized_user_code}")
}

// ── Code generation and normalization ─────────────────────────────────────────

/// Generates a high-entropy opaque device code (256-bit URL-safe random string).
///
/// Delegates to [`crate::pkce::generate_auth_code`] — identical entropy and encoding
/// to authorization codes. Never shown to the user; sent directly to the polling client
/// over TLS in the `POST /device_authorization` response. (RFC 8628 §3.2)
pub fn generate_device_code() -> String {
    crate::pkce::generate_auth_code()
}

/// Generates a short human-typeable user code in `XXXX-XXXX` format.
///
/// Samples 8 characters uniformly at random from the RFC 8628 §6.1 recommended charset
/// (`BCDFGHJKLMNPQRSTVWXZ` — 20 unambiguous uppercase consonants), then groups them as
/// `XXXX-XXXX` for readability. Keyspace: 20^8 ≈ 2.56 × 10^10 combinations.
///
/// The hyphen is for display only; normalization strips it before cache lookup.
pub fn generate_user_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let chars: String = (0..8)
        .map(|_| USER_CODE_CHARSET[rng.gen_range(0..USER_CODE_CHARSET.len())] as char)
        .collect();
    format!("{}-{}", &chars[..4], &chars[4..])
}

/// Normalizes a user-entered code for cache lookup: uppercase and strip hyphens/spaces.
///
/// Accepts any casing and ignores the optional hyphen separator, so `wdjb-mfxg`,
/// `WDJB-MFXG`, `WDJBMFXG`, and `wdjb mfxg` all normalize to the same key `WDJBMFXG`.
/// The normalized form is used as the `mcp:usercode:{…}` cache key (T-203-USERCODE-NORMALIZE).
pub fn normalize_user_code(input: &str) -> String {
    input
        .to_uppercase()
        .chars()
        .filter(|c| *c != '-' && *c != ' ')
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Task 1 tests (TDD RED → GREEN) ────────────────────────────────────────

    /// DeviceGrant serializes to JSON and deserializes back to an equal record.
    #[test]
    fn device_grant_serde_roundtrip() {
        let grant = DeviceGrant {
            client_id: "client-abc".to_string(),
            status: DeviceGrantStatus::Pending,
            user_id: None,
            tenant_id: None,
            created_at: 1_718_000_000,
            last_polled_at: None,
            normalized_user_code: "WDJBMFXG".to_string(),
        };

        let json = serde_json::to_string(&grant).expect("serialize DeviceGrant");
        let back: DeviceGrant = serde_json::from_str(&json).expect("deserialize DeviceGrant");

        assert_eq!(back.client_id, grant.client_id);
        assert_eq!(back.status, grant.status);
        assert_eq!(back.user_id, grant.user_id);
        assert_eq!(back.tenant_id, grant.tenant_id);
        assert_eq!(back.created_at, grant.created_at);
        assert_eq!(back.last_polled_at, grant.last_polled_at);
        assert_eq!(back.normalized_user_code, grant.normalized_user_code);
    }

    /// DeviceGrantStatus variants serialize as snake_case per project convention.
    #[test]
    fn device_grant_status_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&DeviceGrantStatus::Pending).unwrap(),
            r#""pending""#
        );
        assert_eq!(
            serde_json::to_string(&DeviceGrantStatus::Approved).unwrap(),
            r#""approved""#
        );
        assert_eq!(
            serde_json::to_string(&DeviceGrantStatus::Denied).unwrap(),
            r#""denied""#
        );
    }

    // ── Task 2 tests (TDD RED → GREEN) ────────────────────────────────────────

    /// generate_user_code returns length 9 with a hyphen at index 4, all other
    /// chars from the RFC 8628 §6.1 charset.
    #[test]
    fn user_code_format_is_xxxx_hyphen_xxxx() {
        for _ in 0..20 {
            let code = generate_user_code();
            let len = code.len();
            assert_eq!(len, 9, "expected length 9 (XXXX-XXXX), got {len}: {code:?}");
            assert_eq!(
                code.as_bytes()[4],
                b'-',
                "expected hyphen at index 4, got {code:?}"
            );
            for (i, &b) in code.as_bytes().iter().enumerate() {
                if i == 4 {
                    continue; // skip the hyphen
                }
                let ch = b as char;
                assert!(
                    USER_CODE_CHARSET.contains(&b),
                    "char {ch:?} at index {i} is not in RFC 8628 charset: {code:?}"
                );
            }
        }
    }

    /// normalize_user_code strips hyphens and uppercases (RFC 8628 §3.3 case-insensitive).
    #[test]
    fn user_code_normalization_strips_hyphen_and_case() {
        assert_eq!(normalize_user_code("wdjb-mfxg"), "WDJBMFXG");
        assert_eq!(normalize_user_code("WDJB-MFXG"), "WDJBMFXG");
        assert_eq!(normalize_user_code("WDJBMFXG"), "WDJBMFXG");
        assert_eq!(normalize_user_code("wdjb mfxg"), "WDJBMFXG");
    }

    /// generate_device_code returns a non-empty URL-safe string (no `/`, `+`, `=`).
    #[test]
    fn device_code_is_url_safe_nonempty() {
        let code = generate_device_code();
        assert!(!code.is_empty(), "device_code must not be empty");
        assert!(!code.contains('/'), "device_code must not contain '/': {code:?}");
        assert!(!code.contains('+'), "device_code must not contain '+': {code:?}");
        assert!(!code.contains('='), "device_code must not contain '=': {code:?}");
    }
}
