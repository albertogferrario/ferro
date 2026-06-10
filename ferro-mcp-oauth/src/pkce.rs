//! PKCE S256 verification (Plan 03).
//!
//! Implements RFC 7636: `code_challenge = BASE64URL(SHA256(code_verifier))`.
//! Uses `sha2` for hashing and `subtle::ConstantTimeEq` to prevent timing
//! oracles on the challenge comparison.
