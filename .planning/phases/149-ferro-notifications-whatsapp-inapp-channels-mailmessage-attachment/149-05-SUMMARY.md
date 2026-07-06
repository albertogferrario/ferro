---
phase: 149-ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment
plan: 05
subsystem: notifications
tags: [ferro-notifications, whatsapp, dispatcher, adapter, static-facade, channel-arm]

requires:
  - plan: 149-02
    provides: Channel::WhatsApp variant + Notification::to_whatsapp() + Error::WhatsApp(#[from] ferro_whatsapp::Error) + the placeholder match arm to replace
  - plan: 149-04
    provides: Mail driver attachment wiring landed; the `Channel::Mail` path is unaffected by this plan
provides:
  - NotificationConfig::whatsapp_enabled field (default false; D-14)
  - NotificationConfig::with_whatsapp_enabled(bool) consuming builder
  - NotificationConfig::from_env() reads WHATSAPP_ENABLED (parse-failure falls back to false)
  - send_whatsapp adapter that calls ferro_whatsapp::WhatsApp::send via static facade (D-04 / ARCH-FINDING-01)
  - Channel::WhatsApp dispatch arm in NotificationDispatcher::send (no longer a placeholder)
  - Channel::InApp transitional placeholder (Plan 06 finalizes; emits "channel not configured" info log)
affects:
  - 149-06 (InApp dispatcher arm — replaces the transitional placeholder added here)
  - 149-07 (publish.yml wave move + lib.rs sweep + integration tests)

tech-stack:
  added: []
  patterns:
    - Static-facade integration call from adapter — no client object injection (matches the ferro-stripe pattern); the framework gates the feature on a bool flag rather than on `Option<ClientHandle>`
    - Disabled-by-default channel gate as panic safety — `whatsapp_enabled: false` keeps the dispatch arm unreachable unless the consumer explicitly opted in (which implies they also called `WhatsApp::init`); structurally prevents the "init not called" panic for default configurations
    - Three-arm match: dedicated arms for the implemented + about-to-be-implemented channels (WhatsApp wired, InApp transitional placeholder), shared arm for the still-unimplemented pair (Sms | Push)

key-files:
  created: []
  modified:
    - ferro-notifications/src/dispatcher.rs

key-decisions:
  - "Used `unwrap_or(false)` rather than `?` propagation in `send_whatsapp` to read CONFIG: a missing global config is functionally equivalent to whatsapp_enabled=false (the dispatcher cannot be configured = no consumer opted in). Returning Ok(()) silently is the correct UX — same shape as send_sms / send_push placeholders."
  - "Cloned `message.message` inside `send_whatsapp` rather than taking by value because the function takes `&WhatsAppMessage`. `ferro_whatsapp::Message` is `Clone`, the cost is bounded (one Text body or template parameter set), and keeping the call non-consuming preserves caller flexibility (a future retry policy could re-dispatch the same message)."
  - "Channel::InApp transitional placeholder emits 'Channel not configured' (matching the disabled-WhatsApp wording) rather than the legacy 'Channel not implemented'. Rationale: the plan 06 wire-up will use NotificationConfig::in_app: Option<InAppConfig>, so the not-configured framing aligns with the eventual gate logic. Plan 06 will refine this string further if needed."

patterns-established:
  - "Channel-gate idiom for static-facade integrations: read the gate flag once via `CONFIG.get().map(|c| c.<flag>).unwrap_or(false)`, early-return Ok(()) with an info log when disabled, then proceed to the static call. This pattern applies cleanly to any future ferro-stripe-style integrations the dispatcher might consume."
  - "Per-channel match arms split out as soon as their dispatcher logic differs, even if one is still a placeholder. Plan 02 used a shared arm because all four (WhatsApp / InApp / Sms / Push) had identical placeholder behavior; once WhatsApp diverges, the arms split, leaving the still-shared pair (Sms | Push) collapsed."

requirements-completed:
  - ROADMAP-149-03

duration: 4m 7s
completed: 2026-04-28
---

# Phase 149 Plan 05: WhatsApp Channel Adapter — Static-Facade Wiring Summary

**`Channel::WhatsApp` dispatch is end-to-end functional and gated: `NotificationConfig::whatsapp_enabled` (default `false`, env-driven via `WHATSAPP_ENABLED`) controls a `send_whatsapp` adapter that calls `ferro_whatsapp::WhatsApp::send` directly through the static facade — no client injection, no panic risk for default configurations, full propagation of `ferro_whatsapp::Error` via the `#[from]` chain landed in Plan 02.**

## Performance

- **Duration:** 4m 7s
- **Started:** 2026-04-28T22:50:21Z
- **Completed:** 2026-04-28T22:54:28Z
- **Tasks:** 2
- **Files modified:** 1
- **New unit tests:** 6 (5 config / env / builder + 1 disabled-invariant)
- **Commits:** 2 (one per task)

## Accomplishments

- `NotificationConfig::whatsapp_enabled: bool` ships behind `Default` (false). The new field is purely additive — every existing call site that constructs `NotificationConfig::new()` or `NotificationConfig::default()` continues to compile and behave identically.
- `NotificationConfig::from_env()` reads `WHATSAPP_ENABLED`. Parse-failure (e.g. `"yes-please"`) falls back to false rather than propagating an error — same UX as `from_env()`'s other optional reads. Three serial env tests cover true / false / unset / garbage.
- `NotificationConfig::with_whatsapp_enabled(bool)` is the consuming-builder counterpart, matching the existing `mail()` / `slack_webhook()` shape.
- `send_whatsapp` async fn implemented per D-04: reads the `whatsapp_enabled` flag from the global `CONFIG`, returns `Ok(())` early with a structured info log when disabled, and otherwise resolves the recipient phone via `notifiable.route_notification_for(Channel::WhatsApp)`, calling `ferro_whatsapp::WhatsApp::send(&phone, message.message.clone()).await?`. The `?` operator works because `ferro_whatsapp::Error` is reachable via the `Error::WhatsApp(#[from])` variant landed in Plan 02.
- `Channel::WhatsApp` arm in `NotificationDispatcher::send` is no longer part of the placeholder collapse — it now dispatches via `Self::send_whatsapp(notifiable, &wa).await?` when `notification.to_whatsapp()` returns `Some`.
- `Channel::InApp` arm is a transitional placeholder (Plan 06 finalizes). Splitting it out from the shared `Sms | Push` arm now means Plan 06's diff is purely an arm-body replacement — no surrounding scaffolding churn.
- All 20 dispatcher tests pass (14 pre-existing from plans 04-and-prior + 5 new env/builder/default + 1 new disabled-invariant). Workspace builds clean under `cargo fmt --all -- --check`, `cargo clippy --all --all-targets -- -D warnings`, and `cargo test --all-features` (full suite — 480 / 485 / 621 / 229 / 50 etc. — zero failures).
- The `ferro_whatsapp::WhatsApp::send` panic-on-uninit-init risk is structurally mitigated: the `whatsapp_enabled: false` default means the static-facade call is unreachable unless the consumer explicitly opted in (which implies they also called `WhatsApp::init` at startup). Documented in the rustdoc on `send_whatsapp` and in the `whatsapp_enabled` field doc.

## Task Commits

Each task was committed atomically after passing fmt + clippy + test:

1. **Task 1: NotificationConfig::whatsapp_enabled field, builder, and from_env wiring** — `fb64c3ab` (feat)
2. **Task 2: send_whatsapp adapter + Channel::WhatsApp dispatch arm** — `83368453` (feat)

## Files Created/Modified

- `ferro-notifications/src/dispatcher.rs` — modified. Five logical regions touched: (1) `use crate::channels::{...}` extended with `WhatsAppMessage`; (2) `NotificationConfig` struct gains the `whatsapp_enabled: bool` field; (3) `from_env()` reads `WHATSAPP_ENABLED` with bool parse + default-false fallback; (4) `with_whatsapp_enabled(bool)` consuming builder added directly after `slack_webhook(...)`; (5) match block in `send()` split out `Channel::WhatsApp` (real dispatch) and `Channel::InApp` (transitional placeholder) arms — `Channel::Sms | Channel::Push` remain on the shared not-implemented arm; (6) `send_whatsapp` async fn added directly after `send_slack`. Test module gained 6 new tests; one existing test (`test_notification_config_default`) was tightened to also assert the new field's default. No existing test was deleted; assertions only widened.

## Decisions Made

- **`unwrap_or(false)` rather than error propagation when reading CONFIG.** The `send_whatsapp` adapter reads `CONFIG.get().map(|c| c.whatsapp_enabled).unwrap_or(false)`. A missing CONFIG (no `NotificationDispatcher::configure(...)` call) is functionally equivalent to whatsapp_enabled=false — the dispatcher hasn't been configured, so no consumer has opted in. Returning Ok(()) silently with an info log preserves dispatcher behavior parity with the legacy Sms/Push placeholders and avoids surfacing a misleading "config not found" error to the consumer.
- **`message.message.clone()` rather than taking by value.** `ferro_whatsapp::WhatsApp::send` takes `Message` by value. `send_whatsapp` receives `&WhatsAppMessage` (matches the existing `send_mail` / `send_slack` signature shape — non-consuming reference for caller flexibility). Cloning at the call boundary is bounded (one Text body or one template parameter set per send) and preserves the option of future retry-policy logic that re-dispatches the same `WhatsAppMessage` without re-invoking `notification.to_whatsapp()`.
- **Channel::InApp transitional placeholder uses "Channel not configured" wording.** Plan 06 will gate InApp on `NotificationConfig::in_app: Option<InAppConfig>`, mirroring the `whatsapp_enabled` flag pattern. The "not configured" framing aligns with the eventual gate logic and reads consistently with the `send_whatsapp`-disabled log line. The legacy "not implemented" wording stays on the still-shared `Sms | Push` arm where nothing is being added in this phase.
- **Match arms split now rather than in Plan 06.** The plan author called out splitting `InApp` out of the shared placeholder arm at the same time as `WhatsApp`. The cost is one extra match arm; the benefit is Plan 06's diff becomes purely an arm-body replacement, with no surrounding scaffolding churn that would conflict with this commit during review.

## Deviations from Plan

None. Plan 05 executed exactly as written. Both tasks landed in their planned commits with no Rule 1/2/3 fixes required and no Rule 4 architectural decisions surfaced.

The plan's must-haves all hold:

| Must-have | Status |
|-----------|--------|
| Channel::WhatsApp arm wired in NotificationDispatcher::send | ✓ at `83368453` |
| When whatsapp_enabled=true and notifiable returns a phone, send_whatsapp calls ferro_whatsapp::WhatsApp::send(&phone, msg.message) | ✓ verified by `ferro_whatsapp::WhatsApp::send(&phone, message.message.clone()).await?` |
| When whatsapp_enabled=false, send_whatsapp emits info log and returns Ok(()) | ✓ verified by `if !enabled { info!(...); return Ok(()); }` |
| When whatsapp_enabled=true but notifiable returns None, send_whatsapp returns Error::ChannelNotAvailable | ✓ verified by `.ok_or_else(|| Error::ChannelNotAvailable("No WhatsApp route configured".into()))?` |
| ferro_whatsapp::Error propagates as Error::WhatsApp via #[from] from Plan 02 | ✓ verified by `.await?` (the `?` triggers the `#[from]` conversion) |
| NotificationConfig::whatsapp_enabled and from_env reads WHATSAPP_ENABLED | ✓ at `fb64c3ab` |

## Issues Encountered

None of substance. The static-facade pattern from `ferro-whatsapp::WhatsApp::send` integrated cleanly — the only point worth noting is that the panic-on-uninit-init contract ([`ferro-whatsapp/src/client.rs:58`](../../../ferro-whatsapp/src/client.rs)) is not surfaced through the type system, so the structural mitigation (the `whatsapp_enabled: false` default) is the only safety net. This is acceptable given that consumers who flip the flag have unambiguously opted into the integration and are expected to have called `WhatsApp::init` per the `ferro-whatsapp` quickstart docs.

## User Setup Required

None for this plan's verification — the disabled-by-default gate means the dispatcher works end-to-end without any WhatsApp credentials.

For consumers who want to use the WhatsApp channel:

1. Add `ferro-whatsapp` to their `Cargo.toml` (already a dep of `ferro-notifications` per Plan 01, so transitive).
2. Set `WHATSAPP_ACCESS_TOKEN`, `WHATSAPP_PHONE_NUMBER_ID`, and related env vars in `.env`.
3. Call `WhatsApp::init(WhatsAppConfig::from_env(...).expect(...))` at app startup.
4. Set `WHATSAPP_ENABLED=true` in `.env` (or call `NotificationConfig::new().with_whatsapp_enabled(true)` programmatically).
5. Implement `Notification::to_whatsapp` on the relevant notification types.

A live integration test that sends a real WhatsApp message lives downstream in gestiscilo-it Phase 120 (per ROADMAP success criterion #7).

## Next Phase Readiness

Plan 06 (InApp channel adapter) can now reference:
- The transitional `Channel::InApp` arm at the marked placeholder location in `send()` — Plan 06's diff is a body replacement, not a surrounding-scaffolding edit.
- The `NotificationConfig::with_whatsapp_enabled(bool)` builder + `whatsapp_enabled: bool` field shape as the template for the analogous `with_in_app(InAppConfig)` builder + `in_app: Option<InAppConfig>` field.
- The `send_whatsapp` adapter as the template for `send_in_app`: read the gate, early-return Ok(()) when not configured, otherwise dispatch.

Plan 07 (publish.yml wave move + lib.rs sweep + integration tests) is unaffected by this plan — `WhatsAppMessage` was already re-exported from `ferro-notifications/src/lib.rs` in Plan 01.

## Threat Flags

None new. The plan's threat model anticipated all surface changes:
- T-149-W3-01 (phone-number SSRF / spoofing): mitigation delegated to `ferro-whatsapp`'s phone-validator hook (the adapter does NOT add a second validation layer per CONTEXT.md). No change in this plan.
- T-149-W3-02 (panic on uninit `WhatsApp::send`): structurally mitigated by the `whatsapp_enabled: false` default — the panic path is unreachable for default configurations. Documented in the `send_whatsapp` rustdoc and in the `whatsapp_enabled` field doc.
- T-149-W3-03 (recipient phone in info log): accepted — same trust posture as the existing `to = %to` logging in `send_mail` and `send_slack`.
- T-149-W3-04 (error chain via `#[from]`): closed by Plan 02's `Error::WhatsApp(#[from] ferro_whatsapp::Error)` and the `test_error_whatsapp_from_impl` source-chain assertion. The `?` operator in `send_whatsapp` exercises this path.

No new surface introduced beyond the threat model.

## Self-Check: PASSED

Verification commands executed:

```
$ test -f ferro-notifications/src/dispatcher.rs                                              # FOUND
$ grep -q "pub whatsapp_enabled: bool" ferro-notifications/src/dispatcher.rs                  # FOUND
$ grep -q "with_whatsapp_enabled" ferro-notifications/src/dispatcher.rs                       # FOUND
$ grep -q 'env::var("WHATSAPP_ENABLED")' ferro-notifications/src/dispatcher.rs                # FOUND
$ grep -q "Channel::WhatsApp =>" ferro-notifications/src/dispatcher.rs                        # FOUND (1 occurrence — the dispatch arm)
$ grep -q "Channel::InApp =>" ferro-notifications/src/dispatcher.rs                           # FOUND (1 occurrence — the transitional placeholder)
$ grep -q "fn send_whatsapp" ferro-notifications/src/dispatcher.rs                            # FOUND
$ grep -q "ferro_whatsapp::WhatsApp::send" ferro-notifications/src/dispatcher.rs              # FOUND
$ grep -q "use crate::channels::{MailMessage, SlackMessage, WhatsAppMessage}" ferro-notifications/src/dispatcher.rs  # FOUND
$ git log --oneline | grep fb64c3ab                                                           # FOUND (Task 1)
$ git log --oneline | grep 83368453                                                           # FOUND (Task 2)
$ cargo build -p ferro-notifications                                                          # exit 0
$ cargo fmt --all -- --check                                                                  # exit 0
$ cargo clippy --all --all-targets -- -D warnings                                             # exit 0
$ cargo test -p ferro-notifications dispatcher                                                # 20/20 pass
$ cargo test --all-features                                                                   # all suites pass (480 / 485 / 621 / 229 / 50 etc., zero failures)
```

---

*Phase: 149-ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment*
*Completed: 2026-04-28*
