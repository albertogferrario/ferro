//! Token exchange endpoint (Plan 04).
//!
//! Implements `POST /token`: validates auth code, verifies PKCE S256, mints
//! HS256 JWT access token bound to `(user, tenant)`.
