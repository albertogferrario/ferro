# Phase 206 / Plan 01 — Summary

**Plan:** 206-01-PLAN.md (STORAGE_* rename + workspace 0.2.54)
**Wave:** 0 (single wave)
**Shipped:** 2026-06-12 (local commits; not pushed)
**Commits:** 5 phase commits on `ferro` master

---

## Scope shipped

Applied the Phase 204 provider-agnostic naming pattern to the six S3-style env vars across ferro-storage's S3 surface. Hoisted `env_with_fallback` from `cdn::mod` private fn to crate-level `env_helpers` so all four read sites (cdn, config, drivers/s3, facade) share the same deprecation-warning convention. Workspace bumped 0.2.53 → 0.2.54.

## Naming map (verbatim)

| New (primary) | Old (deprecated alias) | Surface |
|---|---|---|
| `STORAGE_ACCESS_KEY_ID` | `AWS_ACCESS_KEY_ID` | `S3Driver::new` |
| `STORAGE_SECRET_KEY` | `AWS_SECRET_ACCESS_KEY` | `S3Driver::new` |
| `STORAGE_REGION` | `AWS_DEFAULT_REGION` | `StorageConfig::from_env` |
| `STORAGE_BUCKET` | `AWS_BUCKET` | `StorageConfig::from_env` (registers `s3` disk when set) |
| `STORAGE_ENDPOINT` | `AWS_URL` | `StorageConfig::from_env` + `Storage::create_driver` |
| `STORAGE_PUBLIC_URL` | `AWS_PUBLIC_URL` | `StorageConfig::from_env` |

## Files touched

| File | Change |
|---|---|
| `ferro-storage/src/env_helpers.rs` | NEW — `pub(crate) fn env_with_fallback` |
| `ferro-storage/src/lib.rs` | declare `mod env_helpers;` |
| `ferro-storage/src/cdn/mod.rs` | drop local `env_with_fallback`; import from `env_helpers` |
| `ferro-storage/src/config.rs` | 4 env reads via `env_with_fallback`; doc-comment rename; `from_env_storage_primary` test |
| `ferro-storage/src/drivers/s3.rs` | 2 env reads via `env_with_fallback`; doc-comment rename |
| `ferro-storage/src/facade.rs` | 1 env read via `env_with_fallback` (S3 endpoint URL) |
| `ferro-storage/tests/s3_integration.rs` | skip-gate `STORAGE_BUCKET || AWS_BUCKET`; URL test reads STORAGE_* primary with AWS_* fallback |
| `ferro-storage/CHANGELOG.md` | `## [0.2.54]` entry with rename + deprecation table |
| `app/.env.example` | "Object Storage Settings" block with 6 STORAGE_* primary vars + AWS_* deprecation alias note |
| `Cargo.toml` | workspace `version = "0.2.54"` |
| `Cargo.lock` | regenerated for 0.2.54 |

## Decisions retained vs. modified

All CONTEXT decisions retained. No deviations.

- **Retained**: Stripe / Resend / WhatsApp Cloud / Anthropic / DigitalOcean control-plane vars keep vendor prefixes (their values are stamped with one vendor's API contract).
- **Retained**: `FILESYSTEM_DISK` stays as driver selector — no change.
- **Retained**: AWS_* fallback cushion for one release — removal slated for 0.2.55+.
- **Retained**: gestiscilo consumer phase is a separate follow-up (not part of 206).

## Verification

See `206-VERIFICATION.md`. All 10 SC PASS. Static + runtime gates both green.

## Next

Operator pushes ferro master (bundles 0.2.53 + 0.2.54 in one publish wave). Then open consumer-side phase in gestiscilo-it/app mirroring Phase 205's shape: ferro bump → `.env.example` STORAGE_* rename → `app-env/production/.env` STORAGE_* rename (preserve values verbatim) → ROADMAP Pending Operator Tasks alignment.
