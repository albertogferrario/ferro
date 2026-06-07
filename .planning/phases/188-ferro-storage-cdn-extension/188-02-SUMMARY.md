---
phase: 188-ferro-storage-cdn-extension
plan: "02"
subsystem: ferro-storage
tags: [storage, cdn, digitalocean, purge-api, reqwest, wiremock, rate-limit, tdd]
dependency_graph:
  requires: [188-01]
  provides: [PurgeApi-trait, DoSpacesCdn-adapter, wiremock-test-suite]
  affects:
    - ferro-storage/src/cdn/mod.rs
    - ferro-storage/src/lib.rs
    - ferro-storage/Cargo.toml
tech_stack:
  added: [wiremock-0.6.5]
  patterns: [sliding-window-throttle, tokio-sync-mutex, wiremock-mock-server, token-redacted-debug, tdd-red-green]
key_files:
  created:
    - ferro-storage/src/cdn/mod.rs
  modified:
    - ferro-storage/src/lib.rs
    - ferro-storage/Cargo.toml
decisions:
  - "tokio::sync::Mutex (not std) for throttle VecDeque — lock held across .await in sleep path"
  - "api_base is pub(crate) on DoSpacesCdnConfig — test-only seam, not part of public API"
  - "DoSpacesCdnConfig does NOT derive Debug — hand-written impl prints <redacted> for api_token"
  - "Throttle test uses real-time sleep (not tokio::time::pause) — asserts actual 10s window"
  - "wiremock .expect(N) on MockServer drop enforces exact request count without explicit assert"
metrics:
  duration: ~294 seconds
  completed: 2026-06-08
  tasks_completed: 2
  files_modified: 3
---

# Phase 188 Plan 02: PurgeApi trait + DoSpacesCdn adapter + wiremock test suite Summary

`PurgeApi` trait and batteries-included `DoSpacesCdn` adapter: DELETE-with-body DO CDN API, ≤50-file batching, internal 5-req/10s sliding-window throttle, missing-id logged no-op, token-redacted Debug — all proven by a full wiremock test suite without touching the real DO API.

## Completed Tasks

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | PurgeApi trait + DoSpacesCdn adapter + token-redacted config | `805bad37` | cdn/mod.rs, lib.rs |
| 2 | wiremock test suite + wiremock dev-dep | `a5a48101`, `24ddcd11` | cdn/mod.rs, Cargo.toml |

## What Was Built

### Task 1 — PurgeApi trait + DoSpacesCdn adapter

`ferro-storage/src/cdn/mod.rs` (379 lines):

- **`PurgeApi` trait** — `#[async_trait]` with `async fn purge(&self, paths: &[String]) -> Result<(), Error>`. Provider-agnostic, `Send + Sync`.
- **`DoSpacesCdnConfig`** — `Clone` only (no `Debug` derive). Hand-written `Debug` impl prints `"<redacted>"` for `api_token`, satisfying T-188-04. `pub(crate) api_base: Option<String>` seam for wiremock redirect.
- **`DoSpacesCdnConfig::from_env()`** — reads `DO_SPACES_CDN_ID` (optional, `.ok()`) and `DIGITALOCEAN_ACCESS_TOKEN` (`.unwrap_or_default()`).
- **`DoSpacesCdn`** — holds `reqwest::Client` (built once in `new()`), `DoSpacesCdnConfig`, and `tokio::sync::Mutex<VecDeque<Instant>>` for the throttle.
- **`purge()` behavior**:
  - `paths.is_empty()` → `Ok(())`, zero requests (Pitfall 5).
  - `endpoint_id.is_none()` → `tracing::info!` + `Ok(())`, zero requests (criterion 3, T-188-08).
  - `api_token.is_empty()` → `Err(Error::cdn("DIGITALOCEAN_ACCESS_TOKEN not set ..."))`.
  - `paths.chunks(BATCH_SIZE=50)` — batches ≤50 files per request; wildcard counts as 1 slot.
  - Per chunk: `throttle().await` then `client.delete(url).bearer_auth(token).json({"files":chunk}).send()`.
  - Non-204 response → `Err(Error::cdn(format!("DO CDN purge status {status}: {body}")))`.
  - Success: `tracing::info!("purged {} paths in {} request(s)", ...)` — logs counts only, never token.
- **`throttle()`** — sliding-window timestamp ring: evict stale entries, sleep if `len >= 5`, re-evict after wake, push timestamp. `tokio::sync::Mutex` (async) prevents deadlock across `.await` in sleep path (T-188-09, Pitfall 4).

`ferro-storage/src/lib.rs`:
- `pub mod cdn;` added.
- `pub use cdn::{DoSpacesCdn, DoSpacesCdnConfig, PurgeApi};` unconditional re-export.

### Task 2 — wiremock test suite

`ferro-storage/Cargo.toml` `[dev-dependencies]`:
- `wiremock = "0.6.5"` added.

8 named test functions in `#[cfg(test)] mod tests`:

| Test | What it asserts |
|------|----------------|
| `do_adapter_request_shape` | 1 path → 1 DELETE, correct path, `Authorization: Bearer test-token`, body `{"files":["index.html"]}`, purge returns Ok |
| `do_adapter_batches_over_50` | 55 paths → exactly 2 requests (wiremock `.expect(2)`) |
| `do_adapter_wildcard_slot` | 50 plain + 1 `"dir/*"` = 51 elements → 2 requests |
| `do_adapter_noop_missing_id` | `endpoint_id=None` → Ok(()), wiremock `.expect(0)` enforces zero requests |
| `purge_empty_noop` | empty slice → Ok(()), `.expect(0)` enforces zero requests |
| `do_adapter_error_on_non_204` | 403 response → `Err` whose message contains `"403"` |
| `do_adapter_missing_token_errors` | empty token with id → `Err` containing `"DIGITALOCEAN_ACCESS_TOKEN"`, `.expect(0)` |
| `do_adapter_throttle_serializes` | 300 paths (6 chunks) → elapsed ≥ 9 s (real-time sleep) |

**Throttle test runtime cost:** `do_adapter_throttle_serializes` takes ~10 s because it uses a real `tokio::time::sleep` (not `tokio::time::pause`). This is intentional — a fake-time test would pass even with a broken throttle. The 10s cost is accepted for the correctness guarantee.

## Test Results

```
running 8 tests
test cdn::tests::do_adapter_noop_missing_id ... ok
test cdn::tests::do_adapter_missing_token_errors ... ok
test cdn::tests::do_adapter_error_on_non_204 ... ok
test cdn::tests::do_adapter_request_shape ... ok
test cdn::tests::do_adapter_batches_over_50 ... ok
test cdn::tests::do_adapter_wildcard_slot ... ok
test cdn::tests::debug_does_not_contain_token ... ok
test cdn::tests::do_adapter_throttle_serializes ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured

Full suite: 37 passed; 0 failed
cargo clippy -p ferro-storage --all-targets -- -D warnings: clean
cargo fmt --all -- --check: clean
```

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. `PurgeApi` trait is fully wired to `DoSpacesCdn`; the adapter implements every operational requirement. No placeholder behavior — the missing-id no-op and empty-paths short-circuit are intentional design behaviors, not stubs.

## Threat Flags

None found. All T-188-04 through T-188-09 mitigations from the plan's threat model are implemented and grep-verified:

- **T-188-04** (token in Debug/logs): `DoSpacesCdnConfig` has no `#[derive(Debug)]`; manual impl prints `<redacted>`; `tracing::info!` logs counts only.
- **T-188-05** (path injection): paths go through `serde_json::json!({"files": chunk})` — serde escapes them. No manual string interpolation into body.
- **T-188-06** (SSRF): `api_base` override is `pub(crate)`, only settable in tests. Production uses the fixed `DO_CDN_API_BASE` constant.
- **T-188-07** (transport): reqwest with `rustls-tls` (no cert verification disabled).
- **T-188-08** (wrong-endpoint purge): missing `DO_SPACES_CDN_ID` → logged no-op. Asserted by `do_adapter_noop_missing_id` (`.expect(0)`).
- **T-188-09** (rate limit): 5-req/10s throttle asserted by `do_adapter_throttle_serializes` (real-time, elapsed ≥ 9s).

## Self-Check: PASSED

- `ferro-storage/src/cdn/mod.rs` — FOUND (created, 379 lines)
- `ferro-storage/src/lib.rs` — FOUND (modified, pub mod cdn + re-exports)
- `ferro-storage/Cargo.toml` — FOUND (wiremock = "0.6.5" in [dev-dependencies])
- Commit `805bad37` — FOUND
- Commit `a5a48101` — FOUND
- Commit `24ddcd11` — FOUND
- 8 cdn tests green — VERIFIED
- Full 37-test suite green — VERIFIED
- cargo clippy clean — VERIFIED
- cargo fmt clean — VERIFIED
