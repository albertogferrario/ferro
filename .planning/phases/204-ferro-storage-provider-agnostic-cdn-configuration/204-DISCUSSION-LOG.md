# Phase 204: ferro-storage provider-agnostic CDN configuration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-11
**Phase:** 204-ferro-storage-provider-agnostic-cdn-configuration
**Mode:** `--auto` (all gray areas auto-selected; recommended option chosen per decision)
**Areas discussed:** Unified cdn::Config shape, Fallback-chain mechanics, CDN_PROVIDER resolution/validation, Feature-gating interaction, Parity preservation, CHANGELOG/version/.env.example

---

## Unified `cdn::Config` shape (D-01)

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated `ferro_storage::cdn::Config` struct + `build_purge_api()` | SC-1 name; owns fallback+validation+redaction; feeds existing per-provider adapters | ✓ |
| Quartet fields on `StorageConfig` | Mixes disk config with CDN provider selection | |
| Free function returning a tuple | Loses redaction/Debug discipline + validation seam | |

**User's choice:** Dedicated `cdn::Config` (auto — recommended)

---

## Fallback-chain mechanics + deprecation warn (D-02)

| Option | Description | Selected |
|--------|-------------|----------|
| `env_with_fallback` helper, one-shot `tracing::warn!` per legacy var | Operators learn to migrate; clean quartet config is silent | ✓ |
| Silent fallback | Deprecation window pointless; operators never migrate | |
| Hard error on legacy vars | Breaks SC-3/SC-4 parity for AWS_CDN_URL-only deployments | |

**User's choice:** Warned fallback helper (auto — recommended)
**Notes:** `CDN_URL`←AWS_CDN_URL/CF_CDN_URL/BUNNY_CDN_URL; `CDN_PURGE_ZONE`←DO_SPACES_CDN_ID/CF_ZONE_ID; `CDN_PURGE_TOKEN`←DIGITALOCEAN_ACCESS_TOKEN/CF_API_TOKEN/BUNNY_ACCESS_KEY. Token value never printed.

---

## CDN_PROVIDER resolution + validation (D-03)

| Option | Description | Selected |
|--------|-------------|----------|
| Parse explicit; infer from legacy cluster on unset; invalid→boot error; none→no-op | Backward-compat for legacy deployments; SC-5 invalid-error | ✓ |
| Require CDN_PROVIDER explicitly | Breaks existing legacy-var deployments that never set it | |
| Default to digitalocean on unset | Wrong for non-DO / no-purge deployments | |

**User's choice:** Parse-or-infer with boot-error on invalid (auto — recommended)
**Notes:** CDN_URL (display) orthogonal to CDN_PROVIDER (purge) — preserves SC-3 parity.

---

## Feature-gating interaction (D-04)

| Option | Description | Selected |
|--------|-------------|----------|
| Selected provider whose feature is off → clear boot error | Fails loud; preserves Phase 188 zero-default-graph-impact | ✓ |
| Silent fallback to no-op purge | Prod thinks purge works when it doesn't (dangerous) | |
| Make all providers always-compiled | Contradicts cargo-tree zero-impact proof | |

**User's choice:** Boot error naming the feature flag (auto — recommended)

---

## Parity preservation — Disk::cdn_url() (D-05)

| Option | Description | Selected |
|--------|-------------|----------|
| Wire CDN_URL into the exact existing AWS_CDN_URL point (config.rs:119) | Byte-identical cdn_url() for unchanged callers (SC-3) | ✓ |
| Re-derive display URL in new code | Risks output drift; SC-3 demands parity | |

**User's choice:** Reuse existing wiring point (auto — recommended)

---

## CHANGELOG, version bump, .env.example (D-06)

| Option | Description | Selected |
|--------|-------------|----------|
| Minor bump + CHANGELOG entry + .env.example quartet migration | Additive + deprecation, no breaking removal; matches cadence | ✓ |
| Major bump | No breaking removal this phase — premature | |
| Skip .env.example | Leaves the consumer-facing example on deprecated names | |

**User's choice:** Minor bump + docs (auto — recommended)
**Notes:** ferro-storage is an existing published crate — CI publish-update handles the bump (no new-crate bootstrap). Supersedes interim env grouping commit 77d360cf.

---

## Claude's Discretion
- Module placement of `env_with_fallback`; `CdnProvider` parse via serde vs FromStr.
- Whether per-provider `*CdnConfig::from_env()` are kept or made pub(crate) constructors.
- Exact warn wording + precise minor version.
- warn vs info level for provider inference / clean none.

## Deferred Ideas
- Removing legacy env vars (future phase, post consumer migration).
- Endpoint-level purge rate-limiting beyond Phase 188 throttles.
- New CDN providers; multi-CDN / per-disk provider.
