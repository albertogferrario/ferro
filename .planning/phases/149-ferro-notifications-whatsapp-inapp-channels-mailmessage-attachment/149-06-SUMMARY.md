---
phase: 149-ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment
plan: 06
subsystem: notifications
tags: [ferro-notifications, in-app, sse, broadcaster, database-store, dispatcher, arch-finding-02]

requires:
  - plan: 149-02
    provides: Channel::InApp variant + Notification::to_in_app() + Error::Broadcast(String) + Error::broadcast(msg) helper + transitional placeholder match arm
  - plan: 149-05
    provides: NotificationConfig builder shape (with_whatsapp_enabled template) + Channel::InApp arm pre-split (transitional placeholder) — Plan 06 replaces only the arm body
provides:
  - InAppConfig struct ({ broker: Arc<ferro_broadcast::Broadcaster>, store: Arc<dyn DatabaseNotificationStore> })
  - NotificationConfig.in_app: Option<InAppConfig> + with_in_app() builder
  - NotificationConfig.database_store: Option<Arc<dyn DatabaseNotificationStore>> + with_database_store() builder
  - send_in_app async fn (writes DB-store leg first, broadcast leg second; either failure bubbles up)
  - send_database now calls DatabaseNotificationStore::store(...) when configured (closes ARCH-FINDING-02)
  - Channel::InApp dispatch arm wired (replaces transitional placeholder from Plan 05)
  - inapp_to_database_message helper (object data → HashMap; non-object → wrap under "payload")
  - ferro-broadcast as a workspace-internal dependency of ferro-notifications
affects:
  - 149-07 (publish.yml wave move + lib.rs sweep + integration tests)

tech-stack:
  added:
    - ferro-broadcast (workspace-internal hard dep — was already via ferro-whatsapp Wave 1b transition in Plan 01, no new wave change)
  patterns:
    - "Two-leg adapter pattern: persist (DB-store) then publish (Broadcaster), both legs share the same Arc'd handles via InAppConfig; failures abort (no partial-success silent fallback)"
    - "Manual error mapping via map_err(|e| Error::broadcast(e.to_string())) — ferro_broadcast::Error has no auto-conversion to ferro_notifications::Error (RESEARCH.md confirmed)"
    - "Conditional wiring through Option<Arc<dyn ...>>: configured path → trait call, unconfigured path → placeholder log; preserves backward-compat for the database channel"
    - "Object/non-object payload normalization: serde_json::Value::Object flattens to HashMap fields; everything else wraps under the 'payload' key (lossless round-trip)"

key-files:
  created: []
  modified:
    - ferro-notifications/Cargo.toml
    - ferro-notifications/src/dispatcher.rs

key-decisions:
  - "send_in_app reads CONFIG.in_app via match Some(c)/None pattern (early-return Ok(()) when None) rather than chained `if let`. Cleaner for the two-leg sequence below since both legs need the cfg reference."
  - "inapp_to_database_message lives at module scope (not as a private associated fn). It is pure (no &self), trivially testable, and adding it inside `impl NotificationDispatcher` would force the test module to spell out type paths it doesn't otherwise need."
  - "The 'wrap under payload' fallback for non-object data is deliberate — InAppMessage.data is `serde_json::Value` (any shape), but DatabaseMessage.data is HashMap<String, Value> (object only). The wrapping makes the round-trip lossless."
  - "test_send_database_calls_store_when_configured exercises the trait directly (not the dispatcher). Reason: CONFIG is a OnceLock global; injecting a fresh store per test would require restructuring the dispatcher signature to take &NotificationConfig — out of scope. The test verifies the wiring shape (Arc<dyn ...> + .store() invocation + AtomicUsize counter increments). End-to-end behavior is verified at the consumer level in gestiscilo-it Phase 120."

patterns-established:
  - "Two-leg adapter writes: DB-first, broadcast-second (the broker can replay on reconnect from the store; the inverse order would risk silent loss). This is the template for any future channel that has both a persisted record and a real-time fanout."
  - "OnceLock-based CONFIG access via `CONFIG.get().and_then(|c| c.<field>.as_ref())`: gates trait calls behind configured handles, falls back to placeholder log for backward-compat."

requirements-completed:
  - ROADMAP-149-04
  - ROADMAP-149-01

duration: 9m 17s
completed: 2026-04-28
---

# Phase 149 Plan 06: InApp Channel Adapter + Database Channel Fix Summary

**`Channel::InApp` dispatch is end-to-end functional: `NotificationConfig::in_app: Option<InAppConfig>` (combining `Arc<Broadcaster>` and `Arc<dyn DatabaseNotificationStore>`) gates `send_in_app`, which writes the DB-store leg first and broadcasts to `user.{id}` second — either failure bubbles up. `send_database` now routes through `DatabaseNotificationStore::store(...)` when `database_store` is configured, closing ARCH-FINDING-02 while preserving the placeholder log path for the unconfigured (backward-compat) case.**

## Performance

- **Duration:** 9m 17s
- **Started:** 2026-04-28T22:58:06Z
- **Completed:** 2026-04-28T23:07:23Z
- **Tasks:** 3
- **Files modified:** 2 (Cargo.toml + Cargo.lock; dispatcher.rs)
- **New unit tests:** 4 (1 builder + 2 inapp→db conversion + 1 store-trait counter)

## Accomplishments

- `ferro-broadcast` added as a workspace-internal dependency of `ferro-notifications` (path = "../ferro-broadcast", version = "0.2"). The crate builds clean. (Plan 07 will move ferro-notifications from publish.yml Wave 1a → Wave 1b — but this was already triggered by `ferro-whatsapp` in Plan 01, so the wave change is bundled there.)
- `InAppConfig { broker: Arc<ferro_broadcast::Broadcaster>, store: Arc<dyn DatabaseNotificationStore> }` is a public struct, exported from `ferro_notifications::dispatcher`. Both fields are `Clone` so the struct derives `Clone`. Together with `NotificationConfig::in_app: Option<InAppConfig>` and `database_store: Option<Arc<dyn DatabaseNotificationStore>>`, the surface now has 5 user-facing config fields: `mail`, `slack_webhook`, `whatsapp_enabled`, `in_app`, `database_store`.
- `with_in_app(InAppConfig)` and `with_database_store(Arc<dyn ...>)` consuming builders mirror the existing `mail()` / `slack_webhook()` / `with_whatsapp_enabled()` shapes. `from_env()` sets both new fields to `None` per D-14 — they require typed handles consumers must construct in code.
- `send_in_app` writes both legs per CONTEXT.md D-08:
  1. `cfg.store.store(notifiable_id, notifiable_type, &message.notification_type, &db_msg).await?` (DB-store leg first; the broker can replay on reconnect from the store, the inverse order would risk silent loss).
  2. `cfg.broker.broadcast(&format!("user.{notifiable_id}"), &format!("Notification.{}", message.notification_type), &message.data).await.map_err(|e| Error::broadcast(e.to_string()))?` (broadcast leg second; `ferro_broadcast::Error` has no `#[from]` impl available so the error message is captured via `Error::broadcast` helper from Plan 02).
  Either failure aborts the dispatch — no partial-success silent fallback.
- `Channel::InApp` arm in `NotificationDispatcher::send` is no longer the transitional placeholder added in Plan 05 — it now calls `notification.to_in_app()` and dispatches via `Self::send_in_app(notifiable, &in_app).await?` when `Some`.
- `send_database` (formerly placeholder log only — see ARCH-FINDING-02) now routes through `DatabaseNotificationStore::store(...)` when `CONFIG.database_store` is `Some`. The unconfigured path retains the existing placeholder log message ("placeholder — no store configured") for backward-compat. Closes ARCH-FINDING-02.
- `inapp_to_database_message(msg: &InAppMessage) -> DatabaseMessage` helper handles the type-shape mismatch between `InAppMessage.data: serde_json::Value` and `DatabaseMessage.data: HashMap<String, Value>`: object inputs flatten to the HashMap fields directly; non-object inputs wrap under the `"payload"` key.
- 24/24 dispatcher tests pass (21 pre-existing + 1 new builder test from Task 2 + 2 new conversion tests from Task 3 + 1 new store-trait counter test). Workspace builds clean under `cargo fmt --all -- --check`, `cargo clippy --all --all-targets -- -D warnings`, and `cargo test --all-features` (full suite — zero failures across all crates).

## Task Commits

Each task was committed atomically after passing fmt + clippy + dispatcher tests:

1. **Task 1: Add ferro-broadcast to ferro-notifications/Cargo.toml** — `2b5b2cd8` (feat)
2. **Task 2: Add InAppConfig + NotificationConfig fields + builders** — `82ab4ffe` (feat)
3. **Task 3: Wire send_in_app + finalize Channel::InApp arm + fix send_database** — `3c359574` (feat)

## Files Created/Modified

- `ferro-notifications/Cargo.toml` — modified. Added `ferro-broadcast = { path = "../ferro-broadcast", version = "0.2" }` to `[dependencies]`, alphabetically placed before `ferro-whatsapp`.
- `Cargo.lock` — modified. Single-line addition: `ferro-broadcast` appears in `ferro-notifications`'s dependency list.
- `ferro-notifications/src/dispatcher.rs` — modified. Six logical regions touched:
  1. Imports: `crate::channels::DatabaseMessage` and `InAppMessage` added to the module-level `use`; `crate::notifiable::DatabaseNotificationStore` and `std::sync::Arc` pulled in.
  2. `NotificationConfig`: two new fields (`in_app`, `database_store`) with full rustdoc.
  3. `InAppConfig` struct: new public type, after the existing `ResendConfig` block.
  4. `from_env()`: extended to set both new fields to `None` (typed handles per D-14).
  5. Two new builders: `with_in_app(cfg)`, `with_database_store(store)`.
  6. `send_database`: rewritten to call `store.store(...)` when configured, retaining placeholder log otherwise.
  7. `send_in_app`: new async fn after `send_whatsapp` — two-leg dispatch with proper error mapping.
  8. `inapp_to_database_message`: module-scope private helper for the InAppMessage → DatabaseMessage shape conversion.
  9. `Channel::InApp` arm in the dispatcher match: replaced the transitional placeholder body (`info!("Channel not configured")`) with a real `Self::send_in_app(...)` call.
  10. Test module: 4 new tests; the existing `test_notification_config_default` was tightened to also assert `in_app.is_none()` and `database_store.is_none()`.

## Decisions Made

- **`match Some(c)/None` over chained `if let` for CONFIG.get().and_then(...).** `send_in_app` early-returns `Ok(())` when InApp is unconfigured. Both legs need the `cfg` reference inside the same scope; a `match` keeps the early-return tight and avoids deeply nested `if let` blocks. `send_database` uses a single `if let` because only one leg is gated.
- **`inapp_to_database_message` at module scope.** The helper is pure (no `&self`), takes a single `&InAppMessage`, returns a `DatabaseMessage`. Putting it inside `impl NotificationDispatcher` would force the test module to import `Self::inapp_to_database_message` via the `super::NotificationDispatcher` path; module-scope is simpler and the function is small enough that "where does this live" is not a navigability issue.
- **`format!("user.{notifiable_id}")` (inlined arg) and `format!("Notification.{}", message.notification_type)` (positional arg).** Clippy's `uninlined_format_args` rejects the first form's `format!("user.{}", notifiable_id)` style. The second form has a `.` field access that cannot be inlined into the format string brace, so the positional form is the correct shape.
- **`test_send_database_calls_store_when_configured` exercises the trait directly, not the dispatcher.** `CONFIG` is a `OnceLock` global — injecting a fresh store per test would require restructuring `send_database`'s signature to take `&NotificationConfig` (out of scope for Phase 149). The test verifies the wiring shape: a counting `Arc<dyn DatabaseNotificationStore>` impl that increments an `AtomicUsize` on each `.store()` call, exercised through the public trait. End-to-end behavior is downstream's job (gestiscilo-it Phase 120).

## Deviations from Plan

None. Plan 06 executed exactly as written. All three tasks landed in their planned commits with no Rule 1/2/3 fixes required and no Rule 4 architectural decisions surfaced.

The plan's must-haves all hold:

| Must-have | Status |
|-----------|--------|
| Channel::InApp dispatches via send_in_app: persists to DatabaseNotificationStore first, then publishes to ferro_broadcast::Broadcaster | ✓ at `3c359574` |
| Channel::Database dispatches via send_database which calls store.store(...) when database_store is configured (closes ARCH-FINDING-02) | ✓ at `3c359574` |
| When InAppConfig is unconfigured, Channel::InApp emits a structured "channel not configured" log and returns Ok(()) | ✓ verified by `match cfg { None => { info!(...); return Ok(()); } }` |
| When database_store is unconfigured, send_database retains the existing placeholder log (backward-compat) | ✓ verified by `else { info!("placeholder — no store configured"); }` arm |
| InApp publishes to channel `format!("user.{}", notifiable_id)` with event `format!("Notification.{}", notification_type)` and the InAppMessage.data as payload | ✓ verified by lines 759-764 of dispatcher.rs |
| If either leg of InApp dispatch fails, the dispatch returns an error (no partial-success silent fallback) | ✓ verified by `?` on store.store and `.map_err(...)?` on broker.broadcast |
| ferro_broadcast::Error is mapped to Error::Broadcast(String) (no #[from] available) | ✓ verified by `.map_err(|e| Error::broadcast(e.to_string()))?` |

## Issues Encountered

Two minor cargo-format / cargo-clippy nits surfaced during verification (both auto-fixed inline within Task 2 and Task 3 respectively, before the commit):

1. **rustfmt's preferred long-`if-let` style.** rustfmt wanted `let data: HashMap<String, serde_json::Value> = if let serde_json::Value::Object(map) = &msg.data\n    {\n        ...` (brace on its own continuation line) rather than my initial assignment-`=`-on-prior-line layout. Trivial reformat. Pre-commit fix.
2. **clippy `uninlined_format_args` warning.** `format!("user.{}", notifiable_id)` triggered the lint; the fix is `format!("user.{notifiable_id}")`. The other format string in the same function (`format!("Notification.{}", message.notification_type)`) cannot be inlined because `message.notification_type` is a `.` field access, not a single identifier. Pre-commit fix.

Neither was a behavior issue. Both are documented here for the record.

## User Setup Required

None for this plan's verification — the unconfigured `in_app` and `database_store` paths preserve current behavior (placeholder logs), so the dispatcher works end-to-end against the existing test suite without any consumer wiring.

For consumers who want to use the InApp channel:

1. Construct an `Arc<ferro_broadcast::Broadcaster>` at app startup (typically the same Broadcaster handle used for the SSE / WebSocket route).
2. Implement `DatabaseNotificationStore` on a SeaORM-backed (or other persistence) struct; wrap it in `Arc::new(...)`.
3. Construct `InAppConfig { broker: ..., store: ... }`.
4. Configure: `NotificationConfig::new().with_in_app(in_app_config).with_database_store(db_store_arc)`.
5. Implement `Notification::to_in_app` on the relevant notification types.
6. (Optional) Configure a `ChannelAuthorizer` on the `Broadcaster` to enforce who can subscribe to `user.*` channels — see T-149-W4-01 in the plan's threat model.

A live integration test that exercises the SSE delivery path lives downstream in gestiscilo-it Phase 120.

For consumers who want to use the Database channel without InApp:

1. Implement `DatabaseNotificationStore` and wrap in `Arc::new(...)`.
2. Configure: `NotificationConfig::new().with_database_store(db_store_arc)`.
3. Implement `Notification::to_database` on the relevant notification types.

The unconfigured Database channel still works — it emits a placeholder log and returns Ok(()), preserving the pre-Phase-149 behavior.

## Next Phase Readiness

Plan 07 (publish.yml wave move + lib.rs sweep + integration tests) can now reference:
- `InAppConfig` — needs to be added to the `pub use dispatcher::{...}` re-export block in `ferro-notifications/src/lib.rs`.
- The publish.yml Wave 1b move was already triggered by `ferro-whatsapp` in Plan 01; `ferro-broadcast` adds a second internal dep but the wave conclusion is unchanged. Plan 07 should still verify the publish.yml is consistent.
- The integration test surface for InApp and Database channels — Plan 07 may add an in-process Broadcaster + counting DatabaseNotificationStore test that exercises the full `send_in_app` path end-to-end (currently exercised by unit tests against the helper functions only).

## Threat Flags

None new. The plan's threat model anticipated all surface changes:
- **T-149-W4-01** (Authorization Bypass — InApp publishes to `user.{id}` channel): mitigation delegated to ferro-broadcast's `ChannelAuthorizer`. The adapter constructs the channel name; ferro-broadcast enforces who can subscribe. Documented in this Summary's "User Setup Required" section.
- **T-149-W4-02** (Tampering — `Arc<dyn DatabaseNotificationStore>` is consumer-supplied): accepted. Same trust model as the existing Slack webhook URL injection.
- **T-149-W4-03** (Repudiation — InApp dispatch failure mid-leg, DB succeeded but broadcast failed): accepted. The error bubbles up; the store retains a record of the persistence side; the broker can replay on client reconnect from the store. Documented in `send_in_app` rustdoc.
- **T-149-W4-04** (Information Disclosure — info! logs include notifiable_id): accepted. Same trust as existing `send_mail` / `send_slack` recipient logging.
- **T-149-W4-05** (Information Disclosure — `inapp_to_database_message` clones the `data` payload): accepted. Caller-supplied; framework does not introspect or log the payload values (only `data = ?message.data` debug-format in the placeholder-log path of send_database).

No new surface introduced beyond the threat model.

## Self-Check: PASSED

Verification commands executed:

```
$ test -f ferro-notifications/Cargo.toml                                                       # FOUND
$ test -f ferro-notifications/src/dispatcher.rs                                                # FOUND
$ grep -q '^ferro-broadcast' ferro-notifications/Cargo.toml                                    # FOUND
$ grep -q "pub struct InAppConfig" ferro-notifications/src/dispatcher.rs                       # FOUND
$ grep -q "pub broker: Arc<ferro_broadcast::Broadcaster>" ferro-notifications/src/dispatcher.rs # FOUND
$ grep -q "pub in_app: Option<InAppConfig>" ferro-notifications/src/dispatcher.rs              # FOUND
$ grep -q "pub database_store: Option<Arc<dyn DatabaseNotificationStore>>" ferro-notifications/src/dispatcher.rs # FOUND
$ grep -q "pub fn with_in_app" ferro-notifications/src/dispatcher.rs                           # FOUND
$ grep -q "pub fn with_database_store" ferro-notifications/src/dispatcher.rs                   # FOUND
$ grep -q "fn send_in_app" ferro-notifications/src/dispatcher.rs                               # FOUND
$ grep -q "fn inapp_to_database_message" ferro-notifications/src/dispatcher.rs                 # FOUND
$ grep -q 'format!("user.{notifiable_id}")' ferro-notifications/src/dispatcher.rs              # FOUND
$ grep -q 'format!("Notification.{}"' ferro-notifications/src/dispatcher.rs                    # FOUND
$ grep -q "Error::broadcast" ferro-notifications/src/dispatcher.rs                             # FOUND
$ grep -q "if let Some(store) = CONFIG.get().and_then" ferro-notifications/src/dispatcher.rs   # FOUND
$ git log --oneline | grep 2b5b2cd8                                                            # FOUND (Task 1)
$ git log --oneline | grep 82ab4ffe                                                            # FOUND (Task 2)
$ git log --oneline | grep 3c359574                                                            # FOUND (Task 3)
$ cargo build -p ferro-notifications                                                           # exit 0
$ cargo fmt --all -- --check                                                                   # exit 0
$ cargo clippy --all --all-targets -- -D warnings                                              # exit 0
$ cargo test -p ferro-notifications dispatcher                                                 # 24/24 pass
$ cargo test --all-features                                                                    # all suites pass, zero failures
```

---

*Phase: 149-ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment*
*Completed: 2026-04-28*
