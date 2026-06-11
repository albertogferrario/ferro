# Changelog

All notable changes to `ferro-storage` are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [0.2.53] - 2026-06-11

### Added

- `cdn::Config` — provider-agnostic CDN configuration struct with fields `url`,
  `provider` (`CdnProvider`), `purge_token` (redacted in `Debug`), `purge_zone`.
- `CdnProvider` enum — `None | DigitalOcean | Bunny | Cloudflare`, parsed
  case-insensitively from `CDN_PROVIDER`. Invalid value → boot error listing
  valid values.
- `Config::from_env()` — reads the unified quartet (`CDN_URL`, `CDN_PROVIDER`,
  `CDN_PURGE_TOKEN`, `CDN_PURGE_ZONE`) as primary, with per-var legacy fallbacks
  emitting a `tracing::warn!` deprecation notice on first use.
- `Config::build_purge_api()` — constructs the active `Box<dyn PurgeApi>` adapter
  from the unified config; returns a clear boot error if the selected provider's
  cargo feature (`cdn-bunny` / `cdn-cloudflare`) is not compiled in.

### Changed

- `StorageConfig::from_env` now sources the S3 disk CDN URL from `cdn::Config`
  (reads `CDN_URL` primary, `AWS_CDN_URL` as deprecated fallback) instead of
  reading `AWS_CDN_URL` directly.

### Deprecated

The following environment variables are deprecated and will be removed in a future
release. They remain as fallbacks for one release. Migrate to the quartet above.

| Deprecated var | Replacement | Notes |
|---|---|---|
| `AWS_CDN_URL` | `CDN_URL` | display URL fallback |
| `BUNNY_CDN_URL` | `CDN_URL` + `CDN_PROVIDER=bunny` | display URL fallback |
| `CF_CDN_URL` | `CDN_URL` + `CDN_PROVIDER=cloudflare` | display URL fallback |
| `DO_SPACES_CDN_ID` | `CDN_PURGE_ZONE` | zone fallback; also infers provider |
| `CF_ZONE_ID` | `CDN_PURGE_ZONE` | zone fallback; also infers provider |
| `DIGITALOCEAN_ACCESS_TOKEN` | `CDN_PURGE_TOKEN` | token fallback |
| `CF_API_TOKEN` | `CDN_PURGE_TOKEN` | token fallback |
| `BUNNY_ACCESS_KEY` | `CDN_PURGE_TOKEN` | token fallback |

### Notes

- Additive: no existing `PurgeApi` adapter signatures changed.
- SC-3 parity: `Disk::cdn_url()` returns byte-identical URLs for `AWS_CDN_URL`-only
  deployments (the legacy fallback path is exercised; only the deprecation warn is new).
- SC-4 parity: legacy DO vars (`DO_SPACES_CDN_ID`, `DIGITALOCEAN_ACCESS_TOKEN`)
  continue to authenticate against the DO Spaces CDN API unchanged.
