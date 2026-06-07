# Phase 188: ferro-storage CDN Extension - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-08
**Phase:** 188-ferro-storage-cdn-extension
**Mode:** `--auto` (all gray areas auto-selected; recommended option chosen per area)
**Areas discussed:** cdn_url placement & config, PurgeApi trait & feature gating, DO adapter operational details, config/env vars, Bunny/CF adapter depth, module/error layout

---

## cdn_url() placement & configuration

| Option | Description | Selected |
|--------|-------------|----------|
| DiskConfig.cdn_url field + facade-level cdn_url() fallback to url() | CDN is a presentation layer over any driver; pure string, zero deps | ✓ |
| Push cdn_url into every StorageDriver impl | Invasive; CDN is orthogonal to the backend | |

**Auto-selected:** `DiskConfig.cdn_url` + `with_cdn_url()` builder; `Disk/Storage::cdn_url(path)` returns `{cdn_base}/{path}` or falls back to origin `url()`. Env `AWS_CDN_URL` for the s3/Spaces disk. **Rationale:** mirrors the existing `url`/`with_url` pattern, keeps drivers unchanged, criterion 1 is pure string composition with zero new deps → always available in default features.

---

## PurgeApi trait & feature gating

| Option | Description | Selected |
|--------|-------------|----------|
| Trait in default (no deps); DO adapter + reqwest in default graph; Bunny/CF feature-gated | Literal criteria reading (crit 2: DO default; crit 4: only Bunny/CF gated) | ✓ |
| Gate DO behind a `cdn` feature too (mirror `s3`) | Leaner default, but contradicts criterion 4's implication that DO is in default graph | |

**Auto-selected:** `PurgeApi` trait in default (async-trait only); DO Spaces adapter + `reqwest` in the default dependency graph (lean `default-features=false, features=["json","rustls-tls"]`); Bunny/Cloudflare behind `cdn-bunny`/`cdn-cloudflare`. **Rationale:** the acceptance contract (criterion 2 "default DO Spaces adapter", criterion 4 names ONLY Bunny/CF as not-in-default-graph) is the verifier's checklist; honor it literally. reqwest lean-rustls features mitigate the default-weight cost.

---

## DigitalOcean Spaces adapter — operational details

| Aspect | Decision | Selected |
|--------|----------|----------|
| Endpoint | `DELETE /v2/cdn/endpoints/{id}/cache`, body `{"files":[...]}`, Bearer auth | ✓ |
| Batching | ≤50 files/request; wildcard = 1 slot | ✓ |
| Rate limit | internal ≤5 req / rolling 10s (caller never manages) | ✓ |
| Missing CDN id | logged no-op returning Ok(()) | ✓ |

**Auto-selected:** all of the above per ROADMAP STOR-F-02. **Rationale:** the success criteria specify these exactly; the DO adapter encapsulating batching/throttle/wildcard is the phase's substance (consumers shouldn't reimplement DO's quirks). Missing-id no-op (criterion 3) keeps non-CDN consumers working.

---

## Config & env vars

| Option | Description | Selected |
|--------|-------------|----------|
| `DoSpacesCdnConfig::from_env()` reads `DO_SPACES_CDN_ID` + `DIGITALOCEAN_ACCESS_TOKEN` | Canonical DO env vars (doctl/terraform); project-agnostic | ✓ |
| Custom/ferro-specific token var | Reinvents the provider's standard env var | |

**Auto-selected:** `DO_SPACES_CDN_ID` + `DIGITALOCEAN_ACCESS_TOKEN` via `from_env()`. **Rationale:** generic provider env vars (not app identity) satisfy the project-agnostic crates rule; using DO's canonical token var means zero surprise for operators.

---

## Bunny / Cloudflare adapter depth

| Option | Description | Selected |
|--------|-------------|----------|
| Real-but-lean compiling adapters behind features | Each calls its provider's purge endpoint; DO is the polished reference | ✓ |
| Empty stubs that just compile | Satisfies "compile" literally but provides no value | |

**Auto-selected:** real, lean `PurgeApi` impls behind `cdn-bunny`/`cdn-cloudflare` (each hits its provider's purge endpoint via the shared reqwest client), not gold-plated. **Rationale:** criterion 4 requires they compile behind features without entering the default graph; substance-first says DO gets the obsessive polish (batching/throttle/wildcard) while Bunny/CF prove the trait generalizes at a "works" bar.

---

## Module layout & error handling

| Option | Description | Selected |
|--------|-------------|----------|
| `src/cdn/` module + `Error::Cdn` variant (thiserror 1.0) | Cohesive; matches crate's existing thiserror 1.0 | ✓ |

**Auto-selected:** `src/cdn/{mod,bunny,cloudflare}.rs`; extend `Error` with a `Cdn(String)` variant (thiserror 1.0, NOT 2 — crate-local consistency); no panics on network paths. **Rationale:** keeps the crate's conventions; failed purge returns `Err`, never panics the host.

---

## Claude's Discretion
- `PurgeApi::purge` exact signature; throttle primitive (token bucket vs timestamp ring); exact reqwest minor; shared HTTP-client helper factoring; whether `cdn_url()` lands on StorageDriver or facade-only (facade-only recommended); Bunny/CF exact endpoint shapes; HTTP test-double approach.

## Deferred Ideas
- Signed/temporary CDN URLs; CDN endpoint provisioning; auto-purge on delete/put; per-key purge-policy helpers; lifecycle-aware purge (B-03).
