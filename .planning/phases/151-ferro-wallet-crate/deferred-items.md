# Phase 151 — Deferred Items

Items discovered during Phase 151 execution that are out of scope for the
phase and tracked for follow-up.

## Doc link warnings in ferro-wallet (discovered 2026-05-11 in 151-09)

`cargo doc --no-deps -p ferro-wallet` exits 0 but emits 6 warnings:

| # | File:line | Issue | Suggested fix |
|---|-----------|-------|---------------|
| 1 | `ferro-wallet/src/apple/mod.rs:32` | Public `new` doc links to private `sign::SigningMaterial::parse` | Drop the link or expose the helper; or rephrase doc to avoid the reference |
| 2 | `ferro-wallet/src/config.rs:12` | Unresolved intra-doc link `ferro_stripe::config::StripeConfig` | Drop the link (`ferro-wallet` doesn't depend on `ferro-stripe`) or remove the cross-crate reference |
| 3 | `ferro-wallet/src/config.rs:70` | Unresolved intra-doc link `framework::config::AppConfig::from_env` | Drop the bracketed link, leave the path as inline `code` |
| 4 | `ferro-wallet/src/google/jwt.rs:70` | Public `save_url` doc links to private `sign_save_jwt` | Same as #1 |
| 5 | `ferro-wallet/src/google/mod.rs:46` | Public `save_jwt` doc links to private `object::build_event_ticket_object` | Same as #1 |
| 6 | `ferro-wallet/src/google/mod.rs:47` | Public `save_jwt` doc links to private `jwt::sign_save_jwt` | Same as #1 |

**Root cause:** Doc comments authored in 151-03 (config), 151-05 (apple
builder), and 151-07 (google builder) referenced private items or a
sibling crate without using fully qualified paths.

**Impact:** Does NOT block the first publish (rustdoc-only warnings,
`cargo publish` does not check intra-doc-link resolution). Recommend a
small follow-up plan (e.g., 151-10-doc-link-cleanup) or bundle into the
next ferro-wallet release.
