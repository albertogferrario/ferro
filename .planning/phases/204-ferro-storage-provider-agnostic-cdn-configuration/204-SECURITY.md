---
phase: 204-ferro-storage-provider-agnostic-cdn-configuration
audited: 2026-06-11
asvs_level: 1
threats_total: 5
threats_closed: 5
threats_open: 0
status: SECURED
---

# Phase 204 Security Audit

**ASVS Level:** 1
**Threats Closed:** 5 / 5
**Open:** 0

## Threat Verification

| Threat ID | Category | Disposition | Status | Evidence |
|-----------|----------|-------------|--------|----------|
| T-204-TOKEN-REDACT | Information Disclosure | mitigate | CLOSED | `cdn/mod.rs:248-258` — hand-written `Debug` for `Config` prints `&"<redacted>"` for `purge_token`. Test `cdn_config_debug_redacts_token` at `cdn/mod.rs:537-552` asserts `secret-xyz` absent and `<redacted>` present. |
| T-204-DEPRECATION-LEAK | Information Disclosure | mitigate | CLOSED | `cdn/mod.rs:215-226` — `env_with_fallback` warn format is `"{alias} is deprecated; use {primary} instead"`. No `{val}` or token value appears in any warn/error format string on the token alias path. |
| T-204-MISCONFIG | Tampering / Availability | mitigate | CLOSED | `error.rs:47-52` — `CdnInvalidProvider` and `CdnFeatureRequired` variants defined. `cdn/mod.rs:337-339` — `build_purge_api` checks `provider_error` first and returns `Err(Error::cdn_invalid_provider(bad))`. `cdn/mod.rs:363-366,379-382` — feature-off Bunny/Cloudflare arms return `Err(Error::cdn_feature_required(...))`. Test `cdn_invalid_provider_from_env_errors` at `cdn/mod.rs:567-583` verifies the env path errors via `build_purge_api`. Neither error message echoes a secret. `config.rs:123-126` — URL read is not gated on provider (orthogonality preserved, SC-3). |
| T-204-SILENT-NOOP | Tampering | mitigate | CLOSED | `cdn/mod.rs:342-344` — `CdnProvider::None` arm logs `tracing::info!("CDN_PROVIDER=none — purge is a no-op")` before returning `Ok(None)`. Not a silent swallow. |
| T-204-PURGE-PARITY | Tampering | mitigate | CLOSED | `config.rs:229-267` — `purge_parity_legacy_do` test asserts `DELETE /v2/cdn/endpoints/legacy-id/cache` with `Authorization: Bearer legacy-token` (wiremock `expect(1)`). Post WR-01: `cdn/mod.rs:302-318` — token/zone resolved after provider, scoped to the resolved provider only; cross-provider credential contamination is structurally eliminated. |

## Accepted Risk Log

| Threat ID | Context | Rationale |
|-----------|---------|-----------|
| T-204-TOKEN-REDACT (parity test fixtures) | `config.rs:239,245` | Test uses synthetic strings `"legacy-id"` / `"legacy-token"`. No real credential in source. Accept confirmed. |
| T-204-DEPRECATION-LEAK (CHANGELOG) | `ferro-storage/CHANGELOG.md:33-42` | Deprecation table names env var names only (e.g. `DIGITALOCEAN_ACCESS_TOKEN`, `BUNNY_ACCESS_KEY`); no values present. Accept confirmed. |

## Unregistered Flags

None. All SUMMARY.md Threat Flags map to registered threats T-204-TOKEN-REDACT through T-204-PURGE-PARITY. WR-01 and WR-02 from 204-REVIEW.md were review findings, not new attack surface; both are now mitigated (commit bf0d6671) and their mitigations are captured under T-204-PURGE-PARITY and T-204-MISCONFIG respectively.
