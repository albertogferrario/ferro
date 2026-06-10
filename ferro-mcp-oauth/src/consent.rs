//! Consent page render + submit (Plan 04).
//!
//! Implements `POST /authorize`: validates CSRF token, issues auth code on
//! approve, redirects with error on deny.
