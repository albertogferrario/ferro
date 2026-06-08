---
phase: 188-ferro-storage-cdn-extension
audited: 2026-06-08
asvs_level: 1
block_on: high
status: SECURED
threats_total: 13
threats_closed: 13
threats_open: 0
---

# Phase 188 Security Audit

**Phase:** 188 — ferro-storage CDN Extension
**ASVS Level:** 1
**Threats Closed:** 13/13
**Threats Open:** 0/13

## Threat Verification

| Threat ID | Category | Disposition | Status | Evidence |
|-----------|----------|-------------|--------|----------|
| T-188-01 | Tampering — cdn_url double-slash | mitigate | CLOSED | `facade.rs:410-411`: `base.trim_end_matches('/')` + `path.trim_start_matches('/')` in `Disk::cdn_url()`; `cdn_url_no_double_slash` test in facade.rs confirms the invariant. |
| T-188-02 | Info Disclosure — AWS_CDN_URL in logs | accept | CLOSED | Accepted: CDN base URL is public-facing config, non-secret. No log redaction required. Disposition documented here. |
| T-188-03 | Info Disclosure — Error::Cdn message content | mitigate | CLOSED | `error.rs:74-76`: `Error::cdn()` constructor carries only caller-supplied strings. All call sites in cdn/mod.rs pass HTTP status codes or env-var names only — no token or credential appears in any `Error::cdn(...)` argument. |
| T-188-04 | Info Disclosure — DIGITALOCEAN_ACCESS_TOKEN in Debug/logs | mitigate | CLOSED | `cdn/mod.rs:67-75`: hand-written `impl std::fmt::Debug for DoSpacesCdnConfig` prints `"<redacted>"` for `api_token`. No `#[derive(Debug)]` on `DoSpacesCdnConfig` (grep of all three cdn files returns no `derive.*Debug`). The two `tracing::info!` lines (mod.rs:154, mod.rs:181) log counts and status only — no token or URL fragment. |
| T-188-05 | Tampering/Injection — consumer purge paths | mitigate | CLOSED | `cdn/mod.rs:170`: DO paths go into `serde_json::json!({ "files": chunk })` — serde escapes all values. `bunny.rs:119`: Bunny paths go through `.query(&[("url", full_url.as_str(),...)])` — reqwest URL-encodes query parameters. `cloudflare.rs:99`: CF paths go into `serde_json::json!({ "files": chunk })`. No manual string interpolation of consumer paths into request lines. |
| T-188-06 | Spoofing/SSRF — API base endpoint | mitigate | CLOSED | `cdn/mod.rs:32`: `const DO_CDN_API_BASE: &str = "https://api.digitalocean.com"` is the fixed production base. `api_base` override field is `pub(crate)` (mod.rs:64) — unreachable from untrusted input. `bunny.rs:118`: hardcoded literal `"https://api.bunny.net/purge"`. `cloudflare.rs:81`: hardcoded literal `"https://api.cloudflare.com/client/v4/zones/{}/purge_cache"` with `zone_id` from trusted env only. |
| T-188-07 | Info Disclosure transport — token over wire (DO) | mitigate | CLOSED | `Cargo.toml:22`: `reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }`. No `native-tls` in Cargo.toml. No `disable_certificate_verification` / `danger_accept_invalid_certs` call found in any cdn file. Default reqwest TLS verification applies. |
| T-188-08 | DoS accidental — purge when unconfigured | mitigate | CLOSED | `cdn/mod.rs:153-156`: `let Some(id) = &self.config.endpoint_id else { tracing::info!(...); return Ok(()); }`. Missing `DO_SPACES_CDN_ID` → logged no-op, zero HTTP requests. Asserted by `do_adapter_noop_missing_id` test with `.expect(0)`. |
| T-188-09 | DoS — exceeding 5 req/10s | mitigate | CLOSED | `cdn/mod.rs:120-144`: loop-based sliding-window throttle using `tokio::sync::Mutex<VecDeque<Instant>>`. WR-01 fix (post-review commit d6f81517) replaced the racy if/else with a loop that re-checks `times.len() < RATE_LIMIT_MAX` while holding the lock before returning. `do_adapter_throttle_serializes` test asserts elapsed >= 9s for 6 chunks. |
| T-188-10 | Info Disclosure — BUNNY_ACCESS_KEY / CF_API_TOKEN in Debug/logs | mitigate | CLOSED | `bunny.rs:25-32`: hand-written `Debug` for `BunnyCdnConfig` prints `"<redacted>"` for `access_key`. `cloudflare.rs:20-28`: hand-written `Debug` for `CloudflareCdnConfig` prints `"<redacted>"` for `api_token`. No `#[derive(Debug)]` on either config struct. No tracing calls exist in bunny.rs or cloudflare.rs. |
| T-188-11 | Tampering/Injection — paths into Bunny query / CF JSON | mitigate | CLOSED | `bunny.rs:119`: `.query(&[("url", full_url.as_str()), ("async", "false")])` — reqwest URL-encodes. `cloudflare.rs:99`: `.json(&serde_json::json!({ "files": chunk }))` — serde escapes. No manual interpolation of paths into request URL or body string. |
| T-188-12 | Spoofing/SSRF — Bunny/CF host | mitigate | CLOSED | `bunny.rs:118`: literal `"https://api.bunny.net/purge"`. `cloudflare.rs:81`: literal `"https://api.cloudflare.com/client/v4/zones/{}/purge_cache"` with `zone_id` from `CF_ZONE_ID` env (trusted operator config). Neither adapter has an `api_base` override field. No untrusted host injection path exists. |
| T-188-13 | Info Disclosure transport — secrets over wire (Bunny/CF) | mitigate | CLOSED | Same dep as T-188-07: `Cargo.toml:22` `rustls-tls`, `default-features = false`, no native-tls. `reqwest::Client::new()` used in all three adapters with default TLS verification. No cert-verification-disabling API call found in bunny.rs or cloudflare.rs. |

## Accepted Risks Log

| Threat ID | Category | Rationale |
|-----------|----------|-----------|
| T-188-02 | Info Disclosure — AWS_CDN_URL in logs | CDN base URLs are publicly served to browsers and are not secrets. No redaction is warranted. Disposition accepted in PLAN 01 threat model. |

## Unregistered Flags

None. No `## Threat Flags` section was present in any SUMMARY.md that maps to an unregistered threat.

## Notes

- The post-review fix for WR-01 (throttle race) is confirmed in code: the `loop` pattern with lock-hold before `return` is present at `cdn/mod.rs:121-144`, closing the concurrent-caller window.
- `api_base` override visibility is `pub(crate)` (mod.rs:64), ensuring no external caller can redirect the DO adapter to an arbitrary host.
- All three adapters use `reqwest::Client::new()` without any builder customization that could weaken TLS.
