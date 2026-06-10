//! Authorization endpoint + login redirect (Plan 04).
//!
//! Implements `GET /authorize`: checks session auth, redirects to login if
//! unauthenticated, validates client_id + redirect_uri, then renders consent.
