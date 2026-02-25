# Phase 68: Resend Mail Driver - Research

**Researched:** 2026-02-25
**Domain:** Resend HTTP API integration for ferro-notifications mail channel
**Confidence:** HIGH

<research_summary>
## Summary

Researched the Resend email API and Rust ecosystem to determine the best approach for adding Resend as an alternative mail transport in ferro-notifications. The Resend API is a simple REST endpoint (POST `/emails`) with Bearer token auth — making raw `reqwest` the right choice over adding the `resend-rs` SDK dependency.

The current `ferro-notifications` dispatcher is hardcoded to SMTP via `lettre`. The scaffolded app config already reads `MAIL_DRIVER` from env but the framework ignores it. The integration requires: (1) a mail driver abstraction in the dispatcher, (2) a Resend transport using existing `reqwest` dependency, and (3) env-based driver selection wired through `NotificationConfig::from_env()`.

**Primary recommendation:** Use raw `reqwest` (already a dependency) to call the Resend API. Add a `MailDriver` enum to `NotificationConfig` and dispatch in `send_mail` based on driver. No new crate dependencies needed.
</research_summary>

<standard_stack>
## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| reqwest | 0.12 | HTTP client for Resend API | Already in Cargo.toml for Slack webhooks. Resend API is one POST call. |
| serde/serde_json | 1.x | JSON request/response serialization | Already in Cargo.toml |
| lettre | 0.11 | SMTP transport (existing) | Stays for SMTP driver, no changes needed |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tracing | 0.1 | Logging (existing) | Already used for mail send logging |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Raw reqwest | resend-rs 0.21.1 (official SDK) | SDK adds dependency weight for a single POST call. Has rate limiting built in, but we don't need it at framework level — Resend handles server-side. Pre-1.0 crate may have breaking changes. |
| Raw reqwest | resend-email crate | Third-party, less maintained than official SDK. Same argument — unnecessary dependency. |

**Installation:**
No new dependencies. `reqwest` with `json` feature already present in `ferro-notifications/Cargo.toml`.
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Recommended Approach: Driver Enum in NotificationConfig

Add a `MailDriver` enum and extend the existing config to support driver selection:

```rust
/// Mail transport driver.
#[derive(Debug, Clone, Default)]
pub enum MailDriver {
    /// SMTP via lettre (default).
    #[default]
    Smtp,
    /// Resend HTTP API.
    Resend,
}
```

### Pattern 1: Config Hierarchy with Shared + Driver-Specific Fields

```rust
/// Unified mail configuration supporting multiple drivers.
#[derive(Clone)]
pub struct MailConfig {
    /// Which driver to use.
    pub driver: MailDriver,
    /// Default from address (shared across all drivers).
    pub from: String,
    /// Default from name (shared across all drivers).
    pub from_name: Option<String>,
    /// SMTP-specific config (only when driver = Smtp).
    pub smtp: Option<SmtpConfig>,
    /// Resend-specific config (only when driver = Resend).
    pub resend: Option<ResendConfig>,
}

pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub tls: bool,
}

pub struct ResendConfig {
    pub api_key: String,
}
```

### Pattern 2: from_env() Driver Selection

```rust
impl MailConfig {
    pub fn from_env() -> Option<Self> {
        let from = env::var("MAIL_FROM_ADDRESS").ok().filter(|s| !s.is_empty())?;
        let from_name = env::var("MAIL_FROM_NAME").ok().filter(|s| !s.is_empty());
        let driver_str = env::var("MAIL_DRIVER").unwrap_or_else(|_| "smtp".into());

        let (driver, smtp, resend) = match driver_str.to_lowercase().as_str() {
            "resend" => {
                let api_key = env::var("RESEND_API_KEY").ok().filter(|s| !s.is_empty())?;
                (MailDriver::Resend, None, Some(ResendConfig { api_key }))
            }
            _ => {
                let host = env::var("MAIL_HOST").ok().filter(|s| !s.is_empty())?;
                // ... existing SMTP env parsing ...
                (MailDriver::Smtp, Some(smtp_config), None)
            }
        };

        Some(Self { driver, from, from_name, smtp, resend })
    }
}
```

### Pattern 3: Dispatch in send_mail

```rust
async fn send_mail<N: Notifiable + ?Sized>(
    notifiable: &N,
    message: &MailMessage,
) -> Result<(), Error> {
    let config = /* get config */;
    match config.driver {
        MailDriver::Smtp => Self::send_mail_smtp(notifiable, message, config).await,
        MailDriver::Resend => Self::send_mail_resend(notifiable, message, config).await,
    }
}
```

### Anti-Patterns to Avoid
- **Trait-based driver abstraction:** Overkill for 2 drivers. Enum dispatch is simpler and the drivers share no complex interface — just "send this MailMessage to this address."
- **Separate Channel variants:** Don't add `Channel::Resend`. Resend is a mail *transport*, not a channel. Users write `Channel::Mail` regardless of driver.
- **Conditional compilation with features:** Don't gate Resend behind a cargo feature. Both drivers should always be available — selection is runtime via env var.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Email deliverability | Custom retry/bounce logic | Resend handles delivery tracking | Resend tracks bounces, opens, clicks server-side. Framework just fires and gets back an ID. |
| Rate limiting | Client-side rate limiter | Resend's 2 req/sec limit + 429 handling | Simple retry-after or error propagation is sufficient. Don't build a token bucket. |
| Email validation | Custom email format validation | Resend API validates addresses | API returns 400 for invalid addresses. Framework already validates via MailMessage builder. |
| HTML to text conversion | Custom text extraction | Resend auto-generates text from HTML | When `text` is omitted, Resend creates a plain text version automatically. |

**Key insight:** Resend is an email *delivery* service. The framework's job is to translate a `MailMessage` into a Resend API call — nothing more. All deliverability, tracking, and queueing happens on Resend's side.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Missing User-Agent Header
**What goes wrong:** Resend API returns 403 (code 1010) with no clear error message.
**Why it happens:** Resend requires a `User-Agent` header on all API requests. `reqwest` sets one by default, but custom client configs might strip it.
**How to avoid:** Use `reqwest::Client::new()` which sets User-Agent automatically. Don't use `Client::builder().no_default_headers()`.
**Warning signs:** 403 errors that aren't API key related.

### Pitfall 2: Breaking MailConfig::from_env() Backwards Compatibility
**What goes wrong:** Existing apps that don't set `MAIL_DRIVER` break after upgrade.
**Why it happens:** If `from_env()` changes behavior when `MAIL_DRIVER` is unset.
**How to avoid:** Default to `MailDriver::Smtp` when `MAIL_DRIVER` is unset or empty. Existing SMTP-only users should see no behavioral change.
**Warning signs:** Mail stops working after framework upgrade with no env changes.

### Pitfall 3: Conflating RESEND_API_KEY with SMTP credentials
**What goes wrong:** Users try to put the Resend API key in `MAIL_PASSWORD` or `MAIL_USERNAME`.
**Why it happens:** Unclear documentation about which env vars apply to which driver.
**How to avoid:** Use a dedicated `RESEND_API_KEY` env var (matches Resend's own convention and the `resend-rs` SDK). Document clearly which vars each driver reads.
**Warning signs:** "Invalid API key" errors when MAIL_PASSWORD is set but RESEND_API_KEY isn't.

### Pitfall 4: Per-Message From Override Not Mapped
**What goes wrong:** `MailMessage.from` field (per-notification override) gets ignored when sending via Resend.
**Why it happens:** Forgetting to check `message.from` and falling back to config default.
**How to avoid:** In `send_mail_resend`, use `message.from.unwrap_or(config.from)` — same pattern as the existing SMTP implementation.
**Warning signs:** All emails come from default address even when notification specifies a different sender.
</common_pitfalls>

<code_examples>
## Code Examples

### Resend API Request (raw reqwest)
```rust
// Source: https://resend.com/docs/api-reference/emails/send-email
use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
struct ResendEmailPayload {
    from: String,
    to: Vec<String>,
    subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cc: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    bcc: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    headers: Option<std::collections::HashMap<String, String>>,
}

async fn send_via_resend(api_key: &str, payload: &ResendEmailPayload) -> Result<String, Error> {
    let client = Client::new();
    let response = client
        .post("https://api.resend.com/emails")
        .bearer_auth(api_key)
        .json(payload)
        .send()
        .await
        .map_err(|e| Error::mail(format!("Resend HTTP request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(Error::mail(format!("Resend API error {}: {}", status, body)));
    }

    // Response: { "id": "uuid" }
    let result: serde_json::Value = response.json().await
        .map_err(|e| Error::mail(format!("Failed to parse Resend response: {}", e)))?;

    Ok(result["id"].as_str().unwrap_or("unknown").to_string())
}
```

### MailMessage to Resend Payload Conversion
```rust
// Source: Ferro framework pattern
fn mail_message_to_resend_payload(
    to: &str,
    message: &MailMessage,
    config: &MailConfig,
) -> ResendEmailPayload {
    let from = message.from.clone().unwrap_or_else(|| {
        if let Some(ref name) = config.from_name {
            format!("{} <{}>", name, config.from)
        } else {
            config.from.clone()
        }
    });

    let headers = if message.headers.is_empty() {
        None
    } else {
        Some(message.headers.iter().cloned().collect())
    };

    ResendEmailPayload {
        from,
        to: vec![to.to_string()],
        subject: message.subject.clone(),
        html: message.html.clone(),
        text: if message.html.is_some() { None } else { Some(message.body.clone()) },
        cc: message.cc.clone(),
        bcc: message.bcc.clone(),
        reply_to: message.reply_to.clone(),
        headers,
    }
}
```

### Resend API Error Response Format
```json
// Source: https://resend.com/docs/api-reference/introduction
// 400 Bad Request
{
  "statusCode": 400,
  "message": "Validation error",
  "name": "validation_error"
}

// 422 - missing required field
{
  "statusCode": 422,
  "message": "Missing `to` field.",
  "name": "missing_required_field"
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| SMTP-only mail | Multi-driver (SMTP + HTTP APIs) | Industry trend 2023+ | HTTP APIs (Resend, SendGrid, Postmark) simpler than SMTP config |
| lettre for all mail | lettre for SMTP, reqwest for HTTP APIs | N/A (framework decision) | Clean separation: SMTP uses lettre, HTTP APIs use reqwest |

**New tools/patterns to consider:**
- **Resend Batch API:** POST `/emails/batch` to send up to 100 emails in one request. Not needed now but good for future bulk notification support.
- **Resend Idempotency Keys:** Pass `Idempotency-Key` header to prevent duplicate sends during retries. Useful for job queue retry scenarios.
- **Resend Scheduled Sending:** `scheduled_at` parameter for future delivery. Could map to a `MailMessage::schedule()` builder method later.

**Deprecated/outdated:**
- N/A — Resend API is stable and current.
</sota_updates>

<open_questions>
## Open Questions

1. **Should `MailMessage` expose Resend-specific features (tags, idempotency)?**
   - What we know: Resend supports `tags` (key/value metadata) and `Idempotency-Key` header that SMTP doesn't.
   - What's unclear: Whether to expose these as `MailMessage` fields or keep them Resend-specific.
   - Recommendation: Keep `MailMessage` transport-agnostic for now. Resend-specific features can use the existing `headers` field or be added later. Tags could map to custom headers.

2. **Should the text body always be sent even when HTML is present?**
   - What we know: Resend auto-generates text from HTML when `text` is omitted. SMTP (lettre) currently sends either HTML or text, not both.
   - What's unclear: Whether sending both provides better deliverability.
   - Recommendation: If `MailMessage` has HTML, send `html` to Resend and let it auto-generate text. If only body is set, send as `text`. Matches current SMTP behavior.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- [Resend Send Email API Reference](https://resend.com/docs/api-reference/emails/send-email) — Full endpoint spec: POST /emails, all parameters, response format
- [Resend API Introduction](https://resend.com/docs/api-reference/introduction) — Base URL, auth, rate limits (2 req/sec), error codes
- Context7 /resend/resend-rust — Official Rust SDK docs: CreateEmailBaseOptions, all builder methods, features
- [docs.rs/resend-rs](https://docs.rs/resend-rs) — v0.21.1 API: Resend client, Config, error types, rate_limit module

### Secondary (MEDIUM confidence)
- [Resend Pricing](https://resend.com/pricing) — Free: 3,000/month, Pro: $20/month 50K, Scale: $90/month 100K
- [resend-rs crates.io](https://crates.io/crates/resend-rs) — Dependency info: uses reqwest + serde internally
- [Send with Rust](https://resend.com/docs/send-with-rust) — Official Rust getting started guide

### Tertiary (LOW confidence - needs validation)
- None — all findings verified against official sources
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Resend HTTP API (POST /emails)
- Ecosystem: resend-rs SDK evaluated, raw reqwest chosen
- Patterns: Mail driver enum, config hierarchy, env-based selection
- Pitfalls: User-Agent header, backwards compat, env var confusion

**Confidence breakdown:**
- Standard stack: HIGH — reqwest already in deps, API is trivial
- Architecture: HIGH — follows existing Slack webhook pattern in codebase
- Pitfalls: HIGH — verified against API docs and error codes
- Code examples: HIGH — from official API reference + framework patterns

**Research date:** 2026-02-25
**Valid until:** 2026-03-27 (30 days — Resend API is stable)
</metadata>

---

*Phase: 68-resend-mail-driver*
*Research completed: 2026-02-25*
*Ready for planning: yes*
