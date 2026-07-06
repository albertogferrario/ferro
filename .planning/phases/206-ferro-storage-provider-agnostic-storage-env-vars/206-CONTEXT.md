# Phase 206 — Context

**Gathered:** 2026-06-12
**Status:** Locked
**Source:** Operator-locked principle (2026-06-12) after Phase 204/205 shipped — ferro env vars must use provider-agnostic naming where the contract is provider-agnostic; provider-specific names are allowed where the value is stamped with a vendor's API contract.

## Phase Boundary

Apply the Phase 204 naming pattern to the S3 surface. ferro-storage's `s3` driver already targets any S3-compatible backend (DO Spaces, Wasabi, R2, Backblaze B2, MinIO) — only the env-var names were AWS-flavored. Rename the six relevant vars to `STORAGE_*`, keep `AWS_*` as deprecated aliases for one release via the hoisted `env_with_fallback` helper. Workspace 0.2.53 → 0.2.54.

## Decisions

### Naming map (verbatim)

| New (primary) | Old (deprecated alias) | Surface |
|---|---|---|
| `STORAGE_ACCESS_KEY_ID` | `AWS_ACCESS_KEY_ID` | `drivers/s3.rs::S3Driver::new` |
| `STORAGE_SECRET_KEY` | `AWS_SECRET_ACCESS_KEY` | `drivers/s3.rs::S3Driver::new` |
| `STORAGE_REGION` | `AWS_DEFAULT_REGION` | `config.rs::StorageConfig::from_env` |
| `STORAGE_BUCKET` | `AWS_BUCKET` | `config.rs::StorageConfig::from_env` (registers `s3` disk) |
| `STORAGE_ENDPOINT` | `AWS_URL` | `config.rs` + `facade.rs::create_driver` |
| `STORAGE_PUBLIC_URL` | `AWS_PUBLIC_URL` | `config.rs::StorageConfig::from_env` |

### Helper hoist

`env_with_fallback` was a private fn inside `cdn::mod`. Hoisted to `crate::env_helpers::env_with_fallback` (`pub(crate)`) so all four read sites (cdn, config, drivers/s3, facade) use the same deprecation-warning convention.

### What stays

- Stripe / Resend / WhatsApp Cloud / Anthropic / DigitalOcean control-plane vars keep vendor prefixes — their values are stamped with one vendor's API contract.
- `FILESYSTEM_DISK` is the driver selector — no change.
- The existing CDN quartet from Phase 204 is unchanged.

### No `[patch.crates-io]` artifacts

This is producer-side ferro work; no patch override to revert.

## Canonical references

- Phase 204 in this repo for the helper pattern (`ferro-storage/src/cdn/mod.rs::env_with_fallback`).
- gestiscilo Phase 205 (consumer-side) for the rename-then-bump consumer pattern.
- Operator-locked principle: `gestiscilo memory feedback_ferro_provider_agnostic_env_vars.md`.

## Specifics

1. **Hoist:** new `ferro-storage/src/env_helpers.rs` with `pub(crate) fn env_with_fallback`; `cdn::mod` imports + drops local copy.
2. **Rename:** 4 source files (`config.rs`, `drivers/s3.rs`, `facade.rs`, `tests/s3_integration.rs`) plus `ferro/app/.env.example` get the new names with legacy fallback.
3. **Test additions:** keep `from_env_cdn_url` + `cdn_url_parity_aws_fallback` as legacy-parity coverage; add `from_env_storage_primary` for the new path.
4. **CHANGELOG + version:** `## [0.2.54]` documents the rename + deprecation table; workspace `version = "0.2.54"`.

## Deferred

- Removing the legacy `AWS_*` fallback (one release cushion — slated for the release after 0.2.54).
- gestiscilo consumer phase (mirrors Phase 205 shape — separate consumer ticket).
- Provider-agnostic rename of any other ferro env vars (queue/cache/projection/etc.) — case-by-case as roles surface.
