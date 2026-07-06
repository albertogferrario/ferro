---
phase: 204-ferro-storage-provider-agnostic-cdn-configuration
reviewed: 2026-06-11T17:37:14Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - ferro-storage/src/cdn/mod.rs
  - ferro-storage/src/cdn/bunny.rs
  - ferro-storage/src/cdn/cloudflare.rs
  - ferro-storage/src/config.rs
  - ferro-storage/src/error.rs
  - ferro-storage/src/lib.rs
findings:
  critical: 0
  warning: 3
  info: 2
  total: 5
status: issues_found
---

# Phase 204: Code Review Report

**Reviewed:** 2026-06-11T17:37:14Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Phase 204 replaces per-provider CDN env-var clusters with a unified quartet
(`CDN_URL` / `CDN_PROVIDER` / `CDN_PURGE_TOKEN` / `CDN_PURGE_ZONE`) plus
deprecated-with-warn fallbacks for one release. Secret safety is solid: all
sensitive fields are `<redacted>` in `Debug`, and no log line prints a token
value. Feature-gate coverage is correct: a feature-off provider returns
`Err(CdnFeatureRequired)` — it cannot silently no-op.

Three logic-level issues were found, two of which can cause wrong-provider
credentials to be used in a legacy mixed deployment:

1. **Token fallback order is provider-agnostic, so the wrong provider's token
   can win** when multiple legacy clusters are partially populated.
2. **Invalid `CDN_PROVIDER` values silently disable purging** instead of
   returning an error to the caller.
3. **`env_with_fallback` tuple format is misleading** — the second element of
   each alias pair is always identical to the first, making the `label`
   parameter redundant and the design intent unclear.

---

## Warnings

### WR-01: `CDN_PURGE_TOKEN` fallback order can pick the wrong provider's token

**File:** `ferro-storage/src/cdn/mod.rs:274-281`

**Issue:** `purge_token` is resolved with a single provider-agnostic fallback
chain that always tries `DIGITALOCEAN_ACCESS_TOKEN` → `CF_API_TOKEN` →
`BUNNY_ACCESS_KEY`, in that fixed order. Provider inference (lines 291-302)
runs separately and only reads presence indicators (`DO_SPACES_CDN_ID`,
`CF_ZONE_ID`, `BUNNY_CDN_URL`). If a deployment has both
`DO_SPACES_CDN_ID` unset and `DIGITALOCEAN_ACCESS_TOKEN` set (e.g., a machine
previously running a DO worker that kept DO creds in the environment), and then
adds Cloudflare variables, the inferred provider becomes `CdnProvider::Cloudflare`
(from `CF_ZONE_ID`) but `purge_token` resolves to the DO access token. The
Cloudflare adapter then authenticates with a DO credential and receives 401s.

The same cross-contamination applies in the reverse direction: inference picks
DO (from `DO_SPACES_CDN_ID`) while `DIGITALOCEAN_ACCESS_TOKEN` is absent but
`CF_API_TOKEN` is present — purge calls fail with an empty-token error that
mentions `DIGITALOCEAN_ACCESS_TOKEN`, confusing the operator.

**Fix:** Resolve `purge_token` (and `purge_zone`) after the provider is known,
using only the aliases that belong to that provider:

```rust
// After provider is determined:
let (purge_token, purge_zone) = match &provider {
    CdnProvider::DigitalOcean => (
        env_with_fallback("CDN_PURGE_TOKEN", &[("DIGITALOCEAN_ACCESS_TOKEN", "DIGITALOCEAN_ACCESS_TOKEN")]),
        env_with_fallback("CDN_PURGE_ZONE",  &[("DO_SPACES_CDN_ID", "DO_SPACES_CDN_ID")]),
    ),
    CdnProvider::Cloudflare => (
        env_with_fallback("CDN_PURGE_TOKEN", &[("CF_API_TOKEN", "CF_API_TOKEN")]),
        env_with_fallback("CDN_PURGE_ZONE",  &[("CF_ZONE_ID", "CF_ZONE_ID")]),
    ),
    CdnProvider::Bunny => (
        env_with_fallback("CDN_PURGE_TOKEN", &[("BUNNY_ACCESS_KEY", "BUNNY_ACCESS_KEY")]),
        None, // Bunny uses cdn_base_url, not a zone id
    ),
    CdnProvider::None => (None, None),
};
```

If provider-agnostic resolution is intentional (e.g., to allow a single
`CDN_PURGE_TOKEN` to override all), document that and add a test that asserts
the expected precedence when multiple provider tokens are set simultaneously.

---

### WR-02: Invalid `CDN_PROVIDER` value silently disables purging instead of failing boot

**File:** `ferro-storage/src/cdn/mod.rs:284-290`

**Issue:** When `CDN_PROVIDER` is set to an unrecognized string (typo, wrong
case if someone bypasses `from_str_ci`, staging/prod config drift), the code
emits `tracing::error!` and then defaults to `CdnProvider::None`. If tracing
is not wired or the operator does not read logs at boot, CDN purging is silently
disabled — cache invalidation stops working without any runtime failure.

```rust
Err(e) => {
    tracing::error!("{e}; defaulting CDN purge to no-op");
    CdnProvider::None   // ← silently continues with no purging
}
```

The `CDN_PROVIDER` key is set intentionally. A typo or misconfiguration that
yields `CdnProvider::None` is almost certainly a bug, not a desired fallback.

**Fix:** Return the error to the caller. `Config::from_env()` currently returns
`Self` — changing to `Result<Self, Error>` lets callers fail the boot sequence.
Alternatively, store the error and propagate it from `build_purge_api()`:

```rust
// Option A: propagate from build_purge_api (smaller blast radius)
pub struct Config {
    pub url: Option<String>,
    pub provider: Result<CdnProvider, Error>,  // store the parse error
    pub purge_token: Option<String>,
    pub purge_zone: Option<String>,
}

// build_purge_api then:
pub fn build_purge_api(&self) -> Result<Option<Box<dyn PurgeApi>>, Error> {
    let provider = self.provider.as_ref().map_err(|e| Error::cdn(e.to_string()))?;
    match provider { ... }
}
```

```rust
// Option B: make from_env() infallible but panic on invalid CDN_PROVIDER
// (acceptable at boot; unrecoverable misconfiguration)
Err(e) => panic!("CDN_PROVIDER: {e}"),
```

Either approach makes misconfiguration loud and immediately visible.

---

### WR-03: Bunny inference trigger (`BUNNY_CDN_URL`) diverges from Bunny's zone/token aliases

**File:** `ferro-storage/src/cdn/mod.rs:297-299`

**Issue:** Bunny provider inference is triggered by `BUNNY_CDN_URL` being set
(line 297). However, the `CDN_PURGE_ZONE` fallback list (lines 267-273) has no
Bunny entry — `BUNNY_CDN_URL` maps only into the `CDN_URL` url fallback, not
`CDN_PURGE_ZONE`. The Bunny adapter reads `cdn_base_url` from `Config::url`
(line 332 in `build_purge_api`), so functionally this works. But the inference
signal (`BUNNY_CDN_URL`) is a URL variable, while every other provider's
inference signal is a zone-id-class variable (`DO_SPACES_CDN_ID`,
`CF_ZONE_ID`). This creates an asymmetry: a deployment that sets
`BUNNY_ACCESS_KEY` and `BUNNY_CDN_URL` will infer Bunny correctly, but a
deployment that sets only `BUNNY_ACCESS_KEY` (and not `BUNNY_CDN_URL`) will
infer `CdnProvider::None` and silently skip Bunny purging even though
credentials are present.

**Fix:** Either add `BUNNY_STORAGE_ZONE` (or `BUNNY_ZONE_NAME`) as the
canonical Bunny zone-id alias and use it as the inference trigger, or add a
separate inference branch for `BUNNY_ACCESS_KEY`. The current signal works for
any deployment that sets the URL, but misses key-only deployments and breaks
the conceptual symmetry with DO and CF:

```rust
// Before BUNNY_CDN_URL branch, check a zone/key signal:
} else if std::env::var("BUNNY_ACCESS_KEY").is_ok() {
    tracing::warn!("CDN_PROVIDER unset; inferred bunny from BUNNY_ACCESS_KEY. Set CDN_PROVIDER=bunny to silence.");
    CdnProvider::Bunny
}
```

---

## Info

### IN-01: `env_with_fallback` label tuple element is always identical to alias name

**File:** `ferro-storage/src/cdn/mod.rs:215-226` (call sites: lines 261-281)

**Issue:** The `aliases` parameter is `&[(&str, &str)]` where `(alias, label)`.
At every call site, label equals alias verbatim:

```rust
("AWS_CDN_URL", "AWS_CDN_URL"),
("CF_CDN_URL", "CF_CDN_URL"),
// …
```

The second element's only visible use is in the deprecation message
(`"{} is deprecated; use {} instead", label, primary`). Since `label == alias`
in all current usages, the parameter adds no information. The design suggests
label was meant to allow a human-friendly name, but no such name is ever
supplied.

**Fix:** Simplify the signature to `aliases: &[&str]` and derive the label from
the alias directly:

```rust
fn env_with_fallback(primary: &str, aliases: &[&str]) -> Option<String> {
    if let Ok(val) = std::env::var(primary) {
        return Some(val);
    }
    for alias in aliases {
        if let Ok(val) = std::env::var(alias) {
            tracing::warn!("{alias} is deprecated; use {primary} instead");
            return Some(val);
        }
    }
    None
}
```

This removes dead indirection and makes the call sites easier to scan.

---

### IN-02: `StorageConfig::from_env()` doc comment still references `AWS_CDN_URL` as the CDN variable

**File:** `ferro-storage/src/config.rs:55`

**Issue:** The doc comment still lists `AWS_CDN_URL` as the env var for the CDN
URL (`/// - \`AWS_CDN_URL\`: CDN base URL fronting the Spaces bucket…`). With
Phase 204 the canonical name is `CDN_URL`; `AWS_CDN_URL` is a deprecated alias.
The comment does not mention `CDN_URL` at all, so an operator reading the docs
will not know about the new primary variable.

**Fix:**

```rust
/// - `CDN_URL`: CDN base URL fronting the bucket (used by `cdn_url()`).
///   Legacy alias `AWS_CDN_URL` is still accepted with a deprecation warning.
```

---

_Reviewed: 2026-06-11T17:37:14Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
