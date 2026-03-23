---
phase: 101-ferro-whatsapp-plugin
plan: "01"
subsystem: ferro-whatsapp
tags: [whatsapp, messaging, meta-api, oncelock, reqwest]
dependency_graph:
  requires: [ferro-events, ferro-queue]
  provides: [ferro-whatsapp crate, WhatsApp facade, WhatsAppConfig, Message, Error types]
  affects: [workspace Cargo.toml]
tech_stack:
  added: [ferro-whatsapp, reqwest 0.12, hmac 0.12, sha2 0.10, hex 0.4, dashmap 6, async-trait 0.1, chrono 0.4]
  patterns: [OnceLock facade, thiserror Error enum, serde rename_all snake_case, serde other fallback]
key_files:
  created:
    - ferro-whatsapp/Cargo.toml
    - ferro-whatsapp/README.md
    - ferro-whatsapp/src/lib.rs
    - ferro-whatsapp/src/error.rs
    - ferro-whatsapp/src/config.rs
    - ferro-whatsapp/src/message.rs
    - ferro-whatsapp/src/client.rs
    - ferro-whatsapp/src/dedup.rs
    - ferro-whatsapp/src/webhook/mod.rs
  modified:
    - Cargo.toml (added ferro-whatsapp workspace member)
decisions:
  - "META_API_VERSION const in config.rs as single source of truth for API version pinning"
  - "SenderIdentity uses externally-tagged serde (tag/content) for clean JSON representation"
  - "build_api_payload helper gated on cfg(any(test, feature = test-helpers)) for HTTP-free unit tests"
  - "dedup.rs and webhook/mod.rs created as empty stubs to allow lib.rs to compile with mod declarations"
metrics:
  duration: "~7 minutes"
  completed_date: "2026-03-23"
  tasks_completed: 2
  files_changed: 9
---

# Phase 101 Plan 01: ferro-whatsapp Crate Foundation Summary

ferro-whatsapp crate with OnceLock facade for outbound text and template messages via Meta Cloud API v23.0, typed Error enum, WhatsAppConfig with closure-based sender identity, and 23 unit tests.

## What Was Built

Created the `ferro-whatsapp` workspace crate from scratch following the ferro-stripe structural pattern. The crate provides:

- **`WhatsApp` facade** — static OnceLock pattern with `init(config)` and `async send(to, message)` methods
- **`WhatsAppConfig`** — struct with 4 env vars (`WHATSAPP_APP_SECRET`, `WHATSAPP_ACCESS_TOKEN`, `WHATSAPP_PHONE_NUMBER_ID`, `WHATSAPP_VERIFY_TOKEN`) plus `is_owner: Box<dyn Fn(&str) -> bool + Send + Sync>` closure
- **`Message` enum** — `Text { body }` and `Template { name, language, parameters }` with `to_api_payload()` producing correct Meta Cloud API JSON
- **`Error` enum** — 7 variants: Config, WebhookVerification, RateLimit, InvalidNumber, AuthError, NetworkError, ApiError
- **`SendResult`** — wraps `wamid: String` for delivery status correlation
- **`SenderIdentity`** — `Owner(String)` / `Customer(String)` enum with serde
- **`DeliveryStatus`** — Sent/Delivered/Read/Failed/Unknown with `#[serde(other)]` catch-all
- **Stub modules** — `dedup.rs` and `webhook/mod.rs` placeholder for Plan 02

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Create ferro-whatsapp crate with core types | 8bb9283 | Cargo.toml, error.rs, config.rs, message.rs, lib.rs, dedup.rs, webhook/mod.rs |
| 2 | Implement OnceLock facade and outbound message sender | 53e57e2 | client.rs |

## Test Results

23 tests across 4 modules, all passing:
- `error`: 7 tests — all 7 Error variants display correctly
- `config`: 3 tests — from_env fails when vars missing, api_url format
- `message`: 8 tests — Text/Template payload JSON structure, SenderIdentity, DeliveryStatus serde
- `client`: 5 tests — URL construction, wamid extraction, HTTP status → Error mapping

## Deviations from Plan

### Auto-fixed Issues

None — plan executed exactly as written.

### Incidental Changes

**[Chore - Formatting] Applied cargo fmt to pre-existing ferro-json-ui formatting issues**
- Found during: running `cargo fmt --all -- --check` as required before commit
- Issue: ferro-json-ui had pre-existing formatting deviations from rustfmt style
- Fix: ran `cargo fmt --all` which also reformatted ferro-json-ui source files
- Files modified: ferro-json-ui/src/layout.rs, ferro-json-ui/src/lib.rs, ferro-json-ui/src/render.rs
- Commit: 3d8416b

## Decisions Made

1. **`META_API_VERSION` const in config.rs** — single location for v23.0 version string, referenced by `api_url()` and `build_api_payload()` test helper
2. **`SenderIdentity` uses externally tagged serde** — `#[serde(tag = "type", content = "phone")]` produces `{"type": "owner", "phone": "..."}` which is clean JSON for webhook event payloads
3. **`build_api_payload` test helper** — gated on `cfg(any(test, feature = "test-helpers"))`, returns `(url, payload)` tuple for verifying HTTP payloads without live HTTP calls
4. **Empty stub modules** — `dedup.rs` and `webhook/mod.rs` declared in `lib.rs` as empty files to allow compilation; Plan 02 will fill them in

## Self-Check: PASSED

Files exist:
- ferro-whatsapp/Cargo.toml: FOUND
- ferro-whatsapp/src/lib.rs: FOUND
- ferro-whatsapp/src/error.rs: FOUND
- ferro-whatsapp/src/config.rs: FOUND
- ferro-whatsapp/src/message.rs: FOUND
- ferro-whatsapp/src/client.rs: FOUND

Commits exist:
- 8bb9283: FOUND
- 53e57e2: FOUND
