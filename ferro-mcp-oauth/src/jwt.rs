//! HS256 mint/decode (Plan 03).
//!
//! Mints and validates HS256 JWTs using `jsonwebtoken` v9. Claims include
//! `sub`, `tenant_id` (exact name — matches `JwtClaimResolver` in
//! `framework/src/tenant/resolver.rs`), `aud`, `iss`, `iat`, and `exp`.
