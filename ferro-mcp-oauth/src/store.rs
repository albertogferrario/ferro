//! OAuthCode (cached) + oauth_clients model (Plan 02/04).
//!
//! `OAuthCode` is stored in `ferro::Cache` with a ~60s TTL (D-03).
//! The `oauth_clients` DB entity is used by DCR (Plan 02) and
//! `/authorize` validation (Plan 04).
