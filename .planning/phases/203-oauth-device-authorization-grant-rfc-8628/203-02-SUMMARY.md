---
phase: 203-oauth-device-authorization-grant-rfc-8628
plan: 02
subsystem: ferro-mcp-oauth
tags: [oauth, device-grant, rfc-8628, discovery, rfc-8414]
dependency_graph:
  requires: []
  provides: [device_authorization_endpoint in authorization_server_metadata, device-code grant type in grant_types_supported]
  affects: [ferro-mcp-oauth/src/discovery.rs]
tech_stack:
  added: []
  patterns: [json!() literal field addition, .iter().any() grant type assertion]
key_files:
  modified:
    - ferro-mcp-oauth/src/discovery.rs
decisions:
  - Field name device_authorization_endpoint is verbatim RFC 8628 §4 — not renamed
  - grant_types_supported extended to array with both values; no index-based assertions remain
metrics:
  duration: 120s
  completed: "2026-06-11"
  tasks_completed: 1
  files_modified: 1
---

# Phase 203 Plan 02: Discovery Metadata Device Grant Advertising Summary

**One-liner:** RFC 8628 §4 device fields added to `authorization_server_metadata` — `device_authorization_endpoint` URL + device-code grant URN in `grant_types_supported`, two new tests green, no index-based assertions remain.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 (RED) | Failing tests for device endpoint + grant type | b4acea4f | ferro-mcp-oauth/src/discovery.rs |
| 1 (GREEN) | Implement device_authorization_endpoint + grant URN + fix index assertion | 09eb5d1a | ferro-mcp-oauth/src/discovery.rs |

## Implementation Notes

`authorization_server_metadata` in `ferro-mcp-oauth/src/discovery.rs` was extended with two fields inside the existing `json!({...})` literal:

- `"device_authorization_endpoint": format!("{}/device_authorization", app_url)` — RFC 8628 §4 verbatim field name; URL built from the function's `app_url` argument (no hardcoded host, consistent with threat model T-203-DISCOVERY-HOST).
- `"grant_types_supported"` array extended from `["authorization_code"]` to `["authorization_code", "urn:ietf:params:oauth:grant-type:device_code"]`.

The pre-existing `authorization_server_has_all_required_fields` test had an index-based assertion `grant_types[0].as_str()...` that would have broken once a second element was added. This was replaced with `.iter().any(|v| v.as_str() == Some("authorization_code"))` per Pitfall 6 guidance in 203-PATTERNS.md.

Two new named tests were added:
- `discovery_advertises_device_authorization_endpoint`
- `discovery_advertises_device_grant_type`

## Test Results

```
running 5 tests
test discovery::tests::authorization_server_has_all_required_fields ... ok
test discovery::tests::discovery_advertises_device_authorization_endpoint ... ok
test discovery::tests::discovery_advertises_device_grant_type ... ok
test discovery::tests::discovery_urls_interpolate_app_url_no_hardcoded_host ... ok
test discovery::tests::protected_resource_has_resource_and_authorization_servers ... ok

test result: ok. 5 passed; 0 failed; 0 ignored
```

## Deviations from Plan

None — plan executed exactly as written.

## Threat Surface Scan

No new network endpoints, auth paths, or trust boundaries introduced. The `device_authorization_endpoint` value is built from `app_url` (caller passes `sanitized_app_url()`), consistent with the existing URL construction pattern and T-203-DISCOVERY-HOST mitigation. Discovery metadata is public by RFC 8414 design (T-203-DISCOVERY-DISCLOSURE accepted).

## Known Stubs

None.

## Self-Check: PASSED

- `ferro-mcp-oauth/src/discovery.rs` contains `device_authorization_endpoint`: FOUND
- `ferro-mcp-oauth/src/discovery.rs` contains `urn:ietf:params:oauth:grant-type:device_code`: FOUND
- `ferro-mcp-oauth/src/discovery.rs` contains `.iter().any(`: FOUND (no index-based grant_types assertion remains)
- Commit b4acea4f (RED tests): FOUND
- Commit 09eb5d1a (GREEN implementation): FOUND
- `cargo test -p ferro-mcp-oauth -- discovery`: 5/5 passed
