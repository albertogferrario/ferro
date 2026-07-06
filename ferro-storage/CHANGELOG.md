# Changelog

All notable changes to `ferro-storage` are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [0.2.54] - 2026-06-12

### Changed

- `StorageConfig::from_env` and `S3Driver::new` and the facade's S3 endpoint
  read now source the six S3-style env vars through `env_with_fallback` —
  `STORAGE_BUCKET` / `STORAGE_REGION` / `STORAGE_ENDPOINT` / `STORAGE_PUBLIC_URL`
  / `STORAGE_ACCESS_KEY_ID` / `STORAGE_SECRET_KEY` are read primary; the
  pre-0.2.54 `AWS_*` names continue to work for one release as deprecated
  aliases emitting a `tracing::warn!` on first use.
- `env_with_fallback` hoisted from `cdn::mod` private fn to crate-level
  `env_helpers` so the same deprecation cushion applies uniformly across CDN,
  config, drivers, and facade.

### Deprecated

The following environment variables are deprecated and will be removed in a future
release. They remain as fallbacks for one release. Migrate to the provider-agnostic
names above.

| Deprecated var | Replacement | Notes |
|---|---|---|
| `AWS_ACCESS_KEY_ID` | `STORAGE_ACCESS_KEY_ID` | S3 access key |
| `AWS_SECRET_ACCESS_KEY` | `STORAGE_SECRET_KEY` | S3 secret key |
| `AWS_DEFAULT_REGION` | `STORAGE_REGION` | S3 region (default: `us-east-1`) |
| `AWS_BUCKET` | `STORAGE_BUCKET` | bucket name; registers the `s3` disk when set |
| `AWS_URL` | `STORAGE_ENDPOINT` | S3 API endpoint (for non-AWS S3-compatible backends) |
| `AWS_PUBLIC_URL` | `STORAGE_PUBLIC_URL` | public URL base for generated file URLs |

### Notes

The rename makes the env-var surface match what ferro-storage already abstracts:
the `s3` driver targets *any* S3-compatible backend (DO Spaces, Wasabi, R2,
Backblaze B2, MinIO), so `STORAGE_*` is honest about the role while `FILESYSTEM_DISK`
selects the driver. AWS-specific naming was a historical artifact, not a contract.

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
