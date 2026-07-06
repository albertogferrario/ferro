---
phase: 141-protocol-uplift
reviewed: 2026-04-20T00:00:00Z
depth: standard
files_reviewed: 22
files_reviewed_list:
  - ferro-json-ui/src/render.rs
  - ferro-stripe/Cargo.toml
  - ferro-stripe/src/lib.rs
  - ferro-stripe/src/testing.rs
  - ferro-stripe/src/webhook/events.rs
  - ferro-stripe/src/webhook/mod.rs
  - ferro-stripe/src/webhook/queue.rs
  - ferro-stripe/src/webhook/sync.rs
  - ferro-stripe/src/webhook/verify.rs
  - ferro-stripe/tests/dispatcher.rs
  - ferro-stripe/tests/fixtures/stripe_events/account_updated.json
  - ferro-stripe/tests/fixtures/stripe_events/charge_dispute_created.json
  - ferro-stripe/tests/fixtures/stripe_events/charge_refunded.json
  - ferro-stripe/tests/fixtures/stripe_events/checkout_session_completed.json
  - ferro-stripe/tests/fixtures/stripe_events/checkout_session_expired.json
  - ferro-stripe/tests/fixtures/stripe_events/customer_subscription_deleted.json
  - ferro-stripe/tests/fixtures/stripe_events/customer_subscription_updated.json
  - ferro-stripe/tests/fixtures/stripe_events/invoice_paid.json
  - ferro-stripe/tests/fixtures/stripe_events/payment_intent_payment_failed.json
  - ferro-stripe/tests/fixtures/stripe_events/payment_intent_succeeded_connect.json
  - ferro-stripe/tests/parser_contract.rs
  - framework/Cargo.toml
  - framework/src/lib.rs
findings:
  critical: 1
  warning: 3
  info: 2
  total: 6
status: issues_found
---

# Phase 141: Code Review Report

**Reviewed:** 2026-04-20
**Depth:** standard
**Files Reviewed:** 22
**Status:** issues_found

## Summary

This phase introduces the ferro-stripe webhook dispatch stack (verify, typed events, sync dispatcher, queue job, testing helpers, parser-contract integration tests) and extends ferro-json-ui's render.rs with new component renderers. The stripe code is well-structured: HMAC verification delegates correctly to the upstream crate, the dispatcher pattern is clean, and the integration tests exercise the golden-JSON fixture contract thoroughly.

Two issues require attention before merging: one XSS surface in the render.rs icon injection path (Critical), and a panic-on-missing-dispatcher in the queue job that should be a recoverable error (Warning). Three lower-severity issues are noted below.

---

## Critical Issues

### CR-01: Raw icon HTML injected without escaping in `render_action_card`

**File:** `ferro-json-ui/src/render.rs:449-452`

**Issue:** The `props.icon` value is interpolated directly into HTML via Rust's `{icon}` format argument, bypassing `html_escape`. If the icon field is ever populated from a data path or external source rather than a hardcoded developer string, this is an XSS injection point. The same unescaped pattern appears in `render_stat_card` (line ~2318, noted with a `// raw` comment) and implicitly in `render_sidebar_nav_item` (icon treated as raw SVG). The `ActionCard` case has no comment normalizing this behavior and no structural guarantee the value is always developer-controlled.

**Fix:** Escape the icon content, or if raw SVG passthrough is intentional, add an explicit comment and ensure the field is not reachable from user-supplied data. For the cases where the icon is guaranteed to be developer-supplied SVG, document the trust boundary:

```rust
// Option A: always escape (safe default)
if let Some(ref icon) = props.icon {
    html.push_str(&format!(
        "<div class=\"w-10 h-10 ...\">{}}</div>",
        html_escape(icon)
    ));
}

// Option B: if raw SVG passthrough is intentional, document and gate it
// Only use raw interpolation when the field is provably not user-reachable.
// Add: #[doc = "Raw HTML — must be developer-supplied SVG only."]
```

---

## Warnings

### WR-01: Panic instead of recoverable error when dispatcher is absent in `ProcessStripeWebhook::handle`

**File:** `ferro-stripe/src/webhook/queue.rs:68-71`

**Issue:** `handle()` calls `.expect()` when `dispatcher` is `None`. The doc comment labels this a "programming error, not a runtime condition," but queue workers typically run in a loop and a panic will crash the entire worker task rather than failing only the bad job. A `Err(JobFailed)` return is more appropriate — it lets the queue mark the job as failed, log it, and continue processing other jobs.

```rust
// Current:
let dispatcher = self
    .dispatcher
    .as_ref()
    .expect("ProcessStripeWebhook requires dispatcher ...");

// Fix:
let dispatcher = self.dispatcher.as_ref().ok_or_else(|| {
    ferro_queue::Error::JobFailed {
        job: "ProcessStripeWebhook".to_string(),
        message: "dispatcher not injected — use ProcessStripeWebhook::new()".to_string(),
    }
})?;
```

### WR-02: Confirm dialog silently drops `message` — only `confirmTitle` ever shown

**File:** `ferro-json-ui/src/render.rs:636-637`

**Issue:** The JavaScript expression `confirm(this.dataset.confirmTitle || this.dataset.confirmMessage)` always evaluates the left operand first. Because `confirmTitle` is always present (the `confirm_attrs` code unconditionally sets `data-confirm-title` from `confirm.title`), the `|| this.dataset.confirmMessage` branch is dead. Users confirming destructive actions see the title string only, never the more detailed message even when it was provided.

**Fix:** Combine both strings in the `confirm()` call, or use only the message field as the prompt:

```rust
// Use message when present, fall back to title:
let onclick = if item.action.confirm.is_some() {
    " onclick=\"return confirm(this.dataset.confirmMessage || this.dataset.confirmTitle)\""
} else {
    ""
};
```

Or concatenate both:
```javascript
return confirm((this.dataset.confirmTitle||'') + (this.dataset.confirmMessage ? '\n'+this.dataset.confirmMessage : ''))
```

### WR-03: `amount_total_cents` defaults to 0 silently for absent checkout amounts

**File:** `ferro-stripe/src/webhook/events.rs:90`

**Issue:** `session.amount_total.unwrap_or(0)` maps a missing `amount_total` to zero. For a `checkout.session.completed` event this field is absent on free / setup-mode sessions, but handlers that act on `amount_total_cents == 0` to gate access or trigger fulfillment cannot distinguish "paid zero" from "amount was not present." This is a logic hazard for callers that check for positive amounts.

**Fix:** Expose the original optionality in the struct or document the zero-means-absent contract explicitly:

```rust
// Option A: keep the field optional
pub amount_total_cents: Option<i64>,
// and: amount_total_cents: session.amount_total,

// Option B (current shape, add doc):
/// Total amount in cents. `0` when `amount_total` is absent from the
/// Stripe event (free or setup-mode sessions). Callers must not use
/// this field to assert payment was received.
pub amount_total_cents: i64,
```

---

## Info

### IN-01: `row_key` extraction logic duplicated four times in `render_data_table`

**File:** `ferro-json-ui/src/render.rs:1151-1161, 1193-1203, 1245-1255, 1288-1298`

**Issue:** The same `row_key` extraction block (resolve from row value, fall back to index) is copied verbatim four times within `render_data_table`. Extracting it into a small closure or helper function would reduce the maintenance surface.

**Fix:**
```rust
let extract_row_key = |row: &Value, index: usize| -> String {
    props.row_key.as_ref()
        .and_then(|rk| row.get(rk))
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            _ => None,
        })
        .unwrap_or_else(|| index.to_string())
};
```

### IN-02: `signed_webhook_payload` in testing.rs uses wall-clock time — subtle constraint for fixture reuse

**File:** `ferro-stripe/src/testing.rs:160`

**Issue:** `signed_webhook_payload` signs with `Utc::now()`. `stripe::Webhook::construct_event` enforces a 300-second tolerance by default, so any signature produced by this helper expires five minutes after generation. The in-process round-trip tests work correctly. However, any future test that pre-generates a signature string (e.g., in a fixture file or snapshot) and later replays it will silently fail with a `WebhookVerification` error that looks like a signature mismatch.

**Suggestion:** Add a doc note clarifying the time-bounded nature of generated signatures, or accept a timestamp parameter so tests that need determinism can pass a fixed value.

---

_Reviewed: 2026-04-20_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
