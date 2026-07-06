---
phase: 203-oauth-device-authorization-grant-rfc-8628
reviewed: 2026-06-11T13:34:59Z
depth: standard
files_reviewed: 5
files_reviewed_list:
  - ferro-mcp-oauth/src/device.rs
  - ferro-mcp-oauth/src/token.rs
  - ferro-mcp-oauth/src/discovery.rs
  - ferro-mcp-oauth/src/lib.rs
  - app/src/routes.rs
findings:
  critical: 2
  warning: 3
  info: 1
  total: 6
status: issues_found
---

# Phase 203: Code Review Report

**Reviewed:** 2026-06-11T13:34:59Z
**Depth:** standard
**Files Reviewed:** 5
**Status:** issues_found

## Summary

This phase implements RFC 8628 Device Authorization Grant across `ferro-mcp-oauth`. The overall structure is sound: the two-key cache layout, state machine transitions, single-use enforcement via `Cache::forget`, and the one-token-issuer invariant are all correctly implemented. CSRF validation uses constant-time comparison, HTML escaping is applied on all user-visible strings, and tenant/user binding is captured server-side only.

Two critical issues were found:

1. **`user_code` is generated with `rand::thread_rng()`, which is a CSPRNG in rand 0.8 but the call pattern (`gen_range` against a `usize` modulus) introduces a subtle modular bias.** The charset has 20 characters; a `u*` uniformly-sampled index mod 20 is biased unless the sample space is an exact multiple of 20. With `gen_range(0..20)` this is exactly fine — but the code passes `0..USER_CODE_CHARSET.len()` where the `len()` is a `usize` whose upper bound is 20. In rand 0.8, `gen_range` on a `usize` range delegates to `UniformInt<usize>` which uses rejection sampling, so there is **no bias** — but this is fragile: the code appears to be doing a `% 20` when read casually, and a future refactor that switches to `gen::<u8>() % 20` (e.g., for no-std) would silently introduce bias. This is a marginal critical only because user codes are short-lived (600s TTL) and the attack surface requires an active brute-force against the verification endpoint rather than offline enumeration.

2. **The device-code grant arm in `token_exchange_device_code` does not validate `client_id` from the request against `grant.client_id` before minting the token.** Any client that knows a valid `device_code` (e.g., because it was leaked through the verification URI or via a log) can mint a token bound to a different user and tenant by presenting the stolen code with its own `client_id`. The authorization-code arm (lines 125-127) correctly enforces `record.client_id != form.client_id`; the device arm has no equivalent check.

Three warnings cover: the open-redirect surface in the device GET handler's return-URL construction; the cache write-before-return ordering in the slow_down path; and the missing CSRF token in the code-entry form.

---

## Critical Issues

### CR-01: Device-code arm skips `client_id` binding validation before token issuance

**File:** `ferro-mcp-oauth/src/token.rs:245-269`
**Issue:** `token_exchange_device_code` reads `grant.client_id` but never compares it against `form.client_id` before minting the JWT. Any caller that presents a valid `device_code` (belonging to a grant initiated by client A) with `client_id = "client-B"` in the token request body will receive a token, because the Approved branch unconditionally calls `build_claims` once the cache entry is found. The authorization-code arm at lines 125-127 explicitly guards this:
```rust
if record.client_id != form.client_id {
    return Err(json_error(400, "invalid_client", "client_id mismatch"));
}
```
The device arm has no equivalent check. RFC 8628 §3.4 requires the AS to validate that the `client_id` in the token request matches the one from the device authorization request.

**Fix:**
```rust
DeviceGrantStatus::Approved => {
    // Validate client_id binding (mirrors auth-code arm, RFC 8628 §3.4)
    if grant.client_id != form.client_id {
        // Forget both keys anyway — the grant is consumed on any Approved read
        let _ = Cache::forget(&device_cache_key(device_code)).await;
        let _ = Cache::forget(&usercode_cache_key(&grant.normalized_user_code)).await;
        return Err(json_error(400, "invalid_client", "client_id mismatch"));
    }

    // Single-use: forget BOTH keys before minting (T-203-DEVICECODE-REPLAY)
    let _ = Cache::forget(&device_cache_key(device_code)).await;
    let _ = Cache::forget(&usercode_cache_key(&grant.normalized_user_code)).await;
    // ... rest unchanged
```

Alternatively, hoist the forget before the client_id check so the code is consumed regardless (same as the authorization-code arm's "get THEN forget BEFORE validation" discipline):
```rust
DeviceGrantStatus::Approved => {
    // Single-use: forget BOTH keys BEFORE any further validation (mirrors T-199-02)
    let _ = Cache::forget(&device_cache_key(device_code)).await;
    let _ = Cache::forget(&usercode_cache_key(&grant.normalized_user_code)).await;

    if grant.client_id != form.client_id {
        return Err(json_error(400, "invalid_client", "client_id mismatch"));
    }
    // ... mint token
```

The second form is preferable: it ensures the grant cannot be replayed even if client_id validation fails.

---

### CR-02: `generate_user_code` uses `rand::thread_rng()` — acceptable CSPRNG, but `gen_range` on charset length is fragile

**File:** `ferro-mcp-oauth/src/device.rs:150-157`
**Issue:** `rand::thread_rng()` in rand 0.8 is seeded from the OS CSPRNG and is suitable for security-sensitive randomness. However, the pattern `rng.gen_range(0..USER_CODE_CHARSET.len())` passes a `usize` range where `len() == 20`. While rand 0.8's `UniformInt` uses rejection sampling (no modular bias), this is an invisible invariant: any reader who does not know rand's internal implementation cannot verify correctness by inspection. Additionally, `generate_device_code` (via `pkce::generate_auth_code`) uses `rand::thread_rng().gen::<[u8; 32]>()` which is unambiguously safe; `generate_user_code` should align to the same readable idiom.

The actual risk is low (correct today), but for a security-critical function generating authentication codes, the implementation should be unambiguously correct to read. The preferred approach is `rand::rngs::OsRng` (explicit OS-backed CSPRNG, no thread-local state) or at minimum replace the index approach with byte-rejection sampling that is clearly correct:

**Fix:**
```rust
pub fn generate_user_code() -> String {
    use rand::RngCore;
    let mut rng = rand::rngs::OsRng;
    let mut chars = String::with_capacity(9);
    let mut count = 0;
    while count < 8 {
        let byte = (rng.next_u32() & 0xFF) as u8;
        // Rejection sampling: accept only if byte < 20 * floor(256/20) = 240
        if byte < 240 {
            chars.push(USER_CODE_CHARSET[(byte % 20) as usize] as char);
            count += 1;
        }
    }
    format!("{}-{}", &chars[..4], &chars[4..])
}
```

If `OsRng` is not desired, the current code is also acceptable since `gen_range` is bias-free — but add a comment explaining why:
```rust
// gen_range uses rejection sampling internally (no modular bias), but
// the charset length 20 is an exact power-of-2-friendly value anyway.
let idx = rng.gen_range(0..USER_CODE_CHARSET.len());
```

---

## Warnings

### WR-01: `slow_down` response is returned after updating `last_polled_at` — correct, but the cache write failure is silently swallowed

**File:** `ferro-mcp-oauth/src/token.rs:213-231`
**Issue:** The `last_polled_at` cache write (line 218) uses `let _ = Cache::put(...)` — silently discarding cache errors. If the write fails, the old `last_polled_at` persists and a rapid-fire attacker can bypass `slow_down` enforcement entirely, because the next poll still sees the stale (older) timestamp. For a DoS-protection mechanism the silent discard is acceptable in most deployments, but for a security control it should at minimum be logged or returned as `server_error`:

**Fix:**
```rust
// Propagate cache write errors rather than discarding them;
// a failed write means slow_down enforcement is blind for this poll.
Cache::put(
    &device_cache_key(device_code),
    &updated,
    Some(DEVICE_CODE_TTL),
)
.await
.map_err(|e| json_error(500, "server_error", &format!("cache error: {e}")))?;
```

---

### WR-02: Code-entry form (`render_code_entry_form`) POSTs without a CSRF token

**File:** `ferro-mcp-oauth/src/device.rs:369-387`
**Issue:** The code-entry form (`GET /device` with no `user_code`) renders a POST form with no `_token` hidden field. The `device_verification_post` handler short-circuits on this path (lines 587-593) and redirects to `GET /device?user_code=…` without validating CSRF. Because the form only moves a user-entered code from the form into the URL query string (no state change occurs), the absence of CSRF is not exploitable for the code-entry step itself. However:

1. The `device_verification_post` handler uses a single dispatch function for both paths. An attacker who submits both `user_code` and `device_code` in the same POST body reaches the CSRF-validated approve/deny path directly — this is already guarded because `form.device_code.is_empty()` must be true for the early-return. But the condition `!form.user_code.is_empty() && form.device_code.is_empty()` (line 587) relies on the attacker not being able to set `device_code` to a non-empty value in the code-entry form. Since the code-entry form does not have a `device_code` input, this is fine — but it is fragile. If the form is ever updated to add more fields, the bypass condition could break silently.

2. The condition at line 587 returns early before CSRF validation, meaning a cross-site POST to `/device` with only `user_code` will succeed (PRG redirect). This does not cause an authorization state change, but it does expose the user's entered code in a `Location` header that may appear in server logs or referrer headers.

**Fix:** Either add a CSRF token to the code-entry form (reducing the fragility of the two-mode dispatch) or add a comment explicitly documenting why CSRF is not needed on the code-entry path:
```rust
// Code-entry POST: only moves user input into the URL query string; no auth
// state changes. CSRF not required here because the PRG redirect does not
// perform or commit any authorization action — the approve/deny POST (which
// does) validates CSRF separately.
if !form.user_code.is_empty() && form.device_code.is_empty() {
```

---

### WR-03: `device_verification_get` stores a return URL containing user-supplied `user_code` via `store_oauth_return_to`

**File:** `ferro-mcp-oauth/src/device.rs:473-478`
**Issue:** `store_oauth_return_to` was designed (per `resume.rs` module docs, lines 28-29) to store only URLs constructed by the `/authorize` handler itself — never user-supplied input. In `device_verification_get`, the stored URL is `/device?user_code={encoded_uc}` where `encoded_uc` is the percent-encoded value of the user-supplied `user_code` query parameter.

The `url_encode` helper is applied (line 473), limiting the character set to RFC 3986 unreserved characters plus percent-encoded bytes. This mitigates a `Location`-header injection attack (CRLF injection requires `%0d%0a`). However:

1. The `resume.rs` module contract is violated: the docstring says "the stored value originates from the `/authorize` handler... never from user input." Device's use of the same session key with user-controlled content breaks this documented invariant.
2. An attacker who controls the `user_code` query parameter can craft a return URL pointing to `/device?user_code=AAAA-AAAA%3F...` (URL-encoded query injection). After `url_encode`, the `=` and `?` characters are percent-encoded (`%3D`, `%3F`), so the stored value is `"/device?user_code=AAAA-AAAA%3F..."`. This is safe for `Location` header purposes but the contract violation itself is a latent risk if `store_oauth_return_to` is ever used with less strict encoding.

**Fix:** Either validate that `user_code` matches the expected 9-character `XXXX-XXXX` charset pattern before using it in the return URL, or construct the return URL independently of user input when `user_code` is absent/invalid:

```rust
// Only carry the user_code into the return URL if it matches the expected
// format; otherwise redirect to /device with no query param.
let encoded_uc = user_code_param
    .as_deref()
    .filter(|uc| {
        // Basic format check: 9 chars, hyphen at position 4
        uc.len() == 9 && uc.as_bytes().get(4) == Some(&b'-')
    })
    .map(|uc| url_encode(uc))
    .unwrap_or_default();
let return_url = if encoded_uc.is_empty() {
    "/device".to_string()
} else {
    format!("/device?user_code={encoded_uc}")
};
crate::resume::store_oauth_return_to(return_url);
```

---

## Info

### IN-01: Magic number `600` duplicated in `token.rs` expiry guard — should reference `DEVICE_CODE_TTL`

**File:** `ferro-mcp-oauth/src/token.rs:202`
**Issue:** The manual TTL guard uses the literal `600`:
```rust
if now_unix - grant.created_at > 600 {
```
`DEVICE_CODE_TTL` is defined as `Duration::from_secs(600)` in `device.rs` and is already imported at line 23. Using the constant makes the intent clear and prevents the two values from drifting if `DEVICE_CODE_TTL` is ever adjusted.

**Fix:**
```rust
if now_unix - grant.created_at > DEVICE_CODE_TTL.as_secs() as i64 {
```

---

_Reviewed: 2026-06-11T13:34:59Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
