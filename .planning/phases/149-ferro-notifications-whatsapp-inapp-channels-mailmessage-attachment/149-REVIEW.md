---
phase: 149-ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment
reviewed: 2026-04-29T00:00:00Z
depth: standard
files_reviewed: 14
files_reviewed_list:
  - ferro-notifications/Cargo.toml
  - ferro-notifications/src/channel.rs
  - ferro-notifications/src/channels/future.rs
  - ferro-notifications/src/channels/in_app.rs
  - ferro-notifications/src/channels/mail.rs
  - ferro-notifications/src/channels/mod.rs
  - ferro-notifications/src/channels/whatsapp.rs
  - ferro-notifications/src/dispatcher.rs
  - ferro-notifications/src/error.rs
  - ferro-notifications/src/lib.rs
  - ferro-notifications/src/notification.rs
  - ferro-notifications/tests/smtp_attachment_integration.rs
  - framework/src/lib.rs
  - .github/workflows/publish.yml
findings:
  critical: 0
  warning: 5
  info: 6
  total: 11
status: issues_found
---

# Phase 149: Code Review Report

**Reviewed:** 2026-04-29
**Depth:** standard
**Files Reviewed:** 14
**Status:** issues_found

## Summary

Phase 149 ships three substantive surface additions to `ferro-notifications`: a WhatsApp channel via static facade, an in-app SSE channel with paired DB+broadcast legs, and `MailMessage` attachment support with a 25 MB cap and dual SMTP/Resend rendering. The work is well-tested (unit tests for builders, dispatcher routing tables, base64 fixture, payload omission guards, Mailpit integration test for the SMTP round-trip) and the type design is clean — fallible builder for attachments, typed error variants for `AttachmentTooLarge` and `WhatsApp`, explicit `whatsapp_enabled=false` default.

The findings below are predominantly non-blocking. The two issues most worth attention are a non-atomic two-leg in-app dispatch (broadcast failure leaves the persisted notification published to nobody — acceptable per CONTEXT.md D-08 but the docstring overstates "either failure aborts the dispatch") and a `MailConfig::credentials` builder that silently mutates a Resend-driver config into an SMTP shape via `get_or_insert`. No critical security issues, no secret leakage in error paths (verified by reading `ferro_whatsapp::Error` Display impls), no path traversal in attachment filenames (lettre handles encoding; Resend receives raw filename in JSON which is the documented contract).

## Warnings

### WR-01: `MailConfig::credentials` silently mutates Resend config into SMTP shape

**File:** `ferro-notifications/src/dispatcher.rs:290-301`
**Issue:** `credentials(...)` calls `self.smtp.get_or_insert(SmtpConfig { host: String::new(), ... })`. If a caller chains `MailConfig::resend(...).credentials(...)` (a plausible mistake — credentials is the natural-feeling method on a mail config), the call inserts a fresh `SmtpConfig` with empty host while leaving `driver: MailDriver::Resend` and `resend: Some(...)`. Result: Resend send paths still work, but the config now carries phantom SMTP state that the SMTP path would happily use if anything later flipped `driver` (in code or in a config-merge utility). It also silently swallows what is almost certainly a bug in the caller.

**Fix:** Either require the SMTP precondition and return `Result`, or no-op + warn when the driver is Resend:

```rust
pub fn credentials(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
    if !matches!(self.driver, MailDriver::Smtp) {
        tracing::warn!("MailConfig::credentials called on non-SMTP driver; ignoring");
        return self;
    }
    let smtp = self.smtp.get_or_insert_with(|| SmtpConfig {
        host: String::new(),
        port: 587,
        username: None,
        password: None,
        tls: true,
    });
    smtp.username = Some(username.into());
    smtp.password = Some(password.into());
    self
}
```

Same concern, lower severity, in `no_tls()` (line 310) — silently no-ops on Resend, which is at least the conservative direction but still hides a caller mistake.

### WR-02: In-app dispatch is not atomic; docstring overstates the guarantee

**File:** `ferro-notifications/src/dispatcher.rs:719-772`
**Issue:** The docstring on `send_in_app` says "Either failure aborts the dispatch (no partial-success silent fallback)." That is technically true for the *return value* — both legs return `Err` on their own failure path — but it is not true for the *system state*. Specifically: if `cfg.store.store(...)` succeeds and `cfg.broker.broadcast(...)` then fails, the persisted DB row exists, the SSE event was never published, and the function returns `Err`. The caller has no way to learn from the error which leg failed, and a retry will write a duplicate DB row. This is the documented design (CONTEXT.md D-08 says DB-leg first specifically so the broker can replay on reconnect) — but the docstring as written suggests stronger atomicity than the code provides.

**Fix:** Soften the docstring and surface the asymmetry to callers:

```rust
/// Writes both legs; returns the first error encountered. The legs are NOT
/// transactional: if the DB write succeeds and the broadcast then fails,
/// the persisted row remains and a naive retry will create a duplicate.
/// Per CONTEXT.md D-08 this is intentional — the broker can replay from the
/// store on reconnect — but callers performing manual retries should
/// dedupe on (notifiable_id, notification_type, idempotency-key).
```

Optionally, return a richer error variant (`Error::InAppPartialSuccess { db_persisted: true, broadcast_error: ... }`) so callers can distinguish "fully failed" from "persisted-but-not-broadcast" without grep'ing the message string.

### WR-03: Resend driver does not validate response body shape; treats 2xx as success even on partial failures

**File:** `ferro-notifications/src/dispatcher.rs:599-617`
**Issue:** The Resend send path checks `response.status().is_success()` and treats any 2xx as a successful send. The Resend API can return 2xx with a JSON body like `{"id": "...", "error": {...}}` for certain edge cases (the public docs are not explicit about this — verify against current Resend API spec via context7 if hardening is desired). More concretely: there is no parsing of the Resend response `id`, so the dispatcher cannot log or return a Resend-side message identifier. A future operator debugging "I sent a notification at 14:32 — where is it in Resend?" has no correlation token.

**Fix:** Parse the success response and log the Resend message id at minimum:

```rust
if !response.status().is_success() { /* existing path */ }
let body: serde_json::Value = response.json().await
    .map_err(|e| Error::mail(format!("Resend response parse failed: {e}")))?;
let id = body.get("id").and_then(|v| v.as_str()).unwrap_or("<no-id>");
info!(to = %to, resend_id = %id, "Mail notification sent via Resend");
```

### WR-04: WhatsApp channel does no rate-limit/retry handling; transient errors propagate as terminal

**File:** `ferro-notifications/src/dispatcher.rs:696-717`
**Issue:** The dispatcher calls `ferro_whatsapp::WhatsApp::send(...).await?` and propagates any `ferro_whatsapp::Error` (including `RateLimit` and `NetworkError`) as a hard failure. This is consistent with the rest of the dispatcher (no retry on Resend or SMTP either) and may be the right call (callers can layer their own retry via `ferro-queue`), but it is worth being explicit: there is no exponential backoff, no jitter, no observability beyond a single `info!` at success and the propagated error. For a v1.0-grade WhatsApp surface that consumers will hit at scale, the asymmetry between "rate limit" (retryable with backoff) and "invalid phone number" (terminal) should be visible to the caller.

**Fix:** Either document explicitly in the dispatcher docstring that retry is the caller's responsibility (and recommend the queue path), or distinguish retryable/terminal errors at the type level. The existing `Error::WhatsApp(#[from] ferro_whatsapp::Error)` preserves the inner variant, so callers *can* match on it — but nothing in the public API points them at this. A 2-line note in the rustdoc on `send_whatsapp` would close this.

### WR-05: Database channel is silent no-op when configured store is absent — distinct from "channel not in via()"

**File:** `ferro-notifications/src/dispatcher.rs:619-657`
**Issue:** When a notification declares `Channel::Database` in `via()` but `NotificationConfig::database_store` is `None`, the dispatcher logs at `info!` level "Database notification stored (placeholder — no store configured)" and returns `Ok(())`. The notification was *not* persisted, but the caller sees success. This is the documented backward-compat behavior (per the docstring on lines 622-624 — "no consumer of the placeholder is broken"), but it diverges from the WhatsApp adapter which gates on a *typed boolean* (`whatsapp_enabled`) that the consumer must explicitly opt out of. A consumer who upgrades to this phase, declares `Channel::Database` in `via()`, and forgets to call `with_database_store(...)` will silently lose every database notification with a misleading log message.

**Fix:** Two options:
1. Match the WhatsApp pattern — require explicit opt-in via something like `database_enabled` and return `Err(Error::ChannelNotAvailable)` when not configured. This is a breaking change for the placeholder-log path but it's pre-1.0 and the placeholder has no real consumers.
2. Keep the silent-success path but emit at `warn!` level so it appears in default-configured log output:

```rust
warn!(
    notification_type = %message.notification_type,
    "Database notification dropped — no store configured. \
     Call NotificationConfig::with_database_store() at startup."
);
```

Option 2 is the minimal-churn fix.

## Info

### IN-01: `Error::Mail` variant is `String`-only — error chains are flattened, breaking the `.source()` walk

**File:** `ferro-notifications/src/error.rs:9-10`
**Issue:** `Mail(String)` is the only error variant (other than `WhatsApp`) that does not preserve a source error. All call sites in the SMTP/Resend paths build the message via `format!("Failed to send email: {e}")` (e.g. dispatcher.rs:547, 606, 612), losing the original `lettre::transport::smtp::Error` or `reqwest::Error`. A consumer cannot programmatically distinguish "DNS lookup failed" from "auth rejected" from "5xx from Resend" — they can only string-match the message.

**Fix:** Migrate to a structured Mail variant when next touching the error type:
```rust
#[error("mail error: {0}")]
Mail(#[from] MailError),  // new error type wrapping lettre + reqwest sources
```
Out of scope for this phase, but worth tracking. The `WhatsApp(#[from])` variant is the right model.

### IN-02: `Error::Broadcast` flattens `ferro_broadcast::Error` to a string

**File:** `ferro-notifications/src/dispatcher.rs:764` and `ferro-notifications/src/error.rs:25-26`
**Issue:** `cfg.broker.broadcast(...).await.map_err(|e| Error::broadcast(e.to_string()))` — same pattern as IN-01. The docstring on the Error variant explicitly notes "no `#[from]` available" but doesn't say why. If `ferro_broadcast::Error` is in the same workspace, an `#[from]` impl is feasible; the current string-based wrap loses type information.

**Fix:** Add `#[from] ferro_broadcast::Error` if there is no foreign-trait collision. If there is, document it inline in the variant doc.

### IN-03: `WhatsAppMessage::template` parameters typed as `Vec<serde_json::Value>` — no compile-time shape check

**File:** `ferro-notifications/src/channels/whatsapp.rs:26-38`
**Issue:** Parameters are untyped `serde_json::Value` with the comment "must contain typed parameter objects per Meta spec, e.g. `serde_json::json!({"type": "text", "text": "value"})`". A typo (`"txt"` instead of `"text"`) is undetectable until the Meta API rejects the send at runtime. This is a known tradeoff — the WhatsApp template parameter spec is large and growing — but it's worth marking as a v1.x hardening item.

**Fix:** Out of scope. Track for a future phase: introduce a typed `TemplateParameter` enum (Text, Currency, DateTime, Image, etc.) in `ferro-whatsapp` and mirror in `ferro-notifications`.

### IN-04: `inapp_to_database_message` flattens object data without conflict detection

**File:** `ferro-notifications/src/dispatcher.rs:779-790`
**Issue:** When `InAppMessage::data` is an object containing a key called `"payload"`, the flattening branch preserves it; when it is *not* an object, the wrapping branch creates a `"payload"` key. There is no semantic distinction in the resulting `DatabaseMessage` between "the original was `{"payload": "x"}`" and "the original was the scalar `"x"`". A downstream consumer that round-trips the data field cannot reconstruct the original `InAppMessage::data`.

**Fix:** Either document this asymmetry explicitly in the helper's rustdoc, or use a sentinel key (`"__inapp_payload"`) for the wrap branch so round-trip is unambiguous. Documentation is the lighter touch.

### IN-05: Resend driver hardcodes `https://api.resend.com/emails` — no override for testing or self-hosted proxies

**File:** `ferro-notifications/src/dispatcher.rs:601`
**Issue:** The Resend endpoint URL is a string literal in the send path. There is no integration-test seam (the SMTP path has Mailpit; Resend has nothing equivalent). For testing the Resend code path locally, a developer must intercept HTTPS traffic via mitmproxy or accept a real Resend API key in env.

**Fix:** Out of scope for this phase. Worth noting for a future test-infrastructure phase: add `ResendConfig::endpoint: Option<String>` defaulting to `https://api.resend.com/emails`, and a corresponding mock-server-backed integration test under the `integration-tests` feature.

### IN-06: `from_env` for `WHATSAPP_ENABLED` accepts only `"true"`/`"false"` (not `"1"`/`"0"`/`"yes"`/etc.)

**File:** `ferro-notifications/src/dispatcher.rs:129-132`
**Issue:** `env::var("WHATSAPP_ENABLED").ok().and_then(|v| v.parse::<bool>().ok()).unwrap_or(false)`. `bool::from_str` only accepts the literal strings `"true"` and `"false"` (case-sensitive). Common conventions like `WHATSAPP_ENABLED=1` or `WHATSAPP_ENABLED=YES` silently fall back to `false`. The unit test `test_notification_config_whatsapp_disabled_when_env_garbage` asserts this behavior with `"yes-please"` — but `"1"` is not garbage, it is the dominant convention in 12-factor apps.

**Fix:** Either document the strict `"true"`/`"false"` requirement in the `from_env` rustdoc, or accept the broader set:

```rust
whatsapp_enabled: env::var("WHATSAPP_ENABLED")
    .ok()
    .map(|v| matches!(v.to_lowercase().as_str(), "true" | "1" | "yes" | "on"))
    .unwrap_or(false),
```

Documentation is sufficient; the broader-set approach is a quality-of-life improvement.

---

_Reviewed: 2026-04-29_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
